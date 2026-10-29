//! HyperspaceDB DePIN Miner Node — Unified Process
//!
//! Single entry-point that:
//!   1. Parses miner-specific CLI args
//!   2. Initialises `hyperspace-billing` (MeteringEngine + redb + SyncWorker)
//!   3. Bootstraps `hyperspace-p2p` (NodeIdentity + libp2p + CoordinatorClient)
//!   4. Forwards all config as env vars that `start_server` reads
//!   5. Calls `start_server` in-process via the `__start_server` feature gate
//!
//! ## Usage
//! ```bash
//! cargo build --bin hyperspace-miner --features depin
//! ./hyperspace-miner \
//!   --coordinator-url http://api.hyperspace.db:8080 \
//!   --data-dir /var/hyperspace/data \
//!   --identity-file /var/hyperspace/identity.json \
//!   --grpc-port 50051 \
//!   --http-port 8080 \
//!   --p2p-port 7777
//! ```

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use clap::Parser;
use hyperspace_billing::BillingContext;
use hyperspace_server::{start_server, Args as ServerArgs};

#[derive(Parser, Debug)]
#[command(
    name = "hyperspace-miner",
    version,
    about = "HyperspaceDB DePIN Miner Node — distributed vector DB",
    long_about = "Single-process miner: billing + P2P + full HyperspaceDB gRPC/HTTP server."
)]
struct MinerArgs {
    // ─── Coordinator ──────────────────────────────────────────────────────────
    /// URL of the centralised coordinator (api_identity_rust)
    #[arg(
        long,
        env = "HS_DEPIN_COORDINATOR_URL",
        default_value = "http://localhost:8080"
    )]
    coordinator_url: String,

    // ─── Identity ─────────────────────────────────────────────────────────────
    /// Path to the node identity JSON file (BIP-39 mnemonic + PeerId)
    #[arg(long, env = "HS_IDENTITY_FILE", default_value = "./identity.json")]
    identity_file: String,

    /// Semantic zone (0–255) — partition of semantic space this node serves
    #[arg(long, env = "HS_SEMANTIC_ZONE", default_value_t = 0u8)]
    semantic_zone: u8,

    // ─── Ports ────────────────────────────────────────────────────────────────
    /// gRPC port (HyperspaceDB API)
    #[arg(short = 'p', long, env = "HS_GRPC_PORT", default_value_t = 50051u16)]
    grpc_port: u16,

    /// HTTP port (REST API + Dashboard)
    #[arg(long, env = "HS_HTTP_PORT", default_value_t = 8080u16)]
    http_port: u16,

    /// P2P networking port (libp2p QUIC)
    #[arg(long, env = "HS_P2P_PORT", default_value_t = 7777u16)]
    p2p_port: u16,

    // ─── Data ─────────────────────────────────────────────────────────────────
    /// Data directory for .hyp chunk files and collections
    #[arg(long, env = "HS_DATA_DIR", default_value = "./data")]
    data_dir: String,

    // ─── Billing ──────────────────────────────────────────────────────────────
    /// Path for the redb billing persistence file
    #[arg(long, env = "HS_DEPIN_DB_PATH", default_value = "./billing.redb")]
    billing_db_path: String,

    /// Billing sync interval in seconds
    #[arg(long, env = "HS_DEPIN_SYNC_INTERVAL_SECS", default_value_t = 60u64)]
    sync_interval_secs: u64,

    /// Node-level GCRA rate limit (max requests/second across all tenants)
    #[arg(long, env = "HS_DEPIN_MAX_RPS", default_value_t = 10_000u32)]
    max_rps: u32,

    // ─── Network ──────────────────────────────────────────────────────────────
    /// Public IP address of this node (auto-detect if not set)
    #[arg(long, env = "HS_PUBLIC_IP")]
    public_ip: Option<String>,

    /// Bootstrap P2P peers (comma-separated multiaddrs)
    #[arg(long, env = "HS_BOOTSTRAP_PEERS", default_value = "")]
    bootstrap_peers: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    hyperspace_core::check_simd();

    // ─── Banner ───────────────────────────────────────────────────────────────
    let version = env!("CARGO_PKG_VERSION");
    println!("\x1b[35m");
    println!("  ╔╦╗╔═╗╔═╗╦╔╗╔  ╔╦╗╦╔╗╔╔═╗╦═╗");
    println!("   ║║║╣ ╠═╝║║║║  ║║║║║║║║╣ ╠╦╝");
    println!("  ═╩╝╚═╝╩  ╩╝╚╝  ╩ ╩╩╝╚╝╚═╝╩╚═");
    println!("  HyperspaceDB DePIN Miner  v{version}");
    println!("  ⛏️  Global Semantic Grid — Web2.5 Phase");
    println!("\x1b[0m");

    // ─── Tracing ──────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "hyperspace_billing=info,hyperspace_p2p=info,warn"
                    .parse()
                    .unwrap()
            }),
        )
        .compact()
        .init();

    let args = MinerArgs::parse();

    // ─── Resolve public IP ────────────────────────────────────────────────────
    let public_ip = match args.public_ip.clone() {
        Some(ip) => ip,
        None => auto_detect_ip().unwrap_or_else(|| "127.0.0.1".to_string()),
    };
    tracing::info!("🌐 Public IP: {public_ip}");

    // ─── Forward all env vars that start_server reads ─────────────────────────
    // start_server() in main.rs reads HS_DEPIN_MODE=1 to auto-init billing,
    // but in miner mode we inject metering directly → no double-init.
    std::env::set_var("HS_DATA_DIR", &args.data_dir);
    std::env::set_var("HS_DEPIN_MODE", "1");
    std::env::set_var("HS_DEPIN_COORDINATOR_URL", &args.coordinator_url);
    std::env::set_var("HS_DEPIN_DB_PATH", &args.billing_db_path);
    std::env::set_var(
        "HS_DEPIN_SYNC_INTERVAL_SECS",
        args.sync_interval_secs.to_string(),
    );
    std::env::set_var("HS_DEPIN_MAX_RPS", args.max_rps.to_string());

    // ─── Phase 1: Init billing subsystem ──────────────────────────────────────
    tracing::info!(
        coordinator = %args.coordinator_url,
        db           = %args.billing_db_path,
        interval_s   = args.sync_interval_secs,
        max_rps      = args.max_rps,
        "🔄 Starting billing subsystem"
    );

    let billing = BillingContext::start(
        &args.billing_db_path,
        args.coordinator_url.clone(),
        args.max_rps,
        args.sync_interval_secs,
    )?;

    tracing::info!(
        "✅ BillingContext ready — SyncWorker → {} (every {}s)",
        args.coordinator_url,
        args.sync_interval_secs
    );

    // ─── Phase 2: P2P identity + coordinator registration ─────────────────────
    tracing::info!(
        identity_file = %args.identity_file,
        zone          = args.semantic_zone,
        p2p_port      = args.p2p_port,
        "🌐 Bootstrapping P2P network"
    );

    let (node_identity, _meta_router) = hyperspace_p2p::bootstrap(
        &args.identity_file,
        args.semantic_zone,
        args.coordinator_url.clone(),
        args.grpc_port,
        args.p2p_port,
        public_ip.clone(),
    )
    .await?;

    let peer_id_str = node_identity.peer_id.to_base58();
    tracing::info!(peer_id = %peer_id_str, zone = node_identity.semantic_zone, "🆔 Node identity ready");

    // Build the ServerArgs that start_server expects, forwarding our miner config
    let server_args = ServerArgs {
        port: args.grpc_port,
        http_port: args.http_port,
        role: "leader".to_string(),
        leader: None,
        user_id: None,
        node_id: Some(peer_id_str),
        replication_allowed: false,
    };

    tracing::info!(
        grpc_port = args.grpc_port,
        http_port = args.http_port,
        data_dir  = %args.data_dir,
        "🚀 DePIN miner ready — starting in-process gRPC/HTTP server"
    );

    // start_server blocks until Ctrl-C or shutdown
    start_server(server_args, Some(billing)).await?;

    tracing::info!("🛑 Miner shutting down");
    Ok(())
}

/// Auto-detect outbound IP via UDP socket (no packets sent).
fn auto_detect_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}
