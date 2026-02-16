#!/usr/bin/env python3
import time
import sys
import os
import torch
import numpy as np
import statistics
import math
import pathlib
import subprocess
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional
from tqdm import tqdm
from datasets import load_dataset
# Adjust paths as needed
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

# --- Imports ---
try:
    from hyperspace import HyperspaceClient
    HYPERSPACE_AVAILABLE = True
except ImportError as e:
    HYPERSPACE_AVAILABLE = False
    print(f"⚠️ Hyperspace SDK not found: {e}")
    # print(f"DEBUG: sys.path includes: {sys.path[-1]}")

try:
    from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType, utility
    MILVUS_AVAILABLE = True
except ImportError as e:
    MILVUS_AVAILABLE = False
    print(f"⚠️ Milvus SDK not found: {e}. Skipping Milvus.")

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, VectorParams, PointStruct
    QDRANT_AVAILABLE = True
except ImportError:
    QDRANT_AVAILABLE = False
    print("⚠️ Qdrant SDK not found. Skipping Qdrant.")

try:
    import chromadb
    CHROMA_AVAILABLE = True
except ImportError:
    CHROMA_AVAILABLE = False
    print("⚠️ ChromaDB SDK not found. Skipping Chroma.")

# --- VectorDB Bench Data Support ---
try:
    from vectordb_bench.backend.data_source import DatasetSource
    from vectordb_bench.models import CaseType
    from vectordb_bench.backend.cases import (
        # 1536D
        Performance1536D50K,
        Performance1536D500K,
        Performance1536D5M,
        Performance1536D500K1P,
        Performance1536D500K99P,
        Performance1536D5M1P,
        Performance1536D5M99P,
                
        # 768D
        Performance768D1M,
        Performance768D10M,
        Performance768D1M1P,
        Performance768D1M99P,
        Performance768D10M1P,
        Performance768D10M99P,
        Performance768D100M,

        # 1024D
        Performance1024D1M,
        Performance1024D10M,

        # Capacity Cases
        CapacityDim128,
        CapacityDim960,
        
        # Base Cases
        CapacityCase,
        PerformanceCase,
        PerformanceCustomDataset,
    )
    VB_AVAILABLE = True
except ImportError:
    VB_AVAILABLE = False
    print("⚠️ vectordb_bench not found. Case support specific to Zilliz bench will be limited to Synthetic data if requested.")

from transformers import AutoModel, AutoTokenizer

# --- CONFIGURATION ---
@dataclass
class Config:
    # Датасет (mteb/nfcorpus - медицинский поиск, отлично для семантики)
    dataset_name: str = "mteb/msmarco"
    split: str = "dev" if "msmarco" in dataset_name else "test"
    # LIMITS
    doc_limit: int = 100_000   # <--- Start with 100k. Set to 500_000 for Stress Test.
    query_limit: int = 1000    # Queries to run
    batch_size: int = 128      # Inference batch size
    
    # Baseline (Euclidean)
    model_path_base: str = "Qwen/Qwen3-Embedding-0.6B" # Замените на ваш Qwen base
    dim_base: int = 1024 # Qwen3-0.6B native dim
    
    # Ours (Hyperbolic)
    # ПУТЬ К ВАШЕЙ МОДЕЛИ!
    model_path_hyp: str = "./data/v5_Embedding_v0.1a.pth" 
    dim_hyp: int = 64
    HYPER_MODE: str = "cosine" # "poincare" or "cosine"
    
    # Selected Case
    target_case: Optional[str] = None
    
    def apply_case(self, case_name: str):
        """Apply Zilliz Bench Case parameters"""
        case_name = case_name.lower()
        self.dataset_name = "Synthetic/Random"
        self.doc_limit = 0
        
        # Dimensions & Counts
        if "1536d" in case_name: self.dim_base = 1536
        elif "1024d" in case_name: self.dim_base = 1024
        elif "768d" in case_name: self.dim_base = 768
        elif "960d" in case_name: self.dim_base = 960
        elif "128d" in case_name: self.dim_base = 128
        
        if "500k" in case_name: self.doc_limit = 500_000
        elif "50k" in case_name: self.doc_limit = 50_000
        elif "5m" in case_name: self.doc_limit = 5_000_000
        elif "10m" in case_name: self.doc_limit = 10_000_000
        elif "100m" in case_name: self.doc_limit = 100_000_000
        elif "1m" in case_name: self.doc_limit = 1_000_000
        elif "capacity" in case_name: self.doc_limit = 1_000_000 # Default for simple capacity check

# --- MODEL WRAPPER ---
# --- HYPERBOLIC GEOMETRY ---
class PoincareBall:
    def __init__(self, c=1.0, clip_r=0.999995):
        self.c = c
        self.clip_r = clip_r
        self.sqrt_c = c ** 0.5

    def _clip_vectors(self, x):
        norm = x.norm(dim=-1, keepdim=True)
        mask = norm > self.clip_r
        scale = self.clip_r / (norm + 1e-15)
        x = torch.where(mask, x * scale, x)
        return x

    def expmap0(self, u):
        u_norm = u.norm(dim=-1, keepdim=True)
        gamma = torch.tanh(self.sqrt_c * u_norm)
        return gamma * (u / (self.sqrt_c * u_norm + 1e-15))

    def logmap0(self, y):
        y = self._clip_vectors(y)
        y_norm = y.norm(dim=-1, keepdim=True)
        arg = torch.clamp(self.sqrt_c * y_norm, max=self.clip_r)
        scale = torch.atanh(arg) / (arg + 1e-15)
        return scale * y

    def dist(self, x, y, keepdim=False):
        x = self._clip_vectors(x)
        y = self._clip_vectors(y)
        sqdist = torch.sum((x - y) ** 2, dim=-1, keepdim=True)
        x_sqnorm = torch.sum(x ** 2, dim=-1, keepdim=True)
        y_sqnorm = torch.sum(y ** 2, dim=-1, keepdim=True)
        denom = (1 - self.c * x_sqnorm) * (1 - self.c * y_sqnorm)
        denom = torch.clamp(denom, min=1e-15)
        arg = 1 + 2 * self.c * sqdist / denom
        arg = torch.clamp(arg, min=1.0 + 1e-7)
        dist = torch.acosh(arg) / self.sqrt_c
        return dist.squeeze(-1) if not keepdim else dist

manifold = PoincareBall()

# --- HYDRA ARCHITECTURE ---
from peft import LoraConfig, TaskType, get_peft_model
import torch.nn as nn
import torch.nn.functional as F

class HydraMRLHead(nn.Module):
    def __init__(self, in_dim, max_out_dim):
        super().__init__()
        self.linear = nn.Linear(in_dim, max_out_dim)
    def forward(self, x, target_dim=None):
        t = self.linear(x)
        if target_dim:
            t = t[:, :target_dim]  # Просто обрезаем размерность
            
            # --- БЫЛО (ОШИБКА) ---
            t = F.normalize(t, p=2, dim=-1)
            # t = t * 0.99995                  <-- УДАЛИТЬ ИЛИ ЗАКОММЕНТИРОВАТЬ
            
            # --- СТАЛО (ПРАВИЛЬНО) ---
            # Мы доверяем модели. Если она обучена под гиперболу, 
            # она сама выдаст нужные нормы.
            # На всякий случай делаем expmap, чтобы загнать в шар (-1, 1)
            return manifold.expmap0(t)
        return t

