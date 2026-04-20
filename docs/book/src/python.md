# Python SDK

The official Python client provides an ergonomic wrapper around the gRPC interface.

## Installation

Install from PyPI:

```bash
pip install hyperspacedb
```

## Quick Start

```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="KEY")

# 1. Insert (id comes first)
client.insert(1, [0.1, 0.2], metadata={"tag": "demo"}, collection="docs")

# 2. Hybrid Search (Semantic + BM25)
results = client.search(
    vector=[0.1, 0.2],
    hybrid_query="autonomous robotics",
    hybrid_alpha=0.7,
    collection="docs"
)
```

## Reference

### `HyperspaceClient`

```python
class HyperspaceClient(host="localhost:50051", api_key=None, embedder=None, user_id=None)
```

- `embedder`: Instance of `BaseEmbedder` subclass for client-side vectorization.
- `user_id`: Tenant identifier for multi-tenancy.

### Methods

#### `insert(id, vector=None, document=None, metadata=None, typed_metadata=None, collection="", durability=Durability.DEFAULT) -> bool`
*   `id` (int): Unique identifier (u32).
*   `vector` (List[float]): The embedding.
*   `document` (str): Raw text to embed (requires client-side embedder).
*   `typed_metadata`: Dict with values of type `str`, `int`, `float`, or `bool`.

#### `insert_text(id, text, metadata=None, collection="", durability=Durability.DEFAULT) -> bool`
Server-side vectorization: inserts raw text to be embedded by the database.

#### `search(vector=None, query_text=None, top_k=10, filter=None, filters=None, hybrid_query=None, hybrid_alpha=None, bm25=None, collection="") -> List[dict]`
- `vector`: Query vector.
- `query_text`: Text to embed client-side (if `vector` is `None`).
- `hybrid_query`: Lexical query for BM25.
- `hybrid_alpha`: Weight fusion factor [0, 1].
- `bm25`: Configuration dict (`method`, `language`, `k1`, `b`).

#### `search_text(text, top_k=10, filter=None, filters=None, hybrid_alpha=None, bm25=None, collection="") -> List[dict]`
Server-side vectorization search.

#### `search_batch(vectors, top_k=10, collection="") -> List[List[dict]]`
Multi-query batch search.

#### `trigger_reconsolidation(collection, target_vector, learning_rate) -> bool`
Traces a Riemannian SGD path on the engine (AI Sleep Mode).

#### `rebuild_index(collection, filter_query=None) -> bool`
Supports metadata-aware pruning:
```python
client.rebuild_index("docs", filter_query={"key": "age", "op": "gt", "value": 30.0})
```

#### `analyze_delta_hyperbolicity(vectors, num_samples=1000) -> (float, str)`
Gromov's delta analysis for metric recommendation.

### Hyperbolic & Cognitive Math

```python
from hyperspace.math import (
    mobius_add,
    exp_map,
    log_map,
    frechet_mean,
    local_entropy,
    lyapunov_convergence,
)
```
- `local_entropy`: Detects hallucination/dispersion.
- `lyapunov_convergence`: Verifies COT stability.

