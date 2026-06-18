use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec,
};
use std::sync::OnceLock;

pub struct CacheMetrics {
    pub l1_hits: CounterVec,
    pub l1_misses: CounterVec,
    pub l2_hits: CounterVec,
    pub l2_misses: CounterVec,
    pub evictions: CounterVec,
    pub inserts: CounterVec,

    pub l1_size: GaugeVec,
    pub l2_index_size: GaugeVec,
    pub l2_pending_size: GaugeVec,
    pub tombstone_size: GaugeVec,

    pub l1_latency: HistogramVec,
    pub l2_latency: HistogramVec,
    pub rebuild_duration: HistogramVec,
}

pub fn metrics() -> &'static CacheMetrics {
    static METRICS: OnceLock<CacheMetrics> = OnceLock::new();
    METRICS.get_or_init(|| {
        let l1_hits = register_counter_vec!(
            "hyperspace_cache_l1_hits_total",
            "Total number of L1 cache hits",
            &["collection"]
        )
        .unwrap();

        let l1_misses = register_counter_vec!(
            "hyperspace_cache_l1_misses_total",
            "Total number of L1 cache misses",
            &["collection"]
        )
        .unwrap();

        let l2_hits = register_counter_vec!(
            "hyperspace_cache_l2_hits_total",
            "Total number of L2 ANN cache hits",
            &["collection"]
        )
        .unwrap();

        let l2_misses = register_counter_vec!(
            "hyperspace_cache_l2_misses_total",
            "Total number of L2 ANN cache misses",
            &["collection"]
        )
        .unwrap();

        let evictions = register_counter_vec!(
            "hyperspace_cache_evictions_total",
            "Total number of cache evictions",
            &["collection", "reason"]
        )
        .unwrap();

        let inserts = register_counter_vec!(
            "hyperspace_cache_inserts_total",
            "Total number of inserts into cache",
            &["collection"]
        )
        .unwrap();

        let l1_size = register_gauge_vec!(
            "hyperspace_cache_l1_size",
            "Current number of entries in L1 cache",
            &["collection"]
        )
        .unwrap();

        let l2_index_size = register_gauge_vec!(
            "hyperspace_cache_l2_index_size",
            "Current number of vectors in L2 HNSW index",
            &["collection"]
        )
        .unwrap();

        let l2_pending_size = register_gauge_vec!(
            "hyperspace_cache_l2_pending_size",
            "Current size of L2 pending rebuild queue",
            &["collection"]
        )
        .unwrap();

        let tombstone_size = register_gauge_vec!(
            "hyperspace_cache_tombstone_size",
            "Current number of tombstone markers in L2 index",
            &["collection"]
        )
        .unwrap();

        let l1_latency = register_histogram_vec!(
            "hyperspace_cache_l1_latency_seconds",
            "L1 cache lookup latency in seconds",
            &["collection"]
        )
        .unwrap();

        let l2_latency = register_histogram_vec!(
            "hyperspace_cache_l2_latency_seconds",
            "L2 ANN cache lookup latency in seconds",
            &["collection"]
        )
        .unwrap();

        let rebuild_duration = register_histogram_vec!(
            "hyperspace_cache_rebuild_duration_seconds",
            "L2 HNSW index rebuild duration in seconds",
            &["collection"]
        )
        .unwrap();

        CacheMetrics {
            l1_hits,
            l1_misses,
            l2_hits,
            l2_misses,
            evictions,
            inserts,
            l1_size,
            l2_index_size,
            l2_pending_size,
            tombstone_size,
            l1_latency,
            l2_latency,
            rebuild_duration,
        }
    })
}
