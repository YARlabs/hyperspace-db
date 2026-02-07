# Vector Database Benchmark Results

**Date**: 2026-02-07 13:32:28  
**Configuration**:
- Vectors: 100,000
- Dimensions: 1024
- Batch Size: 1,000
- Search Queries: 1,000
- Top-K: 10

---

## Insert Performance

| Database | Version | QPS | Total Time | Throughput | Mem (MB) | CPU (%) | Disk (MB) |
|----------|---------|-----|------------|------------|----------|---------|-----------|
| **HyperspaceDB** | 1.5.0 | **712** | 140.4s | 0.73 M dims/s | 316.7 | 898.3 | 1146.7 |

---

## Search Performance

| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|
| **HyperspaceDB** | 1.65 | 1.26 | 5.36 | 9.64 |

---

## Performance Comparison

### Insert Throughput Winner: 🏆 **HyperspaceDB**
- **712 QPS**


### Search Latency Winner: 🏆 **HyperspaceDB**
- **9.64 ms** (p99)


---

## Errors

### Qdrant
- [Errno 61] Connection refused

### Weaviate
- Connection to Weaviate failed. Details: Error: [Errno 61] Connection refused. 
Is Weaviate running and reachable at http://localhost:8080?

### Milvus
- <MilvusException: (code=2, message=Fail connecting to server on localhost:19530, illegal connection params or server unavailable)>


---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "insert_qps": 712.0453205014569,
    "insert_total_time": 140.44049882888794,
    "search_avg_ms": 1.6469366550445557,
    "search_p50_ms": 1.2600421905517578,
    "search_p95_ms": 5.357027053833008,
    "search_p99_ms": 9.64498519897461,
    "memory_mb": 316.734375,
    "cpu_percent": 898.2867938931298,
    "disk_usage_mb": 1146.7105922698975,
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
    "cpu_percent": 0.0,
    "disk_usage_mb": 0.0,
    "errors": [
      "[Errno 61] Connection refused"
    ]
  },
  {
    "database": "Weaviate",
    "version": "latest",
    "insert_qps": 0,
    "insert_total_time": 0,
    "search_avg_ms": 0,
    "search_p50_ms": 0,
    "search_p95_ms": 0,
    "search_p99_ms": 0,
    "memory_mb": 0,
    "cpu_percent": 0.0,
    "disk_usage_mb": 0.0,
    "errors": [
      "Connection to Weaviate failed. Details: Error: [Errno 61] Connection refused. \nIs Weaviate running and reachable at http://localhost:8080?"
    ]
  },
  {
    "database": "Milvus",
    "version": "latest",
    "insert_qps": 0,
    "insert_total_time": 0,
    "search_avg_ms": 0,
    "search_p50_ms": 0,
    "search_p95_ms": 0,
    "search_p99_ms": 0,
    "memory_mb": 0,
    "cpu_percent": 0.0,
    "disk_usage_mb": 0.0,
    "errors": [
      "<MilvusException: (code=2, message=Fail connecting to server on localhost:19530, illegal connection params or server unavailable)>"
    ]
  }
]
```
