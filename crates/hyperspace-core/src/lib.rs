#![recursion_limit = "512"]
#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::inline_always)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unsafe_derive_deserialize)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unreadable_literal)]
#![allow(unknown_lints)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::needless_bool)]

pub mod config;
pub mod fuzzy;
pub mod gpu;
pub mod graph;
pub mod gromov;
pub mod hybrid;
pub mod optim;
pub mod pq;
pub mod region;
pub mod simd;
pub mod turbo_rot;
pub mod vector;
pub use hybrid::{HybridExtremeQuantizedVector, HybridMetric};
pub use pq::{PQLookupTable, PQVector, ProductQuantizer};
pub mod wasserstein;

pub use config::GlobalConfig;
pub mod bm25;
pub use bm25::*;
pub use graph::*;
use vector::{BinaryHyperVector, HyperVector, QuantizedHyperVector};

#[cfg(feature = "nightly-simd")]
pub fn check_simd() {
    println!("🚀 SIMD Acceleration: ENABLED (AVX/Neon)");
    println!("⚡ Runtime f32 kernels: {}", simd::backend().name());
}

#[cfg(not(feature = "nightly-simd"))]
pub fn check_simd() {
    println!("🐢 SIMD Acceleration: DISABLED (Scalar Fallback)");
    println!("⚡ Runtime f32 kernels: {}", simd::backend().name());
}

