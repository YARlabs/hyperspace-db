# User Guide

## Server Configuration

HyperspaceDB is configured via environment variables or a `.env` file.

### Core Settings

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `error`) |
| `HS_PORT` | `50051` | gRPC listening port |
| `HS_HTTP_PORT` | `50050` | HTTP Dashboard port |
| `HS_DATA_DIR` | `./data` | Path to store segments and WAL |
| `HS_IDLE_TIMEOUT_SEC` | `3600` | Inactivity time (seconds) before collection unloads to disk |
| `HS_DIMENSION` | `1024` | Default vector dimensionality (8, 64, 768, 1024, 1536) |
| `HS_METRIC` | `cosine` | Distance metric (`cosine`, `poincare`, `l2`, `euclidean`, `lorentz`) |
| `HS_QUANTIZATION_LEVEL` | `none` | Compression (`none`, `scalar` (i8), `binary` (1-bit)) |

### HNSW Index Tuning

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HS_HNSW_M` | `64` | Max connections per layer |
| `HS_HNSW_EF_CONSTRUCT` | `200` | Build quality (50-500). Higher = slower build, better recall. |
| `HS_HNSW_EF_SEARCH` | `100` | Search beam width (10-500). Higher = slower search, better recall. |
| `HS_INDEXER_CONCURRENCY` | `1` | Check README for threading strategies (0=Auto, 1=Serial) |

### Persistence & Durability

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HYPERSPACE_WAL_SYNC_MODE` | `batch` | WAL Sync strategy: `strict` (fsync), `batch` (100ms lag), `async` (OS cache) |
| `HYPERSPACE_WAL_BATCH_INTERVAL` | `100` | Batch interval in milliseconds |

### Memory Management (Jemalloc)

HyperspaceDB uses **Jemalloc** for efficient memory allocation. Tune it via `MALLOC_CONF`:

* **Low RAM (Aggressive)**: `MALLOC_CONF=background_thread:true,dirty_decay_ms:0,muzzy_decay_ms:0`
* **Balanced (Default)**: `MALLOC_CONF=background_thread:true,dirty_decay_ms:5000,muzzy_decay_ms:5000`

### Security

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HYPERSPACE_API_KEY` | - | If set, requires `x-api-key` header for all requests |

### Multi-Tenancy

HyperspaceDB supports strict data isolation via the `x-hyperspace-user-id` header.

*   **Isolation**: Every request with a `x-hyperspace-user-id` header operates within that user's private namespace.
*   **Internal Naming**: Collections are stored internally as `userid_collectionname`.
*   **Default Admin**: If `x-hyperspace-user-id` is omitted but a valid `x-api-key` is provided, the user is treated as `default_admin`.
*   **SaaS Integration**: Gateways should inject this header after authenticating users.

### Lorentz metric notes

When `HS_METRIC=lorentz`, vectors must satisfy hyperboloid constraints:

- `t > 0` (upper sheet)
- `-t^2 + x_1^2 + ... + x_n^2 = -1`

---

## Web Dashboard

HyperspaceDB includes a comprehensive Web Dashboard at `http://localhost:50050`.

**Features:**
* **Cluster Status**: View node role (Leader/Follower) and topology.
* **Collections**: Create, delete, and inspect collection statistics.
* **Explorer**: View inserted vectors and their metadata.
* **Metrics**: Real-time RAM and CPU usage.

## TUI Dashboard (Legacy)

For terminal-based monitoring:

```bash
./hyperspace-cli
```

### Key Controls
* **TAB**: Switch tabs.
* **[S]**: Trigger snapshot.
* **[V]**: Trigger vacuum.
* **[Q]**: Quit.

