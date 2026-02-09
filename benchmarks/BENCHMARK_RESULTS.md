# Vector Database Benchmark Results

**Date**: 2026-02-09 14:18:21  
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
| **Milvus** | latest | **11,269** | 88.7s | 11.54 M dims/s | 18561.0 MB |
| **Qdrant** | latest | **1,589** | 629.4s | 1.63 M dims/s | 4187.0 MB |
| **Weaviate** | latest | **491** | 2036.3s | 0.50 M dims/s | 5057.0 MB |
| **HyperspaceDB** | 1.5.0 | **17,721** | 56.4s | 18.15 M dims/s | 9003.3 MB |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **Milvus** | 6.69 | 6.13 | 9.44 | 16.12 |
| **Qdrant** | 8.00 | 7.65 | 11.18 | 14.84 |
| **Weaviate** | 6.73 | 6.51 | 8.15 | 9.04 |
| **HyperspaceDB** | 2.10 | 1.53 | 5.29 | 8.22 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **HyperspaceDB**
- **17,721 QPS**

- 1.57x faster than Milvus
- 11.15x faster than Qdrant
- 36.08x faster than Weaviate

### Search Latency Winner: 🏆 **HyperspaceDB**
- **8.22 ms** (p99)

- 1.96x faster than Milvus
- 1.81x faster than Qdrant
- 1.10x faster than Weaviate

---

## Raw Data (JSON)

```json
[
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 11268.587748547208,
    "insert_total_time": 88.74226498603821,
    "search_avg_ms": 6.693858790397644,
    "search_p50_ms": 6.128072738647461,
    "search_p95_ms": 9.435892105102539,
    "search_p99_ms": 16.115188598632812,
    "memory_mb": 0.0,
    "disk_usage_mb": 18561.0,
    "errors": []
  },
  {
    "database": "Qdrant",
    "version": "latest",
    "insert_qps": 1588.7940021511226,
    "insert_total_time": 629.4082169532776,
    "search_avg_ms": 7.996809124946594,
    "search_p50_ms": 7.646083831787109,
    "search_p95_ms": 11.17706298828125,
    "search_p99_ms": 14.842748641967773,
    "memory_mb": 0.0,
    "disk_usage_mb": 4187.0,
    "errors": []
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 491.0907904857622,
    "insert_total_time": 2036.2833499908447,
    "search_avg_ms": 6.728544020652771,
    "search_p50_ms": 6.510019302368164,
    "search_p95_ms": 8.146047592163086,
    "search_p99_ms": 9.042978286743164,
    "memory_mb": 0.0,
    "disk_usage_mb": 5057.0,
    "errors": []
  },
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 17720.97928768356,
    "insert_total_time": 56.43028998374939,
    "search_avg_ms": 2.1044078350067137,
    "search_p50_ms": 1.52587890625,
    "search_p95_ms": 5.292654037475586,
    "search_p99_ms": 8.215904235839844,
    "memory_mb": 0.0,
    "disk_usage_mb": 9003.308634757996,
    "errors": []
  }
]
```
