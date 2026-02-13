#!/usr/bin/env python3
"""
Comprehensive Hyperbolic Efficiency Benchmark
Compares HyperspaceDB, Milvus, Qdrant, and Weaviate.
Scenarios: 
1. High-dim Euclidean baseline (1024d) for competitors.
2. Low-dim Poincaré (64d) for HyperspaceDB advantage.
3. HyperspaceDB baseline in Euclidean space (1024d).
"""

import time
import numpy as np
import networkx as nx
import sys
import os
import statistics
import json
import subprocess
import urllib.request
import urllib.error
from dataclasses import dataclass, asdict
from typing import List, Optional

# Ensure local SDK is used
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

# --- Imports & Availability ---
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

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, VectorParams, PointStruct
    QDRANT_AVAILABLE = True
except ImportError:
    QDRANT_AVAILABLE = False

try:
    import weaviate
    import weaviate.classes as wvc
    WEAVIATE_AVAILABLE = True
except ImportError:
    WEAVIATE_AVAILABLE = False

# --- Config ---
@dataclass
class Config:
    num_nodes: int = 1_000_000      # Dataset size
    branching: int = 6            # Branching factor for Tree
    milvus_dim: int = 1024        # High dim for Euclidean baseline
    hyper_dim: int = 64           # Low dim for Hyperbolic efficiency
    batch_size: int = 1000
    search_queries: int = 10_000
    top_k: int = 10
    host: str = "localhost"

@dataclass
class Result:
    database: str
    dimension: int
    geometry: str
    metric: str
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
        print(f"🌳 Generating Hierarchy ({cfg.num_nodes} nodes)...")
        
        # 1. Structure
        depth = int(np.log(cfg.num_nodes) / np.log(cfg.branching))
        G = nx.balanced_tree(cfg.branching, depth)
        nodes = list(G.nodes())
        if len(nodes) > cfg.num_nodes:
            G.remove_nodes_from(nodes[cfg.num_nodes:])
        
        self.count = G.number_of_nodes()
        print(f"   Graph structure created: {self.count} nodes.")

        # 2. Euclidean Embedding (1024d) - Baseline for everyone
        print(f"   -> Embedding {cfg.milvus_dim}d Euclidean vectors...")
        self.vecs_euc = np.random.randn(self.count, cfg.milvus_dim).astype(np.float32)
        norms = np.linalg.norm(self.vecs_euc, axis=1, keepdims=True)
        self.vecs_euc /= (norms + 1e-9)

        # 3. Hyperbolic Embedding (64d) - Poincaré advantage
        print(f"   -> Embedding {cfg.hyper_dim}d Poincaré vectors...")
        self.vecs_hyper = np.random.uniform(-0.01, 0.01, size=(self.count, cfg.hyper_dim)).astype(np.float32)
        
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
                self.vecs_hyper[node_id] = np.zeros(cfg.hyper_dim)

        print("✅ Dataset Ready in RAM.")

# --- Helpers ---
def get_docker_disk(container_keyword: str) -> str:
    try:
        ps = subprocess.run(["docker", "ps", "--format", "{{.Names}}"], capture_output=True, text=True)
        containers = ps.stdout.strip().split('\n')
        target = next((c for c in containers if container_keyword in c), None)
        if not target: return "N/A"
        
        # Default paths for different DBs
        if "milvus" in container_keyword: path = "/var/lib/milvus"
        elif "qdrant" in container_keyword: path = "/qdrant/storage"
        elif "weaviate" in container_keyword: path = "/var/lib/weaviate"
        else: path = "/data"
            
        res = subprocess.run(["docker", "exec", target, "du", "-sh", path], capture_output=True, text=True)
        return res.stdout.split()[0] if res.returncode == 0 else "Err"
    except:
        return "N/A"

def get_local_disk(path: str) -> str:
    try:
        total = sum(os.path.getsize(os.path.join(dp, f)) for dp, _, fn in os.walk(path) for f in fn if not os.path.islink(os.path.join(dp, f)))
        return f"{total / (1024*1024):.1f}M"
    except:
        return "N/A"

def detect_hyperspace_metric(host: str) -> Optional[str]:
    url = f"http://{host}:50051/api/status"
    try:
        with urllib.request.urlopen(url, timeout=3) as resp:
            payload = json.loads(resp.read().decode("utf-8"))
            metric = payload.get("config", {}).get("metric")
            if isinstance(metric, str):
                return metric.lower()
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError):
        return None
    return None

# --- Benchmark Implementations ---