#[cfg(test)]
mod hybrid_tests;
#[cfg(test)]
mod tests;

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QuantizationMode {
    None,
    ScalarI8,
    Binary,
    AsymmetricHybrid801,
    AsymmetricHybridLowBit,
    AsymmetricHybridExtreme,
    ScalarI4,
    Turbo,
    ProductQuantization,
    OPQ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StorageMode {
    Performance, // Default: Huge single HNSW index, high RAM usage, max speed.
    Tiered,      // LSM Mode: Frequent WAL-to-Chunk flushes, minimizes RAM.
}

/// Metric abstraction for distance calculation
pub struct PoincareMetric;

pub struct EuclideanMetric;

pub struct LorentzMetric;

#[derive(Debug, Clone)]
pub enum FilterExpr {
    Match {
        key: String,
        value: String,
    },
    Prefix {
        key: String,
        prefix: String,
    },
    Range {
        key: String,
        gte: Option<f64>,
        lte: Option<f64>,
    },
    InCone {
        axes: Vec<f64>,
        apertures: Vec<f64>,
        cen: f64, // tolerance factor
    },
    InBox {
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
    },
    InBall {
        center: Vec<f64>,
        radius: f64,
    },
    And(Vec<FilterExpr>),
    Or(Vec<FilterExpr>),
    Not(Box<FilterExpr>),
}

impl FilterExpr {
    pub fn check(
        &self,
        vector: &HyperVector,
        metadata: &std::collections::HashMap<String, String>,
    ) -> bool {
        self.check_coords(&vector.coords, metadata)
    }

    pub fn check_coords(
        &self,
        coords: &[f64],
        metadata: &std::collections::HashMap<String, String>,
    ) -> bool {
        match self {
            Self::Match { key, value } => {
                get_meta_string(metadata, key).as_deref() == Some(value.as_str())
            }
            Self::Prefix { key, prefix } => {
                get_meta_string(metadata, key).is_some_and(|s| s.starts_with(prefix))
            }
            Self::Range { key, gte, lte } => {
                if let Some(val) = get_meta_numeric(metadata, key) {
                    if let Some(g) = gte {
                        if val < *g {
                            return false;
                        }
                    }
                    if let Some(l) = lte {
                        if val > *l {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            Self::InCone {
                axes,
                apertures,
                cen,
            } => region::ConeRegion::check_contains(axes, apertures, *cen, coords),
            Self::InBox {
                min_bounds,
                max_bounds,
            } => region::BoxRegion::check_contains(min_bounds, max_bounds, coords),
            Self::InBall { center, radius } => {
                region::BallRegion::check_contains(center, *radius, coords)
            }
            Self::And(conds) => conds.iter().all(|c| c.check_coords(coords, metadata)),
            Self::Or(conds) => conds.iter().any(|c| c.check_coords(coords, metadata)),
            Self::Not(cond) => !cond.check_coords(coords, metadata),
        }
    }

    /// Metadata-only check (no vector). Geometric filters (`InCone`, `InBall`, `InBox`)
    /// are skipped (return `true`) because vector data is unavailable in scroll context.
    pub fn check_meta(&self, metadata: &std::collections::HashMap<String, String>) -> bool {
        match self {
            Self::Match { key, value } => {
                get_meta_string(metadata, key).as_deref() == Some(value.as_str())
            }
            Self::Prefix { key, prefix } => {
                get_meta_string(metadata, key).is_some_and(|s| s.starts_with(prefix))
            }
            Self::Range { key, gte, lte } => {
                if let Some(val) = get_meta_numeric(metadata, key) {
                    if let Some(g) = gte {
                        if val < *g {
                            return false;
                        }
                    }
                    if let Some(l) = lte {
                        if val > *l {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            // Geometric filters need vector data; pass-through in meta-only context.
            Self::InCone { .. } | Self::InBox { .. } | Self::InBall { .. } => true,
            Self::And(conds) => conds.iter().all(|c| c.check_meta(metadata)),
            Self::Or(conds) => conds.iter().any(|c| c.check_meta(metadata)),
            Self::Not(cond) => !cond.check_meta(metadata),
        }
    }
}

#[inline]
fn get_typed_key<'a>(key: &str, buf: &'a mut [u8; 256]) -> Option<&'a str> {
    const PREFIX: &[u8] = b"__hs_typed__";
    if PREFIX.len() + key.len() <= buf.len() {
        buf[..PREFIX.len()].copy_from_slice(PREFIX);
        buf[PREFIX.len()..PREFIX.len() + key.len()].copy_from_slice(key.as_bytes());
        std::str::from_utf8(&buf[..PREFIX.len() + key.len()]).ok()
    } else {
        None
    }
}

fn get_meta_string<'a>(
    metadata: &'a std::collections::HashMap<String, String>,
    key: &str,
) -> Option<std::borrow::Cow<'a, str>> {
    if let Some(v) = metadata.get(key) {
        return Some(std::borrow::Cow::Borrowed(v.as_str()));
    }
    let mut buf = [0u8; 256];
    let raw_typed = if let Some(k) = get_typed_key(key, &mut buf) {
        metadata.get(k)
    } else {
        metadata.get(&format!("__hs_typed__{key}"))
    }?;

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum ShadowPayload<'b> {
        Str(std::borrow::Cow<'b, str>),
        Int(i64),
        Float(f64),
        Bool(bool),
    }
    #[derive(serde::Deserialize)]
    struct ShadowObj<'b> {
        #[serde(borrow)]
        v: ShadowPayload<'b>,
    }

    if let Ok(shadow) = serde_json::from_str::<ShadowObj>(raw_typed) {
        return match shadow.v {
            ShadowPayload::Str(s) => Some(s),
            ShadowPayload::Int(i) => Some(std::borrow::Cow::Owned(i.to_string())),
            ShadowPayload::Float(f) => Some(std::borrow::Cow::Owned(f.to_string())),
            ShadowPayload::Bool(b) => Some(std::borrow::Cow::Owned(b.to_string())),
        };
    }
    None
}

fn get_meta_numeric(
    metadata: &std::collections::HashMap<String, String>,
    key: &str,
) -> Option<f64> {
    if let Some(v) = metadata.get(key) {
        if let Ok(num) = v.parse::<f64>() {
            return Some(num);
        }
    }
    let mut buf = [0u8; 256];
    let raw_typed = if let Some(k) = get_typed_key(key, &mut buf) {
        metadata.get(k)
    } else {
        metadata.get(&format!("__hs_typed__{key}"))
    }?;

    #[derive(serde::Deserialize)]
    struct ShadowNumeric {
        v: f64,
    }
    serde_json::from_str::<ShadowNumeric>(raw_typed)
        .ok()
        .map(|s| s.v)
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub top_k: usize,
    pub ef_search: usize,
    pub hybrid_query: Option<String>,
    pub hybrid_alpha: Option<f32>,
    pub component_weights: Option<std::collections::HashMap<String, f32>>,
    pub use_wasserstein: bool,
    pub bm25_options: Option<crate::bm25::Bm25Params>,
    pub fusion_method: Option<String>,
    pub mrl_dimension: Option<usize>,
    /// Sidecar Payload Storage (v3.2): if true, the server performs lazy disk I/O
    /// for the final Top-K results. Default false = zero extra I/O.
    pub include_payload: bool,
    pub use_wave: bool,
}

/// A single search result: (id, distance, metadata, `optional_payload`).
/// The `payload` field is `Some(bytes)` ONLY when `SearchParams::include_payload = true`
/// AND the vector was inserted with a payload. The bytes are the original uncompressed
/// document — all decompression is performed server-side via lazy disk I/O.
pub type SearchResult = (
    u32,
    f64,
    std::collections::HashMap<String, String>,
    Option<Vec<u8>>,
);

/// A single search result with 64-bit ID for billion-scale collections.
pub type SearchResultU64 = (
    u64,
    f64,
    std::collections::HashMap<String, String>,
    Option<Vec<u8>>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    Default,
    Async,
    Batch,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VacuumFilterOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,
}

#[derive(Debug, Clone)]
pub struct VacuumFilterQuery {
    pub key: String,
    pub op: VacuumFilterOp,
    pub value: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionUsage {
    pub disk_usage_bytes: u64,
    pub ram_usage_bytes: u64,
    pub active_indexing_tasks: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionConfigUpdate {
    pub ef_search: Option<usize>,
    pub ef_construction: Option<usize>,
    pub m: Option<usize>,
}

/// Read-only snapshot of the current HNSW runtime configuration.
/// Returned by `Collection::get_hnsw_config()` to expose live values to the HTTP API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HnswConfig {
    pub ef_search: usize,
    pub ef_construction: usize,
    pub m: usize,
}

#[async_trait::async_trait]
pub trait Collection: Send + Sync + 'static {
    fn name(&self) -> &str;
    async fn insert(
        &self,
        vector: &[f64],
        id: u32,
        metadata: std::collections::HashMap<String, String>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String>;

    async fn insert_batch(
        &self,
        vectors: Vec<(Vec<f64>, u32, std::collections::HashMap<String, String>)>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        // Default implementation using single insert (slow fallback)
        for (vec, id, meta) in vectors {
            self.insert(&vec, id, meta, clock, durability).await?;
        }
        Ok(())
    }

    /// Insert vector with 64-bit ID for billion-scale collections (> 4.29B records).
    async fn insert_u64(
        &self,
        vector: &[f64],
        id: u64,
        metadata: std::collections::HashMap<String, String>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        if id > u32::MAX as u64 {
            return Err(format!(
                "Collection does not support 64-bit ID {} (> u32::MAX)",
                id
            ));
        }
        self.insert(vector, id as u32, metadata, clock, durability)
            .await
    }

    /// Batch insert vectors with 64-bit IDs.
    async fn insert_batch_u64(
        &self,
        vectors: Vec<(Vec<f64>, u64, std::collections::HashMap<String, String>)>,
        clock: u64,
        durability: Durability,
    ) -> Result<(), String> {
        for (vec, id, meta) in vectors {
            self.insert_u64(&vec, id, meta, clock, durability).await?;
        }
        Ok(())
    }

    fn delete(&self, id: u32) -> Result<(), String>;

    /// Delete vector by 64-bit ID.
    fn delete_u64(&self, id: u64) -> Result<(), String> {
        if id > u32::MAX as u64 {
            return Err(format!(
                "Collection does not support 64-bit ID {} (> u32::MAX)",
                id
            ));
        }
        self.delete(id as u32)
    }

    async fn search(
        &self,
        vector: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResult>, String>;

    /// Search vector returning 64-bit IDs.
    async fn search_u64(
        &self,
        vector: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResultU64>, String> {
        let res = self.search(vector, filter, complex_filters, params).await?;
        Ok(res
            .into_iter()
            .map(|(id, dist, meta, payload)| (id as u64, dist, meta, payload))
            .collect())
    }
    fn count(&self) -> usize;
    fn dimension(&self) -> usize;
    fn metric_name(&self) -> &'static str;
    fn state_hash(&self) -> u64;
    fn buckets(&self) -> Vec<u64>; // New method
    fn queue_size(&self) -> u64; // Indexing queue size for eventual consistency
    fn write_buffer_size(&self) -> usize {
        0
    }
    fn quantization_mode(&self) -> crate::QuantizationMode;
    fn get_usage(&self) -> CollectionUsage;
    fn graph_stats(&self) -> (usize, usize, usize) {
        (0, 0, 0)
    }
    fn get_hnsw_config(&self) -> HnswConfig {
        // Default implementation — overridden by CollectionImpl with real atomics.
        HnswConfig {
            ef_search: 0,
            ef_construction: 0,
            m: 0,
        }
    }
    fn update_config(&self, config: CollectionConfigUpdate) -> Result<(), String>;
    async fn optimize(&self) -> Result<(), String> {
        // Default: No-op for collections lacking optimization support.
        Ok(())
    }
    async fn optimize_with_filter(&self, filter: Option<VacuumFilterQuery>) -> Result<(), String> {
        let _ = filter;
        self.optimize().await
    }
    fn peek(
        &self,
        limit: usize,
        offset: usize,
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)>;
    /// Returns all vectors belonging to the given sync buckets (id % 256 == `bucket_index`).
    /// Used by Delta Sync (Task 2.1) to transfer only changed partitions.
    fn peek_buckets(
        &self,
        bucket_indices: &[u32],
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        let set: std::collections::HashSet<u32> = bucket_indices.iter().copied().collect();
        self.peek(self.count().max(1), 0)
            .into_iter()
            .filter(|(id, _, _)| set.contains(&((*id as usize % 256) as u32)))
            .collect()
    }
    fn graph_neighbors(&self, id: u32, layer: usize, limit: usize) -> Result<Vec<u32>, String>;
    fn graph_neighbor_distances(
        &self,
        source_id: u32,
        neighbor_ids: &[u32],
    ) -> Result<Vec<f64>, String>;
    fn graph_traverse(
        &self,
        start_id: u32,
        layer: usize,
        max_depth: usize,
        max_nodes: usize,
        breadth_limit: usize,
    ) -> Result<Vec<u32>, String>;
    fn graph_clusters(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Result<Vec<Vec<u32>>, String>;

    /// Links two nodes with an explicit directed labeled edge.
    fn graph_link(
        &self,
        source_id: u32,
        target_id: u32,
        relation: u16,
        weight: f32,
    ) -> Result<(), String> {
        let _ = (source_id, target_id, relation, weight);
        Ok(())
    }

    /// Batch inserts multiple directed labeled edges.
    fn graph_link_batch(&self, edges: &[(u32, u32, u16, f32)]) -> Result<usize, String> {
        for &(src, tgt, rel, w) in edges {
            self.graph_link(src, tgt, rel, w)?;
        }
        Ok(edges.len())
    }

    /// Deletes an explicit directed edge with 0 μs invalidation lag.
    fn graph_unlink(&self, source_id: u32, target_id: u32, relation: u16) -> Result<bool, String> {
        let _ = (source_id, target_id, relation);
        Ok(false)
    }

    /// Induces explicit edges from existing metadata fields according to declarative rules.
    fn graph_induce_from_metadata(&self, rules: &[MetadataInductionRule]) -> Result<usize, String> {
        let _ = rules;
        Ok(0)
    }

    /// Repartitions immutable chunks using SuperKMeans into Voronoi partitions.
    fn compact_semantic_chunks(&self, target_partitions: usize) -> Result<String, String> {
        let _ = target_partitions;
        Ok("Semantic compaction not supported for this collection type".into())
    }

    /// Geometric Trust Scoring (Tribunal)
    /// Evaluates a trajectory of Node IDs and returns a trust score [0.0, 1.0]
    /// based on the Lyapunov stability / geometric distance in the local manifold.
    /// A score close to 0 indicates a hallucination / invalid jump.
    fn evaluate_trust_score(&self, trajectory_ids: &[u32]) -> Result<f64, String> {
        let _ = trajectory_ids;
        Err("Tribunal Trust Scoring not implemented for this collection".into())
    }

    /// Fetch raw point data for a given list of IDs. Returns (id, vector, metadata) tuples.
    fn get_points(
        &self,
        ids: &[u32],
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        // Default: use peek and filter by requested IDs
        let id_set: std::collections::HashSet<u32> = ids.iter().copied().collect();
        self.peek(self.count().max(1), 0)
            .into_iter()
            .filter(|(id, _, _)| id_set.contains(id))
            .collect()
    }

    /// Fetch raw point data for 64-bit IDs. Returns (id, vector, metadata) tuples.
    fn get_points_u64(
        &self,
        ids: &[u64],
    ) -> Vec<(u64, Vec<f64>, std::collections::HashMap<String, String>)> {
        let u32_ids: Vec<u32> = ids
            .iter()
            .filter_map(|&id| u32::try_from(id).ok())
            .collect();
        self.get_points(&u32_ids)
            .into_iter()
            .map(|(id, vec, meta)| (id as u64, vec, meta))
            .collect()
    }

    /// Patch-update a point's metadata without touching the vector.
    async fn update_payload(
        &self,
        id: u32,
        patch: std::collections::HashMap<String, String>,
    ) -> Result<(), String>;

    /// Look up the metadata `HashMap` for a given user-facing vector ID.
    fn metadata_by_id(&self, id: u32) -> std::collections::HashMap<String, String>;

    /// Write a heavy payload blob to the disk-only Payload Layer.
    /// Called AFTER a successful `insert()`. The payload is zstd-compressed
    /// before hitting disk. This method MUST NOT store any bytes in RAM.
    /// Default no-op: implementors without payload support silently ignore it.
    async fn insert_payload(&self, _id: u32, _payload: Vec<u8>) -> Result<(), String> {
        Ok(())
    }

    /// Write a batch of payload blobs to disk in a single I/O transaction.
    /// Default implementation loops over `insert_payload`.
    async fn insert_payload_batch(&self, entries: Vec<(u32, Vec<u8>)>) -> Result<(), String> {
        for (id, payload) in entries {
            self.insert_payload(id, payload).await?;
        }
        Ok(())
    }

    /// Fetch decompressed payloads for the given IDs via lazy disk I/O.
    /// MUST be called ONLY after HNSW traversal yields the final Top-K results.
    /// Uses `tokio::task::spawn_blocking` internally to avoid blocking the runtime.
    /// Returns a Vec aligned with `ids`: `None` if the ID has no payload.
    async fn fetch_payloads(&self, _ids: &[u32]) -> Vec<Option<Vec<u8>>> {
        vec![None; _ids.len()]
    }

    /// Iterate over all points, optionally filtered, for data export / Explorer.
    fn scroll(
        &self,
        limit: usize,
        offset: usize,
        filters: &[FilterExpr],
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        // Default implementation using peek + in-memory filter.
        self.peek(self.count().max(1), 0)
            .into_iter()
            .skip(offset)
            .filter(|(_, _vec, meta)| {
                // Build a dummy HyperVector-like slice for geometric checks.
                // We pass the slice to check(); for geometric filters this is best-effort.
                filters.iter().all(|f| {
                    // For metadata-only filters (Match/Range/And/Or/Not)
                    // we can evaluate without a typed HyperVector.
                    // Geometric filters return true by default on unknown vectors.
                    f.check_meta(meta)
                })
            })
            .take(limit)
            .collect()
    }

    /// Count points matching a set of filters.
    fn count_filtered(&self, filters: &[FilterExpr]) -> usize {
        if filters.is_empty() {
            return self.count();
        }
        self.scroll(usize::MAX, 0, filters).len()
    }

    fn cache_stats(&self) -> Result<String, String> {
        Err("Cache not supported".to_string())
    }
    fn cache_clear(&self) -> Result<(), String> {
        Err("Cache not supported".to_string())
    }
    fn cache_update_config(
        &self,
        policy: String,
        ann_threshold: Option<f64>,
    ) -> Result<(), String> {
        let _ = policy;
        let _ = ann_threshold;
        Err("Cache not supported".to_string())
    }
    fn create_snapshot(&self) -> Result<(), String> {
        Ok(())
    }
}

pub trait Metric: Send + Sync + 'static {
    fn name() -> &'static str;
    fn distance(a: &[f64], b: &[f64]) -> f64;

    // Default valid verification: ensure all dimensions are finite
    fn validate(vector: &[f64]) -> Result<(), String> {
        for &val in vector {
            if !val.is_finite() {
                return Err(format!(
                    "Vector contains non-finite values (NaN/Inf) for metric: {}",
                    Self::name()
                ));
            }
        }
        Ok(())
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64;
    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64;

    /// Extrapolates the trajectory using metric-specific geometry (e.g. Koopman/Riemannian tangent projection).
    /// Default returns an error if the metric doesn't support momentum traversal.
    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        let _ = (past, current, steps);
        Err(format!(
            "Momentum extrapolation not supported for metric: {}",
            Self::name()
        ))
    }
}

impl Metric for PoincareMetric {
    fn name() -> &'static str {
        "poincare"
    }

    #[inline(always)]
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        const LANES: usize = 4;
        let n = a.len().min(b.len());
        let mut acc_u = [0.0; LANES];
        let mut acc_v = [0.0; LANES];
        let mut acc_d = [0.0; LANES];
        let chunks_a = a[..n].chunks_exact(LANES);
        let chunks_b = b[..n].chunks_exact(LANES);
        let rem_a = chunks_a.remainder();
        let rem_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            for l in 0..LANES {
                let u = ca[l];
                let v = cb[l];
                acc_u[l] += u * u;
                acc_v[l] += v * v;
                let diff = u - v;
                acc_d[l] += diff * diff;
            }
        }
        let mut norm_u_sq = acc_u[0] + acc_u[1] + acc_u[2] + acc_u[3];
        let mut norm_v_sq = acc_v[0] + acc_v[1] + acc_v[2] + acc_v[3];
        let mut diff_sq = acc_d[0] + acc_d[1] + acc_d[2] + acc_d[3];
        for (&u, &v) in rem_a.iter().zip(rem_b.iter()) {
            norm_u_sq += u * u;
            norm_v_sq += v * v;
            let diff = u - v;
            diff_sq += diff * diff;
        }

        let denom = (1.0 - norm_u_sq) * (1.0 - norm_v_sq);
        let arg = 1.0 + 2.0 * diff_sq / denom.max(1e-9);
        arg.acosh()
    }

    fn validate(vector: &[f64]) -> Result<(), String> {
        for &val in vector {
            if !val.is_finite() {
                return Err("Poincaré vector contains non-finite values (NaN/Inf)".to_string());
            }
        }
        let sq_norm: f64 = vector.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector must be strictly inside the Poincaré ball".to_string());
        }
        Ok(())
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        a.poincare_distance_sq_to_float(b)
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.poincare_distance_sq_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        let c = 1.0;
        let mut x2 = 0.0;
        let mut y2 = 0.0;
        let mut xy = 0.0;
        for i in 0..past.len() {
            x2 += past[i] * past[i];
            y2 += current[i] * current[i];
            xy += past[i] * current[i];
        }

        let num_left = 1.0 + 2.0 * c * xy + c * y2;
        let num_right = 1.0 - c * x2;
        let den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
        if den.abs() < 1e-15 {
            return Err("Mobius addition denominator too small".into());
        }

        let mut delta = vec![0.0; past.len()];
        let mut delta_norm_sq = 0.0;
        for i in 0..past.len() {
            delta[i] = (num_left * (-past[i]) + num_right * current[i]) / den;
            delta_norm_sq += delta[i] * delta[i];
        }
        let delta_norm = delta_norm_sq.sqrt();
        if delta_norm < 1e-15 {
            return Ok(current.to_vec());
        }

        let lambda_x = 2.0 / (1.0 - c * x2).max(1e-15);
        let arg = (c.sqrt() * delta_norm).min(1.0 - 1e-15);
        let factor = (2.0 / (lambda_x * c.sqrt())) * arg.atanh();

        let mut velocity_at_past = vec![0.0; past.len()];
        for i in 0..past.len() {
            velocity_at_past[i] = factor * delta[i] / delta_norm;
        }

        // Simplified extrapolation (Euler step mapped back)
        let mut step = vec![0.0; past.len()];
        let mut v_norm_sq = 0.0;
        for i in 0..past.len() {
            step[i] = velocity_at_past[i] * steps;
            v_norm_sq += step[i] * step[i];
        }
        let v_norm = v_norm_sq.sqrt();
        if v_norm < 1e-15 {
            return Ok(current.to_vec());
        }

        let cx2 = current.iter().map(|&x| x * x).sum::<f64>();
        let lambda_c = 2.0 / (1.0 - c * cx2).max(1e-15);
        let scale = (c.sqrt() * lambda_c * v_norm / 2.0).tanh() / (c.sqrt() * v_norm);

        let mut applied_step = vec![0.0; past.len()];
        for i in 0..past.len() {
            applied_step[i] = scale * step[i];
        }

        let mut as_x2 = 0.0;
        let mut cx_as = 0.0;
        for i in 0..past.len() {
            as_x2 += applied_step[i] * applied_step[i];
            cx_as += current[i] * applied_step[i];
        }
        let num_left2 = 1.0 + 2.0 * c * cx_as + c * as_x2;
        let num_right2 = 1.0 - c * cx2;
        let den2 = 1.0 + 2.0 * c * cx_as + c * c * cx2 * as_x2;

        if den2.abs() < 1e-15 {
            return Err("Mobius addition denominator too small".into());
        }

        let mut result = vec![0.0; past.len()];
        for i in 0..past.len() {
            result[i] = (num_left2 * current[i] + num_right2 * applied_step[i]) / den2;
        }
        Ok(result)
    }
}

/// Default tolerance for the Lorentz hyperboloid membership check
/// (`|-t^2 + |x|^2 + 1|`). Preserves historical strict behaviour.
pub const LORENTZ_VALIDATE_DEFAULT_TOL: f64 = 1e-6;

/// Effective Lorentz validation tolerance, read once from the environment.
///
/// Externally-produced embeddings and quantization round-trips can land
/// slightly off the ideal hyperboloid. `distance` already tolerates this via
/// its acosh domain clamp `(-inner_prod).max(1.0 + 1e-12)`, but `validate`
/// historically rejected anything past `1e-6`. Operators ingesting such
/// vectors can relax the check by setting `HS_LORENTZ_VALIDATE_TOL` (e.g.
/// `0.1`). Invalid or non-positive values fall back to the strict default, so
/// existing deployments are unaffected.
pub fn lorentz_validate_tolerance() -> f64 {
    use std::sync::OnceLock;
    static TOL: OnceLock<f64> = OnceLock::new();
    *TOL.get_or_init(|| {
        std::env::var("HS_LORENTZ_VALIDATE_TOL")
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(LORENTZ_VALIDATE_DEFAULT_TOL)
    })
}

impl LorentzMetric {
    /// Hyperboloid membership check with an explicit tolerance.
    ///
    /// Kept separate from the env-driven `validate` so the geometry can be
    /// tested deterministically without touching process environment.
    pub fn validate_with_tolerance(vector: &[f64], tolerance: f64) -> Result<(), String> {
        if !vector.iter().all(|v| v.is_finite()) {
            return Err("Lorentz vector contains non-finite values".to_string());
        }
        if vector[0] <= 0.0 {
            return Err("Lorentz vector must be on the upper sheet (t > 0)".to_string());
        }

        let mut spatial_sq = 0.0_f64;
        for i in 1..vector.len() {
            spatial_sq += vector[i] * vector[i];
        }
        let minkowski_norm = -vector[0] * vector[0] + spatial_sq;
        if (minkowski_norm + 1.0).abs() > tolerance {
            return Err(format!(
                "Lorentz vector is not on unit hyperboloid: -t^2+|x|^2={minkowski_norm}, expected -1"
            ));
        }
        Ok(())
    }
}

impl Metric for LorentzMetric {
    fn name() -> &'static str {
        "lorentz"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        debug_assert!(
            a.len() >= 2,
            "Lorentz metric requires at least 2 dimensions"
        );
        let mut spatial_dot = 0.0;
        let mut i = 1;
        let n = a.len().min(b.len());

        #[cfg(feature = "nightly-simd")]
        {
            use std::simd::f64x4;
            use std::simd::num::SimdFloat;
            let mut sum_vec = f64x4::splat(0.0);
            while i + 4 <= n {
                let va = f64x4::from_slice(&a[i..i + 4]);
                let vb = f64x4::from_slice(&b[i..i + 4]);
                sum_vec += va * vb;
                i += 4;
            }
            spatial_dot += sum_vec.reduce_sum();
        }

        #[cfg(not(feature = "nightly-simd"))]
        if n > 1 {
            let mut acc = [0.0; 4];
            let (chunks_a, rem_a) = a[1..n].as_chunks::<4>();
            let (chunks_b, rem_b) = b[1..n].as_chunks::<4>();

            for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
                for l in 0..4 {
                    acc[l] += ca[l] * cb[l];
                }
            }
            spatial_dot += acc[0] + acc[1] + acc[2] + acc[3];
            for (&va, &vb) in rem_a.iter().zip(rem_b.iter()) {
                spatial_dot += va * vb;
            }
            i = n;
        }

        while i < n {
            spatial_dot += a[i] * b[i];
            i += 1;
        }

        let inner_prod = -a[0] * b[0] + spatial_dot;
        let arg = (-inner_prod).max(1.0 + 1e-12);
        arg.acosh()
    }

    fn validate(vector: &[f64]) -> Result<(), String> {
        Self::validate_with_tolerance(vector, lorentz_validate_tolerance())
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        a.lorentz_distance_to_float(b)
    }

    fn distance_binary(_a: &BinaryHyperVector, _b: &HyperVector) -> f64 {
        panic!("Binary quantization is not supported for the Lorentz model.");
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        // 1. Convert Lorentz to Poincare
        let to_poincare = |v: &[f64]| {
            let t = v[0];
            let mut p = vec![0.0; v.len() - 1];
            let factor = 1.0 / (t + 1.0);
            for i in 0..p.len() {
                p[i] = v[i + 1] * factor;
            }
            p
        };

        let p_past = to_poincare(past);
        let p_current = to_poincare(current);

        // 2. Extrapolate in Poincare ball
        let p_next = PoincareMetric::extrapolate_momentum(&p_past, &p_current, steps)?;

        // 3. Convert back to Lorentz
        let mut p_norm_sq = 0.0;
        for &val in &p_next {
            p_norm_sq += val * val;
        }
        let den = (1.0 - p_norm_sq).max(1e-15);
        let mut result = vec![0.0; past.len()];
        result[0] = (1.0 + p_norm_sq) / den;
        let factor = 2.0 / den;
        for i in 0..p_next.len() {
            result[i + 1] = p_next[i] * factor;
        }
        Ok(result)
    }
}

impl Metric for EuclideanMetric {
    fn name() -> &'static str {
        "l2"
    }

    #[cfg(feature = "nightly-simd")]
    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        use std::simd::f32x4;
        use std::simd::f64x4;
        use std::simd::num::SimdFloat;

        let mut sum = f32x4::splat(0.0);
        let mut i = 0;
        let n = a.len();

        while i + 8 <= n {
            let a_f64_1 = f64x4::from_slice(&a[i..i + 4]);
            let a_f64_2 = f64x4::from_slice(&a[i + 4..i + 8]);
            let b_f64_1 = f64x4::from_slice(&b[i..i + 4]);
            let b_f64_2 = f64x4::from_slice(&b[i + 4..i + 8]);

            let va1: f32x4 = a_f64_1.cast();
            let va2: f32x4 = a_f64_2.cast();
            let vb1: f32x4 = b_f64_1.cast();
            let vb2: f32x4 = b_f64_2.cast();

            let diff1 = va1 - vb1;
            let diff2 = va2 - vb2;

            sum += diff1 * diff1 + diff2 * diff2;
            i += 8;
        }

        while i + 4 <= n {
            let a_f64 = f64x4::from_slice(&a[i..i + 4]);
            let b_f64 = f64x4::from_slice(&b[i..i + 4]);

            let va: f32x4 = a_f64.cast();
            let vb: f32x4 = b_f64.cast();

            let diff = va - vb;
            sum += diff * diff;
            i += 4;
        }

        let mut total = sum.reduce_sum() as f64;

        while i < n {
            let diff = (a[i] as f32) - (b[i] as f32);
            total += (diff * diff) as f64;
            i += 1;
        }
        total
    }

    #[cfg(not(feature = "nightly-simd"))]
    #[inline(always)]
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        const LANES: usize = 8;
        let n = a.len().min(b.len());
        let mut acc = [0.0_f32; LANES];
        let chunks_a = a[..n].chunks_exact(LANES);
        let chunks_b = b[..n].chunks_exact(LANES);
        let rem_a = chunks_a.remainder();
        let rem_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            for l in 0..LANES {
                let diff = (ca[l] as f32) - (cb[l] as f32);
                acc[l] += diff * diff;
            }
        }
        let mut sum = 0.0_f32;
        for l in 0..LANES {
            sum += acc[l];
        }
        for (x, y) in rem_a.iter().zip(rem_b.iter()) {
            let diff = (*x as f32) - (*y as f32);
            sum += diff * diff;
        }
        f64::from(sum)
    }

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        use std::simd::num::{SimdFloat, SimdInt};
        use std::simd::{f32x4, f64x4, i8x4};

        const SCALE_INV: f32 = 1.0 / 127.0;
        let scale_vec = f32x4::splat(SCALE_INV);

        let mut sum = f32x4::splat(0.0);
        let mut i = 0;
        let n = a.coords.len();

        while i + 8 <= n {
            let a_chunk1 = i8x4::from_slice(&a.coords[i..i + 4]);
            let a_chunk2 = i8x4::from_slice(&a.coords[i + 4..i + 8]);

            let b_f64_1 = f64x4::from_slice(&b.coords[i..i + 4]);
            let b_f64_2 = f64x4::from_slice(&b.coords[i + 4..i + 8]);

            let b_chunk1: f32x4 = b_f64_1.cast();
            let b_chunk2: f32x4 = b_f64_2.cast();

            let a_f32_1: f32x4 = a_chunk1.cast();
            let a_f32_2: f32x4 = a_chunk2.cast();

            let diff1 = a_f32_1 * scale_vec - b_chunk1;
            let diff2 = a_f32_2 * scale_vec - b_chunk2;

            sum += diff1 * diff1 + diff2 * diff2;
            i += 8;
        }

        while i + 4 <= n {
            let a_chunk = i8x4::from_slice(&a.coords[i..i + 4]);
            let b_f64 = f64x4::from_slice(&b.coords[i..i + 4]);
            let b_chunk: f32x4 = b_f64.cast();
            let a_f32: f32x4 = a_chunk.cast();
            let diff = a_f32 * scale_vec - b_chunk;
            sum += diff * diff;
            i += 4;
        }

        let mut total_sum = sum.reduce_sum() as f64;

        while i < n {
            let a_val = f32::from(a.coords[i]) * SCALE_INV;
            let diff = a_val - (b.coords[i] as f32);
            total_sum += (diff * diff) as f64;
            i += 1;
        }

        total_sum
    }

    #[cfg(not(feature = "nightly-simd"))]
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        const LANES: usize = 8;
        let n = a.coords.len().min(b.coords.len());
        let mut acc = [0.0_f64; LANES];
        let chunks_a = a.coords[..n].chunks_exact(LANES);
        let chunks_b = b.coords[..n].chunks_exact(LANES);
        let rem_a = chunks_a.remainder();
        let rem_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            for l in 0..LANES {
                let a_val = f64::from(ca[l]) * SCALE_INV;
                let diff = a_val - cb[l];
                acc[l] += diff * diff;
            }
        }
        let mut sum = 0.0_f64;
        for l in 0..LANES {
            sum += acc[l];
        }
        for (x, y) in rem_a.iter().zip(rem_b.iter()) {
            let a_val = f64::from(*x) * SCALE_INV;
            let diff = a_val - *y;
            sum += diff * diff;
        }
        sum
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        let mut result = vec![0.0; past.len()];
        for i in 0..past.len() {
            let tangent = current[i] - past[i];
            result[i] = current[i] + steps * tangent;
        }
        Ok(result)
    }
}

