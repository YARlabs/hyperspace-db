use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

/// Global runtime configuration for `HyperspaceDB`
/// Thread-safe via atomics, can be changed on-the-fly
#[derive(Debug)]
pub struct GlobalConfig {
    /// `ef_search`: Search depth (higher = more accurate, slower)
    pub ef_search: AtomicUsize,

    /// `ef_construction`: Build quality (higher = better graph, slower indexing)
    pub ef_construction: AtomicUsize,

    /// Queue size tracking for monitoring (tasks in channel)
    pub queue_size: AtomicU64,

    /// Active indexing tasks (being processed right now)
    pub active_indexing: AtomicU64,

    /// `m`: Max connections per layer (dynamic)
    pub m: AtomicUsize,

    /// Whether Anti-Entropy (Gossip) hashing is enabled on the hot path
    pub gossip_enabled: AtomicBool,

    /// Whether to apply expensive Anisotropic Coordinate Descent refinement during quantization
    pub anisotropic_refinement: AtomicBool,

    /// BM25 scoring parameters
    pub bm25_params: std::sync::RwLock<crate::bm25::Bm25Params>,

    /// Fusion method ("rrf" or "weighted")
    pub fusion_method: std::sync::RwLock<String>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            ef_search: AtomicUsize::new(100),       // Default
            ef_construction: AtomicUsize::new(100), // Default
            queue_size: AtomicU64::new(0),
            active_indexing: AtomicU64::new(0),
            m: AtomicUsize::new(16),
            gossip_enabled: AtomicBool::new(false),
            anisotropic_refinement: AtomicBool::new(true), // Default to true for quality, but can be disabled for speed
            bm25_params: std::sync::RwLock::new(crate::bm25::Bm25Params::default()),
            fusion_method: std::sync::RwLock::new("rrf".to_string()),
        }
    }

    pub fn is_gossip_enabled(&self) -> bool {
        self.gossip_enabled.load(Ordering::Relaxed)
    }

    pub fn set_gossip_enabled(&self, val: bool) {
        self.gossip_enabled.store(val, Ordering::Relaxed);
    }

    pub fn is_anisotropic_enabled(&self) -> bool {
        self.anisotropic_refinement.load(Ordering::Relaxed)
    }

    pub fn set_anisotropic_enabled(&self, val: bool) {
        self.anisotropic_refinement.store(val, Ordering::Relaxed);
    }

    pub fn get_ef_search(&self) -> usize {
        self.ef_search.load(Ordering::Relaxed)
    }

    pub fn set_ef_search(&self, val: usize) {
        self.ef_search.store(val, Ordering::Relaxed);
    }

    pub fn get_ef_construction(&self) -> usize {
        self.ef_construction.load(Ordering::Relaxed)
    }

    pub fn set_ef_construction(&self, val: usize) {
        self.ef_construction.store(val, Ordering::Relaxed);
    }

    pub fn get_m(&self) -> usize {
        self.m.load(Ordering::Relaxed)
    }

    pub fn set_m(&self, val: usize) {
        self.m.store(val, Ordering::Relaxed);
    }

    pub fn inc_queue(&self) {
        self.queue_size.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_queue(&self) {
        self.queue_size.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get_queue_size(&self) -> u64 {
        // Return total pending work.
        // Since we dec_queue only after processing, queue_size includes active items.
        self.queue_size.load(Ordering::Relaxed)
    }

    pub fn inc_active(&self) {
        self.active_indexing.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_active(&self) {
        self.active_indexing.fetch_sub(1, Ordering::Relaxed);
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get_bm25_params(&self) -> crate::bm25::Bm25Params {
        self.bm25_params.read().unwrap().clone()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn set_bm25_params(&self, params: crate::bm25::Bm25Params) {
        *self.bm25_params.write().unwrap() = params;
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get_fusion_method(&self) -> String {
        self.fusion_method.read().unwrap().clone()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn set_fusion_method(&self, method: String) {
        *self.fusion_method.write().unwrap() = method;
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}
