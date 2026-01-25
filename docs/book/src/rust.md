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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect
    let mut client = Client::connect("http://127.0.0.1:50051".into()).await?;

    // 2. Insert
    let vec = vec![0.1; 8];
    client.insert(42, vec.clone()).await?;

    // 3. Search
    let results = client.search(vec, 5).await?;
    
    for res in results {
        println!("Match: {} (dist: {})", res.id, res.distance);
    }
    
    Ok(())
}
```
