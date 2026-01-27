use clap::Parser;
use hyperspace_core::vector::{BinaryHyperVector, HyperVector, QuantizedHyperVector};
use hyperspace_core::QuantizationMode;
use hyperspace_core::{GlobalConfig, Metric, PoincareMetric};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{
    ConfigUpdate, DeleteRequest, DeleteResponse, InsertRequest, InsertResponse, MonitorRequest,
    SearchRequest, SearchResponse, SearchResult, SystemStats,
};
use hyperspace_proto::hyperspace::{Empty, ReplicationLog};
use hyperspace_store::wal::Wal;
use hyperspace_store::VectorStore;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{service::Interceptor, transport::Server, Request, Response, Status};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// Role: leader or follower
    #[arg(long, default_value = "leader")]
    role: String,

    /// Leader address (if follower)
    #[arg(long)]
    leader: Option<String>,
}

#[derive(Clone)]
struct AuthInterceptor {
    expected_hash: Option<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(expected) = &self.expected_hash {
            match request.metadata().get("x-api-key") {
                Some(t) => {
                    // Hash the received token
                    if let Ok(token_str) = t.to_str() {
                        let mut hasher = Sha256::new();
                        hasher.update(token_str.as_bytes());
                        let result = hasher.finalize();
                        let request_hash = hex::encode(result);

                        // Constant-time comparison is better for security, but string eq is fine for MVP
                        if request_hash == *expected {
                            return Ok(request);
                        }
                    }
                    Err(Status::unauthenticated("Invalid API Key"))
                }
                None => Err(Status::unauthenticated("Missing x-api-key header")),
            }
        } else {
            Ok(request)
        }
    }
}

#[derive(Debug)]
pub struct HyperspaceService<const N: usize, M: Metric<N>> {
    index: Arc<HnswIndex<N, M>>,
    #[allow(dead_code)]
    store: Arc<VectorStore>,
    wal: Arc<Mutex<Wal>>,
    index_tx: mpsc::Sender<(u32, std::collections::HashMap<String, String>)>,
    config: Arc<GlobalConfig>,
    // Replication (Leader broadcasts to followers)
    replication_tx: broadcast::Sender<ReplicationLog>,
    role: String,
}

#[tonic::async_trait]
impl<const N: usize, M: Metric<N>> Database for HyperspaceService<N, M> {
    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        if self.role == "follower" {
            return Err(Status::permission_denied("Followers are read-only"));
        }
        let req = request.into_inner();
        let vector = req.vector;
        let metadata_map = req.metadata;

        let mut meta = std::collections::HashMap::new();
        for (k, v) in metadata_map {
            meta.insert(k, v);
        }

        // 1. Persistence (Storage + WAL)
        // Check dimension first
        if vector.len() != N {
            return Err(Status::invalid_argument(format!(
                "Vector dimension mismatch: expected {}, got {}",
                N,
                vector.len()
            )));
        }

        // Write to storage and get ID
        let id = self
            .index
            .insert_to_storage(&vector)
            .map_err(Status::internal)?;

        // Write to WAL
        {
            let mut wal = self.wal.lock().unwrap();
            let _ = wal.append(id, &vector, &meta);
        }

        // 2. Async Indexing (Track queue size)
        self.config.inc_queue();
        if self.index_tx.send((id, meta.clone())).await.is_err() {
            self.config.dec_queue(); // Rollback on error
            return Err(Status::internal("Indexer channel closed"));
        }

        // 3. Replicate (Best Effort)
        if self.replication_tx.receiver_count() > 0 {
            let rep_log = ReplicationLog {
                id,
                vector: vector.clone(),
                metadata: meta,
            };
            let _ = self.replication_tx.send(rep_log);
        }

