# Modular Benchmark Report

Testing on **Synthetic/Random** with **50,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,536 | Euclidean | Cosine | 14,681 | 881 | 1.45ms | 88.7% | 88.7% | 1.00 | 0.92 | 1,340 | 10,558 | 11,306 | 756.0M |
| **Milvus** | 1,536 | Euclidean | Cosine | 11,241 | 428 | 3.93ms | 92.7% | 92.7% | 1.00 | 0.95 | 487 | 3,120 | 4,073 | 42.00G |
| **Qdrant** | 1,536 | Euclidean | Cosine | 1,292 | 434 | 3.17ms | 99.8% | 99.8% | 1.00 | 1.00 | 472 | 1,989 | 2,202 | 366.0M |