pub struct CosineMetric;

impl Metric for CosineMetric {
    fn name() -> &'static str {
        "cosine"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        <EuclideanMetric as Metric>::distance(a, b)
    }

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        EuclideanMetric::distance_quantized(a, b)
    }

    #[cfg(not(feature = "nightly-simd"))]
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        EuclideanMetric::distance_quantized(a, b)
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        // Cosine momentum is similar to Euclidean but conceptually on a sphere.
        // For simplicity, we use linear extrapolation.
        EuclideanMetric::extrapolate_momentum(past, current, steps)
    }
}

/// Inner Product (Dot Product) metric (`1.0 - a · b`), monotonically equivalent to `-a · b`
/// and equal to `0.0` for identical unit vectors.
pub struct DotProductMetric;

impl Metric for DotProductMetric {
    fn name() -> &'static str {
        "dot"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        let (chunks_a, rem_a) = a[..n].as_chunks::<4>();
        let (chunks_b, rem_b) = b[..n].as_chunks::<4>();
        let mut acc = [0.0_f64; 4];
        for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
            for l in 0..4 {
                acc[l] += ca[l] * cb[l];
            }
        }
        let mut dot = (acc[0] + acc[1]) + (acc[2] + acc[3]);
        for (&x, &y) in rem_a.iter().zip(rem_b.iter()) {
            dot += x * y;
        }
        1.0 - dot
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = a.coords.len().min(b.coords.len());
        let mut dot = 0.0_f64;
        for i in 0..n {
            dot += (f64::from(a.coords[i]) * SCALE_INV) * b.coords[i];
        }
        1.0 - dot
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        EuclideanMetric::extrapolate_momentum(past, current, steps)
    }
}

