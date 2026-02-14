# 📐 The Hyperbolic Advantage: Full Accuracy Suite

Testing with **1,000,000** nodes. Accuracy based on **1000** query vectors.
HyperspaceDB Mode: **Euclidean 1024d**

| Database | Dim | Geometry | Metric | QPS | P99 | Recall@10 | MRR | NDCG@10 | C1 QPS | C10 QPS | C30 QPS | Disk |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Milvus** | 1024 | Euclidean | Cosine | 12,416 | 5.86ms | 26.3% | 1.00 | 0.42 | 262 | 991 | 1,414 | 14G |
| **ChromaDB** | 1024 | Euclidean | Cosine | 1,041 | 13.21ms | 10.2% | 0.69 | 0.20 | 116 | 209 | 221 | 9.8G |
| **Qdrant** | 1024 | Euclidean | Cosine | 2,430 | 13.23ms | 33.7% | 1.00 | 0.49 | 180 | 984 | 982 | 1.4G |
| **HyperspaceDB** | 1024 | Euclidean | cosine | 17,854 | 18.99ms | 1.5% | 0.14 | 0.03 | 168 | 569 | 679 | 5784.0M |

## 💡 Accuracy Analysis
HyperspaceDB is currently tested in Euclidean mode. Point the server to Poincaré to see the Hyperbolic Advantage.