class HyperQwenHydra(nn.Module):
    def __init__(self, base_name):
        super().__init__()
        self.backbone = AutoModel.from_pretrained(
            base_name, trust_remote_code=True, attn_implementation="eager", dtype=torch.float16
        )
        peft_config = LoraConfig(
            task_type=TaskType.FEATURE_EXTRACTION, r=32, lora_alpha=64, 
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj"]
        )
        self.backbone = get_peft_model(self.backbone, peft_config)
        dim = self.backbone.config.hidden_size
        self.head_reflex = HydraMRLHead(dim, 64)
        self.head_reason = HydraMRLHead(dim, 128)
        
    def forward(self, input_ids, attention_mask, head="reflex", dim=64):
        out = self.backbone(input_ids, attention_mask=attention_mask)
        idx = attention_mask.sum(1) - 1
        vec = out.last_hidden_state[torch.arange(len(input_ids)), idx]
        vec = vec.to(torch.float32)
        if head == "reflex": return self.head_reflex(vec, target_dim=dim)
        if head == "reason": return self.head_reason(vec, target_dim=dim)
        return vec

# --- MODEL WRAPPER ---
def get_device():
    if torch.cuda.is_available(): return "cuda"
    if torch.backends.mps.is_available(): return "mps"
    return "cpu"

class Vectorizer:
    def __init__(self, model_path: str, device=None, is_hyperbolic=False, target_dim=None):
        self.device = device or get_device()
        print(f"Loading {'Hyperbolic' if is_hyperbolic else 'Euclidean'} model from: {model_path} on {self.device}...")
        
        # Try local load first to avoid HF Hub timeouts
        base_name = "Qwen/Qwen3-Embedding-0.6B"
        path = base_name if is_hyperbolic else model_path
        try:
            self.tokenizer = AutoTokenizer.from_pretrained(path, trust_remote_code=True, local_files_only=True)
        except Exception:
            self.tokenizer = AutoTokenizer.from_pretrained(path, trust_remote_code=True)
            
        self.is_hyperbolic = is_hyperbolic
        self.target_dim = target_dim or 64

        if is_hyperbolic:
            self.model = HyperQwenHydra(base_name)
            if model_path.endswith(".pth"):
                # Handle relative paths from script directory
                if not os.path.isabs(model_path):
                    model_path = os.path.abspath(os.path.join(os.path.dirname(__file__), model_path))
                
                if os.path.exists(model_path):
                    print(f"   Loading state dict: {model_path}")
                    self.model.load_state_dict(torch.load(model_path, map_location="cpu"), strict=False)
                else:
                    raise FileNotFoundError(f"Weights file not found: {model_path}")
            self.model.to(self.device).eval()
        else:
            try:
                self.model = AutoModel.from_pretrained(model_path, trust_remote_code=True, dtype=torch.float16, local_files_only=True).to(self.device).eval()
            except Exception:
                self.model = AutoModel.from_pretrained(model_path, trust_remote_code=True, dtype=torch.float16).to(self.device).eval()


    def encode(self, texts: List[str], batch_size: int = 32) -> np.ndarray:
        self.model.eval()
        all_vecs = []
        for i in tqdm(range(0, len(texts), batch_size), desc="Encoding"):
            batch = texts[i : i + batch_size]
            inputs = self.tokenizer(batch, padding=True, truncation=True, max_length=512, return_tensors="pt")
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            with torch.no_grad():
                if self.is_hyperbolic:
                    head = "reflex" if self.target_dim <= 64 else "reason"
                    embeddings = self.model(inputs['input_ids'], inputs['attention_mask'], head=head, dim=self.target_dim)
                else:
                    outputs = self.model(**inputs)
                    attention_mask = inputs['attention_mask']
                    last_hidden = outputs.last_hidden_state
                    input_mask_expanded = attention_mask.unsqueeze(-1).expand(last_hidden.size()).float()
                    sum_embeddings = torch.sum(last_hidden * input_mask_expanded, 1)
                    sum_mask = torch.clamp(input_mask_expanded.sum(1), min=1e-9)
                    embeddings = sum_embeddings / sum_mask
                    
                    if self.target_dim and embeddings.shape[1] > self.target_dim:
                        embeddings = embeddings[:, :self.target_dim]
                    embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
                
            all_vecs.append(embeddings.cpu().numpy())
            
        return np.concatenate(all_vecs, axis=0).astype(np.float32)


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
    recall_sys: float
    mrr: float
    ndcg: float
    c1_qps: float
    c10_qps: float
    c30_qps: float
    disk_usage: str
    status: str

def format_size(size_str: str) -> str:
    """Unify size strings to a readable format (e.g. 1.2G or 450M)"""
    if not size_str or size_str == "N/A" or size_str == "Err":
        return size_str
    
    # Try to extract numeric value and unit
    try:
        val_str = size_str.upper()
        if val_str.endswith('B'):
            bytes_val = float(val_str[:-1])
        elif val_str.endswith('K'):
            bytes_val = float(val_str[:-1]) * 1024
        elif val_str.endswith('M'):
            bytes_val = float(val_str[:-1]) * 1024**2
        elif val_str.endswith('G'):
            bytes_val = float(val_str[:-1]) * 1024**3
        elif val_str.endswith('T'):
            bytes_val = float(val_str[:-1]) * 1024**4
        else:
            # Assume raw bytes
            bytes_val = float(val_str)
            
        if bytes_val < 1024: return f"{bytes_val:.0f}B"
        elif bytes_val < 1024**2: return f"{bytes_val/1024:.1f}K"
        elif bytes_val < 1024**3: return f"{bytes_val/(1024**2):.1f}M"
        else: return f"{bytes_val/(1024**3):.2f}G"
    except:
        return size_str

def get_docker_disk(container_keyword: str) -> str:
    try:
        ps = subprocess.run(["docker", "ps", "--format", "{{.Names}}"], capture_output=True, text=True)
        containers = ps.stdout.strip().split('\n')
        target = next((c for c in containers if container_keyword in c), None)
        if not target: return "N/A"
        
        # Determine internal path based on DB type
        if "milvus" in container_keyword:
             path = "/var/lib/milvus"
        elif "qdrant" in container_keyword:
             path = "/qdrant/storage"
        elif "chroma" in container_keyword:
             # Chroma uses /chroma/data by default in newer versions
             path = "/chroma/data"
        else:
             path = "/data"
             
        res = subprocess.run(["docker", "exec", target, "du", "-sh", path], capture_output=True, text=True)
        if res.returncode != 0:
            # Fallback to /data
            res = subprocess.run(["docker", "exec", target, "du", "-sh", "/data"], capture_output=True, text=True)
            
        return res.stdout.split()[0] if res.returncode == 0 else "Err"
    except: return "N/A"

