# Python SDK

The official Python client provides an ergonomic wrapper around the gRPC interface.

## Installation

Install from PyPI:

```bash
pip install hyperspacedb
```

## Client-Side Vectorization (Fat Client)

The SDK supports built-in embedding generation using popular providers (OpenAI, Cohere, etc.). This allows you to insert and search using raw text.

### Installation with Extras

```bash
# Install with OpenAI support
pip install ".[openai]"

# Install with All embedders support
pip install ".[all]"
```

### Usage

```python
from hyperspace import HyperspaceClient, OpenAIEmbedder

# 1. Init with Embedder
embedder = OpenAIEmbedder(api_key="sk-...")
client = HyperspaceClient(embedder=embedder)

# 2. Insert Document
client.insert(id=1, document="HyperspaceDB supports Hyperbolic geometry.", metadata={"tag": "math"})

# 3. Search by Text
results = client.search(query_text="non-euclidean geometry", top_k=5)
```

## Reference

### `HyperspaceClient`

```python
class HyperspaceClient(host="localhost:50051", api_key=None, embedder=None)
```

*   `embedder`: Instance of `BaseEmbedder` subclass.

### Supported Embedders

*   `OpenAIEmbedder`
*   `OpenRouterEmbedder`
*   `CohereEmbedder`
*   `VoyageEmbedder`
*   `GoogleEmbedder`
*   `SentenceTransformerEmbedder` (Local models)

### Methods

#### `insert(id, vector=None, document=None, metadata=None) -> bool`
*   `id` (int): Unique identifier (u32).
*   `vector` (List[float]): The embedding.
*   `document` (str): Raw text to embed (requires configured embedder).
*   **Note**: Provide either `vector` OR `document`.

#### `search(vector=None, query_text=None, top_k=10, ...) -> List[dict]`
*   `vector` (List[float]): Query vector.
*   `query_text` (str): Raw text query.

#### `search_batch(vectors, top_k=10, collection="") -> List[List[dict]]`
Batch search API that sends multiple `SearchRequest` objects in one gRPC call.

#### `rebuild_index(collection, filter_query=None) -> bool`
Supports metadata-aware pruning during rebuild:

```python
client.rebuild_index(
    "docs_py",
    filter_query={"key": "energy", "op": "lt", "value": 0.1},
)
```

#### Graph traversal methods

- `get_node(collection, id, layer=0)`
- `get_neighbors(collection, id, layer=0, limit=64, offset=0)`
- `get_concept_parents(collection, id, layer=0, limit=32)`
- `traverse(collection, start_id, max_depth=2, max_nodes=256, layer=0, filter=None, filters=None)`
- `find_semantic_clusters(collection, layer=0, min_cluster_size=3, max_clusters=32, max_nodes=10000)`

#### Hyperbolic math utilities

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

