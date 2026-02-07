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
    errors: List[str]


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
    
    def benchmark_hyperspace(self) -> BenchmarkResult:
        """Benchmark HyperspaceDB"""
        print("\n🚀 Benchmarking HyperspaceDB...")
        errors = []
        
        try:
            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
            
            # Create collection
            try:
                client.create_collection("benchmark", dimension=self.config.dimensions)
            except:
                pass  # May already exist
            
            # Insert benchmark
            start = time.time()
            for i in range(0, len(self.vectors), self.config.batch_size):
                batch = self.vectors[i:i+self.config.batch_size]
                for j, vec in enumerate(batch):
                    client.insert(vector=vec.tolist(), metadata={"id": i+j})
                
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
                results = client.search(vector=query, top_k=self.config.top_k)
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            return BenchmarkResult(
                database="HyperspaceDB",
                version="1.4.0",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,  # TODO: Get from server
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="HyperspaceDB",
                version="1.4.0",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, errors=errors
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
                results = client.search(
                    collection_name=collection_name,
                    query_vector=query,
                    limit=self.config.top_k
                )
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            return BenchmarkResult(
                database="Qdrant",
                version="1.7.4",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Qdrant",
                version="1.7.4",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, errors=errors
            )
    
    def benchmark_weaviate(self) -> BenchmarkResult:
        """Benchmark Weaviate"""
        print("\n🟢 Benchmarking Weaviate...")
        errors = []
        
        try:
            client = weaviate.Client("http://localhost:8080")
            class_name = "Benchmark"
            
            # Create schema
            try:
                client.schema.delete_class(class_name)
            except:
                pass
            
            class_obj = {
                "class": class_name,
                "vectorizer": "none",
                "properties": [
                    {"name": "idx", "dataType": ["int"]}
                ]
            }
            client.schema.create_class(class_obj)
            
            # Insert benchmark
            start = time.time()
            with client.batch as batch:
                batch.batch_size = self.config.batch_size
                for i, vec in enumerate(self.vectors):
                    batch.add_data_object(
                        {"idx": i},
                        class_name,
                        vector=vec.tolist()
                    )
                    
                    if (i + 1) % 10000 == 0:
                        elapsed = time.time() - start
                        qps = (i + 1) / elapsed
                        print(f"  Inserted {i+1:,} | {qps:.0f} QPS")
            
            insert_time = time.time() - start
            insert_qps = len(self.vectors) / insert_time
            
            # Search benchmark
            query = self.vectors[0].tolist()
            latencies = []
            
            print(f"  Running {self.config.search_queries} search queries...")
            for i in range(self.config.search_queries):
                start = time.time()
                results = client.query.get(class_name, ["idx"]) \
                    .with_near_vector({"vector": query}) \
                    .with_limit(self.config.top_k) \
                    .do()
                latency = (time.time() - start) * 1000
                latencies.append(latency)
            
            latencies.sort()
            
            return BenchmarkResult(
                database="Weaviate",
                version="1.23.1",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Weaviate",
                version="1.23.1",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, errors=errors
            )
    
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
            
            return BenchmarkResult(
                database="Milvus",
                version="2.3.3",
                insert_qps=insert_qps,
                insert_total_time=insert_time,
                search_avg_ms=statistics.mean(latencies),
                search_p50_ms=latencies[len(latencies)//2],
                search_p95_ms=latencies[int(len(latencies)*0.95)],
                search_p99_ms=latencies[int(len(latencies)*0.99)],
                memory_mb=0.0,
                errors=errors
            )
            
        except Exception as e:
            errors.append(str(e))
            return BenchmarkResult(
                database="Milvus",
                version="2.3.3",
                insert_qps=0, insert_total_time=0,
                search_avg_ms=0, search_p50_ms=0, search_p95_ms=0, search_p99_ms=0,
                memory_mb=0, errors=errors
            )
    
    def run_all(self) -> List[BenchmarkResult]:
        """Run all benchmarks"""
        results = []
        
        if HYPERSPACE_AVAILABLE:
            results.append(self.benchmark_hyperspace())
        
        if QDRANT_AVAILABLE:
            results.append(self.benchmark_qdrant())
        
        if WEAVIATE_AVAILABLE:
            results.append(self.benchmark_weaviate())
        
        if MILVUS_AVAILABLE:
            results.append(self.benchmark_milvus())
        
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

| Database | Version | QPS | Total Time | Throughput |
|----------|---------|-----|------------|------------|
"""
    
    for r in results:
        if r.insert_qps > 0:
            report += f"| **{r.database}** | {r.version} | **{r.insert_qps:,.0f}** | {r.insert_total_time:.1f}s | {r.insert_qps * config.dimensions / 1e6:.2f} M dims/s |\n"
    
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
