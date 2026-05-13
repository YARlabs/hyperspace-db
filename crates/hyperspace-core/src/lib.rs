#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::inline_always)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_range_loop)]

pub mod config;
pub mod fuzzy;
pub mod gpu;
pub mod gromov;
pub mod optim;
pub mod region;
pub mod vector;
pub mod hybrid;
pub use hybrid::HybridMetric;
pub mod wasserstein;

pub use config::GlobalConfig;
pub mod bm25;
pub use bm25::*;
use vector::{BinaryHyperVector, HyperVector, QuantizedHyperVector};

#[cfg(feature = "nightly-simd")]
pub fn check_simd() {
    println!("🚀 SIMD Acceleration: ENABLED (AVX/Neon)");
}

#[cfg(not(feature = "nightly-simd"))]
pub fn check_simd() {
    println!("🐢 SIMD Acceleration: DISABLED (Scalar Fallback)");
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod hybrid_tests;

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QuantizationMode {
    None,
    ScalarI8,
    Binary,
    AsymmetricHybrid801,
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
        match self {
            Self::Match { key, value } => metadata.get(key) == Some(value),
            Self::Range { key, gte, lte } => {
                if let Some(val_str) = metadata.get(key) {
                    if let Ok(val) = val_str.parse::<f64>() {
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
                }
                false
            }
            Self::InCone {
                axes,
                apertures,
                cen,
            } => {
                let region = region::ConeRegion::new(axes.clone(), apertures.clone(), *cen);
                region.contains(vector)
            }
            Self::InBox {
                min_bounds,
                max_bounds,
            } => {
                let region = region::BoxRegion::new(min_bounds.clone(), max_bounds.clone());
                region.contains(vector)
            }
            Self::InBall { center, radius } => {
                let region = region::BallRegion::new(center.clone(), *radius);
                region.contains(vector)
            }
            Self::And(conds) => conds.iter().all(|c| c.check(vector, metadata)),
            Self::Or(conds)  => conds.iter().any(|c| c.check(vector, metadata)),
            Self::Not(cond)  => !cond.check(vector, metadata),
        }
    }

    /// Metadata-only check (no vector). Geometric filters (`InCone`, `InBall`, `InBox`)
    /// are skipped (return `true`) because vector data is unavailable in scroll context.
    pub fn check_meta(
        &self,
        metadata: &std::collections::HashMap<String, String>,
    ) -> bool {
        match self {
            Self::Match { key, value } => metadata.get(key) == Some(value),
            Self::Range { key, gte, lte } => {
                if let Some(val_str) = metadata.get(key) {
                    if let Ok(val) = val_str.parse::<f64>() {
                        if let Some(g) = gte { if val < *g { return false; } }
                        if let Some(l) = lte { if val > *l { return false; } }
                        return true;
                    }
                }
                false
            }
            // Geometric filters need vector data; pass-through in meta-only context.
            Self::InCone { .. } | Self::InBox { .. } | Self::InBall { .. } => true,
            Self::And(conds) => conds.iter().all(|c| c.check_meta(metadata)),
            Self::Or(conds)  => conds.iter().any(|c| c.check_meta(metadata)),
            Self::Not(cond)  => !cond.check_meta(metadata),
        }
    }
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
}

/// A single search result: (id, distance, metadata, optional_payload).
/// The `payload` field is `Some(bytes)` ONLY when `SearchParams::include_payload = true`
/// AND the vector was inserted with a payload. The bytes are the original uncompressed
/// document — all decompression is performed server-side via lazy disk I/O.
pub type SearchResult = (u32, f64, std::collections::HashMap<String, String>, Option<Vec<u8>>);

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
    fn delete(&self, id: u32) -> Result<(), String>;
    async fn search(
        &self,
        vector: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResult>, String>;
    fn count(&self) -> usize;
    fn dimension(&self) -> usize;
    fn metric_name(&self) -> &'static str;
    fn state_hash(&self) -> u64;
    fn buckets(&self) -> Vec<u64>; // New method
    fn queue_size(&self) -> u64; // Indexing queue size for eventual consistency
    fn quantization_mode(&self) -> crate::QuantizationMode;
    fn get_usage(&self) -> CollectionUsage;
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
    ) -> Result<Vec<u32>, String>;
    fn graph_clusters(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Result<Vec<Vec<u32>>, String>;

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

    /// Patch-update a point's metadata without touching the vector.
    fn update_payload(
        &self,
        id: u32,
        patch: std::collections::HashMap<String, String>,
    ) -> Result<(), String>;

    /// Look up the metadata HashMap for a given user-facing vector ID.
    fn metadata_by_id(&self, id: u32) -> std::collections::HashMap<String, String>;

    /// Write a heavy payload blob to the disk-only Payload Layer.
    /// Called AFTER a successful `insert()`. The payload is zstd-compressed
    /// before hitting disk. This method MUST NOT store any bytes in RAM.
    /// Default no-op: implementors without payload support silently ignore it.
    async fn insert_payload(&self, _id: u32, _payload: Vec<u8>) -> Result<(), String> {
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
}

impl Metric for PoincareMetric {
    fn name() -> &'static str {
        "poincare"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let norm_u_sq: f64 = a.iter().map(|&x| x * x).sum();
        let norm_v_sq: f64 = b.iter().map(|&x| x * x).sum();
        let diff_sq: f64 = a.iter().zip(b.iter()).map(|(u, v)| (u - v).powi(2)).sum();

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
}

impl Metric for LorentzMetric {
    fn name() -> &'static str {
        "lorentz"
    }

    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        debug_assert!(a.len() >= 2, "Lorentz metric requires at least 2 dimensions");
        let mut inner_prod = -a[0] * b[0];
        for i in 1..a.len() {
            inner_prod += a[i] * b[i];
        }
        let arg = (-inner_prod).max(1.0 + 1e-12);
        arg.acosh()
    }

    fn validate(vector: &[f64]) -> Result<(), String> {
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
        if (minkowski_norm + 1.0).abs() > 1e-6 {
            return Err(format!(
                "Lorentz vector is not on unit hyperboloid: -t^2+|x|^2={minkowski_norm}, expected -1"
            ));
        }
        Ok(())
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        a.lorentz_distance_to_float(b)
    }

    fn distance_binary(_a: &BinaryHyperVector, _b: &HyperVector) -> f64 {
        panic!("Binary quantization is not supported for the Lorentz model.");
    }
}

