# HyperspaceDB Benchmarks

> **⚠️ ATTENTION:** Don't take anyone's word for it, verify all numbers yourself! We provide the exact scripts used to generate our results so you can reproduce them on your own hardware.

## 1. Project Overview

This directory contains reproducible benchmark tooling for HyperspaceDB and other vector databases.
The main goal is to measure throughput, latency, and retrieval quality on the same datasets and query sets.

The project now includes:
- a **modular plugin-based runner** (`run_benchmark.py`) for scalable adapter growth (add you DB or custom metrics, if you want);

## 2. Core Functionalities

- Run benchmark against all supported databases at once.
- Run benchmark for only one database adapter.
- Add new database by creating one plugin file in `db_plugins/adapters/`.
- Reuse the same data preparation and metric logic across adapters.
- Compare legacy and modular reports to track metric parity.
- Run durability benchmark (`run_durability_benchmark.py`) independently.

## 3. Docs and Libraries

### Main references
- HuggingFace `datasets` for dataset loading.
- `vectordb-bench` for standardized benchmark cases.
- DB SDKs: `pymilvus`, `qdrant-client`, `chromadb`, Hyperspace Python SDK.
- `torch`, `transformers`, `peft` for embedding generation.

### Install
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
pip install -e ../sdks/python
```

### Start DB stack
```bash
docker-compose up -d
```

## 4. Current File Structure (Snapshot)

```text
benchmarks/
├── README.md
├── requirements.txt
├── docker-compose.yml
├── download_dataset.py
├── run_benchmark.py
├── run_durability_benchmark.py
├── plugin_runtime.py
├── db_plugins/
    ├── __init__.py
    ├── base.py
    ├── registry.py
    └── adapters/
        ├── __init__.py
        ├── chroma_plugin.py
        ├── hyperspace_plugin.py
        ├── milvus_plugin.py
        └── qdrant_plugin.py

```

## 5. Run Commands

### Benchmark runner
```bash
python3 run_benchmark.py hyper --case=Performance1024D1M
python3 run_benchmark.py --case=Performance1024D1M
python3 run_benchmark.py hyper
```

## 6. Benchmark Metrics

- Throughput (Insert/Search QPS)
- Latency (P50/P95/P99)
- Recall@10, MRR@10, NDCG@10
- System Recall@10 (vs exact brute-force)
- Concurrency profile (C1/C10/C30)
- Disk usage
