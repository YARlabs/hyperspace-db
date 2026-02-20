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
pub mod vector;

pub use config::GlobalConfig;
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

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMode {
    None,
    ScalarI8,
    Binary,
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
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub top_k: usize,
    pub ef_search: usize,
    pub hybrid_query: Option<String>,
    pub hybrid_alpha: Option<f32>,
}

pub type SearchResult = (u32, f64, std::collections::HashMap<String, String>);

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
    async fn optimize(&self) -> Result<(), String> {
        // Default: No-op for collections lacking optimization support.
        Ok(())
    }
    async fn optimize_with_filter(&self, filter: Option<VacuumFilterQuery>) -> Result<(), String> {
        let _ = filter;
        self.optimize().await
    }
    fn peek(&self, limit: usize)
        -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)>;
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
    fn metadata_by_id(&self, id: u32) -> std::collections::HashMap<String, String>;
    fn quantization_mode(&self) -> QuantizationMode;
}

pub trait Metric<const N: usize>: Send + Sync + 'static {
    fn name() -> &'static str;
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64;

    // Default valid verification (Euclidean accepts all)
    fn validate(vector: &[f64; N]) -> Result<(), String> {
        let _ = vector;
        Ok(())
    }

    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64;
    fn distance_binary(a: &BinaryHyperVector<N>, b: &HyperVector<N>) -> f64;
}

impl<const N: usize> Metric<N> for PoincareMetric {
    fn name() -> &'static str {
        "poincare"
    }

    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        let norm_u_sq: f64 = a.iter().map(|&x| x * x).sum();
        let norm_v_sq: f64 = b.iter().map(|&x| x * x).sum();
        let diff_sq: f64 = a.iter().zip(b.iter()).map(|(u, v)| (u - v).powi(2)).sum();

        let denom = (1.0 - norm_u_sq) * (1.0 - norm_v_sq);
        let arg = 1.0 + 2.0 * diff_sq / denom.max(1e-9);
        arg.acosh()
    }

    fn validate(vector: &[f64; N]) -> Result<(), String> {
        let sq_norm: f64 = vector.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector must be strictly inside the Poincaré ball".to_string());
        }
        Ok(())
    }

    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        a.poincare_distance_sq_to_float(b)
    }

    fn distance_binary(a: &BinaryHyperVector<N>, b: &HyperVector<N>) -> f64 {
        a.poincare_distance_sq_to_float(b)
    }
}

