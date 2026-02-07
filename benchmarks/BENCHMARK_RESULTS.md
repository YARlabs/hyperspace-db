# Vector Database Benchmark Results

**Date**: 2026-02-07 22:38:57  
**Configuration**:
- Vectors: 100,000
- Dimensions: 1024
- Batch Size: 1,000
- Search Queries: 1,000
- Top-K: 10

---

## Insert Performance

| Database | Version | QPS | Total Time | Throughput | Mem (MB) |
|----------|---------|-----|------------|------------|-----------|
| **HyperspaceDB** | 1.5.0 | **5,821** | 17.2s | 5.96 M dims/s | 0.0 |
| **Qdrant** | latest | **1,581** | 63.3s | 1.62 M dims/s | 0.0 |
| **Weaviate** | latest | **476** | 210.2s | 0.49 M dims/s | 0.0 |
| **Milvus** | latest | **13,651** | 7.3s | 13.98 M dims/s | 0.0 |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **HyperspaceDB** | 0.10 | 0.09 | 0.14 | 0.31 |
| **Qdrant** | 5.47 | 4.43 | 12.14 | 17.54 |
| **Weaviate** | 5.10 | 4.48 | 8.22 | 11.91 |
| **Milvus** | 2.61 | 2.44 | 3.91 | 4.98 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **13,651 QPS**

- 2.35x faster than HyperspaceDB
- 8.64x faster than Qdrant
- 28.69x faster than Weaviate

### Search Latency Winner: 🏆 **HyperspaceDB**
- **0.31 ms** (p99)

- 56.95x faster than Qdrant
- 38.66x faster than Weaviate
- 16.17x faster than Milvus

---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 5821.343562536726,
    "insert_total_time": 17.178164958953857,
    "search_avg_ms": 0.10232305526733398,
    "search_p50_ms": 0.08893013000488281,
    "search_p95_ms": 0.1399517059326172,
    "search_p99_ms": 0.30803680419921875,
    "memory_mb": 0.0,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 1580.8136331426717,
    "insert_total_time": 63.25856375694275,
    "search_avg_ms": 5.465744733810425,
    "search_p50_ms": 4.426002502441406,
    "search_p95_ms": 12.140035629272461,
    "search_p99_ms": 17.54283905029297,
    "memory_mb": 0.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 475.79063122256906,
    "insert_total_time": 210.17647981643677,
    "search_avg_ms": 5.10451340675354,
    "search_p50_ms": 4.477977752685547,
    "search_p95_ms": 8.219003677368164,
    "search_p99_ms": 11.909246444702148,
    "memory_mb": 0.0,
    "errors": []
  },
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 13651.226991668604,
    "insert_total_time": 7.325348854064941,
    "search_avg_ms": 2.608966112136841,
    "search_p50_ms": 2.444028854370117,
    "search_p95_ms": 3.9141178131103516,
    "search_p99_ms": 4.9800872802734375,
    "memory_mb": 0.0,
    "errors": []
  }
]
```
