use crate::collection::CollectionImpl;
use async_trait::async_trait;
use hyperspace_core::{
    Collection, CollectionConfigUpdate, CollectionUsage, Durability, FilterExpr, HnswConfig,
    Metric, SearchParams, SearchResult, VacuumFilterQuery,
};
use hyperspace_proto::hyperspace::{CollectionSchema, ReplicationLog};
use hyperspace_store::wal::{Wal, WalSyncMode};
use std::collections::{BinaryHeap, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug)]
struct ScoredCandidate {
    distance: f64,
    id: u32,
    metadata: HashMap<String, String>,
    payload: Option<Vec<u8>>,
}

impl PartialEq for ScoredCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for ScoredCandidate {}

impl PartialOrd for ScoredCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Max-heap: peek() always returns the candidate with the LARGEST distance.
        // This lets us efficiently discard the worst candidate when the heap is full.
        // NaN is treated as greater than any finite value so it is always evicted first.
        other
            .distance
            .partial_cmp(&self.distance)
            .map(|o| o.reverse())
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

#[derive(Debug)]
struct ScoredCandidate64 {
    distance: f64,
    id: u64,
    metadata: HashMap<String, String>,
    payload: Option<Vec<u8>>,
}

impl PartialEq for ScoredCandidate64 {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for ScoredCandidate64 {}

impl PartialOrd for ScoredCandidate64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredCandidate64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .distance
            .partial_cmp(&self.distance)
            .map(|o| o.reverse())
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

/// Thread-Per-Core (Shared-Nothing) Sharded Collection for HyperspaceDB v4.0.0.
///
/// Divides vector storage, WriteBuffers, HNSW graphs, and Direct I/O blocks into N isolated shards.
/// Each CPU core processes shard-local ingestion and scatter-gather searches without cross-core locks.
pub struct ShardedCollection<M: Metric> {
    name: String,
    dimension: usize,
    mode: hyperspace_core::QuantizationMode,
    #[allow(dead_code)]
    schema: CollectionSchema,
    shards: Vec<Arc<CollectionImpl<M>>>,
    num_shards: usize,
    root_hash: AtomicU64,
    wal: Option<Arc<tokio::sync::Mutex<Wal>>>,
}

impl<M: Metric> ShardedCollection<M> {
    /// Creates a new `ShardedCollection` with the requested number of core shards.
    pub async fn new(
        name: String,
        node_id: String,
        col_dir: PathBuf,
        quant_mode: hyperspace_core::QuantizationMode,
        replication_tx: broadcast::Sender<ReplicationLog>,
        dimension: usize,
        schema: CollectionSchema,
        num_shards_opt: Option<usize>,
    ) -> Result<Self, String> {
        let cpu_cores = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
        let requested_shards = num_shards_opt
            .or_else(|| {
                std::env::var("HS_SHARDS").ok().and_then(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") || trimmed == "0"
                    {
                        None
                    } else {
                        trimmed.parse().ok()
                    }
                })
            })
            .unwrap_or_else(|| cpu_cores.clamp(1, 8))
            .max(1);

        // Shard routing is `id % num_shards`, so the shard count is part of the on-disk
        // layout. If shards already exist, they win over CPU count / HS_SHARDS — otherwise
        // a restart on a machine with a different core count (or a new k8s node) would
        // route IDs to the wrong shard and make existing vectors unreachable.
        let mut existing = 0usize;
        while col_dir.join(format!("shard_{existing}")).is_dir() {
            existing += 1;
        }
        let num_shards = if existing > 0 {
            if existing != requested_shards {
                println!(
                    "⚠️  [v4] '{name}' has {existing} shard(s) on disk; ignoring requested {requested_shards} (cores={cpu_cores}). Shard count is fixed at creation."
                );
            }
            existing
        } else {
            requested_shards
        };

        println!(
            "⚡ [v4.0.0 Thread-Per-Core] Initializing ShardedCollection '{name}' across {num_shards} core shard(s)"
        );

