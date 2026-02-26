# Contributing to HyperspaceDB

By submitting contributions to HyperspaceDB, you agree that
YARlabs may use your contributions under both the AGPLv3 license
and under commercial licenses.

This ensures that improvements contributed by the community
can be included in both open-source and commercial releases.

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
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib
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

* **v3.0**: ✅ **Federated Swarm Protocol**. Connecting independent HyperspaceDB instances into a decentralized knowledge graph. Allows agents to "share memories" without centralized servers. *Completed in Task 3.4.*
* **v3.1**: **Generative Memory**. Optional integration with LLMs to perform "Retrieval-Augmented Generation" directly inside the database query pipeline.

Join us in pushing the boundaries of hyperbolic vector search!
