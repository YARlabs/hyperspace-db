#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]

pub mod config;
pub mod vector;

pub use config::GlobalConfig;
use vector::{BinaryHyperVector, HyperVector, QuantizedHyperVector};

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

pub trait Collection: Send + Sync {
    fn name(&self) -> &str;
    fn insert(
        &self,
        vector: &[f64],
        id: u32,
        metadata: std::collections::HashMap<String, String>,
        clock: u64,
    ) -> Result<(), String>;
    fn delete(&self, id: u32) -> Result<(), String>;
    fn search(
        &self,
        query: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<(u32, f64)>, String>;
    fn count(&self) -> usize;
    fn dimension(&self) -> usize;
    fn metric_name(&self) -> &'static str;
    fn state_hash(&self) -> u64;
    fn buckets(&self) -> Vec<u64>; // New method
    fn peek(&self, limit: usize)
        -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)>;
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

    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        // Squared L2 distance for optimization (sqrt is monotonic)
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
    }

    // validate uses default

    fn distance_quantized(a: &QuantizedHyperVector<N>, b: &HyperVector<N>) -> f64 {
        let mut sum_sq_diff = 0.0;
        const SCALE_INV: f64 = 1.0 / 127.0;

        for (a_i8, b_f64) in a.coords.iter().zip(b.coords.iter()) {
            let a_val = (*a_i8 as f64) * SCALE_INV;
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
