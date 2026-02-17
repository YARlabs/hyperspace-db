import time
import uuid
import random
import os
import shutil
import subprocess
import numpy as np
import sys

# Ensure we can import the SDK
sys.path.append(os.path.join(os.path.dirname(__file__), ".."))
try:
    from sdks.python.hyperspace import HyperspaceClient
    from sdks.python.hyperspace.client import Durability
except ImportError:
    print("⚠️  Hyperspace SDK not found in path. Trying pip installed package...")
    from hyperspace import HyperspaceClient
    from hyperspace.client import Durability

def run_benchmark_scenario(env_mode, api_durability):
    mode_name = env_mode
    if api_durability == 3: mode_name += "+req:strict"
    elif api_durability == 2: mode_name += "+req:batch"
    
    print(f"\n🧪 Testing: L2 (1024d) 1M, Server={env_mode}, Req={api_durability}")
    
    # 1. Start Server with clean state
    if os.path.exists("data"): shutil.rmtree("data")
    
    # We rely on .env having HS_METRIC=l2 and HS_DIMENSION=1024
    env = os.environ.copy()
    env["HYPERSPACE_WAL_SYNC_MODE"] = env_mode
    env["HS_HNSW_EF_CONSTRUCT"] = "100"
    
    server = subprocess.Popen(
        ["cargo", "run", "--release", "-p", "hyperspace-server"],
        env=env,
        cwd=os.path.join(os.path.dirname(__file__), ".."),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE 
    )
    
    try:
        # Connectivity check
        connected = False
        import socket
        for attempt in range(60):
            try:
                with socket.create_connection(("localhost", 50051), timeout=1):
                    connected = True
                    break
            except:
                time.sleep(1)
        
        if not connected:
            print("❌ Server unreachable")
            out, err = server.communicate()
            if err: print(err.decode())
            return None

        time.sleep(2)
        client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
        
        try:
             client.create_collection("bench_l2", dimension=1024, metric="l2")
        except Exception as e:
             # print(e) # Ignore or print
             pass

        num_vecs = 1_000_000
        dim = 1024
        col_name = "bench_l2"
        
        print(f"  Generating {num_vecs} vectors...")
        vecs = np.random.randn(num_vecs, dim).astype(np.float32)
        vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
        # L2 doesn't need scaling strictly < 1, but doesn't hurt.
        
        start = time.time()
        # Safe batch size for L2 (large vectors)
        batch_size = 400
        success_count = 0
        
        for i in range(0, num_vecs, batch_size):
            batch = vecs[i:i+batch_size].tolist()
            ids = list(range(i, i+len(batch)))
            metas = [{"i": str(k)} for k in ids]
            
            if client.batch_insert(batch, ids, metas, collection=col_name, durability=api_durability):
                 success_count += len(batch)
            
            if (i+batch_size) % 10000 == 0:
                elapsed = time.time() - start
                current_qps = (i+batch_size)/elapsed if elapsed > 0 else 0
                print(f"  Inserted {(i+batch_size)} | {current_qps:.0f} QPS", end="\r")
        
        total_time = time.time() - start
        if success_count == 0: success_count = 1
        qps = success_count / total_time
        print(f"\n  ✅ Ingest QPS: {qps:.0f}")
        
        return {"mode": mode_name, "qps": qps}

    except Exception as e:
        print(f"\n❌ Error: {e}")
        return None
    finally:
        server.terminate()
        server.wait()
        if os.path.exists("data"): shutil.rmtree("data")

if __name__ == "__main__":
    scenarios = [
        ("async", 0), # Default
        ("async", 3), # Strict Override
        ("batch", 0), # Batch Mode
    ]

    results = []
    print("=== EUCLIDEAN (L2) 1M BENCHMARK ===")
    for env, api in scenarios:
        r = run_benchmark_scenario(env, api)
        if r: results.append(r)
        
    print("\n\n=== FINAL L2 RESULTS (1M) ===")
    print("| Mode | QPS |")
    print("|---|---|")
    for r in results:
        print(f"| {r['mode']} | {r['qps']:.0f} |")
