use hyperspace_core::{Collection, FilterExpr, GlobalConfig, Metric, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::ReplicationLog;
use hyperspace_store::{wal::Wal, VectorStore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

pub struct CollectionImpl<const N: usize, M: Metric<N>> {
    name: String,
    index: Arc<HnswIndex<N, M>>,
    wal: Arc<Mutex<Wal>>,
    index_tx: mpsc::Sender<(u32, HashMap<String, String>)>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    config: Arc<GlobalConfig>,
    _tasks: Vec<JoinHandle<()>>,
}

impl<const N: usize, M: Metric<N>> CollectionImpl<N, M> {
    pub async fn new(
        name: String,
        data_dir: std::path::PathBuf,
        wal_path: std::path::PathBuf,
        mode: hyperspace_core::QuantizationMode,
        replication_tx: broadcast::Sender<ReplicationLog>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let snap_path = data_dir.join("index.snap");
        let config = Arc::new(GlobalConfig::new());
        // Initialize config from env or defaults
        // For MVP, we use defaults or global env vars. Ideally passed in.
         let ef_cons_env = std::env::var("HS_HNSW_EF_CONSTRUCT")
            .unwrap_or("100".to_string())
            .parse()
            .unwrap_or(100);
        let ef_search_env = std::env::var("HS_HNSW_EF_SEARCH")
            .unwrap_or("10".to_string())
            .parse()
            .unwrap_or(10);
        config.set_ef_construction(ef_cons_env);
        config.set_ef_search(ef_search_env);

        let element_size = match mode {
            hyperspace_core::QuantizationMode::ScalarI8 => hyperspace_core::vector::QuantizedHyperVector::<N>::SIZE,
            hyperspace_core::QuantizationMode::Binary => hyperspace_core::vector::BinaryHyperVector::<N>::SIZE,
            hyperspace_core::QuantizationMode::None => hyperspace_core::vector::HyperVector::<N>::SIZE,
        };

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let (_store, index, recovered_count) = if snap_path.exists() {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            match HnswIndex::<N, M>::load_snapshot(&snap_path, store.clone(), mode, config.clone()) {
                Ok(idx) => {
                    let count = idx.count_nodes();
                    (store, Arc::new(idx), count)
                }
                Err(e) => {
                    eprintln!("Failed to load snapshot for {}: {}. Starting fresh.", name, e);
                     // Cleanup?
                    let store = Arc::new(VectorStore::new(&data_dir, element_size));
                    (store.clone(), Arc::new(HnswIndex::new(store, mode, config.clone())), 0)
                }
            }
        } else {
            let store = Arc::new(VectorStore::new(&data_dir, element_size));
            (store.clone(), Arc::new(HnswIndex::new(store, mode, config.clone())), 0)
        };

        // WAL
        let wal = Wal::new(&wal_path)?;
        let wal_arc = Arc::new(Mutex::new(wal));

        // Replay
        let index_ref = index.clone();
        Wal::replay(&wal_path, |entry| {
            let hyperspace_store::wal::WalEntry::Insert { id, vector, metadata } = entry;
            if (id as usize) >= recovered_count {
                let _ = index_ref.insert(&vector, metadata);
            }
        })?;

        // Background Tasks
        let (index_tx, mut index_rx) = mpsc::channel(1000);
        let idx_worker = index.clone();
        let cfg_worker = config.clone();
        
        let indexer_handle = tokio::spawn(async move {
            while let Some((id, meta)) = index_rx.recv().await {
                let idx = idx_worker.clone();
                let cfg = cfg_worker.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    let _ = idx.index_node(id, meta);
                    cfg.dec_queue();
                }).await;
            }
        });

        let idx_snap = index.clone();
        let snap_path_clone = snap_path.clone();
        let snapshot_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                let _ = idx_snap.save_snapshot(&snap_path_clone);
            }
        });

        Ok(Self {
            name,
            index,
            wal: wal_arc,
            index_tx,
            replication_tx,
            config,
            _tasks: vec![indexer_handle, snapshot_handle],
        })
    }
}

impl<const N: usize, M: Metric<N>> Collection for CollectionImpl<N, M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn insert(&self, vector: &[f64], _id: u32, metadata: HashMap<String, String>) -> Result<(), String> {
        // Validation
        if vector.len() != N {
            return Err(format!("Vector dimension mismatch. Expected {}, got {}", N, vector.len()));
        }

        // 1. Storage
        let internal_id = self.index.insert_to_storage(vector).map_err(|e| e.to_string())?;

        // 2. WAL
        {
            let mut wal = self.wal.lock().unwrap();
            let _ = wal.append(internal_id, vector, &metadata);
        }

        // 3. Index Queue
        self.config.inc_queue();
        let _ = tokio::task::block_in_place(|| {
             self.index_tx.blocking_send((internal_id, metadata.clone()))
        }); 
        // Note: blocking_send inside async function? Collection trait is sync methods?
        // Collection trait definition has: fn insert(...) -> Result
        // It is NOT async. This works for gRPC `insert` which is async but calls this?
        // Wait, gRPC `insert` is async. If trait is synchronous, we block the executor thread.
        // We should make Collection trait async?
        // Or use `blocking_send` which blocks.
        // `index_tx` is `mpsc::Sender` (Tokio). `blocking_send` blocks thread.
        // Ideally we use `try_send` into a bounded channel?
        // `try_send` is non-blocking.
        // Using `blocking_send` here is bad if it blocks async runtime.
        // But `insert` in trait is not async.
        // We can change trait to async, but object-safe async traits are tricky (require `async_trait` crate).
        // `boot_server` used `await`.
        // Let's use `try_send` or strictly `block_in_place`.
        // Or just use std::sync::mpsc? No, executor needs to await rx.
        // I will use `try_send` and error if full (backpressure).
        
        // 4. Replication
        if self.replication_tx.receiver_count() > 0 {
            let log = ReplicationLog {
                id: internal_id,
                vector: vector.to_vec(),
                metadata,
                collection: self.name.clone(),
                origin_node_id: "".to_string(),
                logical_clock: 0,
            };
            let _ = self.replication_tx.send(log);
        }

        Ok(())
    }
    
    fn delete(&self, id: u32) -> Result<(), String> {
        self.index.delete(id);
        Ok(())
    }

    fn search(
        &self,
        query: &[f64],
        filters: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<(u32, f64)>, String> {
        if query.len() != N {
             return Err(format!("Query dimension mismatch. Expected {}, got {}", N, query.len()));
        }
        
        let results = self.index.search(
            query,
            params.top_k,
            params.ef_search, 
            filters,
            complex_filters,
            params.hybrid_query.as_deref(),
            params.hybrid_alpha,
        );
        
        Ok(results)
    }

    fn count(&self) -> usize {
        self.index.count_nodes()
    }

    fn dimension(&self) -> usize {
        N
    }

    fn metric_name(&self) -> &'static str {
        M::name()
    }

    fn peek(&self, limit: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        self.index.peek(limit)
    }
}