def get_local_disk(path: str) -> str:
    try:
        if not os.path.isabs(path):
            path = os.path.abspath(os.path.join(os.path.dirname(__file__), path))
        total = sum(os.path.getsize(os.path.join(dp, f)) for dp, _, fn in os.walk(path) for f in fn if not os.path.islink(os.path.join(dp, f)))
        return format_size(str(total))
    except: return "N/A"

def get_hyperspace_disk_api(host="localhost"):
    """Try to get disk usage via Hyperspace Metrics API"""
    import requests
    for port in [50050, 50051, 50052, 50053]:
        try:
            url = f"http://{host}:{port}/api/metrics"
            headers = {"x-api-key": "I_LOVE_HYPERSPACEDB"}
            res = requests.get(url, headers=headers, timeout=2).json()
            if 'disk_usage_mb' in res:
                return f"{res['disk_usage_mb']}M"
        except:
            continue
    return None

def run_concurrency_profile(query_fn, workers_list=(1, 10, 30), queries=500):
    result = {}
    for workers in workers_list:
        start = time.time()
        with ThreadPoolExecutor(max_workers=workers) as ex:
            list(ex.map(lambda _: query_fn(), range(queries)))
        elapsed = time.time() - start
        qps = queries / elapsed if elapsed > 0 else 0.0
        result[workers] = qps
    return result

def calculate_accuracy(results: List[List[str]], ground_truth: List[List[str]], k: int) -> Tuple[float, float, float]:
    """Calculates Recall@K, MRR and NDCG@K for semantic search"""
    if not results or not ground_truth:
        return 0.0, 0.0, 0.0
        
    print(f"   [Debug] Accuracy calculation: {len(results)} results, {len(ground_truth)} GT entries.")
    if len(results) > 0 and len(results[0]) > 0:
        print(f"   [Debug] Sample Result: {results[0]}")
    if len(ground_truth) > 0 and len(ground_truth[0]) > 0:
        print(f"   [Debug] Sample GT: {ground_truth[0]}")
    
    recalls = []
    mrrs = []
    ndcgs = []
    for res, gt_set_list in zip(results, ground_truth):
        gt_set = set(gt_set_list)
        if not gt_set:
            recalls.append(0.0)
            mrrs.append(0.0)
            ndcgs.append(0.0)
            continue
            
        # Recall@K - how many of our top-K are relevant
        intersection = set(res[:k]) & gt_set
        recall = len(intersection) / min(k, len(gt_set))
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

    # Use np.mean for safety against empty lists and more robust statistical handling
    return (
        float(np.mean(recalls)) if recalls else 0.0,
        float(np.mean(mrrs)) if mrrs else 0.0,
        float(np.mean(ndcgs)) if ndcgs else 0.0
    )

def calculate_system_recall(results: List[List[str]], exact_ground_truth: List[List[str]], k: int) -> float:
    """Calculates System Recall@K against exact brute-force nearest neighbors."""
    if not results or not exact_ground_truth:
        return 0.0
    recalls = []
    for res, gt in zip(results, exact_ground_truth):
        gt_set = set(gt[:k])
        found = set(res[:k]) & gt_set
        recall = len(found) / len(gt_set) if gt_set else 0.0
        recalls.append(recall)
    return statistics.mean(recalls) if recalls else 0.0

def calculate_brute_force_gt(query_vecs: np.ndarray, doc_vecs: np.ndarray, doc_ids: List[str], k: int, metric: str) -> List[List[str]]:
    """Builds exact top-K neighbors in-memory for ANN quality evaluation."""
    if query_vecs is None or doc_vecs is None:
        return []

    gt = []
    metric_l = metric.lower()

    if metric_l == "cosine":
        doc_norms = np.linalg.norm(doc_vecs, axis=1, keepdims=True)
        doc_normed = doc_vecs / (doc_norms + 1e-12)
        for q in tqdm(query_vecs, desc="Brute-force GT (cosine)"):
            qn = q / (np.linalg.norm(q) + 1e-12)
            sims = doc_normed @ qn
            top_idx = np.argpartition(-sims, k - 1)[:k]
            top_idx = top_idx[np.argsort(-sims[top_idx])]
            gt.append([doc_ids[idx] for idx in top_idx])
        return gt

    if metric_l in ("poincare", "hyperbolic"):
        doc_norms_sq = np.sum(doc_vecs**2, axis=1)
        for q in tqdm(query_vecs, desc="Brute-force GT (poincare)"):
            q_norm_sq = np.sum(q**2)
            diff_sq = np.sum((doc_vecs - q) ** 2, axis=1)
            dists = diff_sq / ((1 - q_norm_sq) * (1 - doc_norms_sq) + 1e-15)
            top_idx = np.argpartition(dists, k - 1)[:k]
            top_idx = top_idx[np.argsort(dists[top_idx])]
            gt.append([doc_ids[idx] for idx in top_idx])
        return gt

    for q in tqdm(query_vecs, desc="Brute-force GT (l2)"):
        dists = np.sum((doc_vecs - q) ** 2, axis=1)
        top_idx = np.argpartition(dists, k - 1)[:k]
        top_idx = top_idx[np.argsort(dists[top_idx])]
        gt.append([doc_ids[idx] for idx in top_idx])
    return gt

def print_table(results: List[Result]):
    # Sort by Insert QPS
    results.sort(key=lambda x: x.insert_qps, reverse=True)
    header = f"{'Database':<15} | {'Dim':<5} | {'Metric':<8} | {'Ins QPS':<10} | {'Srch QPS':<10} | {'P99 Lat':<10} | {'Recall(Sem)':<11} | {'Recall(Sys)':<11} | {'MRR':<5} | {'NDCG':<5} | {'C1':<8} | {'C10':<8} | {'C30':<8} | {'Disk':<8} | {'Status'}"
    print("\n" + "="*len(header))
    print(header)
    print("-" * len(header))
    for r in results:
        if r.status == "Success":
            print(f"{r.database:<15} | {r.dimension:<5} | {r.metric:<8} | {r.insert_qps:10,.0f} | {r.search_qps:10,.0f} | {r.p99:8.2f} ms | {r.recall:10.1%} | {r.recall_sys:10.1%} | {r.mrr:4.2f} | {r.ndcg:4.2f} | {r.c1_qps:8,.0f} | {r.c10_qps:8,.0f} | {r.c30_qps:8,.0f} | {r.disk_usage:8} | {r.status}")
        else:
            print(f"{r.database:<15} | {r.dimension:<5} | ERROR: {r.status}")
    print("=" * len(header) + "\n")

