# HyperspaceDB v4.2.5 — Refactoring Roadmap & Progress Tracker

## 🎯 High-Level Goals
1. **Thread-per-Core (Shared-Nothing Architecture)**: Transition from shared global `DashMap`/`RwLock` state to core-isolated shards (`ShardedCollection`) with core affinity and lock-free scatter-gather execution.
2. **Direct I/O Storage Engine**: Aligned block storage (4096-byte Direct I/O) bypassing OS page cache pollution and memory contention during heavy disk MRL cascade reads.
3. **Zero-Copy Byte SIMD & 8-Byte Packed Candidate Queues (v4.2.4)**:
   - 8-Byte packed candidate layout (`#[repr(C, align(8))] Candidate { id: u32, distance: f32 }`) turning priority queue comparisons into single 1-cycle integer `u64` instructions and cutting heap memory in half.
   - Zero-copy byte-to-byte SIMD distance kernels (`l2_sq_pair_dispatch`, `lorentz_f32_bytes_pair_thresh`, `hybrid_f32_bytes_pair`, `sq8_lorentz_bytes_pair`) directly comparing vectors in `VectorStore` during `select_neighbors` and `prune_connections`, eliminating up to 256 KB memory copies per node.
   - Fused SQ8 Lorentz dot-product and norm-square accumulation (`sq8_dot_and_qsq`), boosting deep radius ($t \approx 25$) Recall@10 from 31.5% to 93.5% with a 2.14× faster build.
   - KAG-Guided HNSW Traversal expanding CSR graph edges inside `search_layer0` to bridge semantic gaps in a single traversal pass.
4. **Hardware SIMD 801D Hybrid Engine & Numerically Stable Hyperbolic Kernels (v4.2.4)**:
   - Dedicated runtime SIMD dispatch for 801D Hybrid embeddings ($\mathbb{H}^{33} \times \mathbb{R}^{768}$) and `AsymmetricHybrid801` with cross-manifold early-abandon.
   - Numerically stable Lorentz difference identity $\cosh(d) = 1 + \frac{1}{2}(\|\Delta \mathbf{x}\|^2 - \Delta t^2)$ and fast Puiseux series expansion for `acosh`, eliminating cancellation and transcendental CPU overhead.
   - Zero-allocation candidate coordinate decoding and scoped `StorageView` graph traversals.
5. **1.58-Bit Ternary BitNet Quantization & Adaptive Filtered Search (v4.2.3)**:
   - 1.58-bit Ternary BitNet quantization $\{-1, 0, +1\}$ with adaptive Lloyd-Max optimal Gaussian threshold $\tau = 0.67 \cdot \text{mean}(|\mathbf{x}|)$ and dynamic projection scale $\alpha$.
   - Popcount-based SIMD dot product acceleration with zero-block skip logic, delivering 17.4 ns per 1024D vector (25.2× faster than f32) and 92.6% Recall@10 (10x rerank).
   - Adaptive filtered search cost model $\text{clamp}(1.75 \cdot \sqrt{N \cdot \text{ef} \cdot M}, 1000, N / 2)$ and Rayon-parallel chunked scanning, achieving 33,271 QPS at 5% selectivity (154× faster than OpenSearch 3.9).
   - Full support across Euclidean, Cosine, Poincaré, Lorentz, and Hybrid (801D) manifolds.
5. **Hardware SIMD SQ8 f32 Vector Kernels & Epoch Visited Lifecycle (v4.2.2)**:
   - Dynamic CPU SIMD dispatch (AVX-512, AVX2+FMA, ARM NEON, scalar) for `sq8_l2_sq` and `sq8_dot`, quantizing query vectors to f32 once per search and accelerating distance calculations up to 3.4×.
   - Epoch-based (`u16`) visited lifecycle eliminating per-query memory clearing overhead on multi-million node indexes.
   - Automatic snapshot freezing and zero-copy restoration of `FlatLayer0Segment` sidecars (`flat_l0.bin`).
   - Query vector validation zero-alloc fast paths and exact rerank buffer pipelining.
6. **Hardware-Level Bottleneck Elimination & Flat Navigation (v4.2.1)**:
   - Contiguous strided 64-byte cache-aligned Layer-0 graph layout (`FlatLayer0Segment`, `FlatNodeHeader`, `AlignedBuffer`) eliminating pointer chasing and triple-cache-miss lookups.
   - 64x memory-compact 1-bit Visited flat bitset (`words: Vec<u64>`), dropping scratch buffer footprint from 8 MB to 125 KB per thread for 1M nodes.
   - Lookahead $K=2$ software pipelining prefetch in Layer-0 and dynamic layer candidate traversal.
   - O(N) geometric scan → Axis-Aligned Bounding Box (AABB) spatial indexing.
7. **Zero-Downtime Automatic Migration (v3.x → v4.x)**: Automatic detection of legacy v3.x `.hyp` collections in `data/legacy_v3/` or `data/`, instant conversion to v4 sharded Direct I/O layout under `data/v4/`, and safe archiving to `data/legacy_v3_migrated/`. All v4.x collections maintain 100% forward and backward compatibility without manual migrations.
8. **Documentation & Benchmarks**: Complete synchronization of `ARCHITECTURE.md`, `CHANGELOG.md`, `README.md`, `SECURITY.md`, `CONTRIBUTING.md`.

---

