//! # Gossip Protocol — Swarm Discovery & Health (Task 3.4)
//!
//! Lightweight UDP-based gossip for peer-to-peer node discovery.
//!
//! ## Protocol
//! Each node broadcasts a `GossipMessage::Heartbeat` every `HEARTBEAT_INTERVAL`.
//! Peers that haven't been seen for `PEER_TTL` are evicted from the peer table.
//!
//! ## How to enable
//! Set `HS_GOSSIP_PORT` (default: 7946) and `HS_GOSSIP_PEERS` (comma-separated
//! list of known seed peers, e.g. `192.168.1.10:7946,192.168.1.11:7946`).
//!
//! ## Zero-dependency design
//! Uses raw `tokio::net::UdpSocket` — no libp2p required.
//! libp2p / Kademlia DHT is planned for Sprint 6.

use crate::manager::ClusterRole;
use crate::sync::CollectionDigest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

// ─── Constants ─────────────────────────────────────────────────────────────

pub const DEFAULT_GOSSIP_PORT: u16 = 7946;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const PEER_TTL: Duration = Duration::from_secs(30);
const MAX_UDP_PAYLOAD: usize = 4096;

// ─── Data Structures ────────────────────────────────────────────────────────

/// A single gossip message sent over UDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GossipMessage {
    /// Periodic heartbeat — announces presence and basic state.
    Heartbeat {
        node_id: String,
        role: String,
        http_port: u16,
        /// The UDP port this node is listening on for gossip.
        gossip_port: u16,
        logical_clock: u64,
        /// Lightweight collection digests (name + state_hash only, not full bucket list).
        digests: Vec<GossipCollectionSummary>,
        timestamp_secs: u64,
    },
    /// Request full sync handshake from a specific collection.
    SyncRequest {
        from_node_id: String,
        collection: String,
        bucket_hashes: Vec<u64>,
    },
}

/// Lightweight collection summary included in every heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipCollectionSummary {
    pub name: String,
    pub state_hash: u64,
    pub vector_count: usize,
    pub logical_clock: u64,
}

/// State of a known swarm peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    /// Peer's HTTP API address (ip:http_port)
    pub addr: String,
    pub http_port: u16,
    /// Peer's UDP gossip address (ip:gossip_port) — used when re-broadcasting
    pub gossip_addr: String,
    pub role: String,
    pub logical_clock: u64,
    pub collections: Vec<GossipCollectionSummary>,
    pub last_seen_secs: u64,
    /// Derived: whether peer appears healthy (last_seen < TTL).
    pub healthy: bool,
}

impl PeerInfo {
    pub fn is_stale(&self) -> bool {
        let now = now_secs();
        now.saturating_sub(self.last_seen_secs) > PEER_TTL.as_secs()
    }
}

/// Thread-safe shared peer registry.
pub type PeerRegistry = Arc<RwLock<HashMap<String, PeerInfo>>>;

// ─── Gossip Engine ──────────────────────────────────────────────────────────

