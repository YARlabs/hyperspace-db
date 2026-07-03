pub mod accounting;
pub mod metering;
pub mod sync;

use std::sync::Arc;

pub use accounting::{AccountingStore, BillingBalance, BillingStatus};
pub use metering::{MeteringEngine, UsageDelta};
pub use sync::SyncWorker;

/// Single shared billing context, injected into the gRPC server.
#[derive(Clone)]
pub struct BillingContext {
    pub metering: Arc<MeteringEngine>,
    pub accounting: Arc<AccountingStore>,
}

impl BillingContext {
    /// Initialize the billing context:
    /// - Opens the `redb` accounting store at `db_path`
    /// - Creates the in-memory `MeteringEngine`
    /// - Restores known throttle flags from persisted data
    /// - Spawns the background `SyncWorker`
    pub fn start(
        db_path: &str,
        coordinator_base_url: String,
        max_rps: u32,
        sync_interval_secs: u64,
    ) -> anyhow::Result<Self> {
        let accounting = Arc::new(AccountingStore::open(db_path)?);
        let metering = Arc::new(MeteringEngine::new(max_rps));

        // Restore throttle flags from persisted state
        let all_balances = accounting.load_all()?;
        for b in &all_balances {
            if b.status == BillingStatus::InsufficientFunds {
                metering.set_throttled(&b.api_key, true);
            }
        }

        let sync_worker = Arc::new(SyncWorker::new(
            metering.clone(),
            accounting.clone(),
            coordinator_base_url,
            sync_interval_secs,
        ));
        sync_worker.spawn();

        Ok(Self { metering, accounting })
    }
}
