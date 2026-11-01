/// Server startup arguments — shared between `main.rs` (hyperspace-server binary)
/// and `src/bin/hyperspace-miner.rs` (DePIN miner binary).
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

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

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Manage Hyperspace plugins and .hsp packages
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PluginCommands {
    /// Pack a plugin directory (with plugin.toml and compiled binary) into an .hsp package
    Pack {
        /// Path to directory containing plugin.toml and compiled library
        dir: std::path::PathBuf,
        /// Optional destination .hsp file path
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
    },
    /// Install an .hsp package file directly into plugins directory (Air-Gap or Hub)
    Install {
        /// Path to local .hsp file or plugin name from Hub
        target: String,
        /// Target plugins directory (defaults to ./plugins)
        #[arg(short, long, default_value = "./plugins")]
        dir: std::path::PathBuf,
        /// Offline mode: only install from local file, do not contact Hub
        #[arg(long)]
        offline: bool,
    },
}
