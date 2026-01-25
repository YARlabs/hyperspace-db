use hyperspace_proto::hyperspace::database_client::DatabaseClient;
use hyperspace_proto::hyperspace::{InsertRequest, SearchRequest, Empty, ConfigUpdate};
use rand::Rng;
use std::time::Instant;
use tonic::transport::Channel;

const TOTAL_VECTORS: usize = 1_000_000;
const SEARCH_QUERIES: usize = 10_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting HyperspaceDB Stress Test");
    println!("Connecting to localhost:50051...");
    
    // Connect to server
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    let mut client = DatabaseClient::new(channel);
    
    // 1. Configure for speed (High Throughput)
    println!("🔧 Configuring DB for ingestion (ef_construction=50)...");
    let _ = client.configure(ConfigUpdate {
        ef_construction: Some(50),
        ef_search: None,
    }).await?;

    // 2. Insert Vectors
    println!("📦 Inserting {} vectors...", TOTAL_VECTORS);
    let mut rng = rand::thread_rng();
    
    let start_insert = Instant::now();
    for i in 0..TOTAL_VECTORS {
        // Generate random vector
        let mut vector: Vec<f64> = (0..8).map(|_| rng.gen_range(-1.0..1.0)).collect();
        
        // Normalize to Unit Ball (Poincaré Model Requirement)
        let norm_sq: f64 = vector.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm >= 0.99 {
            let scale = 0.99 / norm;
            for x in &mut vector { *x *= scale; }
        }
        
        let req = InsertRequest {
            vector,
            id: i as u32,
            metadata: std::collections::HashMap::new(),
        };
        
        client.insert(req).await?;
        
        if (i + 1) % 1000 == 0 {
            print!(".");
            use std::io::Write;
            std::io::stdout().flush()?;
        }
    }
    let duration_insert = start_insert.elapsed();
    println!("\n✅ Insert done in {:.2?}.", duration_insert);
    println!("   Throughput: {:.2} vectors/sec", TOTAL_VECTORS as f64 / duration_insert.as_secs_f64());

    // 3. Trigger Snapshot (Flush)
    println!("💾 Triggering Snapshot...");
    let _ = client.trigger_snapshot(Empty {}).await?;
    
    // 4. Search Benchmark
    println!("🔍 Running {} Search Queries (top_k=10)...", SEARCH_QUERIES);
    // Configure for accuracy (optional)
    let _ = client.configure(ConfigUpdate {
        ef_search: Some(100),
        ef_construction: None,
    }).await?;

    let start_search = Instant::now();
    for _ in 0..SEARCH_QUERIES {
        let mut query_vec: Vec<f64> = (0..8).map(|_| rng.gen_range(-1.0..1.0)).collect();
        // Normalize
        let norm_sq: f64 = query_vec.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm >= 0.99 {
            let scale = 0.99 / norm;
            for x in &mut query_vec { *x *= scale; }
        }

        let req = SearchRequest {
            vector: query_vec,
            top_k: 10,
            filter: std::collections::HashMap::new(),
            filters: Vec::new(),
        };
        client.search(req).await?;
    }
    let duration_search = start_search.elapsed();
    
    println!("✅ Search done in {:.2?}.", duration_search);
    println!("   QPS: {:.2} queries/sec", SEARCH_QUERIES as f64 / duration_search.as_secs_f64());
    println!("   Avg Latency: {:.2} ms", (duration_search.as_secs_f64() * 1000.0) / SEARCH_QUERIES as f64);

    Ok(())
}
