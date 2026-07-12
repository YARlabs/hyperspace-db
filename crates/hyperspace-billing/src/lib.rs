pub mod accounting;
pub mod metering;
pub mod sync;
pub mod tickets;

use std::sync::Arc;

pub use accounting::{AccountingStore, BillingBalance, BillingStatus};
pub use metering::{MeteringEngine, UsageDelta, storage_cost_microusd};
pub use sync::SyncWorker;
pub use tickets::{PendingTicketStore, SignedTicket, TicketVerifier};

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
    /// - Spawns the background `SyncWorker` (bills storage each tick)
    pub fn start(
        db_path: &str,
        coordinator_base_url: String,
        max_rps: u32,
        sync_interval_secs: u64,
    ) -> anyhow::Result<Self> {
        Self::start_with_callbacks(db_path, coordinator_base_url, max_rps, sync_interval_secs, None)
    }

    /// Initialize with an optional data-deletion callback.
    ///
    /// `on_data_deletion` is called when a tenant's grace period expires.
    /// It receives the `api_key` (user_id) and should delete all data for that tenant.
    ///
    /// # Example
    /// ```rust,ignore
    /// BillingContext::start_with_callbacks(
    ///     db_path, coordinator_url, max_rps, sync_secs,
    ///     Some(Arc::new(move |user_id: String| {
    ///         let mgr = manager.clone();
    ///         tokio::spawn(async move {
    ///             mgr.delete_all_for_owner(&user_id).await;
    ///         });
    ///     })),
    /// )
    /// ```
    pub fn start_with_callbacks(
        db_path: &str,
        coordinator_base_url: String,
        max_rps: u32,
        sync_interval_secs: u64,
        on_data_deletion: Option<Arc<dyn Fn(String) + Send + Sync>>,
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

        let mut sync_worker = SyncWorker::new(
            metering.clone(),
            accounting.clone(),
            coordinator_base_url,
            sync_interval_secs,
        );

        if let Some(cb) = on_data_deletion {
            let cb_fn = move |key: String| cb(key);
            sync_worker = sync_worker.with_data_deletion_callback(cb_fn);
        }

        Arc::new(sync_worker).spawn();

        Ok(Self { metering, accounting })
    }
}
