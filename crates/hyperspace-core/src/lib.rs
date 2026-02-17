#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::inline_always)]
#![allow(clippy::similar_names)]

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

#[derive(Debug, Clone)]
pub enum FilterExpr {
    Match {
        key: String,
        value: String,
    },
    Range {
        key: String,
        gte: Option<i64>,
        lte: Option<i64>,
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
    fn peek(&self, limit: usize)
        -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)>;
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

impl<const N: usize> Metric<N> for EuclideanMetric {
    fn name() -> &'static str {
        "l2"
    }

    #[cfg(feature = "nightly-simd")]
    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        use std::simd::f64x8;
        use std::simd::num::SimdFloat; // for reduce_sum

        let mut sum = f64x8::splat(0.0);
        let mut i = 0;
        const LANES: usize = 8;

        while i + LANES <= N {
            let va = f64x8::from_slice(&a[i..i + LANES]);
            let vb = f64x8::from_slice(&b[i..i + LANES]);
            let diff = va - vb;
            sum += diff * diff;
            i += LANES;
        }

        let mut total = sum.reduce_sum();

        // Scalar Tail
        while i < N {
            let diff = a[i] - b[i];
            total += diff * diff;
            i += 1;
        }
        total
    }

    #[cfg(not(feature = "nightly-simd"))]
    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        // Explicit loop assists LLVM auto-vectorization.
        let mut sum = 0.0;
        for i in 0..N {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum
    }

    // validate uses default

    #[cfg(feature = "nightly-simd")]
    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        use std::simd::num::{SimdFloat, SimdInt};
        use std::simd::{f64x8, i8x8}; // Import needed traits

        const LANES: usize = 8;
        const SCALE_INV: f64 = 1.0 / 127.0;
        let scale_vec = f64x8::splat(SCALE_INV);

        let mut sum = f64x8::splat(0.0);
        let mut i = 0;

        // SIMD Loop
        while i + LANES <= N {
            let a_chunk = i8x8::from_slice(&a.coords[i..i + LANES]);
            let b_chunk = f64x8::from_slice(&b.coords[i..i + LANES]);

            // Vectorized cast i8 -> f64
            let a_f64: f64x8 = a_chunk.cast();

            let a_scaled = a_f64 * scale_vec;
            let diff = a_scaled - b_chunk;
            sum += diff * diff;

            i += LANES;
        }

        let mut total_sum = sum.reduce_sum();

        // Scalar Tail
        while i < N {
            let a_val = f64::from(a.coords[i]) * SCALE_INV;
            let diff = a_val - b.coords[i];
            total_sum += diff * diff;
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
