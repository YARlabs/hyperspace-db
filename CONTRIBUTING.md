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
```

## 🚀 Future Roadmap

We focus on building the **Universal Spatial Memory** for AI Agents.

### Phase 1: Ecosystem & Ubiquity (v1.x)
*The goal: Run everywhere RuVector runs, but faster and with better math.*

* **v1.1**: ✅ **Multi-Tenancy (Collections)**. Support for named Collections within a single instance. Includes Web Dashboard for management. *Completed.*
* **v1.2**: **Universal TypeScript SDK**. Native bindings for Node.js, Deno, and Bun. *Direct challenge to RuVector's ecosystem.*
* **v1.3**: **WASM Core ("Edge Memory")**. Compiling `hyperspace-core` to WebAssembly to run directly in the browser (Local-First AI). Zero latency, zero network calls.
* **v1.4**: **Visual Graph Explorer**. A web-based tool to visualize the Poincaré disk and navigate your data's hierarchy interactively.

### Phase 2: Scale & Structure (v2.x)
*The goal: True Enterprise Scale and Graph Capabilities.*

* **v2.0**: **Distributed Consensus**. Implementation of Raft for horizontal scaling/sharding.
* **v2.1**: **Hyperbolic Graph Traversal API**. Exposing the HNSW graph structure to allow queries like "Get parent concepts" or "Find semantic clusters" without embedding generation. *Beats GNNs in speed.*
* **v2.2**: **Storage Tiering (S3/Blob)**. Automatic offloading of cold segments to object storage.

### Phase 3: Collective Intelligence (v3.x)
*The goal: Beyond storage. The "Digital Thalamus" realization.*

* **v3.0**: **Federated Swarm Protocol**. Connecting independent HyperspaceDB instances into a decentralized knowledge graph. Allows agents to "share memories" without centralized servers.
* **v3.1**: **Generative Memory**. Optional integration with LLMs to perform "Retrieval-Augmented Generation" directly inside the database query pipeline.

Join us in pushing the boundaries of hyperbolic vector search!
