#!/usr/bin/env python3
import os
import sys
import time
import shutil
import subprocess
import socket
import numpy as np
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Dict

# Add SDK path
sdk_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python"))
sys.path.append(sdk_path)

from hyperspace import HyperspaceClient

@dataclass
class BenchmarkResult:
    metric: str
    dimension: int
    cache_off_qps: float
    cache_off_p50: float
    cache_off_p99: float
    l1_hit_qps: float
    l1_hit_p50: float
    l1_hit_p99: float
    l2_hit_qps: float
    l2_hit_p50: float
    l2_hit_p99: float

def kill_existing_server():
    """Kills any running hyperspace-server process to prevent port conflicts."""
    try:
        subprocess.run(["pkill", "-f", "hyperspace-server"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1)
    except Exception:
        pass

def wait_for_port(port=50051, timeout=15):
    """Waits for the server to bind to the specified port."""
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            with socket.create_connection(("localhost", port), timeout=1):
                return True
        except Exception:
            time.sleep(0.5)
    return False

def start_server(cache_enabled: bool, ann_threshold: float = None) -> subprocess.Popen:
    """Starts the HyperspaceDB server under release mode with custom environment variables."""
    kill_existing_server()
    if os.path.exists("data"):
        try:
            shutil.rmtree("data")
        except Exception as e:
            print(f"⚠️ Warning: Failed to clear data directory: {e}")

    env = os.environ.copy()
    env["HYPERSPACE_CACHE_ENABLED"] = "true" if cache_enabled else "false"
    env["HYPERSPACE_CACHE_L1_CAPACITY"] = "20000"
    if ann_threshold is not None:
        env["HYPERSPACE_CACHE_ANN_THRESHOLD"] = str(ann_threshold)
    else:
        # If no threshold is specified, L2 cache is disabled by default
        if "HYPERSPACE_CACHE_ANN_THRESHOLD" in env:
            del env["HYPERSPACE_CACHE_ANN_THRESHOLD"]
            
    env["HYPERSPACE_CACHE_REBUILD_BATCH"] = "10"
    env["HYPERSPACE_WAL_SYNC_MODE"] = "async"
    env["HS_HNSW_EF_CONSTRUCT"] = "100"

    server_bin = os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/release/hyperspace-server"))
    server = subprocess.Popen(
        [server_bin],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    
    if not wait_for_port(50051):
        print("❌ Error: HyperspaceDB server failed to start within timeout.")
        server.terminate()
        sys.exit(1)
        
    time.sleep(1.0) # Warm up connections
    return server

def generate_vector(dim: int, metric: str) -> List[float]:
    """Generates a random vector appropriate for the requested geometry/metric space."""
    if metric == "poincare":
        # Poincaré requires norm < 1. Scale standard normal to norm = 0.9.
        v = np.random.normal(0, 1, dim)
        norm = np.linalg.norm(v)
        if norm > 0:
            v = (v / norm) * 0.9
        return v.tolist()
        
    elif metric == "lorentz":
        # Lorentz Minkowski space: -t^2 + |x|^2 = -1 => t = sqrt(1.0 + |x|^2)
        spatial_dim = dim - 1
        x = np.random.uniform(-0.05, 0.05, spatial_dim)
        spatial_sq = np.sum(x * x)
        t = np.sqrt(1.0 + spatial_sq)
        return [float(t)] + x.tolist()
        
    elif metric == "hybrid":
        # YAR Hybrid (33 Lorentz + 768 L2)
        # 1. First 33 dimensions: Lorentz Minkowski space
        lorentz_dim = 33
        spatial_dim = lorentz_dim - 1
        x = np.random.uniform(-0.05, 0.05, spatial_dim)
        spatial_sq = np.sum(x * x)
        t = np.sqrt(1.0 + spatial_sq)
        lorentz_part = [float(t)] + x.tolist()
        
        # 2. Remaining 768 dimensions: Standard unit-sphere normalized L2
        euclidean_part = np.random.normal(0, 1, 768)
        norm = np.linalg.norm(euclidean_part)
        if norm > 0:
            euclidean_part = euclidean_part / norm
        return lorentz_part + euclidean_part.tolist()
        
    elif metric == "cosine":
        # Cosine distance vectors normalized to unit sphere
        v = np.random.normal(0, 1, dim)
        norm = np.linalg.norm(v)
        if norm > 0:
            v = v / norm
        return v.tolist()
        
    else:
        # Standard L2/Euclidean
        v = np.random.uniform(-0.5, 0.5, dim)
        return v.tolist()

def wait_for_indexing(client: HyperspaceClient, collection: str, timeout=60):
    """Waits for HyperspaceDB background indexing queue to drain."""
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            stats = client.get_collection_stats(collection)
            queue = stats.get("indexing_queue", 0)
            if queue == 0:
                return True
        except Exception:
            pass
        time.sleep(0.5)
    return False

def run_searches(client: HyperspaceClient, queries: List[List[float]], filters: List[Dict] = None, collection: str = "", concurrency: int = 16):
    """Runs a batch of searches concurrently and measures latency metrics."""
    latencies = []
    
    def search_task(args):
        idx, q_vec = args
        filt = filters[idx] if filters else None
        s = time.perf_counter()
        try:
            client.search(vector=q_vec, filter=filt, top_k=10, collection=collection)
        except Exception as e:
            print(f"Search failed: {e}")
        elapsed = time.perf_counter() - s
        latencies.append(elapsed * 1000.0) # convert to milliseconds

    args_list = list(enumerate(queries))
    start_time = time.perf_counter()
    
    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        executor.map(search_task, args_list)
        
    total_time = time.perf_counter() - start_time
    qps = len(queries) / total_time
    
    p50 = float(np.percentile(latencies, 50))
    p99 = float(np.percentile(latencies, 99))
    
    return qps, p50, p99

def test_scenario(metric: str, dim: int, ann_threshold: float, collection_name: str, num_vectors=5000, num_queries=1000):
    print(f"\n🧬 Running Scenario: {metric.upper()} ({dim}d)")
    print("-" * 60)
    
    # 1. Generate Vectors
    print("   👉 Generating test dataset...")
    dataset = [generate_vector(dim, metric) for _ in range(num_vectors)]
    ids = list(range(1, num_vectors + 1))
    
    queries = dataset[:num_queries]
    
    # ==================== Phase A: Cache Disabled ====================
    print("   👉 Testing with Cache DISABLED...")
    server = start_server(cache_enabled=False)
    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    
    client.create_collection(collection_name, dimension=dim, metric=metric)
    
    # Batch Insert
    batch_size = 500
    for i in range(0, num_vectors, batch_size):
        client.batch_insert(
            vectors=dataset[i:i+batch_size],
            ids=ids[i:i+batch_size],
            collection=collection_name
        )
    wait_for_indexing(client, collection_name)
    
    # Warmup
    for q in queries[:50]:
        client.search(vector=q, top_k=10, collection=collection_name)
        
    cache_off_qps, cache_off_p50, cache_off_p99 = run_searches(
        client, queries, collection=collection_name
    )
    print(f"      [Cache Off] QPS: {cache_off_qps:.1f} | p50: {cache_off_p50:.2f}ms | p99: {cache_off_p99:.2f}ms")
    
    client.delete_collection(collection_name)
    server.terminate()
    server.wait()
    
    # ==================== Phase B: Cache Enabled ====================
    print("   👉 Testing with Cache ENABLED...")
    server = start_server(cache_enabled=True, ann_threshold=ann_threshold)
    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    
    client.create_collection(collection_name, dimension=dim, metric=metric)
    
    # Batch Insert (this also populates L1 Cache and schedules L2 pending builds)
    for i in range(0, num_vectors, batch_size):
        client.batch_insert(
            vectors=dataset[i:i+batch_size],
            ids=ids[i:i+batch_size],
            collection=collection_name
        )
    wait_for_indexing(client, collection_name)
    
    # Wait for the eager cache build task and periodic maintenance (5s) to build L2 HNSW map fully and avoid CPU contention
    time.sleep(15.0)
    
    # L1 Hit Test: Exact search by passing target id filter
    l1_filters = [{"id": str(i)} for i in ids[:num_queries]]
    l1_hit_qps, l1_hit_p50, l1_hit_p99 = run_searches(
        client, queries, filters=l1_filters, collection=collection_name
    )
    print(f"      [Cache L1 Hit] QPS: {l1_hit_qps:.1f} | p50: {l1_hit_p50:.2f}ms | p99: {l1_hit_p99:.2f}ms")
    
    # L2 Hit Test: Query exact coordinates but WITHOUT filters (triggers L2 ANN)
    l2_hit_qps, l2_hit_p50, l2_hit_p99 = run_searches(
        client, queries, filters=None, collection=collection_name
    )
    print(f"      [Cache L2 ANN Hit] QPS: {l2_hit_qps:.1f} | p50: {l2_hit_p50:.2f}ms | p99: {l2_hit_p99:.2f}ms")
    
    client.delete_collection(collection_name)
    server.terminate()
    server.wait()
    
    return BenchmarkResult(
        metric=metric,
        dimension=dim,
        cache_off_qps=cache_off_qps,
        cache_off_p50=cache_off_p50,
        cache_off_p99=cache_off_p99,
        l1_hit_qps=l1_hit_qps,
        l1_hit_p50=l1_hit_p50,
        l1_hit_p99=l1_hit_p99,
        l2_hit_qps=l2_hit_qps,
        l2_hit_p50=l2_hit_p50,
        l2_hit_p99=l2_hit_p99
    )

def main():
    print("================================================================")
    print("🔥 HyperspaceDB L0 Hot Tier Caching Concurrency Stress Test 🔥")
    print("================================================================")
    
    # Configure test geometries and their L2 ANN cache distance thresholds
    scenarios = [
        # (metric, dimension, ann_threshold, collection_name)
        ("cosine", 1024, 0.15, "stress_cache_cosine"),
        ("poincare", 64, 0.5, "stress_cache_poincare"),
        ("lorentz", 64, 2.0, "stress_cache_lorentz"),
        ("l2", 1024, 0.05, "stress_cache_l2"),
        ("hybrid", 801, 1.0, "stress_cache_hybrid")
    ]
    
    results = []
    for metric, dim, threshold, col in scenarios:
        try:
            res = test_scenario(metric, dim, threshold, col)
            results.append(res)
        except Exception as e:
            print(f"❌ Failed to run scenario {metric}: {e}")
            kill_existing_server()
            
    # Print beautiful performance report table
    print("\n" + "=" * 125)
    print("📊 HYPERSPACEDB L0 CACHE STRESS TEST REPORT")
    print("=" * 125)
    print(f"{'Geometry & Metric':<25} | {'Cache State':<15} | {'Throughput (QPS)':<18} | {'p50 Latency (ms)':<18} | {'p99 Latency (ms)':<18} | {'Speedup Factor':<15}")
    print("-" * 125)
    
    for r in results:
        label = f"{r.metric.upper()} ({r.dimension}d)"
        
        # Speedups
        l1_speedup = r.l1_hit_qps / r.cache_off_qps if r.cache_off_qps > 0 else 0.0
        l2_speedup = r.l2_hit_qps / r.cache_off_qps if r.cache_off_qps > 0 else 0.0
        
        print(f"{label:<25} | {'Cache DISABLED':<15} | {r.cache_off_qps:18.1f} | {r.cache_off_p50:18.3f} | {r.cache_off_p99:18.3f} | {'Baseline':<15}")
        print(f"{'':<25} | {'L1 Hot Hit':<15} | {r.l1_hit_qps:18.1f} | {r.l1_hit_p50:18.3f} | {r.l1_hit_p99:18.3f} | {l1_speedup:14.2f}x")
        print(f"{'':<25} | {'L2 ANN Hit':<15} | {r.l2_hit_qps:18.1f} | {r.l2_hit_p50:18.3f} | {r.l2_hit_p99:18.3f} | {l2_speedup:14.2f}x")
        print("-" * 125)
        
    print("✨ Caching stress-test run successfully and completed.")
    kill_existing_server()

if __name__ == "__main__":
    main()
