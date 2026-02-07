# Quick Start: Reproducible Benchmarks

Run **real, verifiable benchmarks** comparing HyperspaceDB with Qdrant, Weaviate, and Milvus.

## One-Command Benchmark

```bash
cd benchmarks && ./run_all.sh
```

**Duration**: ~10 minutes  
**Output**: `benchmarks/BENCHMARK_RESULTS.md`

---

## What It Does

1. ✅ Builds HyperspaceDB from source
2. ✅ Starts Qdrant, Weaviate, Milvus in Docker
3. ✅ Runs identical workload on each:
   - Insert 100,000 vectors (1024-dim)
   - Search 1,000 queries
4. ✅ Generates comparison report

---

## Requirements

- **Docker** 20.10+
- **Python** 3.8+
- **20GB** free disk space
- **16GB** RAM minimum

---

## Example Output

```markdown
# Vector Database Benchmark Results

## Insert Performance
| Database      | QPS    | Total Time |
|---------------|--------|------------|
| HyperspaceDB  | 9,087  | 11.0s      |
| Qdrant        | 3,456  | 28.9s      |
| Weaviate      | 2,789  | 35.8s      |
| Milvus        | 4,123  | 24.3s      |

## Search Performance (p99)
| Database      | Latency |
|---------------|---------|
| HyperspaceDB  | 0.18ms  |
| Qdrant        | 0.52ms  |
| Weaviate      | 0.71ms  |
| Milvus        | 0.63ms  |
```

---

## Manual Steps

If you want more control:

```bash
# 1. Start databases
cd benchmarks
docker-compose up -d

# 2. Wait for health checks
sleep 30

# 3. Install Python deps
pip install numpy qdrant-client weaviate-client pymilvus

# 4. Run benchmark
python3 run_benchmark.py

# 5. View results
cat BENCHMARK_RESULTS.md
```

---

## Customize Workload

Edit `benchmarks/run_benchmark.py`:

```python
@dataclass
class BenchmarkConfig:
    dimensions: int = 1024        # Change vector size
    num_vectors: int = 100_000    # Change dataset size
    batch_size: int = 1000        # Change batch size
    search_queries: int = 1000    # Change query count
    top_k: int = 10               # Change top-k
```

---

## Troubleshooting

### Docker not found
```bash
# Install Docker: https://docs.docker.com/get-docker/
```

### Python packages fail
```bash
pip install --upgrade pip
pip install numpy qdrant-client weaviate-client pymilvus
```

### Databases not starting
```bash
# Check logs
docker-compose logs

# Restart
docker-compose restart

# Full reset
docker-compose down -v
docker-compose up -d
```

---

## Share Your Results

Run the benchmark and share your results!

1. Run: `cd benchmarks && ./run_all.sh`
2. Copy: `benchmarks/BENCHMARK_RESULTS.md`
3. Share: GitHub issue, Twitter, Reddit

Include your hardware specs:
```bash
# macOS
sysctl -n machdep.cpu.brand_string

# Linux
lscpu | grep "Model name"
```

---

## Full Documentation

See [`benchmarks/README.md`](benchmarks/README.md) for complete details.
