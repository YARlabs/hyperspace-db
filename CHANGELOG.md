# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features
*   **Client-Side Vectorization (Fat Client)**: SDKs now support built-in embedding generation. Clients can pass raw text documents and queries, which are automatically vectorized before reaching the server.
    *   **Python SDK**: Support for OpenAI, Cohere, Voyage, Google (Gemini), and local SentenceTransformers (`bge-m3`).
    *   **Rust SDK**: Added `Embedder` trait with implementations for OpenAI, OpenRouter, Cohere, Voyage, and Google (Gemini) behind `embedders` feature flag.

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