        let mut shards = Vec::with_capacity(num_shards);
        for i in 0..num_shards {
            let shard_dir = col_dir.join(format!("shard_{i}"));
            std::fs::create_dir_all(&shard_dir)
                .map_err(|e| format!("Failed to create shard dir {}: {e}", shard_dir.display()))?;
            let shard_wal = shard_dir.join("wal.log");

            let shard = CollectionImpl::<M>::new(
                format!("{name}_shard_{i}"),
                node_id.clone(),
                shard_dir,
                shard_wal,
                quant_mode,
                replication_tx.clone(),
                dimension,
                schema.clone(),
            )
            .await
            .map_err(|e| e.to_string())?;

            shards.push(Arc::new(shard));
        }

        let (wal, col_wal_path) = if num_shards > 1 {
            let sync_mode_str = std::env::var("HYPERSPACE_WAL_SYNC_MODE")
                .unwrap_or_else(|_| "async".to_string())
                .to_lowercase();
            let sync_mode = match sync_mode_str.as_str() {
                "strict" | "fsync" => WalSyncMode::Strict,
                "batch" => WalSyncMode::Batch,
                _ => WalSyncMode::Async,
            };
            let p = col_dir.join("wal.log");
            let mut wal_inst = Wal::new(&p, sync_mode)
                .map_err(|e| format!("Failed to create unified WAL {}: {e}", p.display()))?;
            let wal_segment_mb = std::env::var("HS_WAL_SEGMENT_SIZE_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(256)
                .clamp(16, 16384);
            wal_inst.set_size_limit(wal_segment_mb * 1024 * 1024);
            (Some(Arc::new(tokio::sync::Mutex::new(wal_inst))), Some(p))
        } else {
            (None, None)
        };

        if let Some(ref p) = col_wal_path {
            let mut replay_files = Vec::new();
            if let Some(parent) = p.parent() {
                let mut frozen = Vec::new();
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        if file_name.contains(".frozen.") {
                            if let Some(ts_str) = file_name.rsplit('.').next() {
                                if let Ok(ts) = ts_str.parse::<u64>() {
                                    frozen.push((ts, path));
                                }
                            }
                        }
                    }
                }
                frozen.sort_by_key(|(ts, _)| *ts);
                replay_files.extend(frozen.into_iter().map(|(_, path)| path));
            }
            if p.exists() {
                replay_files.push(p.clone());
            }

