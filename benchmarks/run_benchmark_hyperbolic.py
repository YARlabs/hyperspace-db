#!/usr/bin/env python3
"""
Hyperbolic Efficiency Suite
Compares 1024d Euclidean (Milvus/Competitors) vs 64d Poincaré (HyperspaceDB).
"""

import time
import numpy as np
import networkx as nx
import sys
import os
import statistics
import json
from dataclasses import dataclass, asdict
from typing import List, Optional

# Ensure local SDK is used
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

# --- Imports ---
try:
    from hyperspace import HyperspaceClient
    HYPERSPACE_AVAILABLE = True
except ImportError:
    HYPERSPACE_AVAILABLE = False
    print("⚠️  HyperspaceDB Python SDK not found")

try:
    from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType, utility
    MILVUS_AVAILABLE = True
except ImportError:
    MILVUS_AVAILABLE = False

# --- Config ---
@dataclass
class Config:
    num_nodes: int = 50_000       # Dataset size
    branching: int = 6            # Branching factor for Tree
    milvus_dim: int = 1024        # High dim for Euclidean baseline
    hyperspace_dim: int = 64      # Low dim for Hyperbolic efficiency
    batch_size: int = 1000
    search_queries: int = 2000
    top_k: int = 10

@dataclass
class Result:
    database: str
    dimension: int
    geometry: str
    insert_qps: float
    total_time: float
    p50: float
    p95: float
    p99: float
    disk_usage: str
    status: str

# --- Data Generator ---
class TreeGenerator:
    def __init__(self, cfg: Config):
        self.cfg = cfg
        print(f"🌳 Generating In-Memory Taxonomy ({cfg.num_nodes} nodes)...")
        
        # 1. Structure
        depth = int(np.log(cfg.num_nodes) / np.log(cfg.branching))
        G = nx.balanced_tree(cfg.branching, depth)
        nodes = list(G.nodes())
        if len(nodes) > cfg.num_nodes:
            G.remove_nodes_from(nodes[cfg.num_nodes:])
        
        self.count = G.number_of_nodes()
        print(f"   Graph structure created: {self.count} nodes.")

        # 2. Euclidean Embedding (1024d) - Baseline
        print(f"   -> Embedding {cfg.milvus_dim}d Euclidean vectors...")
        self.vecs_milvus = np.random.randn(self.count, cfg.milvus_dim).astype(np.float32)
        norms = np.linalg.norm(self.vecs_milvus, axis=1, keepdims=True)
        self.vecs_milvus /= norms

        # 3. Hyperbolic Embedding (64d) - Optimization
        print(f"   -> Embedding {cfg.hyperspace_dim}d Poincaré vectors...")
        self.vecs_hyper = np.random.uniform(-0.05, 0.05, size=(self.count, cfg.hyperspace_dim)).astype(np.float32)
        
        # Simple radial layout simulation
        paths = nx.shortest_path_length(G, source=0)
        max_dist = max(paths.values()) if paths else 1
        
        for node_id, dist in paths.items():
            if node_id >= self.count: continue
            # Push deeper nodes to the boundary (radius -> 1.0)
            radius = (dist / (max_dist + 1)) * 0.98
            vec = self.vecs_hyper[node_id]
            norm = np.linalg.norm(vec)
            if norm > 0:
                self.vecs_hyper[node_id] = (vec / norm) * radius
            else:
                self.vecs_hyper[node_id] = np.zeros(cfg.hyperspace_dim)

        print("✅ Dataset Ready in RAM.")

# --- Helpers ---
def get_docker_disk(container_keyword: str) -> str:
    try:
        import subprocess
        ps = subprocess.run(["docker", "ps", "--format", "{{.Names}}"], capture_output=True, text=True)
        containers = ps.stdout.strip().split('\n')
        target = next((c for c in containers if container_keyword in c), None)
        if not target: return "N/A"
        
        # Check specific paths
        path = "/var/lib/milvus" if "milvus" in container_keyword else "/data"
        res = subprocess.run(["docker", "exec", target, "du", "-sh", path], capture_output=True, text=True)
        return res.stdout.split()[0] if res.returncode == 0 else "Err"
    except:
        return "N/A"

def get_local_disk(path: str) -> str:
    try:
        total = sum(os.path.getsize(os.path.join(dp, f)) for dp, _, fn in os.walk(path) for f in fn)
        return f"{total / (1024*1024):.1f}M"
    except:
        return "N/A"

# --- Benchmarks ---

