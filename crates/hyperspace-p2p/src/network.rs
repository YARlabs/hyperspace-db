//! libp2p Swarm for HyperspaceDB DePIN.
//!
//! Phase 2 scope:
//!   - QUIC transport (low-latency handshake)
//!   - Noise encryption + Yamux multiplexing
//!   - Kademlia DHT (peer discovery, chunk provider records)
//!   - GossipSub (MetaRouter update broadcasts)
//!   - Identify (peer capability exchange)

use std::time::Duration;

use libp2p::{
    gossipsub, identify, kad,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::identity::NodeIdentity;
use crate::routing::MetaRouterEntry;

pub const METAROUTER_TOPIC: &str = "metarouter-updates";

/// Commands sent to the P2P swarm from application code.
#[derive(Debug)]
pub enum SwarmCommand {
    PublishChunk { chunk_id: u64, centroid: Vec<f32> },
    FindHolders { chunk_id: u64 },
    BroadcastMetaRouter { entry: MetaRouterEntry },
    Shutdown,
}

/// Events emitted by the swarm to application code.
#[derive(Debug)]
pub enum SwarmEvent2 {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    HoldersFound { chunk_id: u64, holders: Vec<PeerId> },
    MetaRouterUpdate(MetaRouterEntry),
}

/// Combined libp2p NetworkBehaviour for HyperspaceDB.
#[derive(NetworkBehaviour)]
pub struct HyperspaceBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
}

/// Build the libp2p Swarm, command sender, and event receiver.
pub async fn build_swarm(
    identity: &NodeIdentity,
) -> anyhow::Result<(
    Swarm<HyperspaceBehaviour>,
    mpsc::Sender<SwarmCommand>,
    mpsc::Receiver<SwarmEvent2>,
)> {
    let libp2p_keypair = identity.libp2p_keypair()?;
    let peer_id = identity.peer_id;

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .max_transmit_size(1 << 20) // 1 MB
        .build()
        .map_err(|e| anyhow::anyhow!("GossipSub config: {e}"))?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(libp2p_keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| anyhow::anyhow!("GossipSub: {e}"))?;

    let kademlia = kad::Behaviour::new(peer_id, kad::store::MemoryStore::new(peer_id));

    let identify = identify::Behaviour::new(identify::Config::new(
        "/hyperspace/1.0.0".to_string(),
        libp2p_keypair.public(),
    ));

    let behaviour = HyperspaceBehaviour {
        kademlia,
        gossipsub,
        identify,
    };

    let swarm = libp2p::SwarmBuilder::with_existing_identity(libp2p_keypair)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)
        .map_err(|e| anyhow::anyhow!("Swarm build: {e}"))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let (cmd_tx, _cmd_rx) = mpsc::channel::<SwarmCommand>(64);
    let (_evt_tx, evt_rx) = mpsc::channel::<SwarmEvent2>(256);

    Ok((swarm, cmd_tx, evt_rx))
}

/// Run the main swarm event loop.
pub async fn run_swarm(
    mut swarm: Swarm<HyperspaceBehaviour>,
    mut cmd_rx: mpsc::Receiver<SwarmCommand>,
    evt_tx: mpsc::Sender<SwarmEvent2>,
    listen_addr: Multiaddr,
    bootstrap_peers: Vec<Multiaddr>,
) -> anyhow::Result<()> {
    let topic = gossipsub::IdentTopic::new(METAROUTER_TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on(listen_addr.clone())?;
    info!("P2P swarm listening on {listen_addr}");

    for addr in bootstrap_peers {
        if let Err(e) = swarm.dial(addr.clone()) {
            warn!("Failed to dial bootstrap {addr}: {e}");
        }
    }

    loop {
        tokio::select! {
            event = futures::StreamExt::next(&mut swarm) => {
                let Some(event) = event else { break };
                handle_event(event, &evt_tx, &topic, swarm.behaviour_mut()).await;
            }
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break };
                match cmd {
                    SwarmCommand::Shutdown => { info!("P2P shutting down"); break; }
                    SwarmCommand::BroadcastMetaRouter { entry } => {
                        if let Ok(bytes) = serde_json::to_vec(&entry) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), bytes);
                        }
                    }
                    SwarmCommand::PublishChunk { chunk_id, .. } => {
                        let key = kad::RecordKey::new(&chunk_id.to_be_bytes());
                        let _ = swarm.behaviour_mut().kademlia.start_providing(key);
                    }
                    SwarmCommand::FindHolders { chunk_id } => {
                        let key = kad::RecordKey::new(&chunk_id.to_be_bytes());
                        swarm.behaviour_mut().kademlia.get_providers(key);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_event(
    event: SwarmEvent<HyperspaceBehaviourEvent>,
    evt_tx: &mpsc::Sender<SwarmEvent2>,
    _topic: &gossipsub::IdentTopic,
    _behaviour: &mut HyperspaceBehaviour,
) {
    match event {
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            info!("🔗 Connected: {peer_id}");
            let _ = evt_tx.send(SwarmEvent2::PeerConnected(peer_id)).await;
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            let _ = evt_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
        }
        SwarmEvent::Behaviour(HyperspaceBehaviourEvent::Gossipsub(gossipsub::Event::Message {
            message,
            ..
        })) => {
            if let Ok(entry) = serde_json::from_slice::<MetaRouterEntry>(&message.data) {
                let _ = evt_tx.send(SwarmEvent2::MetaRouterUpdate(entry)).await;
            }
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            info!("📡 Listening on {address}");
        }
        _ => {}
    }
}

/// Spawn the swarm event loop as a background task.
pub fn spawn_swarm(
    swarm: Swarm<HyperspaceBehaviour>,
    cmd_rx: mpsc::Receiver<SwarmCommand>,
    evt_tx: mpsc::Sender<SwarmEvent2>,
    listen_addr: Multiaddr,
    bootstrap_peers: Vec<Multiaddr>,
) {
    tokio::spawn(async move {
        if let Err(e) = run_swarm(swarm, cmd_rx, evt_tx, listen_addr, bootstrap_peers).await {
            tracing::error!("P2P swarm crashed: {e}");
        }
    });
}