/// Manhattan / Taxicab ($L_1$) metric: $\sum_i |a_i - b_i|$.
pub struct ManhattanMetric;

impl Metric for ManhattanMetric {
    fn name() -> &'static str {
        "l1"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        let (chunks_a, rem_a) = a[..n].as_chunks::<4>();
        let (chunks_b, rem_b) = b[..n].as_chunks::<4>();
        let mut acc = [0.0_f64; 4];
        for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
            for l in 0..4 {
                acc[l] += (ca[l] - cb[l]).abs();
            }
        }
        let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
        for (&x, &y) in rem_a.iter().zip(rem_b.iter()) {
            sum += (x - y).abs();
        }
        sum
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = a.coords.len().min(b.coords.len());
        let mut sum = 0.0_f64;
        for i in 0..n {
            sum += (f64::from(a.coords[i]) * SCALE_INV - b.coords[i]).abs();
        }
        sum
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        EuclideanMetric::extrapolate_momentum(past, current, steps)
    }
}

/// Chebyshev ($L_\infty$) metric: $\max_i |a_i - b_i|$.
pub struct ChebyshevMetric;

impl Metric for ChebyshevMetric {
    fn name() -> &'static str {
        "linf"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        let mut max_diff = 0.0_f64;
        for i in 0..n {
            let d = (a[i] - b[i]).abs();
            if d > max_diff {
                max_diff = d;
            }
        }
        max_diff
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = a.coords.len().min(b.coords.len());
        let mut max_diff = 0.0_f64;
        for i in 0..n {
            let d = (f64::from(a.coords[i]) * SCALE_INV - b.coords[i]).abs();
            if d > max_diff {
                max_diff = d;
            }
        }
        max_diff
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        EuclideanMetric::extrapolate_momentum(past, current, steps)
    }
}

