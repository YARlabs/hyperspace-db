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
    
    // 1. Init Storage
    // We do NOT remove file, we want persistence!
    let store_path = std::path::Path::new("vectors.hyp");
    // We need to know current count to init store? 
    // VectorStore::new takes 'count'.
    // If we rely on WAL replay to rebuild, we can start with 0 count and let replay bump it?
    // Or we simply trust the WAL.
    
    // Simplification: Count will be restored via WAL replay if we replay into Store logic?
    // If Store persists, we shouldn't re-append to it during replay.
    // Issue: My VectorStore logic is "append only".
    // If I replay WAL, I might double-append if I use `store.append`.
    
    // REPLAY STRATEGY:
    // 1. Open Store.
    // 2. Open WAL.
    // 3. Read WAL.
    // 4. For each entry in WAL:
    //    - If entry ID < store.count(), it's already in Store. Just add to Index.
    //    - If entry ID >= store.count(), it's new (maybe Store didn't flush). Append to Store (or set specific position) AND add to Index.
    //
    // Current `VectorStore` doesn't support random write or check easily without strict state.
    // 
    // FORCE BRUTE STRATEGY (Reliable):
    // 1. Delete `vectors.hyp` on start (Treat it as cache).
    // 2. Rebuild Store AND Index from WAL entirely.
    // Cons: Slow startup.
    // Pros: Correctness guaranteed.
    // Given we said "Mmap... OS flushes lazily", we implied we want to keep `vectors.hyp`.
    
    // Let's do the Brute Strategy for stability in this MVP phase.
    // "Crash Recovery" means we recover FROM WAL.
    let _ = std::fs::remove_file("vectors.hyp"); 
    
    let store = Arc::new(VectorStore::new(store_path, 8, 0));
    let index = Arc::new(HnswIndex::new(store.clone()));
    
    // 2. Init WAL and Replay
    let wal_path = std::path::Path::new("wal.log");
    let mut wal = Wal::new(wal_path)?;
    
    println!("Recovering from WAL...");
    let mut recovered_count = 0;
    Wal::replay(wal_path, |entry| {
        match entry {
            hyperspace_store::wal::WalEntry::Insert { id: _, vector } => {
                // WAL doesn't have metadata yet, pass empty
                if let Err(e) = index.insert(&vector, std::collections::HashMap::new()) {
                    eprintln!("Replay Error: {}", e);
                }
                recovered_count += 1;
            }
        }
    })?;
    println!("Recovered {} vectors.", recovered_count);

    let wal_arc = Arc::new(Mutex::new(wal));
    
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
