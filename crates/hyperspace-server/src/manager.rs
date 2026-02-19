use crate::collection::CollectionImpl;
use dashmap::DashMap;
use hyperspace_core::{Collection, CosineMetric, EuclideanMetric, LorentzMetric, PoincareMetric};
use hyperspace_proto::hyperspace::{
    replication_log, CreateCollectionOp, DeleteCollectionOp, ReplicationLog,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use parking_lot::Mutex;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use uuid::Uuid;

fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClusterRole {
    Leader,
    Follower,
    Standalone,
}

pub struct CollectionEntry {
    pub collection: Arc<dyn Collection>,
    pub last_accessed: AtomicU64,
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
    pub system: Arc<Mutex<System>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserUsage {
    pub collection_count: usize,
    pub vector_count: usize,
    pub disk_usage_bytes: u64,
}

impl CollectionManager {
    fn get_internal_name(user_id: &str, collection_name: &str) -> String {
        format!("{user_id}_{collection_name}")
    }

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
            let idle_timeout_sec = std::env::var("HS_IDLE_TIMEOUT_SEC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600);
            let timeout = Duration::from_secs(idle_timeout_sec);

            // Check at least as often as the timeout, but not more than once a minute (unless timeout is small)
            let check_interval = if idle_timeout_sec < 60 {
                Duration::from_secs(idle_timeout_sec)
            } else {
                Duration::from_mins(1)
            };

            loop {
                tokio::time::sleep(check_interval).await;

                let now_secs = current_time_secs();
                let mut to_remove = Vec::new();
                for r in mgr_map.iter() {
                    let key = r.key().clone();
                    let entry = r.value();
                    let last_secs = entry.last_accessed.load(Ordering::Relaxed);
                    if now_secs.saturating_sub(last_secs) > timeout.as_secs() {
                        to_remove.push(key);
                    }
                }

                for key in to_remove {
                    if mgr_map.remove(&key).is_some() {
                        println!("💤 Idling collection '{key}' unloaded from memory");
                    }
                }
            }
        });

        let system = Arc::new(Mutex::new(System::new_all()));
        let sys_clone = system.clone();

        // Spawn background task to refresh system metrics (CPU usage calculation requires history)
        tokio::spawn(async move {
            loop {
                {
                    let mut sys = sys_clone.lock();
                    sys.refresh_all();
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        Self {
            base_path,
            collections,
            replication_tx,
            cluster_state: Arc::new(RwLock::new(state)),
            system,
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

            // Lorentz Model
            (8, "lorentz") => inst!(8, LorentzMetric),
            (16, "lorentz") => inst!(16, LorentzMetric),
            (32, "lorentz") => inst!(32, LorentzMetric),
            (64, "lorentz") => inst!(64, LorentzMetric),
            (128, "lorentz") => inst!(128, LorentzMetric),
            (768, "lorentz") => inst!(768, LorentzMetric),
            (1024, "lorentz") => inst!(1024, LorentzMetric),
            (1536, "lorentz") => inst!(1536, LorentzMetric),
            (2048, "lorentz") => inst!(2048, LorentzMetric),

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
            last_accessed: AtomicU64::new(current_time_secs()),
        };
        self.collections.insert(name.to_string(), entry);
        Ok(())
    }

    pub async fn create_collection(
        &self,
        user_id: &str,
        name: &str,
        dimension: u32,
        metric: &str,
    ) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        self.create_collection_internal(&internal_name, dimension, metric, true)
            .await
    }

    pub async fn create_collection_from_replication(
        &self,
        name: &str,
        dimension: u32,
        metric: &str,
    ) -> Result<(), String> {
        self.create_collection_internal(name, dimension, metric, false)
            .await
    }

    pub async fn rebuild_collection(&self, user_id: &str, name: &str) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        // Trigger optimization (Hot Vacuum)
        if let Some(entry) = self.collections.get(&internal_name) {
            entry
                .collection
                .optimize()
                .await
                .map_err(|e| format!("Optimization failed: {e}"))?;
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    }

    pub fn get_collection_counts(&self) -> (usize, usize) {
        // Active: currently in DashMap (RAM)
        let active = self.collections.len();

        // Total: count directories in data folder
        let total = match std::fs::read_dir(&self.base_path) {
            Ok(entries) => entries
                .filter(|e| e.is_ok() && e.as_ref().unwrap().path().is_dir())
                .count(),
            Err(_) => 0,
        };

        // Idle = Total - Active
        let idle = total.saturating_sub(active);
        (active, idle)
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
                operation: Some(replication_log::Operation::CreateCollection(
                    CreateCollectionOp {
                        dimension,
                        metric: metric.to_string(),
                    },
                )),
            };
            let _ = self.replication_tx.send(log);
        }

        Ok(())
    }

    pub async fn get_internal(&self, internal_name: &str) -> Option<Arc<dyn Collection>> {
        self.collections
            .get(internal_name)
            .map(|c| c.value().collection.clone())
    }

    pub async fn get(&self, user_id: &str, name: &str) -> Option<Arc<dyn Collection>> {
        let internal_name = Self::get_internal_name(user_id, name);

        // 1. Fast path: Check memory
        if let Some(entry) = self.collections.get(&internal_name) {
            // Update LRU clock
            entry
                .last_accessed
                .store(current_time_secs(), Ordering::Relaxed);
            return Some(entry.collection.clone());
        }

        // 2. Slow path: Check disk (Lazy Loading) - Wake up cold collection
        let col_dir = self.base_path.join(&internal_name);
        if col_dir.exists() && col_dir.join("meta.json").exists() {
            // Try to load metadata and revive collection
            if let Ok(meta) = CollectionMetadata::load(&col_dir) {
                println!("🧊 Waking up cold collection: '{internal_name}'");
                if let Ok(()) = self.instantiate_collection(&internal_name, meta).await {
                    // Check map again after loading
                    if let Some(entry) = self.collections.get(&internal_name) {
                        return Some(entry.collection.clone());
                    }
                } else {
                    eprintln!("Failed to revive cold collection '{internal_name}'");
                }
            }
        }

        None
    }

    pub fn list(&self, user_id: &str) -> Vec<String> {
        let prefix = format!("{user_id}_");
        let mut collections: std::collections::HashSet<String> = self
            .collections
            .iter()
            .filter(|entry| entry.key().starts_with(&prefix))
            .map(|entry| {
                entry
                    .key()
                    .strip_prefix(&prefix)
                    .unwrap_or(entry.key())
                    .to_string()
            })
            .collect();

        // Also check disk for cold collections
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with(&prefix) && entry.path().is_dir() {
                        if let Some(stripped) = name.strip_prefix(&prefix) {
                            collections.insert(stripped.to_string());
                        }
                    }
                }
            }
        }

        let mut list: Vec<String> = collections.into_iter().collect();
        list.sort();
        list
    }

    pub fn list_all(&self) -> Vec<String> {
        self.collections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn total_vector_count(&self) -> usize {
        self.collections
            .iter()
            .map(|entry| entry.value().collection.count())
            .sum()
    }

    pub async fn tick_cluster_clock(&self) -> u64 {
        let mut state = self.cluster_state.write().await;
        state.tick()
    }

    pub async fn merge_cluster_clock(&self, remote_clock: u64) {
        let mut state = self.cluster_state.write().await;
        state.merge(remote_clock);
    }

    pub async fn delete_collection(&self, user_id: &str, name: &str) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        self.delete_collection_internal(&internal_name, true).await
    }

    pub async fn delete_collection_from_replication(&self, name: &str) -> Result<(), String> {
        self.delete_collection_internal(name, false).await
    }

    async fn delete_collection_internal(&self, name: &str, replicate: bool) -> Result<(), String> {
        let mut found = false;

        // 1. Remove from in-memory map
        if let Some((_, _col)) = self.collections.remove(name) {
            found = true;
        }

        // 2. Cleanup files (handles cold storage too)
        let col_dir = self.base_path.join(name);
        if col_dir.exists() {
            fs::remove_dir_all(col_dir).map_err(|e| e.to_string())?;
            found = true;
        }

        // 3. Replicate if it was found or if we want to ensure eventual consistency
        if replicate && found {
            let clock = self.tick_cluster_clock().await;
            let log = ReplicationLog {
                logical_clock: clock,
                origin_node_id: self.cluster_state.read().await.node_id.clone(),
                collection: name.to_string(),
                operation: Some(replication_log::Operation::DeleteCollection(
                    DeleteCollectionOp {},
                )),
            };
            let _ = self.replication_tx.send(log);
        }

        // Idempotent: return success even if not found
        Ok(())
    }

    pub fn get_usage_report(&self) -> std::collections::HashMap<String, UserUsage> {
        let mut report = std::collections::HashMap::new();

        // Scan data directory
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Parse {user_id}_{collection_name}
                        // We assume the first part before '_' is user_id.
                        // If no underscore (e.g. legacy), treat as "default_admin" or skip?
                        // Standard format: "{user_id}_{name}"

                        let user_id = if let Some((u, _)) = name.split_once('_') {
                            u
                        } else {
                            "unknown"
                        };

                        let size = calculate_dir_size(&path).unwrap_or(0);
                        let usage = report
                            .entry(user_id.to_string())
                            .or_insert(UserUsage::default());
                        usage.disk_usage_bytes += size;
                        usage.collection_count += 1;

                        // Vector count: only if active in memory
                        if let Some(entry) = self.collections.get(name) {
                            usage.vector_count += entry.collection.count();
                        }
                    }
                }
            }
        }
        report
    }
}

fn calculate_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += calculate_dir_size(&entry.path())?;
            }
        }
    }
    Ok(total_size)
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
