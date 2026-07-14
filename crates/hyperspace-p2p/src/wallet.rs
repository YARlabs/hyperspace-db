//! Node registration with the centralised coordinator (api_identity_rust).
//!
//! Phase 2 scope (Web2.5):
//! - `POST /api/depin/nodes/register` — initial registration
//! - `POST /api/depin/nodes/heartbeat` — periodic keepalive (every 60s)
//! - Auto-retry with exponential back-off on failure

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::identity::{NodeIdentity, SemanticZone};

/// Payload sent to `/api/depin/nodes/register`
#[derive(Debug, Serialize)]
pub struct NodeRegistration {
    #[serde(rename = "peerId")]
    pub peer_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: String, // hex-encoded ed25519 public key
    #[serde(rename = "ipAddress")]
    pub ip_address: String,
    #[serde(rename = "grpcPort")]
    pub grpc_port: u16,
    #[serde(rename = "p2pPort")]
    pub p2p_port: u16,
    #[serde(rename = "semanticZone")]
    pub semantic_zone: SemanticZone,
    #[serde(rename = "version")]
    pub version: String,
}

/// Response from the coordinator on successful registration.
#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    #[serde(rename = "nodeId")]
    pub node_id: Option<String>,
    pub error: Option<String>,
}

/// Heartbeat payload sent to `/api/depin/nodes/heartbeat`
#[derive(Debug, Serialize)]
pub struct NodeHeartbeat {
    #[serde(rename = "peerId")]
    pub peer_id: String,
    /// Current load: 0.0–1.0
    pub load: f32,
    /// Number of chunks held
    #[serde(rename = "chunkCount")]
    pub chunk_count: u64,
    /// Total disk used in bytes
    #[serde(rename = "diskBytes")]
    pub disk_bytes: u64,
}

/// The coordinator client — handles registration + heartbeats.
pub struct CoordinatorClient {
    http: reqwest::Client,
    base_url: String,
    identity: Arc<NodeIdentity>,
    grpc_port: u16,
    p2p_port: u16,
}

impl CoordinatorClient {
    pub fn new(
        base_url: String,
        identity: Arc<NodeIdentity>,
        grpc_port: u16,
        p2p_port: u16,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            http,
            base_url,
            identity,
            grpc_port,
            p2p_port,
        }
    }

    /// Register this node with the coordinator.  Retries up to `max_retries` times.
    pub async fn register(&self, ip: &str, max_retries: u32) -> anyhow::Result<String> {
        let payload = NodeRegistration {
            peer_id: self.identity.peer_id.to_base58(),
            public_key: hex::encode(self.identity.public_key_bytes()),
            ip_address: ip.to_string(),
            grpc_port: self.grpc_port,
            p2p_port: self.p2p_port,
            semantic_zone: self.identity.semantic_zone,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let url = format!("{}/api/depin/nodes/register", self.base_url);

        for attempt in 0..=max_retries {
            match self.http.post(&url).json(&payload).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.json::<RegistrationResponse>().await {
                        Ok(r) if r.success => {
                            let node_id = r.node_id.clone().unwrap_or_default();
                            info!("✅ Node registered — id={node_id} peer={}", payload.peer_id);
                            return Ok(node_id);
                        }
                        Ok(r) => {
                            error!("Registration rejected: {:?}", r.error);
                            return Err(anyhow::anyhow!("Registration failed: {:?}", r.error));
                        }
                        Err(e) => {
                            warn!("Failed to parse registration response (HTTP {status}): {e}");
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Registration attempt {}/{}: HTTP error: {e}",
                        attempt + 1,
                        max_retries + 1
                    );
                }
            }

            if attempt < max_retries {
                let backoff = Duration::from_secs(2u64.pow(attempt.min(6)));
                tokio::time::sleep(backoff).await;
            }
        }

        Err(anyhow::anyhow!(
            "Registration failed after {} attempts",
            max_retries + 1
        ))
    }

    /// Start the background heartbeat loop (every `interval_secs` seconds).
    pub fn spawn_heartbeat(
        self: Arc<Self>,
        interval_secs: u64,
        chunk_count_fn: impl Fn() -> (u64, u64) + Send + Sync + 'static,
    ) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            info!("💓 Heartbeat started (every {interval_secs}s)");

            loop {
                ticker.tick().await;

                let (chunk_count, disk_bytes) = chunk_count_fn();
                let payload = NodeHeartbeat {
                    peer_id: self.identity.peer_id.to_base58(),
                    load: 0.5, // TODO: real CPU/IO load from sysinfo
                    chunk_count,
                    disk_bytes,
                };

                let url = format!("{}/api/depin/nodes/heartbeat", self.base_url);
                match self.http.post(&url).json(&payload).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::debug!("💓 Heartbeat sent — chunks={chunk_count}");
                    }
                    Ok(resp) => {
                        warn!("Heartbeat rejected: HTTP {}", resp.status());
                    }
                    Err(e) => {
                        warn!("Heartbeat HTTP error: {e}");
                    }
                }
            }
        });
    }
}
