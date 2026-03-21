use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashMap;

use hyperspace_index::HnswIndex;
use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_store::VectorStore;

type LocalIndex = HnswIndex<1024, EuclideanMetric>;

#[derive(Parser)]
#[command(author, version, about = "HiveMind CLI - HyperspaceDB Local Demo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Insert a random vector
    Insert { id: u32 },
    /// Search for neighbors
    Search { id: u32, k: u32 },
    /// Show index statistics
    Stats,
}

fn get_db_paths() -> (PathBuf, PathBuf) {
    let base = dirs::home_dir().unwrap().join(".hivemind_cli");
    if !base.exists() {
        std::fs::create_dir_all(&base).unwrap();
    }
    (base.join("store"), base.join("index.snap"))
}

fn init_index(store_path: PathBuf, snap_path: PathBuf) -> Result<LocalIndex> {
    let store = Arc::new(VectorStore::new(&store_path, 4096)); // 1024 * 4 bytes
    let config = Arc::new(GlobalConfig::default());
    
    if snap_path.exists() {
        if let Ok(loaded) = LocalIndex::load_snapshot(&snap_path, store.clone(), QuantizationMode::None, config.clone()) {
            return Ok(loaded);
        }
    }
    
    Ok(LocalIndex::new(store, QuantizationMode::None, config))
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let (store_path, snap_path) = get_db_paths();
    let mut index = init_index(store_path, snap_path.clone())?;

    match cli.command {
        Commands::Insert { id } => {
            let vec = vec![0.1f64; 1024]; // Simple dummy
            index.insert(&vec, HashMap::new())?;
            index.save_snapshot(&snap_path)?;
            println!("✅ Inserted vector {} and saved snapshot.", id);
        }
        Commands::Search { id, k } => {
            let query = vec![0.1f64; 1024];
            let results = index.search(&query, k as usize)?;
            println!("🔍 Top {} results:", results.len());
            for res in results {
                println!("  ID: {}, Distance: {:.4}", res.id, res.distance);
            }
        }
        Commands::Stats => {
            println!("📊 Index Stats:");
            println!("  Nodes: {}", index.count_nodes());
        }
    }

    Ok(())
}
