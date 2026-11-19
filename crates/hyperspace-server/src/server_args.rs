/// Server startup arguments — shared between `main.rs` (hyperspace-server binary)
/// and `src/bin/hyperspace-miner.rs` (DePIN miner binary).
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Port to listen on (gRPC). `--port` wins over `HS_PORT`.
    #[arg(short, long, default_value = "50051", env = "HS_PORT")]
    pub port: u16,

    /// HTTP Dashboard Port. `--http-port` wins over `HS_HTTP_PORT`.
    #[arg(long, default_value = "50050", env = "HS_HTTP_PORT")]
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

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    // One test so the env-var cases never race each other.
    #[test]
    fn ports_come_from_flags_env_or_defaults() {
        std::env::remove_var("HS_PORT");
        std::env::remove_var("HS_HTTP_PORT");
        let a = Args::try_parse_from(["hyperspace-server"]).unwrap();
        assert_eq!((a.port, a.http_port), (50051, 50050));

        std::env::set_var("HS_PORT", "61001");
        std::env::set_var("HS_HTTP_PORT", "61000");
        let a = Args::try_parse_from(["hyperspace-server"]).unwrap();
        assert_eq!((a.port, a.http_port), (61001, 61000));

        let a = Args::try_parse_from([
            "hyperspace-server",
            "--port",
            "62001",
            "--http-port",
            "62000",
        ])
        .unwrap();
        assert_eq!((a.port, a.http_port), (62001, 62000), "flags win over env");

        std::env::set_var("HS_PORT", "not-a-port");
        assert!(Args::try_parse_from(["hyperspace-server"]).is_err());

        std::env::remove_var("HS_PORT");
        std::env::remove_var("HS_HTTP_PORT");
    }
}
