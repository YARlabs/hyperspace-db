use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::accounting::{AccountingStore, BillingBalance, BillingStatus};
use crate::metering::MeteringEngine;

/// Response shape expected from `POST /api/depin/sync`
#[derive(Debug, serde::Deserialize)]
struct SyncResponse {
    success: bool,
    status: Option<String>,
    #[serde(rename = "tokenBalance")]
    token_balance: Option<f64>,
    error: Option<String>,
}

/// Payload sent to `POST /api/depin/sync`
#[derive(Debug, serde::Serialize)]
struct SyncPayload<'a> {
    #[serde(rename = "apiKey")]
    api_key: &'a str,
    #[serde(rename = "deltaInserts")]
    delta_inserts: u64,
    #[serde(rename = "deltaSearches")]
    delta_searches: u64,
    #[serde(rename = "payloadBytes")]
    payload_bytes: u64,
    #[serde(rename = "storageBytes")]
    storage_bytes: u64,
    /// Total cost in micro-USD for this billing period (1 = $0.000001).
    /// Includes: insert cost + search cost + **storage rental cost**.
    #[serde(rename = "costMicroUsd")]
    cost_microusd: u64,
    /// Replication Factor — how many nodes hold a copy of this tenant's data.
    /// Used by coordinator to calculate per-node storage revenue share:
    ///   node_storage_revenue = storage_cost / rf * 0.8 (platform 20%)
    #[serde(rename = "replicationFactor")]
    replication_factor: u8,
}

/// The background worker that every `sync_interval` seconds:
///
/// 1. Charges each tenant for storage rental (`record_storage_rental()`)
/// 2. Drains the `MeteringEngine` (inserts + searches + storage cost)
/// 3. POSTs each tenant's delta to `api_identity_rust /api/depin/sync`
/// 4. Persists the resulting balance in `AccountingStore` (redb)
/// 5. Updates the in-memory throttle flag + grace period
/// 6. Emits a callback for tenants whose grace period has expired (data deletion)
pub struct SyncWorker {
    metering: Arc<MeteringEngine>,
    accounting: Arc<AccountingStore>,
    http: reqwest::Client,
    coordinator_base_url: String,
    sync_interval: Duration,
    /// Called when a tenant's grace period has expired and data should be deleted.
    /// `Box<dyn Fn(String) + Send + Sync>` — receives the api_key of the tenant.
    on_data_deletion: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl SyncWorker {
    pub fn new(
        metering: Arc<MeteringEngine>,
        accounting: Arc<AccountingStore>,
        coordinator_base_url: String,
        sync_interval_secs: u64,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            metering,
            accounting,
            http,
            coordinator_base_url,
            sync_interval: Duration::from_secs(sync_interval_secs),
            on_data_deletion: None,
        }
    }

    /// Register a callback invoked when a tenant's grace period expires.
    /// The callback receives the `api_key` (user_id) and should delete all
    /// collections owned by that tenant.
    pub fn with_data_deletion_callback(
        mut self,
        callback: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        self.on_data_deletion = Some(Arc::new(callback));
        self
    }

    /// Spawn the background sync loop inside the current Tokio runtime.
    pub fn spawn(self: Arc<Self>) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    async fn run(&self) {
        let tick_secs = self.sync_interval.as_secs().max(1);
        let mut ticker = interval(self.sync_interval);
        let replication_factor: u8 = std::env::var("HS_REPLICATION_FACTOR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1); // Web2.5: RF=1 (no replication yet). DePIN Phase 5: RF=3
        info!(
            "SyncWorker started (interval = {:?}, coordinator = {}, RF={})",
            self.sync_interval, self.coordinator_base_url, replication_factor
        );

        loop {
            ticker.tick().await;

            // ─── Step 1: Charge storage rental for ALL known tenants ──────────────
            // This must happen BEFORE drain_deltas() so the cost is included in
            // the current tick's delta.
            for api_key in self.metering.all_api_keys() {
                self.metering.record_storage_rental(&api_key, tick_secs);
            }

            // ─── Step 2: Check for expired grace periods → trigger data deletion ──
            let expired = self.metering.tenants_past_grace_period();
            for api_key in &expired {
                warn!(
                    api_key = %api_key,
                    "⚠️  Grace period expired — initiating data deletion for tenant"
                );
                if let Some(cb) = &self.on_data_deletion {
                    cb(api_key.clone());
                }
            }

            // ─── Step 3: Drain accumulated deltas (inserts + searches + storage) ──
            let deltas = self.metering.drain_deltas();
            if deltas.is_empty() {
                continue;
            }

            let mut updated_balances = Vec::new();

            for delta in &deltas {
                let url = format!("{}/api/depin/sync", self.coordinator_base_url);
                let payload = SyncPayload {
                    api_key: &delta.api_key,
                    delta_inserts: delta.delta_inserts,
                    delta_searches: delta.delta_searches,
                    payload_bytes: delta.payload_bytes,
                    storage_bytes: delta.storage_bytes,
                    cost_microusd: delta.cost_microusd,
                    replication_factor,
                };

                match self.http.post(&url).json(&payload).send().await {
                    Ok(resp) => {
                        let status_code = resp.status();
                        match resp.json::<SyncResponse>().await {
                            Ok(sync_resp) if sync_resp.success => {
                                let billing_status = match sync_resp.status.as_deref() {
                                    Some("active") => BillingStatus::Active,
                                    Some("insufficient_funds") => BillingStatus::InsufficientFunds,
                                    _ => BillingStatus::Unknown,
                                };

                                // Update throttle flag + grace period in memory
                                self.metering.set_throttled(
                                    &delta.api_key,
                                    billing_status == BillingStatus::InsufficientFunds,
                                );

                                if billing_status == BillingStatus::InsufficientFunds {
                                    let deadline = self
                                        .metering
                                        .data_deletion_deadline(&delta.api_key)
                                        .map(|ts| {
                                            let days_left = ts.saturating_sub(
                                                SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                            ) / 86_400;
                                            format!("{} days until data deletion", days_left)
                                        })
                                        .unwrap_or_default();
                                    warn!(
                                        "💸 Tenant {} — insufficient balance. Throttled. {}",
                                        delta.api_key, deadline
                                    );
                                }

                                let now_ts = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                updated_balances.push(BillingBalance {
                                    api_key: delta.api_key.clone(),
                                    status: billing_status,
                                    last_known_balance: sync_resp.token_balance.unwrap_or(0.0),
                                    last_sync_ts: now_ts,
                                });
                            }
                            Ok(sync_resp) => {
                                error!("Sync failed for {}: {:?}", delta.api_key, sync_resp.error);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to decode sync response for {} (HTTP {}): {}",
                                    delta.api_key, status_code, e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "HTTP error during billing sync for {}: {}",
                            delta.api_key, e
                        );
                    }
                }
            }

            // ─── Step 4: Persist all updated balances in a single redb transaction ─
            if !updated_balances.is_empty() {
                if let Err(e) = self.accounting.upsert_batch(&updated_balances) {
                    error!("Failed to persist billing balances to redb: {}", e);
                } else {
                    info!(
                        "SyncWorker: persisted {} balance records (storage billed: {} tenants)",
                        updated_balances.len(),
                        deltas.iter().filter(|d| d.storage_bytes > 0).count()
                    );
                }
            }
        }
    }
}