/// Geodesic Angular distance on the unit hypersphere: $\arccos(\cos(a, b)) / \pi \in [0, 1]$.
/// Unlike raw Cosine distance, Angular distance satisfies the strict triangle inequality.
pub struct AngularMetric;

impl Metric for AngularMetric {
    fn name() -> &'static str {
        "angular"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        let mut dot = 0.0_f64;
        let mut na = 0.0_f64;
        let mut nb = 0.0_f64;
        for i in 0..n {
            let x = a[i];
            let y = b[i];
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        if na <= 1e-24 || nb <= 1e-24 {
            return 1.0;
        }
        let cos = (dot / (na * nb).sqrt()).clamp(-1.0, 1.0);
        cos.acos() / std::f64::consts::PI
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = a.coords.len().min(b.coords.len());
        let mut dot = 0.0_f64;
        let mut na = 0.0_f64;
        let mut nb = 0.0_f64;
        for i in 0..n {
            let x = f64::from(a.coords[i]) * SCALE_INV;
            let y = b.coords[i];
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        if na <= 1e-24 || nb <= 1e-24 {
            return 1.0;
        }
        let cos = (dot / (na * nb).sqrt()).clamp(-1.0, 1.0);
        cos.acos() / std::f64::consts::PI
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        EuclideanMetric::extrapolate_momentum(past, current, steps)
    }
}

/// Continuous Jaccard / Tanimoto metric: $1 - \frac{a \cdot b}{\|a\|^2 + \|b\|^2 - a \cdot b}$.
pub struct JaccardMetric;

impl Metric for JaccardMetric {
    fn name() -> &'static str {
        "jaccard"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        let mut dot = 0.0_f64;
        let mut na = 0.0_f64;
        let mut nb = 0.0_f64;
        for i in 0..n {
            let x = a[i];
            let y = b[i];
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        let denom = na + nb - dot;
        if denom <= 1e-24 {
            return 0.0;
        }
        (1.0 - (dot / denom)).clamp(0.0, 2.0)
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = a.coords.len().min(b.coords.len());
        let mut dot = 0.0_f64;
        let mut na = 0.0_f64;
        let mut nb = 0.0_f64;
        for i in 0..n {
            let x = f64::from(a.coords[i]) * SCALE_INV;
            let y = b.coords[i];
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        let denom = na + nb - dot;
        if denom <= 1e-24 {
            return 0.0;
        }
        (1.0 - (dot / denom)).clamp(0.0, 2.0)
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }
}

/// Normalized Hamming distance over vector coordinates (sign/activation mismatch ratio).
pub struct HammingMetric;

impl Metric for HammingMetric {
    fn name() -> &'static str {
        "hamming"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let mut diff = 0usize;
        for i in 0..n {
            if (a[i] > 0.0) != (b[i] > 0.0) || (a[i] == 0.0) != (b[i] == 0.0) {
                diff += 1;
            }
        }
        diff as f64 / n as f64
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        let n = a.coords.len().min(b.coords.len());
        if n == 0 {
            return 0.0;
        }
        let mut diff = 0usize;
        for i in 0..n {
            if (a.coords[i] > 0) != (b.coords[i] > 0.0)
                || (a.coords[i] == 0) != (b.coords[i] == 0.0)
            {
                diff += 1;
            }
        }
        diff as f64 / n as f64
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }
}

#[derive(Clone, Default)]
pub struct WassersteinMetric;

impl Metric for WassersteinMetric {
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        wasserstein::WassersteinDistance::compute(a, b)
    }

