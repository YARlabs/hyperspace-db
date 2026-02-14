use crate::sync::CollectionDigest;
use dashmap::DashMap;
use hyperspace_core::{Collection, FilterExpr, GlobalConfig, Metric, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::{InsertOp, ReplicationLog, replication_log};
use hyperspace_store::{wal::Wal, VectorStore};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize)]
struct CollectionState {
    id_map: HashMap<u32, u32>,
    reverse_id_map: HashMap<u32, u32>,
    buckets: Vec<u64>,
}

pub struct CollectionImpl<const N: usize, M: Metric<N>> {
    name: String,
    node_id: String,
    index: Arc<HnswIndex<N, M>>,
    wal: Arc<Mutex<Wal>>,
    index_tx: mpsc::UnboundedSender<(u32, HashMap<String, String>)>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    config: Arc<GlobalConfig>,
    _tasks: Vec<JoinHandle<()>>,
    // Buckets for Merkle Tree
    buckets: Arc<Vec<AtomicU64>>,
    // Mapping from user ID to internal ID for upsert support
    id_map: Arc<DashMap<u32, u32>>,
    // Reverse mapping from internal ID to user ID for search results
    reverse_id_map: Arc<DashMap<u32, u32>>,
}

impl<const N: usize, M: Metric<N>> CollectionImpl<N, M> {
    /// Normalizes vector if metric is Cosine.
    /// Returns Cow to avoid allocation if normalization is not needed.
    #[inline]
    fn normalize_if_cosine(vector: &[f64]) -> Cow<'_, [f64]> {
        if M::name() != "cosine" {
            return Cow::Borrowed(vector);
        }

        let norm_sq: f64 = vector.iter().map(|x| x * x).sum();
        // If already unit length (within epsilon) or zero, return as is to save allocation
        if (norm_sq - 1.0).abs() < 1e-9 || norm_sq <= 1e-18 {
            return Cow::Borrowed(vector);
        }