## 📋 Task Checklist & Progress

- [x] **Phase 1: Spatial Indexing & Hot-path CPU Bottlenecks (`hyperspace-index`)**
  - [x] Implement `SpatialAabbIndex` / `AabbSegment` for pruning geometric filters (`InBall`, `InBox`, `InCone`).
  - [x] Integrate AABB pruning into `build_allowed_bitmap` (replacing O(N) vector fetches, discarding 95-99.9% non-matching candidates).
  - [x] Add hardware cache prefetch hints (`_mm_prefetch` on x86_64, `prfm pldl1keep` on ARM64) in `search_layer0` and `search_layer_candidates`.
  - [x] Upgrade `VisitedScratch` to 64-bit generation counter (`u64`), eliminating per-query resets.
  - [x] Unit & regression tests for Phase 1 (`cargo test -p hyperspace-index --lib`).

- [x] **Phase 2: Direct I/O & High-Performance Storage Engine (`hyperspace-store`)**
  - [x] Direct I/O aligned buffer (`AlignedBuffer`, 4096-byte boundary via `posix_memalign`).
  - [x] Direct I/O vector storage (`DirectVectorStore`, `DirectFile`) with cross-platform zero-copy block reads (`O_DIRECT` on Linux / `F_NOCACHE` on macOS).
  - [x] Page-aligned DMA buffer reading bypassing OS page cache double-buffering.
  - [x] Unit & round-trip tests for Phase 2 (`cargo test -p hyperspace-store --lib direct_io`).

- [x] **Phase 3: Thread-per-Core Sharded Engine (`hyperspace-server`)**
  - [x] `ShardedCollection<M>` implementing `hyperspace_core::Collection` with core-pinned partitions.
  - [x] Deterministic routing (`hash(id) % num_shards`) for point operations with zero lock contention.
  - [x] Lock-free scatter-gather parallel search merging candidate heaps in `O(K \log S)`.
  - [x] Unit & concurrency tests for Phase 3 (`cargo test -p hyperspace-server --lib sharded_engine`).

- [x] **Phase 4: Automatic Startup Migration Engine (v3.x → v4.x)**
  - [x] Migration detector for legacy v3 collections in `data/legacy_v3/` and root `data/`.
  - [x] Converter from v3 `chunk_*.hyp` & `payloads.hyp` to v4 sharded storage with AABB spatial indexes.
  - [x] Safe archiving to `data/legacy_v3_migrated/` with SHA-256 validation checksums (`migration_manifest.json`).
  - [x] Server startup hook in `start_server`.
  - [x] Unit & integration tests verifying automatic migration on startup (`cargo test -p hyperspace-server --lib migration`).

- [x] **Phase 5: Documentation, Metrics & Final Release**
  - [x] Update `ARCHITECTURE.md` with Seastar-inspired Thread-per-Core, Direct I/O, AABB tree, and v4 metrics.
  - [x] Update `CHANGELOG.md` with v4.0.0 release notes.
  - [x] Update `README.md` with v4.0.0 features, performance gains, and migration guide.
  - [x] Update `SECURITY.md` with v4.x policy, Direct I/O memory safety, and migration integrity.
  - [x] Full workspace validation (`cargo test --workspace --lib` passed with 0 failures).

- [x] **Phase 6: Production-Ready Hardening & Verification (100% COMPLETE)**
  - [x] **Raw Ingestion Throughput Optimization**:
    - [x] Fast path for single shard in `ShardedCollection::insert_batch` (15,319.4 vec/s).
    - [x] Unified Sequential Collection WAL for multi-shard collections: single sequential append with zero cross-inode lock contention.
    - [x] Lock-free `AtomicUsize` WriteBuffer counter eliminating DashMap shard read-lock iteration.
    - [x] Batched background HNSW graph indexer (up to 64 nodes per batch) eliminating per-node Arc cloning and atomic counter bouncing.
    - [x] Replace `tokio::spawn` with `futures::future::join_all` across shards for zero task-queue scheduling overhead.
    - [x] Intelligent default shard count (`cpu_cores.min(4)`).
  - [x] **WAL Crash Recovery & Replay Integrity**:
    - [x] Chaos Kill test (`kill -9`) during active high-throughput ingestion (0% data loss, 4,750 acknowledged vectors recovered from unified WAL with 100% consistency).
    - [x] Unified WAL CRC32 verification and automatic per-shard replay on boot.
  - [x] **Quantization & Codebook Persistence**:
    - [x] OPQ / PQ / Extreme cold-restart codebook reload test (`test_pq_custom_codebook_training`).
    - [x] ADC Rerank recall verification (100% Recall@10 on 801D/129D MRL SLM, 99.9% on 1024D extreme).
  - [x] **Memory Bounds & Tiered Storage**:
    - [x] MemTable threshold flush to Direct I/O chunks verification.
    - [x] Bounds verification on `raw_vector_cache` and `write_buffer`.
  - [x] **Mixed Read/Write Concurrency**:
    - [x] Search QPS / latency stability under active background streaming ingestion (476 queries, 0 errors, 0 deadlocks, P50 = 36.88 ms).
  - [x] **Comprehensive Benchmark Validation**:
    - [x] Re-run `run_v3_vs_v4_comparison.py` end-to-end (`BENCHMARK_V3_VS_V4_REPORT.md` updated).
    - [x] Re-run `run_quantization_sweep.py` end-to-end (`BENCHMARK_QUANTIZATION_SWEEP.md` generated with 27 configurations).

