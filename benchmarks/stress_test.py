import sys
import os
import time
import numpy as np
import threading
import requests
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Dict

# Add sdk to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdks", "python")))
from hyperspace import HyperspaceClient

@dataclass
class ConcurrencyResult:
    concurrency: int
    ins_total: float
    ins_per_thread: float
    ins_efficiency: float
    srch_total: float
    srch_per_thread: float
    srch_efficiency: float

def wait_for_indexing(host="localhost", port=50050, collection="stress_test", timeout=600):
    """Wait for HyperspaceDB background indexing to complete with visible progress"""
    url = f"http://{host}:{port}/api/collections/{collection}/stats"
    headers = {"x-api-key": "I_LOVE_HYPERSPACEDB", "x-hyperspace-user-id": "default_admin"}
    start_time = time.time()
    while True:
        if timeout and time.time() - start_time > timeout:
            print("\n⚠️  Indexing timeout!")
            break
        try:
            response = requests.get(url, headers=headers, timeout=5)
            if response.status_code == 200:
                data = response.json()
                queue = data.get("indexing_queue", 0)
                count = data.get("count", 0)
                print(f"\r      [Indexing Sync] Queue: {queue:,} | Indexed: {count:,} ", end="", flush=True)
                if queue == 0 and count > 0:
                    print(" Done.")
                    break
            time.sleep(1)
        except:
            time.sleep(1)

def generate_vector(dim, metric):
    if metric == "poincare":
        # Poincaré requires norm < 1. Using small random values is safe.
        v = np.random.uniform(-0.05, 0.05, dim)
        return v.tolist()
    elif metric == "lorentz":
        # Lorentz: -t^2 + |x|^2 = -1 => t = sqrt(1 + |x|^2)
        # We assume dim includes the t component (the first one)
        spatial_dim = dim - 1
        x = np.random.uniform(-0.1, 0.1, spatial_dim)
        spatial_sq = np.sum(x * x)
        t = np.sqrt(1.0 + spatial_sq)
        return [float(t)] + x.tolist()
    else:
        # Euclidean/Cosine
        v = np.random.uniform(-0.3, 0.3, dim)
        return v.tolist()

def run_concurrent_inserts(client, concurrency, total_count, dim, metric, collection):
    # Pre-generate to avoid measuring CPU time for vector generation
    vectors = [generate_vector(dim, metric) for _ in range(total_count)]
    start = time.time()
    
    # Use batch_insert to maximize performance
    # 1024d is ~4KB in float32. 4000 vectors is 16MB - safe for 64MB gRPC limit.
    batch_size_limit = 4000 
    
    def insert_task(batch_vecs, start_id):
        ids = list(range(start_id, start_id + len(batch_vecs)))
        client.batch_insert(batch_vecs, ids, collection=collection)

    # Calculate per-thread work
    total_vectors = len(vectors)
    work_per_thread = total_vectors // concurrency
    
    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = []
        for i in range(concurrency):
            start_off = i * work_per_thread
            end_off = (i + 1) * work_per_thread if i < concurrency - 1 else total_vectors
            thread_vecs = vectors[start_off:end_off]
            
            # Further sub-batch to stay within gRPC limits
            for j in range(0, len(thread_vecs), batch_size_limit):
                batch = thread_vecs[j : j + batch_size_limit]
                futures.append(executor.submit(insert_task, batch, start_off + j))
                
        for f in futures:
            f.result()
            
    dur = time.time() - start
    return total_count / dur

def run_concurrent_searches(client, concurrency, total_count, dim, metric, collection):
    query_vectors = [generate_vector(dim, metric) for _ in range(total_count)]
    start = time.time()

    supports_batch = callable(getattr(client, "search_batch", None))

    def search_task(vecs):
        vectors = [v.tolist() if hasattr(v, "tolist") else v for v in vecs]
        if supports_batch:
            batch_size = 64
            for i in range(0, len(vectors), batch_size):
                client.search_batch(
                    vectors[i : i + batch_size],
                    top_k=10,
                    collection=collection,
                )
            return
        for v in vectors:
            client.search(vector=v, top_k=10, collection=collection)

    batches = np.array_split(query_vectors, concurrency)
    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        list(executor.map(search_task, batches))
            
    dur = time.time() - start
    return total_count / dur