/// Starts the gossip engine as two concurrent tasks:
/// - **Broadcaster**: Sends heartbeats to all known seed peers + discovered peers.
/// - **Listener**: Receives heartbeats and updates the peer registry.
///
/// Returns the shared `PeerRegistry` that the HTTP layer reads for /api/swarm/peers.
pub async fn start_gossip(
    node_id: String,
    role: ClusterRole,
    http_port: u16,
    logical_clock_ref: Arc<tokio::sync::RwLock<u64>>,
    digests_ref: Arc<RwLock<Vec<CollectionDigest>>>,
) -> PeerRegistry {
    let gossip_port = std::env::var("HS_GOSSIP_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(DEFAULT_GOSSIP_PORT);

    let seed_peers: Vec<String> = std::env::var("HS_GOSSIP_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .collect();

    let registry: PeerRegistry = Arc::new(RwLock::new(HashMap::new()));

    // Spawn listener
    let registry_l = Arc::clone(&registry);
    let node_id_l = node_id.clone();
    let gossip_port_l = gossip_port;
    tokio::spawn(async move {
        run_listener(gossip_port_l, node_id_l, registry_l).await;
    });

    // Spawn broadcaster
    let registry_b = Arc::clone(&registry);
    let role_str = format!("{role:?}");
    tokio::spawn(async move {
        run_broadcaster(
            node_id,
            role_str,
            http_port,
            gossip_port,
            seed_peers,
            logical_clock_ref,
            digests_ref,
            registry_b,
        )
        .await;
    });

    println!(
        "🌐 Gossip engine started on UDP:{gossip_port} — \
        set HS_GOSSIP_PEERS=<ip:port,...> to join a swarm"
    );

    registry
}

// ─── Listener Task ──────────────────────────────────────────────────────────

async fn run_listener(port: u16, my_node_id: String, registry: PeerRegistry) {
    let bind_addr = format!("0.0.0.0:{port}");
    let sock = match UdpSocket::bind(&bind_addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠️ Gossip listener failed to bind {bind_addr}: {e}");
            return;
        }
    };

    let mut buf = vec![0u8; MAX_UDP_PAYLOAD];
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, peer_addr)) => {
                handle_incoming(&buf[..len], peer_addr, &my_node_id, &registry).await;
            }
            Err(e) => {
                eprintln!("⚠️ Gossip recv error: {e}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_incoming(
    data: &[u8],
    peer_addr: SocketAddr,
    my_node_id: &str,
    registry: &PeerRegistry,
) {
    let Ok(msg) = serde_json::from_slice::<GossipMessage>(data) else {
        return;
    };

    if let GossipMessage::Heartbeat {
        node_id,
        role,
        http_port,
        gossip_port,
        logical_clock,
        digests,
        timestamp_secs,
    } = msg
    {
        // Ignore our own broadcasts
        if node_id == my_node_id {
            return;
        }

        let peer_ip = peer_addr.ip().to_string();
        let mut reg = registry.write().await;
        let healthy = now_secs().saturating_sub(timestamp_secs) < PEER_TTL.as_secs();
        reg.insert(
            node_id.clone(),
            PeerInfo {
                node_id,
                addr: format!("{peer_ip}:{http_port}"),
                http_port,
                gossip_addr: format!("{peer_ip}:{gossip_port}"),
                role,
                logical_clock,
                collections: digests,
                last_seen_secs: timestamp_secs,
                healthy,
            },
        );
    }
}

// ─── Broadcaster Task ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn run_broadcaster(
    node_id: String,
    role: String,
    http_port: u16,
    gossip_port: u16,
    seed_peers: Vec<String>,
    logical_clock_ref: Arc<tokio::sync::RwLock<u64>>,
    digests_ref: Arc<RwLock<Vec<CollectionDigest>>>,
    registry: PeerRegistry,
) {
    // Bind to any source port for sending
    let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await else {
        eprintln!("⚠️ Gossip broadcaster failed to bind");
        return;
    };

    let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
    loop {
        interval.tick().await;

        let logical_clock = *logical_clock_ref.read().await;
        let digests = digests_ref.read().await;
        let summaries: Vec<GossipCollectionSummary> = digests
            .iter()
            .map(|d| GossipCollectionSummary {
                name: d.collection_name.clone(),
                state_hash: d.state_hash,
                vector_count: d.vector_count,
                logical_clock: d.logical_clock,
            })
            .collect();
        drop(digests);

        let msg = GossipMessage::Heartbeat {
            node_id: node_id.clone(),
            role: role.clone(),
            http_port,
            gossip_port,
            logical_clock,
            digests: summaries,
            timestamp_secs: now_secs(),
        };

        if let Ok(payload) = serde_json::to_vec(&msg) {
            // Send to seed peers
            for peer in &seed_peers {
                let _ = sock.send_to(&payload, peer).await;
            }

            // Send to all discovered peers — use their gossip_addr (not HTTP)
            let reg = registry.read().await;
            for peer in reg.values() {
                let _ = sock.send_to(&payload, &peer.gossip_addr).await;
            }
        }

        // Evict stale peers
        let mut reg = registry.write().await;
        reg.retain(|_, p| !p.is_stale());
        // Update healthy status
        let now = now_secs();
        for peer in reg.values_mut() {
            peer.healthy = now.saturating_sub(peer.last_seen_secs) < PEER_TTL.as_secs();
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_stale_detection() {
        let mut peer = PeerInfo {
            node_id: "abc".to_string(),
            addr: "127.0.0.1:8080".to_string(),
            gossip_addr: "127.0.0.1:7946".to_string(),
            http_port: 8080,
            role: "Leader".to_string(),
            logical_clock: 0,
            collections: vec![],
            last_seen_secs: 0, // Very old
            healthy: false,
        };
        assert!(peer.is_stale());

        peer.last_seen_secs = now_secs();
        assert!(!peer.is_stale());
    }

    #[test]
    fn test_gossip_message_serialization() {
        let msg = GossipMessage::Heartbeat {
            node_id: "node-1".to_string(),
            role: "Leader".to_string(),
            http_port: 50050,
            gossip_port: 7946,
            logical_clock: 42,
            digests: vec![GossipCollectionSummary {
                name: "agents".to_string(),
                state_hash: 0xdead_beef,
                vector_count: 1000,
                logical_clock: 42,
            }],
            timestamp_secs: now_secs(),
        };
        let bytes = serde_json::to_vec(&msg).unwrap();
        assert!(!bytes.is_empty());
        // Ensure it round-trips
        let decoded: GossipMessage = serde_json::from_slice(&bytes).unwrap();
        if let GossipMessage::Heartbeat { node_id, .. } = decoded {
            assert_eq!(node_id, "node-1");
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_collection_summary_state_hash() {
        let summary = GossipCollectionSummary {
            name: "test".to_string(),
            state_hash: 12345,
            vector_count: 500,
            logical_clock: 10,
        };
        assert_eq!(summary.state_hash, 12345);
    }
}