- [x] **Phase 7: v4.1.0 Hardware-Level Micro-Architecture Scaling & Bottleneck Elimination (100% COMPLETE)**
  - [x] **Dynamic Runtime SIMD Dispatch (`crates/hyperspace-core/src/simd.rs`)**:
    - [x] Multi-ISA CPU feature detection: AVX-512F (16xf32), AVX2+FMA (8xf32), ARM NEON (4xf32), Portable scalar.
    - [x] Direct unaligned little-endian float byte reading (`l2_sq_f32_bytes`) without memory copying.
    - [x] Integrated into HNSW inner search traversal loop (`dist_mrl_opt`) with early-exit distance thresholding per 64 floats.
    - [x] Controllable via `HS_SIMD=scalar|avx2|avx512|neon` environment variable.
  - [x] **Zero-Copy Inner Search Traversal Loop**:
    - [x] Implemented zero-copy `distance_bytes_*` methods across all quantization formats (`QuantizedHyperVector`, `BinaryHyperVector`, `HybridQuantizedVector`, `HybridLowBitQuantizedVector`, `HybridExtremeQuantizedVector`, `ScalarI4Vector`).
    - [x] Eliminates up to 50,000 heap allocations per second of `Vec<i8>`, `Vec<f32>`, and `HyperVector` during HNSW graph descent.
  - [x] **Asynchronous Linux `io_uring` Batched Positional Reads (`crates/hyperspace-store/src/uring_io.rs`)**:
    - [x] Single-syscall batch submission (`io_uring_enter`) for multi-id payload retrieval in `PayloadStore::fetch_many` and DirectVectorStore batch reads (`read_vectors_batch`).
    - [x] Transparent fallback to `read_exact_at` loops on macOS, BSD, Windows, or non-io_uring Linux kernels. Controllable via `HS_IO_URING=0`.
  - [x] **Multi-Socket NUMA-Aware Topology & Core Affinity (`crates/hyperspace-server/src/numa.rs`)**:
    - [x] Automated detection of NUMA node socket topology via `/sys/devices/system/node` filtered by active process cgroup cpuset.
    - [x] Round-robin socket affinity pinning for tokio runtime worker and blocking threads to guarantee first-touch memory allocation locality. Controllable via `HS_NUMA=0`.
  - [x] **32-Stripe Sharded Payload Index (`ShardedPayloadIndex`)**:
    - [x] Replaced monolithic global `RwLock<HashMap<u32, PayloadSlot>>` with 32 striped reader/writer locks, eliminating cacheline bouncing under concurrent payload read access.
  - [x] **Hoisted PQ LUT & Bounded Min/Max Heap**:
    - [x] Hoisted Product Quantization lookup table computation in `search_bruteforce_bitmap` and replaced full-array $O(M \log M)$ sorting with a bounded $O(M \log k)$ `BinaryHeap<MaxCandidate>`.
  - [x] **Mathematical Kernels & Graph Engine Hardening**:
    - [x] Vectorized Gromov hyperbolicity evaluation with `as_chunks::<4>()` and direct branchless max comparisons.
    - [x] Static `OnceLock` Fast Walsh-Hadamard Transform plans and unrolled butterfly loops in `TurboRot`.
    - [x] Fast sequential path for short sparse BM25 queries ($\le 8$ tokens) eliminating Rayon thread dispatch overhead.
    - [x] Geometrically pre-allocated adjacency capacity in `DynamicGraph::ensure_node_capacity`.

- [x] **Phase 8: Zero-Knowledge Client Encryption, Vec2Text Defense & Autonomous Agent Memory (v4.1.2 - 100% COMPLETE)**
  - [x] **Client-Side Isometric Orthogonal & Lorentz Projection**:
    - [x] Symmetric QR Householder orthogonal matrix $O(n)$ projection for Euclidean and Cosine manifolds.
    - [x] Proper Lorentz matrix $SO^+(1, d)$ projection preserving Minkowski pseudo-metric $ds^2 = -dt^2 + \sum dx_i^2$ for hyperbolic manifolds.
    - [x] Rigorous mathematical verification: 100.0% Recall@10 preservation and zero accuracy loss ($1.000000$ distance correlation).
  - [x] **Vec2Text Inversion Resistance & Anisotropic Noise Injection**:
    - [x] Keyed PRF anisotropic noise $\vec{n} = \sigma \cdot \mathbf{P}_\parallel(\vec{v}) + \frac{\sigma}{4} \cdot \mathbf{P}_\perp(\vec{v})$ aligned with feature covariance.
    - [x] Empirical inversion suppression: Vec2Text cross-cosine collapses from $0.984$ (unencrypted plaintext) to $0.052$ (orthogonal) and $0.038$ (orthogonal + noise).
  - [x] **Client-Side Payload Cryptography & Schema Metadata**:
    - [x] Authenticated AES-256-GCM chunk payload encryption before gRPC/HTTP wire transmission.
    - [x] Per-collection `encrypted: bool` opt-in flag in protobuf schema, `meta.json`, and collection stats API.
    - [x] UI dashboard security indicator (`ENCRYPTED` badge & Lock icon).
  - [x] **Autonomous Agent Cognitive Memory (Zero LLM Dependency)**:
    - [x] Ebbinghaus temporal exponential forgetting curve: $S(t) = S_0 \cdot \exp(-\lambda \Delta t)$ computed dynamically at search time.
    - [x] Temporal knowledge superseding via directed `SUPERSEDES` graph edges, avoiding contradictory recall without external LLM consensus.
    - [x] Sub-4ms ingestion & recall latency ($2.11 \text{ ms}$ ingest, $3.84 \text{ ms}$ search) vs $1200\text{--}2400 \text{ ms}$ for LLM-based solutions (Mem0/Zep).
  - [x] **Complete Multi-SDK & Ecosystem Integration**:
    - [x] Python, TypeScript, Go, C++, Dart, ROS2 SDKs.
    - [x] `hyperspace-memory-py`, `hyperspace-memory-ts`, `mcp-hyperspace-memory`, `mcp-hyperspacedb`.
    - [x] LangChain (Python/JS), LlamaIndex (Python/JS), and n8n community node.

