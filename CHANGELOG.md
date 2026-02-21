# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.2] - 2026-02-19

### Added
* **GPU roadmap bootstrap in core**:
    * Added WGSL kernels for `L2`, `Cosine`, and `Poincare` batch distance.
    * Added reusable exact re-rank primitive (`rerank_topk_exact`) and CPU reference kernels.
* **GPU runtime dispatch (feature-gated)**:
    * Added `gpu-runtime` feature in `hyperspace-core` (`wgpu`, `pollster`, `bytemuck`).
    * `batch_distance_auto` now executes `L2/Cosine/Poincare/Lorentz` via `wgpu` when enabled, with safe CPU fallback (`GpuFallbackCpu`).
    * Added dedicated Lorentz float runtime kernel and connected `GpuMetric::Lorentz` in `kernel_for_metric`.
    * Added persistent GPU runtime cache (single device/pipeline initialization reused across requests).
    * Added GPU scratch-buffer reuse pool to reduce per-request buffer allocations in batch kernels.
    * Added conservative GPU offload thresholds: `HS_GPU_MIN_BATCH`, `HS_GPU_MIN_DIM`, `HS_GPU_MIN_WORK`.
* **Per-metric GPU controls**:
    * Added `HS_GPU_L2_ENABLED`, `HS_GPU_COSINE_ENABLED`, `HS_GPU_POINCARE_ENABLED`, `HS_GPU_LORENTZ_ENABLED`.
* **Benchmarking**:
    * Added `gpu_dispatch_bench` for CPU-reference vs auto-dispatch comparisons.
* **Batch Search runtime throughput**:
    * `search_batch` server handler now supports bounded per-query fan-out while preserving response order.
    * Added `HS_SEARCH_BATCH_INNER_CONCURRENCY` for deterministic load-scaling control.
* **Search re-rank integration**:
    * Added optional exact re-ranking in server search path via `HS_RERANK_ENABLED`.
    * Added `HS_RERANK_OVERSAMPLE` to control ANN candidate expansion before exact top-K ordering.
* **GPU runtime dispatch contract**:
    * Added `batch_distance_auto` and backend tags in core (`Cpu` / `GpuDispatchPlanned`).
    * Added `HS_GPU_BATCH_ENABLED` configuration toggle for batch kernel dispatch policy.

### Fixed
* **WGSL runtime parser compatibility**:
    * Fixed shader `Params` struct field separators for `wgpu` validation on Apple Silicon runtime path.


## [2.2.1] - 2026-02-17

### Added
* **Vacuum/Rebuild with pruning filter**:
    * `RebuildIndexRequest` now supports optional `filter_query` for metadata-driven forgetting.
    * Supported operators: `lt`, `lte`, `gt`, `gte`, `eq`, `ne`.
* **SDK Hyperbolic Math Utilities**:
    * Added `mobius_add`, `exp_map`, `log_map` in Python SDK.
    * Added `math` module with `mobius_add`, `exp_map`, `log_map` in Rust SDK.
* **Dashboard Graph Explorer activation**:
    * Enabled `/graph` route with working controls for neighbors and concept parents.
* **SDK math expansion**:
    * Added `parallel_transport` and `riemannian_gradient` to Python SDK.
    * Added `parallel_transport` and `riemannian_gradient` to Rust SDK.
    * Added `parallelTransport` and `riemannianGradient` to TypeScript SDK.
    * Added Fréchet mean utilities (`frechet_mean`/`frechetMean`) to Python, Rust, and TypeScript SDKs.
* **CDC SDK coverage**:
    * Added `subscribe_to_events` helpers to Python, TypeScript, and Rust SDKs.
* **Typed numeric filtering upgrade**:
    * Search `Range` filters now evaluate numeric values with `f64` semantics and support typed metadata numeric fields.
    * HTTP filter payloads now support decimal `gte/lte` thresholds.
    * gRPC `Range` extended with backward-compatible `gte_f64` / `lte_f64` fields for decimal thresholds.

### Changed
* **TS SDK upgrade**:
    * Added `rebuildIndex` and `rebuildIndexWithFilter`.
    * Added `HyperbolicMath` helpers in client package.
    * Updated npm package version to `2.2.1`.
* **SDK docs refresh**:
    * Updated TypeScript, Python, and Rust SDK README files for new API surface.
* **Fast upsert path**:
    * For existing IDs, if vector perturbation is below `HS_FAST_UPSERT_DELTA` and metadata is unchanged, updates storage/WAL without full graph relink.