def run_milvus(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🟣 Milvus ({cfg.milvus_dim}d Euclidean)")
    if not MILVUS_AVAILABLE: return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", 0,0,0,0,0, "N/A", "Skipped")
    try:
        connections.connect(host=cfg.host, port="19530", timeout=10)
        name = "bench_hyper_suite"
        if utility.has_collection(name): utility.drop_collection(name)
        
        schema = CollectionSchema([
            FieldSchema("id", DataType.INT64, is_primary=True),
            FieldSchema("vec", DataType.FLOAT_VECTOR, dim=cfg.milvus_dim)
        ])
        col = Collection(name, schema)
        
        t0 = time.time()
        for i in range(0, data.count, cfg.batch_size):
            batch = data.vecs_euc[i : i + cfg.batch_size]
            col.insert([list(range(i, i + len(batch))), batch.tolist()])
        dur = time.time() - t0
        
        col.create_index("vec", {"metric_type":"L2", "index_type":"IVF_FLAT", "params":{"nlist": 128}})
        col.load()
        
        lats = []
        q = [data.vecs_euc[0].tolist()]
        for _ in range(cfg.search_queries):
            ts = time.time()
            col.search(q, "vec", {"metric_type":"L2", "params":{"nprobe": 10}}, limit=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
            
        disk = get_docker_disk("milvus")
        utility.drop_collection(name)
        return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), disk, "Success")
    except Exception as e:
        return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", 0,0,0,0,0, "0", f"Fail: {str(e)[:50]}")

def run_qdrant(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🔷 Qdrant ({cfg.milvus_dim}d Euclidean)")
    if not QDRANT_AVAILABLE: return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0, "N/A", "Skipped")
    try:
        client = QdrantClient(host=cfg.host, port=6333)
        name = "bench_hyper_suite"
        try: client.delete_collection(name)
        except: pass
        client.create_collection(name, vectors_config=VectorParams(size=cfg.milvus_dim, distance=Distance.COSINE))
        
        t0 = time.time()
        for i in range(0, data.count, cfg.batch_size):
            batch = data.vecs_euc[i : i + cfg.batch_size]
            points = [PointStruct(id=i+j, vector=v.tolist()) for j, v in enumerate(batch)]
            client.upsert(collection_name=name, points=points, wait=True)
        dur = time.time() - t0
        
        lats = []
        q = data.vecs_euc[0].tolist()
        for _ in range(cfg.search_queries):
            ts = time.time()
            client.query_points(collection_name=name, query=q, limit=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
            
        disk = get_docker_disk("qdrant")
        client.delete_collection(name)
        return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), disk, "Success")
    except Exception as e:
        return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0, "0", f"Fail: {str(e)[:50]}")

# def run_weaviate(cfg: Config, data: TreeGenerator) -> Result:
#     print(f"\n🟢 Weaviate ({cfg.milvus_dim}d Euclidean)")
#     if not WEAVIATE_AVAILABLE: return Result("Weaviate", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0, "N/A", "Skipped")
#     try:
#         client = weaviate.connect_to_local(port=8080, grpc_port=50052)
#         name = "BenchmarkSuite"
#         try: client.collections.delete(name)
#         except: pass
#         try:
#             col = client.collections.create(
#                 name=name,
#                 vector_config=wvc.config.Configure.Vectors.self_provided()
#             )
#         except Exception:
#             # Backward compatibility with older Weaviate SDK versions.
#             col = client.collections.create(
#                 name=name,
#                 vectorizer_config=wvc.config.Configure.Vectorizer.none()
#             )
        
#         t0 = time.time()
#         for i in range(0, data.count, cfg.batch_size):
#             batch = data.vecs_euc[i : i + cfg.batch_size]
#             with col.batch.dynamic() as b:
#                 for j, v in enumerate(batch):
#                     b.add_object(properties={"idx": i+j}, vector=v.tolist())
#         dur = time.time() - t0
        
#         lats = []
#         q = data.vecs_euc[0].tolist()
#         for _ in range(cfg.search_queries):
#             ts = time.time()
#             col.query.near_vector(near_vector=q, limit=cfg.top_k)
#             lats.append((time.time() - ts) * 1000)
            
#         disk = get_docker_disk("weaviate")
#         client.collections.delete(name)
#         client.close()
#         return Result("Weaviate", cfg.milvus_dim, "Euclidean", "Cosine", data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), disk, "Success")
#     except Exception as e:
#         return Result("Weaviate", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0, "0", f"Fail: {str(e)[:50]}")

