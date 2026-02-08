# Vector Database Benchmark Results

**Date**: 2026-02-08 11:09:32  
**Configuration**:
- Vectors: 100,000
- Dimensions: 1024
- Batch Size: 1,000
- Search Queries: 1,000
- Top-K: 10

---

## Insert Performance

| Database | Version | QPS | Total Time | Throughput | Disk Usage (MB) |
|----------|---------|-----|------------|------------|-----------------|
| **HyperspaceDB** | 1.5.0 | **10,811** | 9.2s | 11.07 M dims/s | 1002.1 MB |
| **Qdrant** | latest | **895** | 111.8s | 0.92 M dims/s | 462.0 MB |
| **Weaviate** | latest | **488** | 204.9s | 0.50 M dims/s | 561.0 MB |
| **Milvus** | latest | **12,913** | 7.7s | 13.22 M dims/s | 4821.0 MB |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **HyperspaceDB** | 2.14 | 1.64 | 4.76 | 8.18 |
| **Qdrant** | 20.08 | 18.54 | 31.27 | 49.06 |
| **Weaviate** | 6.25 | 5.79 | 9.31 | 10.57 |
| **Milvus** | 3.37 | 3.03 | 5.92 | 7.95 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **12,913 QPS**

- 1.19x faster than HyperspaceDB
- 14.44x faster than Qdrant
- 26.46x faster than Weaviate

### Search Latency Winner: 🏆 **Milvus**
- **7.95 ms** (p99)

- 1.03x faster than HyperspaceDB
- 6.17x faster than Qdrant
- 1.33x faster than Weaviate

---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 10811.26307616292,
    "insert_total_time": 9.249613046646118,
    "search_avg_ms": 2.1356606483459473,
    "search_p50_ms": 1.6410350799560547,
    "search_p95_ms": 4.7550201416015625,
    "search_p99_ms": 8.183956146240234,
    "memory_mb": 0.0,
    "disk_usage_mb": 1002.0703649520874,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 894.5676990520752,
    "insert_total_time": 111.78583812713623,
    "search_avg_ms": 20.080114126205444,
    "search_p50_ms": 18.53799819946289,
    "search_p95_ms": 31.26692771911621,
    "search_p99_ms": 49.063920974731445,
    "memory_mb": 0.0,
    "disk_usage_mb": 462.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 488.0256846266145,
    "insert_total_time": 204.90724802017212,
    "search_avg_ms": 6.248524188995361,
    "search_p50_ms": 5.791664123535156,
    "search_p95_ms": 9.31406021118164,
    "search_p99_ms": 10.567903518676758,
    "memory_mb": 0.0,
    "disk_usage_mb": 561.0,
    "errors": []
  },
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 12913.463742474813,
    "insert_total_time": 7.743855714797974,
    "search_avg_ms": 3.371178150177002,
    "search_p50_ms": 3.02886962890625,
    "search_p95_ms": 5.916833877563477,
    "search_p99_ms": 7.9517364501953125,
    "memory_mb": 0.0,
    "disk_usage_mb": 4821.0,
    "errors": []
  }
]
```
