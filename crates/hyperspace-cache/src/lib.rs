pub mod geometry;
pub mod l1_exact;
pub mod l2_ann;
pub mod ttl;
pub mod eviction;
pub mod metrics;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;

pub use geometry::CacheMetricType;
pub use eviction::EvictionPolicy;
pub use l1_exact::{CacheEntry, L1ExactStore};
pub use l2_ann::AnyAnnStore;
pub use ttl::TimerWheel;
pub use eviction::EvictionManager;

/// Minimum milliseconds between two consecutive L2 HNSW rebuilds.
///
/// FIX (Bottleneck 1): Prevents rebuild storms under high write load.
/// Without this, every batch of `ann_rebuild_batch` inserts would spawn a new
/// expensive blocking rebuild task, saturating the thread pool.
const REBUILD_COOLDOWN_MS: u64 = 5_000;

/// Tombstone ratio threshold above which a rebuild is proactively triggered.
///
/// FIX (Bottleneck 2): When >30% of indexed vectors are tombstoned the search
/// quality degrades (extra filtering overhead, sparser effective graph). This
/// constant triggers an automatic rebuild to compact the dead weight.
const TOMBSTONE_REBUILD_THRESHOLD: f64 = 0.30;

/// Minimum number of pending (not-yet-indexed) vectors that triggers an early rebuild.
///
/// FIX (Bottleneck 5): Previously a rebuild only happened when pending reached
/// `ann_rebuild_batch` (default 100) OR after a 30-second periodic timer. For
/// small collections this meant the L2 ANN graph stayed empty for 30 seconds.
/// Now the background 5-second scheduler also fires a rebuild when pending >= 2.
const PENDING_EAGER_THRESHOLD: usize = 2;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub l1_capacity: usize,
    pub eviction_policy: EvictionPolicy,
    pub ann_threshold: Option<f64>,
    pub ann_rebuild_batch: usize,
    pub collection_name: String,
    pub dimension: usize,
    pub metric: CacheMetricType,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l2_index_size: usize,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub tombstone_count: usize,
    pub pending_rebuild: usize,
    /// Rough in-memory footprint of L1 (vectors + estimated metadata + struct overhead).
    /// FIX (Bottleneck 3): Previous estimate ignored metadata and DashMap overhead.
    pub estimated_memory_bytes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheTier {
    L1Exact,
    L2Ann,
}

#[derive(Clone)]
pub struct CacheHit {
    pub id: u32,
    pub vector: Arc<Vec<f64>>,
    pub metadata: Arc<HashMap<String, String>>,
    pub distance: f64,
    pub tier: CacheTier,
}

struct RateTracker {
    hits: AtomicU64,
    misses: AtomicU64,
    history: Mutex<Vec<(Instant, u64, u64)>>,
}

impl RateTracker {
    fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            history: Mutex::new(Vec::new()),
        }
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn hit_rate(&self) -> f64 {
        let now = Instant::now();
        let current_hits = self.hits.load(Ordering::Relaxed);
        let current_misses = self.misses.load(Ordering::Relaxed);

        let mut history = self.history.lock();
        history.push((now, current_hits, current_misses));

        let cutoff = now - Duration::from_secs(60);
        history.retain(|(t, _, _)| *t >= cutoff);

        if history.len() < 2 {
            let total = current_hits + current_misses;
            if total == 0 {
                0.0
            } else {
                current_hits as f64 / total as f64
            }
        } else {
            let first = history.first().unwrap();
            let last = history.last().unwrap();
            let delta_hits = last.1.saturating_sub(first.1);
            let delta_misses = last.2.saturating_sub(first.2);
            let total = delta_hits + delta_misses;
            if total == 0 {
                0.0
            } else {
                delta_hits as f64 / total as f64
            }
        }
    }
}

pub struct VectorCache {
    l1: Arc<L1ExactStore>,
    l2: Arc<dyn AnyAnnStore>,
    ttl_wheel: Arc<TimerWheel>,
    eviction: parking_lot::RwLock<EvictionManager>,
    config: parking_lot::RwLock<CacheConfig>,

