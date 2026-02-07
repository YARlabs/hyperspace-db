# Vector Database Benchmark Suite

**Reproducible, fair comparison** of HyperspaceDB against Qdrant, Weaviate, and Milvus.

## Quick Start

```bash
cd benchmarks
./run_all.sh
```

This will:
1. Build HyperspaceDB Docker image
2. Start all databases (HyperspaceDB, Qdrant, Weaviate, Milvus)
3. Run identical workload on each database
4. Generate comparison report

**Duration**: ~10 minutes  
**Requirements**: Docker, Python 3.8+

---

## What Gets Tested

### Workload
- **Vectors**: 100,000
- **Dimensions**: 1,024
- **Batch Size**: 1,000
- **Search Queries**: 1,000
- **Top-K**: 10

### Metrics
1. **Insert Throughput** (QPS)
2. **Insert Total Time** (seconds)
3. **Search Latency** (avg, p50, p95, p99)

---

## Manual Setup

### 1. Start Databases

```bash
docker-compose up -d
```

Wait for health checks:
```bash
# Check all are running
docker-compose ps

# Check health endpoints
curl http://localhost:50051/health  # HyperspaceDB
curl http://localhost:6333/health   # Qdrant
curl http://localhost:8080/v1/.well-known/ready  # Weaviate
curl http://localhost:9091/healthz  # Milvus
```

### 2. Install Python Dependencies

```bash
pip install numpy qdrant-client weaviate-client pymilvus
```

### 3. Run Benchmark

```bash
python3 run_benchmark.py
```

Results saved to `BENCHMARK_RESULTS.md`.

---

## Configuration

Edit `run_benchmark.py` to change:

```python
@dataclass
class BenchmarkConfig:
    dimensions: int = 1024        # Vector dimensions
    num_vectors: int = 100_000    # Total vectors to insert
    batch_size: int = 1000        # Insert batch size
    search_queries: int = 1000    # Number of search queries
    top_k: int = 10               # Top-K results
```

---

## Database Versions

| Database | Version | Docker Image |
|----------|---------|--------------|
| **HyperspaceDB** | 1.4.0 | Built from source |
| **Qdrant** | 1.7.4 | `qdrant/qdrant:v1.7.4` |
| **Weaviate** | 1.23.1 | `semitechnologies/weaviate:1.23.1` |
| **Milvus** | 2.3.3 | `milvusdb/milvus:v2.3.3` |

---

## Reproducibility

### Hardware
- **CPU**: Document your CPU (e.g., M4 Pro, Intel i9-12900K)
- **RAM**: Document available RAM (e.g., 64GB)
- **Disk**: SSD recommended

### Environment
```bash
# Print system info
uname -a
docker --version
python3 --version

# Print CPU info (macOS)
sysctl -n machdep.cpu.brand_string

# Print CPU info (Linux)
lscpu | grep "Model name"
```

### Seed
Benchmark uses **fixed random seed (42)** for reproducible vector generation.

---

## Cleanup

```bash
# Stop all databases
docker-compose down

# Remove volumes (reset data)
docker-compose down -v
```

---

## Troubleshooting

### HyperspaceDB not starting
```bash
# Check logs
docker-compose logs hyperspace

# Rebuild image
cd ..
docker build -t hyperspace-db:benchmark .
cd benchmarks
docker-compose up -d hyperspace
```

### Qdrant connection refused
```bash
# Wait longer for startup
sleep 30

# Check logs
docker-compose logs qdrant
```

### Weaviate schema errors
```bash
# Reset Weaviate
docker-compose restart weaviate
sleep 10
```

### Milvus dependencies not starting
```bash
# Check etcd and minio
docker-compose logs milvus-etcd
docker-compose logs milvus-minio

# Restart all
docker-compose restart
```

---

## Output Format

### Markdown Report
```markdown
# Vector Database Benchmark Results

## Insert Performance
| Database | QPS | Total Time |
|----------|-----|------------|
| HyperspaceDB | 9,087 | 11.0s |
| Qdrant | 3,456 | 28.9s |
...

## Search Performance
| Database | P99 (ms) |
|----------|----------|
| HyperspaceDB | 0.18 |
| Qdrant | 1.24 |
...
```

### JSON Data
Full results in JSON format at end of report for programmatic analysis.

---

## Contributing

To add a new database:

1. Add service to `docker-compose.yml`
2. Implement `benchmark_<database>()` in `run_benchmark.py`
3. Add to `run_all()` method
4. Update this README

---

## License

Benchmark code: MIT  
HyperspaceDB: AGPLv3

---

## Citation

If you use these benchmarks in research or publications:

```bibtex
@misc{hyperspace_benchmark_2026,
  title={HyperspaceDB Vector Database Benchmark Suite},
  author={YAR Labs},
  year={2026},
  url={https://github.com/YARlabs/hyperspace-db}
}
```
