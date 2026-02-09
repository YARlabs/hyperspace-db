# Vector Database Benchmark Results

**Date**: 2026-02-09 09:41:37  
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
| **Milvus** | latest | **17,149** | 5.8s | 17.56 M dims/s | 6803.0 MB |
| **Qdrant** | latest | **1,892** | 52.9s | 1.94 M dims/s | 905.0 MB |
| **Weaviate** | latest | **556** | 179.9s | 0.57 M dims/s | 526.0 MB |
| **HyperspaceDB** | 1.5.0 | **16,141** | 6.2s | 16.53 M dims/s | 1016.2 MB |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **Milvus** | 1.91 | 1.92 | 2.56 | 3.06 |
| **Qdrant** | 6.65 | 6.46 | 7.75 | 12.28 |
| **Weaviate** | 3.54 | 3.53 | 3.98 | 4.54 |
| **HyperspaceDB** | 2.32 | 1.33 | 6.56 | 10.55 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **17,149 QPS**

- 9.07x faster than Qdrant
- 30.84x faster than Weaviate
- 1.06x faster than HyperspaceDB

### Search Latency Winner: 🏆 **Milvus**
- **3.06 ms** (p99)

- 4.01x faster than Qdrant
- 1.48x faster than Weaviate
- 3.44x faster than HyperspaceDB

---

## Raw Data (JSON)

```json
[
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 17149.349334852595,
    "insert_total_time": 5.831125020980835,
    "search_avg_ms": 1.910473108291626,
    "search_p50_ms": 1.9230842590332031,
    "search_p95_ms": 2.562999725341797,
    "search_p99_ms": 3.0629634857177734,
    "memory_mb": 0.0,
    "disk_usage_mb": 6803.0,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 1891.8034512671977,
    "insert_total_time": 52.85961389541626,
    "search_avg_ms": 6.648202419281006,
    "search_p50_ms": 6.464719772338867,
    "search_p95_ms": 7.751226425170898,
    "search_p99_ms": 12.284994125366211,
    "memory_mb": 0.0,
    "disk_usage_mb": 905.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 556.0104956302897,
    "insert_total_time": 179.85272002220154,
    "search_avg_ms": 3.5388448238372803,
    "search_p50_ms": 3.5309791564941406,
    "search_p95_ms": 3.9780139923095703,
    "search_p99_ms": 4.541158676147461,
    "memory_mb": 0.0,
    "disk_usage_mb": 526.0,
    "errors": []
  },
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 16140.622116236997,
    "insert_total_time": 6.195548057556152,
    "search_avg_ms": 2.315289258956909,
    "search_p50_ms": 1.332998275756836,
    "search_p95_ms": 6.559848785400391,
    "search_p99_ms": 10.545969009399414,
    "memory_mb": 0.0,
    "disk_usage_mb": 1016.2003927230835,
    "errors": []
  }
]
```
