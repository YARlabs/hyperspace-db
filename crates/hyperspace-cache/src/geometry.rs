use hyperspace_core::{CosineMetric, EuclideanMetric, LorentzMetric, Metric, PoincareMetric};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CacheMetricType {
    L2,
    Cosine,
    Poincare,
    Lorentz,
    Hybrid { lorentz_weight: f64, l2_weight: f64 },
}

pub trait CacheDistance: Send + Sync + 'static {
    fn metric_name(&self) -> &'static str;
    fn metric_type(&self) -> CacheMetricType;
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool;
}

pub struct PoincareCacheDistance;
impl CacheDistance for PoincareCacheDistance {
    fn metric_name(&self) -> &'static str {
        "poincare"
    }
    fn metric_type(&self) -> CacheMetricType {
        CacheMetricType::Poincare
    }
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        <PoincareMetric as Metric>::distance(a, b)
    }
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool {
        distance <= threshold
    }
}

pub struct LorentzCacheDistance;
impl CacheDistance for LorentzCacheDistance {
    fn metric_name(&self) -> &'static str {
        "lorentz"
    }
    fn metric_type(&self) -> CacheMetricType {
        CacheMetricType::Lorentz
    }
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        <LorentzMetric as Metric>::distance(a, b)
    }
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool {
        distance <= threshold
    }
}

pub struct L2CacheDistance;
impl CacheDistance for L2CacheDistance {
    fn metric_name(&self) -> &'static str {
        "l2"
    }
    fn metric_type(&self) -> CacheMetricType {
        CacheMetricType::L2
    }
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        <EuclideanMetric as Metric>::distance(a, b)
    }
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool {
        distance <= threshold
    }
}

pub struct CosineCacheDistance;
impl CacheDistance for CosineCacheDistance {
    fn metric_name(&self) -> &'static str {
        "cosine"
    }
    fn metric_type(&self) -> CacheMetricType {
        CacheMetricType::Cosine
    }
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        <CosineMetric as Metric>::distance(a, b)
    }
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool {
        // cosine distance in hyperspace-core is L2 distance of normalized vectors.
        // smaller L2 distance means higher cosine similarity (and closer distance).
        distance <= threshold
    }
}

pub struct HybridCacheDistance {
    pub lorentz_weight: f64,
    pub l2_weight: f64,
}
impl CacheDistance for HybridCacheDistance {
    fn metric_name(&self) -> &'static str {
        "hybrid"
    }
    fn metric_type(&self) -> CacheMetricType {
        CacheMetricType::Hybrid {
            lorentz_weight: self.lorentz_weight,
            l2_weight: self.l2_weight,
        }
    }
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() < 33 || b.len() < 33 {
            return 0.0;
        }
        let d_lor = <LorentzMetric as Metric>::distance(&a[..33], &b[..33]);
        let d_euc = <EuclideanMetric as Metric>::distance(&a[33..], &b[33..]);
        self.lorentz_weight * d_lor + self.l2_weight * d_euc
    }
    fn is_cache_hit(&self, distance: f64, threshold: f64) -> bool {
        distance <= threshold
    }
}

#[derive(Clone, Debug)]
pub struct CacheVector {
    pub coords: Vec<f64>,
    pub id: u32,
    pub metric_type: CacheMetricType,
}

impl instant_distance::Point for CacheVector {
    fn distance(&self, other: &Self) -> f32 {
        let dist = match &self.metric_type {
            CacheMetricType::L2 => {
                <EuclideanMetric as Metric>::distance(&self.coords, &other.coords)
            }
            CacheMetricType::Cosine => {
                <CosineMetric as Metric>::distance(&self.coords, &other.coords)
            }
            CacheMetricType::Poincare => {
                <PoincareMetric as Metric>::distance(&self.coords, &other.coords)
            }
            CacheMetricType::Lorentz => {
                <LorentzMetric as Metric>::distance(&self.coords, &other.coords)
            }
            CacheMetricType::Hybrid {
                lorentz_weight,
                l2_weight,
            } => {
                if self.coords.len() < 33 || other.coords.len() < 33 {
                    0.0
                } else {
                    let d_lor = <LorentzMetric as Metric>::distance(
                        &self.coords[..33],
                        &other.coords[..33],
                    );
                    let d_euc = <EuclideanMetric as Metric>::distance(
                        &self.coords[33..],
                        &other.coords[33..],
                    );
                    lorentz_weight * d_lor + l2_weight * d_euc
                }
            }
        };
        // The `instant_distance` trait requires f32. Our f64 distances are in range [0, very_large]
        // and represent squared Euclidean or similar bounded metrics; truncation is acceptable here.
        #[allow(clippy::cast_possible_truncation)]
        {
            dist as f32
        }
    }
}
