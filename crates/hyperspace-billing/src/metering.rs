use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use std::num::NonZeroU32;

/// Per-tenant usage accumulator (lock-free atomic counters).
#[derive(Default)]
pub struct UsageAccumulator {
    pub inserts: AtomicU64,
    pub searches: AtomicU64,
    pub payload_bytes: AtomicU64,
    pub storage_bytes: AtomicU64,
    /// When this tenant's balance was last confirmed as `active` (Unix seconds)
    pub last_sync_ts: AtomicU64,
    /// Whether the tenant is currently rate-limited due to insufficient funds
    pub throttled: AtomicU64, // 0 = active, 1 = throttled
}

/// Snapshot of a single tenant's accumulated usage (read-once, zero the atomics).
#[derive(Debug, Clone)]
pub struct UsageDelta {
    pub api_key: String,
    pub delta_inserts: u64,
    pub delta_searches: u64,
    pub payload_bytes: u64,
    pub storage_bytes: u64,
}

/// Global rate limiter (node-level DDoS guard — GCRA algorithm via `governor`).
pub type NodeRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Core in-memory metering engine.
#[derive(Clone)]
pub struct MeteringEngine {
    /// api_key → UsageAccumulator
    counters: Arc<DashMap<String, Arc<UsageAccumulator>>>,
    /// Global node-level rate limiter (max requests per second across all tenants)
    pub rate_limiter: NodeRateLimiter,
}

impl MeteringEngine {
    /// Create a new engine with a global burst cap.
    /// `max_rps` — max requests per second at the node level before GCRA kicks in.
    pub fn new(max_rps: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(max_rps).expect("max_rps must be > 0"));
        let rate_limiter = Arc::new(RateLimiter::direct(quota));
        Self {
            counters: Arc::new(DashMap::new()),
            rate_limiter,
        }
    }

    fn get_or_create(&self, api_key: &str) -> Arc<UsageAccumulator> {
        self.counters
            .entry(api_key.to_string())
            .or_insert_with(|| Arc::new(UsageAccumulator::default()))
            .clone()
    }

    /// Record a vector insert operation.
    pub fn record_insert(&self, api_key: &str, count: u64) {
        let acc = self.get_or_create(api_key);
        acc.inserts.fetch_add(count, Ordering::Relaxed);
    }

    /// Record a vector search operation.
    pub fn record_search(&self, api_key: &str, payload_bytes: u64) {
        let acc = self.get_or_create(api_key);
        acc.searches.fetch_add(1, Ordering::Relaxed);
        acc.payload_bytes.fetch_add(payload_bytes, Ordering::Relaxed);
    }

    /// Update persisted storage bytes for a tenant (overwrite, not delta).
    pub fn update_storage_bytes(&self, api_key: &str, bytes: u64) {
        let acc = self.get_or_create(api_key);
        acc.storage_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Check whether a tenant is currently throttled.
    pub fn is_throttled(&self, api_key: &str) -> bool {
        if let Some(acc) = self.counters.get(api_key) {
            acc.throttled.load(Ordering::Relaxed) == 1
        } else {
            false
        }
    }

    /// Mark a tenant as throttled (insufficient balance).
    pub fn set_throttled(&self, api_key: &str, throttled: bool) {
        let acc = self.get_or_create(api_key);
        acc.throttled.store(if throttled { 1 } else { 0 }, Ordering::Relaxed);
    }

    /// Drain accumulated deltas for all tenants, resetting the counters to zero.
    /// Returns only tenants that had non-zero activity since the last drain.
    pub fn drain_deltas(&self) -> Vec<UsageDelta> {
        let mut deltas = Vec::new();
        for entry in self.counters.iter() {
            let acc = entry.value();
            let inserts = acc.inserts.swap(0, Ordering::Relaxed);
            let searches = acc.searches.swap(0, Ordering::Relaxed);
            let payload_bytes = acc.payload_bytes.swap(0, Ordering::Relaxed);
            let storage_bytes = acc.storage_bytes.load(Ordering::Relaxed); // never reset, just read

            if inserts > 0 || searches > 0 || payload_bytes > 0 {
                deltas.push(UsageDelta {
                    api_key: entry.key().clone(),
                    delta_inserts: inserts,
                    delta_searches: searches,
                    payload_bytes,
                    storage_bytes,
                });
            }
        }
        deltas
    }
}