* **CDC reliability**:
    * Event streaming and replication loops now handle `broadcast::Lagged` gracefully instead of terminating stream tasks.
    * Added tunable `HS_EVENT_STREAM_BUFFER` for higher event load.
* **Graph API edge weights**:
    * `GetNeighborsResponse` now returns `edge_weights` aligned with the returned neighbor order.

## [2.2.0] - 2026-02-17

### Added
* **Graph Traversal API (initial production-safe release)**:
    * Added gRPC methods: `GetNode`, `GetNeighbors`, `GetConceptParents`, `Traverse`, `FindSemanticClusters`.
    * Added graph payload models: `GraphNode`, `GraphCluster`, traversal/cluster request-response types.
    * Implemented server-side graph access to HNSW layers with guards (`max_depth`, `max_nodes`, `max_clusters`).
    * Added `offset` + `limit` pagination model for neighbor listing.
* **CDC/Event Stream hardening**:
    * Added `SubscribeToEvents` streaming endpoint.
    * Streams `VectorInserted` and `VectorDeleted` events for external hooks/microservices.
* **Typed Metadata API surface**:
    * Added `MetadataValue` (string/int64/double/bool) in protobuf.
    * Added `typed_metadata` to insert/search/replication event contracts.

### Changed
* **Storage optimization**:
    * Added optional raw storage mode `HS_STORAGE_FLOAT32` (`mode=none` only).
    * Raw vectors can be stored in `f32` and promoted to `f64` in the distance path.
* **Dimension support expansion**:
    * Added/confirmed large-dimension collection support in manager wiring: `3072`, `4096`, `8192`.
* **Filter-aware traversal**:
    * Added graph traversal filtering via `filter` and `filters` (`Match` / `Range`) in `TraverseRequest`.
* **SDK coverage**:
    * Added graph traversal methods to Python, TypeScript and Rust SDK clients.

### Fixed
* **Compatibility path**:
    * Preserved legacy string metadata behavior while enabling typed metadata roundtrip.
* **Validation and tooling**:
    * Added graph correctness tests and traversal benchmark baseline script.

## [2.1.0] - 2026-02-17

### Added
* **gRPC Batch Search API**:
    * Added `SearchBatch` RPC with `BatchSearchRequest` / `BatchSearchResponse`.
    * Implemented server-side handling and SDK support (Python, TypeScript, Rust).
* **Lorentz Metric Support**:
    * Added `LorentzMetric` to core metric abstraction.
    * Added collection instantiation support for `metric="lorentz"` across supported dimensions.
    * Added validation for upper-sheet hyperboloid constraints.

### Changed
* **Benchmark suite alignment**:
    * Updated benchmark scripts to use batch-search paths for Hyperspace where supported.
    * Reduced benchmark-side RPC overhead to better reflect server concurrency behavior.
* **Documentation and SDK publish readiness**:
    * Refreshed SDK docs for PyPI/npm/crates publication quality.
    * Updated API and architecture documentation for v2.1 behavior.

### Fixed
* **Lorentz integration stability**:
    * Fixed manager-side metric import and compile failures.
    * Improved numerical stability in Lorentz distance path.
* **Repository-wide quality gates**:
    * Resolved strict `clippy -D warnings` failures in test targets.
    * Updated outdated tests for WAL API signature and metadata layout.

## [2.0.0] - 2026-02-16

### Added
*   **Replication Anti-Entropy**: Implemented catch-up mechanism for follower nodes using logical clocks in the Write-Ahead Log (WAL).
    *   Followers now report their last persisted clock state upon connection.
    *   Leaders replay missing operations from WAL to ensure consistency.
*   **Multi-Tenancy**: Native support for SaaS-style multi-tenancy.
    *   **Namespace Isolation**: Collections are prefixed with `user_id` (e.g., `{user_id}_{collection_name}`).
    *   **Context Propagation**: `x-hyperspace-user-id` header is propagated through HTTP and gRPC layers.
    *   **Billing Foundations**: New `/api/admin/usage` endpoint provides disk and vector usage breakdown per user.
*   **WASM Flexibility**: Completely refactored `hyperspace-wasm` to support dynamic configurations.
    *   Supports multiple dimensions (384, 768, 1024, 1536) and metrics (Euclidean, Cosine).
    *   Automatic index type selection based on initialization parameters.
*   **Persistence Upgrades**:
    *   **Metadata Persistence**: Filters and deleted items are now correctly saved and restored in snapshots.
    *   **Logical Clocks**: WAL entries now include logical timestamps for precise state restoration.

