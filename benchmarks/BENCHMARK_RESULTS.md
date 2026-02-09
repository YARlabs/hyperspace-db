# Vector Database Benchmark Results

**Date**: 2026-02-08 16:57:11  
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
| **HyperspaceDB** | 1.5.0 | **15,369** | 6.5s | 15.74 M dims/s | 988.0 MB |
| **Qdrant** | latest | **1,422** | 70.3s | 1.46 M dims/s | 462.0 MB |
| **Weaviate** | latest | **552** | 181.0s | 0.57 M dims/s | 502.0 MB |
| **Milvus** | latest | **16,683** | 6.0s | 17.08 M dims/s | 5207.0 MB |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **HyperspaceDB** | 1.80 | 1.20 | 4.93 | 10.62 |
| **Qdrant** | 12.21 | 11.67 | 20.12 | 24.39 |
| **Weaviate** | 3.53 | 3.52 | 4.06 | 4.56 |
| **Milvus** | 2.78 | 2.70 | 3.67 | 5.92 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **16,683 QPS**

- 1.09x faster than HyperspaceDB
- 11.73x faster than Qdrant
- 30.20x faster than Weaviate

### Search Latency Winner: 🏆 **Weaviate**
- **4.56 ms** (p99)

- 2.33x faster than HyperspaceDB
- 5.35x faster than Qdrant
- 1.30x faster than Milvus

---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 15368.960599318172,
    "insert_total_time": 6.50662088394165,
    "search_avg_ms": 1.7972073554992676,
    "search_p50_ms": 1.2049674987792969,
    "search_p95_ms": 4.925966262817383,
    "search_p99_ms": 10.622024536132812,
    "memory_mb": 0.0,
    "disk_usage_mb": 988.0053548812866,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 1422.4107840630525,
    "insert_total_time": 70.30317902565002,
    "search_avg_ms": 12.205270051956177,
    "search_p50_ms": 11.673688888549805,
    "search_p95_ms": 20.11895179748535,
    "search_p99_ms": 24.38521385192871,
    "memory_mb": 0.0,
    "disk_usage_mb": 462.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 552.3989333393937,
    "insert_total_time": 181.02858996391296,
    "search_avg_ms": 3.529292106628418,
    "search_p50_ms": 3.520965576171875,
    "search_p95_ms": 4.061222076416016,
    "search_p99_ms": 4.559755325317383,
    "memory_mb": 0.0,
    "disk_usage_mb": 502.0,
    "errors": []
  },
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 16682.549206274078,
    "insert_total_time": 5.994287729263306,
    "search_avg_ms": 2.7804043292999268,
    "search_p50_ms": 2.6979446411132812,
    "search_p95_ms": 3.6737918853759766,
    "search_p99_ms": 5.920171737670898,
    "memory_mb": 0.0,
    "disk_usage_mb": 5207.0,
    "errors": []
  }
]
```
