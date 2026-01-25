# Quick Start

Once the server is running on `localhost:50051`, you can interact with it using the CLI dashboard or an SDK.

## Using the CLI Dashboard (TUI)

HyperspaceDB comes with a beautiful terminal interface for monitoring and basic administration.

```bash
# Assuming you built from source
./target/release/hyperspace-cli
```

*   **Metric 1: Compression**: Shows how much RAM you are saving with `ScalarI8`.
*   **Metric 2: Index Queue**: Shows if the background worker is keeping up with writes.
*   **Controls**: Press `s` to force a snapshot save.

## First Interaction (Python)

```python
from hyperspace import HyperspaceClient

# 1. Connect
client = HyperspaceClient("localhost:50051")

# 2. Insert (ID must be uint32)
# Vectors must be inside the Poincaré ball (norm < 1.0)
client.insert(id=101, vector=[0.1, 0.5, -0.2, 0.1, 0.0, 0.0, 0.0, 0.1], metadata={"label": "demo"})

# 3. Search
results = client.search(vector=[0.1, 0.5, -0.2, 0.1, 0.0, 0.0, 0.0, 0.0], top_k=1)

print(f"Found ID: {results[0]['id']} with distance {results[0]['distance']}")
```
