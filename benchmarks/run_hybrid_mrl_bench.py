#!/usr/bin/env python3
import time
import numpy as np
import sys
import os
import requests

# Ensure local SDK is used
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

try:
    from hyperspace import HyperspaceClient
except ImportError:
    print("⚠️  HyperspaceDB Python SDK not found")
    sys.exit(1)

def generate_hybrid_vectors(count):
    print(f"🧬 Generating {count} hybrid vectors (801D)...")
    # 33D Lorentz + 768D Euclidean
    
    # Lorentz part
    # We need -t^2 + sum(x^2) = -1. Simplest: t = sqrt(1 + sum(x^2))
    x_lor = np.random.uniform(-0.1, 0.1, size=(count, 32))
    t_lor = np.sqrt(1 + np.sum(x_lor**2, axis=1)).reshape(-1, 1)
    lorentz = np.hstack([t_lor, x_lor]) # [count, 33]
    
    # Euclidean part (768D)
    euclidean = np.random.randn(count, 768).astype(np.float32)
    norms = np.linalg.norm(euclidean, axis=1, keepdims=True)
    euclidean /= (norms + 1e-9) # Normalize to unit sphere
    
    vectors = np.hstack([lorentz, euclidean])
    return vectors.astype(np.float64)

def run_bench():
    host = "localhost"
    client = HyperspaceClient(f"{host}:50051", api_key="I_LOVE_HYPERSPACEDB")
    collection_name = "bench_hybrid_mrl_1m"
    
    # 1. Setup
    print(f"🧹 Cleaning old collection...")
    try: client.delete_collection(collection_name)
    except: pass
    
    print(f"🏗 Creating collection with 'hybrid' metric and 'AsymmetricHybrid801' quantization...")
    # Note: AsymmetricHybrid801 is enabled via quantization config in metadata if supported, 
    # but here we assume the server uses it for N=801 and metric='hybrid'.
    client.create_collection(collection_name, dimension=801, metric="hybrid")
    
    # 2. Ingest 100k (for quick test) or 1M
    total_count = 100_000 # Let's start with 100k for the session
    batch_size = 1000
    vectors = generate_hybrid_vectors(total_count)
    
    print(f"🚀 Ingesting {total_count} vectors...")
    start_time = time.time()
    for i in range(0, total_count, batch_size):
        batch = vectors[i:i+batch_size]
        ids = list(range(i, i+len(batch)))
        client.batch_insert(batch.tolist(), ids, collection=collection_name)
        if (i + batch_size) % 10000 == 0:
            print(f"   Processed {i+batch_size}/{total_count}...")
            
    print(f"✅ Ingestion took {time.time() - start_time:.2f}s")
    
    # Wait for indexing
    print("⏳ Waiting for background indexing...")
    while True:
        try:
            stats = requests.get(f"http://{host}:50050/api/collections/{collection_name}/stats", headers={"x-api-key": "I_LOVE_HYPERSPACEDB"}).json()
            queue = stats.get("indexing_queue", 0)
            if queue == 0: break
            print(f"   Queue: {queue}...")
            time.sleep(2)
        except:
            time.sleep(2)
            
    # 3. Benchmark Search
    query_vec = vectors[0].tolist()
    
    print("\n📊 Benchmarking Search Latency (P99) vs MRL Dimension:")
    mrl_dims = [None, 96, 256, 512, 768]
    
    results = {}
    
    for dim in mrl_dims:
        label = f"MRL={dim}" if dim else "FULL (801D)"
        print(f"   -> Testing {label}...", end=" ", flush=True)
        
        lats = []
        for _ in range(100):
            ts = time.time()
            # Custom request with mrl_dimension if dim is set
            # The current SDK might not have mrl_dimension yet, so we use direct gRPC/HTTP if needed
            # For now, let's assume we can pass it in search params or use a raw request
            
            # Since I haven't updated the Python SDK yet, I'll use direct HTTP search if possible
            # or just assume the SDK will ignore unknown kwargs for now.
            
            # Actually, I'll use a raw requests.post to the server's HTTP API (if it supports it)
            # or I'll update the SDK.
            
            payload = {
                "collection": collection_name,
                "vector": query_vec,
                "top_k": 10
            }
            if dim:
                payload["mrl_dimension"] = dim
                
            res = requests.post(f"http://{host}:50050/api/collections/{collection_name}/search", json=payload, headers={"x-api-key": "I_LOVE_HYPERSPACEDB"})
            lats.append((time.time() - ts) * 1000)
            
        p99 = np.percentile(lats, 99)
        avg = np.mean(lats)
        print(f"Avg: {avg:.2f}ms | P99: {p99:.2f}ms")
        results[label] = p99

    print("\n🏁 Benchmark Finished.")
    print(results)

if __name__ == "__main__":
    run_bench()
