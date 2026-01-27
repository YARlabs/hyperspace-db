# User Guide

## Server Configuration

HyperspaceDB is configured via gRPC commands at runtime, but defaults can be set via environment variables.

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Log level (debug, info, error) |
| `HS_PORT` | `50051` | gRPC listening port |
| `HS_DATA_DIR` | `./data` | Path to store segments and WAL |
| `HS_DIMENSION` | `1024` | Vector dimensionality (8, 768, 1024, 1536) |
| `HS_DISTANCE_METRIC` | `poincare` | Distance metric (poincare) |
| `HS_QUANTIZATION_LEVEL` | `scalar` | Compression (none, scalar, binary) |
| `HS_HNSW_EF_CONSTRUCT` | `100` | Index build quality (50-500) |
| `HS_HNSW_EF_SEARCH` | `10` | Search beam width (10-500) |

## TUI Dashboard (Mission Control)

Launch the dashboard to monitor your instance:

```bash
./hyperspace-cli
```

### Key Controls

* **TAB**: Switch between Overview, Storage, and Admin tabs.
* **[S]**: Trigger an immediate graph snapshot.
* **[V]**: Trigger storage vacuum (compaction, experimental).
* **[Q]**: Quit the dashboard.

## Persistence & Recovery

HyperspaceDB uses a **Write-Ahead Log (WAL)**.

1. **Insert**: Data is appended to `wal.log` immediately.
2. **Snapshot**: The in-memory graph is periodically serialized to `index.snap`.
3. **Recovery**: On restart, the system loads `index.snap` (instant mmap) and replays any entries in `wal.log` that occurred after the snapshot.
