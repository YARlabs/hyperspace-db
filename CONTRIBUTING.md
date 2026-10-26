# Contributing to HyperspaceDB

By submitting contributions to HyperspaceDB, you agree that your contributions will be licensed under the project's MIT License.

## 🛠 Development Setup

1.  **Toolchain**: Install Nightly Rust:
    ```bash
    rustup toolchain install nightly
    rustup default nightly
    ```
2.  **Helpers**: We use `just` for task management:
    ```bash
    cargo install just
    ```
3.  **Build**:
    ```bash
    just build
    ```

4.  **Python SDK**:
    ```bash
    cd sdks/python
    python3 -m venv venv
    source venv/bin/activate
    pip install grpcio-tools grpcio protobuf
    ./generate_protos.sh
    ```

5.  **TypeScript SDK**:
    ```bash
    cd sdks/ts
    npm install
    npm run build
    ```

## 🧪 Testing

We value stability. Please ensure all tests pass before submitting a PR:
```bash
cargo check --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib
# Test native clustering and graph benchmarks
cargo test -p hyperspace-clustering
cargo bench --bench graph_vs_dual --no-run
```

## 📜 Code Style

We follow standard Rust formatting:
```bash
cargo fmt --all
cargo clippy --all-features -- -D warnings
cargo clippy --tests --workspace -- -W clippy::pedantic
```

## 📐 Adding New Metrics

To implement a new metric:
1.  Implement `Metric<N>` trait in `crates/hyperspace-core/src/lib.rs`.
2.  Implement `distance`, `validate`, and quantized distance methods (`distance_quantized`, `distance_binary`).
3.  Register alias and instantiation logic in `crates/hyperspace-server/src/manager.rs`.
4.  Add unit/integration tests in `crates/hyperspace-core/src/tests.rs` and related crates.

## 🧠 Cognitive SDK Development

If you are contributing to the **Cognitive Math SDK** or the **Heterogeneous Tribunal Framework**:
1. Add core math to `crates/hyperspace-sdk/src/math.rs` in Rust.
2. Mirror the functionality to Python (`sdks/python/hyperspace/math.py`) and TypeScript (`sdks/ts/src/math.ts`).
3. If adding a new multi-agent evaluation metric (e.g. `evaluate_claim` for the Tribunal Router), ensure it leverages the Graph Traversal API efficiently and is added symmetrically across the `agents` module in all SDKs.

## 🕸️ Contributing to the Native Vector-Graph (KAG) Subsystem

If you are extending the **Native Vector-Graph (KAG)** engine:
1. **Core Data Structures** (`crates/hyperspace-core/src/graph.rs`):
   - `CompactEdge`: 12-byte packed edge structure (`target_node: u32`, `relation: u16`, `weight: f32`). Always maintain 12-byte struct alignment.
   - `DynamicGraph`: Fine-grained concurrent adjacency list used for ingestion and dynamic edge updates.
   - `CsrGraph`: Static, read-optimized Compressed Sparse Row layout for contiguous L1/L2 cache prefetching and sub-microsecond reachability queries.
2. **Adding Metadata Induction Rules**:
   - New declarative rules belong in `GraphEngine::induce_from_metadata` in `crates/hyperspace-core/src/graph.rs`.
   - Ensure rules are bidirectional when symmetric, and respect fanout bounds (e.g. max 16 clique neighbors for `exact_match`) to avoid quadratic edge explosions.
3. **Thread-Per-Core Integration** (`crates/hyperspace-server/src/sharded_engine.rs`):
   - Every `ShardedCollection<M>` maintains an `Arc<GraphEngine>`.
   - Always translate user 32-bit hash IDs through `self.to_graph_id(user_id)` and `self.to_user_id(graph_id)` (`DashMap<u32, u32>`) to ensure dense $0 \dots N$ CSR arrays without allocating gigabytes of empty row offsets.
   - Changes must persist to `graph_id_map.json` and `graph.csr` upon collection flush.