        let inv_norm = 1.0 / norm_sq.sqrt();
        let normalized: Vec<f64> = vector.iter().map(|x| x * inv_norm).collect();
        Cow::Owned(normalized)
    }

    pub async fn new(
        name: String,
        node_id: String,
        data_dir: std::path::PathBuf,
        wal_path: std::path::PathBuf,
        mode: hyperspace_core::QuantizationMode,
        replication_tx: broadcast::Sender<ReplicationLog>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let snap_path = data_dir.join("index.snap");
        let config = Arc::new(GlobalConfig::new());
        
        let ef_cons_env = std::env::var("HS_HNSW_EF_CONSTRUCT")
            .unwrap_or("100".to_string())
            .parse()
            .unwrap_or(100);
        let ef_search_env = std::env::var("HS_HNSW_EF_SEARCH")
            .unwrap_or("10".to_string())
            .parse()
            .unwrap_or(10);
        config.set_ef_construction(ef_cons_env);
        config.set_ef_search(ef_search_env);

        let element_size = match mode {
            hyperspace_core::QuantizationMode::ScalarI8 => {
                hyperspace_core::vector::QuantizedHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::Binary => {
                hyperspace_core::vector::BinaryHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::None => {
                hyperspace_core::vector::HyperVector::<N>::SIZE
            }
        };

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let (_store, index, recovered_count) = if snap_path.exists() {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            match HnswIndex::<N, M>::load_snapshot(&snap_path, store.clone(), mode, config.clone())
            {
                Ok(idx) => {
                    let count = idx.count_nodes();
                    (store, Arc::new(idx), count)
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load snapshot for {name}: {e}. Starting fresh."
                    );
                    let store = Arc::new(VectorStore::new(&data_dir, element_size));
                    (
                        store.clone(),
                        Arc::new(HnswIndex::new(store, mode, config.clone())),
                        0,
                    )
                }
            }
        } else {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            (
                store.clone(),
                Arc::new(HnswIndex::new(store, mode, config.clone())),
                0,
            )
        };

        // Initialize state
        let state_path = data_dir.join("state.json");
        let mut id_map_data = HashMap::new();
        let mut reverse_id_map_data = HashMap::new();
        let mut buckets_data = vec![0; crate::sync::SYNC_BUCKETS];

        if state_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&state_path) {
                if let Ok(state) = serde_json::from_str::<CollectionState>(&s) {
                    id_map_data = state.id_map.clone();
                    reverse_id_map_data = state.reverse_id_map;
                    if state.buckets.len() == buckets_data.len() {
                        buckets_data = state.buckets;
                    }
                }
            }
        }

        // WAL
        let sync_mode_str = std::env::var("HYPERSPACE_WAL_SYNC_MODE")
            .unwrap_or_else(|_| "async".to_string())
            .to_lowercase();
        
        let sync_mode = match sync_mode_str.as_str() {
            "strict" | "fsync" => hyperspace_store::wal::WalSyncMode::Strict,
            "batch" => hyperspace_store::wal::WalSyncMode::Batch,
            _ => hyperspace_store::wal::WalSyncMode::Async,
        };

        if sync_mode == hyperspace_store::wal::WalSyncMode::Strict {
            println!("🔒 WAL Durability: STRICT (fsync on every write)");
        } else if sync_mode == hyperspace_store::wal::WalSyncMode::Batch {
            println!("🔒 WAL Durability: BATCH (Background fsync every 100ms)");
        }

        let wal_path_clone = wal_path.clone();
        let wal = Wal::new(&wal_path, sync_mode)?;
        let wal_arc = Arc::new(Mutex::new(wal));

        // Replay
        let index_ref = index.clone();
        Wal::replay(&wal_path_clone, |entry| {
            let hyperspace_store::wal::WalEntry::Insert {
                id,
                vector,
                metadata,
            } = entry;
            if (id as usize) >= recovered_count {
                if let Ok(internal_id) = index_ref.insert(&vector, metadata) {
                    id_map_data.insert(id, internal_id);
                    reverse_id_map_data.insert(internal_id, id);
                    
                    let hash = CollectionDigest::hash_entry(id, &vector);
                    let b_idx = CollectionDigest::get_bucket_index(id);
                    buckets_data[b_idx] ^= hash;
                }
            }
        })?;

        // Background Tasks
        let (index_tx, mut index_rx) = mpsc::unbounded_channel();
        let idx_worker = index.clone();
        let cfg_worker = config.clone();


        // Indexer Concurrency Configuration
        // Default: 1 (Serial) for maximum graph quality
        // Set to 0 to use all CPU cores (faster but lower recall due to race conditions)
        let concurrency_env = std::env::var("HS_INDEXER_CONCURRENCY")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<usize>()
            .unwrap_or(1);
            
        let concurrency = if concurrency_env == 0 { 
            std::thread::available_parallelism().map_or(8, std::num::NonZero::get) 
        } else { 
            concurrency_env 
        };
        
        println!("⚙️  Indexer Concurrency: {} thread(s)", concurrency);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));


        let indexer_handle = tokio::spawn(async move {
            while let Some((id, meta)) = index_rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let idx = idx_worker.clone();
                let cfg = cfg_worker.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = tokio::task::spawn_blocking(move || {
                        let _ = idx.index_node(id, meta);
                        cfg.dec_queue();
                    })
                    .await;
                });
            }
        });

        let idx_snap = index.clone();
        let snap_path_clone = snap_path.clone();
        
        let buckets: Arc<Vec<AtomicU64>> = Arc::new(buckets_data.into_iter().map(AtomicU64::new).collect());
        let id_map = Arc::new(id_map_data.into_iter().collect::<DashMap<u32, u32>>());
        let reverse_id_map = Arc::new(
            reverse_id_map_data
                .into_iter()
                .collect::<DashMap<u32, u32>>(),
        );

        let id_map_snap = id_map.clone();
        let reverse_id_map_snap = reverse_id_map.clone();
        let buckets_snap = buckets.clone();
        let state_path_snap = data_dir.join("state.json");

        let snap_interval = std::env::var("HYPERSPACE_SNAPSHOT_INTERVAL_SEC")
             .unwrap_or("60".to_string())
             .parse::<u64>()
             .unwrap_or(60);

        let snapshot_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(snap_interval)).await;
                if let Err(e) = idx_snap.save_snapshot(&snap_path_clone) {
                    eprintln!("Snapshot error: {e}");
                }
                
                // Save State (DashMap iteration)
                let map_data: HashMap<u32, u32> = id_map_snap
                    .iter()
                    .map(|entry| (*entry.key(), *entry.value()))
                    .collect();
                let reverse_map_data: HashMap<u32, u32> = reverse_id_map_snap
                    .iter()
                    .map(|entry| (*entry.key(), *entry.value()))
                    .collect();
                let buckets_data: Vec<u64> = buckets_snap.iter().map(|b| b.load(Ordering::Relaxed)).collect();
                
                let state = CollectionState {
                     id_map: map_data,
                     reverse_id_map: reverse_map_data,
                     buckets: buckets_data,
                };
                
                if let Ok(s) = serde_json::to_string(&state) {
                     let _ = std::fs::write(&state_path_snap, s);
                }
            }
        });

        Ok(Self {
            name,
            node_id,
            index,
            wal: wal_arc,
            index_tx,
            replication_tx,
            config,
            _tasks: vec![indexer_handle, snapshot_handle],
            buckets,
            reverse_id_map,
            id_map,
        })
    }
}