- [x] **Phase 9: v4.2.0 `pgrx`-Style Extension Architecture (`hyperspace-pdk`), 12 Native SIMD Metrics & Modular External Plugins (100% COMPLETE)**
  - [x] **Hyperspace Plugin Development Kit (`crates/hyperspace-pdk`)**:
    - [x] `HsPlugin` trait with pipeline hooks (`BeforeInsert`, `AfterInsert`, `BeforeSearch`, `AfterSearch`, `OnLink`) and JSON-Schema self-describing procedures (`PluginManifest`, `ProcedureSchema`).
    - [x] `hs_export_plugin!` C-ABI export macro (`hs_pdk_api_version` + `hs_plugin_create`) and `libloading` dynamic shared library loader (`.dylib` / `.so` / `.dll`) scanning `HYPERSPACE_PLUGINS_DIR` (`./plugins`).
    - [x] Zero-downtime runtime activation/deactivation (`POST /v1/extensions/{plugin}/enable|disable`) and hot-reload (`POST /v1/extensions/reload`).
    - [x] Interactive Dashboard Workbench (`/plugins` in `PluginsPage.tsx`) for managing plugins, testing procedures live, and inspecting SDK snippets.
  - [x] **Built-in & External Flagship Extensions**:
    - [x] **Built-in `hs-embed` (`embed`)**: Full unification of `crates/hyperspace-embed` (ONNX Runtime, HuggingFace Hub, OpenAI/Cohere/Voyage/Mistral/YarInk remote APIs, `.env` per-metric routing, Markdown chunker, Sidecar Payload storage, and KAG sequential/hierarchical graph linking) into `crates/hyperspace-pdk/src/embed.rs`; duplicate `crates/hyperspace-embed` removed.
    - [x] **Built-in `hs-shield` (`shield`)**: Server-side Zero-Knowledge Orthogonal Projection ($O(D \log D)$ keyed Fast Walsh-Hadamard Transform + SHA-256 Rademacher sign flips + Fisher-Yates permutation) complementing DePIN client-side encryption.
    - [x] **External `hs-git-ast` (`plugins/hs-git-ast`)**: Standalone `cdylib` plugin extracting AST symbols and KAG topology (`CONTAINS`, `CALLS`, `IMPORTS`, `IMPLEMENTS`) across Rust, Python, TypeScript/JavaScript, and Go with zero LLM cost.
    - [x] **External `hs-eco` (`plugins/hs-eco`)**: Standalone `cdylib` plugin for Carbon & ESG Telemetry (`metrics`, `esg_report`, `collection_tier`), replacing `crates/hyperspace-eco` and removing `#![cfg(feature = "eco-monitor")]` from the core server.
  - [x] **12 Universal Hardware-SIMD Vector Metrics & Engine Bottleneck Fixes**:
    - [x] Added native hardware-SIMD metrics: `Manhattan` ($L_1$), `Chebyshev` ($L_\infty$), `Hamming`, `Jaccard` (Tanimoto), `Angular` (Geodesic Arccos), `JensenShannon` (JSD), and `Hellinger` alongside `Euclidean`, `Cosine`, `DotProduct`, `Poincare`, `Lorentz`, and `Hybrid`.
    - [x] Fixed unbounded memory growth in `ShardedCollection::raw_vector_cache` via bounded 64-shard clock-hand eviction, eliminated redundant vector clones in HNSW insertion, and removed blocking `std::sync::Mutex` in `DirectFile::write_all_at`.

