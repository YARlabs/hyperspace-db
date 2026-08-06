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
    #[serde(rename = "ramCapacityBytes")]
    pub ram_capacity_bytes: Option<u64>,
    #[serde(rename = "diskCapacityBytes")]
    pub disk_capacity_bytes: Option<u64>,
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
    #[serde(rename = "ramCapacityBytes")]
    pub ram_capacity_bytes: u64,
    #[serde(rename = "ramUsedBytes")]
    pub ram_used_bytes: u64,
    #[serde(rename = "diskCapacityBytes")]
    pub disk_capacity_bytes: u64,
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
        use sysinfo::{Disks, System};
        let mut sys = System::new_all();
        sys.refresh_all();

        // 1. RAM Capacity
        let ram_capacity_bytes = if let Ok(val) = std::env::var("HS_RAM_CAPACITY_BYTES") {
            val.parse::<u64>().ok()
        } else if let Ok(val) = std::env::var("HS_RAM_CAPACITY_GB") {
            val.parse::<u64>().ok().map(|gb| gb * 1024 * 1024 * 1024)
        } else {
            Some(sys.total_memory())
        };

        // 2. Disk Capacity
        let disk_capacity_bytes = if let Ok(val) = std::env::var("HS_DISK_CAPACITY_BYTES") {
            val.parse::<u64>().ok()
        } else if let Ok(val) = std::env::var("HS_DISK_CAPACITY_GB") {
            val.parse::<u64>().ok().map(|gb| gb * 1024 * 1024 * 1024)
        } else {
            let data_dir = std::env::var("HS_DATA_DIR").unwrap_or_else(|_| ".".to_string());
            let path = std::path::Path::new(&data_dir);
            let disks = Disks::new_with_refreshed_list();
            let mut best_disk_size = 100 * 1024 * 1024 * 1024;
            let mut max_prefix = 0;
            for disk in &disks {
                let mount_point = disk.mount_point();
                if path.starts_with(mount_point) {
                    let len = mount_point.to_string_lossy().len();
                    if len > max_prefix {
                        max_prefix = len;
                        best_disk_size = disk.total_space();
                    }
                }
            }
            Some(best_disk_size)
        };

        let payload = NodeRegistration {
            peer_id: self.identity.peer_id.to_base58(),
            public_key: hex::encode(self.identity.public_key_bytes()),
            ip_address: ip.to_string(),
            grpc_port: self.grpc_port,
            p2p_port: self.p2p_port,
            semantic_zone: self.identity.semantic_zone,
            version: env!("CARGO_PKG_VERSION").to_string(),
            ram_capacity_bytes,
            disk_capacity_bytes,
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
            use sysinfo::{Disks, System};
            let mut sys = System::new_all();

            let mut ticker = interval(Duration::from_secs(interval_secs));
            info!("💓 Heartbeat started (every {interval_secs}s)");

            loop {
                ticker.tick().await;

                sys.refresh_all();

                // 1. RAM Capacity
                let ram_capacity_bytes = if let Ok(val) = std::env::var("HS_RAM_CAPACITY_BYTES") {
                    val.parse::<u64>().unwrap_or(0)
                } else if let Ok(val) = std::env::var("HS_RAM_CAPACITY_GB") {
                    val.parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024
                } else {
                    sys.total_memory()
                };

                // 2. RAM Used
                let pid = sysinfo::get_current_pid().ok();
                let ram_used_bytes = if let Some(p) = pid.and_then(|p| sys.process(p)) {
                    p.memory()
                } else {
                    0
                };

                // 3. Disk Capacity
                let disk_capacity_bytes = if let Ok(val) = std::env::var("HS_DISK_CAPACITY_BYTES") {
                    val.parse::<u64>().unwrap_or(0)
                } else if let Ok(val) = std::env::var("HS_DISK_CAPACITY_GB") {
                    val.parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024
                } else {
                    let data_dir = std::env::var("HS_DATA_DIR").unwrap_or_else(|_| ".".to_string());
                    let path = std::path::Path::new(&data_dir);
                    let disks = Disks::new_with_refreshed_list();
                    let mut best_disk_size = 100 * 1024 * 1024 * 1024;
                    let mut max_prefix = 0;
                    for disk in &disks {
                        let mount_point = disk.mount_point();
                        if path.starts_with(mount_point) {
                            let len = mount_point.to_string_lossy().len();
                            if len > max_prefix {
                                max_prefix = len;
                                best_disk_size = disk.total_space();
                            }
                        }
                    }
                    best_disk_size
                };

                // 4. CPU load
                let load = sys.global_cpu_usage() / 100.0;

                let (chunk_count, disk_bytes) = chunk_count_fn();
                let payload = NodeHeartbeat {
                    peer_id: self.identity.peer_id.to_base58(),
                    load,
                    chunk_count,
                    disk_bytes,
                    ram_capacity_bytes,
                    ram_used_bytes,
                    disk_capacity_bytes,
                };

                let url = format!("{}/api/depin/nodes/heartbeat", self.base_url);
                match self.http.post(&url).json(&payload).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::debug!(
                            "💓 Heartbeat sent — chunks={chunk_count} disk={disk_bytes}"
                        );
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