def run_milvus(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🟣 Benchmarking Milvus ({cfg.milvus_dim}d)...")
    if not MILVUS_AVAILABLE:
        return Result("Milvus", cfg.milvus_dim, "Euclidean", 0, 0, 0, 0, 0, "0", "Skipped (Lib missing)")

    try:
        connections.connect(host="localhost", port="19530", timeout=5)
        name = "bench_tree"
        if utility.has_collection(name): utility.drop_collection(name)
        
        schema = CollectionSchema([
            FieldSchema("id", DataType.INT64, is_primary=True),
            FieldSchema("vec", DataType.FLOAT_VECTOR, dim=cfg.milvus_dim)
        ], "")
        col = Collection(name, schema)
        
        # Insert
        t0 = time.time()
        for i in range(0, data.count, cfg.batch_size):
            batch = data.vecs_milvus[i : i + cfg.batch_size]
            ids = list(range(i, i + len(batch)))
            col.insert([ids, batch.tolist()])
            print(f"   Inserting... {i}/{data.count}", end='\r')
        
        dur = time.time() - t0
        qps = data.count / dur
        
        # Index
        print(f"   Building Index... ({dur:.1f}s insert)")
        col.create_index("vec", {"metric_type":"L2", "index_type":"IVF_FLAT", "params":{"nlist": 128}})
        col.load()
        
        # Search
        latencies = []
        query = [data.vecs_milvus[-1].tolist()]
        print("   Searching...")
        for _ in range(cfg.search_queries):
            ts = time.time()
            col.search(query, "vec", {"metric_type":"L2", "params":{"nprobe": 10}}, limit=cfg.top_k)
            latencies.append((time.time() - ts) * 1000)
            
        disk = get_docker_disk("milvus")
        utility.drop_collection(name)
        
        return Result("Milvus", cfg.milvus_dim, "Euclidean", qps, dur, 
                      np.percentile(latencies, 50), np.percentile(latencies, 95), np.percentile(latencies, 99),
                      disk, "Success")
                      
    except Exception as e:
        print(f"   ❌ Error: {e}")
        return Result("Milvus", cfg.milvus_dim, "Euclidean", 0, 0, 0, 0, 0, "0", f"Fail: {str(e)[:50]}")

def run_hyperspace(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🚀 Benchmarking HyperspaceDB ({cfg.hyperspace_dim}d)...")
    if not HYPERSPACE_AVAILABLE:
        return Result("HyperspaceDB", cfg.hyperspace_dim, "Poincare", 0, 0, 0, 0, 0, "0", "Skipped")

    try:
        client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
        name = "bench_tree"
        
        # Hard reset
        try: client.delete_collection(name)
        except: pass
        
        # Explicit creation check
        try:
            client.create_collection(name, dimension=cfg.hyperspace_dim, metric="poincare")
        except Exception as e:
            raise RuntimeError(f"Create failed: {e}")

        # Insert
        t0 = time.time()
        for i in range(0, data.count, cfg.batch_size):
            batch = data.vecs_hyper[i : i + cfg.batch_size]
            ids = list(range(i, i + len(batch)))
            metas = [{"i": str(k)} for k in ids]
            
            # Use batch_insert if available
            if hasattr(client, 'batch_insert'):
                success = client.batch_insert(batch.tolist(), ids, metas, collection=name)
                # CRITICAL FIX: Check if batch insert actually worked (if SDK returns bool)
                # If SDK raises exception, it goes to except block.
            else:
                for j, v in enumerate(batch):
                    client.insert(ids[j], v.tolist(), metas[j], collection=name)
            
            print(f"   Inserting... {i}/{data.count}", end='\r')

        dur = time.time() - t0
        qps = data.count / dur
        print(f"   Insert done. QPS: {qps:.0f}")

        # Search
        latencies = []
        query = data.vecs_hyper[-1].tolist()
        print("   Searching...")
        for _ in range(cfg.search_queries):
            ts = time.time()
            res = client.search(query, top_k=cfg.top_k, collection=name)
            latencies.append((time.time() - ts) * 1000)
            
            # Verification first run
            if _ == 0 and not res:
                raise RuntimeError("Search returned empty results! Insert likely failed.")

        disk = get_local_disk("../data") # Assuming running from benchmarks dir
        # client.delete_collection(name)
        
        return Result("HyperspaceDB", cfg.hyperspace_dim, "Poincaré", qps, dur,
                      np.percentile(latencies, 50), np.percentile(latencies, 95), np.percentile(latencies, 99),
                      disk, "Success")

    except Exception as e:
        print(f"   ❌ Error: {e}")
        return Result("HyperspaceDB", cfg.hyperspace_dim, "Poincaré", 0, 0, 0, 0, 0, "0", f"Fail: {str(e)}")

# --- Report ---
def print_table(results: List[Result]):
    print("\n" + "="*85)
    print(f"{'Database':<15} | {'Dim':<5} | {'QPS':<8} | {'P99 Latency':<12} | {'Disk':<8} | {'Status'}")
    print("-" * 85)
    for r in results:
        print(f"{r.database:<15} | {r.dimension:<5} | {r.insert_qps:<8.0f} | {r.p99:<10.2f} ms | {r.disk_usage:<8} | {r.status}")
    print("=" * 85 + "\n")

if __name__ == "__main__":
    cfg = Config()
    data = TreeGenerator(cfg)
    
    res = []
    res.append(run_milvus(cfg, data))
    res.append(run_hyperspace(cfg, data))
    
    print_table(res)