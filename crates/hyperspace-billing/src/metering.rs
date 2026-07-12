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
    /// Accumulated cost in micro-USD (1 = $0.000001)
    /// Includes: insert cost + search cost + storage rental cost per tick
    pub cost_microusd: AtomicU64,
    /// When this tenant's balance was last confirmed as `active` (Unix seconds)
    pub last_sync_ts: AtomicU64,
    /// Whether the tenant is currently rate-limited due to insufficient funds
    pub throttled: AtomicU64, // 0 = active, 1 = throttled
    /// Unix timestamp when grace period ends and data MAY be deleted.
    /// 0 = not in grace period (balance OK or never ran out)
    pub grace_period_ends_at: AtomicU64,
}

/// Snapshot of a single tenant's accumulated usage (read-once, zero the atomics).
#[derive(Debug, Clone)]
pub struct UsageDelta {
    pub api_key: String,
    pub delta_inserts: u64,
    pub delta_searches: u64,
    pub payload_bytes: u64,
    pub storage_bytes: u64,
    /// Total cost accumulated since last drain, in micro-USD
    /// Includes insert + search + storage rental.
    pub cost_microusd: u64,
}

/// Global rate limiter (node-level DDoS guard — GCRA algorithm via `governor`).
pub type NodeRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Pricing — read once from env on first use.
/// Override with:
///   HS_PRICE_INSERT_MICROUSD      (default: 1   = $0.000001 per vector)
///   HS_PRICE_SEARCH_MICROUSD      (default: 10  = $0.00001  per search)
///   HS_PRICE_STORAGE_MICROUSD_GB  (default: 100 = $0.10 per GB per month)
///                                 Billing tick: each SyncWorker interval
fn price_insert_microusd() -> u64 {
    std::env::var("HS_PRICE_INSERT_MICROUSD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

fn price_search_microusd() -> u64 {
    std::env::var("HS_PRICE_SEARCH_MICROUSD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// Price per GB of storage per month in micro-USD.
/// Default: $0.10 / GB / month = 100_000 micro-USD / GB / month
pub fn price_storage_microusd_per_gb_month() -> u64 {
    std::env::var("HS_PRICE_STORAGE_MICROUSD_GB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000) // $0.10 / GB / month
}

/// Compute storage rental cost for a billing tick.
///
/// `bytes`       — bytes stored by this tenant on this node
/// `tick_secs`   — how many seconds this billing tick covers (= SyncWorker interval)
/// `rf`          — replication factor (cost split among RF nodes; user pays for 1x)
///
/// Formula:
///   cost_per_tick = (bytes / 1_073_741_824) * price_per_gb_month / (30 * 24 * 3600) * tick_secs
pub fn storage_cost_microusd(bytes: u64, tick_secs: u64) -> u64 {
    if bytes == 0 {
        return 0;
    }
    let price_per_gb_month = price_storage_microusd_per_gb_month();
    // Use integer arithmetic to avoid fp precision issues.
    // Scale: price * bytes / bytes_per_gb / secs_per_month * tick_secs
    let secs_per_month: u64 = 30 * 24 * 3600; // 2_592_000
    let bytes_per_gb: u64 = 1_073_741_824;
    // cost_microusd = price_per_gb_month * bytes * tick_secs / (bytes_per_gb * secs_per_month)
    price_per_gb_month
        .saturating_mul(bytes)
        .saturating_mul(tick_secs)
        / bytes_per_gb
        / secs_per_month
}

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
        acc.cost_microusd.fetch_add(count * price_insert_microusd(), Ordering::Relaxed);
    }

    /// Record a vector search operation.
    pub fn record_search(&self, api_key: &str, payload_bytes: u64) {
        let acc = self.get_or_create(api_key);
        acc.searches.fetch_add(1, Ordering::Relaxed);
        acc.payload_bytes.fetch_add(payload_bytes, Ordering::Relaxed);
        acc.cost_microusd.fetch_add(price_search_microusd(), Ordering::Relaxed);
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
    /// When `throttled = true` and no grace period set yet, starts GRACE_PERIOD_SECS timer.
    /// When `throttled = false`, clears the grace period.
    pub fn set_throttled(&self, api_key: &str, throttled: bool) {
        let acc = self.get_or_create(api_key);
        acc.throttled.store(if throttled { 1 } else { 0 }, Ordering::Relaxed);
        if throttled {
            // Start grace period if not already running
            let current = acc.grace_period_ends_at.load(Ordering::Relaxed);
            if current == 0 {
                let grace_secs: u64 = std::env::var("HS_DEPIN_GRACE_PERIOD_DAYS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(7)
                    * 86_400;
                let deadline = now_unix_secs() + grace_secs;
                acc.grace_period_ends_at.store(deadline, Ordering::Relaxed);
            }
        } else {
            // Balance restored — cancel grace period
            acc.grace_period_ends_at.store(0, Ordering::Relaxed);
        }
    }

    /// Returns the Unix timestamp after which the tenant's data may be purged.
    /// Returns `None` if tenant is active (no grace period running).
    pub fn data_deletion_deadline(&self, api_key: &str) -> Option<u64> {
        self.counters.get(api_key).and_then(|acc| {
            let ts = acc.grace_period_ends_at.load(Ordering::Relaxed);
            if ts > 0 { Some(ts) } else { None }
        })
    }

    /// Returns all tenants whose grace period has expired (ready for data deletion).
    pub fn tenants_past_grace_period(&self) -> Vec<String> {
        let now = now_unix_secs();
        self.counters
            .iter()
            .filter(|e| {
                let ts = e.grace_period_ends_at.load(Ordering::Relaxed);
                ts > 0 && ts <= now
            })
            .map(|e| e.key().clone())
            .collect()
    }

    /// Record periodic storage rental cost for this billing tick.
    ///
    /// Must be called by `SyncWorker` at each tick for every known tenant.
    /// `tick_secs` — the interval between billing ticks (= SyncWorker interval).
    pub fn record_storage_rental(&self, api_key: &str, tick_secs: u64) {
        let acc = self.get_or_create(api_key);
        let bytes = acc.storage_bytes.load(Ordering::Relaxed);
        let cost = storage_cost_microusd(bytes, tick_secs);
        if cost > 0 {
            acc.cost_microusd.fetch_add(cost, Ordering::Relaxed);
        }
    }

    /// Drain accumulated deltas for all tenants, resetting the counters to zero.
    ///
    /// IMPORTANT: Always returns ALL tenants that have `storage_bytes > 0`, even
    /// if they had zero inserts/searches this tick — storage is billed continuously.
    pub fn drain_deltas(&self) -> Vec<UsageDelta> {
        let mut deltas = Vec::new();
        for entry in self.counters.iter() {
            let acc = entry.value();
            let inserts = acc.inserts.swap(0, Ordering::Relaxed);
            let searches = acc.searches.swap(0, Ordering::Relaxed);
            let payload_bytes = acc.payload_bytes.swap(0, Ordering::Relaxed);
            let storage_bytes = acc.storage_bytes.load(Ordering::Relaxed); // never reset, just read
            let cost = acc.cost_microusd.swap(0, Ordering::Relaxed);

            // Include tenant if there was any activity OR they have data stored.
            // Tenants with storage_bytes > 0 are always billed (storage rent is continuous).
            if inserts > 0 || searches > 0 || payload_bytes > 0 || storage_bytes > 0 || cost > 0 {
                deltas.push(UsageDelta {
                    api_key: entry.key().clone(),
                    delta_inserts: inserts,
                    delta_searches: searches,
                    payload_bytes,
                    storage_bytes,
                    cost_microusd: cost,
                });
            }
        }
        deltas
    }

    /// List all known tenants (api keys).
    pub fn all_api_keys(&self) -> Vec<String> {
        self.counters.iter().map(|e| e.key().clone()).collect()
    }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
