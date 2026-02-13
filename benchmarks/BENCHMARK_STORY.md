# 📐 The Hyperbolic Advantage: Absolute Benchmark

Testing with **100,000** nodes in a hierarchical taxonomy.
| Database | Geometry | Metric | Dim | Ingest QPS | Search P99 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Milvus** | Euclidean | L2 | 1024 | 16,934 | 3.68 ms | 19G |
| **HyperspaceDB** | Euclidean | cosine | 1024 | 17,235 | 6.10 ms | 999.3M |
| **HyperspaceDB** | Poincaré | poincare | 64 | 121,067 | 6.73 ms | 101.2M |
| **Weaviate** | Euclidean | Cosine | 1024 | 592 | 7.56 ms | 304.0M |
| **Qdrant** | Euclidean | Cosine | 1024 | 1,957 | 12.93 ms | 263M |

## 💡 Key Takeaways
1. **Latency**: HyperspaceDB (64d) is **0.5x faster** than Milvus (1024d).
2. **Efficiency**: HyperspaceDB uses **187.7x less disk** space compared to Milvus.
