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

We maintain a disciplined release schedule. Below are the key milestones for the upcoming versions.

### Phase 1: Expansion (v1.x)
* **v1.1**: **Multi-Tenancy**. Support for named Collections (Namespaces) within a single instance to isolate datasets.
* **v1.2**: **Official SDKs**. Generating native clients for TypeScript (Node/Deno/Bun) and Go.
* **v1.3**: **Cloud Control Plane**. Web-based GUI (WASM) for managing instances via browser, replacing the TUI for remote deployments.
* **v1.4**: **SaaS / Cloud Hooks**. Webhooks and Events API for deeper integration with serverless platforms.

### Phase 2: Scale (v2.x)
* **v2.0**: **Distributed Consensus**. Implementation of Raft for true horizontal scaling and sharding across multiple nodes.
* **v2.1**: **Storage Tiering**. Automatic offloading of cold segments to S3/Blob Storage to reduce local disk usage.
* **v2.2**: **Product Quantization (PQ)**. Higher-order quantization for even better compression ratios on billion-scale datasets.

### Phase 3: Intelligence (v3.x)
* **v3.0**: **Native Embedding Generation**. Integration of `ort` (ONNX Runtime) to generate embeddings directly within the database engine.

Join us in pushing the boundaries of hyperbolic vector search!
