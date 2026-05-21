use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use arc_swap::ArcSwap;
use dashmap::DashSet;
use parking_lot::Mutex;
use instant_distance::{Builder, HnswMap, Search};
use crate::geometry::{CacheDistance, CacheVector};
use crate::eviction::AnnTombstone;

pub trait AnyAnnStore: AnnTombstone + Send + Sync + 'static {
    fn enqueue(&self, id: u32, vector: Vec<f64>) -> bool; // returns true if rebuild is needed
    fn rebuild(&self, all_entries: Vec<(u32, Vec<f64>)>);
    fn search(&self, query: &[f64], k: usize, threshold: f64) -> Vec<(u32, f64)>;
    fn pending_len(&self) -> usize;
    fn index_len(&self) -> usize;
    fn tombstone_len(&self) -> usize;
    fn clear(&self);

    /// Ratio of tombstones to indexed vectors.
    /// High values (>0.3) indicate a rebuild would significantly improve search quality.
    fn tombstone_ratio(&self) -> f64 {
        let idx_len = self.index_len();
        if idx_len == 0 {
            0.0
        } else {
            self.tombstone_len() as f64 / idx_len as f64
        }
    }
}

pub struct L2AnnStore<D: CacheDistance> {
    index: ArcSwap<Option<HnswMap<CacheVector, u32>>>,
    pending: Mutex<Vec<(u32, Vec<f64>)>>,
    pending_capacity: usize,
    distance: Arc<D>,
    tombstones: DashSet<u32>,
    pub dimension: usize,
    /// Timestamp (ms since UNIX epoch) of the last completed rebuild.
    /// Used by VectorCache to implement cooldown and metrics.
    pub last_rebuild_ms: AtomicU64,
    /// How many candidates the L2 HNSW graph explores per query.
    /// Higher values improve recall at the cost of latency.
    /// Configurable via `HYPERSPACE_CACHE_L2_EF_SEARCH` (default: 64).
    ef_search: usize,
}

impl<D: CacheDistance> L2AnnStore<D> {
    pub fn new(distance: D, dimension: usize, pending_capacity: usize) -> Self {
        let ef_search = std::env::var("HYPERSPACE_CACHE_L2_EF_SEARCH")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(64);
        Self {
            index: ArcSwap::new(Arc::new(None)),
            pending: Mutex::new(Vec::new()),
            pending_capacity,
            distance: Arc::new(distance),
            tombstones: DashSet::new(),
            dimension,
            last_rebuild_ms: AtomicU64::new(0),
            ef_search,
        }
    }

    /// Rebuilds the HNSW index from all provided entries.
    ///
    /// FIX (Bottleneck 1): Clears the pending queue after rebuild so `pending_len()`
    /// correctly returns 0 after the index is up to date. Previously the pending queue
    /// grew indefinitely because it was never cleared.
    ///
    /// FIX (Bottleneck 5): Records the rebuild timestamp so the VectorCache cooldown
    /// logic can track when the last rebuild finished.
    pub fn rebuild(&self, all_entries: Vec<(u32, Vec<f64>)>) {
        // Drain the pending queue. All entries were already inserted into L1 before
        // being enqueued here, so trigger_rebuild() (which reads from L1) already
        // included them in all_entries.
        {
            let mut pending = self.pending.lock();
            pending.clear();
        }

        if all_entries.is_empty() {
            self.index.store(Arc::new(None));
            self.tombstones.clear();
            self.record_rebuild_time();
            return;
        }

        let mut points = Vec::with_capacity(all_entries.len());
        let mut values = Vec::with_capacity(all_entries.len());
        let metric_type = self.distance.metric_type();

        for (id, coords) in all_entries {
            points.push(CacheVector {
                coords,
                id,
                metric_type: metric_type.clone(),
            });
            values.push(id);
        }

        // ef_search and ef_construction are baked into the HnswMap at build time.
        // They control search beam width and graph quality respectively.
        let builder = Builder::default()
            .ef_search(self.ef_search)
            .ef_construction(self.ef_search);
        let hnsw = builder.build(points, values);

        // Atomic swap: readers see either the old or new index, never a partial state.
        self.index.store(Arc::new(Some(hnsw)));
        // Clear tombstones — they are now baked into the new index structure
        // (deleted entries simply aren't present in the new graph).
        self.tombstones.clear();
        self.record_rebuild_time();
    }

    fn record_rebuild_time(&self) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_rebuild_ms.store(now_ms, Ordering::Relaxed);
    }

    pub fn search(&self, query: &[f64], k: usize, threshold: f64) -> Vec<(u32, f64)> {
        let query_point = CacheVector {
            coords: query.to_vec(),
            id: 0,
            metric_type: self.distance.metric_type(),
        };

        // Search uses ef_search baked into the HnswMap at rebuild time.
        let mut search = Search::default();
        if let Some(index) = &**self.index.load() {
            let mut results = Vec::new();
            for item in index.search(&query_point, &mut search) {
                let id = *item.value;
                let dist = item.distance as f64;

                if self.tombstones.contains(&id) {
                    continue;
                }
                if !self.distance.is_cache_hit(dist, threshold) {
                    continue;
                }

                results.push((id, dist));
                if results.len() >= k {
                    break;
                }
            }
            results
        } else {
            Vec::new()
        }
    }
}