- [x] **Phase 8: Flat Navigation & Data-Oriented Memory Layout (v4.2.1 - 100% COMPLETE)**
  - [x] **Contiguous Strided Layer-0 Layout (`FlatLayer0Segment`, `FlatNodeHeader`, `AlignedBuffer`)**:
    - [x] Co-locate 64-byte node header (`neighbor_count`, `flags`, inline `neighbors: [u32; 14]`) and quantized vector payload in contiguous memory.
    - [x] Zero pointer-chasing: direct offset addressing $\text{offset} = \text{node\_id} \times \text{stride}$ with 64-byte cache line alignment.
    - [x] Zero-copy `mmap` backing via `memmap2::Mmap` alongside heap-allocated `AlignedBuffer`.
    - [x] Auto-compaction & snapshot integration: `flat_l0.bin` sidecar persisted in `.hyp` chunk snapshots and loaded automatically.
  - [x] **64x Compact Visited Flat Bitset**:
    - [x] Replaced `VisitedScratch.marks: Vec<u64>` (8 MB per thread for 1M nodes) with 1-bit flat bitset `words: Vec<u64>` (125 KB per thread).
    - [x] Entire visited scratch fits comfortably in L2 CPU cache, dropping L2/L3 cache misses and improving query QPS by +3%.
  - [x] **Two-Phase Lookahead Software Pipelining Prefetch ($K=2$)**:
    - [x] Prefetch candidates $i+2$ ahead in Layer-0 and dynamic layer traversals (`_mm_prefetch` / `prfm`).
    - [x] Validated optimal sweet spot: +9.6% QPS gain on large graphs without cache pollution.
  - [x] **Performance & Quality Validation**:
    - [x] +26.5% QPS boost for 1-bit Extreme quantization (21,362 → 27,024 QPS), 2.3x P99 tail latency drop (169.6 µs → 72.9 µs).
    - [x] +14.3% QPS boost for 8-bit Turbo quantization (3,030 → 3,463 QPS on 150k nodes).
    - [x] 100% search result equivalence between dynamic pointer graph and flat strided graph.
    - [x] Zero compilation warnings (`cargo clippy --all-targets --all-features -- -D warnings`), 100% passing tests.

- [x] **Phase 10: Hardware SIMD SQ8 f32 Vector Kernels, Epoch Visited-Set Lifecycle & Zero-Cost Persistence (v4.2.2 - 100% COMPLETE)**
  - [x] **Runtime SIMD SQ8 f32 Vector Distance Kernels**:
    - [x] `sq8_l2_sq` and `sq8_dot` implemented with dynamic runtime CPU detection (AVX-512, AVX2+FMA, ARM NEON, scalar fallback).
    - [x] Quantize query vector to `f32` once per search query in `search_layer0` and upper layers.
    - [x] Integrated into `distance_bytes_euclidean_sq_f32`, `distance_bytes_poincare_sq_f32`, and `distance_bytes_lorentz_f32`, providing up to 3.4× kernel speedup.
  - [x] **Epoch-Based Visited-Set Lifecycle**:
    - [x] Replaced per-query `fill(0)` on `VisitedScratch` with monotonic `u16` generation epochs (`epoch += 1`).
    - [x] Added zero-cost wraparound clearing with epoch reset when `epoch == 0`.
    - [x] Completely removed query latency scaling on large indices (up to 70× faster visited reset on 50M+ node graphs).
  - [x] **Auto-Freezing Flat Layer 0 & Snapshot Persistence**:
    - [x] Automatic freezing of `FlatLayer0Segment` during snapshot creation without manual compaction triggers.
    - [x] Safe zero-copy loading on index restart; transparent fallback to dynamic graph for pending insertions.
  - [x] **Filter Pruning & Exact Rerank Optimization**:
    - [x] Direct bypass of `build_allowed_bitmap` on empty filter expressions.
    - [x] Fast short-circuiting on tombstones when `metadata.has_deletions()` is false.
    - [x] Single-pass query validation before allocating query structures.
    - [x] Direct slice buffer reuse in `collection.rs` exact rerank pipeline, eliminating intermediate allocations and memcpy overhead.
  - [x] **Docker BuildKit Acceleration & CI Hardening**:
    - [x] `--mount=type=cache` added for Cargo registry and git directories across Dockerfiles.
    - [x] Default compilation target set to `x86-64-v3` for stable multi-CPU Linux deployments.
    - [x] 100% backward and forward compatibility with existing v4 collections (zero manual migrations required).

- [x] **Phase 11: 1.58-Bit Ternary BitNet Quantization & Adaptive Filtered Scanning (v4.2.3 - 100% COMPLETE)**
  - [x] Ternary vector quantization $\{-1, 0, +1\}$ with adaptive Lloyd-Max optimal Gaussian threshold $\tau = 0.67 \cdot \text{mean}(|\mathbf{x}|)$ and dynamic projection scale $\alpha$.
  - [x] Bit-packed storage ($2 \cdot \lceil D/8 \rceil + 8$ bytes) delivering 15.5× compression vs FP32 and 17.4 ns distance evaluation (25.2× faster).
  - [x] Cost-model adaptive filtered search $\text{clamp}(1.75 \cdot \sqrt{N \cdot \text{ef} \cdot M}, 1000, N / 2)$ with Rayon parallel chunked scanner (33,271 QPS at 5% selectivity, 154× faster than OpenSearch 3.9).

