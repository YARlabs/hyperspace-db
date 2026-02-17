use arc_swap::ArcSwap;
use crate::sync::CollectionDigest;
use dashmap::DashMap;
use hyperspace_core::{Collection, FilterExpr, GlobalConfig, Metric, SearchParams, SearchResult};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::{replication_log, InsertOp, ReplicationLog};
use hyperspace_store::{wal::Wal, VectorStore};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize)]
struct CollectionState {
    id_map: HashMap<u32, u32>,
    reverse_id_map: HashMap<u32, u32>,
    buckets: Vec<u64>,
    #[serde(default)]
    last_persisted_clock: u64,
}

pub struct CollectionImpl<const N: usize, M: Metric<N>> {
    name: String,
    node_id: String,
    index_link: Arc<ArcSwap<HnswIndex<N, M>>>,
    wal: Arc<tokio::sync::Mutex<Wal>>,
    index_tx: mpsc::UnboundedSender<(u32, HashMap<String, String>)>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    config: Arc<GlobalConfig>,
    bg_tasks: Vec<JoinHandle<()>>,
    // Buckets for Merkle Tree synchronization
    buckets: Arc<Vec<AtomicU64>>,
    // Mapping from user ID to internal ID for upsert support
    id_map: Arc<DashMap<u32, u32>>,
    // Reverse mapping from internal ID to user ID for search results
    reverse_id_map: Arc<DashMap<u32, u32>>,
    // Data directory for optimization
    data_dir: PathBuf,
    // Quantization Mode
    mode: hyperspace_core::QuantizationMode,
    // Tracking latest clock for persistence/dedup
    last_clock: Arc<AtomicU64>,
}

