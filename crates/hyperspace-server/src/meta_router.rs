//! # MetaRouter — Global In-RAM Chunk Router (Task 1.2)
//!
//! Maps search queries to a small subset of immutable chunk files (`chunk_*.hyp`)
//! by comparing the query against per-chunk centroid vectors stored in a compact
//! flat array. This is an IVF-style routing layer: instead of scanning every
//! chunk we pick the `probe_k` nearest centroids and only load those segments.
//!
//! ## Design
//! - One `ChunkMeta` entry per frozen segment produced by the Flush Worker.
//! - Centroid = arithmetic mean of all vectors in that chunk (computed during flush).
//! - `route()` performs a brute-force L2 scan over centroids (O(C·N) where C is
//!   the number of chunks, typically < 1000 even at 100 M vectors). At N=1024 and
//!   C=100 this is ~0.4 ms — negligible compared with HNSW traversal.
//! - Thread-safe: `DashMap` for concurrent registration, no locks on hot-path read.
//!
//! ## RAM overhead
//! One centroid of N f64 values (1024-d = 8 KB) per chunk.\
//! 1000 chunks → ~8 MB total — well within the 0.1% budget stated in Gate Check 1.2.

use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;

// ─── Data Structures ────────────────────────────────────────────────────────

/// Metadata for a single immutable chunk segment.
#[derive(Debug, Clone)]
pub struct ChunkMeta {
    /// Unique chunk identifier (directory name, e.g. `chunk_<uuid>.hyp`).
    pub chunk_id: String,
    /// Absolute path to the chunk directory on disk.
    pub path: PathBuf,
    /// Centroid of all vectors in the chunk (arithmetic mean, f64 coords).
    pub centroid: Vec<f64>,
    /// Number of vectors stored in the chunk.
    /// Used for load-balancing decisions and future Dashboard storage stats.
    #[allow(dead_code)]
    pub vector_count: u32,
}

/// Macro-cluster grouping multiple chunk centroids for 2-level hierarchical routing.
#[derive(Debug, Clone)]
pub struct MacroCluster {
    /// Arithmetic centroid of the member chunk centroids.
    pub centroid: Vec<f64>,
    /// Indices of chunks in `MetaRouter.chunks` belonging to this macro-cluster.
    pub chunk_indices: Vec<usize>,
}

// ─── MetaRouter ────────────────────────────────────────────────────────────

/// Global in-RAM router from query vectors to relevant chunks.
///
/// Internally a flat list of `ChunkMeta` entries protected by an `RwLock`.
/// When chunk count exceeds 32, automatically maintains 2-level macro-clusters
/// via SuperKMeans to scale routing to hundreds of thousands of chunks (e.g. 21.2B vectors)
/// with sub-millisecond query routing latency.
pub struct MetaRouter {
    /// Registered chunk metadata, keyed by chunk_id for O(1) dedup.
    chunks: Arc<RwLock<Vec<ChunkMeta>>>,
    /// Index for fast O(1) chunk existence check.
    chunk_index: DashMap<String, usize>,
    /// Macro-clusters grouping chunk centroids (SuperKMeans 2-level routing).
    macro_clusters: Arc<RwLock<Vec<MacroCluster>>>,
    /// Count of centroid distance computations (telemetry).
    pub probe_ops: AtomicU64,
    /// Count of search queries routed (telemetry).
    pub query_count: AtomicU64,
}

