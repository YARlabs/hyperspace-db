# Python SDK

The official Python client provides an ergonomic wrapper around the gRPC interface.

## Installation

Currently, install directly from the source:

```bash
git clone https://github.com/yarlabs/hyperspace-db
cd hyperspace-db/sdks/python
pip install .
```

(Coming soon to PyPI as `hyperspacedb`)

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

