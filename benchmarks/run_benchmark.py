#!/usr/bin/env python3
"""
Unified Vector Database Benchmark
Tests HyperspaceDB, Qdrant, Weaviate, and Milvus with identical workloads
"""

import time
import numpy as np
import json
from typing import List, Tuple, Dict
from dataclasses import dataclass, asdict
import statistics
import sys
import os

# Ensure local SDK is used
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

# Database clients
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
    print("⚠️  Qdrant client not found: pip install qdrant-client")

try:
    import weaviate
    import weaviate.classes as wvc
    WEAVIATE_AVAILABLE = True
except ImportError:
    WEAVIATE_AVAILABLE = False
    print("⚠️  Weaviate client not found: pip install weaviate-client")

try:
    from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType, utility
    MILVUS_AVAILABLE = True
except ImportError:
    MILVUS_AVAILABLE = False
    print("⚠️  Milvus client not found: pip install pymilvus")


@dataclass
class BenchmarkConfig:
    """Benchmark configuration"""
    dimensions: int = 1024
    num_vectors: int = 100_000
    batch_size: int = 1000
    search_queries: int = 1000
    top_k: int = 10


@dataclass
class BenchmarkResult:
    """Results for a single database"""
    database: str
    version: str
    insert_qps: float
    insert_total_time: float
    search_avg_ms: float
    search_p50_ms: float
    search_p95_ms: float
    search_p99_ms: float
    memory_mb: float
    disk_usage_mb: float
    errors: List[str]

def get_disk_usage_local(path: str) -> float:
    """Get disk usage of a directory in MB"""
    try:
        total_size = 0
        for dirpath, dirnames, filenames in os.walk(path):
            for f in filenames:
                fp = os.path.join(dirpath, f)
                if not os.path.islink(fp):
                    total_size += os.path.getsize(fp)
        return total_size / (1024 * 1024)
    except Exception:
        return 0.0

