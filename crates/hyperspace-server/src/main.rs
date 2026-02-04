use clap::Parser;
// Remove direct index usage, we go through manager
// use hyperspace_index::HnswIndex; 

mod collection;
mod manager;
mod http_server;
use manager::CollectionManager;

use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{
    ConfigUpdate, DeleteRequest, DeleteResponse, InsertRequest, InsertResponse, MonitorRequest,
    SearchRequest, SearchResponse, SearchResult, SystemStats,
    CreateCollectionRequest, DeleteCollectionRequest, ListCollectionsResponse, CollectionStatsRequest, CollectionStatsResponse, 
};
use hyperspace_proto::hyperspace::{Empty, ReplicationLog};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{service::Interceptor, transport::Server, Request, Response, Status};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on (gRPC)
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// HTTP Dashboard Port
    #[arg(long, default_value = "50050")]
    http_port: u16,

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

pub struct HyperspaceService {
    manager: Arc<CollectionManager>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    role: String,
}

#[tonic::async_trait]
impl Database for HyperspaceService {
    // --- Collection Management ---
    
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let req = request.into_inner();
        if req.name.is_empty() {
             return Err(Status::invalid_argument("Collection name cannot be empty"));
        }
        
        // Map string metric to internal
         // TODO: The manager takes string metric.
        match self.manager.create_collection(&req.name, req.dimension, &req.metric).await {
            Ok(_) => Ok(Response::new(hyperspace_proto::hyperspace::StatusResponse {
                status: format!("Collection '{}' created.", req.name),
            })),
            Err(e) => Err(Status::already_exists(e)),
        }
    }

     async fn delete_collection(
        &self,
        request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let req = request.into_inner();
        match self.manager.delete_collection(&req.name) {
             Ok(_) => Ok(Response::new(hyperspace_proto::hyperspace::StatusResponse {
                status: format!("Collection '{}' deleted.", req.name),
            })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn list_collections(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        let list = self.manager.list();
        Ok(Response::new(ListCollectionsResponse { collections: list }))
    }

    async fn get_collection_stats(
        &self,
        request: Request<CollectionStatsRequest>,
    ) -> Result<Response<CollectionStatsResponse>, Status> {
        let req = request.into_inner();
        if let Some(col) = self.manager.get(&req.name) {
             // We need to extend Collection trait to return dim/metric?
             // collection.name() is available.
             // But dim/metric are generic in implementation, not exposed via trait yet.
             // For now return dummy or count.
             Ok(Response::new(CollectionStatsResponse {
                 count: col.count() as u64,
                 dimension: 0, // TODO: Expose from trait
                 metric: "unknown".into(),
             }))
        } else {
            Err(Status::not_found("Collection not found"))
        }
    }

    // --- Data Plane ---

    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        if self.role == "follower" {
            return Err(Status::permission_denied("Followers are read-only"));
        }
        let req = request.into_inner();
        
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if let Some(col) = self.manager.get(&col_name) {
            let meta: std::collections::HashMap<String, String> = req.metadata.into_iter().collect();
            // id is u32 in proto.
            if let Err(e) = col.insert(&req.vector, req.id, meta) {
                return Err(Status::internal(e));
            }
            Ok(Response::new(InsertResponse { success: true }))
        } else {
            Err(Status::not_found(format!("Collection '{}' not found", col_name)))
        }
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
         let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if let Some(col) = self.manager.get(&col_name) {
             if let Err(e) = col.delete(req.id) {
                return Err(Status::internal(e));
             }
             Ok(Response::new(DeleteResponse { success: true }))
        } else {
             Err(Status::not_found(format!("Collection '{}' not found", col_name)))
        }
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if let Some(col) = self.manager.get(&col_name) {
             let legacy_filter = req.filter.into_iter().collect();
             let mut complex_filters = Vec::new();
             for f in req.filters {
                 if let Some(cond) = f.condition {
                     match cond {
                        hyperspace_proto::hyperspace::filter::Condition::Match(m) => {
                             complex_filters.push(hyperspace_core::FilterExpr::Match {
                                 key: m.key,
                                 value: m.value,
                             });
                        }
                        hyperspace_proto::hyperspace::filter::Condition::Range(r) => {
                             complex_filters.push(hyperspace_core::FilterExpr::Range {
                                 key: r.key,
                                 gte: r.gte,
                                 lte: r.lte,
                             });
                        }
                     }
                 }
             }
             
             // Search Params
             let params = hyperspace_core::SearchParams {
                 top_k: req.top_k as usize,
                 ef_search: 100, // TODO: Load from config/request?
                 hybrid_query: req.hybrid_query,
                 hybrid_alpha: req.hybrid_alpha,
             };

             match col.search(&req.vector, &legacy_filter, &complex_filters, &params) {
                 Ok(res) => {
                     let output = res
                        .into_iter()
                        .map(|(id, dist)| SearchResult { id, distance: dist })
                        .collect();
                     Ok(Response::new(SearchResponse { results: output }))
                 }
                 Err(e) => Err(Status::internal(e)),
             }

        } else {
             Err(Status::not_found(format!("Collection '{}' not found", col_name)))
        }
    }

    type MonitorStream = ReceiverStream<Result<SystemStats, Status>>;

    async fn monitor(
        &self,
        _request: Request<MonitorRequest>,
    ) -> Result<Response<Self::MonitorStream>, Status> {
        // For MVP monitor just sums up everything? Or just monitors default?
        // Let's monitor global stats.
        let (tx, rx) = mpsc::channel(4);
        let manager = self.manager.clone();

        tokio::spawn(async move {
            loop {
                // Aggregate stats
                let collections = manager.list();
                let mut total_count = 0;
                
                for name in &collections {
                    if let Some(c) = manager.get(name) {
                        total_count += c.count();
                    }
                }

                let stats = SystemStats {
                    total_collections: collections.len() as u64,
                    total_vectors: total_count as u64,
                    total_memory_mb: 0.0, // TODO
                    qps: 0.0, // TODO
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
         // Snapshot all?
         // Individual collections handle their own snapshots via background tasks currently.
         // We can force trigger?
         // The trait doesn't have trigger_snapshot.
         Ok(Response::new(
             hyperspace_proto::hyperspace::StatusResponse {
                 status: "Snapshots are handled automatically by background tasks.".into(),
             },
         ))
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
        // TODO: Update config for specific collection
        // ConfigUpdate now has `collection` field.
         let req = request.into_inner();
         let col_name = if req.collection.is_empty() {
             "default".to_string()
         } else {
             req.collection
         };
         
         if self.manager.get(&col_name).is_none() {
             return Err(Status::not_found(format!("Collection '{}' not found", col_name)));
         }
         // Not implemented on trait yet.
         Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse { status: "Dynamic config not yet implemented for collections".into() },
        ))
    }
}

async fn start_server(
    args: Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", args.port).parse()?;
    
    // Setup Manager
    let data_dir = std::path::PathBuf::from("data");
    let (replication_tx, _) = broadcast::channel(1024);
    
    let manager = Arc::new(CollectionManager::new(data_dir, replication_tx.clone()));
    
    // Load existing
    println!("Loading collections...");
    manager.load_existing().await?;
    
    // Create default if not exists?
    // Create default if not exists
    if manager.get("default").is_none() {
         // Use env vars for default
         let dim_str = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
         let dim: u32 = dim_str.parse().unwrap_or(1024);
         
         // Support HS_METRIC (new) and HS_DISTANCE_METRIC (legacy)
         let metric_str = std::env::var("HS_METRIC")
             .or_else(|_| std::env::var("HS_DISTANCE_METRIC"))
             .unwrap_or("poincare".to_string())
             .to_lowercase();
         
         println!("🚀 Booting HyperspaceDB | Dim: {} | Metric: {}", dim, metric_str);

         println!("Creating 'default' collection...");
         if let Err(e) = manager.create_collection("default", dim, &metric_str).await {
             eprintln!("Failed to create default collection: {}", e);
         }
    }

    // Follower Logic
    if args.role == "follower" {
         if let Some(leader) = args.leader.clone() {
            println!("🚀 Starting as FOLLOWER of: {}", leader);
            let manager_weak = Arc::downgrade(&manager);
            
            tokio::spawn(async move {
                 use hyperspace_proto::hyperspace::database_client::DatabaseClient;
                 loop {
                    println!("Connecting to leader {}...", leader);
                     match DatabaseClient::connect(leader.clone()).await {
                         Ok(mut client) => {
                             println!("Connected! Requesting replication stream...");
                             match client.replicate(Empty {}).await {
                                  Ok(resp) => {
                                      let mut stream = resp.into_inner();
                                      while let Ok(Some(log)) = stream.message().await {
                                          if let Some(mgr) = manager_weak.upgrade() {
                                              // Log contains collection name
                                              let col_name = if log.collection.is_empty() { "default" } else { &log.collection };
                                              if let Some(col) = mgr.get(col_name) {
                                                   let meta: std::collections::HashMap<String, String> = log.metadata.into_iter().collect();
                                                   if let Err(e) = col.insert(&log.vector, log.id, meta) {
                                                       eprintln!("Rep Error: {}", e);
                                                   }
                                              } else {
                                                  // Create collection if missing?
                                                  // For now just error.
                                                  eprintln!("Replication received for unknown collection: {}", col_name);
                                              }
                                          } else {
                                              break;
                                          }
                                      }
                                  }
                                  Err(e) => eprintln!("Failed: {}", e),
                             }
                         }
                         Err(e) => eprintln!("Conn failed: {}", e),
                     }
                      tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                 }
            });
         }
    } else {
        println!("🚀 Starting as LEADER");
    }

    // Start HTTP Dashboard
    let http_mgr = manager.clone();
    let http_port = args.http_port;
    tokio::spawn(async move {
        if let Err(e) = http_server::start_http_server(http_mgr, http_port).await {
            eprintln!("HTTP Server panicked: {}", e);
        }
    });

    let service = HyperspaceService {
        manager,
        replication_tx,
        role: args.role,
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

    Server::builder()
        .add_service(DatabaseServer::with_interceptor(service, interceptor))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            println!("\n🛑 Received Ctrl+C. Initiating graceful shutdown...");
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let args = Args::parse();
    start_server(args).await
}
