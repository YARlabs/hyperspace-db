# Modular Benchmark Report

Testing on **Synthetic/Random** with **1,000,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,024 | Euclidean | Cosine | 7,469 | 1,859 | 0.70ms | 89.6% | 89.6% | 1.00 | 0.93 | 2,394 | 4,051 | 4,121 | 9.17G |
| **Milvus** | 1,024 | Euclidean | Cosine | 7,030 | 149 | 13.04ms | 88.8% | 88.8% | 1.00 | 0.92 | 172 | 364 | 456 | 4.20G |
| **Qdrant** | 1,024 | Euclidean | Cosine | 1,583 | 113 | 31.87ms | 91.3% | 91.3% | 1.00 | 0.94 | 423 | 2,046 | 2,215 | 4.10G |
| **ChromaDB** | 1,024 | Euclidean | Cosine | 1,243 | 812 | 1.83ms | 54.3% | 54.3% | 0.83 | 0.61 | 1,168 | 466 | 446 | 8.41G |
| **Weaviate** | 1,024 | Euclidean | Cosine | 981 | 147 | 9.02ms | 89.9% | 89.9% | 0.99 | 0.93 | 160 | 817 | 989 | 4.90G |
