# Hyperbolic Efficiency Benchmark Results

**Date**: 2026-02-09 15:58:45  
**Configuration**:
- Vectors: 1,000,000
- Euclidean Dim: 1024 (Competitors)
- Hyperbolic Dim: 64 (HyperspaceDB)
- Batch Size: 1,000
- Search Queries: 10,000
- Top-K: 10

---

## Insert Performance

| Database | Version | Geometry | Dim | QPS | Total Time | Throughput | Disk Usage (MB) |
|----------|---------|----------|-----|-----|------------|------------|-----------------|
| **HyperspaceDB** | 1.5.0 | Poincaré | 64 | **156,587** | 6.4s | 10.02 M dims/s | 687.6 MB |

---

## Search Performance

| Database | Geometry | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |
|----------|----------|----------|----------|----------|----------|
| **HyperspaceDB** | Poincaré | 1.15 | 1.07 | 2.02 | 2.47 |

---

## Efficiency Comparison

### Throughput Winner: 🏆 **HyperspaceDB**
- **156,587 QPS**


### Latency Winner: 🏆 **HyperspaceDB**
- **2.47 ms** (p99)

---

## Raw Data (JSON)

```json
[
  {
    "database": "HyperspaceDB",
    "version": "1.5.0",
    "dimension": 64,
    "geometry": "Poincar\u00e9",
    "insert_qps": 156587.14639639162,
    "insert_total_time": 6.386220216751099,
    "search_avg_ms": 1.1498857975006103,
    "search_p50_ms": 1.0710954666137695,
    "search_p95_ms": 2.0230293273925772,
    "search_p99_ms": 2.4650311470031743,
    "disk_usage_mb": 687.6159706115723,
    "errors": []
  }
]
```
