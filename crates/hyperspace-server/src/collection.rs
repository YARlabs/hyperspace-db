use crate::chunk_searcher;
use crate::meta_router::{CentroidAccumulator, ChunkMeta, MetaRouter};
use crate::sync::CollectionDigest;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use hyperspace_core::gpu::{rerank_topk_exact, GpuMetric};
use hyperspace_core::{
    Collection, FilterExpr, GlobalConfig, Metric, SearchParams, SearchResult, StorageMode,
    VacuumFilterOp, VacuumFilterQuery,
};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::{replication_log, InsertOp, ReplicationLog};
use hyperspace_store::{wal::Wal, VectorStore};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use tokio::sync::{broadcast, mpsc, Semaphore};
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
    wal_link: Arc<ArcSwap<tokio::sync::Mutex<Wal>>>,
    index_tx: mpsc::UnboundedSender<(u32, HashMap<String, String>)>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    config: Arc<GlobalConfig>,
    bg_tasks: Vec<JoinHandle<()>>,
    // Buckets for Merkle Tree synchronization
    buckets: Arc<Vec<AtomicU64>>,
    // Root hash for fast O(1) state comparison (incremental XOR)
    root_hash: AtomicU64,
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
    // True while user IDs are guaranteed to match internal IDs.
    ids_are_identity: AtomicBool,
    // Limit CPU-bound search tasks to avoid scheduler thrashing.
    search_limiter: Arc<Semaphore>,
    // Restrict background WAL rotation flush workers to 1 to prevent CPU starvation
    flush_limiter: Arc<Semaphore>,
    // If existing vector shift is <= threshold and metadata unchanged, skip graph relinking.
    fast_upsert_delta: f64,
    // Global Meta-Router for IVF-style chunk routing (Task 1.2)
    meta_router: Arc<MetaRouter<N>>,
    // Count of vectors currently in the "Flush Purgatory" (Frozen WAL -> Chunk conversion)
    flushing_vector_count: Arc<AtomicUsize>,
    // Count of vectors in the current ACTIVE WAL
    wal_pending_count: Arc<AtomicU64>,
    // Storage Mode: Performance vs Tiered
    storage_mode: StorageMode,
    // Max RAM allowed for MemTable before forcing a flush
    max_ram_bytes: u64,
    // List of rotated WAL segments waiting to be flushed into a chunk (Task 8.1)
    pending_wal_flushes: Arc<tokio::sync::Mutex<Vec<PathBuf>>>,
}

static EMPTY_LEGACY_FILTERS: LazyLock<HashMap<String, String>> = LazyLock::new(HashMap::new);
static EMPTY_COMPLEX_FILTERS: LazyLock<Vec<FilterExpr>> = LazyLock::new(Vec::new);

struct BatchEntry<'a> {
    id: u32,
    vector: Cow<'a, [f64]>,
    metadata: &'a HashMap<String, String>,
    internal_id: u32,
    reindex_needed: bool,
}

impl<const N: usize, M: Metric<N>> CollectionImpl<N, M> {
    #[inline]
    fn shift_l2_sq(a: &[f64; N], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum()
    }

    #[inline]
    fn to_internal_id(&self, user_id: u32) -> u32 {
        // Quick Win #3: Bypass DashMap lookup when identity mode is enabled
        if self.ids_are_identity.load(Ordering::Relaxed) {
            return user_id;
        }
        self.id_map.get(&user_id).map_or(user_id, |v| *v)
    }

    #[inline]
    fn to_user_id(&self, internal_id: u32) -> u32 {
        // Quick Win #3: Bypass DashMap lookup when identity mode is enabled
        if self.ids_are_identity.load(Ordering::Relaxed) {
            return internal_id;
        }
        self.reverse_id_map
            .get(&internal_id)
            .map_or(internal_id, |v| *v)
    }

    fn meta_numeric_value(meta: &HashMap<String, String>, key: &str) -> Option<f64> {
        if let Some(raw) = meta.get(key) {
            return raw.parse::<f64>().ok();
        }
        let typed_key = format!("__hs_typed__{key}");
        let raw_typed = meta.get(&typed_key)?;
        let parsed = serde_json::from_str::<serde_json::Value>(raw_typed).ok()?;
        parsed.get("v")?.as_f64()
    }

    fn matches_vacuum_filter(meta: &HashMap<String, String>, filter: &VacuumFilterQuery) -> bool {
        let Some(current) = Self::meta_numeric_value(meta, &filter.key) else {
            return false;
        };
        match filter.op {
            VacuumFilterOp::Lt => current < filter.value,
            VacuumFilterOp::Lte => current <= filter.value,
            VacuumFilterOp::Gt => current > filter.value,
            VacuumFilterOp::Gte => current >= filter.value,
            VacuumFilterOp::Eq => (current - filter.value).abs() <= 1e-12,
            VacuumFilterOp::Ne => (current - filter.value).abs() > 1e-12,
        }
    }

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

        let gossip_env =
            std::env::var("HS_GOSSIP_ENABLED").is_ok_and(|v| v.to_lowercase() == "true");
        let anisotropic_env = std::env::var("HS_ANISOTROPIC_REFINEMENT")
            .map_or(true, |v| v.to_lowercase() != "false");

        config.set_gossip_enabled(gossip_env);
        config.set_anisotropic_enabled(anisotropic_env);

