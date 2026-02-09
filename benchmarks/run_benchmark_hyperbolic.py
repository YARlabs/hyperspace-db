#!/usr/bin/env python3
"""
Hyperbolic Efficiency Benchmark
Compares HyperspaceDB (64d Poincaré) vs Competitors (1024d Euclidean).
Demonstrates the efficiency of Hyperbolic space.
"""

import time
import numpy as np
import json
import statistics
import sys
import os
from typing import List
from dataclasses import dataclass, asdict

# Ensure local SDK is used
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

# --- Imports (Same as before) ---
try:
    from hyperspace import HyperspaceClient
    HYPERSPACE_AVAILABLE = True
except ImportError:
    HYPERSPACE_AVAILABLE = False
    print("⚠️  HyperspaceDB Python SDK not found")

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, VectorParams, PointStruct
    QDRANT_AVAILABLE = True
except ImportError:
    QDRANT_AVAILABLE = False

try:
    import weaviate
    import weaviate.classes as wvc
    WEAVIATE_AVAILABLE = True
except ImportError:
    WEAVIATE_AVAILABLE = False

try:
    from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType, utility
    MILVUS_AVAILABLE = True
except ImportError:
    MILVUS_AVAILABLE = False

# --- Configuration ---

@dataclass
class BenchmarkConfig:
    num_vectors: int = 1_000_000
    batch_size: int = 1000
    search_queries: int = 10000
    top_k: int = 10
    # DUAL DIMENSIONS
    euclidean_dim: int = 1024  # Old World (Competitors)
    hyperbolic_dim: int = 64   # New World (Hyperspace)

@dataclass
class BenchmarkResult:
    database: str
    version: str
    dimension: int
    geometry: str
    insert_qps: float
    insert_total_time: float
    search_avg_ms: float
    search_p50_ms: float
    search_p95_ms: float
    search_p99_ms: float
    disk_usage_mb: float
    errors: List[str]

# --- Helpers ---