impl Default for MetaRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaRouter {
    /// Creates an empty MetaRouter.
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(Vec::new())),
            chunk_index: DashMap::new(),
            macro_clusters: Arc::new(RwLock::new(Vec::new())),
            probe_ops: AtomicU64::new(0),
            query_count: AtomicU64::new(0),
        }
    }

    /// Registers a chunk and its centroid.
    /// If a chunk with the same `chunk_id` already exists, it is updated in-place.
    ///
    /// Called by the Flush Worker after building a new `chunk_*.hyp` segment.
    pub fn register(&self, meta: ChunkMeta) {
        let id = meta.chunk_id.clone();
        let mut guard = self.chunks.write();
        let should_rebuild = if let Some(&idx) = self.chunk_index.get(&id).as_deref() {
            guard[idx] = meta;
            false
        } else {
            let idx = guard.len();
            guard.push(meta);
            self.chunk_index.insert(id, idx);
            guard.len() >= 32
        };
        drop(guard);

        if should_rebuild {
            self.rebuild_macro_clusters();
        }
    }

    /// Removes a chunk from the router (called when a chunk is merged or deleted).
    /// Used by Flush Worker and Dashboard storage ops (Task 4.1).
    #[allow(dead_code)]
    pub fn unregister(&self, chunk_id: &str) {
        if let Some((_, removed_idx)) = self.chunk_index.remove(chunk_id) {
            let mut guard = self.chunks.write();
            if removed_idx < guard.len() {
                guard.swap_remove(removed_idx);
                // After swap_remove the last element is now at `removed_idx`.
                // Update its index in chunk_index.
                if removed_idx < guard.len() {
                    let moved_id = guard[removed_idx].chunk_id.clone();
                    self.chunk_index.insert(moved_id, removed_idx);
                }
            }
            drop(guard);
            self.rebuild_macro_clusters();
        }
    }

    /// Rebuilds hierarchical macro-clusters using SuperKMeans when chunk count >= 32.
    /// Reduces routing distance computations from O(C) to O(sqrt(C)).
    pub fn rebuild_macro_clusters(&self) {
        let guard = self.chunks.read();
        let num_chunks = guard.len();
        if num_chunks < 32 {
            self.macro_clusters.write().clear();
            return;
        }

        let k = ((num_chunks as f64).sqrt().ceil() as usize).clamp(2, 512);
        let centroids: Vec<Vec<f64>> = guard.iter().map(|c| c.centroid.clone()).collect();
        drop(guard);

        let Some((macro_centroids, assignments)) =
            hyperspace_clustering::cluster_vectors_f64(&centroids, k)
        else {
            self.macro_clusters.write().clear();
            return;
        };

        let mut clusters: Vec<MacroCluster> = macro_centroids
            .into_iter()
            .map(|c| MacroCluster {
                centroid: c,
                chunk_indices: Vec::new(),
            })
            .collect();

        for (chunk_idx, &cluster_idx) in assignments.iter().enumerate() {
            if cluster_idx < clusters.len() {
                clusters[cluster_idx].chunk_indices.push(chunk_idx);
            }
        }

        clusters.retain(|c| !c.chunk_indices.is_empty());
        *self.macro_clusters.write() = clusters;
    }

    /// Atomically replaces all registered chunks with new compacted chunks.
    pub fn replace_all(&self, new_chunks: Vec<ChunkMeta>) {
        let mut guard = self.chunks.write();
        self.chunk_index.clear();
        guard.clear();
        for (idx, meta) in new_chunks.into_iter().enumerate() {
            self.chunk_index.insert(meta.chunk_id.clone(), idx);
            guard.push(meta);
        }
        drop(guard);
        self.rebuild_macro_clusters();
    }

    /// Returns the number of registered chunks.
    /// Used by Dashboard Storage page and telemetry (Task 4.1).
    #[allow(dead_code)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.read().len()
    }

    /// Returns a snapshot of all chunk metadata (for persistence / diagnostics).
    /// Used by Dashboard Storage page to list NVMe vs S3 segments (Task 4.1).
    #[allow(dead_code)]
    pub fn all_chunks(&self) -> Vec<ChunkMeta> {
        self.chunks.read().clone()
    }

    /// Returns the total number of vectors across all registered chunks.
    pub fn total_vector_count(&self) -> usize {
        self.chunks
            .read()
            .iter()
            .map(|c| c.vector_count as usize)
            .sum()
    }

    /// Routes a search query to the `probe_k` most-relevant chunks.
    ///
    /// # Algorithm
    /// 1. Compute squared L2 distance from `query` to every chunk centroid.
    /// 2. Return the paths of the `probe_k` closest chunks.
    ///
    /// When there are ≤ `probe_k` chunks, ALL chunks are returned (full-scan fallback).
    ///
    /// # Parameters
    /// - `query`: The query vector (must have length N).
    /// - `probe_k`: Number of chunks to probe (recommended: 2–3 for Euclidean/Cosine).
    ///
    /// # Returns
    /// Ordered list `(chunk_id, path, dist_sq)` — nearest first.
    pub fn route(&self, query: &[f64], probe_k: usize) -> Vec<(String, PathBuf, f64)> {
        self.query_count.fetch_add(1, Ordering::Relaxed);

        let guard = self.chunks.read();
        if guard.is_empty() {
            return Vec::new();
        }

        let num_chunks = guard.len();
        // No routing needed when probe_k >= all chunks.
        if probe_k == 0 || probe_k >= num_chunks {
            return guard
                .iter()
                .map(|c| (c.chunk_id.clone(), c.path.clone(), 0.0))
                .collect();
        }

        let macro_guard = self.macro_clusters.read();
        if !macro_guard.is_empty() {
            // ── Hierarchical 2-Level Macro-Routing ──
            // 1. Probe query against macro-centroids
            let m_k = (probe_k.max(1) * 2).clamp(2, macro_guard.len());
            let mut macro_heap: BinaryHeap<(ordered_float::OrderedFloat<f64>, usize)> =
                BinaryHeap::with_capacity(m_k + 1);

            for (m_idx, macro_c) in macro_guard.iter().enumerate() {
                let dist_sq = Self::l2_sq_centroid(query, &macro_c.centroid);
                if macro_heap.len() < m_k {
                    macro_heap.push((ordered_float::OrderedFloat(-dist_sq), m_idx));
                } else if let Some(&(top, _)) = macro_heap.peek() {
                    if ordered_float::OrderedFloat(-dist_sq) > top {
                        macro_heap.pop();
                        macro_heap.push((ordered_float::OrderedFloat(-dist_sq), m_idx));
                    }
                }
            }

            // 2. Gather candidate chunk indices from the winning macro-clusters
            let mut candidate_indices = Vec::new();
            for (_, m_idx) in macro_heap {
                candidate_indices.extend_from_slice(&macro_guard[m_idx].chunk_indices);
            }
            drop(macro_guard);

            self.probe_ops
                .fetch_add(candidate_indices.len() as u64, Ordering::Relaxed);

            // 3. Score candidate chunks and keep probe_k nearest
            let mut heap: BinaryHeap<(ordered_float::OrderedFloat<f64>, usize)> =
                BinaryHeap::with_capacity(probe_k + 1);

            for &idx in &candidate_indices {
                if idx < guard.len() {
                    let dist_sq = Self::l2_sq_centroid(query, &guard[idx].centroid);
                    if heap.len() < probe_k {
                        heap.push((ordered_float::OrderedFloat(-dist_sq), idx));
                    } else if let Some(&(top, _)) = heap.peek() {
                        if ordered_float::OrderedFloat(-dist_sq) > top {
                            heap.pop();
                            heap.push((ordered_float::OrderedFloat(-dist_sq), idx));
                        }
                    }
                }
            }

            let mut results: Vec<(String, PathBuf, f64)> = heap
                .into_iter()
                .map(|(neg_dist, idx)| {
                    let chunk = &guard[idx];
                    (chunk.chunk_id.clone(), chunk.path.clone(), -neg_dist.0)
                })
                .collect();

            results.sort_by(|a, b| a.2.total_cmp(&b.2));
            return results;
        }

        // ── Flat Linear Scan Fallback (num_chunks < 32) ──
        self.probe_ops
            .fetch_add(num_chunks as u64, Ordering::Relaxed);

        // Min-heap trick: negate distances so BinaryHeap (max) acts as min-heap of k nearest.
        // We keep at most `probe_k` entries.
        let mut heap: BinaryHeap<(ordered_float::OrderedFloat<f64>, usize)> =
            BinaryHeap::with_capacity(probe_k + 1);

        for (idx, chunk) in guard.iter().enumerate() {
            let dist_sq = Self::l2_sq_centroid(query, &chunk.centroid);

            // Push negated distance — max-heap becomes nearest-k heap.
            if heap.len() < probe_k {
                heap.push((ordered_float::OrderedFloat(-dist_sq), idx));
            } else if let Some(&(top, _)) = heap.peek() {
                if ordered_float::OrderedFloat(-dist_sq) > top {
                    heap.pop();
                    heap.push((ordered_float::OrderedFloat(-dist_sq), idx));
                }
            }
        }

        // Collect in nearest-first order.
        let mut results: Vec<(String, PathBuf, f64)> = heap
            .into_iter()
            .map(|(neg_dist, idx)| {
                let chunk = &guard[idx];
                (chunk.chunk_id.clone(), chunk.path.clone(), -neg_dist.0)
            })
            .collect();

        results.sort_by(|a, b| a.2.total_cmp(&b.2));
        results
    }

    /// Computes centroid of a batch of f64 vectors.
    /// Returns None if `vectors` is empty.
    /// Called by Flush Worker at segment sealing time.
    #[allow(dead_code)]
    pub fn compute_centroid(vectors: &[Vec<f64>]) -> Option<Vec<f64>> {
        if vectors.is_empty() || vectors[0].is_empty() {
            return None;
        }
        let dim = vectors[0].len();
        let mut centroid = vec![0.0f64; dim];
        for vec in vectors {
            for (c, &v) in centroid.iter_mut().zip(vec.iter()) {
                *c += v;
            }
        }
        let n = vectors.len() as f64;
        for c in &mut centroid {
            *c /= n;
        }
        Some(centroid)
    }

    /// Streaming centroid accumulator — avoids holding all vectors in memory.
    /// Add each vector incrementally, then call `finish()`.
    #[allow(dead_code)]
    pub fn centroid_accumulator(dim: usize) -> CentroidAccumulator {
        CentroidAccumulator::new(dim)
    }

    #[inline]
    fn l2_sq_centroid(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum()
    }

    /// Telemetry: average number of chunks accessed per query.
    /// Exposed via Dashboard metrics API (Task 4.1).
    #[allow(dead_code)]
    pub fn avg_chunks_per_query(&self, probe_k: usize) -> f64 {
        let queries = self.query_count.load(Ordering::Relaxed);
        if queries == 0 {
            return 0.0;
        }
        // Each query probes min(probe_k, total_chunks) chunks.
        probe_k.min(self.chunk_count()) as f64
    }
}

