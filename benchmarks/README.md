# HyperspaceDB Benchmarks

This directory contains the tools and scripts used to benchmark HyperspaceDB against other popular vector databases (Milvus, Qdrant, Weaviate, Chroma).

> **⚠️ ATTENTION:** Don't take anyone's word for it, verify all numbers yourself! We provide the exact scripts used to generate our results so you can reproduce them on your own hardware.

## 🚀 One-Click Reproducibility

We provide a specialized script that automates the entire benchmarking process:

```bash
./run_benchmark.sh
```

**What this script does:**
1.  **Virtual Env**: Creates a clean Python `venv`.
2.  **Dependencies**: Installs all required packages and the Hyperspace SDK.
3.  **Infrastructure**: Deploys the full stack (HyperspaceDB + Competitors) using `docker-compose`.
4.  **Execution**: Runs the `Performance1024D1M` benchmark case (1 million vectors, 1024 dimensions).

## 🛠 Manual Execution

If you prefer to run specific steps manually:

### 1. Setup Environment
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
pip install -e ../sdks/python
```

### 2. Start Databases
```bash
docker-compose up -d
```

### 3. Download Real-World Datasets
Use the helper tool to download standard benchmark datasets (msmarco, glove, etc.):
```bash
python3 download_dataset.py
```

### 4. Run Benchmarks
Run the main benchmarking suite:
```bash
python3 run_benchmark.py --case=Performance1024D1M
python3 run_benchmark.py --case=Performance768D1M
```

## 🧪 Benchmark Scripts

- `run_benchmark.py`: The primary engine for performance comparison (QPS, Recall, Latency).
- `run_durability_benchmark.py`: Tests ingestion speed under different WAL durability settings (Async vs. Strict).
- `download_dataset.py`: Automation script for fetching VectorDBBench datasets from S3.

## 📊 Evaluation Metrics

- **Throughput (QPS)**: Number of queries processed per second.
- **P99 Latency**: The response time for the 99th percentile of queries.
- **Recall@K**: The accuracy of the search results compared to a ground-truth exact search.
- **Indexing Speed**: Time taken to ingest and index large batches of vectors.

---
*© 2026 YARlabs - High Performance Hyperbolic Systems*
