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

## Reference

### `HyperspaceClient`

```python
class HyperspaceClient(host="localhost:50051")
```

Context manager support:
```python
with HyperspaceClient() as client:
    ...
```

### Methods

#### `insert(id, vector, metadata=None) -> bool`
*   `id` (int): Unique identifier (u32).
*   `vector` (List[float]): The embedding. Note: `norm(v) < 1.0` is required.
*   `metadata` (dict): Optional string-string key-value pairs.

#### `search(vector, top_k=10, filter=None) -> List[dict]`
*   `vector` (List[float]): Query vector.
*   `top_k` (int): Number of neighbors to return.
*   `filter` (dict): Exact match filters (e.g. `{"category": "books"}`).

Returns a list of dicts: `[{"id": 123, "distance": 0.45}, ...]`.