4. **Validation**:
   - Run graph unit tests: `cargo test -p hyperspace-core --lib graph`
   - Run empirical benchmark: `python3 benchmarks/run_all_collections_kag_benchmark.py --rounds 10`

## ⚡ Hardware Acceleration & SIMD Development

If you are contributing to low-level mathematical kernels or storage acceleration:
1. **Dynamic SIMD Kernels (`crates/hyperspace-core/src/simd.rs`)**:
   - Explicit target features: Use `#[target_feature(enable = "avx512f")]`, `#[target_feature(enable = "avx2,fma")]`, and `#[target_feature(enable = "neon")]`.
   - Never assume pointer alignment: always use unaligned vector loads (`_mm512_loadu_ps`, `_mm256_loadu_ps`, `vld1q_f32`) so distance kernels can run directly over unaligned slices of memory-mapped or RAM storage.
   - Dual/Quad accumulators: Unroll loops by at least 2 to 4 vector registers to break latency chains and maximize instruction-level parallelism (FMA pipelining).
   - Support `HS_SIMD` override: All dispatch logic must respect `detect()` and runtime override for A/B testing (`HS_SIMD=scalar|avx2|avx512|neon`).
2. **Batched Positional I/O (`crates/hyperspace-store/src/uring_io.rs`)**:
   - The `io-uring` feature is Linux-only. All callers must use `read_batch_at` or `read_raw_batch_at` with zero-cost fallback to `pread` on other platforms.
   - Buffer lifetimes: Memory passed to submission rings must remain valid until completions are reaped (`submit_and_wait`).
3. **Multi-Socket NUMA Placement (`crates/hyperspace-server/src/numa.rs`)**:
   - Always filter detected socket CPU lists against the active process affinity set (`sched_getaffinity`) to respect container cpusets and `taskset`.
   - Thread placement must be configurable via `HS_NUMA=0`.

## 📊 Adding Quantization Modes

To implement a new quantization mode:
1. Add a variant to `QuantizationMode` enum in `crates/hyperspace-core/src/lib.rs`.
2. Implement encoder in `crates/hyperspace-core/src/vector.rs` (see `QuantizedHyperVector::from_float()` for the SQ8 Anisotropic reference — it uses coordinate-descent refinement with an anisotropic loss $L = \|e_\parallel\|^2 + t_w \cdot \|e_\perp\|^2$).
3. Implement a corresponding zero-copy `distance_bytes_*` method on raw byte slices to avoid heap allocations in the HNSW beam traversal loop.
4. Wire the new variant in `CollectionMetadata::quantization_mode()` in `crates/hyperspace-server/src/manager.rs`.
5. Update `HS_QUANTIZATION_LEVEL` docs in `docs/book/src/quantization.md`.

## 🤖 Adding Embedding Providers

To add a new embedding provider to the built-in service:
1. Implement the `Embedder` trait in `crates/hyperspace-embed/src/lib.rs`.
2. Add a new variant to `EmbedProvider` enum and wire it in `EmbedRouter::from_env()`.
3. Add corresponding `HS_EMBED_<METRIC>_PROVIDER=<name>` documentation to `docs/book/src/embeddings.md`.
4. If the provider requires a new Cargo dependency, gate it behind a feature flag.

## 🔐 Zero-Knowledge Encryption & Client SDK Guidelines

If you are contributing to SDK cryptographic features or cognitive memory integrations:
1. **Mathematical Invariance**:
   - For Euclidean and Cosine spaces, only use orthogonal transformations $O(n)$ ($Q^T Q = I$) generated deterministically from the user's master key via seeded QR decomposition.
   - For hyperbolic manifolds, use Lorentz transformations $SO^+(1, d)$ satisfying $L^T \eta L = \eta$ where $\eta = \text{diag}(-1, 1, \dots, 1)$. Never apply naive orthogonal rotations to hyperbolic coordinates as this violates the Minkowski pseudo-norm.
