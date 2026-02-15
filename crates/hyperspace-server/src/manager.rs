use crate::collection::CollectionImpl;
use dashmap::DashMap;
use hyperspace_core::{Collection, EuclideanMetric, PoincareMetric, CosineMetric};
use hyperspace_proto::hyperspace::{ReplicationLog, CreateCollectionOp, DeleteCollectionOp, replication_log};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::time::{Duration, Instant};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClusterRole {
    Leader,
    Follower,
    Standalone,
}

pub struct CollectionEntry {
    pub collection: Arc<dyn Collection>,
    pub last_accessed: Mutex<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub node_id: String,
    pub role: ClusterRole,
    pub upstream_peer: Option<String>, // For followers
    pub downstream_peers: Vec<String>, // For leaders
    pub logical_clock: u64,
}

impl ClusterState {
    pub fn new() -> Self {
        Self {
            node_id: Uuid::new_v4().to_string(),
            role: ClusterRole::Leader, // Defaults to Leader role.
            upstream_peer: None,
            downstream_peers: Vec::new(),
            logical_clock: 0,
        }
    }

    pub fn tick(&mut self) -> u64 {
        self.logical_clock += 1;
        self.logical_clock
    }

    pub fn merge(&mut self, remote_clock: u64) {
        if remote_clock > self.logical_clock {
            self.logical_clock = remote_clock;
        }
        self.logical_clock += 1;
    }
}

pub struct CollectionManager {
    base_path: PathBuf,
    // Stores entries with metadata (e.g., access time).
    collections: Arc<DashMap<String, CollectionEntry>>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    pub cluster_state: Arc<RwLock<ClusterState>>,
}