// ─── CentroidAccumulator ───────────────────────────────────────────────────

/// Streaming accumulator for computing a centroid without buffering all vectors.
///
/// Usage:
/// ```rust,ignore
/// let mut acc = MetaRouter::<1024>::centroid_accumulator();
/// for vec in vectors { acc.add(&vec); }
/// let centroid = acc.finish();
/// ```
pub struct CentroidAccumulator {
    sum: Vec<f64>,
    count: u64,
    dim: usize,
}

impl CentroidAccumulator {
    pub fn new(dim: usize) -> Self {
        Self {
            sum: vec![0.0; dim],
            count: 0,
            dim,
        }
    }

    /// Accumulates one vector. Silently skips vectors with mismatched dimensions.
    pub fn add(&mut self, vec: &[f64]) {
        if vec.len() != self.dim {
            return;
        }
        for (s, &v) in self.sum.iter_mut().zip(vec.iter()) {
            *s += v;
        }
        self.count += 1;
    }

    /// Computes the centroid. Returns `None` if no vectors were added.
    pub fn finish(self) -> Option<Vec<f64>> {
        if self.count == 0 {
            return None;
        }
        let n = self.count as f64;
        Some(self.sum.into_iter().map(|s| s / n).collect())
    }

    /// Number of vectors accumulated so far.
    /// Used when building segment metadata during flush.
    #[allow(dead_code)]
    pub fn count(&self) -> u64 {
        self.count
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_centroid(val: f64) -> Vec<f64> {
        vec![val; 4]
    }

    fn make_meta(id: &str, val: f64) -> ChunkMeta {
        ChunkMeta {
            chunk_id: id.to_string(),
            path: PathBuf::from(format!("/data/{id}")),
            centroid: make_centroid(val),
            vector_count: 100,
        }
    }

    #[test]
    fn test_register_and_route() {
        let router = MetaRouter::new();
        router.register(make_meta("chunk_a", 1.0));
        router.register(make_meta("chunk_b", 5.0));
        router.register(make_meta("chunk_c", 10.0));

        // Query close to chunk_a
        let results = router.route(&[1.1, 1.1, 1.1, 1.1], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "chunk_a");

        // Query close to chunk_c
        let results = router.route(&[9.9, 9.9, 9.9, 9.9], 1);
        assert_eq!(results[0].0, "chunk_c");
    }

    #[test]
    fn test_probe_k_capped_at_chunk_count() {
        let router = MetaRouter::new();
        router.register(make_meta("c1", 1.0));
        router.register(make_meta("c2", 2.0));

        // probe_k > total chunks → returns all
        let results = router.route(&[1.5, 1.5, 1.5, 1.5], 100);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_unregister() {
        let router = MetaRouter::new();
        router.register(make_meta("chunk_x", 3.0));
        router.register(make_meta("chunk_y", 7.0));
        assert_eq!(router.chunk_count(), 2);

        router.unregister("chunk_x");
        assert_eq!(router.chunk_count(), 1);
        let results = router.route(&[3.0, 3.0, 3.0, 3.0], 5);
        assert!(results.iter().all(|(id, _, _)| id != "chunk_x"));
    }

    #[test]
    fn test_centroid_accumulator() {
        let mut acc = CentroidAccumulator::new(3);
        acc.add(&[1.0, 2.0, 3.0]);
        acc.add(&[3.0, 4.0, 5.0]);
        let centroid = acc.finish().unwrap();
        assert!((centroid[0] - 2.0).abs() < 1e-10);
        assert!((centroid[1] - 3.0).abs() < 1e-10);
        assert!((centroid[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_router() {
        let router = MetaRouter::new();
        let results = router.route(&[1.0; 4], 3);
        assert!(results.is_empty());
    }

    #[test]
    fn test_hierarchical_macro_routing_efficiency() {
        let router = MetaRouter::new();
        // Register 64 chunks: 32 clustered around 1.0, 32 clustered around 50.0
        for i in 0..32 {
            let offset = (i as f64) * 0.05;
            router.register(ChunkMeta {
                chunk_id: format!("cluster_low_{i}"),
                path: PathBuf::from(format!("/data/low_{i}")),
                centroid: vec![1.0 + offset; 4],
                vector_count: 50,
            });
        }
        for i in 0..32 {
            let offset = (i as f64) * 0.05;
            router.register(ChunkMeta {
                chunk_id: format!("cluster_high_{i}"),
                path: PathBuf::from(format!("/data/high_{i}")),
                centroid: vec![50.0 + offset; 4],
                vector_count: 50,
            });
        }

        assert_eq!(router.chunk_count(), 64);
        assert!(
            !router.macro_clusters.read().is_empty(),
            "Macro clusters should be built for 64 chunks"
        );

        // Route a query clearly in the high cluster
        let query = vec![50.05; 4];
        let probe_ops_before = router.probe_ops.load(Ordering::Relaxed);
        let results = router.route(&query, 2);
        let probe_ops_after = router.probe_ops.load(Ordering::Relaxed);

        assert_eq!(results.len(), 2);
        assert!(
            results[0].0.starts_with("cluster_high_"),
            "Top chunk should belong to cluster_high"
        );
        // Verify probe_ops is less than checking all 64 chunks
        let ops_used = probe_ops_after - probe_ops_before;
        assert!(
            ops_used < 64,
            "Hierarchical routing must prune search space, used {ops_used} ops < 64"
        );
    }
}