- [x] **Phase 12: Hardware SIMD 801D Hybrid Engine, Stable Lorentz Difference & Zero-Alloc Traversals (v4.2.4 - 100% COMPLETE)**
  - [x] **801D Hybrid SIMD Vectorization & Cross-Manifold Early Abandon**:
    - [x] Added `hybrid_f32_bytes` with 32-lane spatial SIMD and cross-manifold early-abandon after evaluating the 33D hyperbolic head.
    - [x] Added `HybridQuantizedVector::distance_bytes_mrl_f32` with vectorized `simd::sq8_l2_sq` for 768D tail.
    - [x] Connected `"hybrid"` and `AsymmetricHybrid801` to `dist_bytes_opt` and `dist_f32_slices`.
    - [x] 801D Hybrid build accelerated 8.54× (1379.9 ms → 161.6 ms), search QPS boosted 17.41× (1,910 → 33,249 QPS, 100% Top-1 recall).
    - [x] `AsymmetricHybrid801` build accelerated 8.47× (1500.0 ms → 177.0 ms), search QPS boosted 14.20× (2,069 → 29,382 QPS).
  - [x] **Numerically Stable Lorentz Difference & Puiseux `acosh`**:
    - [x] Stable difference identity $\cosh(d) = 1 + \frac{1}{2}(\|\Delta \mathbf{x}\|^2 - \Delta t^2)$ in `lorentz_f32_bytes_thresh`.
    - [x] 4-term Puiseux polynomial for small hyperbolic distances $x < 0.0625$ in `acosh_from_cosh`.
    - [x] 33D Lorentz build time reduced 1.50× (75.28 ms → 50.16 ms), search QPS reaches 98,585 with 100% recall (200/200).
  - [x] **Zero-Allocation Node Decoding & Scoped StorageView**:
    - [x] Zero-allocation decoders in `load_node_coords_f32` for `AsymmetricHybrid801` and `ProductQuantization`/`OPQ`.
    - [x] Scoped `StorageView` across `select_neighbors`, `prune_connections`, and greedy upper-layer zoom-in loops.
    - [x] Bounds-free Product Quantization ADC `distance_bytes_thresh` with 50% early-abandon.
    - [x] WriteBuffer bypass in `collection.rs` on empty buffers.

- [x] **Phase 13: Zero-Copy Byte SIMD, 8B Packed Candidates & KAG Beam Routing (v4.2.5 - 100% COMPLETE)**
  - [x] **8-Byte Packed Candidates (`Candidate` / `MaxCandidate`)**:
    - [x] Packed structure `#[repr(C, align(8))]` with single `u64` key comparisons (v4.2.5 audit: `transmute` replaced by an order-preserving `f32 → u32` key, valid for negative distances).
    - [x] 50% memory reduction in `BinaryHeap` queues (8B vs 16B) with 2x cacheline density.
  - [x] **Zero-Copy Byte-to-Byte SIMD Kernels**:
    - [x] Added `l2_sq_pair_neon`, `l2_sq_pair_avx2`, `l2_sq_pair_dispatch`, `l2_sq_f32_bytes_pair_adsampling`, `lorentz_f32_bytes_pair_thresh`, `hybrid_f32_bytes_pair`, and `sq8_lorentz_bytes_pair`.
    - [x] Direct vector comparisons in `select_neighbors` and `prune_connections` without `load_node_coords_f32` unpacking or intermediate `memcpy` (saving up to 256 KB memory copy per node in 1024D).
    - [x] Inner loop candidate pruning with tight `curr_worst` updating.
  - [x] **Optimized 2-Point ADSampling**:
    - [x] Checkpoints at 50% and 75% dimensions with constant slack bounds (`0.65 * t` and `0.92 * t`) without runtime `sqrt` overhead.
    - [x] v4.2.5 audit: the statistical slack is only valid on randomly rotated data; unrotated storage now uses the exact lower bound (`simd::EXACT_EARLY_ABANDON`) to stop discarding true neighbors on MRL embeddings.
  - [x] **Fused SQ8 Lorentz Kernel & Integer Dot Accumulation (`sq8_dot_and_qsq`)**:
    - [x] Direct integer accumulation (`vmull_s8` + `vmlal_s8` + `vpadalq_s16`) cutting FP FMA ops in half.
    - [x] Boosted 128D Lorentz ScalarI8 deep radius ($t \approx 25$) Recall@10 from 31.5% to 93.5% with 2.14× faster build.
  - [x] **KAG-Guided HNSW Traversal**:
    - [x] Integrated `HnswIndex::set_graph_engine` / `with_graph_engine` to expand active KAG CSR edges inside `search_layer0`.
    - [x] Verified semantic gap bridging (Node 10 -> KAG -> Node 450 in Top-5).
  - [x] **Lazy `SpatialAabbIndex` & Zero-Heap Scratch**:
    - [x] `indexed_count: AtomicU32` and `ensure_spatial_index()` bypassing spatial index lock overhead on pure vector inserts.
    - [x] Thread-local `BYTES_SCRATCH` and `build_hyper_vector` eliminating heap allocations on ingestion/search paths.

---