impl CollectionManager {
    pub fn new(base_path: PathBuf, replication_tx: broadcast::Sender<ReplicationLog>) -> Self {
        // Try load cluster state
        let state_path = base_path.join("cluster.json");
        let state = if state_path.exists() {
            let data = fs::read_to_string(&state_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| ClusterState::new())
        } else {
            let s = ClusterState::new();
            if let Ok(data) = serde_json::to_string_pretty(&s) {
                // Create dir if needed
                let _ = fs::create_dir_all(&base_path);
                let _ = fs::write(&state_path, data);
            }
            s
        };

        let collections = Arc::new(DashMap::<String, CollectionEntry>::new());
        let mgr_map = collections.clone();

        // Spawns background reaper for idle collection eviction.
        tokio::spawn(async move {
            let timeout = Duration::from_hours(1); // 1 hour idle timeout
            loop {
                tokio::time::sleep(Duration::from_mins(1)).await;
                
                let now = Instant::now();
                let mut to_remove = Vec::new();
                
                for r in mgr_map.iter() {
                    let key = r.key().clone();
                    let entry = r.value();
                    if let Ok(last) = entry.last_accessed.lock() {
                        if now.duration_since(*last) > timeout {
                            to_remove.push(key);
                        }
                    }
                }
                
                for key in to_remove {
                    if mgr_map.remove(&key).is_some() {
                        println!("💤 Idling collection '{key}' unloaded from memory");
                    }
                }
            }
        });

        Self {
            base_path,
            collections,
            replication_tx,
            cluster_state: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn load_existing(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)?;
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Load metadata to determine dimension and metric


                    if let Ok(meta) = CollectionMetadata::load(&path) {
                        self.instantiate_collection(name, meta).await?;
                        println!("Loaded collection: {name}");
                    } else {
                        eprintln!("Skipping unknown directory (no meta.json): {name}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn instantiate_collection(
        &self,
        name: &str,
        meta: CollectionMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let col_dir = self.base_path.join(name);
        let wal_path = col_dir.join("wal.log");
        let quant_mode = meta.quantization_mode();
        let node_id = self.cluster_state.read().await.node_id.clone();

        // Helper macro to reduce boilerplate
        macro_rules! inst {
            ($N:expr, $M:ty) => {
                Arc::new(
                    CollectionImpl::<$N, $M>::new(
                        name.to_string(),
                        node_id.clone(),
                        col_dir.clone(),
                        wal_path.clone(),
                        quant_mode,
                        self.replication_tx.clone(),
                    )
                    .await?,
                )
            };
        }

        let collection: Arc<dyn Collection> = match (meta.dimension, meta.metric.as_str()) {
            // Hyperbolic (Poincaré)
            (8, "poincare") => inst!(8, PoincareMetric),
            (16, "poincare") => inst!(16, PoincareMetric),
            (32, "poincare") => inst!(32, PoincareMetric),
            (64, "poincare") => inst!(64, PoincareMetric),
            (128, "poincare") => inst!(128, PoincareMetric),
            (768, "poincare") => inst!(768, PoincareMetric),
            (1024, "poincare") => inst!(1024, PoincareMetric),
            (1536, "poincare") => inst!(1536, PoincareMetric),
            (2048, "poincare") => inst!(2048, PoincareMetric),

            // Euclidean (L2)
            (8, "euclidean" | "l2") => inst!(8, EuclideanMetric),
            (16, "euclidean" | "l2") => inst!(16, EuclideanMetric),
            (32, "euclidean" | "l2") => inst!(32, EuclideanMetric),
            (64, "euclidean" | "l2") => inst!(64, EuclideanMetric),
            (128, "euclidean" | "l2") => inst!(128, EuclideanMetric),
            (768, "euclidean" | "l2") => inst!(768, EuclideanMetric),
            (1024, "euclidean" | "l2") => inst!(1024, EuclideanMetric),
            (1536, "euclidean" | "l2") => inst!(1536, EuclideanMetric),
            (2048, "euclidean" | "l2") => inst!(2048, EuclideanMetric),

            // Cosine Similarity
            (8, "cosine") => inst!(8, CosineMetric),
            (16, "cosine") => inst!(16, CosineMetric),
            (32, "cosine") => inst!(32, CosineMetric),
            (64, "cosine") => inst!(64, CosineMetric),
            (128, "cosine") => inst!(128, CosineMetric),
            (768, "cosine") => inst!(768, CosineMetric),
            (1024, "cosine") => inst!(1024, CosineMetric),
            (1536, "cosine") => inst!(1536, CosineMetric),
            (2048, "cosine") => inst!(2048, CosineMetric),

            _ => {
                return Err(format!(
                    "Unsupported configuration: dim={}, metric={}",
                    meta.dimension, meta.metric
                )
                .into());
            }
        };

        
        let entry = CollectionEntry {
            collection,
            last_accessed: Mutex::new(Instant::now()),
        };
        self.collections.insert(name.to_string(), entry);
        Ok(())
    }

    pub async fn create_collection(
        &self,
        name: &str,
        dimension: u32,
        metric: &str,
    ) -> Result<(), String> {
        self.create_collection_internal(name, dimension, metric, true).await
    }

    pub async fn create_collection_from_replication(
        &self,
        name: &str,
        dimension: u32,
        metric: &str,
    ) -> Result<(), String> {
        self.create_collection_internal(name, dimension, metric, false).await
    }

    pub async fn rebuild_collection(&self, name: &str) -> Result<(), String> {
        // 1. Trigger optimization (builds new index side-by-side)
        if let Some(entry) = self.collections.get(name) {
             entry.collection.optimize().map_err(|e| format!("Optimization failed: {e}"))?;
        } else {
             return Err("Collection not found".to_string());
        }

        // 2. Remove from memory (triggers Drop -> tasks abort)
        self.collections.remove(name);
        
        // 3. Filesystem Swap
        let col_dir = self.base_path.join(name);
        let index_path = col_dir.join("index");
        let opt_path = col_dir.join("index.optimized");
        let backup_path = col_dir.join("index.backup");
        
        if opt_path.exists() {
             println!("🔄 Swapping index for '{name}'...");
             if index_path.exists() {
                 std::fs::rename(&index_path, &backup_path).map_err(|e| e.to_string())?;
             }
             if let Err(e) = std::fs::rename(&opt_path, &index_path) {
                 // Rollback
                 println!("❌ Swap failed: {e}. Rolling back...");
                 if backup_path.exists() {
                    let _ = std::fs::rename(&backup_path, &index_path);
                 }
                 return Err(e.to_string());
             }
             // Cleanup backup
             let _ = std::fs::remove_dir_all(&backup_path);
        } else {
            // If optimization didn't create file (e.g. empty collection), just reload
            println!("⚠️ No optimized index found (maybe empty?). Reloading existing.");
        }
        
        // 4. Reload
        let meta = CollectionMetadata::load(&col_dir).map_err(|e| e.to_string())?;
        self.instantiate_collection(name, meta).await.map_err(|e| e.to_string())?;
        
        println!("✅ Collection '{name}' rebuilt and reloaded successfully.");
        Ok(())
    }

    async fn create_collection_internal(
        &self,
        name: &str,
        dimension: u32,
        metric: &str,
        replicate: bool,
    ) -> Result<(), String> {
        if self.collections.contains_key(name) {
            return Err(format!("Collection '{name}' already exists"));
        }

        let col_dir = self.base_path.join(name);
        if !col_dir.exists() {
            fs::create_dir_all(&col_dir).map_err(|e| e.to_string())?;
        }

        let quantization = std::env::var("HS_QUANTIZATION_LEVEL")
            .unwrap_or("scalar".to_string())
            .to_lowercase();

        let meta = CollectionMetadata {
            dimension,
            metric: metric.to_string(),
            quantization,
        };

        meta.save(&col_dir).map_err(|e| e.to_string())?;

        self.instantiate_collection(name, meta)
            .await
            .map_err(|e| e.to_string())?;

        if replicate {
            // Broadcast replication event
            let clock = self.tick_cluster_clock().await;
            let log = ReplicationLog {
                logical_clock: clock,
                origin_node_id: self.cluster_state.read().await.node_id.clone(),
                collection: name.to_string(),
                operation: Some(replication_log::Operation::CreateCollection(CreateCollectionOp {
                    dimension,
                    metric: metric.to_string(),
                })),
            };
            let _ = self.replication_tx.send(log);
        }

        Ok(())
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Collection>> {
        // 1. Fast path: Check memory
        if let Some(entry) = self.collections.get(name) {
            // Update LRU clock
            if let Ok(mut t) = entry.last_accessed.lock() {
                *t = Instant::now();
            }
            return Some(entry.collection.clone());
        }

        // 2. Slow path: Check disk (Lazy Loading) - Wake up cold collection
        let col_dir = self.base_path.join(name);
        if col_dir.exists() && col_dir.join("meta.json").exists() {
            // Try to load metadata and revive collection
            if let Ok(meta) = CollectionMetadata::load(&col_dir) {
                println!("🧊 Waking up cold collection: '{name}'");
                if let Ok(()) = self.instantiate_collection(name, meta).await {
                    // Check map again after loading
                    if let Some(entry) = self.collections.get(name) {
                        return Some(entry.collection.clone());
                    }
                } else {
                    eprintln!("Failed to revive cold collection '{name}'");
                }
            }
        }
        
        None
    }

    pub fn list(&self) -> Vec<String> {
        self.collections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub async fn tick_cluster_clock(&self) -> u64 {
        let mut state = self.cluster_state.write().await;
        state.tick()
    }

    pub async fn merge_cluster_clock(&self, remote_clock: u64) {
        let mut state = self.cluster_state.write().await;
        state.merge(remote_clock);
    }

    pub async fn delete_collection(&self, name: &str) -> Result<(), String> {
        self.delete_collection_internal(name, true).await
    }

    pub async fn delete_collection_from_replication(&self, name: &str) -> Result<(), String> {
        self.delete_collection_internal(name, false).await
    }

    async fn delete_collection_internal(&self, name: &str, replicate: bool) -> Result<(), String> {
        if let Some((_, _col)) = self.collections.remove(name) {
            // Cleanup files
            let col_dir = self.base_path.join(name);
            if col_dir.exists() {
                fs::remove_dir_all(col_dir).map_err(|e| e.to_string())?;
            }

            if replicate {
                 let clock = self.tick_cluster_clock().await;
                 let log = ReplicationLog {
                    logical_clock: clock,
                    origin_node_id: self.cluster_state.read().await.node_id.clone(),
                    collection: name.to_string(),
                    operation: Some(replication_log::Operation::DeleteCollection(DeleteCollectionOp {})),
                 };
                 let _ = self.replication_tx.send(log);
            }
            Ok(())
        } else {
            Err(format!("Collection '{name}' not found"))
        }
    }


}

#[derive(serde::Serialize, serde::Deserialize)]
struct CollectionMetadata {
    dimension: u32,
    metric: String,
    quantization: String,
}

impl CollectionMetadata {
    fn save(&self, dir: &Path) -> std::io::Result<()> {
        let s = serde_json::to_string_pretty(self)?;
        fs::write(dir.join("meta.json"), s)
    }

    fn load(dir: &Path) -> std::io::Result<Self> {
        let s = fs::read_to_string(dir.join("meta.json"))?;
        let meta: Self = serde_json::from_str(&s)?;
        Ok(meta)
    }

    fn quantization_mode(&self) -> hyperspace_core::QuantizationMode {
        match self.quantization.as_str() {
            "binary" => hyperspace_core::QuantizationMode::Binary,
            "none" => hyperspace_core::QuantizationMode::None,
            _ => hyperspace_core::QuantizationMode::ScalarI8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_clock() {
        let mut state = ClusterState::new();
        assert_eq!(state.logical_clock, 0);

        // Tick
        let t1 = state.tick();
        assert_eq!(t1, 1);
        assert_eq!(state.logical_clock, 1);

        // Merge (no change)
        state.merge(0);
        assert_eq!(state.logical_clock, 2); // merge behaves as event (+1)

        // Merge (remote is ahead)
        state.merge(10);
        assert_eq!(state.logical_clock, 11); // max(2, 10) + 1 = 11
    }
}