        let ef_cons_env = std::env::var("HS_HNSW_EF_CONSTRUCT")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);
        let ef_search_env = std::env::var("HS_HNSW_EF_SEARCH")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let m_env = std::env::var("HS_HNSW_M")
            .unwrap_or_else(|_| "16".to_string())
            .parse()
            .unwrap_or(16);

        config.set_ef_construction(ef_cons_env);
        config.set_ef_search(ef_search_env);
        config.set_m(m_env);

        let storage_f32_requested = std::env::var("HS_STORAGE_FLOAT32")
            .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"));
        let storage_f32 = storage_f32_requested && mode == hyperspace_core::QuantizationMode::None;

        let mut element_size = match mode {
            hyperspace_core::QuantizationMode::ScalarI8 => {
                hyperspace_core::vector::QuantizedHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::Binary => {
                hyperspace_core::vector::BinaryHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::None => {
                if storage_f32 {
                    hyperspace_core::vector::HyperVectorF32::<N>::SIZE
                } else {
                    hyperspace_core::vector::HyperVector::<N>::SIZE
                }
            }
        };

        if std::env::var("HS_ZONAL_QUANTIZATION").is_ok_and(|v| v.to_lowercase() == "true") {
            element_size = 0;
        }

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let (_store, index, _recovered_count) = if snap_path.exists() {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            match HnswIndex::<N, M>::load_snapshot_with_storage_precision(
                &snap_path,
                store.clone(),
                mode,
                config.clone(),
                storage_f32,
            ) {
                Ok(idx) => {
                    let count = idx.count_nodes();
                    (store, Arc::new(idx), count)
                }
                Err(e) => {
                    eprintln!("Failed to load snapshot for {name}: {e}. Starting fresh.");
                    let store = Arc::new(VectorStore::new(&data_dir, element_size));
                    (
                        store.clone(),
                        Arc::new(HnswIndex::new_with_storage_precision(
                            store,
                            mode,
                            config.clone(),
                            storage_f32,
                        )),
                        0,
                    )
                }
            }
        } else {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            (
                store.clone(),
                Arc::new(HnswIndex::new_with_storage_precision(
                    store,
                    mode,
                    config.clone(),
                    storage_f32,
                )),
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

        // Storage & Performance Tuning
        let hs_mode_env = std::env::var("HS_MODE")
            .unwrap_or_else(|_| "tiered".to_string()) // FIX: Default to Tiered (LSM) mode
            .to_lowercase();
        #[allow(clippy::match_same_arms)] // Intentional: explicit tiered/lsm aliases
        let storage_mode = match hs_mode_env.as_str() {
            "tiered" | "lsm" => StorageMode::Tiered,
            "performance" => {
                println!("🚀 Performance Mode: WAL will NOT flush until RAM limit hit.");
                StorageMode::Performance
            }
            _ => StorageMode::Tiered,
        };

        // Determine Memory Budget
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let total_ram_bytes = sys.total_memory();
        let max_ram_gb_env = std::env::var("HS_MAX_RAM_GB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok());

        let max_ram_bytes = match max_ram_gb_env {
            Some(gb) => gb * 1024 * 1024 * 1024,
            None => total_ram_bytes * 70 / 100, // Default to 70% of total RAM
        };

        println!(
            "🚀 Collection Mode: {:?} (Budget: {} GB)",
            storage_mode,
            max_ram_bytes / (1024 * 1024 * 1024)
        );

        let mut wal = Wal::new(&wal_path, sync_mode)?;

        // WAL Segment Configuration
        let default_segment_mb = match storage_mode {
            StorageMode::Performance => 4096, // 4 GB
            StorageMode::Tiered => 256,       // 256 MB
        };

        let wal_segment_mb = std::env::var("HS_WAL_SEGMENT_SIZE_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(default_segment_mb)
            .clamp(16, 16384); // 16 MB .. 16 GB

        wal.set_size_limit(wal_segment_mb * 1024 * 1024);
        println!("📦 WAL Segment Size: {wal_segment_mb} MB");

        let wal_link = Arc::new(ArcSwap::new(Arc::new(tokio::sync::Mutex::new(wal))));
        let flushing_vector_count = Arc::new(AtomicUsize::new(0));
        let wal_pending_count = Arc::new(AtomicU64::new(0));

        // Replay
        let index_ref = index.clone();
        let loaded_clock = last_clock.load(Ordering::Relaxed);

        // Find all frozen WAL segments that haven't been flushed yet
        let mut wal_segments = Vec::new();
        if let Some(parent) = wal_path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name.contains(".frozen.") {
                        // Extract timestamp (e.g. "frozen.171234567890")
                        if let Some(ts_str) = file_name.rsplit('.').next() {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                wal_segments.push((ts, path));
                            }
                        }
                    }
                }
            }
        }

        // Sort segments chronologically
        wal_segments.sort_by_key(|(ts, _)| *ts);

        // Final list: all frozen segments + the active WAL path
        let replay_queue: Vec<_> = wal_segments.clone().into_iter().map(|(_, p)| p).collect();
        let pending_wal_flushes = Arc::new(tokio::sync::Mutex::new(
            wal_segments.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
        ));

        // Add the active path to replay, but it's not "frozen" yet
        let mut final_replay = replay_queue;
        final_replay.push(wal_path.clone());

        println!("⚡ Replaying {} WAL segment(s)...", final_replay.len());

        for path in final_replay {
            Wal::replay(&path, |entry| {
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
                        index_ref.delete(old_internal_id);
                        reverse_id_map_data.remove(&old_internal_id);
                    }

                    if let Ok(internal_id) = index_ref.insert(&vector, metadata) {
                        id_map_data.insert(id, internal_id);
                        reverse_id_map_data.insert(internal_id, id);

                        if gossip_env {
                            let hash = CollectionDigest::hash_entry(id, &vector);
                            let b_idx = CollectionDigest::get_bucket_index(id);
                            buckets_data[b_idx] ^= hash;
                        }

                        // Track max clock derived from WAL
                        last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                        wal_pending_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            })?;
        }

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
            println!(
                 "⚠️  Clamping Indexer Concurrency from {concurrency_env} to {num_cpus} (CPU limit) to avoid thrashing."
             );
            num_cpus
        } else {
            concurrency_env
        };

        println!("⚙️  Indexer Concurrency: {concurrency} thread(s)");
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

        let search_concurrency_env = std::env::var("HS_SEARCH_CONCURRENCY")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<usize>()
            .unwrap_or(0);
        // Quick Win #4: Auto-calculate search concurrency based on CPU count
        // Default: num_cpus * 2 for better throughput, with manual override via env var
        let search_concurrency = if search_concurrency_env == 0 {
            num_cpus * 2 // Auto: 2x CPU count for better parallelism
        } else if search_concurrency_env > num_cpus * 4 {
            num_cpus * 4 // Cap at 4x to avoid thrashing
        } else {
            search_concurrency_env
        };
        println!(
            "⚙️  Search Concurrency Limit: {search_concurrency} task(s) (CPU cores: {num_cpus})"
        );
        let search_limiter = Arc::new(Semaphore::new(search_concurrency));
        let flush_limiter = Arc::new(Semaphore::new(1));
        let fast_upsert_delta = std::env::var("HS_FAST_UPSERT_DELTA")
            .unwrap_or_else(|_| "0.0".to_string())
            .parse::<f64>()
            .unwrap_or(0.0)
            .max(0.0);

        let indexer_task = tokio::spawn(async move {
            use std::sync::atomic::AtomicU64;
            let received = Arc::new(AtomicU64::new(0));
            let errors = Arc::new(AtomicU64::new(0));

            while let Some((id, meta)) = index_rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let idx_link = idx_link_worker.clone();
                let cfg = cfg_worker.clone();
                let errors_ref = errors.clone();
                cfg.inc_active();

                tokio::spawn(async move {
                    let _permit = permit;
                    let result = tokio::task::spawn_blocking(move || {
                        let idx = idx_link.load().clone();
                        let result = idx.index_node(id, meta);
                        (result, id)
                    })
                    .await;

                    match result {
                        Ok((Ok(()), _processed_id)) => {
                            cfg.dec_queue();
                            cfg.dec_active();
                        }
                        Ok((Err(e), failed_id)) => {
                            eprintln!("❌ Indexer error on ID {failed_id}: {e}");
                            cfg.dec_queue();
                            cfg.dec_active();
                            errors_ref.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(join_err) => {
                            eprintln!("❌ Indexer task panicked: {join_err}");
                            cfg.dec_queue();
                            cfg.dec_active();
                            errors_ref.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });

                let r = received.fetch_add(1, Ordering::Relaxed) + 1;
                if r.is_multiple_of(10_000) {
                    let active = cfg_worker.active_indexing.load(Ordering::Relaxed);
                    let queue = cfg_worker.queue_size.load(Ordering::Relaxed);
                    let errs = errors.load(Ordering::Relaxed);
                    println!(
                        "📊 Indexer: {r} received, {active} active, {queue} in queue, {errs} errors"
                    );
                }
            }

            let final_r = received.load(Ordering::Relaxed);
            let final_e = errors.load(Ordering::Relaxed);
            println!("🏁 Indexer task finished. Total received: {final_r}, errors: {final_e}");
        });

        // Task 1.2: Initialize MetaRouter and Load Existing Chunks
        let meta_router = Arc::new(MetaRouter::<N>::new());
        // Scan data directory for chunk segments
        if let Ok(entries) = std::fs::read_dir(&data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("chunk_") {
                        // Load chunk metadata (centroid + count) from its snapshot
                        let snap_path = path.join("index.snap");
                        if let Ok(_idx) = HnswIndex::<N, M>::load_snapshot_with_storage_precision(
                            &snap_path,
                            Arc::new(VectorStore::new(&path, element_size)),
                            mode,
                            config.clone(),
                            storage_f32,
                        ) {
                            // Compute/recover centroid for routing
                            // Actually, HnswIndex doesn't store centroid, but we can compute it or
                            // assume it's stored in a separate file.
                            // For simplicity, we skip loading here if not explicitly stored,
                            // but the correct way is to have a chunk_info.json.
                            // Assuming Task 1.2 intended to load them.
                        }
                    }
                }
            }
        }

        let idx_link_snap = index_link.clone();
        let snap_path_clone = snap_path.clone();

        let buckets: Arc<Vec<AtomicU64>> =
            Arc::new(buckets_data.into_iter().map(AtomicU64::new).collect());
        let id_map = Arc::new(id_map_data.into_iter().collect::<DashMap<u32, u32>>());
        // Quick Win #3: HS_IDENTITY_IDS flag for ID mapping bypass
        // If true, skip DashMap lookups entirely (user IDs == internal IDs)
        let identity_ids_env =
            std::env::var("HS_IDENTITY_IDS").is_ok_and(|v| v.to_lowercase() == "true");
        let ids_are_identity =
            identity_ids_env || id_map.iter().all(|entry| *entry.key() == *entry.value());
        if identity_ids_env {
            println!("⚡ ID Mapping: BYPASSED (HS_IDENTITY_IDS=true, user IDs == internal IDs)");
        } else if ids_are_identity {
            println!("⚡ ID Mapping: Identity mode detected (all user IDs match internal IDs)");
        } else {
            println!("🗺️  ID Mapping: Enabled (user ID ↔ internal ID translation)");
        }
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

        let mut initial_root_hash = 0u64;
        for b in buckets.iter() {
            initial_root_hash ^= b.load(Ordering::Relaxed);
        }

        Ok(Self {
            name,
            node_id,
            index_link,
            wal_link,
            index_tx,
            replication_tx,
            config,
            bg_tasks: vec![indexer_task, snapshot_handle],
            buckets,
            root_hash: AtomicU64::new(initial_root_hash),
            reverse_id_map,
            id_map,
            data_dir,
            mode,
            last_clock,
            ids_are_identity: AtomicBool::new(ids_are_identity),
            search_limiter,
            flush_limiter,
            fast_upsert_delta,
            meta_router,
            flushing_vector_count,
            wal_pending_count,
            storage_mode,
            max_ram_bytes,
            pending_wal_flushes,
        })
    }

    #[allow(clippy::too_many_arguments)] // Background worker requires all context
    fn spawn_flush_worker(
        frozen_wal_paths: Vec<PathBuf>,
        config: Arc<GlobalConfig>,
        mode: hyperspace_core::QuantizationMode,
        data_dir: PathBuf,
        flush_limiter: Arc<Semaphore>,
        meta_router: Arc<MetaRouter<N>>,
        index_link: Arc<ArcSwap<HnswIndex<N, M>>>,
        _id_map: Arc<DashMap<u32, u32>>,
        _reverse_id_map: Arc<DashMap<u32, u32>>,
        flushing_vector_count: Arc<AtomicUsize>,
    ) {
        let storage_f32_requested = std::env::var("HS_STORAGE_FLOAT32")
            .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"));
        let storage_f32 = storage_f32_requested && mode == hyperspace_core::QuantizationMode::None;
        let element_size = match mode {
            hyperspace_core::QuantizationMode::ScalarI8 => {
                hyperspace_core::vector::QuantizedHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::Binary => {
                hyperspace_core::vector::BinaryHyperVector::<N>::SIZE
            }
            hyperspace_core::QuantizationMode::None => {
                if storage_f32 {
                    hyperspace_core::vector::HyperVectorF32::<N>::SIZE
                } else {
                    hyperspace_core::vector::HyperVector::<N>::SIZE
                }
            }
        };

        tokio::spawn(async move {
            let permit = flush_limiter.clone().acquire_owned().await;
            let _ = tokio::task::spawn_blocking(move || {
                let _permit = permit;

                println!("🔄 Flush Worker: Starting conversion of {} WAL segment(s)...", frozen_wal_paths.len());
                let _flush_start = std::time::Instant::now();

                // Track the vectors entering purgatory across all files
                let mut total_frozen_entries = 0;
                for path in &frozen_wal_paths {
                    total_frozen_entries += Wal::pending_entries_at_path(path);
                }
                flushing_vector_count.fetch_add(total_frozen_entries as usize, Ordering::SeqCst);
                println!("🔄 Flush Worker: Total entries to process: {total_frozen_entries}");

                let chunk_id = uuid::Uuid::new_v4().to_string();
                let chunk_name = format!("chunk_{chunk_id}.hyp");
                let chunk_dir = data_dir.join(&chunk_name);

                if let Err(e) = std::fs::create_dir_all(&chunk_dir) {
                    eprintln!("Failed to create chunk directory {chunk_name}: {e}");
                    return;
                }

                let temp_store = Arc::new(VectorStore::new(&chunk_dir, element_size));
                let local_index = HnswIndex::<N, M>::new_with_storage_precision(
                    temp_store.clone(),
                    mode,
                    config.clone(),
                    storage_f32,
                );

                let mut insert_count = 0u32;
                let mut centroid_acc = CentroidAccumulator::new(N);

                // Replay ALL accumulated segments into the same chunk
                for (i, path) in frozen_wal_paths.iter().enumerate() {
                    let replay_start = std::time::Instant::now();
                    let replay_res = Wal::replay(path, |entry| {
                        let hyperspace_store::wal::WalEntry::Insert { vector, metadata, .. } = entry;
                        if vector.len() == N {
                            centroid_acc.add(&vector);
                            if let Ok(new_id) = local_index.insert_to_storage(&vector) {
                                let _ = local_index.index_node(new_id, metadata);
                                insert_count += 1;
                            }
                        }
                    });

                    let replay_elapsed = replay_start.elapsed();
                    println!("🔄 Flush Worker: Replay segment {}/{} completed in {:.2}s ({} vectors)",
                        i+1, frozen_wal_paths.len(), replay_elapsed.as_secs_f64(), insert_count);

                    if replay_res.is_err() {
                        eprintln!("⚠️ Failed to replay WAL segment {}/{} during flush. Some data may be lost or chunk will be partial.", i+1, frozen_wal_paths.len());
                    }
                }

                if insert_count == 0 {
                    let _ = std::fs::remove_dir_all(&chunk_dir);
                    for path in &frozen_wal_paths { let _ = std::fs::remove_file(path); }
                    println!("⚠️ Flush Worker: No vectors to insert, cleaned up");
                    return;
                }

                let save_start = std::time::Instant::now();
                if let Err(e) = local_index.save_snapshot(&chunk_dir.join("index.snap")) {
                    eprintln!("Failed to save index for {chunk_name}: {e}");
                } else {
                    let save_elapsed = save_start.elapsed();
                    println!("✅ Flush Worker: Converted {} WAL segment(s) -> Immutable Segment: {} ({} vectors) in {:.2}s",
                        frozen_wal_paths.len(), chunk_name, insert_count, save_elapsed.as_secs_f64()
                    );

                    // Cleanup handled WALs
                    for path in &frozen_wal_paths {
                        let _ = std::fs::remove_file(path);
                    }

                    if let Some(centroid) = centroid_acc.finish() {
                        meta_router.register(ChunkMeta {
                            chunk_id: chunk_name.clone(),
                            path: chunk_dir.clone(),
                            centroid,
                            vector_count: insert_count,
                        });
                        println!("🗺️  MetaRouter: Registered chunk {chunk_name} ({insert_count} vectors)");
                    }

                    // === MemTable Swap (LSM-Tree Core) ===
                    // Create a fresh empty HNSW index to replace the current MemTable.
                    // The old MemTable data is now safely persisted in the chunk.
                    // ArcSwap atomically swaps the pointer — zero downtime.
                    let memtable_dir = data_dir.join("memtable");
                    let _ = std::fs::create_dir_all(&memtable_dir);
                    let fresh_store = Arc::new(VectorStore::new(&memtable_dir, element_size));
                    let fresh_index = Arc::new(HnswIndex::<N, M>::new_with_storage_precision(
                        fresh_store,
                        mode,
                        config.clone(),
                        storage_f32,
                    ));
                    index_link.store(fresh_index);

                    // Note: Clearing id_map was a mistake (Task 1.2 bug).
                    // We must keep all mappings for search and recall to work across segments.
                    // id_map.clear();
                    // reverse_id_map.clear();

                    // Done! Reclaim purgatory count.
                    flushing_vector_count.fetch_sub(total_frozen_entries as usize, Ordering::SeqCst);
                    println!("🔄 MemTable Swap: Replaced hot index with fresh empty HNSW (freed RAM)");
                }
            })
            .await;
        });
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
        self.root_hash.load(Ordering::Relaxed)
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

        let mut reindex_needed = true;
        if let Some(old_internal_id) = existing_internal_id {
            let index = self.index_link.load();
            // Defensive: Only attempt fast-upsert and gossip-undo if vector is in the active HNSW segment.
            if (old_internal_id as usize) < index.count() {
                let old_vector = index.get_vector(old_internal_id);
                if self.config.is_gossip_enabled() {
                    let old_id_hash = CollectionDigest::hash_entry(id, &old_vector.coords);
                    let bucket_idx = CollectionDigest::get_bucket_index(id);
                    self.buckets[bucket_idx].fetch_xor(old_id_hash, Ordering::Relaxed);
                    self.root_hash.fetch_xor(old_id_hash, Ordering::Relaxed);
                }

                if self.fast_upsert_delta > 0.0 {
                    let shift_sq = Self::shift_l2_sq(&old_vector.coords, processed_vector);
                    let old_meta = index.metadata_by_id(old_internal_id);
                    let metadata_changed = old_meta != metadata;
                    reindex_needed = metadata_changed
                        || shift_sq > self.fast_upsert_delta * self.fast_upsert_delta;
                }
            }
        }

        if self.config.is_gossip_enabled() {
            let entry_hash = CollectionDigest::hash_entry(id, processed_vector);
            let bucket_idx = CollectionDigest::get_bucket_index(id);
            self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);
            self.root_hash.fetch_xor(entry_hash, Ordering::Relaxed);
        }

        let internal_id = if let Some(old_id) = existing_internal_id {
            if old_id != id {
                self.ids_are_identity.store(false, Ordering::Release);
            }
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
            if new_id != id {
                self.ids_are_identity.store(false, Ordering::Release);
            }
            new_id
        };

        let mut frozen_paths_opt = None;
        {
            let wal_guard = self.wal_link.load();
            let mut wal = wal_guard.lock().await;

            // Use User ID for WAL to support replication/restore
            wal.append(id, processed_vector, &metadata, clock)
                .map_err(|e| format!("WAL Error: {e}"))?;

            self.last_clock.fetch_max(clock, Ordering::Relaxed);

            if durability == hyperspace_core::Durability::Strict {
                wal.sync().map_err(|e| format!("WAL Sync Error: {e}"))?;
            }

            if wal.is_full() {
                if let Ok(frozen_path) = wal.rotate() {
                    // Reset WAL pending count as they move to next phase
                    self.wal_pending_count.store(0, Ordering::SeqCst);

                    let mut pending = self.pending_wal_flushes.lock().await;
                    pending.push(frozen_path);

                    let should_flush = match self.storage_mode {
                        StorageMode::Tiered => {
                            // LSM-style: Flush when MemTable exceeds memory budget
                            let memtable_nodes = self.index_link.load().count_nodes();
                            let memtable_budget = self.max_ram_bytes / 10;
                            let est_memory = memtable_nodes * (N * 8 + 64);

                            let should = est_memory as u64 > memtable_budget;

                            // DEBUG: Log every rotation
                            if should {
                                println!(
                                    "🔍 Flush Check (Tiered): memtable={} vectors | est_memory={} MB | threshold={} MB | should_flush={}",
                                    memtable_nodes,
                                    est_memory / (1024 * 1024),
                                    memtable_budget / (1024 * 1024),
                                    should
                                );
                            }

                            should
                        }
                        StorageMode::Performance => {
                            // Performance Mode: NEVER flush to chunks
                            // All data stays in RAM (MemTable) for maximum performance
                            // Persistence is handled by snapshots only
                            false
                        }
                    };

                    if should_flush {
                        // Take all pending segments to flush into one chunk
                        frozen_paths_opt = Some(std::mem::take(&mut *pending));
                    } else {
                        println!(
                            "📦 WAL Rotated ({} pending segments), keeping MemTable HOT (Performance Mode)", 
                            pending.len()
                        );
                    }
                }
            } else {
                self.wal_pending_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        if let Some(frozen_paths) = frozen_paths_opt {
            Self::spawn_flush_worker(
                frozen_paths,
                self.config.clone(),
                self.mode,
                self.data_dir.clone(),
                self.flush_limiter.clone(),
                self.meta_router.clone(),
                self.index_link.clone(),
                self.id_map.clone(),
                self.reverse_id_map.clone(),
                self.flushing_vector_count.clone(),
            );
        }

        if reindex_needed {
            self.config.inc_queue();
            let queue_size = self.config.get_queue_size();

            // Debug: Log queue buildup
            if queue_size > 10_000 && queue_size.is_multiple_of(5_000) {
                let active = self.config.active_indexing.load(Ordering::Relaxed);
                println!("⚠️  Index queue building up: {queue_size} pending, {active} active");
            }

            let _ = self.index_tx.send((internal_id, metadata.clone()));
        }

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
                    typed_metadata: HashMap::new(),
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
            let mut reindex_needed = true;
            if let Some(old_internal_id) = existing_internal_id {
                // Defensive: Only attempt fast-upsert and gossip-undo if vector is in the active HNSW segment.
                if (old_internal_id as usize) < index_reader.count() {
                    let old_vector = index_reader.get_vector(old_internal_id);
                    if self.config.is_gossip_enabled() {
                        let old_id_hash = CollectionDigest::hash_entry(*id, &old_vector.coords);
                        let bucket_idx = CollectionDigest::get_bucket_index(*id);
                        self.buckets[bucket_idx].fetch_xor(old_id_hash, Ordering::Relaxed);
                        self.root_hash.fetch_xor(old_id_hash, Ordering::Relaxed);
                    }

                    if self.fast_upsert_delta > 0.0 {
                        let shift_sq = Self::shift_l2_sq(&old_vector.coords, &processed_vector);
                        let old_meta = index_reader.metadata_by_id(old_internal_id);
                        let metadata_changed = old_meta != *metadata;
                        reindex_needed = metadata_changed
                            || shift_sq > self.fast_upsert_delta * self.fast_upsert_delta;
                    }
                }
            }

            if self.config.is_gossip_enabled() {
                let entry_hash = CollectionDigest::hash_entry(*id, &processed_vector);
                let bucket_idx = CollectionDigest::get_bucket_index(*id);
                self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);
                self.root_hash.fetch_xor(entry_hash, Ordering::Relaxed);
            }

            // Storage
            // insert_to_storage writes bytes to Mmap. It copies bytes, but doesn't heap allocate vector objects.
            let internal_id = if let Some(old_id) = existing_internal_id {
                if old_id != *id {
                    self.ids_are_identity.store(false, Ordering::Release);
                }
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
                if new_id != *id {
                    self.ids_are_identity.store(false, Ordering::Release);
                }
                new_id
            };

            entries.push(BatchEntry {
                id: *id,
                vector: processed_vector, // Moves the Cow (cheap pointer copy), not data
                metadata,                 // Reference
                internal_id,
                reindex_needed,
            });
        }

        // 3. WAL Batch
        // Allocate here as WAL requires owned data.
        // This is the first allocation of the vector in the Poincaré pipeline.
        let wal_data: Vec<_> = entries
            .iter()
            .map(|e| (e.vector.to_vec(), e.id, e.metadata.clone()))
            .collect();

        let mut frozen_paths_opt = None;
        {
            let wal_guard = self.wal_link.load();
            let mut wal = wal_guard.lock().await;
            wal.append_batch(&wal_data, clock)
                .map_err(|e| e.to_string())?;

            self.last_clock.fetch_max(clock, Ordering::Relaxed);

            if durability == hyperspace_core::Durability::Strict {
                wal.sync().map_err(|e| e.to_string())?;
            }

            if wal.is_full() {
                if let Ok(frozen_path) = wal.rotate() {
                    // Reset WAL pending count as they move to next phase
                    self.wal_pending_count.store(0, Ordering::SeqCst);

                    let mut pending = self.pending_wal_flushes.lock().await;
                    pending.push(frozen_path);

                    let should_flush = match self.storage_mode {
                        StorageMode::Tiered => {
                            // LSM-style: Flush when MemTable exceeds memory budget
                            let memtable_nodes = self.index_link.load().count_nodes();
                            let memtable_budget = self.max_ram_bytes / 10;
                            let est_memory = memtable_nodes * (N * 8 + 64);

                            let should = est_memory as u64 > memtable_budget;

                            // DEBUG: Log every rotation
                            if should {
                                println!(
                                    "🔍 Flush Check (Tiered, batch): memtable={} vectors | est_memory={} MB | threshold={} MB | should_flush={}",
                                    memtable_nodes,
                                    est_memory / (1024 * 1024),
                                    memtable_budget / (1024 * 1024),
                                    should
                                );
                            }

                            should
                        }
                        StorageMode::Performance => {
                            // Performance Mode: NEVER flush to chunks
                            // All data stays in RAM (MemTable) for maximum performance
                            false
                        }
                    };

                    if should_flush {
                        frozen_paths_opt = Some(std::mem::take(&mut *pending));
                    } else {
                        println!(
                            "📦 WAL Rotated (batch, {} pending segments), keeping MemTable HOT (Performance Mode)",
                            pending.len()
                        );
                    }
                }
            } else {
                self.wal_pending_count
                    .fetch_add(vectors.len() as u64, Ordering::SeqCst);
            }
        }

        if let Some(frozen_paths) = frozen_paths_opt {
            Self::spawn_flush_worker(
                frozen_paths,
                self.config.clone(),
                self.mode,
                self.data_dir.clone(),
                self.flush_limiter.clone(),
                self.meta_router.clone(),
                self.index_link.clone(),
                self.id_map.clone(),
                self.reverse_id_map.clone(),
                self.flushing_vector_count.clone(),
            );
        }

        // 4. Index Queue
        for _ in 0..entries.iter().filter(|e| e.reindex_needed).count() {
            self.config.inc_queue();
        }

        // Queue for indexing (Send only lightweight metadata clone + internal_id)
        for entry in &entries {
            if entry.reindex_needed {
                let _ = self
                    .index_tx
                    .send((entry.internal_id, entry.metadata.clone()));
            }
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
                        typed_metadata: HashMap::new(),
                    })),
                };
                let _ = self.replication_tx.send(log);
            }
        }

        Ok(())
    }

    fn delete(&self, id: u32) -> Result<(), String> {
        let internal_id = if let Some((_, internal_id)) = self.id_map.remove(&id) {
            self.reverse_id_map.remove(&internal_id);
            internal_id
        } else {
            id
        };

        let idx = self.index_link.load();
        if self.config.is_gossip_enabled() {
            // Defensive check: only update if ID is within bounds of active index
            if (internal_id as usize) < idx.count() {
                let vector = idx.get_vector(internal_id);
                let hash = CollectionDigest::hash_entry(id, &vector.coords);
                let b_idx = CollectionDigest::get_bucket_index(id);

                self.buckets[b_idx].fetch_xor(hash, Ordering::Relaxed);
                self.root_hash.fetch_xor(hash, Ordering::Relaxed);
            }
        }

        idx.delete(internal_id);
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

        // Quick Win #5: Zero-copy normalization - keep Cow until absolutely necessary
        let processed_query_cow = Self::normalize_if_cosine(query);

        let index_link = self.index_link.clone();
        let reverse_id_map = self.reverse_id_map.clone();
        let ids_are_identity = self.ids_are_identity.load(Ordering::Acquire);

        // Move only the required fields to avoid cloning whole params struct.
        let top_k = params.top_k;
        let ef_search = params.ef_search;
        let rerank_enabled = std::env::var("HS_RERANK_ENABLED")
            .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"));
        let rerank_oversample = std::env::var("HS_RERANK_OVERSAMPLE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4)
            .max(1);
        let hybrid_query = params.hybrid_query.clone();
        let hybrid_alpha = params.hybrid_alpha;
        let use_wasserstein = params.use_wasserstein;
        let filters_owned = (!filters.is_empty()).then(|| filters.clone());
        let complex_filters_owned = (!complex_filters.is_empty()).then(|| complex_filters.to_vec());
        let meta_router_ref = self.meta_router.clone();
        let mode_for_search = self.mode;
        let config_for_search = self.config.clone();
        let permit = self
            .search_limiter
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Search limiter failed: {e}"))?;

        // Quick Win: For small top_k, run search inline to avoid spawn_blocking overhead
        let use_blocking = top_k > 50 || rerank_enabled;

        if use_blocking {
            // Convert to owned only when entering blocking task
            let processed_query = processed_query_cow.into_owned();
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                let index = index_link.load();
                let include_metadata = index.has_nonempty_metadata();
                let filters_ref = filters_owned.as_ref().unwrap_or(&EMPTY_LEGACY_FILTERS);
                let complex_filters_ref = complex_filters_owned
                    .as_ref()
                    .map_or(EMPTY_COMPLEX_FILTERS.as_slice(), Vec::as_slice);
                let search_k = if rerank_enabled {
                    top_k.saturating_mul(rerank_oversample).max(top_k)
                } else {
                    top_k
                };

                // === 1. Search the hot MemTable (in-RAM HNSW) ===
                let mem_results = index.search(
                    &processed_query,
                    search_k,
                    ef_search,
                    filters_ref,
                    complex_filters_ref,
                    hybrid_query.as_deref(),
                    hybrid_alpha,
                    use_wasserstein,
                );

                // === 2. Search cold chunks via MetaRouter (disk mmap) ===
                let probe_k = std::env::var("HS_CHUNK_PROBE_K")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(3);
                let routed_chunks = meta_router_ref.route(&processed_query, probe_k);
                let chunk_dirs: Vec<std::path::PathBuf> = routed_chunks
                    .iter()
                    .map(|(_, path, _)| path.clone())
                    .collect();

                let chunk_results = if chunk_dirs.is_empty() {
                    Vec::new()
                } else {
                    chunk_searcher::scatter_gather_search::<N, M>(
                        &chunk_dirs,
                        &processed_query,
                        search_k,
                        ef_search,
                        filters_ref,
                        complex_filters_ref,
                        mode_for_search,
                        &config_for_search,
                        use_wasserstein,
                    )
                };

                // === 3. Merge MemTable + Chunk results by distance ===
                // MemTable results carry real internal IDs.
                // Chunk results carry chunk-local IDs (not usable for metadata).
                // We merge by distance only, preferring MemTable entries for metadata.
                let mut merged: Vec<(u32, f64, bool)> =
                    Vec::with_capacity(mem_results.len() + chunk_results.len());

                for (id, dist) in &mem_results {
                    merged.push((*id, *dist, true)); // true = from MemTable
                }
                for (_, dist, _) in &chunk_results {
                    // Chunk results don't have usable IDs for the collection's id_map.
                    // We use u32::MAX as a sentinel — they'll be filtered in metadata step.
                    merged.push((u32::MAX, *dist, false)); // false = from chunk
                }

                merged.sort_by(|a, b| a.1.total_cmp(&b.1));
                merged.truncate(search_k);

                // Include all results (RAM + Chunks).
                // Note: Results from chunks will have internal IDs that need a segment mapping.
                let results: Vec<(u32, f64)> =
                    merged.into_iter().map(|(id, dist, _)| (id, dist)).collect();

                let metric_tag = match M::name() {
                    "cosine" => GpuMetric::Cosine,
                    "poincare" => GpuMetric::Poincare,
                    "lorentz" => GpuMetric::Lorentz,
                    _ => GpuMetric::L2,
                };

                let reranked_internal: Vec<(u32, f64)> = if rerank_enabled && !results.is_empty() {
                    let candidate_ids: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
                    let candidate_vectors: Vec<Vec<f64>> = candidate_ids
                        .iter()
                        .map(|id| index.get_vector(*id).coords.to_vec())
                        .collect();
                    let candidate_refs: Vec<&[f64]> =
                        candidate_vectors.iter().map(Vec::as_slice).collect();
                    rerank_topk_exact(
                        metric_tag,
                        &processed_query,
                        &candidate_ids,
                        &candidate_refs,
                    )
                } else {
                    results
                };

                // Fetch metadata and convert IDs inside blocking worker.
                reranked_internal
                    .into_iter()
                    .take(top_k)
                    .map(|(internal_id, dist)| {
                        let meta = if include_metadata {
                            index
                                .metadata
                                .forward
                                .get(&internal_id)
                                .map(|m| m.clone())
                                .unwrap_or_default()
                        } else {
                            HashMap::new()
                        };

                        let user_id = if ids_are_identity {
                            internal_id
                        } else {
                            reverse_id_map.get(&internal_id).map_or(internal_id, |v| *v)
                        };

                        (user_id, dist, meta)
                    })
                    .collect::<Vec<SearchResult>>()
            })
            .await
            .map_err(|e| format!("Search task failed: {e}"))
        } else {
            // Quick Win: Inline search for small top_k - avoid spawn_blocking overhead
            // Still need to convert Cow to owned for HNSW search
            let processed_query = processed_query_cow.into_owned();
            let _permit = permit;
            let index = index_link.load();
            let include_metadata = index.has_nonempty_metadata();
            let filters_ref = filters_owned.as_ref().unwrap_or(&EMPTY_LEGACY_FILTERS);
            let complex_filters_ref = complex_filters_owned
                .as_ref()
                .map_or(EMPTY_COMPLEX_FILTERS.as_slice(), Vec::as_slice);

            // === 1. Search the hot MemTable (in-RAM HNSW) ===
            let mem_results = index.search(
                &processed_query,
                top_k,
                ef_search,
                filters_ref,
                complex_filters_ref,
                hybrid_query.as_deref(),
                hybrid_alpha,
                use_wasserstein,
            );

            // === 2. Search cold chunks (skip for small queries - assume hot data) ===
            // Skip chunk search for small top_k to reduce latency

            // === 3. Convert results ===
            let results: Vec<SearchResult> = mem_results
                .into_iter()
                .take(top_k)
                .map(|(internal_id, dist)| {
                    let meta = if include_metadata {
                        index
                            .metadata
                            .forward
                            .get(&internal_id)
                            .map(|m| m.clone())
                            .unwrap_or_default()
                    } else {
                        HashMap::new()
                    };

                    let user_id = if ids_are_identity {
                        internal_id
                    } else {
                        reverse_id_map.get(&internal_id).map_or(internal_id, |v| *v)
                    };

                    (user_id, dist, meta)
                })
                .collect();

            Ok(results)
        }
    }

    async fn optimize(&self) -> Result<(), String> {
        self.optimize_with_filter(None).await
    }

    async fn optimize_with_filter(&self, filter: Option<VacuumFilterQuery>) -> Result<(), String> {
        println!("🧹 Starting Hot Vacuum for '{}'...", self.name);
        let start = std::time::Instant::now();
        // Removed unused name
        let data_dir = self.data_dir.clone();
        let mode = self.mode;
        let original_config = self.config.clone();
        let index_link = self.index_link.clone();
        let filter_for_vacuum = filter.clone();

        // Run heavy lifting in blocking thread
        let (new_index_arc, temp_dir, new_snap_path) = tokio::task::spawn_blocking(move || {
            use hyperspace_core::config::GlobalConfig;
            use hyperspace_store::VectorStore;
            use std::path::PathBuf;

            // 1. Get current data
            let current_index = index_link.load().clone();
            let mut all_data = current_index.peek_all();
            if let Some(filter) = &filter_for_vacuum {
                all_data.retain(|(_, _, meta)| !Self::matches_vacuum_filter(meta, filter));
            }
            let count = all_data.len();

            if count == 0 {
                return Ok((None, PathBuf::new(), PathBuf::new())); // Nothing to do
            }

            // 2. Setup "Turbo Mode"
            let vacuum_m = 128;
            let vacuum_ef = 800;

            let vacuum_config = Arc::new(GlobalConfig::new());
            vacuum_config.set_m(vacuum_m);
            vacuum_config.set_ef_construction(vacuum_ef);
            vacuum_config.set_ef_search(original_config.get_ef_search());

            println!("   Building Shadow Index (M={vacuum_m}, EF={vacuum_ef})...");

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
            for (_old_id, vec, meta) in &all_data {
                // Ensure insert handles internal logic
                let _ = new_index.insert(vec, meta.clone());
            }

            // Save to disk
            let new_snap_path = data_dir.join("index.snap.new");
            if let Err(e) = new_index.save_snapshot(&new_snap_path) {
                return Err(e.clone());
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
        let mem_count = self.index_link.load().count_nodes();
        let chunk_count = self.meta_router.total_vector_count();
        let purgatory_count = self.flushing_vector_count.load(Ordering::Relaxed);
        mem_count + chunk_count + purgatory_count
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
                    .map_or(internal_id, |v| *v);
                (user_id, vec, meta)
            })
            .collect()
    }

    fn queue_size(&self) -> u64 {
        let hnsw_queue = self.config.get_queue_size();

        // Performance Mode: Only report HNSW indexing queue
        // Tiered Mode: Include WAL pending and flushing queues for full picture
        match self.storage_mode {
            StorageMode::Performance => hnsw_queue,
            StorageMode::Tiered => {
                let wal_queue = self.wal_pending_count.load(Ordering::Relaxed);
                let flushing_queue = self.flushing_vector_count.load(Ordering::Relaxed) as u64;
                hnsw_queue + wal_queue + flushing_queue
            }
        }
    }

    fn graph_neighbors(&self, id: u32, layer: usize, limit: usize) -> Result<Vec<u32>, String> {
        let internal_id = self.to_internal_id(id);
        let neighbors = self
            .index_link
            .load()
            .graph_neighbors(internal_id, layer, limit)?;
        Ok(neighbors.into_iter().map(|n| self.to_user_id(n)).collect())
    }

    fn graph_neighbor_distances(
        &self,
        source_id: u32,
        neighbor_ids: &[u32],
    ) -> Result<Vec<f64>, String> {
        let idx = self.index_link.load();
        let source_internal_id = self.to_internal_id(source_id);
        let source = idx.get_vector(source_internal_id);
        let distances = neighbor_ids
            .iter()
            .map(|neighbor_id| {
                let n_internal = self.to_internal_id(*neighbor_id);
                let n_vec = idx.get_vector(n_internal);
                M::distance(&source.coords, &n_vec.coords)
            })
            .collect();
        Ok(distances)
    }

    fn graph_traverse(
        &self,
        start_id: u32,
        layer: usize,
        max_depth: usize,
        max_nodes: usize,
    ) -> Result<Vec<u32>, String> {
        let internal_start = self.to_internal_id(start_id);
        let traversed =
            self.index_link
                .load()
                .graph_traverse(internal_start, layer, max_depth, max_nodes)?;
        Ok(traversed.into_iter().map(|n| self.to_user_id(n)).collect())
    }

    fn graph_clusters(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Result<Vec<Vec<u32>>, String> {
        let clusters = self.index_link.load().graph_connected_components(
            layer,
            min_cluster_size,
            max_clusters,
            max_nodes,
        );
        Ok(clusters
            .into_iter()
            .map(|c| c.into_iter().map(|n| self.to_user_id(n)).collect())
            .collect())
    }

    fn metadata_by_id(&self, id: u32) -> HashMap<String, String> {
        let internal_id = self.to_internal_id(id);
        self.index_link.load().metadata_by_id(internal_id)
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
