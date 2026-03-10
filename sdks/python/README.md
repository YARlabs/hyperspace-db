# HyperspaceDB Python SDK

Official Python client for HyperspaceDB gRPC API (v3.0.0-alpha.2).

The SDK is designed for production services and benchmark tooling:
- collection management
- single and batch insert
- single and batch vector search
- graph traversal API methods
- optional embedder integrations
- multi-tenant metadata headers

## Requirements

- Python 3.8+
- Running HyperspaceDB server (default gRPC endpoint: `localhost:50051`)

## Installation

```bash
pip install hyperspacedb
```

Optional embedder extras:

```bash
pip install "hyperspacedb[openai]"
pip install "hyperspacedb[all]"
```

## Quick Start

```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
collection = "docs_py"

client.delete_collection(collection)
client.create_collection(collection, dimension=3, metric="cosine")

client.insert(
    id=1,
    vector=[0.1, 0.2, 0.3],
    metadata={"source": "demo"},
    collection=collection,
)

results = client.search(
    vector=[0.1, 0.2, 0.3],
    top_k=5,
    collection=collection,
)
print(results)

client.close()
```

## Batch Search (Recommended for Throughput)

```python
queries = [
    [0.1, 0.2, 0.3],
    [0.3, 0.1, 0.4],
]

batch_results = client.search_batch(
    vectors=queries,
    top_k=10,
    collection="docs_py",
)
```

`search_batch` reduces per-request RPC overhead and should be preferred for high concurrency.

## API Summary

### Collection Operations

- `create_collection(name, dimension, metric) -> bool`
- `delete_collection(name) -> bool`
- `list_collections() -> list[str]`
- `get_collection_stats(name) -> dict`

### Data Operations

- `insert(id, vector=None, document=None, metadata=None, collection="", durability=Durability.DEFAULT) -> bool`
- `batch_insert(vectors, ids, metadatas=None, collection="", durability=Durability.DEFAULT) -> bool`
- `search(vector=None, query_text=None, top_k=10, filter=None, filters=None, hybrid_query=None, hybrid_alpha=None, collection="") -> list[dict]`
- `search_batch(vectors, top_k=10, collection="") -> list[list[dict]]`
- `search_wasserstein(vector, top_k=10, collection="") -> list[dict]`
- `search_multi_collection(vector, collections, top_k=10) -> dict[str, list[dict]]`

For `filters` with `type="range"`, decimal thresholds are supported (`gte_f64/lte_f64` in gRPC payload are set automatically for non-integer values).

### Maintenance Operations

- `rebuild_index(collection, filter_query=None) -> bool`
- `trigger_vacuum() -> bool`
- `trigger_snapshot() -> bool`
- `configure(ef_search=None, ef_construction=None, collection="") -> bool`
- `trigger_reconsolidation(collection, target_vector, learning_rate) -> bool`
- `subscribe_to_events(types=None, collection=None) -> Iterator[dict]`

`filter_query` example:
```python
client.rebuild_index(
    "docs_py",
    filter_query={"key": "energy", "op": "lt", "value": 0.1},
)
```

CDC subscription example:
```python
for event in client.subscribe_to_events(types=["insert", "delete"], collection="docs_py"):
    print(event)
```

### Hyperbolic Math Utilities

```python
from hyperspace.math import (
    mobius_add,
    exp_map,
    log_map,
    parallel_transport,
    riemannian_gradient,
    frechet_mean,
)
```

### Cognitive Math SDK (Spatial AI Engine)

Provides advanced tools for Agentic AI, running entirely on the client side:

