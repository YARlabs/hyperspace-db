# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.1.0] - 2026-04-18

### Added
* **BM25 Lexical Upgrade (Sprint 13 / Stability Release)**:
    * **Native BM25 Scoring**: Integrated BM25 algorithm into the core index, supporting `k1`, `b`, and `delta` parameters.
    * **Hybrid Search Fusion**: Full support for RRF (Reciprocal Rank Fusion) and Linear Weighted Fusion to merge vector and keyword results.
    * **New Proto Messages**: Added `Bm25Options` to `SearchRequest` and `SearchTextRequest`.
    * **SDK Parity Milestone**:
        * **TypeScript**: Standardized `insert` argument order. Added `Bm25Options` and full geometric search support.
        * **Python**: Synchronized `HyperspaceClient` signatures. Added support for `hybrid_query` and full `BM25` configuration.
        * **Go**: Regenerated protos and updated `SearchText` to support `BM25`.
        * **C++**: Updated gRPC bindings and documentation for hybrid and geometric search.
    * **Documentation**: Unified all SDK READMEs and updated the official book with Hybrid Search specifications.

### Fixed
* **SDK Inconsistency**: Fixed `insert` argument order in Python and TS SDKs to match the `(id, vector, ...)` convention used in Rust and Go.
* **Hybrid Search Logic**: Resolved issues where `query_text` and `hybrid_query` were ambiguous in client-side embedding flows.
* **Environment Integrity**: Improved AVX2/NEON detection in `hyperspace-core`.

## [3.0.0-rc.2] - 2026-03-21

### Added
* **Finalized Embedding Pipeline (Sprint 12 / Final Release Candidate)**:
    * **New gRPC Endpoints**: Added `InsertText`, `Vectorize`, and `SearchText` RPCs for native server-side vectorization across all geometries.
    * **Native Model Support**:
        * **Qwen3-Embedding-0.6B**: First-class support for L2 and Cosine geometries (1024d, 32K context).
        * **YAR v5 Embedding**: Recommended models for Poincaré (128d) and Lorentz (129d) geometries.
    * **Advanced Text Handling**:
        * **Chunking & Overlap**: Added `HS_EMBED_<M>_CHUNK_SIZE` and `HS_EMBED_<M>_OVERLAP` to handle long text by vectorizing chunks and performing mean-pooling of resulting embeddings.
        * **HS_EMBED_<M>_HF_FILENAME**: Direct control over which file to load from a HuggingFace repository (e.g., `onnx/model.onnx`).
    * **SDK Parity**:
        * **Python**: Added `insert_text`, `vectorize`, `search_text`, and `search_multi_collection_text`.
        * **Rust**: Integrated `insert_text`, `vectorize`, and `search_text` into high-level `Client`.
        * **TS/Go/CPP/Dart**: Full API alignment for server-side embedding tasks.
    * **Integration Tests**: Added `benchmarks/test_all_embeddings.py` verifying 100% pass rate across L2, Cosine, Poincaré, and Lorentz with Qwen3 and YAR v5.

### Fixed
* **Clippy compliance**: Resolved redundant slicing and useless `as_ref()` warnings in the `hyperspace-embed` crate.
* **Documentation**: Updated all SDK READMEs and the official book with the finalized embedding API.

## [3.0.0-alpha.3] - 2026-03-05

### Added
* **SQ8 Anisotropic Quantization (Sprint 6.2 / 7.1)**:
    * Implemented ScaNN-inspired anisotropic loss function $L = \|e_\parallel\|^2 + t_w \cdot \|e_\perp\|^2$ ($t_w = 10$) in `QuantizedHyperVector::from_float()`.
    * Coordinate-descent refinement (±1 in i8-space) applied to all Cosine/L2/Euclidean collections by default. Lorentz uses `from_float_lorentz()` with dynamic-range scaling.
    * Expected Recall@10 improvement: Cosine **+5.3%**, L2 **+3.8%**, Lorentz **+2.4%** at 8x compression (1 byte/dim).
    * `HS_QUANTIZATION_LEVEL=scalar` (default) now enables Anisotropic SQ8 automatically. No code changes required for existing deployments.
    * `HS_ZONAL_QUANTIZATION=true` enables MOND-style zonal storage (`ZonalVector::Core(i8) / Boundary(f64)`), completely replacing the mmap store for nodes.
