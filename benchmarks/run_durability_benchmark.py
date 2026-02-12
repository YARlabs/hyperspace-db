#!/usr/bin/env python3
import time
import subprocess
import os
import sys
import numpy as np
import shutil

# Ensure SDK path
sdk_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python"))
sys.path.append(sdk_path)

try:
    from hyperspace import HyperspaceClient
except ImportError as e:
    print(f"Hyperspace SDK import failed in {sdk_path}: {e}")
    # Don't exit, try to continue or debug
    sys.exit(1)

if __name__ == "__main__":
    from hyperspace.client import Durability

    scenarios = [
        # (Metric, Dim, EnvMode, ApiMode)
        ("poincare", 64, "async", Durability.DEFAULT),
        ("poincare", 64, "async", Durability.STRICT),
        ("poincare", 64, "batch", Durability.DEFAULT),
    ]
    
    table_rows = []
    
    for metric, dim, env_mode, api_mode in scenarios:
        # We need to adapt run_benchmark signature
        # Or just pass env_mode and use api_mode inside
        # Refactoring run_benchmark slightly
        pass

    # ... refactoring logic below ...

def run_benchmark_scenario(metric, dim, env_mode, api_durability):
    mode_name = env_mode
    if api_durability == 3: mode_name += "+req:strict"
    elif api_durability == 2: mode_name += "+req:batch"
    
    print(f"\n🧪 Testing: {metric}, {dim}d, Server={env_mode}, Req={api_durability}")
    
    # ... setup as before ...
    if os.path.exists("data"): shutil.rmtree("data")
    env = os.environ.copy()
    env["HS_METRIC"] = metric
    env["HS_DIMENSION"] = str(dim)
    env["HYPERSPACE_WAL_SYNC_MODE"] = env_mode
    env["HS_HNSW_EF_CONSTRUCT"] = "100"
    
    server = subprocess.Popen(
        ["cargo", "run", "--release", "-p", "hyperspace-server"],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE 
    )
    
    try:
        # Connectivity check...
        connected = False
        import socket
        for attempt in range(30):
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
             client.create_collection("bench", dimension=dim, metric=metric)
        except: pass

        num_vecs = 100000 # Smaller for strict test
        # If strict, 3000 vectors ~ 6s. 
        if api_durability != 3 and env_mode != "strict":
             num_vecs = 100000 if env_mode != "strict" else 100000
        
        print(f"  Generating {num_vecs} vectors...")
        vecs = np.random.randn(num_vecs, dim).astype(np.float32)
        vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
        # For Poincare, vectors must be < 1 (strictly inside unit ball)
        # Scale to 0.99 to be safe
        vecs *= 0.99
        
        start = time.time()
        # Max gRPC msg 4MB.
        # 1024d double = 8KB. 500 * 8KB = 4MB usually limit. Safe 400.
        # 64d double = 512B. 1000 * 512B = 0.5MB. Safe 1000.
        batch_size = 1000 if dim <= 64 else 400
        success_count = 0
        
        for i in range(0, num_vecs, batch_size):
            batch = vecs[i:i+batch_size].tolist()
            ids = list(range(i, i+len(batch)))
            metas = [{"i": str(k)} for k in ids]
            
            
            # Pass durability here!
            if client.batch_insert(batch, ids, metas, collection="bench", durability=api_durability):
                 success_count += len(batch)
            
            if (i+batch_size) % 10000 == 0:
                elapsed = time.time() - start
                qps = (i+batch_size)/elapsed
                print(f"  Inserted {(i+batch_size)} | {qps:.0f} QPS", end="\r")
        
        total_time = time.time() - start
        if success_count == 0: success_count = 1 # Avoid div by zero
        qps = success_count / total_time
        print(f"\n  ✅ Ingest QPS: {qps:.0f}")

        # Search
        latencies = []
        q = vecs[0].tolist()
        for _ in range(50):
            s = time.time()
            client.search(q, top_k=10, collection="bench")
            latencies.append((time.time()-s)*1000)
        p95 = np.percentile(latencies, 95)
        
        return {"metric": metric, "dim": dim, "mode": mode_name, "qps": qps, "p95": p95}

    except Exception as e:
        print(f"Error: {e}")
        return None
    finally:
        server.terminate()
        server.wait()
        if os.path.exists("data"): shutil.rmtree("data")

if __name__ == "__main__":
    from hyperspace.client import Durability
    
    scenarios = [
        # (Metric, Dim, EnvMode, ApiMode)
        ("l2", 1024, "async", Durability.DEFAULT),
        ("l2", 1024, "async", Durability.STRICT),
        ("l2", 1024, "batch", Durability.DEFAULT),
        ("poincare", 64, "async", Durability.DEFAULT),
        ("poincare", 64, "async", Durability.STRICT),
        ("poincare", 64, "batch", Durability.DEFAULT),
    ]

    results = []
    for m, d, env, api in scenarios:
        r = run_benchmark_scenario(m, d, env, api)
        if r: results.append(r)
        
    print("\n\n=== FINAL RESULTS ===")
    print("| Metric | Mode | QPS | P95 (ms) |")
    print("|---|---|---|---|")
    for r in results:
        print(f"| {r['metric']} | {r['mode']} | {r['qps']:.0f} | {r['p95']:.2f} |")
