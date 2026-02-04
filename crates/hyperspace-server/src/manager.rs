use crate::collection::CollectionImpl;
use dashmap::DashMap;
use hyperspace_core::{Collection, PoincareMetric, EuclideanMetric};
use hyperspace_proto::hyperspace::ReplicationLog;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct CollectionManager {
    base_path: PathBuf,
    collections: DashMap<String, Arc<dyn Collection>>,
    replication_tx: broadcast::Sender<ReplicationLog>,
}

impl CollectionManager {
    pub fn new(base_path: PathBuf, replication_tx: broadcast::Sender<ReplicationLog>) -> Self {
        Self {
            base_path,
            collections: DashMap::new(),
            replication_tx,
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
                    // Try to load config.json to know dimension/metric?
                    // For now, MVP assumes 1024 Poincare for everyone or uses a metadata file.
                    // To properly support dynamic loading, we need to save metadata about each collection.
                    // For this MVP, let's assume we store a "meta.json" in the collection dir.
                    
                    if let Ok(meta) = CollectionMetadata::load(&path) {
                         self.instantiate_collection(name, meta).await?;
                         println!("Loaded collection: {}", name);
                    } else {
                        eprintln!("Skipping unknown directory (no meta.json): {}", name);
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

        // Dispatch based on generic args (N, M).
        // This is the "Dispatcher" logic moved here.
        let collection: Arc<dyn Collection> = match (meta.dimension, meta.metric.as_str()) {
            // Hyperbolic (Poincaré) configurations
            (16, "poincare") => Arc::new(
                CollectionImpl::<16, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (32, "poincare") => Arc::new(
                CollectionImpl::<32, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (64, "poincare") => Arc::new(
                CollectionImpl::<64, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (128, "poincare") => Arc::new(
                CollectionImpl::<128, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (768, "poincare") => Arc::new(
                CollectionImpl::<768, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (1024, "poincare") => Arc::new(
                CollectionImpl::<1024, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (1536, "poincare") => Arc::new(
                CollectionImpl::<1536, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (2048, "poincare") => Arc::new(
                CollectionImpl::<2048, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (8, "poincare") => Arc::new(
                CollectionImpl::<8, PoincareMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            
            // Euclidean configurations
            (1024, "euclidean") => Arc::new(
                CollectionImpl::<1024, EuclideanMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (1536, "euclidean") => Arc::new(
                CollectionImpl::<1536, EuclideanMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            (2048, "euclidean") => Arc::new(
                CollectionImpl::<2048, EuclideanMetric>::new(
                    name.to_string(),
                    col_dir,
                    wal_path,
                    quant_mode,
                    self.replication_tx.clone(),
                )
                .await?,
            ),
            
            // Add more as needed
            _ => return Err(format!("Unsupported configuration: dim={}, metric={}", meta.dimension, meta.metric).into()),
        };

        self.collections.insert(name.to_string(), collection);
        Ok(())
    }

    pub async fn create_collection(
        &self,
        name: &str,
        dimension: u32,
        metric: &str,
    ) -> Result<(), String> {
        if self.collections.contains_key(name) {
            return Err(format!("Collection '{}' already exists", name));
        }

        let col_dir = self.base_path.join(name);
        if !col_dir.exists() {
            fs::create_dir_all(&col_dir).map_err(|e| e.to_string())?;
        }

        let meta = CollectionMetadata {
            dimension,
            metric: metric.to_string(),
            quantization: "scalar".to_string(), // Default to scalar
        };
        
        meta.save(&col_dir).map_err(|e| e.to_string())?;

        self.instantiate_collection(name, meta)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Collection>> {
        self.collections.get(name).map(|c| c.clone())
    }

    pub fn delete_collection(&self, name: &str) -> Result<(), String> {
        if let Some((_, _col)) = self.collections.remove(name) {
            // Cleanup files
            let col_dir = self.base_path.join(name);
            if col_dir.exists() {
                fs::remove_dir_all(col_dir).map_err(|e| e.to_string())?;
            }
            Ok(())
        } else {
            Err(format!("Collection '{}' not found", name))
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.collections.iter().map(|entry| entry.key().clone()).collect()
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
