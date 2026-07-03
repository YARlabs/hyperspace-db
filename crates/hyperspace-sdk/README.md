# hyperspace-sdk (Rust)

Official Rust client for HyperspaceDB gRPC data plane.

This crate provides:
- authenticated gRPC client
- collection management
- insert/search APIs
- recursive logical filters (`AND`, `OR`, `NOT`)
- bulk point management (`get_points`, `update_payload`, `scroll`, `count`)
- high-throughput `search_batch`
- `f32` helper methods for Euclidean workloads (`insert_f32`, `search_f32`, `search_batch_f32`)
- system health monitoring (`health_check`)

## Installation

```toml
[dependencies]
hyperspace-sdk = "3.1.0"
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

    // Use the new Schema-driven API
    use hyperspace_proto::hyperspace::{CollectionSchema, VectorComponent};
    let schema = CollectionSchema {
        components: vec![VectorComponent {
            name: "primary".to_string(),
            metric: "cosine".to_string(),
            full_dimension: 3,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
    };

    client.create_collection(collection.clone(), schema).await?;

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

## Hybrid & Lexical Search (BM25)

HyperspaceDB supports advanced BM25 lexical ranking and hybrid fusion.

### 1. Pure Lexical Search
Use `search_text` without serving a vector. You can optionally configure BM25 parameters:

```rust
use hyperspace_proto::hyperspace::Bm25Options;

let bm25_config = Bm25Options {
    method: Some("bm25plus".to_string()),
    k1: Some(1.2),
    b: Some(0.75),
    language: Some("english".to_string()),
    ..Default::default()
};

let results = client.search_text(
    "hyperspatial retrieval".to_string(),
    10,
    Some("docs".to_string()),
    Some(bm25_config),
).await?;
```

### 2. Hybrid Search
Combine semantic vector search with BM25 lexical ranking using `search_hybrid`.

```rust
let vector = vec![0.1, -0.2, 0.5];
let text = "quantum computing".to_string();
let alpha = 0.7; // weighting: 70% vector, 30% lexical

let results = client.search_hybrid(
    vector,
    text,
    alpha,
    10,
    Some("docs".to_string()),
    None, // Use collection defaults for BM25
).await?;
```

## Matryoshka Representation Learning (MRL) & Cascading

HyperspaceDB supports MRL through its **Cascade Pipeline**. This allows you to perform initial fast search on a truncated low-dimensional vector (e.g., 64D) and then rerank the results using the full vector (e.g., 1024D).

```rust
use hyperspace_proto::hyperspace::{CollectionSchema, VectorComponent, MrlLayer};

let schema = CollectionSchema {
    components: vec![VectorComponent {
        name: "primary".to_string(),
        metric: "lorentz".to_string(),
        full_dimension: 1025, // 1024 + 1
        weight: 1.0,
    }],
    cascade_pipeline: vec![MrlLayer {
        component_name: "primary".to_string(),
        cutoff_dimension: 129, // Perform initial search on 128D (+1)
        store_in_ram: true,
        rerank_top_k: 100,
    }],
};

client.create_collection("mrl_collection".to_string(), schema).await?;
```

## f32 Helpers

When your app keeps Euclidean vectors in `f32`, use conversion helpers:

- `insert_f32`
- `search_f32`
- `search_batch_f32`

The crate converts to protocol `f64` once per call.

## API Surface (Core)

- `Client::connect`
- `create_collection(name, schema)` (takes `CollectionSchema`)
- `delete_collection`
- `list_collections` (returns `Vec<CollectionSummary>` with name, count, and schema)
- `insert`, `insert_f32`

- `insert_text` (server-side vectorization and storage)
- `get_points` (bulk point retrieval)
- `update_payload` (metadata patching)
- `scroll` (paginated scanning)
- `count` (filtered point counting)
- `health_check` (server status)
- `vectorize` (convert text to vector on server)
- `search`, `search_f32`, `search_advanced`
- `search_text` (lexical search, vectorized on server)
- `search_hybrid` (combined lexical + vector search)
- `search_batch`, `search_batch_f32`, `search_wasserstein`, `search_multi_collection`
- `delete`
- `configure`
- `get_collection_stats`, `get_digest`
- `trigger_vacuum`, `trigger_reconsolidation`, `rebuild_index`, `rebuild_index_with_filter`
- `get_neighbors_with_weights` (graph edges with distances)
- `get_subsumption_tree` (extract hierarchy from Lorentz/Poincare data)
- `explore_graph` (returns ego-graph for visualization)
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

### Graph Diagnostics (Gromov Delta)
Analyze your dataset structure directly on the client to select the correct metric:
```rust
use hyperspace_sdk::gromov::analyze_delta_hyperbolicity;

