use hyperspace_sdk::Client;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect
    let mut client = Client::connect("http://localhost:50051".to_string(), Some("I_LOVE_HYPERSPACEDB".to_string())).await?;
    println!("Connected to HyperspaceDB!");

    // 2. Create Collection
    let col_name = "sdk_test_collection";
    let _ = client.delete_collection(col_name.to_string()).await; // Cleanup
    client.create_collection(col_name.to_string(), 8, "l2".to_string()).await?;
    println!("Created collection: {}", col_name);

    // 3. Insert Vectors
    for i in 0..10 {
        let vector = vec![0.1 * (i as f64); 8];
        let mut meta = HashMap::new();
        meta.insert("category".to_string(), "test".to_string());
        
        client.insert(i, vector, meta, Some(col_name.to_string())).await?;
    }
    println!("Inserted 10 vectors.");

    // 4. Search
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Wait for indexing (async)
    let query = vec![0.1; 8];
    let results = client.search(query, 5, Some(col_name.to_string())).await?;
    
    println!("Search Results:");
    for res in results {
        println!("  ID: {}, Score: {:.4}", res.id, res.distance);
    }

    // 5. Cleanup
    client.delete_collection(col_name.to_string()).await?;
    println!("Deleted collection.");

    Ok(())
}