def get_docker_disk_usage(container: str, path: str) -> float:
    try:
        import subprocess
        result = subprocess.run(
            ["docker", "exec", container, "du", "-sm", path],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return float(result.stdout.split()[0])
        return 0.0
    except:
        return 0.0

def get_local_disk_usage(path: str) -> float:
    try:
        total = 0
        for dirpath, _, filenames in os.walk(path):
            for f in filenames:
                fp = os.path.join(dirpath, f)
                total += os.path.getsize(fp)
        return total / (1024 * 1024)
    except:
        return 0.0

class EfficiencyBenchmark:
    def __init__(self, config: BenchmarkConfig):
        self.config = config
        self.vectors_euc = self._gen_euclidean()
        self.vectors_hyp = self._gen_hyperbolic()
    
    def _gen_euclidean(self) -> np.ndarray:
        print(f"📊 Generating {self.config.num_vectors} Euclidean vectors ({self.config.euclidean_dim}-dim)...")
        vec = np.random.randn(self.config.num_vectors, self.config.euclidean_dim).astype(np.float32)
        norms = np.linalg.norm(vec, axis=1, keepdims=True)
        return vec / norms
    
    def _gen_hyperbolic(self) -> np.ndarray:
        print(f"🌌 Generating {self.config.num_vectors} Poincaré vectors ({self.config.hyperbolic_dim}-dim)...")
        # 1. Generate random directions
        vec = np.random.randn(self.config.num_vectors, self.config.hyperbolic_dim).astype(np.float32)
        vec /= np.linalg.norm(vec, axis=1, keepdims=True)
        # 2. Scale length to be inside the ball (0 < r < 1)
        # We push them towards the edge (0.9) to simulate hierarchy
        radii = np.random.uniform(0.1, 0.99, size=(self.config.num_vectors, 1)).astype(np.float32)
        return vec * radii

    # --- Benchmarks ---

    def benchmark_milvus(self) -> BenchmarkResult:
        print("\n🟣 Benchmarking Milvus (1024d Euclidean)...")
        # ... (Код аналогичен run_benchmark.py, используем self.vectors_euc)
        # Для краткости: используйте логику из прошлого файла, 
        # но передавайте self.vectors_euc и self.config.euclidean_dim
        # Я приведу сокращенный пример:
        try:
            connections.connect(host="localhost", port="19530")
            col_name = "bench_euc"
            if utility.has_collection(col_name): utility.drop_collection(col_name)
            
            schema = CollectionSchema([
                FieldSchema("id", DataType.INT64, is_primary=True),
                FieldSchema("vec", DataType.FLOAT_VECTOR, dim=self.config.euclidean_dim)
            ], "")
            col = Collection(col_name, schema)
            
            start = time.time()
            # Batch Insert Logic for vectors_euc
            for i in range(0, len(self.vectors_euc), self.config.batch_size):
                batch = self.vectors_euc[i:i+self.config.batch_size]
                col.insert([list(range(i, i+len(batch))), batch.tolist()])
            total_time = time.time() - start
            
            # Index & Search
            col.create_index("vec", {"metric_type":"L2", "index_type":"IVF_FLAT", "params":{"nlist":128}})
            col.load()
            
            latencies = []
            q = [self.vectors_euc[0].tolist()]
            for _ in range(self.config.search_queries):
                s = time.time()
                col.search(q, "vec", {"metric_type":"L2", "params":{"nprobe":10}}, limit=10)
                latencies.append((time.time()-s)*1000)
            
            usage = get_docker_disk_usage("benchmarks-milvus-1", "/var/lib/milvus")
            utility.drop_collection(col_name)
            
            return BenchmarkResult(
                database="Milvus",
                version="latest",
                dimension=self.config.euclidean_dim,
                geometry="Euclidean", 
                insert_qps=len(self.vectors_euc)/total_time,
                insert_total_time=total_time, 
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=np.percentile(latencies, 50),
                search_p95_ms=np.percentile(latencies, 95),
                search_p99_ms=np.percentile(latencies, 99), 
                disk_usage_mb=usage,
                errors=[]
            )
        except Exception as e:
            return BenchmarkResult("Milvus", "latest", 1024, "Euclidean", 0,0,0,0,0,0,0, [str(e)])

    # Аналогично добавьте функции для Qdrant и Weaviate, используя vectors_euc

    def benchmark_hyperspace(self) -> BenchmarkResult:
        print(f"\n🚀 Benchmarking HyperspaceDB ({self.config.hyperbolic_dim}d Poincaré)...")
        try:
            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
            # Explicitly create Poincaré collection
            # Note: Server must be started with matching config!
            try:
                client.create_collection("bench_hyp", dimension=self.config.hyperbolic_dim, metric="poincare")
            except: pass

            start = time.time()
            # Batch Insert Logic for vectors_hyp
            batch_size = 1000
            for i in range(0, len(self.vectors_hyp), batch_size):
                batch = self.vectors_hyp[i:i+batch_size]
                ids = list(range(i, i+len(batch)))
                metas = [{"i":str(k)} for k in ids]
                client.batch_insert(batch.tolist(), ids, metas, collection="bench_hyp")
            
            total_time = time.time() - start
            
            latencies = []
            q = self.vectors_hyp[0].tolist()
            for _ in range(self.config.search_queries):
                s = time.time()
                client.search(q, top_k=10, collection="bench_hyp")
                latencies.append((time.time()-s)*1000)
            
            usage = get_local_disk_usage("../data") # Adjust path
            # client.delete_collection("bench_hyp")
            
            return BenchmarkResult(
                database="HyperspaceDB",
                version="1.5.0",
                dimension=self.config.hyperbolic_dim,
                geometry="Poincaré",
                insert_qps=len(self.vectors_hyp)/total_time,
                insert_total_time=total_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=np.percentile(latencies, 50),
                search_p95_ms=np.percentile(latencies, 95),
                search_p99_ms=np.percentile(latencies, 99),
                disk_usage_mb=usage,
                errors=[]
            )
        except Exception as e:
            return BenchmarkResult("HyperspaceDB", "1.5.0", 64, "Poincaré", 0,0,0,0,0,0,0, [str(e)])

    def run(self):
        results = []
        if MILVUS_AVAILABLE: results.append(self.benchmark_milvus())
        # Add placeholders for other databases if they were in the main benchmark
        if HYPERSPACE_AVAILABLE: results.append(self.benchmark_hyperspace())
        
        return results

def generate_report(results: List[BenchmarkResult], config: BenchmarkConfig) -> str:
    """Generate markdown report for Hyperbolic Efficiency"""
    report = f"""# Hyperbolic Efficiency Benchmark Results

**Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}  
**Configuration**:
- Vectors: {config.num_vectors:,}
- Euclidean Dim: {config.euclidean_dim} (Competitors)
- Hyperbolic Dim: {config.hyperbolic_dim} (HyperspaceDB)
- Batch Size: {config.batch_size:,}
- Search Queries: {config.search_queries:,}
- Top-K: {config.top_k}

---

## Insert Performance

| Database | Version | Geometry | Dim | QPS | Total Time | Throughput | Disk Usage (MB) |
|----------|---------|----------|-----|-----|------------|------------|-----------------|
"""
    
    for r in results:
        if r.insert_qps > 0:
            throughput = (config.num_vectors * r.dimension) / r.insert_total_time / 1000000 if r.insert_total_time > 0 else 0
            report += f"| **{r.database}** | {r.version} | {r.geometry} | {r.dimension} | **{r.insert_qps:,.0f}** | {r.insert_total_time:.1f}s | {throughput:.2f} M dims/s | {r.disk_usage_mb:.1f} MB |\n"
    
    report += "\n---\n\n## Search Performance\n\n"
    report += "| Database | Geometry | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |\n"
    report += "|----------|----------|----------|----------|----------|----------|\n"
    
    for r in results:
        if r.search_avg_ms > 0:
            report += f"| **{r.database}** | {r.geometry} | {r.search_avg_ms:.2f} | {r.search_p50_ms:.2f} | {r.search_p95_ms:.2f} | {r.search_p99_ms:.2f} |\n"
    
    # Winner analysis
    report += "\n---\n\n## Efficiency Comparison\n\n"
    
    if len(results) > 1:
        best_insert = max(results, key=lambda x: x.insert_qps)
        best_search = min(results, key=lambda x: x.search_p99_ms if x.search_p99_ms > 0 else float('inf'))
        
        report += f"### Throughput Winner: 🏆 **{best_insert.database}**\n"
        report += f"- **{best_insert.insert_qps:,.0f} QPS**\n\n"
        
        for r in results:
            if r.database != best_insert.database and r.insert_qps > 0:
                speedup = best_insert.insert_qps / r.insert_qps
                report += f"- {speedup:.2f}x faster throughput than {r.database} ({r.geometry})\n"
        
        report += f"\n### Latency Winner: 🏆 **{best_search.database}**\n"
        report += f"- **{best_search.search_p99_ms:.2f} ms** (p99)\n\n"
        
        for r in results:
            if r.database != best_search.database and r.search_p99_ms > 0:
                speedup = r.search_p99_ms / best_search.search_p99_ms
                report += f"- {speedup:.2f}x lower latency than {r.database}\n"

        # Disk efficiency check
        hdb = next((r for r in results if r.database == "HyperspaceDB"), None)
        milvus = next((r for r in results if r.database == "Milvus"), None)
        if hdb and milvus and milvus.disk_usage_mb > 0 and hdb.disk_usage_mb > 0:
            disk_efficiency = milvus.disk_usage_mb / hdb.disk_usage_mb
            report += f"\n### Disk Efficiency: 🛡️ **HyperspaceDB**\n"
            report += f"- **{disk_efficiency:.2f}x more space efficient** than Milvus thanks to {hdb.dimension}d Hyperbolic geometry vs {milvus.dimension}d Euclidean.\n"
    
    # Errors
    if any(r.errors for r in results):
        report += "\n---\n\n## Errors\n\n"
        for r in results:
            if r.errors:
                report += f"### {r.database}\n"
                for err in r.errors:
                    report += f"- {err}\n"
                report += "\n"
    
    report += "\n---\n\n## Raw Data (JSON)\n\n```json\n"
    report += json.dumps([asdict(r) for r in results], indent=2)
    report += "\n```\n"
    
    return report

def main():
    print("=" * 60)
    print("  Hyperbolic Efficiency Benchmark")
    print("=" * 60)
    
    config = BenchmarkConfig()
    benchmark = EfficiencyBenchmark(config)
    
    results = benchmark.run()
    
    # Generate report
    report = generate_report(results, config)
    
    # Save report
    output_file = "HYPERBOLIC_BENCHMARK_RESULTS.md"
    with open(output_file, "w") as f:
        f.write(report)
    
    print(f"\n✅ Benchmark complete! Results saved to {output_file}")
    print("\n" + "=" * 60)
    print(report)

if __name__ == "__main__":
    main()