struct BatchEntry<'a> {
    id: u32,
    vector: Cow<'a, [f64]>,
    metadata: &'a HashMap<String, String>,
    internal_id: u32,
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
        let m_env = std::env::var("HS_HNSW_M")
            .unwrap_or("16".to_string())
            .parse()
            .unwrap_or(16);

        config.set_ef_construction(ef_cons_env);
        config.set_ef_search(ef_search_env);
        config.set_m(m_env);
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

        let (_store, index, _recovered_count) = if snap_path.exists() {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            match HnswIndex::<N, M>::load_snapshot(&snap_path, store.clone(), mode, config.clone())
            {
                Ok(idx) => {
                    let count = idx.count_nodes();
                    (store, Arc::new(idx), count)
                }
                Err(e) => {
                    eprintln!("Failed to load snapshot for {name}: {e}. Starting fresh.");
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

        // Wrap index in ArcSwap for Lock-Free Hot Swap
        let index_link = Arc::new(ArcSwap::new(index.clone()));

        // Initialize state
        let state_path = data_dir.join("state.json");
        let mut id_map_data = HashMap::new();
        let mut reverse_id_map_data = HashMap::new();
        let mut buckets_data = vec![0; crate::sync::SYNC_BUCKETS];
        let last_clock = Arc::new(AtomicU64::new(0));

        if state_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&state_path) {
                if let Ok(state) = serde_json::from_str::<CollectionState>(&s) {
                    id_map_data.clone_from(&state.id_map);
                    reverse_id_map_data = state.reverse_id_map;
                    if state.buckets.len() == buckets_data.len() {
                        buckets_data = state.buckets;
                    }
                    last_clock.store(state.last_persisted_clock, Ordering::Relaxed);
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
        let wal_arc = Arc::new(tokio::sync::Mutex::new(wal));

        // Replay
        let index_ref = index.clone();
        let loaded_clock = last_clock.load(Ordering::Relaxed);

        Wal::replay(&wal_path_clone, |entry| {
            let hyperspace_store::wal::WalEntry::Insert {
                id,
                vector,
                metadata,
                logical_clock,
            } = entry;

            // Only replay operations strictly newer than what's persisted in state.json
            if logical_clock > loaded_clock {
                // If ID exists, delete old version from index to prevent leaks (Upsert)
                if let Some(&old_internal_id) = id_map_data.get(&id) {
                    let _ = index_ref.delete(old_internal_id);
                    reverse_id_map_data.remove(&old_internal_id);
                }

                if let Ok(internal_id) = index_ref.insert(&vector, metadata) {
                    id_map_data.insert(id, internal_id);
                    reverse_id_map_data.insert(internal_id, id);

                    let hash = CollectionDigest::hash_entry(id, &vector);
                    let b_idx = CollectionDigest::get_bucket_index(id);
                    buckets_data[b_idx] ^= hash;

                    // Track max clock derived from WAL
                    last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                }
            }
        })?;

        // Background Tasks
        let (index_tx, mut index_rx) = mpsc::unbounded_channel();
        let idx_link_worker = index_link.clone();
        let cfg_worker = config.clone();

        // Indexer Concurrency Configuration
        // Default: 1 (Serial) for maximum graph quality
        // Set to 0 to use all CPU cores (faster but lower recall due to race conditions)
        let num_cpus = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
        let concurrency_env = std::env::var("HS_INDEXER_CONCURRENCY")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<usize>()
            .unwrap_or(1);

        let concurrency = if concurrency_env == 0 {
            num_cpus
        } else if concurrency_env > num_cpus {
             println!("⚠️  Clamping Indexer Concurrency from {} to {} (CPU limit) to avoid thrashing.", concurrency_env, num_cpus);
             num_cpus
        } else {
            concurrency_env
        };

        println!("⚙️  Indexer Concurrency: {concurrency} thread(s)");
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

        let indexer_task = tokio::spawn(async move {
            while let Some((id, meta)) = index_rx.recv().await {
                // ... (rest of task)
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let idx_link = idx_link_worker.clone();
                let cfg = cfg_worker.clone();
                cfg.inc_active();

                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = tokio::task::spawn_blocking(move || {
                        let idx = idx_link.load().clone();
                        let _ = idx.index_node(id, meta);
                        cfg.dec_queue();
                        cfg.dec_active();
                    }).await;
                });
            }
        });

        // ...
        // (Skipping to insert_batch changes - I will use a separate block for insert_batch if needed, but the tool supports one block if contiguous.
        // Wait, insert_batch is far away (line 457). I should use `MultiReplaceFileContent` or two calls.
        // I will use `replacement_chunks`.


        let idx_link_snap = index_link.clone();
        let snap_path_clone = snap_path.clone();

        let buckets: Arc<Vec<AtomicU64>> =
            Arc::new(buckets_data.into_iter().map(AtomicU64::new).collect());
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
        let last_clock_snap = last_clock.clone();

        let snap_interval = std::env::var("HYPERSPACE_SNAPSHOT_INTERVAL_SEC")
            .unwrap_or("60".to_string())
            .parse::<u64>()
            .unwrap_or(60);

        let snapshot_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(snap_interval)).await;
                let idx = idx_link_snap.load().clone();
                if let Err(e) = idx.save_snapshot(&snap_path_clone) {
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
                let buckets_data: Vec<u64> = buckets_snap
                    .iter()
                    .map(|b| b.load(Ordering::Relaxed))
                    .collect();

                let state = CollectionState {
                    id_map: map_data,
                    reverse_id_map: reverse_map_data,
                    buckets: buckets_data,
                    last_persisted_clock: last_clock_snap.load(Ordering::Relaxed),
                };

                if let Ok(s) = serde_json::to_string(&state) {
                    let _ = std::fs::write(&state_path_snap, s);
                }
            }
        });

        Ok(Self {
            name,
            node_id,
            index_link,
            wal: wal_arc,
            index_tx,
            replication_tx,
            config,
            bg_tasks: vec![indexer_task, snapshot_handle],
            buckets,
            reverse_id_map,
            id_map,
            data_dir,
            mode,
            last_clock,
        })
    }
}