2. **Deterministic Seed Derivation**:
   - Always derive matrix seeds via HKDF-SHA256 from the master secret (`master_key`) and salt (`salt` or collection name). Never rely on non-cryptographic RNGs.
3. **Zero Knowledge Guarantee**:
   - The server engine must **never** receive raw unencrypted vectors or AES encryption keys.
   - Plaintext metadata chunks must be encrypted client-side via authenticated AES-256-GCM before transport.
   - Collections with encrypted vectors/payloads must set `encrypted: true` in `CollectionSchema` to enable dashboard security badges and audit logs.
4. **Autonomous Memory Integrations**:
   - When contributing to `hyperspace-memory-*` or MCP tools, Ebbinghaus temporal decay ($S(t) = S_0 \cdot \exp(-\lambda \Delta t)$) and knowledge superseding (`SUPERSEDES` graph edges) must execute autonomously without calling external LLMs, ensuring sub-5ms operational latency.

## 🚀 Future Roadmap

We focus on building the **Universal Spatial Memory** for AI Agents.

### Phase 1: Ecosystem & Ubiquity (v1.x)
*The goal: Run everywhere RuVector runs, but faster and with better math.*

* **v1.1**: ✅ **Multi-Tenancy (Collections)**. Support for named Collections within a single instance. *Completed.*
* **v1.2**: ✅ **Web Dashboard & Euclidean Support**. Full management UI, L2 Metric support, and Presets. *Completed.*
* **v1.3**: ✅ **Universal TypeScript SDK**. Native bindings for Node.js, Deno, and Bun. *Completed.*
* **v1.4**: ✅ **WASM Core ("Edge Memory")**. Compiling `hyperspace-core` to WebAssembly to run directly in the browser (Local-First AI). Zero latency, zero network calls. *Core implementation ready.*

### Phase 2: Scale & Structure (v2.x)
*The goal: Serverless Economy and Cloud-Native Architecture.*

* **v2.0**: ✅ **Serverless Core**. Idle unloading, cold start, multi-tenancy, and Jemalloc tuning. *Completed.*
* **v2.1**: ✅ **Data-Plane Throughput Upgrade**. Batch search API, Lorentz metric integration, SDK/doc refresh. *Completed.*
* **v2.2**: ✅ **Hyperbolic Graph Traversal API** (planned). Graph-native traversal endpoints and neighborhood/cluster primitives (not fully implemented yet). *Completed.*
* **v2.3**: **Storage Tiering (S3/Blob)**. Automatic backup of idle collections to object storage.


### Phase 3: Collective Intelligence (v3.x)
*The goal: Beyond storage. The "Digital Thalamus" realization.*

* **v3.0-alpha.1**: ✅ **Federated Swarm Protocol & Graph Diagnostics**. Connecting independent HyperspaceDB instances into a decentralized knowledge graph. Allows agents to "share memories" without centralized servers. Also added AI Sleep Mode / Memory Reconsolidation. *Completed.*
* **v3.0-alpha.2**: ✅ **Multi-Geometry Benchmark & SDK Sync**. Graph Diagnostics in SDK, Multi-Geometry Search API, Wasserstein metric (native O(N) 1D), dependency pruning. *Completed.*
* **v3.0-alpha.3**: ✅ **Anisotropic SQ8 & Per-Geometry Embedding System**. ScaNN-inspired coordinate-descent quantization for Cosine/L2 (+5.3% / +3.8% Recall@10). Full embedding service with Local ONNX, HuggingFace Hub, and 6 remote API providers. Documentation overhaul. *Completed.*
* **v3.1.0**: ✅ **Schema-Driven Cascade & MRL**. Transition to `CollectionSchema` for multi-vector support. Native Matryoshka (MRL) cascading for RAM-optimized sub-millisecond search. *Completed.*
* **v3.1.2**: ✅ **DePIN Infrastructure & Zero-Knowledge Privacy**. Unified `hyperspace-miner` node, signed ticket validation, per-byte storage billing, and client-side ZK-encryption. *Completed.*
* **v3.1.3**: ✅ **AsymmetricHybrid801 Quantization & Cognitive MCP**. hardware resource telemetry (`sysinfo`), Zero-RAM Sidecar Payload integration for LangChain/LlamaIndex, and 6 standardized Cognitive Skills tools. *Completed.*

