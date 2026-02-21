# Modular Benchmark Report

Testing on **Synthetic/Random** with **50,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,536 | Euclidean | Cosine | 13,473 | 1,214 | 0.90ms | 90.6% | 90.6% | 1.00 | 0.94 | 1,019 | 1,432 | 1,402 | 756.0M |
| **Milvus** | 1,536 | Euclidean | Cosine | 10,777 | 332 | 4.63ms | 92.3% | 92.3% | 1.00 | 0.95 | 358 | 2,797 | 3,774 | 2.20G |
| **Qdrant** | 1,536 | Euclidean | Cosine | 1,283 | 372 | 4.76ms | 99.9% | 99.9% | 1.00 | 1.00 | 400 | 1,657 | 1,811 | 336.0M |
| **Weaviate** | 1,536 | Euclidean | Cosine | 769 | 115 | 11.36ms | 99.7% | 99.7% | 1.00 | 1.00 | 118 | 607 | 793 | 351.6M |
