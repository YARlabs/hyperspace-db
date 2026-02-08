import sys
import os
import time
import numpy as np
import threading
from concurrent.futures import ThreadPoolExecutor

# Add sdk to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdks", "python")))
from hyperspace import HyperspaceClient

def generate_hyp_vector(dim=8):
    # Generate vector inside Poincare ball (norm < 1)
    v = np.random.uniform(-0.3, 0.3, dim)
    return v.tolist()

def bench_inserts(client, count=1000):
    start = time.time()
    for _ in range(count):
        client.insert(generate_hyp_vector())
    end = time.time()
    dur = end - start
    print(f"✅ Inserted {count} vectors in {dur:.2f}s ({count/dur:.0f} vectors/s)")
    return dur

def bench_search(client, count=1000, threads=4):
    vectors = [generate_hyp_vector() for _ in range(count)]
    
    start = time.time()
    with ThreadPoolExecutor(max_workers=threads) as executor:
        list(executor.map(lambda v: client.search(v, top_k=10), vectors))
    end = time.time()
    
    dur = end - start
    print(f"🚀 Performed {count} searches in {dur:.2f}s ({count/dur:.0f} QPS) with {threads} threads")
    return dur

if __name__ == "__main__":
    client = HyperspaceClient("localhost:50051")
    
    print("🔥 Starting HyperspaceDB Benchmark...")
    
    # 1. Stress Inserts
    bench_inserts(client, 5000)
    
    # 2. Stress Search
    bench_search(client, 1000, threads=8)
    
    print("✨ Benchmark finished!")