impl<const N: usize, M: Metric<N>> Collection for CollectionImpl<N, M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn metric_name(&self) -> &'static str {
        M::name()
    }

    fn state_hash(&self) -> u64 {
        let mut root = 0;
        for b in self.buckets.iter() {
            root ^= b.load(Ordering::Relaxed);
        }
        root
    }

    fn buckets(&self) -> Vec<u64> {
        self.buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }

    fn insert(
        &self,
        vector: &[f64],
        id: u32,
        metadata: HashMap<String, String>,
        clock: u64,
        durability: hyperspace_core::Durability,
    ) -> Result<(), String> {
        if vector.len() != N {
            return Err(format!(
                "Vector dimension mismatch. Expected {}, got {}",
                N,
                vector.len()
            ));
        }
        
        let processed_vector_cow = Self::normalize_if_cosine(vector);
        // We need a slice for ops, and maybe an owned vec for storage if new
        let processed_vector = &processed_vector_cow;

        // Check if this user ID already exists (for upsert)
        let existing_internal_id = self.id_map.get(&id).map(|v| *v);

        if let Some(old_internal_id) = existing_internal_id {
            let old_vector = self.index.get_vector(old_internal_id);
            let old_hash = CollectionDigest::hash_entry(id, &old_vector.coords);
            let bucket_idx = CollectionDigest::get_bucket_index(id);
            self.buckets[bucket_idx].fetch_xor(old_hash, Ordering::Relaxed);
        }

        let entry_hash = CollectionDigest::hash_entry(id, processed_vector);
        let bucket_idx = CollectionDigest::get_bucket_index(id);
        self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);

        let internal_id = if let Some(old_id) = existing_internal_id {
            self.index
                .update_storage(old_id, processed_vector)
                .map_err(|e| e.clone())?;
            old_id
        } else {
            let new_id = self
                .index
                .insert_to_storage(processed_vector)
                .map_err(|e| e.clone())?;
            self.id_map.insert(id, new_id);
            self.reverse_id_map.insert(new_id, id);
            new_id
        };

        {
            let mut wal = self.wal.lock().map_err(|_| "Failed to lock WAL")?;
            wal.append(internal_id, processed_vector, &metadata)
                .map_err(|e| format!("WAL Error: {e}"))?;

            if durability == hyperspace_core::Durability::Strict {
                wal.sync().map_err(|e| format!("WAL Sync Error: {e}"))?;
            }
        }
        
        self.config.inc_queue();
        let _ = self.index_tx.send((internal_id, metadata.clone()));

        if self.replication_tx.receiver_count() > 0 {
            // Need owned vector for replication
            let vector_owned = processed_vector_cow.into_owned();
            let log = ReplicationLog {
                logical_clock: clock,
                origin_node_id: self.node_id.clone(),
                collection: self.name.clone(),
                operation: Some(replication_log::Operation::Insert(InsertOp {
                    id,
                    vector: vector_owned,
                    metadata,
                })),
            };
            let _ = self.replication_tx.send(log);
        }

        Ok(())
    }

    fn insert_batch(
        &self,
        vectors: Vec<(Vec<f64>, u32, HashMap<String, String>)>,
        clock: u64,
        durability: hyperspace_core::Durability,
    ) -> Result<(), String> {
        // 1. Validation
        for (vec, _, _) in &vectors {
            if vec.len() != N {
                return Err(format!(
                    "Vector dimension mismatch. Expected {}, got {}",
                    N,
                    vec.len()
                ));
            }
        }

        // OPTIMIZATION: Use lifetime to hold reference to input vectors.
        // This avoids allocation #1 if normalization is not needed (Poincare).
        struct BatchEntry<'a> {
            id: u32,
            vector: Cow<'a, [f64]>,        // <--- CHANGED: Cow instead of Vec<f64>
            metadata: &'a HashMap<String, String>, // <--- CHANGED: Reference instead of Clone
            internal_id: u32,
        }

        let mut entries = Vec::with_capacity(vectors.len());

        // 2. Process Logic (Zero-Copy Path)
        // Note: We iterate by reference (&vectors) to keep the original data alive
        for (vector, id, metadata) in &vectors {
            // Returns Borrowed for Poincare (No Allocation)
            let processed_vector = Self::normalize_if_cosine(vector);
            
            // Check existing
            let existing_internal_id = self.id_map.get(id).map(|v| *v);

            // Bucket updates (Read-only access to vector)
            if let Some(old_internal_id) = existing_internal_id {
                let old_vector = self.index.get_vector(old_internal_id);
                let old_hash = CollectionDigest::hash_entry(*id, &old_vector.coords);
                let bucket_idx = CollectionDigest::get_bucket_index(*id);
                self.buckets[bucket_idx].fetch_xor(old_hash, Ordering::Relaxed);
            }

            let entry_hash = CollectionDigest::hash_entry(*id, &processed_vector);
            let bucket_idx = CollectionDigest::get_bucket_index(*id);
            self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);

            // Storage
            // insert_to_storage writes bytes to Mmap. It copies bytes, but doesn't heap allocate vector objects.
            let internal_id = if let Some(old_id) = existing_internal_id {
                self.index
                    .update_storage(old_id, &processed_vector)
                    .map_err(|e| e.clone())?;
                old_id
            } else {
                let new_id = self
                    .index
                    .insert_to_storage(&processed_vector)
                    .map_err(|e| e.clone())?;
                
                self.id_map.insert(*id, new_id);
                self.reverse_id_map.insert(new_id, *id);
                new_id
            };

            entries.push(BatchEntry {
                id: *id,
                vector: processed_vector, // Moves the Cow (cheap pointer copy), not data
                metadata,                 // Reference
                internal_id,
            });
        }

        // 3. WAL Batch
        // Only NOW we allocate. WAL requires owned data.
        // This is the FIRST and ONLY allocation of the vector in the pipeline for Poincare.
        let wal_data: Vec<_> = entries
            .iter()
            .map(|e| (e.vector.to_vec(), e.internal_id, e.metadata.clone()))
            .collect();

        {
            let mut wal = self.wal.lock().map_err(|_| "Failed to lock WAL")?;
            wal.append_batch(&wal_data).map_err(|e| e.to_string())?;

            if durability == hyperspace_core::Durability::Strict {
                 wal.sync().map_err(|e| e.to_string())?;
            }
        }

        // 4. Index Queue
        for _ in 0..entries.len() {
            self.config.inc_queue();
        }

        // Queue for indexing (Send only lightweight metadata clone + internal_id)
        for entry in &entries {
            let _ = self.index_tx.send((entry.internal_id, entry.metadata.clone()));
        }

        // 5. Replication
        if self.replication_tx.receiver_count() > 0 {
            for entry in entries {
                let log = ReplicationLog {
                    logical_clock: clock,
                    origin_node_id: self.node_id.clone(),
                    collection: self.name.clone(),
                    operation: Some(replication_log::Operation::Insert(InsertOp {
                        id: entry.id,
                        // Convert Cow to Owned. If we already cloned for WAL, this is unfortunate 
                        // but necessary as channels need ownership. 
                        // Since WAL path above didn't consume `entries`, we still have the Cows.
                        // .into_owned() performs a clone if it was borrowed.
                        vector: entry.vector.into_owned(), 
                        metadata: entry.metadata.clone(),
                    })),
                };
                let _ = self.replication_tx.send(log);
            }
        }

        Ok(())
    }

    fn delete(&self, id: u32) -> Result<(), String> {
        if let Some((_, internal_id)) = self.id_map.remove(&id) {
            self.reverse_id_map.remove(&internal_id);
            self.index.delete(internal_id);
        } else {
            self.index.delete(id);
            self.reverse_id_map.remove(&id);
        }
        Ok(())
    }

    fn search(
        &self,
        query: &[f64],
        filters: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<(u32, f64, HashMap<String, String>)>, String> {
        if query.len() != N {
            return Err(format!(
                "Query dimension mismatch. Expected {}, got {}",
                N,
                query.len()
            ));
        }

        // Zero-copy normalization if possible
        let processed_query = Self::normalize_if_cosine(query);
        
        let results = self.index.search(
            &processed_query,
            params.top_k,
            params.ef_search,
            filters,
            complex_filters,
            params.hybrid_query.as_deref(),
            params.hybrid_alpha,
        );

        // Fetch metadata and convert IDs
        // DashMap allows concurrent reading without locking the whole map
        let results_with_meta = results
            .into_iter()
            .map(|(internal_id, dist)| {
                let meta = self
                    .index
                    .metadata
                    .forward
                    .get(&internal_id)
                    .map(|m| m.clone())
                    .unwrap_or_default();
                
                let user_id = self
                    .reverse_id_map
                    .get(&internal_id)
                    .map(|v| *v)
                    .unwrap_or(internal_id);
                
                (user_id, dist, meta)
            })
            .collect();

        Ok(results_with_meta)
    }

    fn count(&self) -> usize {
        self.index.count_nodes()
    }

    fn dimension(&self) -> usize {
        N
    }

    fn queue_size(&self) -> u64 {
        self.config.get_queue_size()
    }

    fn peek(&self, limit: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        self.index.peek(limit)
    }
}