// Returns the delta value and recommended metric (lorentz, poincare, cosine, l2)
let (delta, metric) = analyze_delta_hyperbolicity(&vectors, 100);
```

### AI Sleep Mode / Memory Reconsolidation
Trigger the database to run Riemannian SGD via Flow Matching natively:
```rust
client.trigger_reconsolidation("my_collection".to_string(), target_vector, 0.01).await?;
```

### Cognitive Math SDK (Spatial AI Engine)
Provides advanced tools for Agentic AI, running entirely on the client side:
```rust
use hyperspace_sdk::math::{
    local_entropy, lyapunov_convergence, koopman_extrapolate, context_resonance
};

// 1. Detect Hallucinations (Entropy approaches 1.0)
let entropy = local_entropy(&thought_vector, &neighbors, curvature)?;

// 2. Proof of Convergence (Negative derivative = convergence)
let stability = client.get_trust_score(trajectory_ids, None).await?;

// 3. Extrapolate next thought (Koopman linearization)
let next_thought = client.predict_momentum(trajectory_ids, 1.0).await?;

// 4. Phase-Locked Loop for topic tracking
let synced_thought = context_resonance(&thought, &global_context, 0.5, curvature)?;

// 5. Predict Semantic Relation (A + R ≈ B)
let relation = client.predict_relation(id_a, id_b, None).await?;
```

## Implicit Graph Engine (v3.1)

HyperspaceDB treats your vectors as nodes in a dynamic graph. Relationships are inferred from the geometry:
- **Lorentz / Poincare**: Hierarchy and subsumption (light cones).
- **L2 / Cosine**: Semantic similarity and adjacency.

### Subsumption Trees
Extract directed hierarchies from Lorentz-encoded data:
```rust
let tree = client.get_subsumption_tree(root_id, 5, None).await?;
```

### Advanced Traversal
Navigate the graph using physical kernels:
```rust
let results = client.traverse(
    start_id,
    2, // max depth
    256, // max nodes
    0, // layer
    Some(2), // traversal_mode (2 = MOMENTUM)
    Some(5), // breadth_limit
    None, // filter
    vec![], // filters
    None, // collection
).await?;
```
```

## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** — each of the 5 distance types (`l2`, `cosine`, `poincare`, `lorentz`, `hybrid`) can have its own embedding backend configured independently.

### Available Backends

| Backend | Feature Flag | Description |
|---|---|---|
| Local ONNX | `local-onnx` | Load `model.onnx` + `tokenizer.json` from disk |
| HuggingFace Hub | `huggingface` | Auto-download `model.onnx` + `tokenizer.json` from Hub |
| OpenAI / OpenRouter | `embedders` | Cloud API with OpenAI-compatible protocol |
| Cohere | `embedders` | Cohere `/v1/embed` endpoint |
| Voyage AI | `embedders` | Voyage `/v1/embeddings` endpoint |
| Google Gemini | `embedders` | Gemini `embedContent` endpoint |

### Usage

```toml
# Cargo.toml
[dependencies]
# API providers only
hyperspace-sdk = { version = "3.0.0", features = ["embedders"] }

# Local ONNX files (no network required at inference time)
hyperspace-sdk = { version = "3.0.0", features = ["local-onnx"] }

# Download from HuggingFace Hub (includes local-onnx)
hyperspace-sdk = { version = "3.0.0", features = ["huggingface"] }
```

### EmbedGeometry

Every embedder requires specifying the target geometry, which controls post-processing:

```rust
use hyperspace_sdk::embedder::EmbedGeometry;

// Cosine / dot-product: vectors are unit-normalized
let geom = EmbedGeometry::Cosine;

// L2 / Euclidean: vectors are unit-normalized
let geom = EmbedGeometry::L2;

// Poincaré ball: vectors are clamped inside the unit ball (||x|| < 1)
let geom = EmbedGeometry::Poincare;

// Lorentz hyperboloid: no post-processing (model head handles constraint)
let geom = EmbedGeometry::Lorentz;

// Parse from collection metric string
let geom = EmbedGeometry::from_str("poincare");
```

### Local ONNX Embedder

```rust
use hyperspace_sdk::embedder::{LocalOnnxEmbedder, EmbedGeometry, Embedder};

let embedder = LocalOnnxEmbedder::new(
    "./models/bge-small-en-v1.5.onnx",
    "./models/bge-small-en-v1.5-tokenizer.json",
    EmbedGeometry::Cosine,
)?;

let vector = embedder.encode("Hello, hyperbolic world!").await?;
```

