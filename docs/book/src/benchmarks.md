# Evaluation & Benchmarks

HyperspaceDB is optimized for two critical metrics: **Throughput** (Ingestion speed) and **Latency** (Search speed).

## Test Environment
*   **Hardware**: Apple M4 Pro (Emulated Environment) / Linux AVX2
*   **Dataset**: 1,000,000 vectors, 8 Dimensions, Random Distribution in Unit Ball.
*   **Config**: `ef_construction=100`, `ef_search=100`

## Results

### 🚀 Ingestion Speed

Thanks to the Async Write Buffer (WAL) and background indexing, ingestion does not block user requests.

| Count | Time | Throughput | Storage Sements |
| :--- | :--- | :--- | :--- |
| 10,000 | 0.6s | 15,624 vec/s | 1 |
| 100,000 | 6.5s | 15,300 vec/s | 2 |
| **1,000,000** | **64.8s** | **15,420 vec/s** | **15** |

### 🔍 Search Latency (1M Scale)

At 1 million vectors, search performance degrades linearly with graph depth ($\log N$), proving effective HNSW implementation.

| Metric | Value |
| :--- | :--- |
| **QPS** | 14,668 queries/sec |
| **Avg Latency** | **0.07 ms** |
| **P99 Latency** | < 1.0 ms |

## Why is it so fast?

1.  **ScalarI8 Quantization**: Fits 8x more vectors in CPU cache.
2.  **No `acosh`**: Inner loop uses a monotonic proxy function ($\delta$).
3.  **SIMD**: Vector operations use platform-specific intrinsics.
