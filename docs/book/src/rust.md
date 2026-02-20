# Rust SDK

For low-latency applications, connect directly using the Rust SDK.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hyperspace-sdk = "2.2.1"
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
    let mut client = Client::connect(
        "http://127.0.0.1:50051".into(),
        api_key,
        None
    ).await?;

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
    
    client.insert(42, vec.clone(), meta, None).await?;

    // 3. Basic Search
    let results = client.search(vec.clone(), 5, None).await?;
    
    // 4. Advanced / Hybrid Search
    // e.g. Find semantically similar items that also mention "item"
    let hybrid = Some(("item".to_string(), 0.5)); 
    let results = client.search_advanced(vec, 5, vec![], hybrid, None).await?;
    
    for res in results {
        println!("Match: {} (dist: {})", res.id, res.distance);
    }
    
    Ok(())
}
```

## Features

*   `embedders`: Enables `set_embedder`, `insert_document`, and `search_document`. Requires `reqwest` and `serde`.

## Batch Search

Use `search_batch` or `search_batch_f32` to reduce per-request overhead in high-concurrency workloads.

## Graph Traversal API

Rust SDK exposes graph calls directly:

- `get_node`
- `get_neighbors`
- `get_concept_parents`
- `traverse`
- `find_semantic_clusters`

## Rebuild with Metadata Pruning

Use `rebuild_index_with_filter` to run vacuum/rebuild and prune vectors in one request:

```rust
client
    .rebuild_index_with_filter(
        "docs_rust".to_string(),
        "energy".to_string(),
        "lt".to_string(),
        0.1,
    )
    .await?;
```

## Hyperbolic Math Utilities

```rust
use hyperspace_sdk::math::{
    mobius_add, exp_map, log_map, parallel_transport, riemannian_gradient, frechet_mean
};
```