            for file_path in replay_files {
                let mut replay_entries = Vec::new();
                let _ = Wal::replay(&file_path, |entry| {
                    replay_entries.push(entry);
                });
                if !replay_entries.is_empty() {
                    println!(
                        "⚡ [v4.0.0] Replaying {} unified WAL entries from {} across {} shards...",
                        replay_entries.len(),
                        file_path.display(),
                        num_shards
                    );
                    for entry in replay_entries {
                        match entry {
                            hyperspace_store::wal::WalEntry::Insert {
                                id,
                                vector,
                                metadata,
                                logical_clock,
                            } => {
                                let shard_idx = (id as usize) % num_shards;
                                let shard = &shards[shard_idx];
                                if logical_clock > shard.last_clock() {
                                    let _ = shard
                                        .insert_internal(
                                            &vector,
                                            id,
                                            metadata,
                                            logical_clock,
                                            Durability::Async,
                                            false,
                                        )
                                        .await;
                                }
                            }
                            hyperspace_store::wal::WalEntry::Insert64 {
                                id,
                                vector,
                                metadata,
                                logical_clock,
                            } => {
                                let shard_idx = (id % (num_shards as u64)) as usize;
                                let local_id = (id / (num_shards as u64)) as u32;
                                let shard = &shards[shard_idx];
                                if logical_clock > shard.last_clock() {
                                    let _ = shard
                                        .insert_internal(
                                            &vector,
                                            local_id,
                                            metadata,
                                            logical_clock,
                                            Durability::Async,
                                            false,
                                        )
                                        .await;
                                }
                            }
                            hyperspace_store::wal::WalEntry::Delete { id, logical_clock } => {
                                let shard_idx = (id % (num_shards as u64)) as usize;
                                let local_id = (id / (num_shards as u64)) as u32;
                                let shard = &shards[shard_idx];
                                if logical_clock > shard.last_clock() {
                                    let _ = shard.delete(local_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            name,
            dimension,
            mode: quant_mode,
            schema,
            shards,
            num_shards,
            root_hash: AtomicU64::new(0),
            wal,
        })
    }

    #[inline]
    fn get_shard_index(&self, id: u32) -> usize {
        (id as usize) % self.num_shards
    }

    /// Decomposes a global 64-bit ID into a target shard index and local 32-bit ID.
    /// This enables collections with > 5B to 21.2B+ vectors (e.g. 34.3B vectors across 8 shards)
    /// without converting internal HNSW graph pointers to 64-bit (saving gigabytes/terabytes of RAM).
    #[inline]
    pub fn decompose_id64(&self, id: u64) -> Result<(usize, u32), String> {
        let n = self.num_shards as u64;
        let shard_idx = (id % n) as usize;
        let local_id = id / n;
        if local_id > u32::MAX as u64 {
            return Err(format!(
                "ID {} exceeds shard capacity with {} shards (local_id {} > u32::MAX)",
                id, self.num_shards, local_id
            ));
        }
        Ok((shard_idx, local_id as u32))
    }

    /// Composes a global 64-bit ID from shard index and local 32-bit ID.
    #[inline]
    pub fn compose_id64(&self, shard_idx: usize, local_id: u32) -> u64 {
        (local_id as u64) * (self.num_shards as u64) + (shard_idx as u64)
    }

    /// Flushes all pending writes, waits for asynchronous indexing to finish,
    /// and saves an HNSW snapshot along with state.json for every shard.
    pub async fn flush_and_snapshot(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        loop {
            let q = self.queue_size();
            let w = self.write_buffer_size();
            let a = self.get_usage().active_indexing_tasks;
            if q == 0 && w == 0 && a == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            if start.elapsed().as_secs() > 180 {
                return Err("Timeout waiting for sharded index queue to drain".to_string());
            }
        }
        self.create_snapshot()
    }
}

#[async_trait]
impl<M: Metric> Collection for ShardedCollection<M> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn insert(
        &self,
        vector: &[f64],
        id: u32,
        metadata: HashMap<String, String>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        if self.num_shards == 1 {
            return self.shards[0]
                .insert(vector, id, metadata, clock, durability)
                .await;
        }

        if let Some(ref wal) = self.wal {
            let mut wal_guard = wal.lock().await;
            wal_guard
                .append(id, vector, &metadata, clock)
                .map_err(|e| format!("Unified WAL append failed: {e}"))?;
            if durability == Durability::Strict {
                wal_guard
                    .sync()
                    .map_err(|e| format!("Unified WAL sync failed: {e}"))?;
            }
        }

        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx]
            .insert_internal(vector, id, metadata, clock, durability, false)
            .await?;
        self.root_hash
            .fetch_xor(id as u64 ^ clock, Ordering::Relaxed);
        Ok(())
    }

    async fn insert_batch(
        &self,
        vectors: Vec<(Vec<f64>, u32, HashMap<String, String>)>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        if self.num_shards == 1 {
            return self.shards[0]
                .insert_batch(vectors, clock, durability)
                .await;
        }

        let t0 = std::time::Instant::now();
        // 1. Sequential batch append to unified collection WAL
        if let Some(ref wal) = self.wal {
            let mut wal_guard = wal.lock().await;
            wal_guard
                .append_batch(&vectors, clock)
                .map_err(|e| format!("Unified WAL append_batch failed: {e}"))?;
            if durability == Durability::Strict {
                wal_guard
                    .sync()
                    .map_err(|e| format!("Unified WAL sync failed: {e}"))?;
            }
        }
        let t_wal = t0.elapsed();

        let batch_len = vectors.len();
        let t1 = std::time::Instant::now();
        let mut buckets: Vec<Vec<(Vec<f64>, u32, HashMap<String, String>)>> =
            vec![Vec::new(); self.num_shards];

        for item in vectors {
            let shard_idx = self.get_shard_index(item.1);
            buckets[shard_idx].push(item);
        }
        let t_bucket = t1.elapsed();

        // 3. Concurrently insert into shards without per-shard disk WAL overhead
        let t2 = std::time::Instant::now();
        let futures: Vec<_> = buckets
            .into_iter()
            .enumerate()
            .filter(|(_, batch)| !batch.is_empty())
            .map(|(shard_idx, batch)| {
                let shard = Arc::clone(&self.shards[shard_idx]);
                async move {
                    shard
                        .insert_batch_internal(batch, clock, durability, false)
                        .await
                }
            })
            .collect();

        for res in futures::future::join_all(futures).await {
            res?;
        }
        let t_shards = t2.elapsed();
        if t_shards.as_millis() > 500 {
            eprintln!(
                "⚠️ [Batch {}] slow insert: wal={:?}, bucket={:?}, shards={:?}",
                batch_len, t_wal, t_bucket, t_shards
            );
        }
        Ok(())
    }

    async fn insert_u64(
        &self,
        vector: &[f64],
        id: u64,
        metadata: HashMap<String, String>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        let (shard_idx, local_id) = self.decompose_id64(id)?;

        if let Some(ref wal) = self.wal {
            let mut wal_guard = wal.lock().await;
            wal_guard
                .append_u64(id, vector, &metadata, clock)
                .map_err(|e| format!("Unified WAL append_u64 failed: {e}"))?;
            if durability == Durability::Strict {
                wal_guard
                    .sync()
                    .map_err(|e| format!("Unified WAL sync failed: {e}"))?;
            }
        }

        self.shards[shard_idx]
            .insert_internal(vector, local_id, metadata, clock, durability, false)
            .await?;
        self.root_hash.fetch_xor(id ^ clock, Ordering::Relaxed);
        Ok(())
    }

    async fn insert_batch_u64(
        &self,
        vectors: Vec<(Vec<f64>, u64, HashMap<String, String>)>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        if vectors.is_empty() {
            return Ok(());
        }

        if let Some(ref wal) = self.wal {
            let mut wal_guard = wal.lock().await;
            wal_guard
                .append_batch_u64(&vectors, clock)
                .map_err(|e| format!("Unified WAL append_batch_u64 failed: {e}"))?;
            if durability == Durability::Strict {
                wal_guard
                    .sync()
                    .map_err(|e| format!("Unified WAL sync failed: {e}"))?;
            }
        }

        let mut buckets: Vec<Vec<(Vec<f64>, u32, HashMap<String, String>)>> =
            vec![Vec::new(); self.num_shards];

        for (vec, id, meta) in vectors {
            let (shard_idx, local_id) = self.decompose_id64(id)?;
            buckets[shard_idx].push((vec, local_id, meta));
        }

        let futures: Vec<_> = buckets
            .into_iter()
            .enumerate()
            .filter(|(_, batch)| !batch.is_empty())
            .map(|(shard_idx, batch)| {
                let shard = Arc::clone(&self.shards[shard_idx]);
                async move {
                    shard
                        .insert_batch_internal(batch, clock, durability, false)
                        .await
                }
            })
            .collect();

        for res in futures::future::join_all(futures).await {
            res?;
        }
        Ok(())
    }

    fn delete(&self, id: u32) -> Result<(), String> {
        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx].delete(id)
    }

    fn delete_u64(&self, id: u64) -> Result<(), String> {
        let (shard_idx, local_id) = self.decompose_id64(id)?;
        self.shards[shard_idx].delete(local_id)
    }

    fn update_payload(&self, id: u32, metadata: HashMap<String, String>) -> Result<(), String> {
        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx].update_payload(id, metadata)
    }

