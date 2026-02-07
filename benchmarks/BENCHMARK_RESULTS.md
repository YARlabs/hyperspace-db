# Vector Database Benchmark Results

**Date**: 2026-02-07 22:48:32  
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
| **HyperspaceDB** | 1.5.0 | **175** | 573.0s | 0.18 M dims/s | 0.0 |
| **Weaviate** | latest | **487** | 205.5s | 0.50 M dims/s | 0.0 |
| **Milvus** | latest | **13,585** | 7.4s | 13.91 M dims/s | 0.0 |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **HyperspaceDB** | 0.50 | 0.44 | 0.79 | 1.63 |
| **Weaviate** | 4.79 | 4.58 | 6.33 | 8.59 |
| **Milvus** | 2.71 | 2.53 | 4.12 | 5.04 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **Milvus**
- **13,585 QPS**

- 77.84x faster than HyperspaceDB
- 27.92x faster than Weaviate

### Search Latency Winner: 🏆 **HyperspaceDB**
- **1.63 ms** (p99)

- 5.29x faster than Weaviate
- 3.10x faster than Milvus

---

## Errors

### Qdrant
- timed out


---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 174.51089296637412,
    "insert_total_time": 573.030131816864,
    "search_avg_ms": 0.5034077167510986,
    "search_p50_ms": 0.4439353942871094,
    "search_p95_ms": 0.7863044738769531,
    "search_p99_ms": 1.6252994537353516,
    "memory_mb": 0.0,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 0,
    "insert_total_time": 0,
    "search_avg_ms": 0,
    "search_p50_ms": 0,
    "search_p95_ms": 0,
    "search_p99_ms": 0,
    "memory_mb": 0,
    "errors": [
      "timed out"
    ]
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 486.59550560387345,
    "insert_total_time": 205.5095019340515,
    "search_avg_ms": 4.791643142700195,
    "search_p50_ms": 4.575967788696289,
    "search_p95_ms": 6.329059600830078,
    "search_p99_ms": 8.589982986450195,
    "memory_mb": 0.0,
    "errors": []
  },
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 13584.620008163783,
    "insert_total_time": 7.3612658977508545,
    "search_avg_ms": 2.7058982849121094,
    "search_p50_ms": 2.529144287109375,
    "search_p95_ms": 4.1179656982421875,
    "search_p99_ms": 5.037069320678711,
    "memory_mb": 0.0,
    "errors": []
  }
]
```