impl<D: CacheDistance> AnnTombstone for L2AnnStore<D> {
    fn tombstone(&self, id: u32) {
        self.tombstones.insert(id);
    }
}

impl<D: CacheDistance> AnyAnnStore for L2AnnStore<D> {
    fn enqueue(&self, id: u32, vector: Vec<f64>) -> bool {
        let mut pending = self.pending.lock();
        pending.retain(|(p_id, _)| *p_id != id);
        pending.push((id, vector));
        pending.len() >= self.pending_capacity
    }

    fn rebuild(&self, all_entries: Vec<(u32, Vec<f64>)>) {
        self.rebuild(all_entries);
    }

    fn search(&self, query: &[f64], k: usize, threshold: f64) -> Vec<(u32, f64)> {
        self.search(query, k, threshold)
    }

    fn pending_len(&self) -> usize {
        self.pending.lock().len()
    }

    fn index_len(&self) -> usize {
        if let Some(index) = &**self.index.load() {
            index.values.len()
        } else {
            0
        }
    }

    fn tombstone_len(&self) -> usize {
        self.tombstones.len()
    }

    fn clear(&self) {
        self.index.store(Arc::new(None));
        self.pending.lock().clear();
        self.tombstones.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::L2CacheDistance;

    #[test]
    fn test_ann_search_empty() {
        let store = L2AnnStore::new(L2CacheDistance, 3, 5);
        let results = store.search(&[1.0, 2.0, 3.0], 5, 10.0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_ann_search_after_rebuild() {
        let store = L2AnnStore::new(L2CacheDistance, 2, 5);
        let entries = vec![
            (1, vec![1.0, 0.0]),
            (2, vec![0.0, 1.0]),
            (3, vec![10.0, 10.0]),
        ];
        store.rebuild(entries);

        let results = store.search(&[1.1, 0.0], 2, 2.0);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 1);

        // [1.1, 0.0] to [1.0, 0.0] distance squared is 0.01.
        // [1.1, 0.0] to [0.0, 1.0] distance squared is 1.21 + 1.0 = 2.21.
        // If threshold is 2.0, only id=1 (dist=0.01) is returned.
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_tombstone_filters_result() {
        let store = L2AnnStore::new(L2CacheDistance, 2, 5);
        let entries = vec![
            (1, vec![1.0, 0.0]),
            (2, vec![0.0, 1.0]),
        ];
        store.rebuild(entries);

        store.tombstone(1);
        let results = store.search(&[1.0, 0.0], 2, 10.0);
        assert_eq!(results[0].0, 2);
    }

    #[test]
    fn test_rebuild_clears_tombstones() {
        let store = L2AnnStore::new(L2CacheDistance, 2, 5);
        let entries = vec![
            (1, vec![1.0, 0.0]),
        ];
        store.rebuild(entries);
        store.tombstone(1);
        assert_eq!(store.tombstone_len(), 1);

        store.rebuild(vec![(1, vec![1.0, 0.0])]);
        assert_eq!(store.tombstone_len(), 0);
    }

    /// FIX verification: pending queue must be cleared after rebuild.
    #[test]
    fn test_rebuild_clears_pending() {
        let store = L2AnnStore::new(L2CacheDistance, 2, 100); // high capacity, won't auto-trigger

        // Manually enqueue some entries
        store.enqueue(1, vec![1.0, 0.0]);
        store.enqueue(2, vec![0.0, 1.0]);
        assert_eq!(store.pending_len(), 2);

        // rebuild should clear the pending queue
        store.rebuild(vec![(1, vec![1.0, 0.0]), (2, vec![0.0, 1.0])]);
        assert_eq!(store.pending_len(), 0, "pending queue must be empty after rebuild");
    }

    /// FIX verification: tombstone_ratio() computed correctly.
    #[test]
    fn test_tombstone_ratio() {
        let store = L2AnnStore::new(L2CacheDistance, 2, 5);
        // Empty index → ratio is 0.0 (no division by zero)
        assert_eq!(store.tombstone_ratio(), 0.0);

        store.rebuild(vec![(1, vec![1.0, 0.0]), (2, vec![0.0, 1.0]), (3, vec![5.0, 5.0])]);
        assert_eq!(store.index_len(), 3);

        store.tombstone(1);
        let ratio = store.tombstone_ratio();
        assert!((ratio - 1.0 / 3.0).abs() < 1e-9);
    }
}