- [x] **Phase 14: Performance & Recall Audit — Quantizers, Flat Graph, Durability (v4.2.5 - 100% COMPLETE)**
  - [x] **Cold chunks**: process-wide LRU of loaded chunk indexes (`HS_CHUNK_CACHE_SIZE`), `chunk.manifest` (layout + local→global IDs), correct MemTable/chunk merge (no `u32::MAX` sentinels), single `storage_element_size` for writer and reader.
  - [x] **Semantic compaction without data loss**: re-ingests chunks + MemTable, deletes old chunks only after the new ones are written.
  - [x] **Filters**: order-preserving `f64` numeric range index (typed values included, no query-time parsing); `Not` via `insert_range`; ACORN-style 2-hop traversal for selective filters (`HS_FILTER_ACORN`).
  - [x] **Recall fixes**: exact early abandon on unrotated data, signed candidate keys (`dot`), `cosh − 1` bias for hyperbolic candidates, frozen flat L0 used only while it covers every live node, metric-consistent KAG scores, full top-k from the L2-ANN cache, `ef_search` applied on the inline path, full per-shard `ef_search` in scatter-gather search (4.2.4 divided it by √shards: recall@10 92.9% → 99.0%).
  - [x] **Hot loop**: per-query `DistKernel` dispatch; flat seqlocked Layer-0 arena (`layer0.rs`) with next-slot prefetch; multi-line vector prefetch; zero-copy `f64` kernels; AVX-512 pair kernel; adaptive pruning (eager with SIMD pair kernels, 25% slack for decode-heavy modes); standard `1/M` level distribution.
  - [x] **Hybrid metric**: internal squared distance (`hybrid_sq_f32_bytes`), `acosh²` series without `ln` / `sqrt` near the origin, `sqrt` applied only on output.
  - [x] **New quantizers**: `RaBitQ` (1-bit + unbiased popcount estimator, trained centroid) and `PQ4` fast-scan (4-bit PQ, L1-resident `u8` LUT, batched `scan_block`); calibrated by index rebuild; exposed in the server, dashboard and all SDKs.
  - [x] **Graph maintenance**: FreshDiskANN-style `repair_deleted` (α = 1.2, reverse edges, entry-point recovery), triggered automatically after deletes; Vamana `HS_HNSW_ALPHA`; adaptive-ef patience (`HS_SEARCH_PATIENCE`).
  - [x] **Durability & I/O**: WAL group commit (fsync off the WAL lock, background batch syncer); `HS_MMAP_WILLNEED` / `HS_MMAP_HUGEPAGES`; atomic flat-L0 snapshot writes (fixes an unkillable kernel deadlock after rebuild on macOS).
  - [x] **Wire & build**: `vector_f32` compact vectors in proto + every SDK (`compact_vectors`); PGO build (`scripts/pgo_build.sh`, `just build-pgo`).
  - [x] **Allocations**: `Arc`-shared metadata between write buffer and indexer queue; single f64→f32 conversion on quantized inserts.
  - [x] **Online calibration**: trained quantizers fit on exact vectors at `HS_AUTO_CALIBRATE_MIN` (4,096 for PQ / OPQ), re-fit after 8× growth, codes re-encoded in place under a calibration gate; `quantizer_calibrated` in stats.
  - [x] **Per-request `ef_search`** in proto, gRPC, HTTP, all SDKs, ROS2 and MCP (`HS_MAX_EF_SEARCH` guard); `ConfigUpdate` ef now honored by searches.
  - [x] **Kernels**: `extreme` / `ternary` nibble tables (×11–×20), `medium_plus` NEON block kernel (×7).
  - [x] **CI**: SDK proto drift check, offline SDK request tests, Dart live-server tests, recall gate on the quantization sweep.
  - [x] **Flat scan for small shards** (cost model `ef · 2M`), metadata after merge, no redundant rerank for full precision, per-collection cache budgets (`HS_RAW_CACHE_MB`, hot-tier L1); fixed double distance finalization in filtered brute force and stale mmap segments after repair.
  - [x] **Integer SQ8**: per-vector scaled `medium` codes + int8 `SDOT` / NEON / AVX2 kernel (scan ×3.5, recall 88.6% → 98.5%), marker-gated format (`sq8.scaled`).
  - [x] **Two-stage search by default**: rerank once on the merged scatter-gather candidates (`HS_RERANK_DEFER`), memory-mapped exact store (`exact_f32/`), rerank on by default; fixed exact copies missing for batch inserts and the payload / exact-vector key collision.
  - [x] **Streaming `RebuildIndex`**: exact vectors streamed from the exact store, parallel graph build, BFS renumbering, all id-keyed structures remapped and persisted (fixes wrong vectors / metadata after rebuilds with deletions).
  - [x] **`hs_loadbench`**: concurrent gRPC load benchmark (QPS, tail latency, recall, CPU/query, `SearchBatch`, rebuild timing + peak RSS).
  - [ ] Rebuild peak memory is dominated by the second graph (`RebuildIndex` builds with M = 128): build in place or with the collection's M.
  - [ ] f16 storage for full precision — deferred: `medium` + exact rerank (100% recall, 9.0k QPS on 20k × 1024-D at 16 clients) already beats what f16 can reach for `none` (bandwidth-bound, ≈2× of 2.8–3.4k QPS), and f16 would make `none` inexact.
  - [ ] Configurable PQ sub-space count: needs the PQ `M` stored in collection metadata and threaded through every storage-layout site (22) — use `pq4` for higher recall at ~2× the bytes meanwhile.
  - [x] ~~Cooperative shard pruning~~: evaluated, rejected (a global k-th bound stops HNSW traversal before it reaches near nodes; 46% top-10 agreement).

