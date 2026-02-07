# Sprint 4: Write Performance Optimization

## Use Case & Goal

HyperspaceDB has **phenomenal search performance (<1ms P99)** but **poor write performance (197 QPS)**.

**Use Case**: Batch ingestion for initial data loading, analytics pipelines, and large-scale AI agent memory updates. Current write speeds (63x slower than Milvus) are a barrier for big data adoption, though acceptable for real-time agent memory.

**Goal**: Achieve **2,000 - 3,000 QPS** for batch inserts (10-15x improvement).

## Root Cause Analysis (Hypothesis)

1.  **Synchronous HNSW**: HyperspaceDB updates the HNSW graph *synchronously* during insertion, rebalancing and recalculating neighbors immediately. Competitors (Milvus, Qdrant) often buffer writes to a WAL and build the index asynchronously or in batches.
2.  **WAL fsync**: Strict durability guarantees (fsync on every write) kills disk throughput.
3.  **Global Lock / Single Thread**: Potential lack of parallelism during batch index updates.

## Optimization Strategy

### 1. Bulk Insert Mode (Deferred Indexing)
**Status: COMPLETED (v1.5)**
Implemented `BatchInsert` gRPC endpoint and batched WAL updates. Optimized server logic to reduce lock contention and syscalls.

*   **Result**: **2,550 QPS** (13x improvement over 197 QPS).
*   **Target Met**: Yes (> 2,000 QPS).

### 2. Dynamic WAL Configuration
**Priority: Medium**
Allow relaxed durability for batch jobs.

*   **Config**: `WAL_SYNC_MODE=async` (flush every N seconds or M bytes) vs `sync` (current default).
*   **API**: `client.configure(wal_sync="async")`

### 3. Parallel Indexing (Rayon)
**Priority: High**
Use `rayon` to parallelize distance calculations and graph updates during batch insertion.

*   **Current**: Sequential processing of batch items?
*   **Optimized**: `batch.par_iter().for_each(...)` with fine-grained locking on the graph.

## Success Metrics

| Metric | Current (v1.4) | Target (v1.5) | Stretch Goal |
|--------|----------------|---------------|--------------|
| Insert QPS | 197 | **2,000** | 5,000 |
| Latency P99 | 0.97 ms | < 1.5 ms | < 1.0 ms |

> **Note**: Search latency must *not* degrade significantly due to write optimizations.