def run_concurrency_suite(dim, metric, label):
    host = "localhost"
    port = 50051
    api_key = "I_LOVE_HYPERSPACEDB"
    collection_base = f"stress_{metric}_{dim}"
    concurrencies = [1, 10, 50, 100, 500, 1000]
    results = []
    
    print(f"\n⚡ STEP: Testing {label} ({dim}d, metric: {metric})")
    print("-" * 100)
    
    base_ins_qps = 0
    base_srch_qps = 0
    
    for c in concurrencies:
        coll = f"{collection_base}_{c}"
        client = HyperspaceClient(f"{host}:{port}", api_key=api_key)
        client.delete_collection(coll)
        
        if not client.create_collection(coll, dimension=dim, metric=metric):
            print(f"   ❌ Failed to create collection {coll}. Skipping.")
            continue
        
        # 1. Inserts (using increased count for stress testing)
        num_inserts = 20000
        print(f"   🚀 Concurrency {c:4} | Phase: Inserts ({num_inserts})...", end="", flush=True)
        ins_qps = run_concurrent_inserts(client, c, num_inserts, dim, metric, coll)
        if c == 1: base_ins_qps = ins_qps
        
        # Sync
        wait_for_indexing(collection=coll)
        
        # 2. Searches
        num_searches = 5000
        print(f"   🔍 Concurrency {c:4} | Phase: Searches ({num_searches})...", end="", flush=True)
        srch_qps = run_concurrent_searches(client, c, num_searches, dim, metric, coll)
        if c == 1: base_srch_qps = srch_qps
        
        # Stats
        results.append(ConcurrencyResult(
            concurrency=c,
            ins_total=ins_qps,
            ins_per_thread=ins_qps / c,
            ins_efficiency=(ins_qps / (base_ins_qps * c)) * 100 if base_ins_qps > 0 else 0,
            srch_total=srch_qps,
            srch_per_thread=srch_qps / c,
            srch_efficiency=(srch_qps / (base_srch_qps * c)) * 100 if base_srch_qps > 0 else 0
        ))
        client.delete_collection(coll)
    
    return results

def print_results(results, label):
    print("\n" + "=" * 110)
    print(f"📊 REPORT: {label}")
    print("-" * 110)
    print(f"{'Threads':<12} | {'Total Ins QPS':<15} | {'Ins QPS/Thr':<15} | {'Ins Eff%':<10} | {'Total Srch QPS':<15} | {'Srch QPS/Thr':<15} | {'Srch Eff%':<10}")
    print("-" * 110)
    for r in results:
        print(f"{r.concurrency:<12} | {r.ins_total:15.0f} | {r.ins_per_thread:15.1f} | {r.ins_efficiency:9.1f}% | {r.srch_total:15.0f} | {r.srch_per_thread:15.1f} | {r.srch_efficiency:9.1f}%")
    print("=" * 110)

def main():
    print("🔥 Starting Comprehensive HyperspaceDB Stress Test (Euclidean vs Hyperbolic)")
    print("   Note: Using batch_insert to maximize performance figures.")
    
    # Step 1: Euclidean Baseline
    euc_results = run_concurrency_suite(dim=1024, metric="cosine", label="Euclidean Baseline")
    
    # Step 2: Hyperbolic Efficiency (Poincaré)
    hyp_results = run_concurrency_suite(dim=64, metric="poincare", label="Hyperbolic Efficiency (Poincaré)")
    
    # Step 3: Lorentz Model (Minkowski space)
    lor_results = run_concurrency_suite(dim=64, metric="lorentz", label="Lorentz Hyperboloid")
    
    # Final Reports
    print_results(euc_results, "EUCLIDEAN (1024d Cosine)")
    print_results(hyp_results, "HYPERBOLIC (64d Poincaré)")
    print_results(lor_results, "LORENTZ (64d Hyperboloid)")
    
    print("\n✨ All tests finished. All temporary data cleared.")

if __name__ == "__main__":
    main()
