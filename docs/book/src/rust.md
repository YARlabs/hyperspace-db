# Rust SDK

For low-latency applications, connect directly using the Rust SDK.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hyperspace-sdk = { git = "https://github.com/yarlabs/hyperspace-db" }
tokio = { version = "1", features = ["full"] }
```

## Usage

```rust
use hyperspace_sdk::Client;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect (with optional API Key)
    let api_key = std::env::var("HYPERSPACE_API_KEY").ok();
    let mut client = Client::connect("http://127.0.0.1:50051".into(), api_key).await?;

    // --- Optional: Configure Embedder (Feature: "embedders") ---
    #[cfg(feature = "embedders")]
    {
        // Example: OpenAI
        use hyperspace_sdk::OpenAIEmbedder;
        let openai_key = std::env::var("OPENAI_API_KEY").unwrap();
        let embedder = OpenAIEmbedder::new(openai_key, "text-embedding-3-small".to_string());
        
        // Or: Voyage AI
        // use hyperspace_sdk::VoyageEmbedder;
        // let embedder = VoyageEmbedder::new(api_key, "voyage-large-2".to_string());

        client.set_embedder(Box::new(embedder));
        
        // Insert Document
        let mut meta = HashMap::new();
        meta.insert("tag".to_string(), "rust".to_string());
        client.insert_document(100, "Rust is blazing fast.", meta).await?;
        
        // Search Document
        let results = client.search_document("fast systems language", 5).await?;
        println!("Document Search Results: {:?}", results);
    }
    // -----------------------------------------------------------

    // 2. Insert with Vector (Low-Level)
    let vec = vec![0.1; 8];
    let mut meta = HashMap::new();
    meta.insert("name".to_string(), "item-42".to_string());
    
    client.insert(42, vec.clone(), meta).await?;

    // 3. Basic Search
    let results = client.search(vec.clone(), 5).await?;
    
    // 4. Advanced / Hybrid Search
    // e.g. Find semantically similar items that also mention "item"
    let hybrid = Some(("item".to_string(), 0.5)); 
    let results = client.search_advanced(vec, 5, vec![], hybrid).await?;
    
    for res in results {
        println!("Match: {} (dist: {})", res.id, res.distance);
    }
    
    Ok(())
}
```

## Features

*   `embedders`: Enables `set_embedder`, `insert_document`, and `search_document`. Requires `reqwest` and `serde`.