impl Metric for EuclideanMetric {
    fn name() -> &'static str {
        "l2"
    }

    #[cfg(feature = "nightly-simd")]
    #[inline(always)]
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        use std::simd::f32x8;
        use std::simd::num::SimdFloat;

        let mut sum = f32x8::splat(0.0);
        let mut i = 0;
        const LANES: usize = 8;
        let n = a.len();

        while i + LANES <= n {
            let mut a_buf = [0.0f32; LANES];
            let mut b_buf = [0.0f32; LANES];
            for k in 0..LANES {
                a_buf[k] = a[i + k] as f32;
                b_buf[k] = b[i + k] as f32;
            }
            let va = f32x8::from_slice(&a_buf);
            let vb = f32x8::from_slice(&b_buf);
            let diff = va - vb;
            sum += diff * diff;
            i += LANES;
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
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        let mut sum = 0.0f32;
        let n = a.len();
        for i in 0..n {
            let diff = (a[i] as f32) - (b[i] as f32);
            sum += diff * diff;
        }
        f64::from(sum)
    }

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        use std::simd::num::{SimdFloat, SimdInt};
        use std::simd::{f32x8, i8x8};

        const LANES: usize = 8;
        const SCALE_INV: f32 = 1.0 / 127.0;
        let scale_vec = f32x8::splat(SCALE_INV);

        let mut sum = f32x8::splat(0.0);
        let mut i = 0;
        let n = a.coords.len();

        while i + LANES <= n {
            let a_chunk = i8x8::from_slice(&a.coords[i..i + LANES]);
            let mut query_buf = [0.0f32; LANES];
            for k in 0..LANES {
                query_buf[k] = b.coords[i + k] as f32;
            }
            let b_chunk = f32x8::from_slice(&query_buf);

            let a_f32: f32x8 = a_chunk.cast();
            let a_scaled = a_f32 * scale_vec;
            let diff = a_scaled - b_chunk;
            sum += diff * diff;
            i += LANES;
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
    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let mut sum_sq_diff = 0.0;
        for (a_i8, b_f64) in a.coords.iter().zip(b.coords.iter()) {
            let a_val = f64::from(*a_i8) * SCALE_INV;
            let diff = a_val - b_f64;
            sum_sq_diff += diff * diff;
        }
        sum_sq_diff
    }

    fn distance_binary(a: &BinaryHyperVector, b: &HyperVector) -> f64 {
        a.poincare_distance_sq_to_float(b)
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
        a.poincare_distance_sq_to_float(b)
    }
}
