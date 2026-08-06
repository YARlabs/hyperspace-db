#![allow(clippy::pedantic)]
pub mod accounting;
pub mod metering;
pub mod sync;
pub mod tickets;

use std::sync::Arc;

pub use accounting::{AccountingStore, BillingBalance, BillingStatus};
pub use metering::{storage_cost_microusd, MeteringEngine, UsageDelta};
pub use sync::SyncWorker;
pub use tickets::{PendingTicketStore, SignedTicket, TicketVerifier};

/// Single shared billing context, injected into the gRPC server.
#[derive(Clone)]
pub struct BillingContext {
    pub metering: Arc<MeteringEngine>,
    pub accounting: Arc<AccountingStore>,
    pub ticket_store: Arc<PendingTicketStore>,
    pub ticket_verifier: Arc<std::sync::Mutex<TicketVerifier>>,
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
        Self::start_with_callbacks(
            db_path,
            coordinator_base_url,
            max_rps,
            sync_interval_secs,
            None,
        )
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

        let tickets_db_path = if let Some(stripped) = db_path.strip_suffix(".redb") {
            format!("{}_tickets.redb", stripped)
        } else {
            format!("{}_tickets", db_path)
        };
        let ticket_store = Arc::new(PendingTicketStore::open(&tickets_db_path)?);
        let ticket_verifier = Arc::new(std::sync::Mutex::new(TicketVerifier::new()));

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

        Ok(Self {
            metering,
            accounting,
            ticket_store,
            ticket_verifier,
        })
    }
}
