#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]

pub mod config;
pub mod vector;

pub use config::GlobalConfig;

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMode {
    None,
    ScalarI8,
    Binary,
}

/// Metric abstraction for distance calculation
pub trait Metric<const N: usize>: Send + Sync + 'static {
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64;
}

pub struct PoincareMetric;


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
    fn insert(&self, vector: &[f64], id: u32, metadata: std::collections::HashMap<String, String>) -> Result<(), String>;
    fn delete(&self, id: u32) -> Result<(), String>;
    fn search(
        &self,
        query: &[f64],
        filters: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<(u32, f64)>, String>;
    fn count(&self) -> usize;
}

impl<const N: usize> Metric<N> for PoincareMetric {


    #[inline(always)]
    fn distance(a: &[f64; N], b: &[f64; N]) -> f64 {
        // We use the math from HyperVector::poincare_distance_sq here
        // But HyperVector stores precomputed alpha.
        // Metric trait assumes raw coordinates?
        // Or should Metric work on HyperVector?
        // User prompt Example: "fn distance(a: &[f32; N], b: &[f32; N])"
        // And implementation computes norm_sq inside.
        // This recalculates norms every time? That's slow.
        // But HNSW stores Vectors.
        // Ideally HnsIndex should call HyperVector::distance.
        // But HyperVector handles Poincare logic internally.
        // If we want to switch metric to Cosine, HyperVector logic (alpha) might be useless.
        // However, for MVP, let's follow the User's snippet pattern but Keep HyperVector optimization if possible.
        // Actually, existing code uses `HyperVector::poincare_distance_sq`.
        // If we switch to Generic M, `HnswIndex` will call `M::distance`.
        // We should move the optimized logic into `PoincareMetric::distance`.
        // AND we should probably pass HyperVector to it if we want to use precomputed alpha?
        // Or the User snippet implies recalculating norms is acceptable for "Generic" check?
        // Let's implement the User's logic for now (Raw calculation).
        // Optimization: We can specialize later or pass structs.

        let norm_u_sq: f64 = a.iter().map(|&x| x * x).sum();
        let norm_v_sq: f64 = b.iter().map(|&x| x * x).sum();
        let diff_sq: f64 = a.iter().zip(b.iter()).map(|(u, v)| (u - v).powi(2)).sum();

        let denom = (1.0 - norm_u_sq) * (1.0 - norm_v_sq);
        let arg = 1.0 + 2.0 * diff_sq / denom.max(1e-9);
        arg.acosh()
    }
}
