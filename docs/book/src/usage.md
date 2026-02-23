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
| `HS_DIMENSION` | `1024` | Default vector dimensionality (8, 64, 768, 1024, 1536, 3072, 4096, 8192) |
| `HS_METRIC` | `cosine` | Distance metric (`cosine`, `poincare`, `l2`, `euclidean`, `lorentz`) |
| `HS_QUANTIZATION_LEVEL` | `none` | Compression (`none`, `scalar` (i8), `binary` (1-bit)) |
| `HS_STORAGE_FLOAT32` | `false` | Store raw vectors as `f32` (`mode=none`) and promote to `f64` in distance kernels |
| `HS_FAST_UPSERT_DELTA` | `0.0` | Fast upsert L2 threshold. `0.0` disables; typical `0.001..0.05` for iterative updates; too high can keep stale graph links |
| `HS_EVENT_STREAM_BUFFER` | `1024` | Broadcast ring size for CDC and replication streams |
| `HS_RERANK_ENABLED` | `false` | Enable exact top-K re-ranking after ANN candidate retrieval |
| `HS_RERANK_OVERSAMPLE` | `4` | Candidate multiplier used before exact re-rank (`top_k * factor`) |
| `HS_GPU_BATCH_ENABLED` | `false` | Enable runtime auto-dispatch policy for batch metric kernels |
| `HS_GPU_MIN_BATCH` | `128` | Minimum batch size for GPU offload policy |
| `HS_GPU_MIN_DIM` | `1024` | Minimum vector dimension for GPU offload policy |
| `HS_GPU_MIN_WORK` | `262144` | Minimum workload (`batch * dim`) for GPU offload |
| `HS_GPU_L2_ENABLED` | `true` | Enable GPU dispatch for L2 batch kernel (requires `gpu-runtime` feature) |
| `HS_GPU_COSINE_ENABLED` | `true` | Enable GPU dispatch for cosine batch kernel (requires `gpu-runtime` feature) |
| `HS_GPU_POINCARE_ENABLED` | `true` | Enable GPU dispatch for Poincaré batch kernel (requires `gpu-runtime` feature) |
| `HS_GPU_LORENTZ_ENABLED` | `true` | Enable GPU dispatch for Lorentz float batch kernel (runtime path) |
| `HS_SEARCH_BATCH_INNER_CONCURRENCY` | `1` | Internal parallel fan-out in `SearchBatch` handler (bounded) |
| `HS_SEARCH_CONCURRENCY` | `0` | Global concurrent search-task limit per collection (`0` = auto by CPU cores, max clamped to `CPU*4`) |

### Cloud Tiering (S3)

*Enabled only when compiled with `s3-tiering` feature.*

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HS_STORAGE_BACKEND` | `local` | `local` (all chunks on disk) or `s3` (offload cold chunks) |
| `HS_MAX_LOCAL_CACHE_GB` | `10` | Hard limit for local disk cache in Gigabytes |
| `HS_S3_BUCKET` | - | Target S3 bucket name |
| `HS_S3_REGION` | `us-east-1` | AWS Region |
| `HS_S3_ENDPOINT` | - | Custom endpoint (e.g. `http://minio:9000`) |
| `HS_S3_ACCESS_KEY` | - | S3 Access Key ID |
| `HS_S3_SECRET_KEY` | - | S3 Secret Access Key |
| `HS_S3_MAX_RETRIES` | `5` | Retries for failed uploads/downloads |
| `HS_S3_UPLOAD_CONCURRENCY` | `4` | Semaphore-limited parallel uploads |
| `HS_WAL_SEGMENT_SIZE_MB` | `256` | Size before WAL rotation (influences chunk size) |
| `HS_CHUNK_PROBE_K` | `3` | Number of most relevant chunks to search per query |

### HNSW Index Tuning

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HS_HNSW_M` | `64` | Max connections per layer |
| `HS_HNSW_EF_CONSTRUCT` | `200` | Build quality (50-500). Higher = slower build, better recall. |
| `HS_HNSW_EF_SEARCH` | `100` | Search beam width (10-500). Higher = slower search, better recall. |
| `HS_FILTER_BRUTEFORCE_THRESHOLD` | `50000` | If filtered candidate count is below threshold, layer-0 uses exact brute-force instead of graph traversal |
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
* **Explorer**: Search playground with filters and typed metadata visibility.
* **Graph Explorer**: Query neighbors and concept-parent graph views from HNSW layers.
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