    fn metadata_by_id(&self, id: u32) -> HashMap<String, String> {
        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx].metadata_by_id(id)
    }

    async fn search(
        &self,
        vector: &[f64],
        filter: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResult>, String> {
        if self.num_shards == 1 {
            return self.shards[0]
                .search(vector, filter, complex_filters, params)
                .await;
        }

        // ── Scatter-Gather Oversampling ──────────────────────────────────────
        let top_k = params.top_k.max(1);
        let shard_top_k = (top_k + 6).max(top_k * 2);
        let shard_ef = params.ef_search.max(shard_top_k);

        let shard_params = SearchParams {
            top_k: shard_top_k,
            ef_search: shard_ef,
            hybrid_query: params.hybrid_query.clone(),
            hybrid_alpha: params.hybrid_alpha,
            component_weights: params.component_weights.clone(),
            use_wasserstein: params.use_wasserstein,
            bm25_options: params.bm25_options.clone(),
            fusion_method: params.fusion_method.clone(),
            mrl_dimension: params.mrl_dimension,
            include_payload: params.include_payload,
            use_wave: params.use_wave,
        };

        let futures: Vec<_> = self
            .shards
            .iter()
            .map(|shard| {
                let shard_ref = Arc::clone(shard);
                let sp = shard_params.clone();
                async move { shard_ref.search(vector, filter, complex_filters, &sp).await }
            })
            .collect();

        let shard_results = futures::future::join_all(futures).await;

        // ── Global Top-K Merge ───────────────────────────────────────────────
        // Max-heap keyed by distance: peek() always holds the worst (largest
        // distance) candidate currently in the set, making eviction O(log N).
        let mut heap: BinaryHeap<ScoredCandidate> = BinaryHeap::with_capacity(top_k + 1);

        for res in shard_results {
            match res {
                Ok(items) => {
                    for (id, dist, meta, payload) in items {
                        if heap.len() < top_k {
                            heap.push(ScoredCandidate {
                                distance: dist,
                                id,
                                metadata: meta,
                                payload,
                            });
                        } else {
                            // SAFETY: heap is non-empty because len >= top_k >= 1.
                            let worst = heap.peek().unwrap().distance;
                            if dist < worst {
                                heap.pop();
                                heap.push(ScoredCandidate {
                                    distance: dist,
                                    id,
                                    metadata: meta,
                                    payload,
                                });
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Drain heap; it pops in descending distance order (max-heap), so
        // reverse to produce best-first (ascending distance) output.
        let mut output: Vec<SearchResult> = heap
            .into_sorted_vec()
            .into_iter()
            .map(|c| (c.id, c.distance, c.metadata, c.payload))
            .collect();
        // `into_sorted_vec` already returns ascending order for a max-heap;
        // verify and ensure correctness regardless of future stdlib changes.
        output.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(output)
    }

    async fn search_u64(
        &self,
        vector: &[f64],
        filter: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<hyperspace_core::SearchResultU64>, String> {
        let top_k = params.top_k.max(1);
        let shard_top_k = (top_k + 6).max(top_k * 2);
        let shard_ef = params.ef_search.max(shard_top_k);

        let shard_params = SearchParams {
            top_k: shard_top_k,
            ef_search: shard_ef,
            hybrid_query: params.hybrid_query.clone(),
            hybrid_alpha: params.hybrid_alpha,
            component_weights: params.component_weights.clone(),
            use_wasserstein: params.use_wasserstein,
            bm25_options: params.bm25_options.clone(),
            fusion_method: params.fusion_method.clone(),
            mrl_dimension: params.mrl_dimension,
            include_payload: params.include_payload,
            use_wave: params.use_wave,
        };

        let futures: Vec<_> = self
            .shards
            .iter()
            .enumerate()
            .map(|(shard_idx, shard)| {
                let shard_ref = Arc::clone(shard);
                let sp = shard_params.clone();
                async move {
                    let res = shard_ref.search(vector, filter, complex_filters, &sp).await;
                    (shard_idx, res)
                }
            })
            .collect();

        let shard_results = futures::future::join_all(futures).await;

        let mut heap: BinaryHeap<ScoredCandidate64> = BinaryHeap::with_capacity(top_k + 1);

        for (shard_idx, res) in shard_results {
            match res {
                Ok(items) => {
                    for (local_id, dist, meta, payload) in items {
                        let global_id = self.compose_id64(shard_idx, local_id);
                        if heap.len() < top_k {
                            heap.push(ScoredCandidate64 {
                                distance: dist,
                                id: global_id,
                                metadata: meta,
                                payload,
                            });
                        } else {
                            let worst = heap.peek().unwrap().distance;
                            if dist < worst {
                                heap.pop();
                                heap.push(ScoredCandidate64 {
                                    distance: dist,
                                    id: global_id,
                                    metadata: meta,
                                    payload,
                                });
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        let mut output: Vec<hyperspace_core::SearchResultU64> = heap
            .into_sorted_vec()
            .into_iter()
            .map(|c| (c.id, c.distance, c.metadata, c.payload))
            .collect();
        output.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(output)
    }

    fn count(&self) -> usize {
        self.shards.iter().map(|s| s.count()).sum()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn metric_name(&self) -> &'static str {
        M::name()
    }

    fn state_hash(&self) -> u64 {
        self.shards.iter().fold(0u64, |acc, s| acc ^ s.state_hash())
    }

    fn buckets(&self) -> Vec<u64> {
        let mut result = vec![0u64; 256];
        for s in &self.shards {
            for (i, b) in s.buckets().into_iter().enumerate() {
                result[i] ^= b;
            }
        }
        result
    }

    fn queue_size(&self) -> u64 {
        self.shards.iter().map(|s| s.queue_size()).sum()
    }

    fn write_buffer_size(&self) -> usize {
        self.shards.iter().map(|s| s.write_buffer_size()).sum()
    }

    fn quantization_mode(&self) -> hyperspace_core::QuantizationMode {
        self.mode
    }

    fn get_usage(&self) -> CollectionUsage {
        let mut total = CollectionUsage {
            disk_usage_bytes: 0,
            ram_usage_bytes: 0,
            active_indexing_tasks: 0,
        };
        for s in &self.shards {
            let u = s.get_usage();
            total.disk_usage_bytes += u.disk_usage_bytes;
            total.ram_usage_bytes += u.ram_usage_bytes;
            total.active_indexing_tasks += u.active_indexing_tasks;
        }
        total
    }

    fn get_hnsw_config(&self) -> HnswConfig {
        self.shards
            .first()
            .map(|s| s.get_hnsw_config())
            .unwrap_or(HnswConfig {
                ef_search: 64,
                ef_construction: 64,
                m: 16,
            })
    }

    fn update_config(&self, config: CollectionConfigUpdate) -> Result<(), String> {
        for s in &self.shards {
            s.update_config(config.clone())?;
        }
        Ok(())
    }

    fn create_snapshot(&self) -> Result<(), String> {
        for s in &self.shards {
            s.create_snapshot()?;
        }
        if let Some(ref wal) = self.wal {
            if let Ok(mut wal_guard) = wal.try_lock() {
                let _ = wal_guard.sync();
            }
        }
        Ok(())
    }

    async fn optimize(&self) -> Result<(), String> {
        let mut futures = Vec::with_capacity(self.num_shards);
        for s in &self.shards {
            let s_ref = Arc::clone(s);
            futures.push(async move { s_ref.optimize().await });
        }
        for res in futures::future::join_all(futures).await {
            res?;
        }
        Ok(())
    }

    async fn optimize_with_filter(&self, filter: Option<VacuumFilterQuery>) -> Result<(), String> {
        let mut futures = Vec::with_capacity(self.num_shards);
        for s in &self.shards {
            let s_ref = Arc::clone(s);
            let f = filter.clone();
            futures.push(async move { s_ref.optimize_with_filter(f).await });
        }
        for res in futures::future::join_all(futures).await {
            res?;
        }
        Ok(())
    }

    fn peek(&self, limit: usize, offset: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let mut results = Vec::new();
        for s in &self.shards {
            let items = s.peek(limit, 0);
            results.extend(items);
            if results.len() >= limit + offset {
                break;
            }
        }
        results.into_iter().skip(offset).take(limit).collect()
    }

    /// Delta-sync bucket extraction. The default trait impl calls `peek(count, 0)` which on a
    /// sharded collection over-fetches `count` vectors from *every* shard; delegating to each
    /// shard keeps the work proportional to the shard size.
    fn peek_buckets(
        &self,
        bucket_indices: &[u32],
    ) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let mut out = Vec::new();
        for s in &self.shards {
            out.extend(s.peek_buckets(bucket_indices));
        }
        out
    }

    fn get_points(&self, ids: &[u32]) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let mut shard_buckets: Vec<Vec<(usize, u32)>> = vec![Vec::new(); self.num_shards];
        for (pos, &id) in ids.iter().enumerate() {
            let shard_idx = self.get_shard_index(id);
            shard_buckets[shard_idx].push((pos, id));
        }

        let mut results = vec![None; ids.len()];
        for (shard_idx, reqs) in shard_buckets.into_iter().enumerate() {
            if reqs.is_empty() {
                continue;
            }
            let local_ids: Vec<u32> = reqs.iter().map(|(_, lid)| *lid).collect();
            let points = self.shards[shard_idx].get_points(&local_ids);
            let map: HashMap<u32, (Vec<f64>, HashMap<String, String>)> = points
                .into_iter()
                .map(|(lid, vec, meta)| (lid, (vec, meta)))
                .collect();
            for (pos, lid) in reqs {
                if let Some((vec, meta)) = map.get(&lid) {
                    results[pos] = Some((lid, vec.clone(), meta.clone()));
                }
            }
        }

        results.into_iter().flatten().collect()
    }

    fn get_points_u64(&self, ids: &[u64]) -> Vec<(u64, Vec<f64>, HashMap<String, String>)> {
        let mut shard_buckets: Vec<Vec<(usize, u32)>> = vec![Vec::new(); self.num_shards];
        for (pos, &id) in ids.iter().enumerate() {
            if let Ok((shard_idx, local_id)) = self.decompose_id64(id) {
                shard_buckets[shard_idx].push((pos, local_id));
            }
        }

        let mut results = vec![None; ids.len()];
        for (shard_idx, reqs) in shard_buckets.into_iter().enumerate() {
            if reqs.is_empty() {
                continue;
            }
            let local_ids: Vec<u32> = reqs.iter().map(|(_, lid)| *lid).collect();
            let points = self.shards[shard_idx].get_points(&local_ids);
            let map: HashMap<u32, (Vec<f64>, HashMap<String, String>)> = points
                .into_iter()
                .map(|(lid, vec, meta)| (lid, (vec, meta)))
                .collect();
            for (pos, lid) in reqs {
                if let Some((vec, meta)) = map.get(&lid) {
                    let global_id = ids[pos];
                    results[pos] = Some((global_id, vec.clone(), meta.clone()));
                }
            }
        }

        results.into_iter().flatten().collect()
    }

    fn graph_neighbors(&self, id: u32, layer: usize, limit: usize) -> Result<Vec<u32>, String> {
        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx].graph_neighbors(id, layer, limit)
    }

    fn graph_neighbor_distances(
        &self,
        source_id: u32,
        neighbor_ids: &[u32],
    ) -> Result<Vec<f64>, String> {
        let shard_idx = self.get_shard_index(source_id);
        self.shards[shard_idx].graph_neighbor_distances(source_id, neighbor_ids)
    }

    fn graph_traverse(
        &self,
        start_id: u32,
        layer: usize,
        max_depth: usize,
        max_nodes: usize,
        breadth_limit: usize,
    ) -> Result<Vec<u32>, String> {
        let shard_idx = self.get_shard_index(start_id);
        self.shards[shard_idx].graph_traverse(start_id, layer, max_depth, max_nodes, breadth_limit)
    }

    fn graph_clusters(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Result<Vec<Vec<u32>>, String> {
        let mut all_clusters = Vec::new();
        for s in &self.shards {
            if let Ok(clusters) = s.graph_clusters(layer, min_cluster_size, max_clusters, max_nodes)
            {
                all_clusters.extend(clusters);
            }
        }
        all_clusters.truncate(max_clusters);
        Ok(all_clusters)
    }

    fn compact_semantic_chunks(&self, target_partitions: usize) -> Result<String, String> {
        let mut results = Vec::new();
        for (idx, s) in self.shards.iter().enumerate() {
            if let Ok(msg) = s.compact_semantic_chunks(target_partitions) {
                results.push(format!("Shard {idx}: {msg}"));
            }
        }
        Ok(results.join("; "))
    }

    async fn insert_payload(&self, id: u32, payload: Vec<u8>) -> Result<(), String> {
        let shard_idx = self.get_shard_index(id);
        self.shards[shard_idx].insert_payload(id, payload).await
    }

    async fn insert_payload_batch(&self, entries: Vec<(u32, Vec<u8>)>) -> Result<(), String> {
        let mut buckets: Vec<Vec<(u32, Vec<u8>)>> = vec![Vec::new(); self.num_shards];
        for item in entries {
            let shard_idx = self.get_shard_index(item.0);
            buckets[shard_idx].push(item);
        }

        let mut tasks = Vec::with_capacity(self.num_shards);
        for (shard_idx, batch) in buckets.into_iter().enumerate() {
            if !batch.is_empty() {
                let shard = Arc::clone(&self.shards[shard_idx]);
                tasks.push(tokio::spawn(async move {
                    shard.insert_payload_batch(batch).await
                }));
            }
        }

        for task in tasks {
            task.await
                .map_err(|e| format!("Shard insert payload task panicked: {e}"))??;
        }
        Ok(())
    }

    async fn fetch_payloads(&self, ids: &[u32]) -> Vec<Option<Vec<u8>>> {
        let mut shard_ids: Vec<Vec<(usize, u32)>> = vec![Vec::new(); self.num_shards];
        for (orig_idx, &id) in ids.iter().enumerate() {
            let s_idx = self.get_shard_index(id);
            shard_ids[s_idx].push((orig_idx, id));
        }

        let mut results = vec![None; ids.len()];
        let mut tasks = Vec::with_capacity(self.num_shards);
        for (shard_idx, items) in shard_ids.into_iter().enumerate() {
            if !items.is_empty() {
                let shard = Arc::clone(&self.shards[shard_idx]);
                tasks.push(tokio::spawn(async move {
                    let sub_ids: Vec<u32> = items.iter().map(|(_, id)| *id).collect();
                    let payloads = shard.fetch_payloads(&sub_ids).await;
                    (items, payloads)
                }));
            }
        }

        for task in tasks {
            if let Ok((items, payloads)) = task.await {
                for ((orig_idx, _), payload) in items.into_iter().zip(payloads) {
                    results[orig_idx] = payload;
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperspace_core::EuclideanMetric;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sharded_collection_insert_and_scatter_gather_search() {
        let dir = tempdir().unwrap();
        let (tx, _) = broadcast::channel(128);
        let schema = CollectionSchema {
            components: vec![hyperspace_proto::hyperspace::VectorComponent {
                name: "default".to_string(),
                metric: "euclidean".to_string(),
                full_dimension: 4,
                weight: 1.0,
            }],
            cascade_pipeline: vec![],
        };

        // Create sharded collection with 4 core shards
        let collection = ShardedCollection::<EuclideanMetric>::new(
            "test_sharded".to_string(),
            "node_1".to_string(),
            dir.path().to_path_buf(),
            hyperspace_core::QuantizationMode::None,
            tx,
            4,
            schema,
            Some(4),
        )
        .await
        .unwrap();

        assert_eq!(collection.num_shards, 4);

        // Insert vectors across different IDs
        let v0 = vec![1.0, 0.0, 0.0, 0.0];
        let v1 = vec![0.0, 1.0, 0.0, 0.0];
        let v2 = vec![0.0, 0.0, 1.0, 0.0];
        let v3 = vec![0.0, 0.0, 0.0, 1.0];

        collection
            .insert(&v0, 0, HashMap::new(), 1, Durability::Default)
            .await
            .unwrap();
        collection
            .insert(&v1, 1, HashMap::new(), 2, Durability::Default)
            .await
            .unwrap();
        collection
            .insert(&v2, 2, HashMap::new(), 3, Durability::Default)
            .await
            .unwrap();
        collection
            .insert(&v3, 3, HashMap::new(), 4, Durability::Default)
            .await
            .unwrap();

        assert_eq!(collection.count(), 4);

        // Search near v0
        let query = vec![0.9, 0.1, 0.0, 0.0];
        let params = SearchParams {
            top_k: 2,
            ef_search: 16,
            ..Default::default()
        };

        let results = collection
            .search(&query, &HashMap::new(), &[], &params)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        // ID 0 must be closest to query [0.9, 0.1, 0, 0]
        assert_eq!(results[0].0, 0);

        // Test delete: soft delete marks the node as deleted in index
        collection.delete(1).unwrap();
        let results_after = collection
            .search(&[0.0, 1.0, 0.0, 0.0], &HashMap::new(), &[], &params)
            .await
            .unwrap();
        assert!(results_after.iter().all(|r| r.0 != 1));
    }
}
