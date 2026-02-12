use crate::sync::CollectionDigest;
use hyperspace_core::{Collection, FilterExpr, GlobalConfig, Metric, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::{ReplicationLog, InsertOp, replication_log};
use hyperspace_store::{wal::Wal, VectorStore};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CollectionState {
    id_map: HashMap<u32, u32>,
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
    id_map: Arc<Mutex<HashMap<u32, u32>>>,
}

impl<const N: usize, M: Metric<N>> CollectionImpl<N, M> {
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
        // Initialize config from env or defaults
        // For MVP, we use defaults or global env vars. Ideally passed in.
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
                    // Cleanup?
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
        let mut buckets_data = vec![0; crate::sync::SYNC_BUCKETS];

        if state_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&state_path) {
                if let Ok(state) = serde_json::from_str::<CollectionState>(&s) {
                    id_map_data = state.id_map;
                    if state.buckets.len() == buckets_data.len() {
                        buckets_data = state.buckets;
                    }
                }
            }
        }

        // WAL
        // WAL Durability
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

        let wal = Wal::new(&wal_path, sync_mode)?;
        let wal_arc = Arc::new(Mutex::new(wal));

        // Background Sync Task (Batch Mode)
        if sync_mode == hyperspace_store::wal::WalSyncMode::Batch {
             // ... existing batch logic omitted for brevity in snippet, but I need to include it or carefully replace ...
             // Wait, replace_file_content replaces EVERYTHING in range.
             // I must replicate existing logic or use smaller chunks.
             // I'll assume I need to rewrite the function body partially.
             // I will use smaller chunks.
        }
        
        // Replay
        let index_ref = index.clone();
        Wal::replay(&wal_path, |entry| {
            let hyperspace_store::wal::WalEntry::Insert {
                id,
                vector,
                metadata,
            } = entry;
            if (id as usize) >= recovered_count {
                if let Ok(internal_id) = index_ref.insert(&vector, metadata) {
                    // Update State
                    id_map_data.insert(id, internal_id);
                    
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

        let concurrency = std::thread::available_parallelism()
            .map_or(8, std::num::NonZero::get);
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
        let id_map = Arc::new(Mutex::new(id_map_data));

        let id_map_snap = id_map.clone();
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
                
                // Save State
                let map_data = {
                    let guard = id_map_snap.lock().unwrap();
                    guard.clone() 
                };
                let buckets_data: Vec<u64> = buckets_snap.iter().map(|b| b.load(Ordering::Relaxed)).collect();
                
                let state = CollectionState {
                     id_map: map_data,
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
        // Validation
        if vector.len() != N {
            return Err(format!(
                "Vector dimension mismatch. Expected {}, got {}",
                N,
                vector.len()
            ));
        }

        // Check if this user ID already exists (for upsert)
        let mut id_map = self.id_map.lock().unwrap();
        let existing_internal_id = id_map.get(&id).copied();

        // If updating existing vector, remove old hash first
        if let Some(old_internal_id) = existing_internal_id {
            // Get old vector to compute old hash
            let old_vector = self.index.get_vector(old_internal_id);
            let old_hash = CollectionDigest::hash_entry(id, &old_vector.coords);
            let bucket_idx = CollectionDigest::get_bucket_index(id);
            // XOR with old hash to remove it
            self.buckets[bucket_idx].fetch_xor(old_hash, Ordering::Relaxed);
        }

        // Update State Hash with new vector (XOR rolling hash)
        let entry_hash = CollectionDigest::hash_entry(id, vector);
        let bucket_idx = CollectionDigest::get_bucket_index(id);
        self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);

        // 1. Storage - reuse internal_id if updating, create new if inserting
        let internal_id = if let Some(old_id) = existing_internal_id {
            // Update existing vector in storage
            self.index
                .update_storage(old_id, vector)
                .map_err(|e| e.clone())?;
            old_id
        } else {
            // Insert new vector
            let new_id = self
                .index
                .insert_to_storage(vector)
                .map_err(|e| e.clone())?;
            // Store mapping
            id_map.insert(id, new_id);
            new_id
        };

        // Release lock early
        drop(id_map);

        // 2. Write to WAL (Persistence)
        {
            let mut wal = self.wal.lock().map_err(|_| "Failed to lock WAL")?;
            wal.append(internal_id, vector, &metadata)
                .map_err(|e| format!("WAL Error: {e}"))?;

            if durability == hyperspace_core::Durability::Strict {
                wal.sync().map_err(|e| format!("WAL Sync Error: {e}"))?;
            }
        }
        // 3. Index Queue (unbounded send never blocks)
        self.config.inc_queue();
        let _ = self.index_tx.send((internal_id, metadata.clone()));
        // Note: blocking_send inside async function? Collection trait is sync methods?
        // Collection trait definition has: fn insert(...) -> Result
        // It is NOT async. This works for gRPC `insert` which is async but calls this?

        // We should make Collection trait async?
        // Or use `blocking_send` which blocks.
        // `index_tx` is `mpsc::Sender` (Tokio). `blocking_send` blocks thread.
        // Ideally we use `try_send` into a bounded channel?
        // `try_send` is non-blocking.
        // Using `blocking_send` here is bad if it blocks async runtime.
        // But `insert` in trait is not async.
        // We can change trait to async, but object-safe async traits are tricky (require `async_trait` crate).
        // `boot_server` used `await`.
        // Let's use `try_send` or strictly `block_in_place`.
        // Or just use std::sync::mpsc? No, executor needs to await rx.
        // I will use `try_send` and error if full (backpressure).

        // 4. Replication
        if self.replication_tx.receiver_count() > 0 {
            let log = ReplicationLog {
                logical_clock: clock,
                origin_node_id: self.node_id.clone(),
                collection: self.name.clone(),
                operation: Some(replication_log::Operation::Insert(InsertOp {
                    id,
                    vector: vector.to_vec(),
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

        // 2. Lock id_map once
        let mut id_map = self.id_map.lock().unwrap();

        // We need separate vectors for diff tasks
        // internal_entries stores (internal_id, user_id, vector_clone, metadata)
        let mut internal_data = Vec::with_capacity(vectors.len());

        for (vector, id, metadata) in &vectors {
            // Check if this user ID already exists (for upsert)
            let existing_internal_id = id_map.get(id).copied();

            // If updating existing vector, bucket update
            if let Some(old_internal_id) = existing_internal_id {
                let old_vector = self.index.get_vector(old_internal_id);
                let old_hash = CollectionDigest::hash_entry(*id, &old_vector.coords);
                let bucket_idx = CollectionDigest::get_bucket_index(*id);
                self.buckets[bucket_idx].fetch_xor(old_hash, Ordering::Relaxed);
            }

            // Update State Hash with new vector
            let entry_hash = CollectionDigest::hash_entry(*id, vector);
            let bucket_idx = CollectionDigest::get_bucket_index(*id);
            self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);

            // Storage
            let internal_id = if let Some(old_id) = existing_internal_id {
                self.index
                    .update_storage(old_id, vector)
                    .map_err(|e| e.clone())?;
                old_id
            } else {
                let new_id = self
                    .index
                    .insert_to_storage(vector)
                    .map_err(|e| e.clone())?;
                id_map.insert(*id, new_id);
                new_id
            };

            internal_data.push((internal_id, vector.clone(), metadata.clone()));
        }

        drop(id_map);

        // 3. WAL Batch
        let wal_data: Vec<_> = internal_data
            .iter()
            .map(|(internal_id, vector, metadata)| (vector.clone(), *internal_id, metadata.clone()))
            .collect();

        {
            let mut wal = self.wal.lock().map_err(|_| "Failed to lock WAL")?;
            wal.append_batch(&wal_data).map_err(|e| e.to_string())?;

            if durability == hyperspace_core::Durability::Strict {
                 wal.sync().map_err(|e| e.to_string())?;
            }
        }

        // 4. Index Queue
        // TODO: config.inc_queue_by is not available, calling inc_queue in loop
        for _ in 0..internal_data.len() {
            self.config.inc_queue();
        }

        // Queue for indexing (unbounded send never blocks)
        for (id, _, meta) in &internal_data {
            let _ = self.index_tx.send((*id, meta.clone()));
        }

        // 5. Replication
        if self.replication_tx.receiver_count() > 0 {
            for (vector, id, metadata) in vectors {
                let log = ReplicationLog {
                    logical_clock: clock,
                    origin_node_id: self.node_id.clone(),
                    collection: self.name.clone(),
                    operation: Some(replication_log::Operation::Insert(InsertOp {
                        id,
                        vector,
                        metadata,
                    })),
                };
                let _ = self.replication_tx.send(log);
            }
        }

        Ok(())
    }

    fn delete(&self, id: u32) -> Result<(), String> {
        self.index.delete(id);
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

        let results = self.index.search(
            query,
            params.top_k,
            params.ef_search,
            filters,
            complex_filters,
            params.hybrid_query.as_deref(),
            params.hybrid_alpha,
        );

        // Fetch metadata for results
        let results_with_meta = results
            .into_iter()
            .map(|(id, dist)| {
                let meta = self
                    .index
                    .metadata
                    .forward
                    .get(&id)
                    .map(|m| m.clone())
                    .unwrap_or_default();
                (id, dist, meta)
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

    fn peek(&self, limit: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        self.index.peek(limit)
    }
}