        Ok(Response::new(InsertResponse { success: true }))
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        self.index.delete(req.id);
        Ok(Response::new(DeleteResponse { success: true }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let legacy_filter = req.filter;

        if req.vector.len() != N {
            return Err(Status::invalid_argument(format!(
                "Vector dimension mismatch: expected {}, got {}",
                N,
                req.vector.len()
            )));
        }

        let mut complex_filters = Vec::new();
        for f in req.filters {
            if let Some(cond) = f.condition {
                match cond {
                    hyperspace_proto::hyperspace::filter::Condition::Match(m) => {
                        complex_filters.push(hyperspace_index::FilterExpr::Match {
                            key: m.key,
                            value: m.value,
                        });
                    }
                    hyperspace_proto::hyperspace::filter::Condition::Range(r) => {
                        complex_filters.push(hyperspace_index::FilterExpr::Range {
                            key: r.key,
                            gte: r.gte,
                            lte: r.lte,
                        });
                    }
                }
            }
        }

        // Use dynamic ef_search from config
        let ef_search = self.config.get_ef_search();

        let hybrid_query = req.hybrid_query.as_deref();
        // Convert input slice to array for Metric/HNSW
        // HNSW search takes &[f64]. Inside it converts.

        let res = self.index.search(
            &req.vector,
            req.top_k as usize,
            ef_search,
            &legacy_filter,
            &complex_filters,
            hybrid_query,
            req.hybrid_alpha,
        );

        let output = res
            .into_iter()
            .map(|(id, dist)| SearchResult { id, distance: dist })
            .collect();

        Ok(Response::new(SearchResponse { results: output }))
    }

    type MonitorStream = ReceiverStream<Result<SystemStats, Status>>;

    async fn monitor(
        &self,
        _request: Request<MonitorRequest>,
    ) -> Result<Response<Self::MonitorStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let index = self.index.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                let count = index.count_nodes() as u64;
                let soft_deleted = index.count_deleted() as u64;

                let (segments, bytes) = index.storage_stats();

                let raw_size_bytes = (count as f64) * (N as f64) * 8.0;
                let actual_size_bytes = bytes as f64;

                let compression = if actual_size_bytes > 0.0 {
                    raw_size_bytes / actual_size_bytes
                } else {
                    0.0
                };

                let stats = SystemStats {
                    indexed_vectors: count,
                    soft_deleted,
                    raw_data_size_mb: raw_size_bytes / 1024.0 / 1024.0,
                    actual_storage_mb: actual_size_bytes / 1024.0 / 1024.0,
                    compression_ratio: compression,
                    active_segments: segments as u32,
                    qps: 0.0,
                    ram_usage_mb: 0.0,
                    ef_search: config.get_ef_search() as u32,
                    ef_construction: config.get_ef_construction() as u32,
                    index_queue_size: config.get_queue_size(),
                };

                if tx.send(Ok(stats)).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ReplicateStream = ReceiverStream<Result<ReplicationLog, Status>>;

    async fn replicate(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ReplicateStream>, Status> {
        if self.role == "follower" {
            return Err(Status::failed_precondition(
                "I am a follower, cannot replicate from me",
            ));
        }

        let mut rx = self.replication_tx.subscribe();
        let (tx, out_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Ok(log) = rx.recv().await {
                if tx.send(Ok(log)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(out_rx)))
    }

    async fn trigger_snapshot(
        &self,
        _request: Request<hyperspace_proto::hyperspace::Empty>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        match self.index.save_snapshot(std::path::Path::new("index.snap")) {
            Ok(_) => Ok(Response::new(
                hyperspace_proto::hyperspace::StatusResponse {
                    status: "Snapshot saved".into(),
                },
            )),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn trigger_vacuum(
        &self,
        _request: Request<hyperspace_proto::hyperspace::Empty>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse {
                status: "Vacuum started (simulated)".into(),
            },
        ))
    }

    async fn configure(
        &self,
        request: Request<ConfigUpdate>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let req = request.into_inner();
        let mut changes = Vec::new();

        if let Some(ef_s) = req.ef_search {
            self.config.set_ef_search(ef_s as usize);
            changes.push(format!("ef_search={}", ef_s));
        }

        if let Some(ef_c) = req.ef_construction {
            self.config.set_ef_construction(ef_c as usize);
            changes.push(format!("ef_construction={}", ef_c));
        }

        let status = if changes.is_empty() {
            "No changes applied".to_string()
        } else {
            format!("Config updated: {}", changes.join(", "))
        };

        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse { status },
        ))
    }
}

async fn boot_server<const N: usize, M: Metric<N>>(
    args: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", args.port).parse()?;
    let wal_path = std::path::Path::new("wal.log");

    // Directory for segmented storage
    let data_dir = std::path::Path::new("data");
    let snap_path = std::path::Path::new("index.snap");

    // Global Runtime Config (loaded from env in main, but let's refresh or pass it?
    // GlobalConfig::new reads defaults? No, it's just atomic counters.
    // IndexConfig (construction vars) is also needed.
    // Env vars: HS_HNSW_EF_CONSTRUCT, HS_HNSW_M, HS_QUANTIZATION_LEVEL
    // We should read them here or pass them.
    // Since GlobalConfig is 'Runtime' config (ef_search), we create it here.

    // Default values for HNSW construction
    let ef_cons_env = std::env::var("HS_HNSW_EF_CONSTRUCT")
        .unwrap_or("100".to_string())
        .parse()
        .unwrap_or(100);
    let ef_search_env = std::env::var("HS_HNSW_EF_SEARCH")
        .unwrap_or("10".to_string())
        .parse()
        .unwrap_or(10);

    // Note: ef_construction is currently in GlobalConfig separately?
    // HnswIndex has GlobalConfig. And select_neighbors uses config.
    // We should initialze GlobalConfig with env vars.
    let config = Arc::new(GlobalConfig::new());
    config.set_ef_construction(ef_cons_env);
    config.set_ef_search(ef_search_env);

    // Quantization
    let quant_env = std::env::var("HS_QUANTIZATION_LEVEL").unwrap_or("scalar".to_string());
    let mode = match quant_env.as_str() {
        "binary" => QuantizationMode::Binary,
        "none" => QuantizationMode::None,
        _ => QuantizationMode::ScalarI8,
    };
    println!("Init Server [N={}] Mode={:?}", N, mode);

    let (store, index, recovered_count) = if snap_path.exists() {
        println!("Found snapshot. Loading...");

        let element_size = match mode {
            QuantizationMode::ScalarI8 => QuantizedHyperVector::<N>::SIZE,
            QuantizationMode::Binary => BinaryHyperVector::<N>::SIZE,
            QuantizationMode::None => HyperVector::<N>::SIZE,
        };

        let store = Arc::new(VectorStore::new(data_dir, element_size));
        match HnswIndex::<N, M>::load_snapshot(snap_path, store.clone(), mode, config.clone()) {
            Ok(idx) => {
                let count = idx.count_nodes();
                println!("Snapshot loaded. Nodes: {}", count);
                (store, Arc::new(idx), count)
            }
            Err(e) => {
                eprintln!("Failed to load snapshot: {}. Starting fresh.", e);
                if data_dir.exists() {
                    let _ = std::fs::remove_dir_all(data_dir);
                }
                let store = Arc::new(VectorStore::new(data_dir, element_size));
                (
                    store.clone(),
                    Arc::new(HnswIndex::new(store, mode, config.clone())),
                    0,
                )
            }
        }
    } else {
        println!("No snapshot found. Starting fresh.");
        if data_dir.exists() {
            let _ = std::fs::remove_dir_all(data_dir);
        }

        let element_size = match mode {
            QuantizationMode::ScalarI8 => QuantizedHyperVector::<N>::SIZE,
            QuantizationMode::Binary => BinaryHyperVector::<N>::SIZE,
            QuantizationMode::None => HyperVector::<N>::SIZE,
        };

        let store = Arc::new(VectorStore::new(data_dir, element_size));
        (
            store.clone(),
            Arc::new(HnswIndex::new(store, mode, config.clone())),
            0,
        )
    };

    // 2. Init WAL and Replay
    let wal = Wal::new(wal_path)?;

    println!(
        "Recovering from WAL (Skipping first {})...",
        recovered_count
    );
    let mut replayed = 0;
    Wal::replay(wal_path, |entry| match entry {
        hyperspace_store::wal::WalEntry::Insert {
            id,
            vector,
            metadata,
        } => {
            if (id as usize) >= recovered_count {
                if let Err(e) = index.insert(&vector, metadata) {
                    eprintln!("Replay Error: {}", e);
                }
                replayed += 1;
            }
        }
    })?;
    println!("Replayed {} new vectors from WAL.", replayed);

    let wal_arc = Arc::new(Mutex::new(wal));

    // Channel for Leader -> Followers
    let (replication_tx, _) = broadcast::channel(1024);

    // FOLLOWER LOGIC
    if args.role == "follower" {
        if let Some(leader) = args.leader.clone() {
            println!("🚀 Starting as FOLLOWER of: {}", leader);
            let index_weak = Arc::downgrade(&index);

            // Spawn replication task
            tokio::spawn(async move {
                use hyperspace_proto::hyperspace::database_client::DatabaseClient;
                // Retry loop
                loop {
                    println!("Connecting to leader {}...", leader);
                    match DatabaseClient::connect(leader.clone()).await {
                        Ok(mut client) => {
                            println!("Connected! Requesting replication stream...");
                            match client.replicate(Empty {}).await {
                                Ok(resp) => {
                                    let mut stream = resp.into_inner();
                                    while let Ok(Some(log)) = stream.message().await {
                                        if let Some(idx) = index_weak.upgrade() {
                                            if let Err(e) = idx.insert(&log.vector, log.metadata) {
                                                eprintln!("Replication Error: {}", e);
                                            }
                                        } else {
                                            break; // shutting down
                                        }
                                    }
                                    println!("Replication stream ended.");
                                }
                                Err(e) => eprintln!("Failed to start replication: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to connect to leader: {}", e),
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        } else {
            eprintln!("Error: --leader is required for follower role");
            std::process::exit(1);
        }
    } else {
        println!("🚀 Starting as LEADER");
    }

    // 3. Spawn Snapshot Task
    let index_clone = index.clone();
    let snapshot_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            println!("Saving snapshot...");
            if let Err(e) = index_clone.save_snapshot(std::path::Path::new("index.snap")) {
                eprintln!("Snapshot save failed: {}", e);
            } else {
                println!("Snapshot saved.");
            }
        }
    });

    // 4. Spawn Indexer Worker (Async Write)
    let (index_tx, mut index_rx) = mpsc::channel(1000);
    let index_worker = index.clone();
    let config_worker = config.clone();

    let indexer_handle = tokio::spawn(async move {
        println!("⚙️ Background Indexer started");
        while let Some((id, meta)) = index_rx.recv().await {
            let idx = index_worker.clone();
            let cfg = config_worker.clone();
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = idx.index_node(id, meta) {
                    eprintln!("Indexer error for ID {}: {}", id, e);
                }
                cfg.dec_queue();
            })
            .await;
        }
        println!("⚙️ Indexer shutting down...");
    });

    let service = HyperspaceService {
        index: index.clone(),
        store,
        wal: wal_arc,
        index_tx: index_tx.clone(),
        config: config.clone(),
        replication_tx: replication_tx.clone(),
        role: args.role.clone(),
    };

    println!("HyperspaceDB listening on {}", addr);

    // Setup Auth
    let api_key = std::env::var("HYPERSPACE_API_KEY").ok();
    let interceptor = if let Some(key) = api_key {
        println!("🔒 API Auth Enabled");
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        AuthInterceptor {
            expected_hash: Some(hash),
        }
    } else {
        println!("⚠️ API Auth Disabled");
        AuthInterceptor {
            expected_hash: None,
        }
    };

    // Graceful Shutdown Handler
    let server = Server::builder()
        .add_service(DatabaseServer::with_interceptor(service, interceptor))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            println!("\n🛑 Received Ctrl+C. Initiating graceful shutdown...");
        });