* **Per-Geometry Embedding System (Sprint 11.1)**:
    * Each of the four distance geometries (`l2`, `cosine`, `poincare`, `lorentz`) now has an independent, configurable embedding backend.
    * **Local ONNX provider**: Load any ONNX model + tokenizer from disk. Zero network at inference time. Configured via `HS_EMBED_<METRIC>_PROVIDER=local`.
    * **HuggingFace Hub provider**: Auto-downloads ONNX model and tokenizer on first startup, cached at `~/.cache/huggingface`. Configured via `HS_EMBED_<METRIC>_PROVIDER=huggingface` + `HS_EMBED_<METRIC>_HF_MODEL_ID`. Supports private/gated models via `HF_TOKEN`.
    * **Remote API providers**: `openai`, `cohere`, `mistral`, `voyage`, `openrouter`, `generic` (any OpenAI-compatible endpoint). Configured per-geometry via `HS_EMBED_<METRIC>_API_KEY` / `HS_EMBED_<METRIC>_EMBED_MODEL`.
    * **EmbedGeometry normalization**: `cosine`/`l2` → unit-normalize; `poincare` → clamp to unit ball; `lorentz` → pass-through (model enforces hyperboloid constraint).
    * **Configuration priority**: Per-metric vars (`HS_EMBED_<METRIC>_*`) → global fallback (`HYPERSPACE_EMBED_*`) → disabled.
    * **Client-side SDK embedders**: `LocalOnnxEmbedder` and `HuggingFaceEmbedder` available via `hyperspace-sdk` feature flags `local-onnx` / `huggingface`.

### Changed
* **Quantization documentation** (`docs/book/src/quantization.md`): Full rewrite. Mode table now includes SQ8 Anisotropic, Lorentz SQ8, Zonal (MOND). Decision guide updated. CLI `--mode` flags removed (never existed; all config via `HS_QUANTIZATION_LEVEL`).
* **Lorentz quantization documentation** (`docs/book/src/lorentz_quantization.md`): Added "Anisotropic Refinement for Lorentz SQ8" section with loss function, Rust pseudocode, and expected recall table.
* **Embeddings documentation** (`docs/book/src/embeddings.md`): Full rewrite. New: per-geometry architecture diagram, `EmbedGeometry` table, full `.env` example with all 4 geometries, SDK code examples in Rust/Python/TypeScript.

### Fixed
* **Documentation accuracy**: Removed non-existent `--mode binary/none` CLI flags from `quantization.md`. Corrected `HS_ZONAL_QUANTIZATION` env var description (it is a separate flag from `HS_QUANTIZATION_LEVEL`, not a value of it).

## [3.0.0-alpha.2] - 2026-03-03

### Added
* **Multi-Geometry Benchmark & SDK Sync (Sprint 10)**:
    * **Graph Diagnostics in SDK**: Added `gromov.rs` in `hyperspace_sdk` to analyze datasets client-side using Gromov's 4-point condition (delta-hyperbolicity) without loading the core DB. Recommends metric (`lorentz`, `poincare`, `cosine`, or `l2`).
    * **AI Sleep Mode / Memory Reconsolidation**: Added `MemoryReconsolidator` directly in `hyperspace_core` leveraging Riemannian SGD to pull vectors closer using Poincare geometry. Exposed via `TriggerReconsolidation` RPC.
    * **Multi-Geometry Search API**: Added `search_multi_collection` to perform parallel batched top-K queries against L2, Cosine, Poincare, and Lorentz metrics simultaneously.
    * **Wasserstein Metric**: Replaced heavy tensor dependency (`wass`) with a native, ultra-fast $O(N)$ 1D L1-CDF algorithm for Cross-Feature Matching (Wasserstein-1). Exposed via `search_wasserstein` in SDK and `use_wasserstein` flag in Proto.
    * **Dependency Pruning**: Removed heavy `hyperball`, `wass`, `ndarray`, and `skel` libraries from core to ensure Hyperspace DB remains lightweight and ultra-fast. Math operations (SGD, metric logic) replaced with custom inline $O(N)$ implementations.
    * **SDK Generation**: Regenerated protobufs spanning Python, TypeScript, Go, and C++ for synchronizing features (Wasserstein, Reconsolidation RPCs).

## [3.0.0-alpha.1] - 2026-02-23

