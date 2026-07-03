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
}

/// The background worker that:
/// 1. Drains the `MeteringEngine` every N seconds
/// 2. POSTs each tenant's delta to `api_identity_rust /api/depin/sync`
/// 3. Persists the resulting balance in `AccountingStore` (redb)
/// 4. Updates the in-memory throttle flag
pub struct SyncWorker {
    metering: Arc<MeteringEngine>,
    accounting: Arc<AccountingStore>,
    http: reqwest::Client,
    coordinator_base_url: String,
    sync_interval: Duration,
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
        }
    }

    /// Spawn the background sync loop inside the current Tokio runtime.
    pub fn spawn(self: Arc<Self>) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    async fn run(&self) {
        let mut ticker = interval(self.sync_interval);
        info!(
            "SyncWorker started (interval = {:?}, coordinator = {})",
            self.sync_interval, self.coordinator_base_url
        );

        loop {
            ticker.tick().await;
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

                                // Update throttle flag in memory
                                self.metering.set_throttled(
                                    &delta.api_key,
                                    billing_status == BillingStatus::InsufficientFunds,
                                );

                                if billing_status == BillingStatus::InsufficientFunds {
                                    warn!(
                                        "Tenant {} has insufficient balance — throttling enabled",
                                        delta.api_key
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
                                error!(
                                    "Sync failed for {}: {:?}",
                                    delta.api_key, sync_resp.error
                                );
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

            // Persist all updated balances in a single redb transaction
            if !updated_balances.is_empty() {
                if let Err(e) = self.accounting.upsert_batch(&updated_balances) {
                    error!("Failed to persist billing balances to redb: {}", e);
                } else {
                    info!(
                        "SyncWorker: persisted {} balance records",
                        updated_balances.len()
                    );
                }
            }
        }
    }
}