## 📈 Metric Goals & Tracking
| Metric | v3.x Baseline | v4.0.0 Target | Current Status |
|---|---|---|---|
| **Raw Vector Ingestion (1024D)** | 10,932.9 vec/s | > 10,000 vec/s | **15,319.4 vec/s** (1 shard) / **67,500 vec/s** (Server core rate) :white_check_mark: |
| **Total Index Build Time (25k 1024D)** | 121.98 s | < 80 s | **78.54 s** (1.55× faster) :white_check_mark: |
| **1024D L2 Build Time (1500 vec)** | 436.29 ms | < 350 ms | **304.75–323.46 ms** (26–30% faster) :white_check_mark: |
| **1024D L2 Frozen Search QPS** | 7,964.4 QPS | > 8,000 QPS | **8,355–8,712 QPS** (flat L0 active) :white_check_mark: |
| **801D Hybrid f32 Build Time (1500 vec)** | 406.14 ms | < 350 ms | **290.98 ms** (28.4% faster) :white_check_mark: |
| **801D Hybrid f32 Search QPS (ef=48)** | 1,910 QPS | > 20,000 QPS | **33,249 QPS** (17.41× higher QPS, 100% Recall) :white_check_mark: |
| **801D AsymmetricHybrid801 Search QPS** | 2,069 QPS | > 20,000 QPS | **29,382 QPS** (14.20× higher QPS, 100% Recall) :white_check_mark: |
| **128D Lorentz f32 Build Time (2000 vec)** | 183.38 ms | < 160 ms | **146.10–151.63 ms** (17–20% faster) :white_check_mark: |
| **128D Lorentz f32 Search QPS** | 21,139 QPS | > 20,000 QPS | **21,743–22,608 QPS** (0.980 Recall@10) :white_check_mark: |
| **128D Lorentz SQ8 Deep Radius Build** | 48.0 ms | < 30 ms | **22.11 ms** (2.14× faster build) :white_check_mark: |
| **128D Lorentz SQ8 Deep Radius Recall** | 31.5% | > 90.0% | **93.5%** (~80,000 QPS, +62.0% Recall gain) :white_check_mark: |
| **KAG Semantic-Gap Traversal** | Structural gap | Bridge in Top-5 | **100% Connected** (Node 10 -> Node 450 in Top-5) :white_check_mark: |
| **33D Lorentz f32 Search QPS (ef=48)** | 40,127 QPS | > 75,000 QPS | **98,585 QPS** (200/200 Top-1 Recall) :white_check_mark: |
| **Geometric Filter Latency (1M)** | ~120 - 450 ms (O(N) full scan) | < 10 ms (AABB pruning) | **0.51 ms** (1,774 QPS, 22.3× faster) :white_check_mark: |
| **Dense 1024D Search QPS (C=1)** | 31.5 QPS (P50 30.2 ms) | > 100 QPS | **142.5 QPS** (P50 6.98 ms, 4.52× higher QPS, 99.90% Recall) :white_check_mark: |
| **MRL Cascade Disk Rerank QPS** | 64.5 QPS (P50 15.0 ms) | > 200 QPS | **397.1 QPS** (P50 2.50 ms, 6.16× higher QPS, 99.87% Recall) :white_check_mark: |
| **Search Concurrency C=10 / C=30** | 238.4 / 290.0 QPS | > 300 QPS | **311.6 / 296.1 QPS** (P99 reduced from 137.6ms to 99.2ms) :white_check_mark: |
| **HNSW Visited Reset Pause** | Periodic spike on wrap | 0.00 µs (64-bit gen) | **0.00 µs** (Monotonic u64) :white_check_mark: |
| **Cross-Core Lock Contention** | 280-450 ns (RwLock) | 0 ns (Thread-per-core) | **0 ns** (Shared-Nothing) :white_check_mark: |
| **Legacy v3 Migration** | Manual / None | 100% Automatic on boot | **100% Autonomous** (`MigrationEngine`) :white_check_mark: |
| **ROMASHKA SLM 801D/129D Recall** | ~99.7% | 99.9%+ | **100.0%** (ADC Rerank ×4) :white_check_mark: |
| **Encrypted Vector Recall@10** | N/A | 100.0% | **100.0%** (Exact Isometric Invariance) :white_check_mark: |
| **Vec2Text Inversion Resistance** | 0.984 (Vulnerable) | < 0.100 (Unrecoverable) | **0.038** (Cosine Similarity to plaintext) :white_check_mark: |
| **Autonomous Memory Latency (P50)**| > 1,200 ms (LLM-based) | < 10 ms | **2.11 ms ingest / 3.84 ms search** :white_check_mark: |
| **v4.2.5 Sweep: 1024D search QPS (32 configs)** | v4.2.4 baseline | ≥ v4.2.4, no recall loss | **×2.16 geomean**, recall@10 +1.8 pp avg (`none` 92.90% → 100%) :white_check_mark: |
| **v4.2.5 Sweep: ingest throughput** | v4.2.4 baseline | ≥ v4.2.4 | **×1.44 median** (`extreme` 5,169 → 11,409 vec/s) :white_check_mark: |
| **v4.2.5 Low-bit search (ternary / extreme / medium_plus)** | 97 / 811 / 597 QPS | ≥ 2× | **1,337 / 1,982 / 1,607 QPS** :white_check_mark: |
| **v4.2.5 RaBitQ 1024D (rerank ×4)** | N/A | > 95% recall, > 1,000 QPS | **98.55%** @ **1,576 QPS** (29× compression) :white_check_mark: |
| **v4.2.5 PQ4 fast-scan 1024D (rerank ×8)** | N/A | > 99% recall | **99.75%** @ **1,112 QPS** :white_check_mark: |

> Rows above are measured (Apple Silicon, single node) — see `BENCHMARK_V3_VS_V4_REPORT.md`, `BENCHMARK_QUANTIZATION_SWEEP.md`, and `HONEST_CAPABILITY_AND_BENCHMARK_REPORT.md`.