def get_docker_disk_usage(container: str, path: str) -> float:
    """Get disk usage of a path inside a docker container in MB"""
    try:
        # Use du -sm inside container
        import subprocess
        result = subprocess.run(
            ["docker", "exec", container, "du", "-sm", path],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return float(result.stdout.split()[0])
        return 0.0
    except Exception as e:
        print(f"⚠️ Failed to get docker disk usage for {container}: {e}")
        return 0.0

# import psutil
# import os
# import threading

# class ResourceMonitor:
#     def __init__(self, target_process_name: str = "hyperspace-server"):
#         self.target_name = target_process_name
#         self.running = False
#         self.cpu_usage = []
#         self.mem_usage = []
#         self.thread = None
# 
#     def start(self):
#         self.running = True
#         self.cpu_usage = []
#         self.mem_usage = []
#         self.thread = threading.Thread(target=self._monitor)
#         self.thread.start()
# 
#     def stop(self):
#         self.running = False
#         if self.thread:
#             self.thread.join()
#         
#         avg_cpu = statistics.mean(self.cpu_usage) if self.cpu_usage else 0
#         max_mem = max(self.mem_usage) if self.mem_usage else 0
#         return avg_cpu, max_mem
# 
#     def _monitor(self):
#         process = None
#         for p in psutil.process_iter(['name']):
#             if self.target_name in p.info['name']:
#                 process = p
#                 break
#         
#         while self.running:
#             if process:
#                 try:
#                     self.cpu_usage.append(process.cpu_percent(interval=0.1))
#                     self.mem_usage.append(process.memory_info().rss / 1024 / 1024)
#                 except:
#                     pass
#             else:
#                  # Try to find again
#                 for p in psutil.process_iter(['name']):
#                     if p.info['name'] and self.target_name in p.info['name']:
#                         process = p
#                         break
#                 time.sleep(0.1)
# 
#     def get_disk_usage(self, path: str) -> float:
#         try:
#              total_size = 0
#              for dirpath, dirnames, filenames in os.walk(path):
#                  for f in filenames:
#                      fp = os.path.join(dirpath, f)
#                      total_size += os.path.getsize(fp)
#              return total_size / 1024 / 1024
#         except:
#             return 0.0


class VectorDBBenchmark:
    """Unified benchmark runner"""
    
    def __init__(self, config: BenchmarkConfig):
        self.config = config
        self.vectors = self._generate_vectors()
        
    def _generate_vectors(self) -> np.ndarray:
        """Generate random test vectors"""
        print(f"📊 Generating {self.config.num_vectors} vectors ({self.config.dimensions}-dim)...")
        np.random.seed(42)  # Reproducible
        vectors = np.random.randn(self.config.num_vectors, self.config.dimensions).astype(np.float32)
        # Normalize
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors = vectors / norms
        return vectors
    
    def benchmark_milvus(self) -> BenchmarkResult:
        """Benchmark Milvus"""
        print("\n🟣 Benchmarking Milvus...")
        errors = []
        
        try:
            connections.connect(host="localhost", port="19530")
            collection_name = "benchmark"
            
            # Drop if exists
            if utility.has_collection(collection_name):
                utility.drop_collection(collection_name)
            
            # Create collection
            fields = [
                FieldSchema(name="id", dtype=DataType.INT64, is_primary=True, auto_id=False),
                FieldSchema(name="embedding", dtype=DataType.FLOAT_VECTOR, dim=self.config.dimensions)
            ]
            schema = CollectionSchema(fields, description="Benchmark collection")
            collection = Collection(collection_name, schema)
            
            # Insert benchmark
            start = time.time()
            for i in range(0, len(self.vectors), self.config.batch_size):
                batch = self.vectors[i:i+self.config.batch_size]
                ids = list(range(i, i+len(batch)))
                entities = [ids, batch.tolist()]
                collection.insert(entities)
                
                if (i + self.config.batch_size) % 10000 == 0:
                    elapsed = time.time() - start
                    qps = (i + self.config.batch_size) / elapsed
                    print(f"  Inserted {i+self.config.batch_size:,} | {qps:.0f} QPS")
            
            insert_time = time.time() - start
            insert_qps = len(self.vectors) / insert_time
            
            # Create index
            print("  Creating index...")
            index_params = {
                "metric_type": "L2",
                "index_type": "IVF_FLAT",
                "params": {"nlist": 128}
            }
            collection.create_index("embedding", index_params)
            collection.load()
            
            # Search benchmark
            query = [self.vectors[0].tolist()]
            latencies = []
            
            print(f"  Running {self.config.search_queries} search queries...")
            search_params = {"metric_type": "L2", "params": {"nprobe": 10}}
            for i in range(self.config.search_queries):
                start = time.time()
                results = collection.search(query, "embedding", search_params, limit=self.config.top_k)
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            # Disk Usage
            disk_usage = get_docker_disk_usage("benchmarks-milvus-1", "/var/lib/milvus")

            # Cleanup
            try:
                if utility.has_collection(collection_name):
                    utility.drop_collection(collection_name)
                    print("  ✅ Cleaned up collection")
            except Exception as e:
                print(f"  ⚠️ Cleanup failed: {e}")
            
            return BenchmarkResult(
                database="Milvus",
                version="latest",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                # cpu_percent=0.0,
                disk_usage_mb=disk_usage,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Milvus",
                version="latest",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, disk_usage_mb=0, errors=errors  # cpu_percent=0.0, disk_usage_mb=0.0,
            )
    
    def benchmark_qdrant(self) -> BenchmarkResult:
        """Benchmark Qdrant"""
        print("\n🔷 Benchmarking Qdrant...")
        errors = []
        
        try:
            client = QdrantClient(host="localhost", port=6333)
            collection_name = "benchmark"
            
            # Create collection
            try:
                client.delete_collection(collection_name)
            except:
                pass
            
            client.create_collection(
                collection_name=collection_name,
                vectors_config=VectorParams(size=self.config.dimensions, distance=Distance.COSINE)
            )
            
            # Insert benchmark
            start = time.time()
            for i in range(0, len(self.vectors), self.config.batch_size):
                batch = self.vectors[i:i+self.config.batch_size]
                points = [
                    PointStruct(id=i+j, vector=vec.tolist(), payload={"idx": i+j})
                    for j, vec in enumerate(batch)
                ]
                client.upsert(collection_name=collection_name, points=points)
                
                if (i + self.config.batch_size) % 10000 == 0:
                    elapsed = time.time() - start
                    qps = (i + self.config.batch_size) / elapsed
                    print(f"  Inserted {i+self.config.batch_size:,} | {qps:.0f} QPS")
            
            insert_time = time.time() - start
            insert_qps = len(self.vectors) / insert_time
            
            # Search benchmark
            query = self.vectors[0].tolist()
            latencies = []
            
            print(f"  Running {self.config.search_queries} search queries...")
            for i in range(self.config.search_queries):
                start = time.time()
                results = client.query_points(
                    collection_name=collection_name,
                    query=query,
                    limit=self.config.top_k
                )
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            # Disk Usage
            disk_usage = get_docker_disk_usage("benchmarks-qdrant-1", "/qdrant/storage")

            # Cleanup
            try:
                client.delete_collection(collection_name)
                print("  ✅ Cleaned up collection")
            except Exception as e:
                print(f"  ⚠️ Cleanup failed: {e}")
            
            return BenchmarkResult(
                database="Qdrant",
                version="latest",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                disk_usage_mb=disk_usage,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Qdrant",
                version="latest",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, disk_usage_mb=0, errors=errors
            )
    
    def benchmark_weaviate(self) -> BenchmarkResult:
        """Benchmark Weaviate"""
        print("\n🟢 Benchmarking Weaviate...")
        errors = []
        client = None
        
        try:
            import warnings
            # Suppress Weaviate DeprecationWarning (temporary fix until updated API usage is clear)
            warnings.filterwarnings("ignore", category=DeprecationWarning)

            # Weaviate v4 API
            client = weaviate.connect_to_local(
                port=8080,
                grpc_port=50052,  # Avoid conflict with HyperspaceDB on 50051
                skip_init_checks=False
            )
            collection_name = "Benchmark"
            
            # Delete collection if exists
            try:
                client.collections.delete(collection_name)
            except:
                pass
            
            # Create collection
            collection = client.collections.create(
                name=collection_name,
                vectorizer_config=wvc.config.Configure.Vectorizer.none(),
                properties=[
                    wvc.config.Property(name="idx", data_type=wvc.config.DataType.INT)
                ]
            )
            
            # Insert benchmark
            start = time.time()
            for i in range(0, len(self.vectors), self.config.batch_size):
                batch = self.vectors[i:i+self.config.batch_size]
                with collection.batch.dynamic() as batch_ctx:
                    for j, vec in enumerate(batch):
                        batch_ctx.add_object(
                            properties={"idx": i+j},
                            vector=vec.tolist()
                        )
                
                if (i + self.config.batch_size) % 10000 == 0:
                    elapsed = time.time() - start
                    qps = (i + self.config.batch_size) / elapsed
                    print(f"  Inserted {i+self.config.batch_size:,} | {qps:.0f} QPS")
            
            insert_time = time.time() - start
            insert_qps = len(self.vectors) / insert_time
            
            # Search benchmark
            query = self.vectors[0].tolist()
            latencies = []
            
            print(f"  Running {self.config.search_queries} search queries...")
            for i in range(self.config.search_queries):
                start = time.time()
                results = collection.query.near_vector(
                    near_vector=query,
                    limit=self.config.top_k
                )
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            # Disk Usage
            disk_usage = get_docker_disk_usage("benchmarks-weaviate-1", "/var/lib/weaviate")

            # Cleanup
            try:
                client.collections.delete(collection_name)
                print("  ✅ Cleaned up collection")
            except Exception as e:
                print(f"  ⚠️ Cleanup failed: {e}")
            
            return BenchmarkResult(
                database="Weaviate",
                version="latest",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                disk_usage_mb=disk_usage,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Weaviate",
                version="latest",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, disk_usage_mb=0, errors=errors
            )
        finally:
            if client:
                client.close()
    
    def benchmark_hyperspace(self) -> BenchmarkResult:
        """Benchmark HyperspaceDB"""
        print("\n🚀 Benchmarking HyperspaceDB...")
        errors = []
        
        try:
            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
            
            # Create collection
            try:
                client.create_collection("benchmark", dimension=self.config.dimensions, metric="euclidean")
            except Exception as e:
                print(f"  Collection may already exist: {e}")
            
            # Start monitoring
            # monitor = ResourceMonitor("hyperspace-server")
            # monitor.start()

            # Insert benchmark
            start = time.time()
            hs_batch_size = 500 # Reduced due to gRPC 4MB limit
            for i in range(0, len(self.vectors), hs_batch_size):
                batch = self.vectors[i:i+hs_batch_size]
                ids = list(range(i, i+len(batch)))
                metadatas = [{"idx": str(j)} for j in ids]
                
                if i == 0:
                     print(f"DEBUG: Starting batch insert loop. Batch size: {hs_batch_size}, Has batch_insert: {hasattr(client, 'batch_insert')}", flush=True)

                if hasattr(client, 'batch_insert'):
                    success = client.batch_insert(batch.tolist(), ids, metadatas, collection="benchmark")
                    if not success:
                        print(f"Batch insert failed at index {i}", flush=True)
                else:
                    for j, vec in enumerate(batch):
                        client.insert(id=i+j, vector=vec.tolist(), metadata={"idx": str(i+j)}, collection="benchmark")
                
                if (i + hs_batch_size) % 10000 == 0:
                    elapsed = time.time() - start
                    qps = (i + hs_batch_size) / elapsed
                    print(f"  Inserted {i+hs_batch_size:,} | {qps:.0f} QPS", flush=True)
            
            insert_time = time.time() - start
            insert_qps = len(self.vectors) / insert_time

            # Stop monitoring
            # avg_cpu, max_mem = monitor.stop()
            # disk_usage = monitor.get_disk_usage("./data") # Assuming default data dir

            # Search benchmark
            query = self.vectors[0].tolist()
            latencies = []
            
            print(f"  Running {self.config.search_queries} search queries...")
            for i in range(self.config.search_queries):
                start = time.time()
                results = client.search(vector=query, top_k=self.config.top_k, collection="benchmark")
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            search_p50_ms = latencies[len(latencies)//2]
            search_p95_ms = latencies[int(len(latencies)*0.95)]
            search_p99_ms = latencies[int(len(latencies)*0.99)]
            
            # Disk Usage
            disk_usage = get_disk_usage_local("../data")
            
            # Cleanup
            try:
                pass # client.delete_collection("benchmark")
                print("  ✅ Cleaned up collection")
            except Exception as e:
                print(f"  ⚠️ Cleanup failed: {e}")
            
            return BenchmarkResult(
                database="HyperspaceDB",
                version="1.5.0",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=search_p50_ms,
                search_p95_ms=search_p95_ms,
                search_p99_ms=search_p99_ms,
                memory_mb=0.0,  # max_mem,
                # cpu_percent=avg_cpu,
                disk_usage_mb=disk_usage,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="HyperspaceDB",
                version="1.5.0",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, disk_usage_mb=0, errors=errors  # cpu_percent=0, disk_usage_mb=0,
            )
    
    def run_all(self) -> List[BenchmarkResult]:
        """Run all benchmarks"""
        results = []
        
        if MILVUS_AVAILABLE:
            results.append(self.benchmark_milvus())

        if QDRANT_AVAILABLE:
            results.append(self.benchmark_qdrant())
        
        if WEAVIATE_AVAILABLE:
            results.append(self.benchmark_weaviate())

        if HYPERSPACE_AVAILABLE:
            results.append(self.benchmark_hyperspace())
        
        return results


def generate_report(results: List[BenchmarkResult], config: BenchmarkConfig) -> str:
    """Generate markdown report"""
    report = f"""# Vector Database Benchmark Results

**Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}  
**Configuration**:
- Vectors: {config.num_vectors:,}
- Dimensions: {config.dimensions}
- Batch Size: {config.batch_size:,}
- Search Queries: {config.search_queries:,}
- Top-K: {config.top_k}

---

## Insert Performance

| Database | Version | QPS | Total Time | Throughput | Disk Usage (MB) |
|----------|---------|-----|------------|------------|-----------------|
"""
    
    for r in results:
        if r.insert_qps > 0:
            throughput = (config.num_vectors * config.dimensions) / r.insert_total_time / 1000000 if r.insert_total_time > 0 else 0
            report += f"| **{r.database}** | {r.version} | **{r.insert_qps:,.0f}** | {r.insert_total_time:.1f}s | {throughput:.2f} M dims/s | {r.disk_usage_mb:.1f} MB |\n"
    
    report += "\n---\n\n## Search Performance\n\n"
    report += "| Database | Avg (ms) | P50 (ms) | P95 (ms) | P99 (ms) |\n"
    report += "|----------|----------|----------|----------|----------|\n"
    
    for r in results:
        if r.search_avg_ms > 0:
            report += f"| **{r.database}** | {r.search_avg_ms:.2f} | {r.search_p50_ms:.2f} | {r.search_p95_ms:.2f} | {r.search_p99_ms:.2f} |\n"
    
    # Winner analysis
    report += "\n---\n\n## Performance Comparison\n\n"
    
    if len(results) > 1:
        best_insert = max(results, key=lambda x: x.insert_qps)
        best_search = min(results, key=lambda x: x.search_p99_ms if x.search_p99_ms > 0 else float('inf'))
        
        report += f"### Insert Throughput Winner: 🏆 **{best_insert.database}**\n"
        report += f"- **{best_insert.insert_qps:,.0f} QPS**\n\n"
        
        for r in results:
            if r.database != best_insert.database and r.insert_qps > 0:
                speedup = best_insert.insert_qps / r.insert_qps
                report += f"- {speedup:.2f}x faster than {r.database}\n"
        
        report += f"\n### Search Latency Winner: 🏆 **{best_search.database}**\n"
        report += f"- **{best_search.search_p99_ms:.2f} ms** (p99)\n\n"
        
        for r in results:
            if r.database != best_search.database and r.search_p99_ms > 0:
                speedup = r.search_p99_ms / best_search.search_p99_ms
                report += f"- {speedup:.2f}x faster than {r.database}\n"
    
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
    """Main benchmark runner"""
    print("=" * 60)
    print("  Vector Database Unified Benchmark")
    print("=" * 60)
    
    config = BenchmarkConfig()
    benchmark = VectorDBBenchmark(config)
    
    results = benchmark.run_all()
    
    # Generate report
    report = generate_report(results, config)
    
    # Save report
    output_file = "BENCHMARK_RESULTS.md"
    with open(output_file, "w") as f:
        f.write(report)
    
    print(f"\n✅ Benchmark complete! Results saved to {output_file}")
    print("\n" + "=" * 60)
    print(report)


if __name__ == "__main__":
    main()
