# User Guide

## Server Configuration

HyperspaceDB is configured via gRPC commands at runtime, but defaults can be set via environment variables.

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Log level (debug, info, error) |
| `HS_PORT` | `50051` | gRPC listening port |
| `HS_DATA_DIR` | `./data` | Path to store segments and WAL |

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