#[async_trait::async_trait]
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

    async fn insert(
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
            let old_vector = self.index_link.load().get_vector(old_internal_id);
            let old_hash = CollectionDigest::hash_entry(id, &old_vector.coords);
            let bucket_idx = CollectionDigest::get_bucket_index(id);
            self.buckets[bucket_idx].fetch_xor(old_hash, Ordering::Relaxed);
        }

        let entry_hash = CollectionDigest::hash_entry(id, processed_vector);
        let bucket_idx = CollectionDigest::get_bucket_index(id);
        self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);

        let internal_id = if let Some(old_id) = existing_internal_id {
            self.index_link
                .load()
                .update_storage(old_id, processed_vector)
                .map_err(|e| e.clone())?;
            old_id
        } else {
            let new_id = self
                .index_link
                .load()
                .insert_to_storage(processed_vector)
                .map_err(|e| e.clone())?;
            self.id_map.insert(id, new_id);
            self.reverse_id_map.insert(new_id, id);
            new_id
        };

        {
            let mut wal = self.wal.lock().await;
            // Use User ID for WAL to support replication/restore
            wal.append(id, processed_vector, &metadata, clock)
                .map_err(|e| format!("WAL Error: {e}"))?;

            self.last_clock.fetch_max(clock, Ordering::Relaxed);

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

    async fn insert_batch(
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

        // Optimization: Use lifetime to hold reference to input vectors to avoid allocation.

        let mut entries = Vec::with_capacity(vectors.len());

        // 2. Process Logic (Zero-Copy Path)
        // Note: Iterate by reference to preserve original data lifetimes.
        
        // HOISTED LOCK: Load the index pointer to avoid taking the RwLock for every item.
        // ArcSwap provides zero-contention access to the index.
        let index_reader = self.index_link.load();

        for (vector, id, metadata) in &vectors {
            // Returns Borrowed for Poincare (No Allocation)
            let processed_vector = Self::normalize_if_cosine(vector);

            // Check existing
            let existing_internal_id = self.id_map.get(id).map(|v| *v);

            // Bucket updates (Read-only access to vector)
            if let Some(old_internal_id) = existing_internal_id {
                let old_vector = index_reader.get_vector(old_internal_id);
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
                index_reader
                    .update_storage(old_id, &processed_vector)
                    .map_err(|e| e.clone())?;
                old_id
            } else {
                let new_id = index_reader
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
        // Allocate here as WAL requires owned data.
        // This is the first allocation of the vector in the Poincaré pipeline.
        let wal_data: Vec<_> = entries
            .iter()
            .map(|e| (e.vector.to_vec(), e.id, e.metadata.clone()))
            .collect();

        {
            let mut wal = self.wal.lock().await;
            wal.append_batch(&wal_data, clock)
                .map_err(|e| e.to_string())?;

            self.last_clock.fetch_max(clock, Ordering::Relaxed);

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
            let _ = self
                .index_tx
                .send((entry.internal_id, entry.metadata.clone()));
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
                        // Convert Cow to Owned for channel transmission.
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
            self.index_link.load().delete(internal_id);
        } else {
            self.index_link.load().delete(id);
            self.reverse_id_map.remove(&id);
        }
        Ok(())
    }

    async fn search(
        &self,
        query: &[f64],
        filters: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResult>, String> {
        if query.len() != N {
            return Err(format!(
                "Query dimension mismatch. Expected {}, got {}",
                N,
                query.len()
            ));
        }

        // Zero-copy normalization if possible
        // We must own the data for spawn_blocking
        let processed_query = Self::normalize_if_cosine(query).into_owned();
        
        let index_link = self.index_link.clone();
        let reverse_id_map = self.reverse_id_map.clone();
        
        // Clone arguments for move
        let params = params.clone();
        let filters = filters.clone();
        let complex_filters = complex_filters.to_vec();

        let (tx, rx) = oneshot::channel();

        rayon::spawn(move || {
            let index = index_link.load();
            let results = index.search(
                &processed_query,
                params.top_k,
                params.ef_search,
                &filters,
                &complex_filters,
                params.hybrid_query.as_deref(),
                params.hybrid_alpha,
            );

            // Fetch metadata and convert IDs inside the Rayon thread
            let mapped_results: Vec<SearchResult> = results
                .into_iter()
                .map(|(internal_id, dist)| {
                    let meta = index
                        .metadata
                        .forward
                        .get(&internal_id)
                        .map(|m| m.clone())
                        .unwrap_or_default();

                    let user_id = reverse_id_map.get(&internal_id).map_or(internal_id, |v| *v);

                    (user_id, dist, meta)
                })
                .collect();

            let _ = tx.send(mapped_results);
        });

        rx.await.map_err(|e| format!("Search task failed: {}", e))
    }

    async fn optimize(&self) -> Result<(), String> {
        println!("🧹 Starting Hot Vacuum for '{}'...", self.name);
        let start = std::time::Instant::now();
        // Removed unused name
        let data_dir = self.data_dir.clone();
        let mode = self.mode;
        let original_config = self.config.clone();
        let index_link = self.index_link.clone();

        // Run heavy lifting in blocking thread
        let (new_index_arc, temp_dir, new_snap_path) = tokio::task::spawn_blocking(move || {
            use hyperspace_core::config::GlobalConfig;
            use hyperspace_store::VectorStore;
            use std::path::PathBuf;

            // 1. Get current data
            let current_index = index_link.load().clone();
            let all_data = current_index.peek_all();
            let count = all_data.len();

            if count == 0 {
                return Ok((None, PathBuf::new(), PathBuf::new())); // Nothing to do
            }

            // 2. Setup "Turbo Mode"
            let vacuum_m = 64;
            let vacuum_ef = 500;

            let vacuum_config = Arc::new(GlobalConfig::new());
            vacuum_config.set_m(vacuum_m);
            vacuum_config.set_ef_construction(vacuum_ef);
            vacuum_config.set_ef_search(original_config.get_ef_search());

            println!(
                "   Building Shadow Index (M={}, EF={})...",
                vacuum_m, vacuum_ef
            );

            // 3. Create temp storage
            let temp_dir = data_dir.join(format!("idx_opt_{}", uuid::Uuid::new_v4()));
            if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                return Err(e.to_string());
            }

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

            let temp_store = Arc::new(VectorStore::new(&temp_dir, element_size));
            let new_index = HnswIndex::<N, M>::new(temp_store, mode, vacuum_config);

            // 4. Sequential Insertion
            // No yielding needed in blocking thread, OS handles scheduling.
            for (_i, (_old_id, vec, meta)) in all_data.iter().enumerate() {
                // Ensure insert handles internal logic
                let _ = new_index.insert(vec, meta.clone());
            }

            // Save to disk
            let new_snap_path = data_dir.join("index.snap.new");
            if let Err(e) = new_index.save_snapshot(&new_snap_path) {
                return Err(e.to_string());
            }

            Ok((Some(Arc::new(new_index)), temp_dir, new_snap_path))
        })
        .await
        .map_err(|e| e.to_string())??;

        if let Some(new_index) = new_index_arc {
            // 5. Hot Swap
            {
                println!("🔄 Swapping indexes in memory...");
                self.index_link.store(new_index);
            }

            // 6. Finalize on disk
            let snap_path = self.data_dir.join("index.snap");
            // Rename overwrites
            std::fs::rename(&new_snap_path, &snap_path).map_err(|e| e.to_string())?;
            std::fs::remove_dir_all(&temp_dir).ok();

            println!(
                "✨ Vacuum Complete in {:?}. Recall upgraded.",
                start.elapsed()
            );
        }

        Ok(())
    }

    fn count(&self) -> usize {
        self.index_link.load().count_nodes()
    }

    fn dimension(&self) -> usize {
        N
    }

    fn quantization_mode(&self) -> hyperspace_core::QuantizationMode {
        self.mode
    }

    // Updated peek to use index_link
    fn peek(&self, limit: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let items = self.index_link.load().peek(limit);
        items
            .into_iter()
            .map(|(internal_id, vec, meta)| {
                let user_id = self
                    .reverse_id_map
                    .get(&internal_id)
                    .map(|v| *v)
                    .unwrap_or(internal_id);
                (user_id, vec, meta)
            })
            .collect()
    }

    fn queue_size(&self) -> u64 {
        self.config.get_queue_size()
    }
}

impl<const N: usize, M: Metric<N>> Drop for CollectionImpl<N, M> {
    fn drop(&mut self) {
        println!(
            "🗑️ Dropping collection '{}'. Stopping background tasks...",
            self.name
        );
        // Abort background tasks (indexer, snapshot)
        for task in &self.bg_tasks {
            task.abort();
        }
    }
}