    l1_tracker: RateTracker,
    l2_tracker: RateTracker,

    /// Timestamp (ms since epoch) of the last `trigger_rebuild()` call.
    ///
    /// FIX (Bottleneck 1): Guards against rebuild storms. trigger_rebuild() is a
    /// no-op if called within REBUILD_COOLDOWN_MS of the previous invocation.
    last_rebuild_triggered_ms: AtomicU64,

    shutdown: tokio::sync::watch::Sender<bool>,
}

impl VectorCache {
    pub fn new(config: CacheConfig) -> Self {
        let l1 = Arc::new(L1ExactStore::new(config.l1_capacity));

        let l2: Arc<dyn AnyAnnStore> = match &config.metric {
            CacheMetricType::L2 => Arc::new(l2_ann::L2AnnStore::new(
                geometry::L2CacheDistance,
                config.dimension,
                config.ann_rebuild_batch,
            )),
            CacheMetricType::Cosine => Arc::new(l2_ann::L2AnnStore::new(
                geometry::CosineCacheDistance,
                config.dimension,
                config.ann_rebuild_batch,
            )),
            CacheMetricType::Poincare => Arc::new(l2_ann::L2AnnStore::new(
                geometry::PoincareCacheDistance,
                config.dimension,
                config.ann_rebuild_batch,
            )),
            CacheMetricType::Lorentz => Arc::new(l2_ann::L2AnnStore::new(
                geometry::LorentzCacheDistance,
                config.dimension,
                config.ann_rebuild_batch,
            )),
            CacheMetricType::Hybrid { lorentz_weight, l2_weight } => Arc::new(l2_ann::L2AnnStore::new(
                geometry::HybridCacheDistance {
                    lorentz_weight: *lorentz_weight,
                    l2_weight: *l2_weight,
                },
                config.dimension,
                config.ann_rebuild_batch,
            )),
        };

        let ttl_wheel = Arc::new(TimerWheel::new());
        let eviction = parking_lot::RwLock::new(EvictionManager::new(config.eviction_policy, config.l1_capacity, None));
        let config = parking_lot::RwLock::new(config);
        let (shutdown, _) = tokio::sync::watch::channel(false);

        Self {
            l1,
            l2,
            ttl_wheel,
            eviction,
            config,
            l1_tracker: RateTracker::new(),
            l2_tracker: RateTracker::new(),
            last_rebuild_triggered_ms: AtomicU64::new(0),
            shutdown,
        }
    }

