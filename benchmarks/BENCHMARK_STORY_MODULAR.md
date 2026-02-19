# Modular Benchmark Report

Testing on **Synthetic/Random** with **50,000** docs and **1,000** queries.

| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Hyperspace** | 1,536 | Euclidean | Cosine | 14,045 | 427 | 2.69ms | 90.0% | 90.0% | 1.00 | 0.93 | 581 | 4,634 | 4,997 | 756.0M |