    fn name() -> &'static str {
        "wasserstein"
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        let alpha = f64::from(a.alpha);
        let a_f64: Vec<f64> = a.coords.iter().map(|&c| f64::from(c) * alpha).collect();
        Self::distance(&a_f64, &b.coords)
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.adc_distance_to_float(b)
    }
}

/// Canonical list of built-in multi-metric spaces supported by HyperspaceDB.
pub const SUPPORTED_METRICS: &[&str] = &[
    "l2",
    "cosine",
    "dot",
    "l1",
    "linf",
    "angular",
    "jaccard",
    "hamming",
    "poincare",
    "lorentz",
    "hybrid",
    "wasserstein",
];

/// Dynamically evaluates distance between two vectors under the named metric.
pub fn compute_metric_distance(metric: &str, a: &[f64], b: &[f64]) -> Result<f64, String> {
    match metric.trim().to_ascii_lowercase().as_str() {
        "l2" | "euclidean" => Ok(EuclideanMetric::distance(a, b)),
        "cosine" => Ok(CosineMetric::distance(a, b)),
        "dot" | "ip" | "inner_product" => Ok(DotProductMetric::distance(a, b)),
        "l1" | "manhattan" | "taxicab" => Ok(ManhattanMetric::distance(a, b)),
        "linf" | "chebyshev" | "chessboard" => Ok(ChebyshevMetric::distance(a, b)),
        "angular" => Ok(AngularMetric::distance(a, b)),
        "jaccard" | "tanimoto" => Ok(JaccardMetric::distance(a, b)),
        "hamming" => Ok(HammingMetric::distance(a, b)),
        "poincare" => Ok(PoincareMetric::distance(a, b)),
        "lorentz" => Ok(LorentzMetric::distance(a, b)),
        "hybrid" => Ok(HybridMetric::distance(a, b)),
        "wasserstein" | "emd" | "sinkhorn" => Ok(WassersteinMetric::distance(a, b)),
        other => Err(format!("Unsupported metric '{other}'")),
    }
}