### HuggingFace Hub Embedder

Downloads `model.onnx` and `tokenizer.json` automatically from the Hub on first use. Files are cached locally (`~/.cache/huggingface/hub`).

```rust
use hyperspace_sdk::embedder::{HuggingFaceEmbedder, EmbedGeometry, Embedder};

// Public model — no token needed
let embedder = HuggingFaceEmbedder::new(
    "BAAI/bge-small-en-v1.5",
    None, // HF token (None for public models)
    EmbedGeometry::Cosine,
)?;

// Private or gated model — provide HF_TOKEN
let embedder = HuggingFaceEmbedder::new(
    "your-org/cde-spatial-lorentz-128d",
    std::env::var("HF_TOKEN").ok(),
    EmbedGeometry::Lorentz,
)?;

let vector = embedder.encode("Retrieve context").await?;
```

### OpenAI / Remote API Embedder

```rust
use hyperspace_sdk::embedder::{OpenAIEmbedder, Embedder};

let embedder = OpenAIEmbedder::new(
    std::env::var("OPENAI_API_KEY").unwrap(),
    "text-embedding-3-small".to_string(),
);

let vector = embedder.encode("my document text").await?;
```

### Server-Side Embedding (`InsertText` / `SearchText`)

The server can embed text automatically. Configure in `.env` (see server docs):

```env
HYPERSPACE_EMBED=true

# Cosine geometry via HuggingFace
HS_EMBED_COSINE_PROVIDER=huggingface
HS_EMBED_COSINE_HF_MODEL_ID=BAAI/bge-small-en-v1.5
HS_EMBED_COSINE_DIM=384
HF_TOKEN=hf_your_token_here  # Optional: for gated models

# Lorentz geometry via local ONNX
HS_EMBED_LORENTZ_PROVIDER=local
HS_EMBED_LORENTZ_MODEL_PATH=./models/lorentz_128d.onnx
HS_EMBED_LORENTZ_TOKENIZER_PATH=./models/lorentz_128d_tokenizer.json
HS_EMBED_LORENTZ_DIM=129
```

- Reuse long-lived clients instead of reconnecting per request.
- Prefer `search_batch` on concurrency-heavy paths.
- Keep collection metric/dimension consistent with your vector source.
- For `huggingface` provider, models are cached; first startup incurs download time.
- For `lorentz` geometry, dimension is typically spatial_dim + 1 (the time component).

## Zero-Knowledge Client-Side Encryption (ZK-Privacy)

HyperspaceDB v3.1.1 supports Zero-Knowledge client-side encryption (ZK-Privacy) for Rust. Private vectors, metadata, and sidecar payloads are projected, noise-injected, and encrypted *before* they are sent to the database server. All decryption happens locally on the client.

### Usage Example

```rust
use hyperspace_sdk::{Client, CollectionSchema, VectorComponent};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to server
    let mut client = Client::connect("http://localhost:50051".to_string(), Some("I_LOVE_HYPERSPACEDB".to_string()), None).await?;

    let collection = "secure_collection";
    let secret_key = "my-super-secret-key-rust";

    // 2. Configure schema
    let schema = CollectionSchema {
        components: vec![VectorComponent {
            name: "primary".to_string(),
            metric: "cosine".to_string(),
            full_dimension: 3,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
    };

    // 3. Register client key to enable ZK client-side encryption flow
    // noise_sigma defaults to 0.02
    client.register_collection_key(collection.to_string(), secret_key.to_string(), "cosine".to_string(), 0.02, Some(schema.clone()));

    client.create_collection(collection.to_string(), schema).await?;

    // 4. Insert (Auto ZK-privacy: projected, noise injected, payload encrypted, metadata obfuscated)
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), "private".to_string());
    let payload = b"Secret payload content".to_vec();

    client.insert_secure(
        1,
        vec![0.1, 0.2, 0.3],
        metadata,
        Some(payload),
        Some(collection.to_string()),
    ).await?;

    // 5. Search (Query vector projected and filters hashed; payloads decrypted locally)
    let results = client.search(
        vec![0.1, 0.2, 0.3],
        5,
        Some(collection.to_string()),
        None,
        false,
        HashMap::new(),
        false,
        None,
    ).await?;

    for res in results {
        println!("ID: {}, Distance: {}", res.id, res.distance);
        if let Some(p) = res.payload {
            println!("Decrypted Payload: {}", String::from_utf8(p)?);
        }
    }

    Ok(())
}
```