### Changed
*   **Major Version Bump**: All crates updated to v2.0.0.
*   **API Updates**:
    *   `Replicate` gRPC method now accepts `ReplicationRequest` instead of `Empty`.
    *   Collection listing now filters by `user_id` context.

### Fixed
*   **WAL Replay**: Fixed issue where legacy WAL entries (OpCode 2) could cause replay failures; implemented backward compatibility.
*   **Docker Build**: Optimized Docker images with `strip` and LTO for smaller footprint.

## [1.6.0] - 2026-02-15

### Added
*   **Cold Storage Architecture**: Implemented lazy loading mechanism where collections are loads from disk only upon first access, optimizing startup time and resource usage.
*   **Idle Eviction**: Introduced a background monitor (Reaper) that automatically unloads collections inactive for more than 1 hour, freeing up RAM.
*   **Graceful Shutdown**: Implemented `Drop` trait for Collections to ensure immediate cancellation of background tasks (indexing, snapshots) upon deletion or eviction, preventing memory leaks and panics.
*   **Manual Vacuum**: Enhanced `trigger_vacuum` endpoint to explicitly trigger memory cleanup routines.
*   **Index Rebuild**: Added `rebuild_index` API to defragment and optimize collections live without downtime.
*   **Queue Monitoring**: Exposed `indexing_queue` size in collection stats for real-time visibility into ingestion backlog.

### Changed
*   **Async Access**: Refactored `CollectionManager` to use asynchronous retrieval (`get().await`), enabling non-blocking disk I/O for cold collections.
*   **Stability**: Fixed "Snapshot Error" panics caused by orphaned background tasks.


## [1.5.0] - 2026-02-09

### Added
*   **Hyperbolic Efficiency**: Optimized Poincaré ball model implementation for 64d vectors, achieving 2.47ms p99 latency with significant storage savings (64d vs 1024d is 16x compression).
*   **Benchmarks**: Added comprehensive benchmarking suite comparisons against Milvus, Qdrant, and Weaviate.
    *   `run_benchmark_hyperbolic.py`: Specific script for demonstrating Hyperbolic vs Euclidean efficiency.
    *   `BENCHMARK_RESULTS.md` and `HYPERBOLIC_BENCHMARK_RESULTS.md`: Official performance reports.

### Performance
*   **Instant Startup**: Implemented `mmap` (memory-mapped file I/O) for snapshot loading.
    *   Replaces synchronous read-all-at-once approach.
    *   Added visual progress bar for graph reconstruction.
    *   Significantly reduced memory pressure during startup.
*   **High-Throughput Ingestion**: Replaced bounded channels with **Unbounded Channels** in the ingestion pipeline.
    *   Eliminated backpressure bottlenecks that caused performance degradation after 100k vectors.
    *   Ingestion stability improved to consistent ~156k QPS for 64d vectors.
*   **Zero-Copy Deserialization**: Enhanced `rkyv` usage with `mmap` for true zero-copy snapshot restoration.

### Fixed
*   **Panic in Search**: Resolved `Index out of bounds` panic in `search_layer` caused by empty layers in edge cases.
*   **WASM Compatibility**: Fixed missing `export` and `from_bytes` methods in `hyperspace-wasm` when using `mmap` feature.
*   **Benchmark Script**: Fixed API key authentication issues and Weaviate deprecation warnings in benchmark scripts.

## [1.4.0] - 2026-02-05

### Added
*   **WebAssembly Core**: `hyperspace-core` and indexes now compile to WASM (`wasm32-unknown-unknown`).
*   **Edge Database**: New `hyperspace-wasm` crate for running the database purely in-browser (RamStore backend).
*   **Architecture**:
    *   **RAM Vector Store**: In-memory storage backend for runtime environments without disk access.
    *   **Feature Gating**: Optional `mmap` and `persistence` features for `no_std` / WASM compatibility.

## [1.2.0] - 2026-02-04

### Added
* **Multi-Tenancy (Collections)**: Full support for named collections within a single instance.
    * Each collection has independent dimension, metric (Poincaré/Euclidean), and quantization settings.
    * Persistent metadata storage (`meta.json`) for collection configuration.
    * gRPC APIs: `CreateCollection`, `DeleteCollection`, `ListCollections`, `GetCollectionStats`.
* **Web Dashboard**: Professional React-based management interface.
    * **Authentication**: API key-based access control (default: `I_LOVE_HYPERSPACEDB`).
    * **Collection Management**: Create/delete collections with preset configurations:
        * Hyperbolic: 16D, 32D, 64D, 128D (Poincaré metric)
        * Euclidean: 1024D, 1536D, 2048D (L2 metric)
    * **Poincaré Disk Visualizer**: Interactive canvas-based visualization of hyperbolic vector space.
    * **System Metrics**: Real-time monitoring of collections, vectors, memory, and QPS.
    * **Responsive UI**: Modern design with tab-based navigation.
