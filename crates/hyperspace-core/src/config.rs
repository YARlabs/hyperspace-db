use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

/// Global runtime configuration for HyperspaceDB
/// Thread-safe via atomics, can be changed on-the-fly
#[derive(Debug)]
pub struct GlobalConfig {
    /// ef_search: Search depth (higher = more accurate, slower)
    pub ef_search: AtomicUsize,
    
    /// ef_construction: Build quality (higher = better graph, slower indexing)
    pub ef_construction: AtomicUsize,
    
    /// Queue size tracking for monitoring
    pub queue_size: AtomicU64,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            ef_search: AtomicUsize::new(100),        // Default
            ef_construction: AtomicUsize::new(100),  // Default
            queue_size: AtomicU64::new(0),
        }
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
    
    pub fn inc_queue(&self) {
        self.queue_size.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn dec_queue(&self) {
        self.queue_size.fetch_sub(1, Ordering::Relaxed);
    }
    
    pub fn get_queue_size(&self) -> u64 {
        self.queue_size.load(Ordering::Relaxed)
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}