### Phase 4: Thread-per-Core, Hardware Direct I/O & Micro-Architecture Acceleration (v4.x)
*The goal: Reflex-Level Speed, Zero Lock Contention, and Hardware NVMe/SIMD Alignment.*

* **v4.0.0**: ✅ **Thread-per-Core Shared-Nothing Sharding & Direct I/O**. *Completed.*
  - `ShardedCollection` eliminating cross-core lock contention (0 ns lock contention).
  - `DirectVectorStore` hardware page-aligned DMA (`O_DIRECT` / `fcntl F_NOCACHE`), reducing NVMe jitter to < 0.35 ms and MRL cascade tail latency (P99) to 3.14 ms.
  - Hierarchical Spatial AABB index with 95–99.9% candidate pruning (1,774 QPS).
  - Autonomous zero-downtime startup migration engine (`data/` -> `data_v4/`).
  - Production Product Quantization (`hyperspace-core::pq`) with OPQ and RaBitQ 1-bit ADC.
  - Server core ingest rate of 67,500 vec/s and index construction in 78.54 s.
* **v4.1.0**: ✅ **Native Vector-Graph (KAG) Engine & Declarative Induction**. *Completed.*
  - Internalized Property CSR Graph (`DynamicGraph`, 12-byte `CompactEdge`) eliminating dual-database patterns.
  - Explicit edge linking (`link`, `link_batch`, `unlink`) and declarative metadata induction (`path_hierarchy`, `foreign_key`, `exact_match`).
  - Minkowski Light-Cone causal subsumption projection for 33D Lorentz hyperbolic manifolds.
  - Out-of-Core SuperKMeans Streaming Clustering cutting peak memory by 1,000× (58 MB on 10M vectors).
  - Specialized Domain Cognitive SDKs (`chemSDK`, `bioSDK`, `secSDK`, `legalSDK`, `ontoSDK`, `finSDK`).
* **v4.1.0 (Performance Engine)**: ✅ **Hardware-Level Micro-Architecture Scaling**. *Completed.*
  - Dynamic Runtime SIMD Dispatch (AVX-512F / AVX2+FMA / ARM NEON / Scalar).
  - Zero-Copy Inner Search Traversal kernels (`distance_bytes_*`) eliminating heap allocations in HNSW beam descent.
  - 32-Stripe Sharded Payload Storage Index (`ShardedPayloadIndex`) with Linux `io_uring` batched reads.
  - Multi-Socket NUMA-Aware Topology & Core Affinity pinning.
  - Hoisted PQ lookup tables and bounded candidate heaps.
* **v4.1.2**: ✅ **Zero-Knowledge Client Encryption, Vec2Text Defense & Autonomous Agent Cognitive Memory**. *Completed.*
  - Client-Side Isometric Orthogonal $O(n)$ and Lorentz $SO^+(1, d)$ Projections with 100% mathematical recall preservation.
  - Anisotropic noise perturbation suppressing Vec2Text inversion similarity from $0.984$ to $0.038$.
  - End-to-end AES-256-GCM chunk payload encryption and per-collection `encrypted` schema flag.
  - Autonomous Ebbinghaus exponential decay $S(t) = S_0 \cdot \exp(-\lambda \Delta t)$ and temporal superseding via vector-graph edges (<4ms latency, 0 LLM calls).

Join us in pushing the boundaries of hyperbolic vector search!


