# hyperspace-sdk (Rust)

Official Rust client for HyperspaceDB gRPC data plane.

This crate provides:
- authenticated gRPC client
- collection management
- insert/search APIs
- high-throughput `search_batch`
- `f32` helper methods for Euclidean workloads (`insert_f32`, `search_f32`, `search_batch_f32`)

## Installation

```toml
[dependencies]
hyperspace-sdk = "2.2.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use hyperspace_sdk::Client;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
        None,
    ).await?;

    let collection = "docs_rust".to_string();
    let _ = client.delete_collection(collection.clone()).await;
    client.create_collection(collection.clone(), 3, "cosine".to_string()).await?;

    client.insert(
        1,
        vec![0.1, 0.2, 0.3],
        HashMap::new(),
        Some(collection.clone()),
    ).await?;

    let results = client.search(
        vec![0.1, 0.2, 0.3],
        10,
        Some(collection.clone()),
    ).await?;

    println!("results: {}", results.len());
    Ok(())
}
```

## Batch Search

Use `search_batch` to reduce RPC overhead:

```rust
let responses = client.search_batch(
    vec![
        vec![0.1, 0.2, 0.3],
        vec![0.3, 0.1, 0.4],
    ],
    10,
    Some("docs_rust".to_string()),
).await?;
```

Each entry in `responses` corresponds to one query vector.

## f32 Helpers

When your app keeps Euclidean vectors in `f32`, use conversion helpers:

- `insert_f32`
- `search_f32`
- `search_batch_f32`

The crate converts to protocol `f64` once per call.

## API Surface (Core)

- `Client::connect`
- `create_collection`, `delete_collection`, `list_collections`
- `insert`, `insert_f32`
- `search`, `search_f32`, `search_advanced`
- `search_batch`, `search_batch_f32`
- `delete`
- `configure`
- `get_collection_stats`, `get_digest`
- `trigger_vacuum`, `rebuild_index`, `rebuild_index_with_filter`
- `get_neighbors_with_weights` (graph edges with distances)
- `subscribe_to_events` (CDC stream)

### Rebuild with Pruning

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

### Hyperbolic Math Utilities

```rust
use hyperspace_sdk::math::{
    mobius_add, exp_map, log_map, parallel_transport, riemannian_gradient, frechet_mean
};
```

## Optional Feature: Embedders

Enable with:

```toml
hyperspace-sdk = { version = "2.2.1", features = ["embedders"] }
```

## Production Notes

- Reuse long-lived clients instead of reconnecting per request.
- Prefer `search_batch` on concurrency-heavy paths.
- Keep collection metric/dimension consistent with your vector source.

