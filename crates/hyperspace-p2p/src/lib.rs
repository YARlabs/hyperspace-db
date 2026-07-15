#![allow(clippy::pedantic)]
//! hyperspace-p2p — DePIN P2P networking layer
//!
//! # Phase 2 scope
//! - Node identity (BIP-39 mnemonic → ed25519 + libp2p PeerId)
//! - libp2p Swarm (QUIC + Kademlia DHT + GossipSub)
//! - MetaRouter (local chunk → peer routing cache)
//! - Coordinator client (registration + heartbeat → api_identity_rust)
//!
//! # Module structure
//! - `identity`  — NodeIdentity, key derivation, persistence
//! - `network`   — libp2p Swarm, behaviour, event loop
//! - `routing`   — MetaRouter, MetaRouterEntry, blake3 Merkle root
//! - `wallet`    — CoordinatorClient, registration, heartbeat

pub mod identity;
pub mod network;
pub mod proofs;
pub mod routing;
pub mod wallet;

use std::sync::Arc;

pub use identity::{NodeIdentity, PersistedIdentity, SemanticZone};
pub use network::{SwarmCommand, SwarmEvent2};
pub use routing::{MetaRouter, MetaRouterEntry};
pub use wallet::CoordinatorClient;

/// Single entrypoint to bootstrap the entire P2P stack.
///
/// 1. Load (or generate) the `NodeIdentity`.
/// 2. Register with the coordinator (`api_identity_rust`).
/// 3. Start the background heartbeat.
/// 4. *(Phase 2.2)* Build and spawn the libp2p Swarm.
///
/// Returns the live `NodeIdentity` and a `MetaRouter` handle.
pub async fn bootstrap(
    identity_path: &str,
    semantic_zone: SemanticZone,
    coordinator_url: String,
    grpc_port: u16,
    p2p_port: u16,
    public_ip: String,
) -> anyhow::Result<(Arc<NodeIdentity>, MetaRouter)> {
    // Step 1 — identity
    let identity = Arc::new(NodeIdentity::load_or_generate(
        identity_path,
        semantic_zone,
    )?);
    tracing::info!(
        "🆔 Node PeerId: {}  zone: {}",
        identity.peer_id,
        identity.semantic_zone
    );

    // Step 2 — register with coordinator
    let coordinator = Arc::new(CoordinatorClient::new(
        coordinator_url.clone(),
        identity.clone(),
        grpc_port,
        p2p_port,
    ));

    // Try to register — non-fatal on failure (node still serves data)
    match coordinator.register(&public_ip, 3).await {
        Ok(node_id) => tracing::info!("📋 Registered as node_id={node_id}"),
        Err(e) => tracing::warn!("⚠️  Registration failed (will retry on heartbeat): {e}"),
    }

    // Step 3 — heartbeat (chunk stats via directory traversal)
    let coordinator_for_hb = coordinator.clone();
    coordinator_for_hb.spawn_heartbeat(60, || {
        let data_dir = std::env::var("HS_DATA_DIR").unwrap_or_else(|_| ".".to_string());
        let path = std::path::Path::new(&data_dir);
        
        let mut chunk_count = 0;
        let mut disk_bytes = 0;
        
        fn traverse(dir: &std::path::Path, chunks: &mut u64, bytes: &mut u64) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
                            *bytes += len;
                            if entry.path().extension().map_or(false, |ext| ext == "hyp") {
                                *chunks += 1;
                            }
                        } else if file_type.is_dir() {
                            traverse(&entry.path(), chunks, bytes);
                        }
                    }
                }
            }
        }
        
        traverse(path, &mut chunk_count, &mut disk_bytes);
        (chunk_count, disk_bytes)
    });

    // Step 4 — MetaRouter
    let meta_router = MetaRouter::new();

    // Step 5 — libp2p Swarm (Phase 2.2 — requires keypair refactor in identity.rs)
    // Skipped for now: build_swarm() needs the raw secret bytes from NodeIdentity.
    // Will be enabled in Phase 2.2 after the Zeroizing<[u8;32]> refactor.
    tracing::info!("ℹ️  libp2p Swarm deferred to Phase 2.2 (keypair refactor pending)");

    Ok((identity, meta_router))
}
