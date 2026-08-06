/// Server startup arguments — shared between `main.rs` (hyperspace-server binary)
/// and `src/bin/hyperspace-miner.rs` (DePIN miner binary).
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Port to listen on (gRPC)
    #[arg(short, long, default_value = "50051")]
    pub port: u16,

    /// HTTP Dashboard Port
    #[arg(long, default_value = "50050")]
    pub http_port: u16,

    /// Role: leader or follower
    #[arg(long, default_value = "leader")]
    pub role: String,

    /// Leader address (if follower)
    #[arg(long)]
    pub leader: Option<String>,

    /// User ID for multi-tenant replication (if follower)
    #[arg(long)]
    pub user_id: Option<String>,

    /// Unique Node ID for this instance
    #[arg(long)]
    pub node_id: Option<String>,

    /// Allow outgoing replication streams?
    #[arg(long, default_value = "false", env = "HS_REPLICATION_ALLOWED")]
    pub replication_allowed: bool,
}