def run_hyperspace(cfg: Config, data: TreeGenerator, use_hyper: bool) -> Result:
    dim = cfg.hyper_dim if use_hyper else cfg.milvus_dim
    metric = "poincare" if use_hyper else "cosine"
    geom = "Poincaré" if use_hyper else "Euclidean"
    label = f"HyperspaceDB ({geom} {dim}d)"
    
    print(f"\n🚀 {label}")
    if not HYPERSPACE_AVAILABLE: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "N/A", "Skipped")
    
    try:
        client = HyperspaceClient(f"{cfg.host}:50051", api_key="I_LOVE_HYPERSPACEDB")
        server_metric = detect_hyperspace_metric(cfg.host)
        if server_metric in ("poincare", "hyperbolic") and not use_hyper:
            return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "N/A", f"Skipped: server metric={server_metric}")
        if server_metric in ("cosine", "l2", "euclidean") and use_hyper:
            return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "N/A", f"Skipped: server metric={server_metric}")

        name = "bench_suite_hyper" if use_hyper else "bench_suite_euc"
        client.delete_collection(name)
        if not client.create_collection(name, dimension=dim, metric=metric):
            return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "0", f"Fail: create_collection({name})")
        
        vecs = data.vecs_hyper if use_hyper else data.vecs_euc
        t0 = time.time()
        batch_size = 1000 if use_hyper else 400 # Small batch for large vectors
        for i in range(0, data.count, batch_size):
            batch = vecs[i : i + batch_size]
            ids = list(range(i, i + len(batch)))
            metas = [{"i": str(k)} for k in ids]
            ok = client.batch_insert(batch.tolist(), ids, metas, collection=name)
            if not ok:
                return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "0", f"Fail: batch_insert({name})")
        dur = time.time() - t0
        
        lats = []
        q = vecs[0].tolist()
        for _ in range(cfg.search_queries):
            ts = time.time()
            res = client.search(q, top_k=cfg.top_k, collection=name)
            if not res:
                return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "0", f"Fail: empty search({name})")
            lats.append((time.time() - ts) * 1000)
            
        disk = get_local_disk("../data")
        client.delete_collection(name)
        return Result("HyperspaceDB", dim, geom, metric, data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), disk, "Success")
    except Exception as e:
        return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0, "0", f"Fail: {str(e)[:100]}")

def print_table(results: List[Result]):
    print("\n" + "="*110)
    print(f"{'Database':<15} | {'Geom':<10} | {'Metric':<8} | {'Dim':<5} | {'QPS':<8} | {'P99 Lat':<10} | {'Disk':<8} | {'Status'}")
    print("-" * 110)
    results.sort(key=lambda x: x.p99 if x.p99 > 0 else 999999)
    for r in results:
        print(f"{r.database:<15} | {r.geometry:<10} | {r.metric:<8} | {r.dimension:<5} | {r.insert_qps:<8.0f} | {r.p99:<8.2f} ms | {r.disk_usage:<8} | {r.status}")
    print("=" * 110 + "\n")

if __name__ == "__main__":
    cfg = Config()
    data = TreeGenerator(cfg)
    
    res = []
    res.append(run_milvus(cfg, data))
    res.append(run_qdrant(cfg, data))
    # res.append(run_weaviate(cfg, data))
    res.append(run_hyperspace(cfg, data, use_hyper=False))
    res.append(run_hyperspace(cfg, data, use_hyper=True))
    
    print_table(res)
    
    # Write to report
    with open("BENCHMARK_STORY.md", "w") as f:
        f.write("# 📐 The Hyperbolic Advantage: Absolute Benchmark\n\n")
        f.write(f"Testing with **{cfg.num_nodes:,}** nodes in a hierarchical taxonomy.\n")
        f.write("| Database | Geometry | Metric | Dim | Ingest QPS | Search P99 | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for r in res:
            f.write(f"| **{r.database}** | {r.geometry} | {r.metric} | {r.dimension} | {r.insert_qps:,.0f} | {r.p99:.2f} ms | {r.disk_usage} |\n")
        
        f.write("\n## 💡 Key Takeaways\n")
        h_hyp = next((r for r in res if r.database == "HyperspaceDB" and r.geometry == "Poincaré"), None)
        others = [r for r in res if r.database != "HyperspaceDB"]
        if h_hyp and others:
            best_other = min(others, key=lambda x: x.p99 if x.p99 > 0 else 9999)
            if best_other.p99 > 0:
                speedup = best_other.p99 / h_hyp.p99
                f.write(f"1. **Latency**: HyperspaceDB ({h_hyp.dimension}d) is **{speedup:.1f}x faster** than {best_other.database} ({best_other.dimension}d).\n")
            
            milvus = next((r for r in res if r.database == "Milvus"), None)
            if milvus and milvus.disk_usage != "N/A" and h_hyp.disk_usage != "N/A":
                # Very rough parser
                try:
                    m_val = float(milvus.disk_usage.replace('M', '').replace('G', '000'))
                    h_val = float(h_hyp.disk_usage.replace('M', '').replace('G', '000'))
                    f.write(f"2. **Efficiency**: HyperspaceDB uses **{m_val/h_val:.1f}x less disk** space compared to Milvus.\n")
                except: pass
