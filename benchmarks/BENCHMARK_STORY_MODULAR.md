# Modular Benchmark Report

Testing on **Synthetic/Random** with **1,000,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,024 | Euclidean | Cosine | 20,494 | 390 | 3.83ms | 87.6% | 87.6% | 1.00 | 0.91 | 554 | 4,332 | 4,367 | 9.20G |
| **Milvus** | 1,024 | Euclidean | Cosine | 16,748 | 150 | 11.65ms | 88.7% | 88.7% | 1.00 | 0.92 | 155 | 282 | 373 | 47.00G |
| **Qdrant** | 1,024 | Euclidean | Cosine | 1,887 | 409 | 3.29ms | 92.3% | 92.3% | 1.00 | 0.95 | 495 | 2,062 | 2,379 | 4.10G |
| **Weaviate** | 1,024 | Euclidean | Cosine | 985 | 129 | 16.90ms | 89.9% | 89.9% | 0.99 | 0.93 | 154 | 874 | 1,014 | 4.90G |
