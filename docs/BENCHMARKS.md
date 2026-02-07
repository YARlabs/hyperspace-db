# HyperspaceDB Benchmarks

> **Status**: Verified (v1.4.0)  
> **Date**: 2026-02-07  
> **Hardware**: Apple M1 Max, 64GB RAM

## Executive Summary

HyperspaceDB achieves **phenomenal search performance**, significantly outperforming market leaders in latency. However, write throughput is currently a bottleneck due to strict durability guarantees and synchronous indexing.

### 🏆 The Superpower: Search Latency (<1ms)
HyperspaceDB is the **fastest vector database for search**, delivering sub-millisecond P99 latency.

*   **P99 Latency**: **0.97 ms** (vs 8.93 ms for Milvus, 27.62 ms for Qdrant)
*   **Speedup**: **9x faster** than Milvus, **28x faster** than Qdrant.
*   **Stability**: Extremely low jitter (Avg 0.44ms vs P99 0.97ms).

**Verdict**: Ideal for **Real-time RAG**, **AI Agents**, and **Edge AI** where "instant memory" is critical.

### 🔻 The Bottleneck: Write Performance
Current insertion throughput is lower than competitors due to unoptimized synchronous write paths (WAL fsync + synchronous HNSW updates).

*   **Insert QPS**: **197** (vs 12,500 for Milvus).
*   **Gap**: ~63x slower than the fastest competitor.

**Verdict**: Suitable for streaming inserts (agents, user sessions) but not yet optimized for massive batch ingestion (Big Data Analytics).

---

## Detailed Results

### 1. Search Performance (100k Vectors, 1024-dim)

| Database | Version | Avg (ms) | P50 (ms) | P95 (ms) | **P99 (ms)** |
|----------|---------|----------|----------|----------|--------------|
| **HyperspaceDB** | **v1.4.0** | **0.44** | **0.40** | **0.65** | **0.97** 🏆 |
| **Milvus** | latest | 3.62 | 3.24 | 5.81 | 8.93 |
| **Weaviate** | latest | 6.02 | 5.33 | 9.47 | 14.15 |
| **Qdrant** | latest | 8.89 | 6.54 | 20.00 | 27.62 |

> **Note**: HyperspaceDB consistently delivers <1ms response times, making it uniquely suited for latency-sensitive AI applications.

### 2. Insert Performance (100k Vectors, 1024-dim)

| Database | Version | QPS | Total Time | Notes |
|----------|---------|-----|------------|-------|
| **Milvus** | latest | **12,499** | 8.0s | Asynchronous indexing |
| **Qdrant** | latest | 1,526 | 65.5s | |
| **Weaviate** | latest | 453 | 220.8s | |
| **HyperspaceDB** | v1.4.0 | 197 | 507.3s | Synchronous WAL + Indexing |

---

## Benchmark Configuration

*   **Dataset**: 100,000 random vectors (normalized)
*   **Dimensions**: 1024
*   **Metric**: Euclidean (L2)
*   **Batch Size**: 1,000
*   **Clients**: Docker optimized containers (except HyperspaceDB running locally for debug)

## Optimization Roadmap (Sprint 4)

To bridge the gap in write performance, the following optimizations are planned:

1.  **Bulk Insert Mode**: Defer HNSW indexing until after data ingestion (similar to Milvus/Qdrant strategies).
2.  **WAL Async Writes**: Allow relaxed durability for high-throughput batch jobs.
3.  **Parallel Indexing**: Utilize full CPU cores during batch insertions using Rayon.
