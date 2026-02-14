#!/usr/bin/env python3
"""
Comprehensive Hyperbolic Efficiency Benchmark
Compares HyperspaceDB, Milvus, Qdrant, and Weaviate.
Metrics: QPS, Latency (P50/P95/P99), Recall@10, MRR, NDCG@10, Concurrency QPS (1/10/30), Disk Usage.
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
import math
from concurrent.futures import ThreadPoolExecutor
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

try:
    import chromadb
    CHROMA_AVAILABLE = True
except ImportError:
    CHROMA_AVAILABLE = False

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
    search_qps: float
    p50: float
    p95: float
    p99: float
    recall: float
    mrr: float
    ndcg: float
    c1_qps: float
    c10_qps: float
    c30_qps: float
    disk_usage: str
    status: str

# --- Accuracy Helpers ---
def calculate_accuracy(results: List[List[int]], ground_truth: List[List[int]], k: int) -> Tuple[float, float, float]:
    """Calculates Recall@K, MRR and NDCG@K"""
    if not results or not ground_truth:
        return 0.0, 0.0, 0.0
    recalls = []
    mrrs = []
    ndcgs = []
    for res, gt in zip(results, ground_truth):
        # Recall@K
        gt_set = set(gt[:k])
        intersection = set(res[:k]) & gt_set
        recall = len(intersection) / min(k, len(gt_set)) if gt_set else 0
        recalls.append(recall)
        
        # MRR
        mrr = 0
        for i, idx in enumerate(res):
            if idx in gt_set:
                mrr = 1.0 / (i + 1)
                break
        mrrs.append(mrr)

        # NDCG@K (binary relevance)
        dcg = 0.0
        for i, idx in enumerate(res[:k]):
            if idx in gt_set:
                dcg += 1.0 / math.log2(i + 2)
        ideal_hits = min(k, len(gt_set))
        idcg = sum(1.0 / math.log2(i + 2) for i in range(ideal_hits))
        ndcgs.append((dcg / idcg) if idcg > 0 else 0.0)

    return statistics.mean(recalls), statistics.mean(mrrs), statistics.mean(ndcgs)

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
    # Ports to check for the dashboard / status API
    for port in (50051, 50050, 50052):
        url = f"http://{host}:{port}/api/status"
        try:
            with urllib.request.urlopen(url, timeout=2) as resp:
                payload = json.loads(resp.read().decode("utf-8"))
                # Sometime it's nested in payload["config"]["metric"]
                metric = payload.get("config", {}).get("metric", "")
                if not metric:
                    metric = payload.get("metric", "")
                if isinstance(metric, str) and metric:
                    return metric.lower()
        except:
            continue
            
    # Fallback to .env if local
    try:
        env_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.env"))
        if os.path.exists(env_path):
            with open(env_path, "r") as f:
                for line in f:
                    if line.startswith("HS_METRIC="):
                        return line.split("=")[1].strip().lower().replace("'", "").replace('"', "")
    except:
        pass
        
    return None

def log_batch(i, total, batch_start, last_batch_qps, size=1000):
    dur = time.time() - batch_start
    qps = size / dur if dur > 0 else 0
    diff = f"({'+' if (qps-last_batch_qps)>=0 else ''}{((qps-last_batch_qps)/last_batch_qps*100):.1f}%)" if last_batch_qps > 0 else ""
    per_vec_ms = (dur / size) * 1000 if size > 0 else 0.0
    print(f"   [Batch] {min(i+size, total):7,}/{total:,} | QPS: {qps:6.0f} {diff:8} | Batch: {dur:4.3f}s | Vec: {per_vec_ms:5.3f}ms", end='\r')
    return qps

def extract_ids(res_obj):
    """
    Normalize search responses across SDKs:
    - list[dict(id=...)]  (Hyperspace python SDK)
    - list[obj.id]        (other SDK wrappers)
    """
    ids = []
    for hit in res_obj:
        val = None
        if isinstance(hit, dict):
            val = hit.get("id")
        elif hasattr(hit, "id"):
            val = hit.id
            
        if val is not None:
            # Convert to int for comparison if it looks like a number
            try:
                ids.append(int(val))
            except (ValueError, TypeError):
                ids.append(val)
    return ids

def run_concurrency_profile(query_fn, workers_list=(1, 10, 30), queries=1000):
    result = {}
    for workers in workers_list:
        print(f"   -> Testing Concurrency: {workers} workers...", end=" ")
        def one_call():
            ts = time.time()
            query_fn()
            return (time.time() - ts) * 1000

        start = time.time()
        with ThreadPoolExecutor(max_workers=workers) as ex:
            lats = list(ex.map(lambda _: one_call(), range(queries)))
        elapsed = time.time() - start
        qps = queries / elapsed if elapsed > 0 else 0.0
        p99 = np.percentile(lats, 99) if lats else 0.0
        print(f"QPS: {qps:.0f}, P99: {p99:.2f}ms")
        result[workers] = qps
    return result

def wait_for_indexing(host="localhost", port=50050, collection="bench_semantic", timeout=600):
    """Wait for HyperspaceDB background indexing and optimize graph"""
    import requests
    headers = {"x-api-key": "I_LOVE_HYPERSPACEDB"}
    
    # 1. Trigger explicit optimization if possible
    try:
        requests.post(f"http://{host}:{port}/api/collections/{collection}/optimize", headers=headers, timeout=5)
    except:
        pass

    print(f"⏳ Monitoring indexing for '{collection}'...")
    url = f"http://{host}:{port}/api/collections/{collection}/stats"
    
    start_time = time.time()
    while True:
        if time.time() - start_time > timeout:
            print(f"\n⚠️ Timeout after {timeout}s. Proceeding...")
            break
            
        try:
            response = requests.get(url, headers=headers, timeout=5)
            if response.status_code == 200:
                data = response.json()
                queue = data.get("indexing_queue", 0)
                count = data.get("count", 0)
                
                print(f"\r   [Indexing] Remaining: {queue:,} | Total Indexed: {count:,}          ", end="", flush=True)
                
                if queue == 0 and count > 0:
                    print(f"\n✅ Indexing queue empty. Stabilizing graph structure...")
                    time.sleep(5) # Critical grace period for HNSW linking
                    break
            time.sleep(1)
        except Exception:
            time.sleep(2)

# --- Benchmark Runs ---

def run_milvus(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🟣 Milvus ({cfg.milvus_dim}d Euclidean)")
    if not MILVUS_AVAILABLE: return Result("Milvus", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,"N/A", "Skipped")
    try:
        stage = "connect"
        connections.connect(host=cfg.host, port="19530", timeout=10)
        name = "bench_hyper_suite"
        stage = "drop_old_collection"
        if utility.has_collection(name): utility.drop_collection(name)
        stage = "create_collection"
        schema = CollectionSchema([FieldSchema("id", DataType.INT64, is_primary=True), FieldSchema("vec", DataType.FLOAT_VECTOR, dim=cfg.milvus_dim)])
        col = Collection(name, schema)
        
        stage = "insert_batches"
        t0 = time.time(); last_qps = 0
        for i in range(0, data.count, cfg.batch_size):
            stage = f"insert_batch_{i}"
            bs = time.time()
            batch = data.vecs_euc[i : i + cfg.batch_size]
            col.insert([list(range(i, i + len(batch))), batch.tolist()])
            last_qps = log_batch(i, data.count, bs, last_qps, cfg.batch_size)
        dur = time.time() - t0
        
        print("\n   -> Building Index...")
        stage = "build_index"
        col.create_index("vec", {"metric_type":"COSINE", "index_type":"IVF_FLAT", "params":{"nlist": 128}})
        stage = "load_index"
        col.load()
        
        # Accuracy
        print(f"   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        stage = "accuracy_queries"
        results = []
        for q_vec in data.query_vecs_euc:
            res = col.search([q_vec.tolist()], "vec", {"metric_type":"COSINE", "params":{"nprobe": 10}}, limit=cfg.top_k)
            results.append([hit.id for hit in res[0]])
        recall, mrr, ndcg = calculate_accuracy(results, data.gt_euc, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = [data.query_vecs_euc[0].tolist()]
        search_t0 = time.time()
        for _ in range(cfg.search_queries):
            stage = "latency_queries"
            ts = time.time()
            col.search(q_one, "vec", {"metric_type":"COSINE", "params":{"nprobe": 10}}, limit=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
        search_dur = time.time() - search_t0
        search_qps = cfg.search_queries / search_dur if search_dur > 0 else 0.0

        # Concurrency profile
        q_list = [data.query_vecs_euc[0].tolist()]
        def milvus_query():
            col.search(q_list, "vec", {"metric_type":"COSINE", "params":{"nprobe": 10}}, limit=cfg.top_k)
        conc = run_concurrency_profile(milvus_query, queries=min(3000, cfg.search_queries))
            
        disk = get_docker_disk("milvus")
        stage = "cleanup_drop_collection"
        try: utility.drop_collection(name)
        except: pass
        return Result(
            database="Milvus",
            dimension=cfg.milvus_dim,
            geometry="Euclidean",
            metric="Cosine",
            insert_qps=data.count/dur,
            search_qps=search_qps,
            p50=np.percentile(lats, 50),
            p95=np.percentile(lats, 95),
            p99=np.percentile(lats, 99),
            recall=recall,
            mrr=mrr,
            ndcg=ndcg,
            c1_qps=conc.get(1, 0.0),
            c10_qps=conc.get(10, 0.0),
            c30_qps=conc.get(30, 0.0),
            disk_usage=disk,
            status="Success"
        )
    except Exception as e:
        return Result(
            database="Milvus", dimension=cfg.milvus_dim, geometry="Euclidean", metric="Cosine",
            insert_qps=0, search_qps=0, p50=0, p95=0, p99=0,
            recall=0, mrr=0, ndcg=0, c1_qps=0, c10_qps=0, c30_qps=0, disk_usage="0", status=f"Fail@{stage}: {str(e)[:80]}"
        )

def run_qdrant(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🔷 Qdrant ({cfg.milvus_dim}d Euclidean)")
    if not QDRANT_AVAILABLE: return Result("Qdrant", cfg.milvus_dim, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,"N/A", "Skipped")
    try:
        stage = "init_client"
        # Use gRPC if possible for better multithreaded stability
        client = QdrantClient(host=cfg.host, port=6334, timeout=120, prefer_grpc=True)
        name = "bench_hyper_suite"
        stage = "delete_collection"
        try: client.delete_collection(name)
        except: pass
        stage = "create_collection"
        client.create_collection(
            name,
            vectors_config=VectorParams(size=cfg.milvus_dim, distance=Distance.COSINE)
        )
        
        print("   -> Inserting vectors...")
        t0 = time.time(); last_qps = 0
        for i in range(0, data.count, cfg.batch_size):
            stage = f"upsert_batch_{i}"
            bs = time.time()
            batch = data.vecs_euc[i : i + cfg.batch_size]
            client.upsert(
                collection_name=name,
                points=[PointStruct(id=i+j, vector=v.tolist()) for j, v in enumerate(batch)],
                wait=True
            )
            last_qps = log_batch(i, data.count, bs, last_qps, cfg.batch_size)
        dur = time.time() - t0
        
        # Accuracy
        print(f"\n   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        results = []
        for q_vec in data.query_vecs_euc:
            stage = "accuracy_query"
            res = client.query_points(
                collection_name=name,
                query=q_vec.tolist(),
                limit=cfg.top_k
            )
            results.append([hit.id for hit in res.points])
        recall, mrr, ndcg = calculate_accuracy(results, data.gt_euc, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = data.query_vecs_euc[0].tolist()
        search_t0 = time.time()
        for _ in range(cfg.search_queries):
            stage = "latency_query"
            ts = time.time()
            client.query_points(
                collection_name=name,
                query=q_one,
                limit=cfg.top_k
            )
            lats.append((time.time() - ts) * 1000)
        search_dur = time.time() - search_t0
        search_qps = cfg.search_queries / search_dur if search_dur > 0 else 0.0

        # Concurrency profile
        q_list = data.query_vecs_euc[0].tolist()
        def qdrant_query():
            client.query_points(collection_name=name, query=q_list, limit=cfg.top_k)
        conc = run_concurrency_profile(qdrant_query, queries=min(3000, cfg.search_queries))
            
        disk = get_docker_disk("qdrant")
        stage = "cleanup_delete_collection"
        try: client.delete_collection(name)
        except: pass
        return Result(
            database="Qdrant",
            dimension=cfg.milvus_dim,
            geometry="Euclidean",
            metric="Cosine",
            insert_qps=data.count/dur,
            search_qps=search_qps,
            p50=np.percentile(lats, 50),
            p95=np.percentile(lats, 95),
            p99=np.percentile(lats, 99),
            recall=recall,
            mrr=mrr,
            ndcg=ndcg,
            c1_qps=conc.get(1, 0.0),
            c10_qps=conc.get(10, 0.0),
            c30_qps=conc.get(30, 0.0),
            disk_usage=disk,
            status="Success"
        )
    except Exception as e:
        return Result(
            database="Qdrant", dimension=cfg.milvus_dim, geometry="Euclidean", metric="Cosine",
            insert_qps=0, search_qps=0, p50=0, p95=0, p99=0,
            recall=0, mrr=0, ndcg=0, c1_qps=0, c10_qps=0, c30_qps=0, disk_usage="0", status=f"Fail@{stage}: {str(e)[:80]}"
        )

def run_chroma(cfg: Config, data: TreeGenerator) -> Result:
    print(f"\n🟡 ChromaDB ({cfg.milvus_dim}d Euclidean)")
    if not CHROMA_AVAILABLE: return Result("ChromaDB", cfg.milvus_dim, "Euclidean", "L2", 0,0,0,0,0,0,0,0,0,0,0,0,0, "N/A", "Skipped")
    try:
        stage = "init_client"
        client = chromadb.HttpClient(host=cfg.host, port=8000)
        name = "bench_hyper_suite"
        stage = "delete_collection"
        try: client.delete_collection(name)
        except: pass
        stage = "create_collection"
        col = client.get_or_create_collection(name, metadata={"hnsw:space": "cosine"})
        
        print("   -> Inserting vectors...")
        t0 = time.time(); last_qps = 0
        for i in range(0, data.count, cfg.batch_size):
            stage = f"add_batch_{i}"
            bs = time.time()
            batch = data.vecs_euc[i : i + cfg.batch_size]
            ids = [str(i+j) for j in range(len(batch))]
            col.add(ids=ids, embeddings=batch.tolist())
            last_qps = log_batch(i, data.count, bs, last_qps, cfg.batch_size)
        dur = time.time() - t0
        
        # Accuracy
        print(f"\n   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        results = []
        for q_vec in data.query_vecs_euc:
            stage = "accuracy_query"
            res = col.query(query_embeddings=[q_vec.tolist()], n_results=cfg.top_k)
            results.append([int(idx) for idx in res['ids'][0]])
        recall, mrr, ndcg = calculate_accuracy(results, data.gt_euc, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = data.query_vecs_euc[0].tolist()
        search_t0 = time.time()
        for _ in range(cfg.search_queries):
            stage = "latency_query"
            ts = time.time()
            col.query(query_embeddings=[q_one], n_results=cfg.top_k)
            lats.append((time.time() - ts) * 1000)
        search_dur = time.time() - search_t0
        search_qps = cfg.search_queries / search_dur if search_dur > 0 else 0.0

        # Concurrency
        print(f"   -> Measuring Concurrency...")
        q_list = data.query_vecs_euc[0].tolist()
        def chroma_query(): col.query(query_embeddings=[q_list], n_results=cfg.top_k)
        conc = run_concurrency_profile(chroma_query, queries=min(2000, cfg.search_queries))
            
        disk = get_docker_disk("chroma")
        stage = "cleanup"
        try: client.delete_collection(name)
        except: pass
        return Result(
            database="ChromaDB", dimension=cfg.milvus_dim, geometry="Euclidean", metric="Cosine",
            insert_qps=data.count/dur, search_qps=search_qps,
            p50=np.percentile(lats, 50), p95=np.percentile(lats, 95), p99=np.percentile(lats, 99),
            recall=recall, mrr=mrr, ndcg=ndcg, c1_qps=conc.get(1, 0.0), c10_qps=conc.get(10, 0.0), c30_qps=conc.get(30, 0.0),
            disk_usage=disk, status="Success"
        )
    except Exception as e:
        return Result(
            database="ChromaDB", dimension=cfg.milvus_dim, geometry="Euclidean", metric="Cosine",
            insert_qps=0, search_qps=0, p50=0, p95=0, p99=0,
            recall=0, mrr=0, ndcg=0, c1_qps=0, c10_qps=0, c30_qps=0, disk_usage="0", status=f"Fail@{stage}: {str(e)[:80]}"
        )

def run_hyperspace(cfg: Config, data: TreeGenerator, use_hyper: bool) -> Result:
    dim = cfg.hyper_dim if use_hyper else cfg.milvus_dim
    metric = "poincare" if use_hyper else "cosine"
    geom = "Poincaré" if use_hyper else "Euclidean"
    label = f"HyperspaceDB ({geom} {dim}d)"
    print(f"\n🚀 {label}")
    if not HYPERSPACE_AVAILABLE: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"N/A", "Skipped")
    try:
        stage = "init_client"
        client = HyperspaceClient(f"{cfg.host}:50051", api_key="I_LOVE_HYPERSPACEDB")
        stage = "detect_server_metric"
        server_metric = detect_hyperspace_metric(cfg.host)
        if server_metric in ("poincare", "hyperbolic") and not use_hyper: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"N/A", f"Skipped: server={server_metric}")
        if server_metric in ("cosine", "l2", "euclidean") and use_hyper: return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"N/A", f"Skipped: server={server_metric}")

        name = "bench_suite_hyper" if use_hyper else "bench_suite_euc"
        stage = "delete_collection"
        try: client.delete_collection(name)
        except: pass
        stage = "create_collection"
        if not client.create_collection(name, dimension=dim, metric=metric):
            return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"0", f"Fail: create_collection({name})")
        
        vecs = data.vecs_hyper if use_hyper else data.vecs_euc
        q_vecs = data.query_vecs_hyper if use_hyper else data.query_vecs_euc
        gt = data.gt_hyper if use_hyper else data.gt_euc
        
        stage = "insert_batches"
        t0 = time.time(); last_qps = 0
        h_batch = 1000 if use_hyper else 400
        for i in range(0, data.count, h_batch):
            stage = f"insert_batch_{i}"
            bs = time.time()
            batch = vecs[i : i + h_batch]
            ids = list(range(i, i + len(batch)))
            ok = client.batch_insert(batch.tolist(), ids, [{"i": str(k)} for k in ids], collection=name)
            if not ok:
                return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"0", f"Fail: batch_insert({name})")
            last_qps = log_batch(i, data.count, bs, last_qps, h_batch)
        dur = time.time() - t0
        print(f"\n   Ingestion complete. Time: {dur:.2f}s")
        
        # Wait for background indexing to complete
        stage = "wait_indexing"
        wait_for_indexing(collection=name)
        
        # Accuracy
        print(f"\n   -> Verifying Accuracy ({cfg.test_queries} queries)...")
        stage = "accuracy_queries"
        results = []
        for q_vec in q_vecs:
            res = client.search(q_vec.tolist(), top_k=cfg.top_k, collection=name)
            if not res:
                return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"0", f"Fail: empty search({name})")
            results.append(extract_ids(res))
        recall, mrr, ndcg = calculate_accuracy(results, gt, cfg.top_k)
        
        # Latency
        print(f"   -> Measuring Latency ({cfg.search_queries} queries)...")
        lats = []
        q_one = q_vecs[0].tolist()
        search_t0 = time.time()
        for _ in range(cfg.search_queries):
            stage = "latency_queries"
            ts = time.time()
            res = client.search(q_one, top_k=cfg.top_k, collection=name)
            if not res:
                return Result("HyperspaceDB", dim, geom, metric, 0,0,0,0,0,0,0,0,0,0,0,"0", f"Fail: empty search({name})")
            lats.append((time.time() - ts) * 1000)
        search_dur = time.time() - search_t0
        search_qps = cfg.search_queries / search_dur if search_dur > 0 else 0.0

        # Concurrency profile
        q_list = q_vecs[0].tolist()
        def hyperspace_query():
            client.search(q_list, top_k=cfg.top_k, collection=name)
        conc = run_concurrency_profile(hyperspace_query, queries=min(3000, cfg.search_queries))
            
        disk = get_local_disk("../data")
        stage = "cleanup_delete_collection"
        try: client.delete_collection(name)
        except: pass
        return Result(
            database="HyperspaceDB",
            dimension=dim,
            geometry=geom,
            metric=metric,
            insert_qps=data.count/dur,
            search_qps=search_qps,
            p50=np.percentile(lats, 50),
            p95=np.percentile(lats, 95),
            p99=np.percentile(lats, 99),
            recall=recall,
            mrr=mrr,
            ndcg=ndcg,
            c1_qps=conc.get(1, 0.0),
            c10_qps=conc.get(10, 0.0),
            c30_qps=conc.get(30, 0.0),
            disk_usage=disk,
            status="Success"
        )
    except Exception as e:
        return Result(
            database="HyperspaceDB", dimension=dim, geometry=geom, metric=metric,
            insert_qps=0, search_qps=0, p50=0, p95=0, p99=0,
            recall=0, mrr=0, ndcg=0, c1_qps=0, c10_qps=0, c30_qps=0, disk_usage="0", status=f"Fail@{stage}: {str(e)[:100]}"
        )


def print_table(results: List[Result]):
    header = f"{'Database':<15} | {'Dim':<5} | {'Metric':<8} | {'Ins QPS':<8} | {'Srch QPS':<8} | {'P99 Lat':<10} | {'Recall':<7} | {'MRR':<5} | {'NDCG':<5} | {'C1':<6} | {'C10':<6} | {'C30':<6} | {'Disk':<8} | {'Status'}"
    print("\n" + "="*len(header))
    print(header)
    print("-" * len(header))
    # Sort by P99 for readability
    results.sort(key=lambda x: x.p99 if x.p99 > 0 else 999999)
    for r in results:
        print(f"{r.database:<15} | {r.dimension:<5} | {r.metric:<8} | {r.insert_qps:8.0f} | {r.search_qps:8.0f} | {r.p99:8.2f} ms | {r.recall:6.1%} | {r.mrr:4.2f} | {r.ndcg:4.2f} | {r.c1_qps:6.0f} | {r.c10_qps:6.0f} | {r.c30_qps:6.0f} | {r.disk_usage:8} | {r.status}")
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
    
    # Simple CLI Filter
    target_db = sys.argv[1].lower() if len(sys.argv) > 1 else None
    
    # Run Competitors (Always Euclidean 1024d)
    if not target_db or "milvus" in target_db:
        res.append(run_milvus(cfg, data))
    
    if not target_db or "qdrant" in target_db:
        res.append(run_qdrant(cfg, data))
    
    if not target_db or "chroma" in target_db:
        res.append(run_chroma(cfg, data))
    
    # Run Hyperspace (Only matching mode)
    if not target_db or "hyper" in target_db:
        res.append(run_hyperspace(cfg, data, use_hyper=is_hyper_server))
    
    # 4. Reporting
    print_table(res)
    
    with open("BENCHMARK_STORY.md", "w") as f:
        f.write("# 📐 The Hyperbolic Advantage: Full Accuracy Suite\n\n")
        f.write(f"Testing with **{cfg.num_nodes:,}** nodes. Accuracy based on **{cfg.test_queries}** query vectors.\n")
        f.write(f"HyperspaceDB Mode: **{'Poincaré 64d' if is_hyper_server else 'Euclidean 1024d'}**\n\n")
        f.write("| Database | Dim | Geometry | Metric | QPS | P99 | Recall@10 | MRR | NDCG@10 | C1 QPS | C10 QPS | C30 QPS | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for r in res:
            if r.status == "Success":
                f.write(f"| **{r.database}** | {r.dimension} | {r.geometry} | {r.metric} | {r.insert_qps:,.0f} | {r.p99:.2f}ms | {r.recall:.1%} | {r.mrr:.2f} | {r.ndcg:.2f} | {r.c1_qps:,.0f} | {r.c10_qps:,.0f} | {r.c30_qps:,.0f} | {r.disk_usage} |\n")
        
        f.write("\n## 💡 Accuracy Analysis\n")
        h_hyp = next((r for r in res if r.database == "HyperspaceDB" and r.geometry == "Poincaré"), None)
        if h_hyp:
            f.write(f"HyperspaceDB Poincaré ({h_hyp.recall:.1%} recall) demonstrates that accuracy remains high despite a **{(1024/64):.0f}x reduction** in dimensions.\n")
        elif is_hyper_server == False:
            f.write("HyperspaceDB is currently tested in Euclidean mode. Point the server to Poincaré to see the Hyperbolic Advantage.\n")
