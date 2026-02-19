# Quick Start

Once the server is running on `localhost:50051`, you can use any official SDK.

## 1) Start server

```bash
cargo build --release
./target/release/hyperspace-server
```

## 2) Open dashboard

```text
http://localhost:50050
```

## 3) First interaction (Python)

```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
collection = "quickstart"

client.delete_collection(collection)
client.create_collection(collection, dimension=3, metric="cosine")

client.insert(id=1, vector=[0.1, 0.2, 0.3], collection=collection)
client.insert(id=2, vector=[0.2, 0.1, 0.4], collection=collection)

print(client.search(vector=[0.1, 0.2, 0.3], top_k=2, collection=collection))

# Batch search (recommended for throughput)
batch = client.search_batch(
    vectors=[[0.1, 0.2, 0.3], [0.2, 0.1, 0.4]],
    top_k=2,
    collection=collection,
)
print(batch)
```

## 4) Metric notes

- `cosine`, `l2`, `euclidean`: general embeddings.
- `poincare`: vectors must satisfy `||x|| < 1`.
- `lorentz`: vectors must be on upper hyperboloid sheet.
