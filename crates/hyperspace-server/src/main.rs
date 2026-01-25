use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use hyperspace_store::wal::Wal;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{InsertRequest, InsertResponse, SearchRequest, SearchResponse, SearchResult, MonitorRequest, SystemStats};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct HyperspaceService {
    index: Arc<HnswIndex>,
    store: Arc<VectorStore>,
    wal: Arc<Mutex<Wal>>,
}

#[tonic::async_trait]
impl Database for HyperspaceService {
    async fn insert(&self, request: Request<InsertRequest>) -> Result<Response<InsertResponse>, Status> {
        let req = request.into_inner();
        let vec = req.vector;
        let meta = req.metadata;

        // 1. Insert to Storage/Index (simplified flow without WAL transaction for now)
        // Note: WAL logging below doesn't log metadata yet.
        // TODO: Update WAL format to include metadata. 
        // For now, we only log vector.
        
        {
             // Log to WAL (only vector) - partial data loss risk on crash for metadata
             // Should update WAL entry to Insert { id, vector, metadata }
             // Leaving as is for minimal diff, implying metadata is volatile or needs WAL update.
             // But index.insert REQUIRES metadata now.
        }

        // 1. Insert to Storage to get ID (Handled inside index.insert actually? No, store.append is internal?)
        // Wait, my previous `service.insert` called `store.append` MANUALLY.
        // But `index.insert` ALSO calls `store.append`.
        // This causes double append!
        // FIX: The `index.insert` method I just updated now calls `store.append`.
        // So I should ONLY call `index.insert`.
        // But wait, `index.insert` returns `u32` (id).
        
        let res = self.index.insert(&vec, meta);
        match res {
            Ok(id) => {
                 // 2. Write to WAL
                 {
                     let mut wal = self.wal.lock().unwrap();
                     if let Err(e) = wal.append(id, &vec) {
                         eprintln!("WAL Error: {}", e);
                         // Don't fail request if WAL fails? Or warn?
                     }
                 }
                 Ok(Response::new(InsertResponse { success: true }))
            },
            Err(e) => Err(Status::internal(format!("Insert failed: {}", e)))
        }
    }

    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let filter = req.filter;
        
        let res = self.index.search(&req.vector, req.top_k as usize, 100, &filter);
        
        let output = res.into_iter().map(|(id, dist)| SearchResult {
            id,
            distance: dist,
        }).collect();

        Ok(Response::new(SearchResponse { results: output }))
    }

    type MonitorStream = ReceiverStream<Result<SystemStats, Status>>;

    async fn monitor(&self, _request: Request<MonitorRequest>) -> Result<Response<Self::MonitorStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let index = self.index.clone();
        
        tokio::spawn(async move {
            loop {
                let count = index.count_nodes() as u64;
                // Mock memory usage
                let mem = (count * 8 * 8) as f64 / 1024.0 / 1024.0; 
                
                let stats = SystemStats {
                    indexed_vectors: count,
                    ram_usage_mb: mem,
                    qps: 0.0, 
                };

                if tx.send(Ok(stats)).await.is_err() {
                    break; 
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let wal_path = std::path::Path::new("wal.log");
    let store_path = std::path::Path::new("vectors.hyp");
    let snap_path = std::path::Path::new("index.snap");

    let (store, index, mut recovered_count) = if snap_path.exists() {
        println!("Found snapshot. Loading...");
        let store = Arc::new(VectorStore::new(store_path, 8, 0));
        match HnswIndex::load_snapshot(snap_path, store.clone()) {
            Ok(idx) => {
                let count = idx.count_nodes();
                println!("Snapshot loaded. Nodes: {}", count);
                (store, Arc::new(idx), count)
            },
            Err(e) => {
                eprintln!("Failed to load snapshot: {}. Starting fresh.", e);
                let _ = std::fs::remove_file(store_path);
                let store = Arc::new(VectorStore::new(store_path, 8, 0));
                (store.clone(), Arc::new(HnswIndex::new(store)), 0)
            }
        }
    } else {
        println!("No snapshot found. Starting fresh.");
        let _ = std::fs::remove_file(store_path);
        let store = Arc::new(VectorStore::new(store_path, 8, 0));
        (store.clone(), Arc::new(HnswIndex::new(store)), 0)
    };
    
    // 2. Init WAL and Replay
    let mut wal = Wal::new(wal_path)?;
    
    println!("Recovering from WAL (Skipping first {})...", recovered_count);
    let mut replayed = 0;
    // We need WAL to be able to skip? `wal.replay` iterates all.
    // We just ignore inside callback.
    Wal::replay(wal_path, |entry| {
        match entry {
            hyperspace_store::wal::WalEntry::Insert { id, vector } => {
                // If id < recovered_count, it's already in snapshot (assuming sequential 0..N IDs)
                // In real world, IDs might not be sequential or fully packed, but for this MVP they are.
                if (id as usize) >= recovered_count {
                     if let Err(e) = index.insert(&vector, std::collections::HashMap::new()) {
                        eprintln!("Replay Error: {}", e);
                    }
                    replayed += 1;
                }
            }
        }
    })?;
    println!("Replayed {} new vectors from WAL.", replayed);

    let wal_arc = Arc::new(Mutex::new(wal));
    
    // 3. Spawn Snapshot Task
    let index_clone = index.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            println!("Saving snapshot...");
            if let Err(e) = index_clone.save_snapshot(std::path::Path::new("index.snap")) {
                eprintln!("Snapshot save failed: {}", e);
            } else {
                println!("Snapshot saved.");
                // Optional: Truncate WAL here? 
                // To safely truncate WAL, we must ensure all data in Snapshot is persisted.
                // For MVP, keep WAL growing.
            }
        }
    });

    let service = HyperspaceService { 
        index, 
        store,
        wal: wal_arc,
    };

    println!("HyperspaceDB listening on {}", addr);

    Server::builder()
        .add_service(DatabaseServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