* **Euclidean Metric**: Added `EuclideanMetric` implementation for standard L2 distance.
* **Extended Dimension Support**: Added support for 16D, 32D, 64D, 128D, 2048D vectors.
* **HTTP API**: RESTful endpoints for dashboard integration:
    * `GET /api/collections` - List all collections with detailed stats (count, dimension, metric)
    * `POST /api/collections` - Create new collection
    * `DELETE /api/collections/{name}` - Delete collection
    * `GET /api/collections/{name}/stats` - Get collection statistics
    * `GET /api/collections/{name}/peek?limit=N` - View recent vectors with metadata
    * `POST /api/collections/{name}/search` - Search vectors via HTTP (top_k configurable)
    * `GET /api/status` - System status and configuration
    * `GET /api/metrics` - Real-time metrics (vectors, collections, RAM, CPU)
    * `GET /api/logs` - Live server logs
* **Data Explorer**: New dashboard page for inspecting raw vector data and testing search queries.
* **Search Playground**: Interactive UI for validating search functionality with custom vectors.
* **Federated Clustering (Beta)**: Initial implementation of distributed state managment.
    *   **Node Identity**: Each node has a persistent `node_id`.
    *   **Cluster Topology**: HTTP API `/api/cluster/status` to view mesh topology.
    *   **Logical Clocks**: Lamport clocks added to replication logs for causal ordering.
* **SDK Ecosystem Expansion**:
    *   **Python SDK**: Complete API coverage including collection management (`create_collection`, `list_collections`).
    *   **TypeScript SDK (Beta)**: Native Node.js client with Promise-based API.
    *   **Rust SDK**: Updated for v1.2.0 with cluster awareness.
* **shadcn/ui Components**: Production-ready UI component library integration.


### Changed
* **Default HTTP Port**: Changed from 3000 to 50050 to avoid conflicts.
* **Collection-Scoped Operations**: All data operations (insert/search/delete) now support `collection` field.
* **Backward Compatibility**: Empty `collection` field defaults to `"default"` collection.

### Fixed
* **Blocking Send Panic**: Wrapped `blocking_send` in `tokio::task::block_in_place` to prevent runtime panics.
* **Collection Metadata**: Proper persistence and loading of collection configuration.

### Security
* **Dashboard Authentication**: API key validation middleware for all HTTP endpoints.
* **SHA-256 Hashing**: Secure API key comparison using cryptographic hashing.

## [1.1.0] - 2026-01-27
### Added
* **Generic Dimensions**: Support for 1024, 1536, and 768 dimensional vectors (previously limited to 8).
* **Runtime Config**: Configuration via `.env` files (Dispatcher pattern) for dimensions and HNSW params.
* **Metric Abstraction**: Generic `Metric` trait for swappable distance formulas.
* **Client-Side Vectorization (Fat Client)**: SDKs now support built-in embedding generation.
    *   **Python SDK**: Support for OpenAI, Cohere, Voyage, Google (Gemini), and local SentenceTransformers (`bge-m3`).
    *   **Rust SDK**: Added `Embedder` trait with implementations for OpenAI, OpenRouter, Cohere, Voyage, and Google (Gemini).

## [1.0.0] - 2026-01-25

### 🚀 Initial Release ("Hyperspace One")

HyperspaceDB v1.0 is the first production-ready release of the fastest hyperbolic vector database.

### Features
*   **Core Engine**: Hyperbolic HNSW implementation optimized for Poincaré ball model.
*   **Performance**: Sub-millisecond search at 1M+ vector scale.
*   **Storage**: Segmented memory-mapped storage with `ScalarI8` and `Binary` quantization (8x and 32x-64x compression respectively).
*   **Persistence**: Write-Ahead Log (WAL) and Zero-Copy Snapshots (Rkyv).
*   **Concurrency**: Async Write Buffer handling 15k+ inserts/sec.
*   **Monitoring**: Real-time TUI dashboard (ratatui) for QPS and system health.
*   **Deployment**: Docker/Docker-Compose support.
*   **SDKs**: Initial Beta support for Python and Rust.

### Improvements
*   Use `std::simd` (Portable SIMD) for distance calculations on nightly Rust.
*   Dynamic configuration of `ef_search` and `ef_construction` via gRPC.
