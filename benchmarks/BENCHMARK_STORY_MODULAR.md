# Modular Benchmark Report

Testing on **Synthetic/Random** with **50,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,536 | Euclidean | Cosine | 11,855 | 2,377 | 0.57ms | 90.6% | 90.6% | 1.00 | 0.94 | 2,518 | 3,316 | 3,263 | 756.0M |
| **Milvus** | 1,536 | Euclidean | Cosine | 10,818 | 328 | 5.31ms | 92.9% | 92.9% | 1.00 | 0.95 | 347 | 2,524 | 3,443 | 6.00G |
| **Qdrant** | 1,536 | Euclidean | Cosine | 1,257 | 376 | 4.04ms | 99.9% | 99.9% | 1.00 | 1.00 | 413 | 1,737 | 1,917 | 336.0M |
| **Weaviate** | 1,536 | Euclidean | Cosine | 749 | 94 | 16.30ms | 99.7% | 99.7% | 1.00 | 1.00 | 109 | 594 | 722 | 351.5M |