def detect_hyperspace_metric(host="localhost"):
    """Detect if HyperspaceDB is in poincare or cosine mode via API"""
    import requests
    # Check common dashboard/api ports
    for port in [50050, 50051, 50052, 50053]:
        try:
            url = f"http://{host}:{port}/api/status"
            res = requests.get(url, timeout=3).json()
            # Try to find metric in config
            if 'config' in res and 'metric' in res['config']:
                return res['config']['metric'].lower()
            if 'metric' in res:
                return res['metric'].lower()
        except:
            continue
    return None

def extract_ids(res_obj):
    """Normalize search responses across SDKs"""
    ids = []
    for hit in res_obj:
        if isinstance(hit, dict):
            # Hyperspace SDK returns dicts with metadata
            if "metadata" in hit and "doc_id" in hit["metadata"]:
                ids.append(hit["metadata"]["doc_id"])
            elif "id" in hit:
                ids.append(hit["id"])
        elif hasattr(hit, "id"):
            ids.append(hit.id)
        elif isinstance(hit, str):
            ids.append(hit)
    return ids

def wait_for_indexing(host="localhost", port=50050, collection="bench_semantic", timeout=None):
    """Wait for HyperspaceDB background indexing to complete with progress display"""
    import requests
    print(f"⏳ Monitoring indexing for '{collection}' (waiting for queue to clear)...")
    url = f"http://{host}:{port}/api/collections/{collection}/stats"
    
    # Ключ по умолчанию для HyperspaceDB
    headers = {"x-api-key": "I_LOVE_HYPERSPACEDB"}
    
    start_time = time.time()
    
    while True:
        if timeout and time.time() - start_time > timeout:
            print(f"\n⚠️ Timeout after {timeout}s. Proceeding with partial index...")
            break
            
        try:
            # Передаем ключ в заголовках
            response = requests.get(url, headers=headers, timeout=5)
            
            if response.status_code == 200:
                data = response.json()
                # Server might use 'count' or 'vector_count'
                count = data.get("count", data.get("vector_count", 0))
                queue = data.get("indexing_queue", 0)
                
                elapsed = time.time() - start_time
                print(f"\r   [Indexing] Remaining: {queue:,} | Total Indexed: {count:,} | Elapsed: {elapsed:.0f}s          ", end="", flush=True)
                
                if "indexing_queue" not in data:
                     # If we don't have queue info, just wait a bit if count > 0
                     if count > 0:
                         print("\n✅ Count > 0. Stability wait...")
                         time.sleep(5)
                         break
                elif queue == 0 and count > 0:
                    print(f"\n✅ Indexing complete! Docs ready: {count:,}")
                    break
            elif response.status_code == 401:
                print(f"\r   🚫 Auth Error (401): Check HYPERSPACE_API_KEY in .env          ", end="", flush=True)
                time.sleep(5) 
                continue
            elif response.status_code == 404:
                print(f"\r   ⌛ Collection '{collection}' initializing...", end="", flush=True)
            else:
                print(f"\r   ⚠️ Server returned status {response.status_code}            ", end="", flush=True)
                
            time.sleep(2)
        except Exception as e:
            print(f"\r   ⏳ Connection issue (retrying... {str(e)[:40]})          ", end="", flush=True)
            time.sleep(2)

def load_data_smart(cfg: Config) -> Tuple[List[str], List[str], List[str], List[str], Dict[str, List[str]]]:
    """Helper to load Heavy Datasets like MS MARCO using streaming to avoid OOM."""
    print(f"\n📚 Loading dataset: {cfg.dataset_name}...")
    
    # MS MARCO is special (huge)
    if "msmarco" in cfg.dataset_name.lower():
        try:
            # Load streaming to avoid RAM explosion
            print(f"   [Mode] Streaming enabled for {cfg.dataset_name}")
            corpus_data = load_dataset(cfg.dataset_name, "corpus", split="corpus", streaming=True)
            queries_data = load_dataset(cfg.dataset_name, "queries", split="queries", streaming=True)
            # Qrels for dev set (usually small enough to fit in RAM)
            qrels_data = load_dataset(cfg.dataset_name, "default", split=cfg.split)
        except Exception as e:
            print(f"❌ Failed to load MS MARCO: {e}")
            return [], [], [], [], {}

        docs = []
        doc_ids = []
        print(f"   Streaming {cfg.doc_limit:,} docs from MS MARCO...")
        
        # Iterate stream
        for i, row in tqdm(enumerate(corpus_data), total=cfg.doc_limit, desc="Streaming Corpus"):
            if i >= cfg.doc_limit: break
            # Combine title + text for better context
            text = (row.get('title', '') + " " + row.get('text', '')).strip()
            if len(text) > 10: 
                docs.append(text)
                doc_ids.append(row['_id'])
        
        # Prepare Qrels
        valid_qrels = {}
        doc_id_set = set(doc_ids)
        
        print(f"   Filtering qrels against {len(doc_id_set):,} loaded docs...")
        relevant_query_ids = set()
        for row in qrels_data:
            qid = row['query-id']
            did = row['corpus-id']
            if did in doc_id_set:
                if qid not in valid_qrels: valid_qrels[qid] = []
                valid_qrels[qid].append(did)
                relevant_query_ids.add(qid)
        
        # Load Query Text
        test_queries = []
        test_query_ids = []
        
        print(f"   Collecting {cfg.query_limit} relevant queries...")
        count = 0
        for row in queries_data:
            if count >= cfg.query_limit: break
            qid = row['_id']
            if qid in relevant_query_ids:
                test_queries.append(row['text'])
                test_query_ids.append(qid)
                count += 1
                
        return docs, doc_ids, test_queries, test_query_ids, valid_qrels

    else:
        # Standard MTEB loader (NFCorpus, SciFact, etc.)
        try:
            corpus = load_dataset(cfg.dataset_name, "corpus", split="corpus")
            queries = load_dataset(cfg.dataset_name, "queries", split="queries")
            qrels = load_dataset(cfg.dataset_name, "default", split=cfg.split)
        except Exception as e:
            print(f"❌ Error loading dataset {cfg.dataset_name}: {e}")
            return [], [], [], [], {}

        docs = []
        doc_ids = []
        for row in corpus:
            if len(docs) >= cfg.doc_limit: break
            text = (row.get('title', '') + " " + row.get('text', '')).strip()
            docs.append(text)
            doc_ids.append(row['_id'])
            
        valid_qrels = {}
        doc_id_set = set(doc_ids)
        for row in qrels:
            qid = row['query-id']
            did = row['corpus-id']
            if did in doc_id_set:
                if qid not in valid_qrels: valid_qrels[qid] = []
                valid_qrels[qid].append(did)
                
        query_map = {row['_id']: row['text'] for row in queries}
        test_queries = []
        test_query_ids = []
        for qid in list(valid_qrels.keys())[:cfg.query_limit]:
            if qid in query_map:
                test_queries.append(query_map[qid])
                test_query_ids.append(qid)
            
        return docs, doc_ids, test_queries, test_query_ids, valid_qrels

