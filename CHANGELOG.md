# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
