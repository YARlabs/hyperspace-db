# Vector Database Benchmark Results

**Date**: 2026-02-09 11:39:05  
**Configuration**:
- Vectors: 1,000,000
- Dimensions: 1024
- Batch Size: 1,000
- Search Queries: 10,000
- Top-K: 10

---

## Insert Performance

| Database | Version | QPS | Total Time | Throughput | Disk Usage (MB) |
|----------|---------|-----|------------|------------|-----------------|
| **Milvus** | latest | **17,220** | 58.1s | 17.63 M dims/s | 14632.0 MB |
| **Qdrant** | latest | **1,712** | 584.2s | 1.75 M dims/s | 4187.0 MB |
| **Weaviate** | latest | **486** | 2055.9s | 0.50 M dims/s | 5041.0 MB |
| **HyperspaceDB** | 1.5.0 | **803** | 1244.8s | 0.82 M dims/s | 9128.2 MB |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **Milvus** | 7.10 | 6.94 | 10.64 | 15.15 |
| **Qdrant** | 8.36 | 8.08 | 11.76 | 14.75 |
| **Weaviate** | 3.72 | 3.63 | 4.50 | 5.45 |
| **HyperspaceDB** | 1.97 | 1.44 | 4.91 | 7.69 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **17,220 QPS**

- 10.06x faster than Qdrant
- 35.40x faster than Weaviate
- 21.43x faster than HyperspaceDB

### Search Latency Winner: 🏆 **Weaviate**
- **5.45 ms** (p99)

- 2.78x faster than Milvus
- 2.71x faster than Qdrant
- 1.41x faster than HyperspaceDB

---

## Raw Data (JSON)

```json
[
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 17219.92950456697,
    "insert_total_time": 58.07224702835083,
    "search_avg_ms": 7.102082443237305,
    "search_p50_ms": 6.9427490234375,
    "search_p95_ms": 10.637044906616211,
    "search_p99_ms": 15.151023864746094,
    "memory_mb": 0.0,
    "disk_usage_mb": 14632.0,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 1711.7688753331988,
    "insert_total_time": 584.1910169124603,
    "search_avg_ms": 8.36308195590973,
    "search_p50_ms": 8.079051971435547,
    "search_p95_ms": 11.757135391235352,
    "search_p99_ms": 14.747858047485352,
    "memory_mb": 0.0,
    "disk_usage_mb": 4187.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 486.413177124161,
    "insert_total_time": 2055.8653569221497,
    "search_avg_ms": 3.718737483024597,
    "search_p50_ms": 3.629922866821289,
    "search_p95_ms": 4.497289657592773,
    "search_p99_ms": 5.45191764831543,
    "memory_mb": 0.0,
    "disk_usage_mb": 5041.0,
    "errors": []
  },
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 803.3604878668461,
    "insert_total_time": 1244.7712018489838,
    "search_avg_ms": 1.9650747299194335,
    "search_p50_ms": 1.4429092407226562,
    "search_p95_ms": 4.911899566650391,
    "search_p99_ms": 7.688999176025391,
    "memory_mb": 0.0,
    "disk_usage_mb": 9128.15635585785,
    "errors": []
  }
]
```
