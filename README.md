# HyperspaceDB 🌌

**HyperspaceDB** is a high-performance, production-grade Vector Database written in **Rust**, optimized for Hierarchical Data using the **Poincaré Ball** model (Hyperbolic Geometry).

It is designed for specialized AI applications: Knowledge Graphs, Bioinformatics, and Semantic Search where hierarchy matters.

---

## 🏗 Architecture Overview

HyperspaceDB is built as a monolithic system with a clear separation of concerns, optimized for modern hardware.

1.  **Transport Layer (gRPC/Tonic):**
    *   High-performance entry point using `tonic`.
    *   Zero-copy deserialization where possible.
    *   Protocol Buffers (`hyperspace.proto`) interface for clients (Python, Go, Node).

2.  **The Brain (In-Memory HNSW):**
    *   **Graph-based Index** tailored for Hyperbolic space.
    *   Lives entirely in RAM.
    *   Uses fine-grained locking (`parking_lot::RwLock`) for high concurrency.
    *   Stores only topology (topology-aware), delegating data storage to the persistence layer.

3.  **The Heart (Hyperbolic SIMD):**
    *   Powered by Nightly Rust `portable_simd`.
    *   Computes Poincaré distance using **AVX-512** (x86) or **NEON** (ARM) instructions.
    *    Processes 8 dimensions per CPU cycle.

4.  **The Memory (Mmap Storage):**
    *   **Memory-Mapped File** approach.
    *   Reliable persistence managed by the OS Page Cache.
    *   Supports databases larger than physical RAM (lazy loading/swapping).

---

## 🚀 Getting Started

### Prerequisites
*   Rust **Nightly** (required for SIMD)
*   `protobuf` compiler (`protoc`)

### Quick Start
```bash
# 1. Build the project
cargo build --release

# 2. Run the Server
cargo run -p hyperspace-server

# 3. Run the TUI Dashboard (in a new terminal)
cargo run -p hyperspace-cli

# 4. Run Python Client Test
cd sdks/python
python3 -m venv venv && source venv/bin/activate
pip install grpcio grpcio-tools numpy
python test_search.py
```

---

## 🗺 Roadmap (v0.2.0+)

The current version (v0.1.0) is a functional engine. The following features are planned for the next release:

### 1. Crash Recovery (WAL)
*   **Problem:** Mmap flushes are lazy; power loss means data loss.
*   **Solution:** Implement **Write Ahead Log (WAL)**. Append operations to a log file before modifying the mmap/index. Replay log on startup.

### 2. Metadata Filtering
*   **Problem:** Search currently only considers vector similarity.
*   **Solution:** Integrate **Bitmap** or **Inverted Indexing**. Apply pre-filtering predicates before HNSW traversal or during neighbor selection.

### 3. Scalar Quantization (SQ)
*   **Problem:** `f64` vectors consume significant memory (12KB per 1536-dim vector).
*   **Solution:** Compress vectors to `u8` or `i8` quantization, reducing memory footprint by ~8x with minimal accuracy loss. By default SQ will be disabled.

---

## 📂 Project Structure

*   `crates/hyperspace-core`: SIMD Math & Primitives
*   `crates/hyperspace-index`: HNSW Graph Implementation
*   `crates/hyperspace-store`: Mmap Persistence Layer
*   `crates/hyperspace-proto`: gRPC Definitions
*   `crates/hyperspace-server`: Server Entrypoint
*   `crates/hyperspace-cli`: TUI Dashboard

Happy Hacking! 🚀