    server.await?;

    // Shutdown sequence
    println!(
        "Draining index queue ({} pending)...",
        config.get_queue_size()
    );
    drop(index_tx);
    let _ = indexer_handle.await;

    println!("Saving final snapshot...");
    if let Err(e) = index.save_snapshot(snap_path) {
        eprintln!("Final snapshot failed: {}", e);
    } else {
        println!("Final snapshot saved.");
    }

    snapshot_handle.abort();
    println!("✅ HyperspaceDB shutdown complete.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Reading .env

    let args = Args::parse();

    // Read Config
    let dim_str = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
    let dim: usize = dim_str.parse().expect("HS_DIMENSION must be a number");
    let metric_str = std::env::var("HS_DISTANCE_METRIC").unwrap_or("poincare".to_string());

    println!(
        "DISPATCHER: Booting kernel with N={}, Metric={}",
        dim, metric_str
    );

    // Dispatcher
    match (dim, metric_str.as_str()) {
        (1024, "poincare") => boot_server::<1024, PoincareMetric>(args).await?,
        (768, "poincare") => boot_server::<768, PoincareMetric>(args).await?,
        (1536, "poincare") => boot_server::<1536, PoincareMetric>(args).await?,
        (8, "poincare") => boot_server::<8, PoincareMetric>(args).await?,

        // Add more combinations here (e.g. Cosine)
        // (1024, "cosine") => boot_server::<1024, CosineMetric>(args).await?,

        _ => panic!("Unsupported combination: Dimension {} with Metric {}. Please update main.rs Dispatcher.", dim, metric_str),
    }

    Ok(())
}