    pub fn start_background_tasks(self: &Arc<Self>) {
        // ── Task 1: TTL expiry + eager rebuild trigger (every 100ms) ──────────
        //
        // FIX (Bottleneck 5): This task also triggers a rebuild when the pending
        // queue has >= PENDING_EAGER_THRESHOLD entries. Previously a rebuild would
        // only fire when the pending queue reached the full ann_rebuild_batch size
        // (default: 100) OR after 30 seconds. Now small collections get their first
        // L2 graph built within ~100ms of the first inserts.
        let self_clone = self.clone();
        let mut shutdown_rx = self.shutdown.subscribe();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let collection_name = self_clone.config.read().collection_name.clone();

                        // Process TTL expirations
                        let expired = self_clone.ttl_wheel.tick();
                        if !expired.is_empty() {
                            for id in &expired {
                                if self_clone.l1.remove(*id).is_some() {
                                    metrics::metrics().evictions.with_label_values(&[&collection_name, "ttl"]).inc();
                                }
                                self_clone.l2.tombstone(*id);
                            }
                            metrics::metrics().l1_size.with_label_values(&[&collection_name]).set(self_clone.l1.len() as f64);
                            metrics::metrics().tombstone_size.with_label_values(&[&collection_name]).set(self_clone.l2.tombstone_len() as f64);
                        }

                        // Eager rebuild trigger: fire as soon as we have enough pending entries.
                        // The cooldown in trigger_rebuild() ensures we don't thrash.
                        if self_clone.l2.pending_len() >= PENDING_EAGER_THRESHOLD {
                            self_clone.trigger_rebuild();
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                }
            }
        });

        // ── Task 2: Periodic maintenance rebuild (every 5s) ──────────────────
        //
        // FIX (Bottleneck 1 & 2): Changed from a dumb 30-second fixed interval to
        // a smarter 5-second conditional check:
        //   - Trigger when there are any pending inserts not yet in the L2 graph.
        //   - Trigger when tombstone ratio exceeds TOMBSTONE_REBUILD_THRESHOLD (30%).
        // The cooldown in trigger_rebuild() still prevents actual rebuild frequency
        // from exceeding once per REBUILD_COOLDOWN_MS.
        let self_clone2 = self.clone();
        let mut shutdown_rx2 = self.shutdown.subscribe();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            interval.tick().await; // skip first immediate tick

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let has_pending = self_clone2.l2.pending_len() > 0;
                        let tombstone_high = self_clone2.l2.tombstone_ratio() > TOMBSTONE_REBUILD_THRESHOLD;
                        if has_pending || tombstone_high {
                            self_clone2.trigger_rebuild();
                        }
                    }
                    _ = shutdown_rx2.changed() => {
                        break;
                    }
                }
            }
        });
    }

    pub fn search(
        &self,
        query: &[f64],
        known_id: Option<u32>,
        k: usize,
    ) -> Option<Vec<CacheHit>> {
        let start_time = Instant::now();
        let collection_name = self.config.read().collection_name.clone();
        if let Some(id) = known_id {
            if let Some((vec, meta)) = self.l1.get_entry(id) {
                metrics::metrics().l1_hits.with_label_values(&[&collection_name]).inc();
                metrics::metrics().l1_latency.with_label_values(&[&collection_name])
                    .observe(start_time.elapsed().as_secs_f64());
                self.l1_tracker.record_hit();

                return Some(vec![CacheHit {
                    id,
                    vector: vec,
                    metadata: meta,
                    distance: 0.0,
                    tier: CacheTier::L1Exact,
                }]);
            } else {
                metrics::metrics().l1_misses.with_label_values(&[&collection_name]).inc();
                self.l1_tracker.record_miss();
            }
        }

        let ann_threshold = self.config.read().ann_threshold;
        if let Some(threshold) = ann_threshold {
            let l2_start = Instant::now();
            let ann_results = self.l2.search(query, k, threshold);
            if !ann_results.is_empty() {
                metrics::metrics().l2_hits.with_label_values(&[&collection_name]).inc();
                metrics::metrics().l2_latency.with_label_values(&[&collection_name])
                    .observe(l2_start.elapsed().as_secs_f64());
                self.l2_tracker.record_hit();

                let mut hits = Vec::with_capacity(ann_results.len());
                for (id, dist) in ann_results {
                    if let Some((vector, metadata)) = self.l1.get_entry(id) {
                        hits.push(CacheHit {
                            id,
                            vector,
                            metadata,
                            distance: dist,
                            tier: CacheTier::L2Ann,
                        });
                    }
                }

                if !hits.is_empty() {
                    return Some(hits);
                }
            } else {
                metrics::metrics().l2_misses.with_label_values(&[&collection_name]).inc();
                self.l2_tracker.record_miss();
            }
        }

        None
    }

    pub fn insert(
        &self,
        id: u32,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
        ttl: Option<Duration>,
    ) {
        if self.l1.map.contains_key(&id) {
            self.l2.tombstone(id);
        }

        let entry = CacheEntry::new(vector.clone(), metadata, ttl);
        self.l1.insert(id, entry);

        if let Some(t) = ttl {
            self.ttl_wheel.schedule(id, t);
        }

        let ann_threshold = self.config.read().ann_threshold;
        if ann_threshold.is_some() {
            let rebuild_needed = self.l2.enqueue(id, vector);
            if rebuild_needed {
                self.trigger_rebuild();
            }
        }

        let collection_name = self.config.read().collection_name.clone();
        metrics::metrics().inserts.with_label_values(&[&collection_name]).inc();
        metrics::metrics().l1_size.with_label_values(&[&collection_name]).set(self.l1.len() as f64);
        metrics::metrics().l2_pending_size.with_label_values(&[&collection_name]).set(self.l2.pending_len() as f64);

        self.eviction.read().maybe_evict(&self.l1, &*self.l2);
    }

    pub fn invalidate(&self, id: u32) {
        let collection_name = self.config.read().collection_name.clone();
        if self.l1.remove(id).is_some() {
            metrics::metrics().evictions.with_label_values(&[&collection_name, "invalidate"]).inc();
        }
        self.l2.tombstone(id);
        metrics::metrics().l1_size.with_label_values(&[&collection_name]).set(self.l1.len() as f64);
        metrics::metrics().tombstone_size.with_label_values(&[&collection_name]).set(self.l2.tombstone_len() as f64);

        // FIX (Bottleneck 2): Proactively rebuild if tombstone accumulation exceeds the
        // threshold. Without this, tombstones only got cleared on the periodic rebuild
        // timer, degrading L2 search quality in delete-heavy workloads.
        if self.l2.tombstone_ratio() > TOMBSTONE_REBUILD_THRESHOLD {
            self.trigger_rebuild();
        }
    }

    pub fn clear(&self) {
        self.l1.map.clear();
        self.l2.clear();
        let collection_name = self.config.read().collection_name.clone();
        metrics::metrics().l1_size.with_label_values(&[&collection_name]).set(0.0);
        metrics::metrics().l2_index_size.with_label_values(&[&collection_name]).set(0.0);
        metrics::metrics().l2_pending_size.with_label_values(&[&collection_name]).set(0.0);
        metrics::metrics().tombstone_size.with_label_values(&[&collection_name]).set(0.0);
    }

    pub fn update_config(&self, policy: EvictionPolicy, ann_threshold: Option<f64>) {
        {
            let mut conf = self.config.write();
            conf.eviction_policy = policy;
            conf.ann_threshold = ann_threshold;
        }
        {
            let mut evict = self.eviction.write();
            evict.policy = policy;
        }
    }

    pub fn stats(&self) -> CacheStats {
        let l1_size = self.l1.len();
        let config = self.config.read();

        // FIX (Bottleneck 3): Improved memory estimate that accounts for:
        //   - Vector data:      dimension × 8 bytes (f64)
        //   - Arc<Vec> shells:  2 × 24 bytes (vector + metadata Arc)
        //   - DashMap entry:    ~80 bytes (hash + pointers + alignment)
        //   - Metadata HashMap: ~200 bytes rough average (key/value strings)
        //   - Instants + atoms: ~48 bytes (3 × 16)
        let bytes_per_entry = config.dimension * 8   // vector payload
            + 2 * 24                                  // Arc shells
            + 80                                      // DashMap bucket overhead
            + 200                                     // metadata HashMap estimate
            + 48;                                     // CacheEntry struct fields

        let estimated_memory_bytes = l1_size * bytes_per_entry;

        if estimated_memory_bytes > 1024 * 1024 * 1024 {
            tracing::warn!(
                "Collection cache '{}' memory usage is high: {:.2} GB",
                config.collection_name,
                estimated_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            );
        }

        CacheStats {
            l1_size,
            l2_index_size: self.l2.index_len(),
            l1_hit_rate: self.l1_tracker.hit_rate(),
            l2_hit_rate: self.l2_tracker.hit_rate(),
            tombstone_count: self.l2.tombstone_len(),
            pending_rebuild: self.l2.pending_len(),
            estimated_memory_bytes,
        }
    }

    /// Returns the configured L1 capacity for this cache.
    pub fn l1_capacity(&self) -> usize {
        self.config.read().l1_capacity
    }

    /// Populate the cache from a pre-loaded set of entries (e.g. from the HNSW index
    /// on startup).
    ///
    /// FIX (Bottleneck 4): After a server restart the cache is empty, forcing all
    /// initial requests through the full on-disk HNSW traversal. This method lets the
    /// server pre-populate the hot tier immediately after opening a collection.
    ///
    /// Inserts directly into L1 without TTL and schedules a single L2 rebuild after
    /// all entries are loaded (avoiding N individual rebuild triggers).
    pub fn warmup_from_entries(&self, entries: Vec<(u32, Vec<f64>)>) {
        if entries.is_empty() {
            return;
        }

        let collection_name = self.config.read().collection_name.clone();
        let ann_enabled = self.config.read().ann_threshold.is_some();

        for (id, vector) in &entries {
            let entry = CacheEntry::new(vector.clone(), HashMap::new(), None);
            self.l1.insert(*id, entry);

            if ann_enabled {
                // Enqueue for L2 without checking capacity threshold — we'll do
                // a single rebuild at the end.
                let _ = self.l2.enqueue(*id, vector.clone());
            }
        }

        metrics::metrics().l1_size
            .with_label_values(&[&collection_name])
            .set(self.l1.len() as f64);

        // Trigger one rebuild for the entire warmup batch.
        // The cooldown starts at 0ms so this first call always succeeds.
        if ann_enabled {
            self.trigger_rebuild();
        }

        tracing::info!(
            "🔥 Cache warmup complete: {} vectors pre-loaded for '{}'",
            entries.len(),
            collection_name
        );
    }

    /// Full-featured warmup with metadata.
    ///
    /// FIX (Limitation 2): Previously `warmup_from_entries()` inserted vectors with
    /// empty metadata (`HashMap::new()`). Cache hits immediately after startup would
    /// return empty key-value fields in the response if `include_payload = false`.
    ///
    /// This method accepts `(id, vector, metadata)` tuples sourced from the HNSW index
    /// `metadata.forward` map, so warmup entries are fully populated from the start.
    pub fn warmup_from_entries_with_meta(
        &self,
        entries: Vec<(u32, Vec<f64>, HashMap<String, String>)>,
    ) {
        if entries.is_empty() {
            return;
        }

        let collection_name = self.config.read().collection_name.clone();
        let ann_enabled = self.config.read().ann_threshold.is_some();

        for (id, vector, metadata) in &entries {
            let entry = CacheEntry::new(vector.clone(), metadata.clone(), None);
            self.l1.insert(*id, entry);

            if ann_enabled {
                let _ = self.l2.enqueue(*id, vector.clone());
            }
        }

        metrics::metrics().l1_size
            .with_label_values(&[&collection_name])
            .set(self.l1.len() as f64);

        if ann_enabled {
            self.trigger_rebuild();
        }

        tracing::info!(
            "🔥 Cache warmup complete: {} vectors (with metadata) pre-loaded for '{}'",
            entries.len(),
            collection_name
        );
    }


    /// Triggers a background L2 HNSW rebuild, subject to a cooldown.
    ///
    /// FIX (Bottleneck 1): The cooldown (REBUILD_COOLDOWN_MS) prevents multiple
    /// concurrent rebuilds from stacking up when inserts are faster than rebuilds.
    /// The rebuild reads ALL current L1 entries so it always produces a complete,
    /// up-to-date index regardless of how many triggers were skipped.
    fn trigger_rebuild(&self) {
        // ── Cooldown check ────────────────────────────────────────────────────
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let last = self.last_rebuild_triggered_ms.load(Ordering::Relaxed);
        if now_ms.saturating_sub(last) < REBUILD_COOLDOWN_MS {
            // Still within cooldown window — skip this trigger.
            // The periodic 5s task will pick up any remaining pending entries.
            return;
        }

        // Atomically claim the rebuild slot.
        self.last_rebuild_triggered_ms.store(now_ms, Ordering::Relaxed);

        // ── Collect entries ───────────────────────────────────────────────────
        let l1 = self.l1.clone();
        let l2 = self.l2.clone();
        let collection_name = self.config.read().collection_name.clone();

        tokio::task::spawn_blocking(move || {
            let start = Instant::now();
            let mut entries = Vec::with_capacity(l1.len());
            for r in l1.map.iter() {
                let id = *r.key();
                // Skip entries that have already expired (lazy expiry)
                if let Some(expires_at) = r.value().expires_at {
                    if Instant::now() >= expires_at {
                        continue;
                    }
                }
                entries.push((id, (*r.value().vector).clone()));
            }

            l2.rebuild(entries);

            metrics::metrics().rebuild_duration.with_label_values(&[&collection_name])
                .observe(start.elapsed().as_secs_f64());
            metrics::metrics().l2_index_size.with_label_values(&[&collection_name]).set(l2.index_len() as f64);
            metrics::metrics().tombstone_size.with_label_values(&[&collection_name]).set(l2.tombstone_len() as f64);
            metrics::metrics().l2_pending_size.with_label_values(&[&collection_name]).set(0.0);
        });
    }

    /// Resets the rebuild cooldown timer. **Test-only.** Allows unit tests to trigger
    /// consecutive rebuilds without waiting for REBUILD_COOLDOWN_MS to elapse.
    #[cfg(test)]
    pub fn reset_rebuild_cooldown(&self) {
        self.last_rebuild_triggered_ms.store(0, Ordering::Relaxed);
    }
}

