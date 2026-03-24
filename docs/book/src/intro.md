# [H] HyperspaceDB

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](#)
[![Version](https://img.shields.io/badge/version-3.0.1-blue.svg)](#)

**Fastest Vector Database for Hierarchical & Flat Data written in Rust.**  
HyperspaceDB natively supports both the **Poincaré ball model** (for hierarchies) and **Euclidean space** (for standard OpenAI/BGE embeddings), delivering extreme performance through specialized SIMD kernels.

---

## 🚀 Key Features

*   **⚡️ Extreme Performance**: Built with Nightly Rust and SIMD intrinsics for maximum search throughput.
*   **📐 Cognitive Math Engine**: Hyperbolic HNSW optimized for the Poincaré and Lorentz metrics, and O(N) Wasserstein-1 logic.
*   **📦 Compression**: Integrated `ScalarI8` and `Binary` quantization reduces memory footprint by 87% to 98%.
*   **🧵 Async Write Pipeline**: Decoupled ingestion with a background indexing worker and WAL for 10x faster inserts.
*   **🖥️ Mission Control TUI**: Real-time terminal dashboard for monitoring QPS, segments, and system health.
*   **🕸️ Edge Ready**: WASM compilation target allows running the full DB in browser with **Local-First** privacy and **IndexedDB** persistence.
*   **🛠️ Runtime Tuning**: Dynamically adjust `ef_search` and `ef_construction` parameters via gRPC on-the-fly.
*   **🏙 Multi-Tenancy**: Native SaaS support with namespace isolation (`user_id`) and billing stats.
*   **🔁 Replication**: Leader-Follower architecture with Anti-Entropy catch-up for high availability.
*   **⚖️ Cognitive Math & Tribunal Router**: Native SDK utilities for calculating geometric trust scores on graphs to detect LLM hallucinations.
*   **📡 Memory Reconsolidation**: Trigger AI sleep mode natively within the DB to restructure vectors via Flow Matching / Riemannian SGD.

---

## 🛠 Architecture

HyperspaceDB follows a **Persistence-First, Index-Second** design:
1.  **gRPC Request**: Insert/Search commands arrive via a high-performance Tonic server.
2.  **WAL & Segmented Storage**: Every insert is immediate persisted to a Write-Ahead Log and a memory-mapped segmented file store.
3.  **Background Indexer**: The HNSW graph is updated asynchronously by a dedicated thread-pool, ensuring 0ms search blocking.
4.  **Snapshots**: Real-time graph topology is periodically serialized using `rkyv` for near-instant restarts.

---

## 🏃 Quick Start

### 1. Build and Start Server
Make sure you have `just` and `nightly rust` installed.

```bash
cargo build --release
./target/release/hyperspace-server
```

### 2. Launch Dashboard
```bash
./target/release/hyperspace-cli
```

### 3. Use Python SDK
```bash
pip install ./sdks/python
```

```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051")
client.insert(vector=[0.1]*8, metadata={"category": "tech"})
results = client.search(vector=[0.11]*8, top_k=5)
```

---

## 📊 Performance Benchmarks
*Tested on M4 Pro (Emulated), 1M Vectors (8D)*

*   **Insert Throughput**: ~156,000 vectors/sec (Sustained)
*   **Search Latency**: ~2.47ms (156,000 QPS) @ 1M scale
*   **Storage Efficiency**: Automatic segmentation + mmap

### "The 1 Million Challenge"
HyperspaceDB successfully handles **1,000,000** vectors with zero degradation compared to traditional vector DBs, maintaining 156,000 QPS at the 1M scale.

---

## 📄 License
AGPLv3 © YARlabs
