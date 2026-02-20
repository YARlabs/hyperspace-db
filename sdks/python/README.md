# HyperspaceDB Python SDK

Official Python client for HyperspaceDB gRPC API (v2.2.1).

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

For `filters` with `type="range"`, decimal thresholds are supported (`gte_f64/lte_f64` in gRPC payload are set automatically for non-integer values).

### Maintenance Operations

- `rebuild_index(collection, filter_query=None) -> bool`
- `trigger_vacuum() -> bool`
- `trigger_snapshot() -> bool`
- `configure(ef_search=None, ef_construction=None, collection="") -> bool`
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
from hyperspace import (
    mobius_add,
    exp_map,
    log_map,
    parallel_transport,
    riemannian_gradient,
    frechet_mean,
)
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

## Best Practices

- Reuse one client instance per worker/process.
- Prefer `search_batch` for benchmark and high-QPS paths.
- Chunk large inserts instead of one huge request.
- Keep vector dimensionality aligned with collection configuration.

## Error Handling

The SDK catches gRPC errors and returns `False` / `[]` in many methods.
For strict production observability, log return values and attach metrics around failed operations.