# --- BENCHMARK ---
def run_benchmark():
    cfg = Config()
    final_results = []

    # 1. Load Data
    # Detect Case from CLI
    case_arg = next((arg for arg in sys.argv if arg.startswith("--case=")), None)
    if case_arg:
        cfg.target_case = case_arg.split("=", 1)[1]
        cfg.apply_case(cfg.target_case)
        print(f"🚀 Running Case: {cfg.target_case} (Dim: {cfg.dim_base}, Count: {cfg.doc_limit:,})")

    # 1. Load Data
    docs = []
    doc_ids = []
    test_queries = []
    test_query_ids = []
    
    # Placeholders for embeddings
    doc_vecs_euc = None
    q_vecs_euc = None
    doc_vecs_hyp = None
    q_vecs_hyp = None
    
    math_gt_euc = []
    math_gt_hyp = []
    valid_qrels = {}

    if cfg.target_case:
        if not VB_AVAILABLE:
            print("❌ Error: 'vectordb_bench' library is required for running specific cases with REAL data.")
            print("   Please install it: pip install vectordb-bench")
            return

        print(f"\n📚 Loading REAL dataset for Case: {cfg.target_case}...")
        
        case_map = {
            "Performance1536D50K": Performance1536D50K,
            "Performance1536D500K": Performance1536D500K,
            "Performance1536D500K1P": Performance1536D500K1P,
            "Performance1536D500K99P": Performance1536D500K99P,
            "Performance1536D5M": Performance1536D5M,
            "Performance1536D5M1P": Performance1536D5M1P,
            "Performance1536D5M99P": Performance1536D5M99P,
            "Performance768D1M": Performance768D1M,
            "Performance768D1M1P": Performance768D1M1P,
            "Performance768D1M99P": Performance768D1M99P,
            "Performance768D10M": Performance768D10M,
            "Performance768D10M1P": Performance768D10M1P,
            "Performance768D10M99P": Performance768D10M99P,
            "Performance768D100M": Performance768D100M,
            "Performance1024D1M": Performance1024D1M,
            "Performance1024D10M": Performance1024D10M,
            "CapacityDim128": CapacityDim128,
            "CapacityDim960": CapacityDim960,
            "CapacityCase": CapacityCase,
            "PerformanceCase": PerformanceCase,
            "PerformanceCustomDataset": PerformanceCustomDataset,
        }
        
        if cfg.target_case not in case_map:
            print(f"❌ Unknown case: {cfg.target_case}. Available: {list(case_map.keys())}")
            return
            
        case_cls = case_map[cfg.target_case]
        case_inst = case_cls()
        ds = case_inst.dataset
        print(f"   Downloading/Verifying {ds.data.name} (Size: {ds.data.size}, Dim: {ds.data.dim})...")
        
        # Download if needed
        try:
            ds.prepare(source=DatasetSource.S3)
        except Exception as e:
            print(f"   ⚠️ Download failed: {e}. Checking local cache...")
        
        # Load Data
        data_dir = pathlib.Path(ds.data_dir)
        print(f"   Loading from: {data_dir}")
        
        if not data_dir.exists():
            print(f"❌ Data directory not found: {data_dir}")
            return

        # Train (Docs)
        train_path = data_dir / "train.parquet"
        if not train_path.exists():
            train_path = data_dir / "shuffle_train.parquet"

        if train_path.exists():
            import pandas as pd
            print(f"   Reading Train vectors from {train_path.name}...")
            df_train = pd.read_parquet(train_path)
            
            if "emb" in df_train.columns:
                 doc_vecs_euc = np.stack(df_train["emb"].values).astype(np.float32)
            elif "vector" in df_train.columns:
                 doc_vecs_euc = np.stack(df_train["vector"].values).astype(np.float32)
            else:
                 # Try 2nd column (usually 1st is ID, 2nd is vector)
                doc_vecs_euc = np.stack(df_train.iloc[:, 1].values).astype(np.float32)

            limit = cfg.doc_limit if cfg.doc_limit > 0 else 100_000_000
            if len(doc_vecs_euc) > limit:
                doc_vecs_euc = doc_vecs_euc[:limit]
            
            docs = [f"Real Doc {i}" for i in range(len(doc_vecs_euc))]
            doc_ids = [str(i) for i in range(len(doc_vecs_euc))]
        else:
            print(f"❌ Train data file not found: {train_path}")
            return
            
        # Test (Queries)
        test_path = data_dir / "test.parquet"
        if test_path.exists():
            print("   Reading Test queries...")
            df_test = pd.read_parquet(test_path)
            if "emb" in df_test.columns:
                 q_vecs_euc = np.stack(df_test["emb"].values).astype(np.float32)
            elif "vector" in df_test.columns:
                 q_vecs_euc = np.stack(df_test["vector"].values).astype(np.float32)
            else:
                 q_vecs_euc = np.stack(df_test.iloc[:, 1].values).astype(np.float32)
        elif doc_vecs_euc is not None:
            print("   ⚠️ Test queries not found. Using first 100 train docs as queries...")
            q_vecs_euc = doc_vecs_euc[:100]
            test_queries = [f"Query {i}" for i in range(len(q_vecs_euc))]
            test_query_ids = [str(i) for i in range(len(q_vecs_euc))]
            # Identity mapping for fallback
            for i in range(len(q_vecs_euc)):
                valid_qrels[str(i)] = [str(i)]
        else:
            print(f"❌ Test queries file not found and no train data: {test_path}")
            return
            
        if q_vecs_euc is not None:
            # Limit queries
            if len(q_vecs_euc) > cfg.query_limit:
                 q_vecs_euc = q_vecs_euc[:cfg.query_limit]
                 
            test_queries = [f"Query {i}" for i in range(len(q_vecs_euc))]
            test_query_ids = [str(i) for i in range(len(q_vecs_euc))]
            
        # Neighbors (Ground Truth)
        neighbors_path = data_dir / "neighbors.parquet"
        if neighbors_path.exists():
             print("   Reading Ground Truth...")
             df_gt = pd.read_parquet(neighbors_path)
             # Support both 'neighbors' and 'labels' column names
             col = "neighbors" if "neighbors" in df_gt.columns else ("labels" if "labels" in df_gt.columns else None)
             if col:
                 # Take first K
                 math_gt_euc = [list(x)[:10] for x in df_gt[col].values[:len(q_vecs_euc)]]
                 # Ensure strings if we use string IDs
                 # Mapping int index to string ID
                 math_gt_euc = [[str(idx) for idx in row] for row in math_gt_euc]
                 
                 # Populate valid_qrels for semantic recall
                 for i, row in enumerate(math_gt_euc):
                     if i < len(test_query_ids):
                         qid = test_query_ids[i]
                         valid_qrels[qid] = row
                 print(f"   Populated valid_qrels with {len(valid_qrels)} items from {col}.")
             else:
                 print(f"   ⚠️ Could not find neighbors/labels column in GT. Columns: {df_gt.columns.tolist()}")

        # Override cfg dimensions
        if doc_vecs_euc is not None:
            if len(doc_vecs_euc.shape) > 1:
                cfg.dim_base = doc_vecs_euc.shape[1]
            else:
                cfg.dim_base = len(doc_vecs_euc[0])
            print(f"   Loaded: {len(doc_vecs_euc):,} docs, {len(q_vecs_euc)} queries. Dim: {cfg.dim_base}")
        else:
             print("❌ Failed to load document vectors.")
             return

    else:
        # Standard MTEB Logic / Smart Streaming Loader
        docs, doc_ids, test_queries, test_query_ids, valid_qrels = load_data_smart(cfg)
        
        if not docs:
            print("❌ Data loading failed. Check dataset name or connection.")
            return

    print(f"✅ Data prepared: {len(docs)} docs, {len(test_queries)} queries with ground truth.")

    # Parse args ignoring flags like --case
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    target_db = args[0].lower() if args else None
    ds_slug = cfg.dataset_name.replace("/", "_")

    # ==========================================
    # DATA VECTORIZATION (Standalone)
    # ==========================================

    # Euclidean (1024d)
    need_euc = (not target_db) or \
               any(db in target_db for db in ["milvus", "qdrant", "chroma"]) or \
               ("hyper" in (target_db or "") and cfg.HYPER_MODE == "cosine")
               
    if need_euc:
        if cfg.target_case and doc_vecs_euc is not None:
            print(f"   Used loaded real data. Dim: {cfg.dim_base}")
        else:
            cache_file = f"cache_{ds_slug}_euclidean_1024d_{cfg.doc_limit}.npz"
            if os.path.exists(cache_file):
                print(f"   Loading cached Euclidean embeddings...")
                data = np.load(cache_file)
                doc_vecs_euc = data['embeddings']
            else:
                print("   Vectorizing Euclidean baseline...")
                model_base = Vectorizer(cfg.model_path_base, is_hyperbolic=False, target_dim=cfg.dim_base)
                doc_vecs_euc = model_base.encode(docs, batch_size=cfg.batch_size)
                np.savez_compressed(cache_file, embeddings=doc_vecs_euc, doc_ids=np.array(doc_ids))
                del model_base

            q_cache_file = f"cache_{ds_slug}_euclidean_queries_{cfg.query_limit}.npy"
            if os.path.exists(q_cache_file):
                q_vecs_euc = np.load(q_cache_file)
            else:
                model_base = Vectorizer(cfg.model_path_base, is_hyperbolic=False, target_dim=cfg.dim_base)
                q_vecs_euc = model_base.encode(test_queries, batch_size=cfg.batch_size)
                np.save(q_cache_file, q_vecs_euc)
                del model_base

    # Hyperbolic (64d)
    need_hyp = (not target_db or "hyper" in target_db) and cfg.HYPER_MODE == "poincare"
    if need_hyp:
        cache_file = f"cache_{ds_slug}_hyperbolic_64d_{cfg.doc_limit}.npz"
        if os.path.exists(cache_file):
            print(f"   Loading cached Hyperbolic embeddings...")
            data = np.load(cache_file)
            doc_vecs_hyp = data['embeddings']
        else:
            print("   Vectorizing Hyperbolic embeddings...")
            model_hyp = Vectorizer(cfg.model_path_hyp, is_hyperbolic=True, target_dim=cfg.dim_hyp)
            doc_vecs_hyp = model_hyp.encode(docs, batch_size=cfg.batch_size)
            np.savez_compressed(cache_file, embeddings=doc_vecs_hyp, doc_ids=np.array(doc_ids))
            del model_hyp

        q_cache_file = f"cache_{ds_slug}_hyperbolic_queries_{cfg.query_limit}.npy"
        if os.path.exists(q_cache_file):
            q_vecs_hyp = np.load(q_cache_file)
        else:
            model_hyp = Vectorizer(cfg.model_path_hyp, is_hyperbolic=True, target_dim=cfg.dim_hyp)
            q_vecs_hyp = model_hyp.encode(test_queries, batch_size=cfg.batch_size)
            np.save(q_cache_file, q_vecs_hyp)
            del model_hyp

    # Build exact nearest-neighbor ground truth for ANN quality (System Recall)
    # Build exact nearest-neighbor ground truth for ANN quality (System Recall)
    if need_euc and doc_vecs_euc is not None and q_vecs_euc is not None and not math_gt_euc:
        print("\n🧮 Building exact System GT for Euclidean/Cosine vectors...")
        math_gt_euc = calculate_brute_force_gt(q_vecs_euc, doc_vecs_euc, doc_ids, k=10, metric="cosine")
        
        # FIX for missing semantic GT (Recalls = 0.0)
        # If we don't have valid_qrels (e.g. synthetic dataset), use System GT as Semantic GT
        if not valid_qrels:
            print("   ⚠️ No semantic ground truth found. Using System GT as proxy for Recall/MRR.")
            for i, q_id in enumerate(test_query_ids):
                # Use top-10 system results as "relevant" docs
                if i < len(math_gt_euc):
                    valid_qrels[q_id] = math_gt_euc[i]

    if need_hyp and doc_vecs_hyp is not None and q_vecs_hyp is not None and not math_gt_hyp:
        print("\n🧮 Building exact System GT for Poincare vectors...")
        math_gt_hyp = calculate_brute_force_gt(q_vecs_hyp, doc_vecs_hyp, doc_ids, k=10, metric="poincare")

    # ==========================================
    # PHASE 1: Baseline (Milvus + 1024d)
    # ==========================================
    if MILVUS_AVAILABLE and (not target_db or "milvus" in target_db):
        print("\n🔵 PHASE 1: Milvus (1024d Euclidean)")

        try:
            # Connect & Setup
            connections.connect(host="localhost", port="19530")
            if utility.has_collection("bench_semantic"): utility.drop_collection("bench_semantic")
            
            schema = CollectionSchema([
                FieldSchema("id", DataType.INT64, is_primary=True, auto_id=True),
                FieldSchema("doc_id", DataType.VARCHAR, max_length=128),
                FieldSchema("vec", DataType.FLOAT_VECTOR, dim=cfg.dim_base)
            ], "")
            col = Collection("bench_semantic", schema)
            
            # Insert
            print("   Inserting into Milvus...")
            t0 = time.time()
            m_batch_size = max(10, int(3_000_000 / (cfg.dim_base * 8)))
            for i in tqdm(range(0, len(doc_vecs_euc), m_batch_size)): # Larger batches for insertion
                batch_vecs = doc_vecs_euc[i : i + m_batch_size]
                batch_ids = doc_ids[i : i + m_batch_size]
                col.insert([batch_ids, batch_vecs.tolist()])
            v_dur = time.time() - t0
            
            # Ensure data is persisted and indexed properly
            print("   Flushing and Indexing Milvus...")
            col.flush()
            col.create_index("vec", {"metric_type":"COSINE", "index_type":"IVF_FLAT", "params":{"nlist":128}})
            col.load()
            time.sleep(5) 
            
            # Search & Eval
            all_res_ids = []
            all_gt_ids = []
            lats = []
            
            print("   Searching Milvus...")
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(q_vecs_euc)):
                q_id = test_query_ids[i]
                all_gt_ids.append(valid_qrels.get(q_id, []))
                
                ts = time.time()
                res = col.search(
                    [q_vec.tolist()], 
                    "vec", 
                    {"metric_type":"COSINE", "params":{"nprobe":10}}, 
                    limit=10,
                    output_fields=["doc_id"]
                )
                lats.append((time.time() - ts) * 1000)
                
                # Extract IDs
                all_res_ids.append([hit.entity.get("doc_id") for hit in res[0]])
            
            search_dur = time.time() - search_t0
            recall, mrr, ndcg = calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = calculate_system_recall(all_res_ids, math_gt_euc, 10)
            
            # Concurrency Profile
            print("   Testing Milvus Concurrency...")
            q_list = q_vecs_euc[0].tolist()
            def milvus_query():
                col.search([q_list], "vec", {"metric_type":"COSINE", "params":{"nprobe":10}}, limit=10)
            conc = run_concurrency_profile(milvus_query)
            
            disk = format_size(get_docker_disk("milvus"))
            
            final_results.append(Result(
                database="Milvus",
                dimension=cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(docs) / v_dur,
                search_qps=len(test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            ))
            
            # Cleanup
            utility.drop_collection("bench_semantic")
        except Exception as e:
            print(f"   ⚠️ Milvus error: {e}")
            final_results.append(Result("Milvus", cfg.dim_base, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}"))

    # ==========================================
    # PHASE 1.1: Qdrant (1024d Euclidean)
    # ==========================================
    if QDRANT_AVAILABLE and (not target_db or "qdrant" in target_db):
        print("\n🔷 PHASE 1.1: Qdrant (1024d Euclidean)")
        try:
            client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)
            name = "bench_semantic"
            try: client.delete_collection(name)
            except Exception: pass # Silencing gRPC errors
            
            client.create_collection(
                name,
                vectors_config=VectorParams(size=cfg.dim_base, distance=Distance.COSINE)
            )
            
            print("   Inserting into Qdrant...")
            t0 = time.time()
            q_batch_size = max(10, int(3_000_000 / (cfg.dim_base * 8)))
            for i in tqdm(range(0, len(doc_vecs_euc), q_batch_size)):
                batch_vecs = doc_vecs_euc[i : i + q_batch_size]
                batch_ids = doc_ids[i : i + q_batch_size]
                points = [
                    PointStruct(id=i+j, vector=v.tolist(), payload={"doc_id": batch_ids[j]})
                    for j, v in enumerate(batch_vecs)
                ]
                client.upsert(collection_name=name, points=points, wait=True)
            v_dur = time.time() - t0
            
            print("   Waiting for Qdrant to settle indexing...")
            time.sleep(5)
            
            # Search
            all_res_ids = []
            all_gt_ids = []
            lats = []
            
            print("   Searching Qdrant...")
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(q_vecs_euc)):
                q_id = test_query_ids[i]
                all_gt_ids.append(valid_qrels.get(q_id, []))
                
                ts = time.time()
                res = client.query_points(collection_name=name, query=q_vec.tolist(), limit=10)
                lats.append((time.time() - ts) * 1000)
                
                all_res_ids.append([hit.payload.get("doc_id") for hit in res.points])
            
            search_dur = time.time() - search_t0
            recall, mrr, ndcg = calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = calculate_system_recall(all_res_ids, math_gt_euc, 10)
            
            # Concurrency
            print("   Testing Qdrant Concurrency...")
            q_list = q_vecs_euc[0].tolist()
            def qdrant_query():
                client.query_points(collection_name=name, query=q_list, limit=10)
            conc = run_concurrency_profile(qdrant_query)
            
            disk = format_size(get_docker_disk("qdrant"))
            
            final_results.append(Result(
                database="Qdrant",
                dimension=cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(docs) / v_dur,
                search_qps=len(test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            ))
            client.delete_collection(name)
        except Exception as e:
            print(f"   ⚠️ Qdrant error: {e}")
            final_results.append(Result("Qdrant", cfg.dim_base, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}"))

    # ==========================================
    # PHASE 1.2: ChromaDB (1024d Euclidean)
    # ==========================================
    if CHROMA_AVAILABLE and (not target_db or "chroma" in target_db):
        print("\n🟡 PHASE 1.2: ChromaDB (1024d Euclidean)")
        try:
            name = "bench_semantic"
            client = None
            col = None

            if hasattr(chromadb, "HttpClient"):
                try:
                    client = chromadb.HttpClient(host="localhost", port=8000)
                    try:
                        client.delete_collection(name)
                    except Exception:
                        pass
                    col = client.create_collection(name, metadata={"hnsw:space": "cosine"})
                except Exception:
                    client = None
                    col = None

            # Fallback to local persistent storage when remote REST API is unavailable.
            if col is None:
                if hasattr(chromadb, "PersistentClient"):
                    chroma_local_dir = os.path.join(os.path.dirname(__file__), ".chroma_bench_data")
                    client = chromadb.PersistentClient(path=chroma_local_dir)
                else:
                    from chromadb.config import Settings
                    client = chromadb.Client(Settings(
                        chroma_api_impl="rest",
                        chroma_server_host="localhost",
                        chroma_server_http_port="8000"
                    ))
                try:
                    client.delete_collection(name)
                except Exception:
                    pass
                col = client.create_collection(name, metadata={"hnsw:space": "cosine"})
            
            print("   Inserting into Chroma...")
            t0 = time.time()
            c_batch_size = max(10, int(3_000_000 / (cfg.dim_base * 8)))
            for i in tqdm(range(0, len(doc_vecs_euc), c_batch_size)):
                batch_vecs = doc_vecs_euc[i : i + c_batch_size]
                batch_ids = doc_ids[i : i + c_batch_size]
                col.add(ids=batch_ids, embeddings=batch_vecs.tolist())
            v_dur = time.time() - t0
            
            # Search
            all_res_ids = []
            all_gt_ids = []
            lats = []

            print("   Searching Chroma...")
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(q_vecs_euc)):
                q_id = test_query_ids[i]
                all_gt_ids.append(valid_qrels.get(q_id, []))
                
                ts = time.time()
                res = col.query(query_embeddings=[q_vec.tolist()], n_results=10)
                lats.append((time.time() - ts) * 1000)
                
                all_res_ids.append(res['ids'][0])
            
            search_dur = time.time() - search_t0
            recall, mrr, ndcg = calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = calculate_system_recall(all_res_ids, math_gt_euc, 10)
            
            # Concurrency
            print("   Testing Chroma Concurrency...")
            q_list = q_vecs_euc[0].tolist()
            def chroma_query():
                col.query(query_embeddings=[q_list], n_results=10)
            conc = run_concurrency_profile(chroma_query)
            
            disk = format_size(get_docker_disk("chroma"))
            
            final_results.append(Result(
                database="ChromaDB",
                dimension=cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(docs) / v_dur,
                search_qps=len(test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            ))
            client.delete_collection(name)
        except Exception as e:
            import traceback
            print(f"   ⚠️ Chroma error: {e}")
            traceback.print_exc()
            final_results.append(Result("ChromaDB", cfg.dim_base, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}"))

    # ==========================================
    # PHASE 2: Hyperspace (Universal Mode)
    # ==========================================
    if HYPERSPACE_AVAILABLE and (not target_db or "hyper" in target_db):
        mode = cfg.HYPER_MODE.lower()
        use_hyp = (mode == "poincare")
        
        target_vecs = doc_vecs_hyp if use_hyp else doc_vecs_euc
        target_q_vecs = q_vecs_hyp if use_hyp else q_vecs_euc
        target_dim = cfg.dim_hyp if use_hyp else cfg.dim_base
        geom_name = "Poincaré" if use_hyp else "Euclidean"
        
        print(f"\n🚀 PHASE 2: HyperspaceDB ({target_dim}d {geom_name})")

        try:
            # 1. Metric Detection
            server_metric = detect_hyperspace_metric()
            if server_metric and server_metric != mode:
                print(f"   ⚠️ Skipping: Server is in '{server_metric}' mode, but benchmark wants '{mode}'.")
                final_results.append(Result("Hyperspace", target_dim, geom_name, mode.capitalize(), 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Skipped: mode mismatch ({server_metric})"))
                # Go to next phase
                raise StopIteration
            
            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
            coll_name = "bench_semantic"
            try: client.delete_collection(coll_name)
            except Exception: pass # Silencing gRPC errors
            
            if not client.create_collection(coll_name, dimension=target_dim, metric=mode):
                print(f"❌ Failed to create collection '{coll_name}'. Check server logs.")
                raise RuntimeError("Collection creation failed")
            
            # Insert
            print("   Inserting into Hyperspace...")
            t0 = time.time()
            
            # gRPC limit is 4MB. 1536D * 8 bytes (safe float64) = ~12KB per vector.
            # 4MB / 12KB = 333 vectors max. We use 250 for safety.
            # For 128D, 4MB / (128*8) = 4000 vectors max. We use 2000.
            h_batch_size = max(10, int(4_000_000 / (target_dim * 8)))
            print(f"   Using batch size: {h_batch_size} (based on dim {target_dim})")
            
            failed_inserts = 0
            total_batches = 0
            
            for i in tqdm(range(0, len(target_vecs), h_batch_size), desc="Inserting"):
                batch_vecs = target_vecs[i : i + h_batch_size]
                batch_ids = doc_ids[i : i + h_batch_size]
                int_ids = list(range(i, i + len(batch_ids)))
                metas = [{"doc_id": did} for did in batch_ids]
                
                if not client.batch_insert(batch_vecs.tolist(), int_ids, metas, collection=coll_name):
                    failed_inserts += 1
                total_batches += 1
            
            if failed_inserts > 0:
                print(f"⚠️ Warning: {failed_inserts}/{total_batches} batches failed insertion!")
                
            v_dur = time.time() - t0
            print(f"   Ingestion finished. Time: {v_dur:.2f}s")
            
            # Wait for background indexing to complete (eventual consistency)
            wait_for_indexing(collection=coll_name)
            
            # Search & Eval
            all_res_ids = []
            all_gt_ids = []
            lats = []
            
            print("   Searching Hyperspace...")
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(target_q_vecs)):
                q_id = test_query_ids[i]
                all_gt_ids.append(valid_qrels.get(q_id, []))
                
                ts = time.time()
                res = client.search(q_vec.tolist(), top_k=10, collection=coll_name)
                lats.append((time.time() - ts) * 1000)
                
                # Extract IDs
                all_res_ids.append(extract_ids(res))
            
            search_dur = time.time() - search_t0
            recall, mrr, ndcg = calculate_accuracy(all_res_ids, all_gt_ids, 10)
            gt_for_mode = math_gt_hyp if use_hyp else math_gt_euc
            recall_sys = calculate_system_recall(all_res_ids, gt_for_mode, 10)
            
            # Concurrency
            print("   Testing Hyperspace Concurrency...")
            q_list = target_q_vecs[0].tolist()
            def hyperspace_query():
                client.search(q_list, top_k=10, collection=coll_name)
            conc = run_concurrency_profile(hyperspace_query)
            
            # Get disk usage - prefer API, fallback to local path sensing
            disk = get_hyperspace_disk_api()
            if not disk:
                disk = get_local_disk("../data")
            disk = format_size(disk)
            
            final_results.append(Result(
                database="Hyperspace",
                dimension=target_dim,
                geometry=geom_name,
                metric=mode.capitalize(),
                insert_qps=len(docs) / v_dur,
                search_qps=len(test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            ))
            
            print(f"   📊 Hyperspace Results: Semantic Recall@10={recall:.4f}, System Recall@10={recall_sys:.4f}, MRR@10={mrr:.4f}")
            
            # Cleanup
            client.delete_collection("bench_semantic")
        except Exception as e:
            print(f"   ⚠️ Hyperspace error: {e}")
            final_results.append(Result("Hyperspace", target_dim, geom_name, mode.capitalize(), 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}"))

    # ==========================================
    # FINAL REPORT
    # ==========================================
    print_table(final_results)
    
    # Write to Markdown
    with open("BENCHMARK_STORY.md", "w") as f:
        f.write("# 📐 Semantic Hyperbolic Advantage: Comparison Report\n\n")
        f.write(f"Testing on **{cfg.dataset_name}**. Dataset subset: **{len(docs):,}** docs.\n")
        f.write(f"Accuracy based on **{len(test_queries):,}** semantic queries.\n\n")
        f.write("| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for r in final_results:
            if r.status == "Success":
                f.write(f"| **{r.database}** | {r.dimension:,} | {r.geometry} | {r.metric} | {r.insert_qps:,.0f} | {r.search_qps:,.0f} | {r.p99:.2f}ms | {r.recall:.1%} | {r.recall_sys:.1%} | {r.mrr:.2f} | {r.ndcg:.2f} | {r.c1_qps:,.0f} | {r.c10_qps:,.0f} | {r.c30_qps:,.0f} | {r.disk_usage} |\n")
        
        f.write("\n## 💡 Accuracy Analysis\n")
        hyp = next((r for r in final_results if r.database == "Hyperspace"), None)
        base = next((r for r in final_results if r.database == "Milvus"), None)
        
        if hyp and base:
            f.write(f"Hyperspace 64d Recall: {hyp.recall:.1%}\n")
            f.write(f"Milvus 1024d Recall: {base.recall:.1%}\n\n")
            if hyp.recall >= base.recall * 0.95:
                f.write("✅ **Semantic Equivalence:** 64d Hyperbolic vectors achieve comparable accuracy to 1024d Euclidean embeddings while being **16x smaller**.\n")
            else:
                f.write(f"⚠️ **Accuracy Gap:** Hyperspace is {(base.recall - hyp.recall)*100:.1f}% behind baseline. Consider additional fine-tuning epochs.\n")

if __name__ == "__main__":
    run_benchmark()