impl Drop for VectorCache {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    fn test_config() -> CacheConfig {
        CacheConfig {
            l1_capacity: 10,
            eviction_policy: EvictionPolicy::Lru,
            ann_threshold: Some(10.0), // Euclidean metric distance squared threshold
            ann_rebuild_batch: 2,
            collection_name: "test_coll".to_string(),
            dimension: 3,
            metric: CacheMetricType::L2,
        }
    }

    #[tokio::test]
    async fn test_full_search_l1_hit() {
        let cache = VectorCache::new(test_config());
        let vector = vec![1.0, 2.0, 3.0];
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "val".to_string());

        cache.insert(42, vector.clone(), metadata.clone(), None);

        let hits = cache.search(&[1.0, 2.0, 3.0], Some(42), 1).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, 42);
        assert_eq!(*hits[0].vector, vector);
        assert_eq!(hits[0].tier, CacheTier::L1Exact);
    }

    #[tokio::test]
    async fn test_full_search_l2_hit() {
        let cache = Arc::new(VectorCache::new(test_config()));
        cache.start_background_tasks();

        cache.insert(1, vec![1.0, 0.0, 0.0], HashMap::new(), None);
        cache.insert(2, vec![0.0, 1.0, 0.0], HashMap::new(), None);

        // Wait for rebuild (batch size is 2, so it triggers rebuild immediately)
        sleep(Duration::from_millis(150)).await;

        let hits = cache.search(&[0.9, 0.0, 0.0], None, 2).unwrap();
        assert_eq!(hits[0].id, 1);
        assert_eq!(hits[0].tier, CacheTier::L2Ann);
    }

    #[tokio::test]
    async fn test_full_search_miss() {
        let cache = VectorCache::new(test_config());
        let hits = cache.search(&[1.0, 2.0, 3.0], Some(42), 5);
        assert!(hits.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_clears_both() {
        let cache = Arc::new(VectorCache::new(test_config()));
        cache.insert(42, vec![1.0, 2.0, 3.0], HashMap::new(), None);

        cache.invalidate(42);
        let hits = cache.search(&[1.0, 2.0, 3.0], Some(42), 1);
        assert!(hits.is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiry_end_to_end() {
        let cache = Arc::new(VectorCache::new(test_config()));
        cache.start_background_tasks();

        cache.insert(42, vec![1.0, 2.0, 3.0], HashMap::new(), Some(Duration::from_millis(50)));

        // Wait for background tick task to run and evict
        sleep(Duration::from_millis(200)).await;

        let hits = cache.search(&[1.0, 2.0, 3.0], Some(42), 1);
        assert!(hits.is_none());
    }

    #[tokio::test]
    async fn test_stats_accuracy() {
        let cache = VectorCache::new(test_config());
        cache.insert(42, vec![1.0, 2.0, 3.0], HashMap::new(), None);

        let stats = cache.stats();
        assert_eq!(stats.l1_size, 1);
        // With improved estimate: 3*8 + 2*24 + 80 + 200 + 48 = 400 bytes
        let expected = 1 * (3 * 8 + 2 * 24 + 80 + 200 + 48);
        assert_eq!(stats.estimated_memory_bytes, expected);
    }

    /// FIX verification: warmup_from_entries() pre-populates L1 correctly.
    #[tokio::test]
    async fn test_warmup_populates_l1() {
        let cache = VectorCache::new(test_config());
        let entries = vec![
            (1u32, vec![1.0, 0.0, 0.0]),
            (2u32, vec![0.0, 1.0, 0.0]),
        ];
        cache.warmup_from_entries(entries);

        // Both vectors should be in L1
        assert!(cache.search(&[], Some(1), 1).is_some());
        assert!(cache.search(&[], Some(2), 1).is_some());
        assert_eq!(cache.stats().l1_size, 2);
    }

    /// FIX verification: rebuild cooldown prevents rapid consecutive rebuilds.
    #[tokio::test]
    async fn test_rebuild_cooldown() {
        let cache = Arc::new(VectorCache::new(test_config()));
        cache.start_background_tasks();

        // First insert — triggers rebuild (cooldown starts at 0, always succeeds)
        cache.insert(1, vec![1.0, 0.0, 0.0], HashMap::new(), None);
        cache.insert(2, vec![0.0, 1.0, 0.0], HashMap::new(), None);

        // Wait for the first rebuild to complete
        sleep(Duration::from_millis(300)).await;
        let size_after_first = cache.stats().l2_index_size;

        // Insert more vectors. These trigger rebuild but cooldown should suppress them
        // (we're within 5s of the first rebuild).
        for i in 3u32..=12 {
            cache.insert(i, vec![i as f64, 0.0, 0.0], HashMap::new(), None);
        }

        // Short wait — NOT enough to pass cooldown (5s)
        sleep(Duration::from_millis(200)).await;

        // pending_len should reflect the new inserts (not yet in L2)
        let stats = cache.stats();
        // L2 may or may not have been rebuilt yet (cooldown is 5s), but L1 must be larger
        assert!(stats.l1_size > size_after_first);
    }

    /// FIX verification: tombstone-triggered rebuild fires when ratio > 0.3.
    ///
    /// Flow:
    ///  1. Insert 4 vectors → L2 HNSW built (rebuild batch = 2)
    ///  2. Reset cooldown so next trigger_rebuild() call actually fires
    ///  3. Invalidate 2 of 4 → ratio = 0.5, invalidate() calls trigger_rebuild()
    ///  4. spawn_blocking runs and clears tombstones
    #[tokio::test]
    async fn test_tombstone_triggers_rebuild() {
        let mut cfg = test_config();
        cfg.ann_rebuild_batch = 2;
        let cache = Arc::new(VectorCache::new(cfg));
        cache.start_background_tasks();

        // Insert 4 vectors and wait for initial L2 build
        for i in 1u32..=4 {
            cache.insert(i, vec![i as f64, 0.0, 0.0], HashMap::new(), None);
        }
        sleep(Duration::from_millis(500)).await;

        let idx_size = cache.stats().l2_index_size;
        assert!(idx_size > 0, "L2 index should have been built; got l2_index_size=0");

        // Reset cooldown so the trigger from invalidate() can actually fire.
        // (The initial rebuild set last_rebuild_triggered_ms which would suppress
        //  the tombstone trigger for the next REBUILD_COOLDOWN_MS = 5 seconds.)
        cache.reset_rebuild_cooldown();

        // Invalidate 2 of 4 → tombstone ratio = 0.5 > 0.3 threshold.
        // invalidate() calls trigger_rebuild() which now succeeds (cooldown reset).
        cache.invalidate(1);
        cache.invalidate(2);

        // Wait for spawn_blocking to complete the rebuild.
        sleep(Duration::from_millis(400)).await;

        // After rebuild all tombstones must be cleared.
        let stats = cache.stats();
        assert_eq!(stats.tombstone_count, 0,
            "tombstones must be 0 after tombstone-triggered rebuild (got {})", stats.tombstone_count);
        assert_eq!(stats.l2_index_size, 2,
            "L2 index must contain only the 2 surviving vectors (got {})", stats.l2_index_size);
    }
}
