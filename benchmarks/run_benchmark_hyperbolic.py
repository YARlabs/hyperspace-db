#!/usr/bin/env python3
"""
Comprehensive Hyperbolic Efficiency Benchmark
Compares HyperspaceDB, Milvus, Qdrant, and Weaviate.
Metrics: QPS, Latency (P50/P95/P99), Recall@10, MRR, Disk Usage.
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
from typing import List, Optional, Tuple

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

# --- Config ---
@dataclass
class Config:
    num_nodes: int = 1_000_000      # Dataset size
    branching: int = 6            # Branching factor for Tree
    milvus_dim: int = 1024        # High dim for Euclidean baseline
    hyper_dim: int = 64           # Low dim for Hyperbolic efficiency
    batch_size: int = 1000
    search_queries: int = 10_000  # total queries for latency testing
    test_queries: int = 1_000       # queries for recall/accuracy testing
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
    recall: float
    mrr: float
    disk_usage: str
    status: str

# --- Accuracy Helpers ---
def calculate_accuracy(results: List[List[int]], ground_truth: List[List[int]], k: int) -> Tuple[float, float]:
    """Calculates Recall@K and MRR"""
    if not results or not ground_truth: return 0.0, 0.0
    recalls = []
    mrrs = []
    for res, gt in zip(results, ground_truth):
        # Recall@K
        gt_set = set(gt[:k])
        intersection = set(res[:k]) & gt_set
        recall = len(intersection) / len(gt_set) if gt_set else 0
        recalls.append(recall)
        
        # MRR
        mrr = 0
        for i, idx in enumerate(res):
            if idx in gt_set:
                mrr = 1.0 / (i + 1)
                break
        mrrs.append(mrr)
    return statistics.mean(recalls), statistics.mean(mrrs)

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

        # 2. Euclidean Embedding (1024d)
        print(f"   -> Embedding {cfg.milvus_dim}d Euclidean vectors...")
        self.vecs_euc = np.random.randn(self.count, cfg.milvus_dim).astype(np.float32)
        norms = np.linalg.norm(self.vecs_euc, axis=1, keepdims=True)
        self.vecs_euc /= (norms + 1e-9)

        # 3. Hyperbolic Embedding (64d)
        print(f"   -> Embedding {cfg.hyper_dim}d Poincaré vectors...")
        self.vecs_hyper = np.random.uniform(-0.01, 0.01, size=(self.count, cfg.hyper_dim)).astype(np.float32)
        
        paths = nx.shortest_path_length(G, source=0)
        max_dist = max(paths.values()) if paths else 1
        
        for node_id, dist in paths.items():
            if node_id >= self.count: continue
            radius = (dist / (max_dist + 1)) * 0.98
            vec = self.vecs_hyper[node_id]
            norm = np.linalg.norm(vec)
            if norm > 0:
                self.vecs_hyper[node_id] = (vec / norm) * radius
            else:
                self.vecs_hyper[node_id] = np.zeros(cfg.hyper_dim)

        # 4. Ground Truth Calculation (Expensive)
        print(f"   -> Computing Ground Truth for {cfg.test_queries} queries...")
        self.test_ids = np.random.choice(self.count, cfg.test_queries, replace=False)
        self.query_vecs_euc = self.vecs_euc[self.test_ids]
        self.query_vecs_hyper = self.vecs_hyper[self.test_ids]

        # Euclidean GT (L2/Cosine)
        print("      * Euclidean GT...")
        self.gt_euc = []
        for q_vec in self.query_vecs_euc:
            # For normalized vectors, L2 order is same as Cosine order
            dists = np.sum((self.vecs_euc - q_vec)**2, axis=1)
            self.gt_euc.append(np.argsort(dists)[:cfg.top_k].tolist())

        # Poincaré GT
        print("      * Poincaré GT...")
        self.gt_hyper = []
        for q_vec in self.query_vecs_hyper:
            q_norm_sq = np.sum(q_vec**2)
            v_norms_sq = np.sum(self.vecs_hyper**2, axis=1)
            diff_sq = np.sum((self.vecs_hyper - q_vec)**2, axis=1)
            # Poincaré distance is monotonic with (diff_sq / ((1-q_norm_sq)*(1-v_norms_sq)))
            dists = diff_sq / ((1 - q_norm_sq) * (1 - v_norms_sq) + 1e-12)
            self.gt_hyper.append(np.argsort(dists)[:cfg.top_k].tolist())

        print("✅ Dataset and Ground Truth Ready.")

# --- Helpers ---
def get_docker_disk(container_keyword: str) -> str:
    try:
        ps = subprocess.run(["docker", "ps", "--format", "{{.Names}}"], capture_output=True, text=True)
        containers = ps.stdout.strip().split('\n')
        target = next((c for c in containers if container_keyword in c), None)
        if not target: return "N/A"
        path = "/var/lib/milvus" if "milvus" in container_keyword else ("/qdrant/storage" if "qdrant" in container_keyword else "/data")
        res = subprocess.run(["docker", "exec", target, "du", "-sh", path], capture_output=True, text=True)
        return res.stdout.split()[0] if res.returncode == 0 else "Err"
    except: return "N/A"

def get_local_disk(path: str) -> str:
    try:
        total = sum(os.path.getsize(os.path.join(dp, f)) for dp, _, fn in os.walk(path) for f in fn if not os.path.islink(os.path.join(dp, f)))
        return f"{total / (1024*1024):.1f}M"
    except: return "N/A"

def detect_hyperspace_metric(host: str) -> Optional[str]:
    url = f"http://{host}:50051/api/status"
    try:
        with urllib.request.urlopen(url, timeout=3) as resp:
            payload = json.loads(resp.read().decode("utf-8"))
            return payload.get("config", {}).get("metric", "").lower()
    except: return None

def log_batch(i, total, batch_start, last_batch_qps, size=1000):
    dur = time.time() - batch_start
    qps = size / dur if dur > 0 else 0
    diff = f"({'+' if (qps-last_batch_qps)>=0 else ''}{((qps-last_batch_qps)/last_batch_qps*100):.1f}%)" if last_batch_qps > 0 else ""
    print(f"   [Batch] {i+size:7,}/{total:,} | QPS: {qps:6.0f} {diff:8} | Time: {dur:4.3f}s", end='\r')
    return qps

# --- Benchmark Runs ---

def run_milvus(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🟣 Milvus ({cfg.milvus_dim}d Euclidean)")
    if not MILVUS_AVAILABLE: return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", 0,0,0,0,0,0,0, "N/A", "Skipped")
    try:
        connections.connect(host=cfg.host, port="19530", timeout=10)
        name = "bench_hyper_suite"
        if utility.has_collection(name): utility.drop_collection(name)
        schema = CollectionSchema([FieldSchema("id", DataType.INT64, is_primary=True), FieldSchema("vec", DataType.FLOAT_VECTOR, dim=cfg.milvus_dim)])
        col = Collection(name, schema)
        
        t0 = time.time(); last_qps = 0
        for i in range(0, data.count, cfg.batch_size):
            bs = time.time()
            batch = data.vecs_euc[i : i + cfg.batch_size]
            col.insert([list(range(i, i + len(batch))), batch.tolist()])
            last_qps = log_batch(i, data.count, bs, last_qps, cfg.batch_size)
        dur = time.time() - t0
        
        print("\n   -> Building Index...")
        col.create_index("vec", {"metric_type":"L2", "index_type":"IVF_FLAT", "params":{"nlist": 128}})
        col.load()
        
        # Accuracy
        print(f"   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        results = []
        for q_vec in data.query_vecs_euc:
            res = col.search([q_vec.tolist()], "vec", {"metric_type":"L2", "params":{"nprobe": 10}}, limit=cfg.top_k)
            results.append([hit.id for hit in res[0]])
        recall, mrr = calculate_accuracy(results, data.gt_euc, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = [data.query_vecs_euc[0].tolist()]
        for _ in range(cfg.search_queries):
            ts = time.time()
            col.search(q_one, "vec", {"metric_type":"L2", "params":{"nprobe": 10}}, limit=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
            
        disk = get_docker_disk("milvus")
        utility.drop_collection(name)
        return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), recall, mrr, disk, "Success")
    except Exception as e: return Result("Milvus", cfg.milvus_dim, "Euclidean", "L2", 0,0,0,0,0,0,0, "0", f"Fail: {str(e)[:50]}")

def run_qdrant(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🔷 Qdrant ({cfg.milvus_dim}d Euclidean)")
    if not QDRANT_AVAILABLE: return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0,0,0, "N/A", "Skipped")
    try:
        client = QdrantClient(host=cfg.host, port=6333)
        name = "bench_hyper_suite"
        try: client.delete_collection(name)
        except: pass
        client.create_collection(name, vectors_config=VectorParams(size=cfg.milvus_dim, distance=Distance.COSINE))
        
        t0 = time.time(); last_qps = 0
        for i in range(0, data.count, cfg.batch_size):
            bs = time.time()
            batch = data.vecs_euc[i : i + cfg.batch_size]
            client.upsert(name, [PointStruct(id=i+j, vector=v.tolist()) for j, v in enumerate(batch)], wait=True)
            last_qps = log_batch(i, data.count, bs, last_qps, cfg.batch_size)
        dur = time.time() - t0
        
        # Accuracy
        print(f"\n   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        results = []
        for q_vec in data.query_vecs_euc:
            res = client.query_points(name, q_vec.tolist(), limit=cfg.top_k)
            results.append([hit.id for hit in res.points])
        recall, mrr = calculate_accuracy(results, data.gt_euc, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = data.query_vecs_euc[0].tolist()
        for _ in range(cfg.search_queries):
            ts = time.time()
            client.query_points(name, q_one, limit=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
            
        disk = get_docker_disk("qdrant")
        client.delete_collection(name)
        return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), recall, mrr, disk, "Success")
    except Exception as e: return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0,0,0, "0", f"Fail: {str(e)[:50]}")

def run_hyperspace(cfg: Config, data: TreeGenerator, use_hyper: bool) -> Result:
    dim = cfg.hyper_dim if use_hyper else cfg.milvus_dim
    metric = "poincare" if use_hyper else "cosine"
    geom = "Poincaré" if use_hyper else "Euclidean"
    label = f"HyperspaceDB ({geom} {dim}d)"
    print(f"\n🚀 {label}")
    if not HYPERSPACE_AVAILABLE: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0, "N/A", "Skipped")
    try:
        client = HyperspaceClient(f"{cfg.host}:50051", api_key="I_LOVE_HYPERSPACEDB")
        server_metric = detect_hyperspace_metric(cfg.host)
        if server_metric in ("poincare", "hyperbolic") and not use_hyper: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0, "N/A", f"Skipped: server={server_metric}")
        if server_metric in ("cosine", "l2", "euclidean") and use_hyper: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0, "N/A", f"Skipped: server={server_metric}")

        name = "bench_suite_hyper" if use_hyper else "bench_suite_euc"
        client.delete_collection(name)
        client.create_collection(name, dimension=dim, metric=metric)
        
        vecs = data.vecs_hyper if use_hyper else data.vecs_euc
        q_vecs = data.query_vecs_hyper if use_hyper else data.query_vecs_euc
        gt = data.gt_hyper if use_hyper else data.gt_euc
        
        t0 = time.time(); last_qps = 0
        h_batch = 1000 if use_hyper else 400
        for i in range(0, data.count, h_batch):
            bs = time.time()
            batch = vecs[i : i + h_batch]
            ids = list(range(i, i + len(batch)))
            client.batch_insert(batch.tolist(), ids, [{"i": str(k)} for k in ids], collection=name)
            last_qps = log_batch(i, data.count, bs, last_qps, h_batch)
        dur = time.time() - t0
        
        # Accuracy
        print(f"\n   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        results = []
        for q_vec in q_vecs:
            res = client.search(q_vec.tolist(), top_k=cfg.top_k, collection=name)
            results.append([hit.id for hit in res])
        recall, mrr = calculate_accuracy(results, gt, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = q_vecs[0].tolist()
        for _ in range(cfg.search_queries):
            ts = time.time()
            client.search(q_one, top_k=cfg.top_k, collection=name)
            lats.append((time.time() - ts) * 1000)
            
        disk = get_local_disk("../data")
        client.delete_collection(name)
        return Result("HyperspaceDB", dim, geom, metric, data.count/dur, dur, np.percentile(lats, 50), np.percentile(lats, 95), np.percentile(lats, 99), recall, mrr, disk, "Success")
    except Exception as e: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0, "0", f"Fail: {str(e)[:100]}")

def print_table(results: List[Result]):
    header = f"{'Database':<15} | {'Dim':<5} | {'Metric':<8} | {'QPS':<8} | {'P99 Lat':<10} | {'Recall':<7} | {'MRR':<5} | {'Disk':<8} | {'Status'}"
    print("\n" + "="*len(header))
    print(header)
    print("-" * len(header))
    # Sort by P99 for readability
    results.sort(key=lambda x: x.p99 if x.p99 > 0 else 999999)
    for r in results:
        print(f"{r.database:<15} | {r.dimension:<5} | {r.metric:<8} | {r.insert_qps:8.0f} | {r.p99:8.2f} ms | {r.recall:6.1%} | {r.mrr:4.2f} | {r.disk_usage:8} | {r.status}")
    print("=" * len(header) + "\n")

if __name__ == "__main__":
    cfg = Config()
    
    # 1. Detection Phase
    print(f"🔍 Detecting HyperspaceDB configuration on {cfg.host}:50051...")
    server_metric = detect_hyperspace_metric(cfg.host)
    
    if server_metric:
        print(f"✨ HyperspaceDB is running in '{server_metric}' mode.")
        is_hyper_server = server_metric in ("poincare", "hyperbolic")
    else:
        print("⚠️  Could not detect HyperspaceDB metric via API. Assuming Poincaré 64d by default.")
        is_hyper_server = True

    # 2. Data Generation Phase
    data = TreeGenerator(cfg)
    
    # 3. Execution Phase
    res = []
    
    # Run Competitors (Always Euclidean 1024d)
    res.append(run_milvus(cfg, data))
    res.append(run_qdrant(cfg, data))
    
    # Run Hyperspace (Only matching mode)
    res.append(run_hyperspace(cfg, data, use_hyper=is_hyper_server))
    
    # 4. Reporting
    print_table(res)
    
    with open("BENCHMARK_STORY.md", "w") as f:
        f.write("# 📐 The Hyperbolic Advantage: Full Accuracy Suite\n\n")
        f.write(f"Testing with **{cfg.num_nodes:,}** nodes. Accuracy based on **{cfg.test_queries}** query vectors.\n")
        f.write(f"HyperspaceDB Mode: **{'Poincaré 64d' if is_hyper_server else 'Euclidean 1024d'}**\n\n")
        f.write("| Database | Dim | Geometry | Metric | QPS | P99 | Recall@10 | MRR | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for r in res:
            if r.status == "Success":
                f.write(f"| **{r.database}** | {r.dimension} | {r.geometry} | {r.metric} | {r.insert_qps:,.0f} | {r.p99:.2f}ms | {r.recall:.1%} | {r.mrr:.2f} | {r.disk_usage} |\n")
        
        f.write("\n## 💡 Accuracy Analysis\n")
        h_hyp = next((r for r in res if r.database == "HyperspaceDB" and r.geometry == "Poincaré"), None)
        if h_hyp:
            f.write(f"HyperspaceDB Poincaré ({h_hyp.recall:.1%} recall) demonstrates that accuracy remains high despite a **{(1024/64):.0f}x reduction** in dimensions.\n")
        elif is_hyper_server == False:
            f.write("HyperspaceDB is currently tested in Euclidean mode. Point the server to Poincaré to see the Hyperbolic Advantage.\n")
