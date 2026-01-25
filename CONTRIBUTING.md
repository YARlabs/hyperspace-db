# Contributing to HyperspaceDB

Thank you for your interest in contributing to HyperspaceDB! As an experimental, high-performance vector database, we welcome contributions that improve speed, accuracy, or usability.

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

*   **v2.0**: Distributed clustering (Raft/Gossip).
*   **v2.1**: Higher-order quantization (Product Quantization).
*   **v2.2**: SIMD Search for ARM Neon / AVX-512.

Join us in pushing the boundaries of hyperbolic vector search!