### Added
* **LSM-Tree Vector Search Architecture (Sprint 1)**:
    * **MemTable & Flush Worker**: Active WAL segments now rotate and flush into immutable `HNSW` chunks. RAM is reclaimed by atomically swapping the hot index (MemTable) after flush.
    * **Global Meta-Router**: IVF-style routing layer that maps search queries to relevant immutable chunks via centroid-based pruning (200x faster than linear scan).
    * **Scatter-Gather Search Pipeline**: Read-path now queries the hot MemTable and multiple cold chunks in parallel using Rayon, merging results by distance.
* **Optional S3 Tiering (`hyperspace-tiering` crate)**:
    * **Cloud-Native Backend**: Cold chunks can now be offloaded to AWS S3, MinIO, or Ceph.
    * **LRU Disk Cache**: Byte-weighted cache (`moka`) manages local storage, automatically evicting cold chunks to S3 when `HS_MAX_LOCAL_CACHE_GB` is reached.
    * **Lazy Loading & Resilient I/O**: Automated S3 download on cache miss with exponential backoff and jitter-aware retries.
    * **Feature Gating**: All cloud dependencies are strictly optional via `s3-tiering` cargo feature, ensuring zero overhead for edge deployments.
* **Tiering Configuration**:
    * Added comprehensive S3 settings: `HS_STORAGE_BACKEND`, `HS_S3_BUCKET`, `HS_S3_REGION`, `HS_S3_ENDPOINT`, `HS_S3_MAX_RETRIES`, `HS_S3_UPLOAD_CONCURRENCY`.
* **Delta Sync Protocol (Task 2.1)**:
    * **Merkle Tree Bucket Sync**: Replaced linear replication with a O(1) lock-free 256-bucket XOR hash state tracker for granular structural diffing.
    * **Two-way gRPC Sync**: Added `SyncHandshake`, `SyncPull`, and `SyncPush` RPCs for efficient bilateral delta transfer (minimizing bandwidth to O(dirty_buckets)).
    * **HTTP Sync APIs**: Exposed `POST /api/collections/{name}/sync/handshake` and `/sync/pull` for WASM and REST clients.
    * **WASM Edge Sync**: Fully integrated synchronization into `hyperspace-wasm` (JS `.get_digest()`, `.apply_sync_vectors()`) with `IndexedDB` persistence for bucket hashes, enabling offline-first Edge-to-Cloud sync.
* **Peer-to-Peer (Edge-to-Edge) Gossip Swarm**:
    * **UDP Heartbeats**: Replaced centralized mesh logic with zero-dependency peer-to-peer UDP broadcasts. Nodes transmit state, role, and logical clocks (`tokio::net::UdpSocket`).
    * **Network Discovery**: `HS_GOSSIP_PEERS` enables dynamic cluster topologies without central coordinators. Stale peers are evicted via configurable `PEER_TTL` logic.
    * **Swarm Topology API**: `GET /api/swarm/peers` added for real-time visualization of the P2P Graph in the Dashboard.
* **Cognitive Math SDK & Heterogeneous Tribunal Framework**:
    * **Math Functions**: Implementations of `local_entropy`, `lyapunov_convergence`, `koopman_extrapolate` and `context_resonance` added to Python, TypeScript, Rust, and C++.
    * **Tribunal Router (`hyperspace.agents`)**: Added `TribunalContext` for evaluating LLM hallucination dynamically via geometric trust scores over the Graph Traversal API.
    * **Robotics Stack (ROS2)**: Initializing C++ and Go SDKs with gRPC pooling and Arena Serialization, alongside a ROS2 package `ros2_hyperspace_node` offering `NavigateToAttractor.srv`.

## [2.2.2] - 2026-02-19

### Added
* **Filtered search brute-force fallback**:
    * Added `HS_FILTER_BRUTEFORCE_THRESHOLD` routing: if filtered candidate bitmap is small, layer-0 performs exact brute-force scan instead of HNSW traversal.
    * Improves latency and recall stability on heavily filtered queries.
* **Hybrid search BM25 upgrade**:
    * Replaced token-overlap lexical scoring with BM25 (`idf`, document length normalization, per-document term frequency).
    * Added global lexical statistics in metadata index: token DF, per-document token length, term frequencies, and aggregate token length.
    * Hybrid lexical branch now respects the same filter constraints as vector search before RRF fusion.
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