impl<const N: usize> Metric<N> for LorentzMetric {
    fn name() -> &'static str {
        "lorentz"
    }

    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        debug_assert!(N >= 2, "Lorentz metric requires at least 2 dimensions");
        let mut inner_prod = -a[0] * b[0];
        for i in 1..N {
            inner_prod += a[i] * b[i];
        }
        // Hyperboloid distance: d(x, y) = arcosh(-<x, y>_L), where <.,.>_L uses signature (-, +, ... ,+).
        let arg = (-inner_prod).max(1.0 + 1e-12);
        arg.acosh()
    }

    fn validate(vector: &[f64; N]) -> Result<(), String> {
        if !vector.iter().all(|v| v.is_finite()) {
            return Err("Lorentz vector contains non-finite values".to_string());
        }
        if vector[0] <= 0.0 {
            return Err("Lorentz vector must be on the upper sheet (t > 0)".to_string());
        }

        // Point must satisfy: -t^2 + x1^2 + ... + xn^2 = -1
        let mut spatial_sq = 0.0_f64;
        for i in 1..N {
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

    fn distance_quantized(_a: &QuantizedHyperVector<N>, _b: &HyperVector<N>) -> f64 {
        panic!("Scalar quantization is not yet supported for the Lorentz model. Use HS_QUANTIZATION_LEVEL=none");
    }

    fn distance_binary(_a: &BinaryHyperVector<N>, _b: &HyperVector<N>) -> f64 {
        panic!("Binary quantization is not supported for Lorentz.");
    }
}

impl<const N: usize> Metric<N> for EuclideanMetric {
    fn name() -> &'static str {
        "l2"
    }

    #[cfg(feature = "nightly-simd")]
    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        use std::simd::f32x8;
        use std::simd::num::SimdFloat;

        let mut sum = f32x8::splat(0.0);
        let mut i = 0;
        const LANES: usize = 8;

        while i + LANES <= N {
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

        // Scalar Tail
        while i < N {
            let diff = (a[i] as f32) - (b[i] as f32);
            total += (diff * diff) as f64;
            i += 1;
        }
        total
    }

    #[cfg(not(feature = "nightly-simd"))]
    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        // Euclidean path uses f32 math by design.
        // Hyperbolic workloads remain on f64 in `PoincareMetric`.
        let mut sum = 0.0f32;
        for i in 0..N {
            let diff = (a[i] as f32) - (b[i] as f32);
            sum += diff * diff;
        }
        f64::from(sum)
    }

    // validate uses default

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        use std::simd::num::{SimdFloat, SimdInt};
        use std::simd::{f32x8, i8x8};

        const LANES: usize = 8;
        const SCALE_INV: f32 = 1.0 / 127.0;
        let scale_vec = f32x8::splat(SCALE_INV);

        let mut sum = f32x8::splat(0.0);
        let mut i = 0;

        // SIMD Loop
        while i + LANES <= N {
            // 1. Load quantized vector (i8)
            let a_chunk = i8x8::from_slice(&a.coords[i..i + LANES]);

            // 2. Load Query (f64) and convert to f32
            let mut query_buf = [0.0f32; LANES];
            for k in 0..LANES {
                query_buf[k] = b.coords[i + k] as f32;
            }
            let b_chunk = f32x8::from_slice(&query_buf);

            // 3. Vectorized cast i8 -> f32
            let a_f32: f32x8 = a_chunk.cast();

            // 4. Math in f32 (AVX2/AVX512 friendly)
            let a_scaled = a_f32 * scale_vec;
            let diff = a_scaled - b_chunk;
            sum += diff * diff;

            i += LANES;
        }

        let mut total_sum = sum.reduce_sum() as f64;

        // Scalar Tail
        while i < N {
            let a_val = f32::from(a.coords[i]) * SCALE_INV;
            let diff = a_val - (b.coords[i] as f32);
            total_sum += (diff * diff) as f64;
            i += 1;
        }

        total_sum
    }

    #[cfg(not(feature = "nightly-simd"))]
    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        const SCALE_INV: f64 = 1.0 / 127.0;
        let mut sum_sq_diff = 0.0;
        for (a_i8, b_f64) in a.coords.iter().zip(b.coords.iter()) {
            let a_val = f64::from(*a_i8) * SCALE_INV;
            let diff = a_val - b_f64;
            sum_sq_diff += diff * diff;
        }
        sum_sq_diff
    }

    // Binary implementation calls the method added to vector struct
    fn distance_binary(a: &BinaryHyperVector<N>, b: &HyperVector<N>) -> f64 {
        a.l2_distance_sq_to_float(b)
    }
}

/// Cosine metric for normalized vectors.
/// Uses squared L2 distance to preserve graph geometry for HNSW.
#[derive(Debug, Clone, Copy)]
pub struct CosineMetric;

impl<const N: usize> Metric<N> for CosineMetric {
    fn name() -> &'static str {
        "cosine"
    }

    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        // Cosine distance implementation: equivalent to squared Euclidean distance on normalized vectors.
        // Ranking is preserved and triangle inequality holds.
        <EuclideanMetric as Metric<N>>::distance(a, b)
    }

    // validate uses default

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        // Re-use Euclidean logic as Cosine is just L2 on normalized vectors
        EuclideanMetric::distance_quantized(a, b)
    }

    #[cfg(not(feature = "nightly-simd"))]
    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        EuclideanMetric::distance_quantized(a, b)
    }

    fn distance_binary(a: &BinaryHyperVector<N>, b: &HyperVector<N>) -> f64 {
        // Approximates Hamming distance for binary vectors.
        a.l2_distance_sq_to_float(b)
    }
}
