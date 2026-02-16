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
cargo test --all-features
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
4.  Add integration tests in `tests/`.

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

* **v2.0**: ✅ **Serverless Core**. Idle Unloading, Cold Start, Multi-tenancy, and Jemalloc tuning. *Completed.*
* **v2.1**: **Storage Tiering (S3/Blob)**. Automatic backup of idle collections to object storage.
* **v2.2**: **Distributed Consensus**. Raft implementation for horizontal scaling (Cluster Mode).
* **v2.3**: **Hyperbolic Graph Traversal API**. Exposing the HNSW graph structure to allow queries like "Get parent concepts" or "Find semantic clusters" without embedding generation. *Beats GNNs in speed.*

### Phase 3: Collective Intelligence (v3.x)
*The goal: Beyond storage. The "Digital Thalamus" realization.*

* **v3.0**: **Federated Swarm Protocol**. Connecting independent HyperspaceDB instances into a decentralized knowledge graph. Allows agents to "share memories" without centralized servers.
* **v3.1**: **Generative Memory**. Optional integration with LLMs to perform "Retrieval-Augmented Generation" directly inside the database query pipeline.

Join us in pushing the boundaries of hyperbolic vector search!