```python
from hyperspace.math import (
    local_entropy,
    lyapunov_convergence,
    koopman_extrapolate,
    context_resonance,
)

# 1. Detect Hallucinations (Entropy approaches 1.0)
entropy = local_entropy(candidate=thought_vector, neighbors=neighbors, c=1.0)

# 2. Proof of Convergence (Negative derivative = convergence)
stability = lyapunov_convergence(trajectory=chain_of_thought, c=1.0)

# 3. Extrapolate next thought (Koopman linearization)
next_thought = koopman_extrapolate(past, current, steps=1.0, c=1.0)

# 4. Phase-Locked Loop for topic tracking
synced_thought = context_resonance(thought, global_context, resonance_factor=0.5, c=1.0)
```

## Durability Levels

Use `Durability` enum values:
- `Durability.DEFAULT`
- `Durability.ASYNC`
- `Durability.BATCH`
- `Durability.STRICT`

## Multi-Tenancy

Pass `user_id` to include `x-hyperspace-user-id` on all requests:

```python
client = HyperspaceClient(
    "localhost:50051",
    api_key="I_LOVE_HYPERSPACEDB",
    user_id="tenant_a",
)
```

## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** — each geometry (`l2`, `cosine`, `poincare`, `lorentz`) can use its own backend independently.

### Quick Setup via Environment Variables

```bash
export HYPERSPACE_EMBED=true

# Cosine geometry → OpenAI API
export HS_EMBED_COSINE_PROVIDER=openai
export HS_EMBED_COSINE_EMBED_MODEL=text-embedding-3-small
export HS_EMBED_COSINE_API_KEY=sk-...

# Poincaré geometry → HuggingFace Hub (auto-downloads ONNX model)
export HS_EMBED_POINCARE_PROVIDER=huggingface
export HS_EMBED_POINCARE_HF_MODEL_ID=your-org/cde-spatial-poincare-128d
export HS_EMBED_POINCARE_DIM=128
export HF_TOKEN=hf_...  # Optional: for gated models

# Lorentz geometry → Local ONNX file
export HS_EMBED_LORENTZ_PROVIDER=local
export HS_EMBED_LORENTZ_MODEL_PATH=./models/lorentz_128d.onnx
export HS_EMBED_LORENTZ_TOKENIZER_PATH=./models/lorentz_128d_tokenizer.json
export HS_EMBED_LORENTZ_DIM=129
```

### Client-Side Embedder

The Python SDK also includes client-side embedders (no server config needed):

```python
from hyperspace.embedder import OpenAIEmbedder, LocalOnnxEmbedder, HuggingFaceEmbedder

# OpenAI
embedder = OpenAIEmbedder(api_key="sk-...", model="text-embedding-3-small")
vector = await embedder.encode("my text")

# Local ONNX — load from disk
embedder = LocalOnnxEmbedder(
    model_path="./models/bge-small.onnx",
    tokenizer_path="./models/bge-small-tokenizer.json",
    geometry="cosine",
)
vector = await embedder.encode("my text")

# HuggingFace Hub — auto-downloads on first use
# Cached at ~/.cache/huggingface/hub
embedder = HuggingFaceEmbedder(
    model_id="BAAI/bge-small-en-v1.5",
    geometry="cosine",
    hf_token=None,  # Set for gated/private models
)
vector = await embedder.encode("my text")
```

### Supported Geometries

| Geometry | Post-Processing | Typical Use Case |
|---|---|---|
| `cosine` | Unit normalize | Semantic similarity |
| `l2` | Unit normalize | Euclidean distance |
| `poincare` | Clamp to unit ball | Hierarchical data (ontologies) |
| `lorentz` | None (model handles it) | Mixed hierarchical + semantic |

## Best Practices

- Reuse one client instance per worker/process.
- Prefer `search_batch` for benchmark and high-QPS paths.
- Chunk large inserts instead of one huge request.
- Keep vector dimensionality aligned with collection configuration.
- For `lorentz` geometry, dimension = spatial_dim + 1 (the time component x₀).
- For `huggingface` provider, models are cached after first download.

## Error Handling

The SDK catches gRPC errors and returns `False` / `[]` in many methods.
For strict production observability, log return values and attach metrics around failed operations.

