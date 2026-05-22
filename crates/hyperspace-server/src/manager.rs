use crate::collection::CollectionImpl;
use dashmap::DashMap;
use hyperspace_core::VacuumFilterQuery;
use hyperspace_core::{
    Collection, CosineMetric, EuclideanMetric, HybridMetric, LorentzMetric, PoincareMetric,
};
use hyperspace_proto::hyperspace::{
    replication_log, CreateCollectionOp, DeleteCollectionOp, ReplicationLog,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
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
    pub meta: CollectionMetadata,
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
    pub event_tx: broadcast::Sender<hyperspace_proto::hyperspace::EventMessage>,
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

    pub fn new(
        base_path: PathBuf,
        replication_tx: broadcast::Sender<ReplicationLog>,
        event_tx: broadcast::Sender<hyperspace_proto::hyperspace::EventMessage>,
    ) -> Self {
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
            event_tx,
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

        macro_rules! inst {
            ($M:ty) => {
                Arc::new(
                    CollectionImpl::<$M>::new(
                        name.to_string(),
                        node_id.clone(),
                        col_dir.clone(),
                        wal_path.clone(),
                        quant_mode,
                        self.replication_tx.clone(),
                        meta.dimension() as usize,
                        meta.get_schema(),
                    )
                    .await?,
                )
            };
        }

        let collection: Arc<dyn Collection> = match meta.metric_name().as_str() {
            "poincare" => inst!(PoincareMetric),
            "euclidean" | "l2" => inst!(EuclideanMetric),
            "cosine" => inst!(CosineMetric),
            "lorentz" => inst!(LorentzMetric),
            "hybrid" => inst!(HybridMetric),
            _ => {
                return Err(format!(
                    "Unsupported configuration: dim={}, metric={}",
                    meta.dimension(),
                    meta.metric_name()
                )
                .into());
            }
        };

        let entry = CollectionEntry {
            collection,
            last_accessed: AtomicU64::new(current_time_secs()),
            meta,
        };
        self.collections.insert(name.to_string(), entry);
        Ok(())
    }

    pub async fn create_collection(
        &self,
        user_id: &str,
        name: &str,
        schema: hyperspace_proto::hyperspace::CollectionSchema,
    ) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        self.create_collection_internal(&internal_name, schema, true)
            .await
    }

    pub async fn create_collection_from_replication(
        &self,
        name: &str,
        schema: hyperspace_proto::hyperspace::CollectionSchema,
    ) -> Result<(), String> {
        self.create_collection_internal(name, schema, false).await
    }

    pub async fn rebuild_collection(&self, user_id: &str, name: &str) -> Result<(), String> {
        self.rebuild_collection_with_filter(user_id, name, None)
            .await
    }

    pub async fn rebuild_collection_with_filter(
        &self,
        user_id: &str,
        name: &str,
        filter: Option<VacuumFilterQuery>,
    ) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        // Trigger optimization (Hot Vacuum)
        if let Some(entry) = self.collections.get(&internal_name) {
            entry
                .collection
                .optimize_with_filter(filter)
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
        schema: hyperspace_proto::hyperspace::CollectionSchema,
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
            .unwrap_or("medium".to_string())
            .to_lowercase();
        // Normalise user-facing level names to canonical storage values:
        //   none   → "none"    (no quantization, full f64)
        //   medium → "medium"  (ScalarI8 for standard metrics; AsymmetricHybrid801 for hybrid dim=801)
        //   extreme → "extreme" (Binary: 1-bit per dimension)
        // Legacy aliases: "scalar" → "medium", "binary" → "extreme", "asymmetric_hybrid_801" → "medium"
        let quantization = match quantization.as_str() {
            "none" => "none".to_string(),
            "extreme" | "binary" => "extreme".to_string(),
            "asymmetric_hybrid_801" => "medium".to_string(), // legacy alias
            _ => "medium".to_string(), // scalar, medium, anything else
        };

        let meta = CollectionMetadata {
            schema: Some(schema.clone()),
            dimension: None,
            metric: None,
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
                        schema: Some(schema),
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
            .map(|entry| entry.collection.clone())
    }

    pub async fn get_metadata(&self, user_id: &str, name: &str) -> Option<CollectionMetadata> {
        let internal_name = Self::get_internal_name(user_id, name);
        self.collections
            .get(&internal_name)
            .map(|entry| entry.meta.clone())
    }

    pub fn is_active(&self, user_id: &str, name: &str) -> bool {
        let internal_name = Self::get_internal_name(user_id, name);
        self.collections.contains_key(&internal_name)
    }

    pub fn get_metadata_no_wake(&self, user_id: &str, name: &str) -> Option<CollectionMetadata> {
        let internal_name = Self::get_internal_name(user_id, name);
        if let Some(entry) = self.collections.get(&internal_name) {
            return Some(entry.meta.clone());
        }
        let col_dir = self.base_path.join(&internal_name);
        if col_dir.exists() && col_dir.join("meta.json").exists() {
            if let Ok(meta) = CollectionMetadata::load(&col_dir) {
                return Some(meta);
            }
        }
        None
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

    pub async fn list_detailed(
        &self,
        user_id: &str,
    ) -> Vec<hyperspace_proto::hyperspace::CollectionSummary> {
        let names = self.list(user_id);
        let mut summaries = Vec::new();
        for name in names {
            if let Some(col) = self.get(user_id, &name).await {
                let schema = self
                    .get_metadata(user_id, &name)
                    .await
                    .and_then(|m| m.schema);
                summaries.push(hyperspace_proto::hyperspace::CollectionSummary {
                    name: name.clone(),
                    count: col.count() as u64,
                    schema,
                });
            }
        }
        summaries
    }

    pub fn list_all(&self) -> Vec<String> {
        self.collections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn list_active_collections(&self) -> Vec<(String, Arc<dyn Collection>)> {
        self.collections
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().collection.clone()))
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

    pub async fn freeze_collection(&self, user_id: &str, name: &str) -> Result<(), String> {
        let internal_name = Self::get_internal_name(user_id, name);
        let col_dir = self.base_path.join(&internal_name);

        // 1. Remove from RAM if loaded
        if self.collections.remove(&internal_name).is_some() {
            println!("💤 Collection '{name}' manually frozen (unloaded from memory).");
            return Ok(());
        }

        // 2. If not in RAM, check if it exists on disk (already cold/frozen)
        if col_dir.exists() && col_dir.join("meta.json").exists() {
            return Ok(());
        }

        Err("Collection not found".to_string())
    }

    pub async fn unfreeze_collection(&self, user_id: &str, name: &str) -> Result<(), String> {
        // self.get() automatically wakes up the cold collection if it's on disk
        if self.get(user_id, name).await.is_some() {
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
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

    pub fn get_user_usage(&self, user_id: &str) -> UserUsage {
        let prefix = format!("{user_id}_");
        let mut usage = UserUsage::default();

        // 1. Scan memory for active collections vector count
        for entry in self.collections.iter() {
            if entry.key().starts_with(&prefix) {
                usage.vector_count += entry.value().collection.count();
                usage.collection_count += 1;
            }
        }

        // 2. Scan disk for all user directories (including cold ones)
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with(&prefix) {
                            usage.disk_usage_bytes += calculate_dir_size(&path).unwrap_or(0);

                            // If this collection wasn't found in memory during step 1,
                            // it means it's an idle collection on disk.
                            if !self.collections.contains_key(name) {
                                usage.collection_count += 1;
                            }
                        }
                    }
                }
            }
        }
        usage
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
                        // If no underscore, treat as "default_admin" or skip.
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CollectionMetadata {
    pub schema: Option<hyperspace_proto::hyperspace::CollectionSchema>,
    pub dimension: Option<u32>,
    pub metric: Option<String>,
    pub quantization: String,
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

    pub fn get_schema(&self) -> hyperspace_proto::hyperspace::CollectionSchema {
        if let Some(schema) = &self.schema {
            schema.clone()
        } else {
            hyperspace_proto::hyperspace::CollectionSchema {
                components: vec![hyperspace_proto::hyperspace::VectorComponent {
                    name: "default".to_string(),
                    metric: self.metric.clone().unwrap_or_else(|| "l2".to_string()),
                    full_dimension: self.dimension.unwrap_or(0),
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            }
        }
    }

    pub fn dimension(&self) -> usize {
        self.get_schema()
            .components
            .first()
            .map_or(0, |c| c.full_dimension as usize)
    }

    pub fn metric_name(&self) -> String {
        self.get_schema()
            .components
            .first()
            .map_or_else(|| "l2".to_string(), |c| c.metric.clone())
    }

    fn quantization_mode(&self) -> hyperspace_core::QuantizationMode {
        let metric = self.metric_name();
        let dim = self.dimension();
        match self.quantization.as_str() {
            "none" => hyperspace_core::QuantizationMode::None,
            "extreme" | "binary" => hyperspace_core::QuantizationMode::Binary,
            // "medium" (and legacy "scalar", "asymmetric_hybrid_801"):
            // Auto-select AsymmetricHybrid801 when the collection is Hybrid+dim=801,
            // otherwise use ScalarI8. This prevents:
            //   - ScalarI8 corrupting Lorentz time coordinates in hybrid vecs (quality loss)
            //   - AsymmetricHybrid801 panicking on non-801-dim collections
            _ => {
                if metric == "hybrid" && dim == 801 {
                    hyperspace_core::QuantizationMode::AsymmetricHybrid801
                } else {
                    hyperspace_core::QuantizationMode::ScalarI8
                }
            }
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
