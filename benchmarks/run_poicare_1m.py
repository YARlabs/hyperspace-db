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
    
    print(f"\n🧪 Testing: Poincare (64d) 1M, Server={env_mode}, Req={api_durability}")
    
    # 1. Start Server with clean state
    if os.path.exists("data"): shutil.rmtree("data")
    
    # We rely on .env having HS_METRIC=poincare and HS_DIMENSION=64
    # But we override WAL mode
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
        
        # Explicit collection creation relies on .env defaults if not specified? 
        # But create_collection takes explicit parameters.
        # User insisted on .env.
        # If we use .env defaults, we might just use "default" collection or create "bench" with explicit params matching env.
        # Let's create explicitly to be safe, matching .env
        try:
             client.create_collection("bench_poincare", dimension=64, metric="poincare")
        except Exception as e:
             print(f"Create Warning: {e}")

        num_vecs = 1_000_000
        dim = 64
        col_name = "bench_poincare"
        
        print(f"  Generating {num_vecs} vectors...")
        vecs = np.random.randn(num_vecs, dim).astype(np.float32)
        vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
        # Poincare safety
        vecs *= 0.99
        
        start = time.time()
        # Safe batch size for Poincare
        batch_size = 1000
        success_count = 0
        
        for i in range(0, num_vecs, batch_size):
            batch = vecs[i:i+batch_size].tolist()
            ids = list(range(i, i+len(batch)))
            metas = [{"i": str(k)} for k in ids]
            
            if client.batch_insert(batch, ids, metas, collection=col_name, durability=api_durability):
                 success_count += len(batch)
            
            if (i+batch_size) % 50000 == 0:
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
    print("=== HYPERBOLIC (POINCARE) 1M BENCHMARK ===")
    for env, api in scenarios:
        r = run_benchmark_scenario(env, api)
        if r: results.append(r)
        
    print("\n\n=== FINAL POINCARE RESULTS (1M) ===")
    print("| Mode | QPS |")
    print("|---|---|")
    for r in results:
        print(f"| {r['mode']} | {r['qps']:.0f} |")
