use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{InsertRequest, InsertResponse, SearchRequest, SearchResponse, SearchResult, MonitorRequest, SystemStats};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct HyperspaceService {
    index: Arc<HnswIndex>,
}

#[tonic::async_trait]
impl Database for HyperspaceService {
    async fn insert(&self, request: Request<InsertRequest>) -> Result<Response<InsertResponse>, Status> {
        // Simple stub: just print. 
        // Real logic: append to store, then insert to index.
        let req = request.into_inner();
        println!("Received insert request for ID: {}", req.id);
        
        // In this MVP we can't easily write to Store/Index safely from async context 
        // without more wrapping (Store needs to be mutable or internal mutability).
        // VectorStore currently uses file mmap, appending is separate logic.
        // For now, assume this is a read-only demo after startup population.
        
        Ok(Response::new(InsertResponse { success: true }))
    }

    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        // Convert proto vector to slice
        let res = self.index.search(&req.vector, req.top_k as usize, 100);
        
        // Map results
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
                // Mock memory usage for MVP
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
    
    // 1. Init Storage (create file if not exists)
    // IMPORTANT: In a real run, we might want to clean up vectors.hyp to start fresh for this test
    let _ = std::fs::remove_file("vectors.hyp");
    let store = Arc::new(VectorStore::new(std::path::Path::new("vectors.hyp"), 8, 0));
    
    // 2. Init Index
    let index = Arc::new(HnswIndex::new(store.clone()));
    
    // 3. Populate with dummy data 
    println!("Populating index with 100 vectors...");
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for _ in 0..100 {
         let mut vec = [0.0; 8];
         // Generate random vector inside Poincaré ball (norm < 1)
         let mut norm_sq = 0.0;
         for v in &mut vec {
             *v = rng.gen_range(-0.5..0.5) / 4.0; // Small coords to ensure norm < 1
             norm_sq += *v * *v;
         }
         
         // Insert
         index.insert(&vec).unwrap();
    }
    println!("Population done! Starting server...");
    
    let service = HyperspaceService { index };

    println!("HyperspaceDB listening on {}", addr);

    Server::builder()
        .add_service(DatabaseServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
