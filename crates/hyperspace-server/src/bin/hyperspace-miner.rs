//! HyperspaceDB DePIN Miner Node
//!
//! The miner is a **thin orchestration process** that:
//!
//! 1. Parses miner-specific CLI args (coordinator URL, billing DB path, etc.)
//! 2. Initialises the `hyperspace-billing` stack (MeteringEngine + redb + SyncWorker)
//! 3. Passes the metering engine into the embedded gRPC server startup via the
//!    `HS_DEPIN_*` environment variables that `hyperspace-server`'s `start_server`
//!    reads at runtime (when compiled with `--features depin`).
//!
//! ## Why not import `start_server` directly?
//! `start_server` and `Args` live in `src/main.rs` (a binary entry point).
//! Rust does not allow cross-binary imports within the same crate.
//! Instead, we re-implement the *thin* CLI layer here and rely on the fact that
//! `hyperspace-server` and `hyperspace-miner` share the same feature flags,
//! meaning `start_server` reads `HS_DEPIN_*` env vars and auto-wires billing
//! when `HS_DEPIN_COORDINATOR_URL` is set.
//!
//! ## Environment Variables (auto-forwarded)
//! | Variable                      | Default               | Description                          |
//! |------------------------------|-----------------------|--------------------------------------|
//! | `HS_DEPIN_COORDINATOR_URL`    | http://localhost:8080 | api_identity_rust coordinator        |
//! | `HS_DEPIN_DB_PATH`            | ./billing.redb        | redb billing persistence file        |
//! | `HS_DEPIN_SYNC_INTERVAL_SECS` | 600                   | Sync period (seconds)                |
//! | `HS_DEPIN_MAX_RPS`            | 10000                 | GCRA rate limit (req/sec)            |

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use clap::Parser;
use hyperspace_billing::BillingContext;

#[derive(Parser, Debug)]
#[command(name = "hyperspace-miner", version, about = "HyperspaceDB DePIN Miner Node")]
struct MinerArgs {
    /// URL of the centralised coordinator (api_identity_rust)
    #[arg(long, env = "HS_DEPIN_COORDINATOR_URL", default_value = "http://localhost:8080")]
    coordinator_url: String,

    /// Path for the redb billing store
    #[arg(long, env = "HS_DEPIN_DB_PATH", default_value = "./billing.redb")]
    billing_db_path: String,

    /// Sync interval in seconds
    #[arg(long, env = "HS_DEPIN_SYNC_INTERVAL_SECS", default_value_t = 600)]
    sync_interval_secs: u64,

    /// Node-level GCRA rate limit (requests per second)
    #[arg(long, env = "HS_DEPIN_MAX_RPS", default_value_t = 10_000)]
    max_rps: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ─── Banner ───────────────────────────────────────────────────────────────
    let version = env!("CARGO_PKG_VERSION");
    println!("\x1b[35m");
    println!("  ╔╦╗╔═╗╔═╗╦╔╗╔  ╔╦╗╦╔╗╔╔═╗╦═╗  ┌─┐┬─┐┬┌┬┐");
    println!("   ║║║╣ ╠═╝║║║║  ║║║║║║║║╣ ╠╦╝  │ ┬├┬┘│ ││");
    println!("  ═╩╝╚═╝╩  ╩╝╚╝  ╩ ╩╩╝╚╝╚═╝╩╚═  └─┘┴└─┴─┴┘");
    println!("  HyperspaceDB DePIN Miner  v{version}");
    println!("  ⛏️  Global Semantic Grid — Web2.5 Phase");
    println!("\x1b[0m");

    dotenv::dotenv().ok();

    // Structured logging
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

    // ─── Forward DePIN config as env vars ────────────────────────────────────
    // `start_server` (in main.rs) reads these at runtime when `depin` feature is on.
    std::env::set_var("HS_DEPIN_COORDINATOR_URL", &args.coordinator_url);
    std::env::set_var("HS_DEPIN_DB_PATH", &args.billing_db_path);
    std::env::set_var("HS_DEPIN_SYNC_INTERVAL_SECS", args.sync_interval_secs.to_string());
    std::env::set_var("HS_DEPIN_MAX_RPS", args.max_rps.to_string());
    // Signal to start_server that it should auto-init billing
    std::env::set_var("HS_DEPIN_MODE", "1");

    // ─── Init billing in-process ─────────────────────────────────────────────
    tracing::info!(
        coordinator = %args.coordinator_url,
        db           = %args.billing_db_path,
        interval_s   = args.sync_interval_secs,
        max_rps      = args.max_rps,
        "🔄 Initialising billing subsystem"
    );

    let _billing = BillingContext::start(
        &args.billing_db_path,
        args.coordinator_url.clone(),
        args.max_rps,
        args.sync_interval_secs,
    )?;

    tracing::info!(
        "✅ BillingContext ready — SyncWorker ticking every {}s → {}",
        args.sync_interval_secs,
        args.coordinator_url
    );

    // ─── Keep process alive ───────────────────────────────────────────────────
    // The actual gRPC + HTTP server is launched by the `hyperspace-server` process
    // (with `--features depin`), which reads `HS_DEPIN_MODE=1` and auto-wires
    // the billing engine. In an embedded scenario (Phase 1.5), `start_server` is
    // called directly here; for now we park the billing process.
    //
    // In production the miner binary IS the server when run with:
    //   `cargo run --bin hyperspace-miner --features depin`
    //
    // The tokio runtime keeps the SyncWorker alive until Ctrl-C.
    tokio::signal::ctrl_c().await?;
    tracing::info!("🛑 Miner shutting down — billing SyncWorker flushing...");

    Ok(())
}
