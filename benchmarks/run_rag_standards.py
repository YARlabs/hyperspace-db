#!/usr/bin/env python3
import os
import sys
import time
import json
import math
import hashlib
import numpy as np
import pandas as pd
import argparse
from tqdm import tqdm
from concurrent.futures import ThreadPoolExecutor
import warnings

# Suppress generic and ORM Milvus deprecation warnings from flooding benchmark logs
warnings.filterwarnings("ignore", category=DeprecationWarning)
try:
    from pymilvus import PyMilvusDeprecationWarning
    warnings.filterwarnings("ignore", category=PyMilvusDeprecationWarning)
except ImportError:
    pass

# Add python SDK to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdks", "python")))

# Database client imports
try:
    from hyperspace import HyperspaceClient
    HYPERSPACE_AVAILABLE = True
except ImportError:
    HYPERSPACE_AVAILABLE = False

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, PointStruct, VectorParams
    QDRANT_AVAILABLE = True
except ImportError:
    QDRANT_AVAILABLE = False

try:
    import chromadb
    from chromadb.config import Settings
    CHROMA_AVAILABLE = True
    # Suppress ChromaDB telemetry capture warnings completely by patching the Posthog capture class
    try:
        import chromadb.telemetry.product.posthog
        chromadb.telemetry.product.posthog.Posthog.capture = lambda *args, **kwargs: None
        chromadb.telemetry.product.posthog.Posthog._direct_capture = lambda *args, **kwargs: None
    except Exception:
        pass
except ImportError:
    CHROMA_AVAILABLE = False

try:
    from pymilvus import connections, Collection, CollectionSchema, DataType, FieldSchema, utility
    MILVUS_AVAILABLE = True
except ImportError:
    MILVUS_AVAILABLE = False

try:
    import weaviate
    WEAVIATE_AVAILABLE = True
except ImportError:
    WEAVIATE_AVAILABLE = False

try:
    from sentence_transformers import SentenceTransformer
    EMBEDDER_AVAILABLE = True
except ImportError:
    EMBEDDER_AVAILABLE = False

try:
    from datasets import load_dataset
    DATASETS_AVAILABLE = True
except ImportError:
    DATASETS_AVAILABLE = False

import threading

class ResourceSampler(threading.Thread):
    def __init__(self, db_name, interval=0.01):
        super().__init__()
        self.db_name = db_name
        self.interval = interval
        self.stop_event = threading.Event()
        self.cpu_samples = []
        self.ram_samples = []
        self.container_map = {
            "Qdrant": "benchmarks-qdrant-1",
            "ChromaDB": "benchmarks-chroma-1",
            "Milvus": "benchmarks-milvus-1",
            "Weaviate": "benchmarks-weaviate-1"
        }
        
    def run(self):
        import subprocess
        import json
        import requests
        
        container_name = self.container_map.get(self.db_name)
        is_hs = self.db_name in ["HyperspaceDB", "HyperspaceDB-Wave", "HyperspaceDB-WaveServer"]
        
        while not self.stop_event.is_set():
            cpu_val = 0.0
            ram_val = 0.0
            
            if is_hs:
                # Query Hyperspace metrics API
                try:
                    url = "http://localhost:50050/api/metrics"
                    headers = {
                        "x-api-key": "I_LOVE_HYPERSPACEDB",
                        "x-hyperspace-user-id": "default_admin"
                    }
                    r = requests.get(url, headers=headers, timeout=2)
                    if r.status_code == 200:
                        data = r.json()
                        cpu_val = float(data.get("cpu_usage_percent", 0.0))
                        ram_val = float(data.get("ram_usage_mb", 0.0))
                except Exception:
                    pass
            else:
                if container_name:
                    try:
                        # Get cpu and memory from docker stats
                        # Output format Name:CPUPerc:MemUsage
                        res = subprocess.run(
                            ["docker", "stats", "--no-stream", "--format", "{{.CPUPerc}}:{{.MemUsage}}", container_name],
                            capture_output=True, text=True, timeout=2
                        )
                        if res.returncode == 0 and res.stdout.strip():
                            parts = res.stdout.strip().split(":")
                            if len(parts) >= 2:
                                cpu_str = parts[0].replace("%", "").strip()
                                cpu_val = float(cpu_str)
                                
                                mem_str = parts[1].split("/")[0].strip()
                                # parse mem_str like 361.2MiB or 1.431GiB
                                ram_mb = 0.0
                                if "GiB" in mem_str:
                                    ram_mb = float(mem_str.replace("GiB", "").strip()) * 1024
                                elif "MiB" in mem_str:
                                    ram_mb = float(mem_str.replace("MiB", "").strip())
                                elif "KiB" in mem_str:
                                    ram_mb = float(mem_str.replace("KiB", "").strip()) / 1024
                                elif "B" in mem_str:
                                    ram_mb = float(mem_str.replace("B", "").strip()) / (1024 * 1024)
                                ram_val = ram_mb
                    except Exception:
                        pass
                        
            self.cpu_samples.append(cpu_val)
            self.ram_samples.append(ram_val)
                
            time.sleep(self.interval)
            
    def stop(self):
        self.stop_event.set()


def get_db_disk_usage(db_name):
    import subprocess
    import os
    
    disk_str = "N/A"
    
    # 1. Hyperspace Disk Usage (API or directory size)
    if db_name in ["HyperspaceDB", "HyperspaceDB-Wave", "HyperspaceDB-WaveServer"]:
        # Try API first
        try:
            import requests
            url = "http://localhost:50050/api/metrics"
            headers = {
                "x-api-key": "I_LOVE_HYPERSPACEDB",
                "x-hyperspace-user-id": "default_admin"
            }
            r = requests.get(url, headers=headers, timeout=2)
            if r.status_code == 200:
                data = r.json()
                disk_mb = float(data.get("disk_usage_mb", 0.0))
                return f"{disk_mb:.2f} MB"
        except Exception:
            pass
        # Fallback to local directory size
        try:
            db_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "data"))
            total_size = 0
            if os.path.exists(db_dir):
                for dirpath, dirnames, filenames in os.walk(db_dir):
                    for f in filenames:
                        fp = os.path.join(dirpath, f)
                        if not os.path.islink(fp):
                            total_size += os.path.getsize(fp)
            disk_str = f"{total_size / (1024 * 1024):.2f} MB"
        except Exception:
            pass
    # 2. Other DBs
    else:
        disk_cmds = {
            "Qdrant": ["docker", "exec", "benchmarks-qdrant-1", "du", "-sk", "/qdrant/storage"],
            "ChromaDB": ["docker", "exec", "benchmarks-chroma-1", "du", "-sk", "/data"],
            "Milvus": ["docker", "exec", "benchmarks-milvus-minio-1", "du", "-sk", "/minio_data"],
            "Weaviate": ["docker", "exec", "benchmarks-weaviate-1", "du", "-sk", "/var/lib/weaviate"]
        }
        cmd = disk_cmds.get(db_name)
        if cmd:
            try:
                du_res = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
                if du_res.returncode == 0 and du_res.stdout.strip():
                    kb_val = int(du_res.stdout.strip().split()[0])
                    disk_str = f"{kb_val / 1024:.2f} MB"
            except Exception:
                pass
                
    return disk_str


def wait_for_indexing(host="localhost", port=50050, collection="bench_semantic", timeout=None):
    """Wait for HyperspaceDB background indexing to complete with progress display"""
    import requests
    print(f"\n   ⏳ Monitoring indexing for '{collection}' (waiting for queue to clear)...")
    url = f"http://{host}:{port}/api/collections/{collection}/stats"
    
    headers = {
        "x-api-key": "I_LOVE_HYPERSPACEDB",
        "x-hyperspace-user-id": "default_admin"
    }
    
    start_time = time.time()
    sleep_time = 0.05  # Start with highly responsive 50ms polling
    
    while True:
        if timeout and time.time() - start_time > timeout:
            print(f"\n   ⚠️ Timeout after {timeout}s. Proceeding with partial index...")
            break
            
        try:
            response = requests.get(url, headers=headers, timeout=5)
            
            if response.status_code == 200:
                data = response.json()
                count = data.get("count", data.get("vector_count", 0))
                queue = data.get("indexing_queue", 0)
                
                elapsed = time.time() - start_time
                print(f"\r   [Indexing] Remaining: {queue:,} | Total Indexed: {count:,} | Elapsed: {elapsed:.2f}s          ", end="", flush=True)
                
                if "indexing_queue" not in data:
                     if count > 0:
                         print("\n   ✅ Count > 0. Stability wait...")
                         time.sleep(1)
                         break
                elif queue == 0 and count > 0:
                     print(f"\n   ✅ Indexing complete! Docs ready: {count:,}")
                     break
            elif response.status_code == 401:
                print(f"\r   🚫 Auth Error (401): Check HYPERSPACE_API_KEY          ", end="", flush=True)
                time.sleep(1) 
                continue
            elif response.status_code == 404:
                print(f"\r   ⌛ Collection '{collection}' initializing...", end="", flush=True)
            else:
                print(f"\r   ⚠️ Server returned status {response.status_code}            ", end="", flush=True)
                
            time.sleep(sleep_time)
            # Exponential back-off to avoid API spamming for long operations
            sleep_time = min(sleep_time * 1.5, 1.0)
        except Exception as e:
            print(f"\r   ⏳ Connection issue (retrying... {str(e)[:40]})          ", end="", flush=True)
            time.sleep(1)


def hs_cognitive_rerank(client, results, coll_name, limit):
    if len(results) <= 2:
        return results
    ids = [r["id"] for r in results]
    try:
        pts = client.get_points(ids, collection=coll_name)
    except Exception as e:
        print(f"   ⚠️ Could not fetch vectors for reranking: {e}")
        return results[:limit]
    
    id_to_vec = {p["id"]: p["vector"] for p in pts}
    scored_results = []
    
    for h in results:
        node_id = h.get("id")
        if node_id is None:
            scored_results.append((h, 1.0 / (1.0 + h.get("distance", 1.0))))
            continue
            
        vec = id_to_vec.get(node_id)
        if vec is None:
            scored_results.append((h, 1.0 / (1.0 + h.get("distance", 1.0))))
            continue
            
        # Scale spatial vectors to Poincare ball (norm < 1.0)
        norm = math.sqrt(sum(x*x for x in vec))
        if norm > 0:
            scaled_vec = [x * 0.95 / norm for x in vec]
        else:
            scaled_vec = [0.0] * len(vec)
            
        neighbors = []
        for other_id in ids:
            if other_id != node_id and other_id in id_to_vec:
                other_vec = id_to_vec[other_id]
                other_norm = math.sqrt(sum(x*x for x in other_vec))
                if other_norm > 0:
                    scaled_other = [x * 0.95 / other_norm for x in other_vec]
                else:
                    scaled_other = [0.0] * len(other_vec)
                neighbors.append(scaled_other)
                
        if neighbors:
            from hyperspace.math import local_entropy
            try:
                entropy = local_entropy(scaled_vec, neighbors, c=1.0)
            except Exception:
                entropy = 0.5
        else:
            entropy = 0.5
            
        similarity = 1.0 / (1.0 + h.get("distance", 1.0))
        # Outlier spatial points with high local entropy are penalized
        rerank_score = similarity * (1.0 - 0.25 * entropy)
        scored_results.append((h, rerank_score))
        
    scored_results.sort(key=lambda x: x[1], reverse=True)
    return [item[0] for item in scored_results][:limit]


class RAGStandardsEvaluator:
    def __init__(self, limit=1000, query_limit=200, model_name="all-MiniLM-L6-v2", ef_search=32):
        self.limit = limit
        self.query_limit = query_limit
        self.model_name = model_name
        self.ef_search = ef_search
        self.hybrid = False
        
        # Load local embedder
        if EMBEDDER_AVAILABLE:
            print(f"⚙️ Loading local embedding model '{model_name}'...")
            self.model = SentenceTransformer(model_name)
            self.dim = self.model.get_embedding_dimension() if hasattr(self.model, "get_embedding_dimension") else self.model.get_sentence_embedding_dimension()
            print(f"✅ Embedder loaded. Dimension: {self.dim}")
        else:
            print("❌ sentence-transformers package is missing. Cannot proceed.")
            sys.exit(1)

    # Domains served from data/extra/ instead of data/ragbench/
    EXTRA_DOMAINS = {"maud", "legalbench", "trec_ct", "bioasq", "scidocs", "frames", "rgb"}

    # Human-readable domain labels for the HTML report
    DOMAIN_LABELS = {
        "covidqa":    "CovidQA",
        "finqa":      "FinQA",
        "cuad":       "CUAD (Legal)",
        "msmarco":    "MSMARCO",
        "tatqa":      "TatQA",
        "pubmedqa":   "PubMedQA",
        "techqa":     "TechQA",
        "hotpotqa":   "HotpotQA",
        "maud":       "MAUD (M&A)",
        "legalbench": "LegalBench",
        "trec_ct":    "TREC Clinical Trials",
        "bioasq":     "BioASQ",
        "scidocs":    "SciDocs (Citations)",
        "frames":     "Google FRAMES (Multi-hop)",
        "rgb":        "RGB (Noise Robustness)",
    }

    def load_domain_dataset(self, domain):
        label = self.DOMAIN_LABELS.get(domain, domain.upper())
        print(f"\n📦 Loading {label} dataset...")
        loaded_real = False
        corpus = []
        queries = []

        # Route cache directory: extra datasets live in data/extra/, RAGBench in data/ragbench/
        bench_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "data"))
        if domain in self.EXTRA_DOMAINS:
            data_dir = os.path.join(bench_dir, "extra")
        else:
            data_dir = os.path.join(bench_dir, "ragbench")
        local_path = os.path.join(data_dir, f"{domain}_ragbench.jsonl")
        
        # 1. Attempt to load from local cache file if it exists
        if os.path.exists(local_path):
            print(f"   💾 [Local Cache] Found real pre-downloaded dataset at: {local_path}")
            try:
                rows = []
                with open(local_path, "r", encoding="utf-8") as f:
                    for line in f:
                        rows.append(json.loads(line))
                
                passages_set = set()
                query_list = []
                
                for row in rows[:self.query_limit * 3]:
                    q_text = row["question"]
                    doc_list = row["documents"]
                    relevant_keys = row.get("all_relevant_sentence_keys", [])
                    
                    relevant_indices = set()
                    for key in relevant_keys:
                        digits = []
                        for char in key:
                            if char.isdigit():
                                digits.append(char)
                            else:
                                break
                        if digits:
                            relevant_indices.add(int("".join(digits)))
                            
                    if not relevant_indices and doc_list:
                        relevant_indices.add(0)
                        
                    gold_passages = []
                    for idx in relevant_indices:
                        if idx < len(doc_list):
                            gold_passages.append(doc_list[idx])
                            
                    for doc in doc_list:
                        passages_set.add(doc)
                        
                    query_list.append({
                        "question": q_text,
                        "gold_passages": gold_passages,
                        "all_passages": doc_list
                    })
                
                corpus = sorted(list(passages_set))[:self.limit]
                corpus_set = set(corpus)
                
                queries = []
                for q in query_list:
                    valid_gold = [p for p in q["gold_passages"] if p in corpus_set]
                    if valid_gold and len(queries) < self.query_limit:
                        queries.append({
                            "question": q["question"],
                            "gold_passages": valid_gold
                        })
                
                if len(corpus) > 0 and len(queries) > 0:
                    loaded_real = True
                    print(f"   💾 [Local Cache] Successfully loaded {len(corpus)} unique documents and {len(queries)} RAG queries.")
            except Exception as e:
                print(f"⚠️ Error reading local cache: {e}")
                
        # 2. If not loaded from local cache, try to auto-download via download_ragbench.py
        if not loaded_real and DATASETS_AVAILABLE:
            if domain in self.EXTRA_DOMAINS:
                # For extra domains, point user to the downloader — schema is more complex
                print(f"   ⚠️  [{domain}] not cached locally. Run to download:")
                print(f"      python3 download_ragbench.py --domain {domain}")
            else:
              try:
                print(f"   📡 [HF Streaming] Streaming and caching dataset from Hugging Face...")
                dataset = load_dataset("rungalileo/ragbench", domain, split="test", streaming=False)
                
                # Save locally for future use so we never use stubs
                os.makedirs(data_dir, exist_ok=True)
                print(f"   💾 [Local Cache] Saving real dataset to: {local_path}")
                with open(local_path, "w", encoding="utf-8") as f:
                    for row in dataset:
                        clean_row = {
                            "question": row["question"],
                            "documents": row["documents"],
                            "all_relevant_sentence_keys": row.get("all_relevant_sentence_keys", [])
                        }
                        f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                
                # Recursively call load_domain_dataset to load from the newly cached file
                return self.load_domain_dataset(domain)
              except Exception as e:
                print(f"⚠️ Could not stream/cache remote Hugging Face dataset (likely offline/timeout): {e}")

        if not loaded_real:
            raise FileNotFoundError(
                f"Dataset for domain '{domain}' could not be loaded from local cache or Hugging Face. "
                f"Please ensure the real dataset is cached locally by running:\n"
                f"  python3 download_ragbench.py --domain {domain}"
            )
            
        return corpus, queries

    def run_eval(self, db_name, corpus, queries, embeddings, q_embeddings, setup_fn, insert_fn, search_fn, cleanup_fn):
        print(f"\n🚀 Evaluating Database: {db_name}")
        
        # 1. Setup
        db_context = setup_fn()
        
        # 2. Inserting
        print("   Inserting into database...")
        t0 = time.time()
        insert_fn(db_context, embeddings, corpus)
        insert_time = time.time() - t0
        # If insert was bypassed, insert_time might be close to 0, avoid ZeroDivisionError
        insert_qps = len(corpus) / max(insert_time, 0.0001)
        print(f"   Indexed in {insert_time:.2f}s (Ingest QPS: {insert_qps:.1f})")
        
        # Settle indexing
        time.sleep(2)
        
        # 3. Search and Accuracy Metrics
        print("   Executing RAG retrieval queries...")
        latencies = []
        precisions = []
        recalls = []
        ndcgs = []
        mrrs = []
        context_precisions = []
        
        # Cache inspect.signature check outside the timed search loop
        import inspect
        sig = inspect.signature(search_fn)
        has_q_text = "q_text" in sig.parameters
        
        # Pre-build gold sets to avoid repeatedly doing it inside the loop
        gold_sets = [set(q["gold_passages"]) for q in queries]
        
        # Start resource sampler if enabled
        sampler = None
        if getattr(self, "measure_resources", False):
            sampler = ResourceSampler(db_name)
            sampler.start()
        
        for i, q_emb in enumerate(q_embeddings):
            gold_set = gold_sets[i]
            question = queries[i]["question"]
            
            # Execute search
            t_start = time.time()
            if has_q_text:
                hits = search_fn(db_context, q_emb, limit=10, q_text=question)
            else:
                hits = search_fn(db_context, q_emb, limit=10)
            latency = (time.time() - t_start) * 1000
            latencies.append(latency)
            
            retrieved = [h["text"] for h in hits]
            
            # ── Calculate Precision@10 & Recall@10 ──
            hits_in_gold = [p for p in retrieved if p in gold_set]
            precision = len(hits_in_gold) / len(retrieved) if retrieved else 0.0
            recall = len(hits_in_gold) / len(gold_set) if gold_set else 0.0
            precisions.append(precision)
            recalls.append(recall)
            
            # ── Calculate MRR ──
            mrr = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    mrr = 1.0 / (rank + 1)
                    break
            mrrs.append(mrr)
            
            # ── Calculate NDCG@10 ──
            dcg = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    dcg += 1.0 / np.log2(rank + 2)
            ideal_hits = min(10, len(gold_set))
            idcg = sum(1.0 / np.log2(r + 2) for r in range(ideal_hits))
            ndcgs.append(dcg / idcg if idcg > 0.0 else 0.0)
            
            # ── Calculate Context Precision ──
            num_relevant = 0
            precision_sum = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    num_relevant += 1
                    precision_sum += num_relevant / (rank + 1)
            context_precision = precision_sum / num_relevant if num_relevant > 0 else 0.0
            context_precisions.append(context_precision)
            
        # Stop resource sampler if enabled
        if sampler:
            sampler.stop()
            sampler.join()
            
        avg_precision = np.mean(precisions)
        avg_recall = np.mean(recalls)
        avg_ndcg = np.mean(ndcgs)
        avg_mrr = np.mean(mrrs)
        avg_context_precision = np.mean(context_precisions)
        
        p50 = np.percentile(latencies, 50)
        p99 = np.percentile(latencies, 99)
        overall_qps = len(queries) / (sum(latencies) / 1000.0)
        
        print(f"📊 Results for {db_name}:")
        print(f"   Precision@10: {avg_precision:.1%}")
        print(f"   Recall@10:    {avg_recall:.1%}")
        print(f"   NDCG@10:      {avg_ndcg:.3f}")
        print(f"   MRR:          {avg_mrr:.3f}")
        print(f"   Context Prec: {avg_context_precision:.1%}")
        print(f"   Latency p50:  {p50:.2f}ms | p99: {p99:.2f}ms (QPS: {overall_qps:.0f})")
        
        # 4. Measure resources before cleanup if enabled
        if getattr(self, "measure_resources", False):
            disk_usage = get_db_disk_usage(db_name)
            if sampler and sampler.ram_samples:
                ram_usage = f"{np.max(sampler.ram_samples):.1f} MB"
            else:
                ram_usage = "N/A"
            if sampler and sampler.cpu_samples:
                cpu_usage = f"Avg: {np.mean(sampler.cpu_samples):.1f}% / Max: {np.max(sampler.cpu_samples):.1f}%"
            else:
                cpu_usage = "N/A"
            print(f"   Resource usage -> RAM: {ram_usage} | Disk: {disk_usage} | CPU: {cpu_usage}")
        else:
            ram_usage, disk_usage, cpu_usage = "N/A", "N/A", "N/A"
        
        # 5. Cleanup
        cleanup_fn(db_context)
        
        result_dict = {
            "db_name": db_name,
            "precision": avg_precision,
            "recall": avg_recall,
            "ndcg": avg_ndcg,
            "mrr": avg_mrr,
            "context_precision": avg_context_precision,
            "p50": p50,
            "p99": p99,
            "qps": overall_qps,
            "insert_qps": insert_qps,
            "ram_usage": ram_usage,
            "disk_usage": disk_usage,
            "cpu_usage": cpu_usage
        }
        return result_dict

    def generate_html_report(self, domain_results):
        html_path = "RAG_STANDARDS_REPORT.html"
        
        # Render Javascript and Tabs
        tabs_html = ""
        tables_html = ""
        js_charts_data = {}
        
        for idx, (domain, results) in enumerate(domain_results.items()):
            active_class = "active" if idx == 0 else ""
            display_style = "block" if idx == 0 else "none"
            
            tabs_html += f"""
            <button class="tab-link {active_class}" onclick="openDomain(event, '{domain}')">{domain.upper()}</button>
            """
            
            rows = ""
            js_labels = []
            js_qps = []
            js_recall = []
            js_c_precision = []
            
            for r in results:
                js_labels.append(r["db_name"])
                js_qps.append(r["qps"])
                js_recall.append(r["recall"] * 100)
                js_c_precision.append(r["context_precision"] * 100)
                
                ram_val = r.get("ram_usage", "N/A")
                disk_val = r.get("disk_usage", "N/A")
                cpu_val = r.get("cpu_usage", "N/A")
                
                rows += f"""
                <tr>
                    <td style="font-weight: bold; color: #38bdf8;">{r['db_name']}</td>
                    <td>{r['insert_qps']:.0f}</td>
                    <td>{r['qps']:.0f}</td>
                    <td>{r['p50']:.2f} ms</td>
                    <td>{r['p99']:.2f} ms</td>
                    <td style="color: #a5f3fc;">{ram_val}</td>
                    <td style="color: #fde047;">{disk_val}</td>
                    <td style="color: #f472b6;">{cpu_val}</td>
                    <td>{r['precision']:.1%}</td>
                    <td style="font-weight: bold; color: #10b981;">{r['recall']:.1%}</td>
                    <td>{r['ndcg']:.3f}</td>
                    <td style="font-weight: bold; color: #a78bfa;">{r['context_precision']:.1%}</td>
                </tr>
                """
                
            js_charts_data[domain] = {
                "labels": js_labels,
                "qps": js_qps,
                "recall": js_recall,
                "c_precision": js_c_precision
            }
            
            tables_html += f"""
            <div id="{domain}" class="tab-content" style="display: {display_style};">
                <div class="grid">
                    <div class="card">
                        <h2>Retrieval Throughput ({domain.upper()} QPS)</h2>
                        <canvas id="qpsChart_{domain}"></canvas>
                    </div>
                    <div class="card">
                        <h2>Retrieval Quality ({domain.upper()} Recall & Context Precision)</h2>
                        <canvas id="qualityChart_{domain}"></canvas>
                    </div>
                </div>
 
                <div class="card" style="overflow-x: auto;">
                    <h2>Performance Leaderboard ({domain.upper()})</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Database</th>
                                <th>Insert QPS</th>
                                <th>Search QPS</th>
                                <th>Latency p50</th>
                                <th>Latency p99</th>
                                <th>RAM Usage</th>
                                <th>Disk Usage</th>
                                <th>CPU Usage</th>
                                <th>Precision@10</th>
                                <th>Recall@10</th>
                                <th>NDCG@10</th>
                                <th>Context Precision</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows}
                        </tbody>
                    </table>
                </div>
            </div>
            """
            
        html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Standardized RAG Retrieval Benchmark Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        :root {{
            --bg: #0b0f19;
            --card-bg: #111827;
            --text: #f3f4f6;
            --accent: #38bdf8;
            --border: #1f2937;
        }}
        body {{
            font-family: 'Inter', sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 2rem;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        header {{
            text-align: center;
            margin-bottom: 2rem;
        }}
        h1 {{
            color: var(--accent);
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
            text-shadow: 0 0 15px rgba(56, 189, 248, 0.3);
        }}
        p.subtitle {{
            color: #9ca3af;
            font-size: 1.1rem;
        }}
        .tab-bar {{
            display: flex;
            justify-content: center;
            gap: 1rem;
            margin-bottom: 2rem;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1rem;
        }}
        .tab-link {{
            background: #1f2937;
            border: 1px solid var(--border);
            color: #9ca3af;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            cursor: pointer;
            font-weight: 600;
            transition: all 0.2s ease;
        }}
        .tab-link.active, .tab-link:hover {{
            background: var(--accent);
            color: var(--bg);
            border-color: var(--accent);
            box-shadow: 0 0 10px rgba(56, 189, 248, 0.4);
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 2rem;
            margin-bottom: 3rem;
        }}
        .card {{
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 1rem;
            padding: 1.5rem;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.2);
        }}
        .card h2 {{
            font-size: 1.2rem;
            margin-top: 0;
            color: #94a3b8;
            border-bottom: 1px solid var(--border);
            padding-bottom: 0.5rem;
        }}
        canvas {{
            max-height: 300px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
            border-radius: 0.5rem;
            overflow: hidden;
        }}
        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--border);
        }}
        th {{
            background: #1f2937;
            color: #9ca3af;
            font-weight: 600;
        }}
        tr:hover {{
            background: rgba(31, 41, 55, 0.5);
        }}
        .badge {{
            background: rgba(56, 189, 248, 0.1);
            color: var(--accent);
            padding: 0.25rem 0.5rem;
            border-radius: 0.375rem;
            font-size: 0.85rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Standardized RAG Benchmark</h1>
            <p class="subtitle">Verifiable Retrieval Quality Evaluation across Galileo RAGBench domains</p>
        </header>

        <div class="tab-bar">
            {tabs_html}
        </div>

        {tables_html}
    </div>

    <script>
        function openDomain(evt, domainName) {{
            var i, tabcontent, tablinks;
            tabcontent = document.getElementsByClassName("tab-content");
            for (i = 0; i < tabcontent.length; i++) {{
                tabcontent[i].style.display = "none";
            }}
            tablinks = document.getElementsByClassName("tab-link");
            for (i = 0; i < tablinks.length; i++) {{
                tablinks[i].className = tablinks[i].className.replace(" active", "");
            }}
            document.getElementById(domainName).style.display = "block";
            evt.currentTarget.className += " active";
        }}

        const chartData = {json.dumps(js_charts_data)};
        
        // Render charts dynamically for all domains
        for (const [domain, data] of Object.entries(chartData)) {{
            // QPS Chart
            new Chart(document.getElementById('qpsChart_' + domain), {{
                type: 'bar',
                data: {{
                    labels: data.labels,
                    datasets: [{{
                        label: 'Query QPS',
                        data: data.qps,
                        backgroundColor: 'rgba(56, 189, 248, 0.5)',
                        borderColor: '#38bdf8',
                        borderWidth: 1
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }},
                        x: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }}
                    }}
                }}
            }});

            // Quality Chart
            new Chart(document.getElementById('qualityChart_' + domain), {{
                type: 'bar',
                data: {{
                    labels: data.labels,
                    datasets: [
                        {{
                            label: 'Recall@10 (%)',
                            data: data.recall,
                            backgroundColor: 'rgba(16, 185, 129, 0.5)',
                            borderColor: '#10b981',
                            borderWidth: 1
                        }},
                        {{
                            label: 'Context Precision (%)',
                            data: data.c_precision,
                            backgroundColor: 'rgba(167, 139, 250, 0.5)',
                            borderColor: '#a78bfa',
                            borderWidth: 1
                        }}
                    ]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{ min: 0, max: 100, grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }},
                        x: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }}
                    }}
                }}
            }});
        }}
    </script>
</body>
</html>
"""
        with open(html_path, "w") as f:
            f.write(html_content)
        print(f"\n✅ Standardized RAG Master Report Generated: {os.path.abspath(html_path)}")


STOPWORDS = {"a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are", "arent", "as", "at", "be", "because", "been", "before", "being", "below", "between", "both", "but", "by", "cant", "cannot", "could", "couldnt", "did", "didnt", "do", "does", "doesnt", "doing", "dont", "down", "during", "each", "few", "for", "from", "further", "had", "hadnt", "has", "hasnt", "have", "havent", "having", "he", "hed", "hell", "hes", "her", "here", "heres", "hers", "herself", "him", "himself", "his", "how", "hows", "i", "id", "ill", "im", "ive", "if", "in", "into", "is", "isnt", "it", "its", "itself", "lets", "me", "more", "most", "mustnt", "my", "myself", "no", "nor", "not", "of", "off", "on", "once", "only", "or", "other", "ought", "our", "ours", "ourselves", "out", "over", "own", "same", "shant", "she", "shed", "shell", "shes", "should", "shouldnt", "so", "some", "such", "than", "that", "thats", "the", "their", "theirs", "them", "themselves", "then", "there", "theres", "these", "they", "theyd", "theyll", "theyre", "theyve", "this", "those", "through", "to", "too", "under", "until", "up", "very", "was", "wasnt", "we", "wed", "well", "were", "weve", "werent", "what", "whats", "when", "whens", "where", "wheres", "which", "while", "who", "whos", "whom", "why", "whys", "with", "wont", "would", "wouldnt", "you", "youd", "youll", "youre", "youve", "your", "yours", "yourself", "yourselves"}


def clean_tokenize(text):
    text = text.lower()
    for char in '.,!?;:"()[]{}<>-_/\\*&^%$#@`~+=|':
        text = text.replace(char, ' ')
    tokens = text.split()
    return [t for t in tokens if t not in STOPWORDS and len(t) > 1]


class SimpleBM25:
    def __init__(self, corpus):
        self.corpus_size = len(corpus)
        self.avgdl = sum(len(clean_tokenize(doc)) for doc in corpus) / self.corpus_size if self.corpus_size > 0 else 1.0
        self.doc_freqs = []
        self.idf = {}
        self.doc_lens = []
        
        # Calculate doc frequencies and lengths
        for doc in corpus:
            words = clean_tokenize(doc)
            self.doc_lens.append(len(words))
            freqs = {}
            for word in words:
                freqs[word] = freqs.get(word, 0) + 1
            self.doc_freqs.append(freqs)
            for word in freqs:
                self.idf[word] = self.idf.get(word, 0) + 1
                
        # Calculate IDF
        for word, freq in self.idf.items():
            self.idf[word] = math.log(1.0 + (self.corpus_size - freq + 0.5) / (freq + 0.5))
            
    def get_scores(self, query, k1=1.5, b=0.75):
        query_words = clean_tokenize(query)
        scores = []
        for i in range(self.corpus_size):
            score = 0.0
            doc_len = self.doc_lens[i]
            freqs = self.doc_freqs[i]
            for word in query_words:
                if word in freqs:
                    freq = freqs[word]
                    idf_val = self.idf.get(word, 0.0)
                    score += idf_val * (freq * (k1 + 1)) / (freq + k1 * (1 - b + b * doc_len / self.avgdl))
            scores.append(score)
        return scores


def get_hybrid_weight(domain):
    # Returns vector_weight (alpha). BM25 weight is 1.0 - alpha.
    # Structural/legal domains need higher keyword weights:
    if domain in ["cuad", "maud", "legalbench", "frames", "scidocs", "trec_ct"]:
        return 0.55
    # Factual/question domains need higher semantic vector weights:
    return 0.85


def run_score_fusion(vector_hits, text_hits, vector_weight, limit=10):
    # vector_hits is a list of {"text": text, "score": score/dist}
    # text_hits is a list of {"text": text, "score": score}
    if not vector_hits and not text_hits:
        return []
        
    v_scores = {h["text"]: h["score"] for h in vector_hits}
    t_scores = {h["text"]: h["score"] for h in text_hits}
    
    all_texts = set(v_scores.keys()) | set(t_scores.keys())
    
    # Normalize vector scores to [0, 1]
    if v_scores:
        v_min = min(v_scores.values())
        v_max = max(v_scores.values())
        v_range = v_max - v_min
        if v_range < 1e-9: v_range = 1e-9
        
        # Detect if larger is better (similarity) or smaller is better (distance)
        first_text = vector_hits[0]["text"]
        last_text = vector_hits[-1]["text"]
        first_score = v_scores[first_text]
        last_score = v_scores[last_text]
        
        if first_score >= last_score:
            # Larger is better (similarity)
            v_norm = {t: (score - v_min) / v_range for t, score in v_scores.items()}
        else:
            # Smaller is better (distance)
            v_norm = {t: 1.0 - (score - v_min) / v_range for t, score in v_scores.items()}
    else:
        v_norm = {}
        
    # Normalize BM25 scores to [0, 1] (larger is always better)
    if t_scores:
        t_min = min(t_scores.values())
        t_max = max(t_scores.values())
        t_range = t_max - t_min
        if t_range < 1e-9: t_range = 1e-9
        t_norm = {t: (score - t_min) / t_range for t, score in t_scores.items()}
    else:
        t_norm = {}
        
    # Fused score calculation
    fused = {}
    vec_alpha = vector_weight
    key_alpha = 1.0 - vec_alpha
    
    for t in all_texts:
        vs = v_norm.get(t, 0.0)
        ts = t_norm.get(t, 0.0)
        fused[t] = vs * vec_alpha + ts * key_alpha
        
    sorted_hits = sorted(fused.items(), key=lambda x: x[1], reverse=True)
    return [{"text": text} for text, _ in sorted_hits[:limit]]


def main():
    parser = argparse.ArgumentParser(description="Standardized RAG Database Evaluator")
    parser.add_argument("--db", nargs="+", help="Specific DBs to test (hyperspace qdrant chroma milvus weaviate)")
    parser.add_argument("--limit", type=int, default=1000, help="Document corpus limit")
    parser.add_argument("--query-limit", type=int, default=200, help="RAG query limit")
    parser.add_argument("--ef-search", type=int, default=128, help="HNSW ef_search for candidate generation")
    parser.add_argument("--hybrid", action="store_true", help="Enable native Hybrid search (Vector + Lexical/BM25)")
    parser.add_argument(
        "--domain", nargs="+", default=["covidqa"],
        help=(
            "One or more domains to evaluate. Use 'all' for all 15 domains.\n"
            "RAGBench: covidqa finqa cuad msmarco tatqa pubmedqa techqa hotpotqa\n"
            "Structural: maud legalbench trec_ct bioasq scidocs frames rgb"
        )
    )
    parser.add_argument("--measure-resources", action="store_true", help="Enable RAM and Disk utilization measurement (requires docker/ps)")
    args = parser.parse_args()

    evaluator = RAGStandardsEvaluator(
        limit=args.limit,
        query_limit=args.query_limit,
        ef_search=args.ef_search
    )
    evaluator.hybrid = args.hybrid
    evaluator.measure_resources = args.measure_resources
    
    ALL_DOMAINS = [
        "covidqa", "finqa", "cuad", "msmarco", "tatqa", "pubmedqa", "techqa", "hotpotqa",
        "maud", "legalbench", "trec_ct", "bioasq", "scidocs", "frames", "rgb"
    ]
    if len(args.domain) == 1 and args.domain[0].lower() == "all":
        domains = ALL_DOMAINS
    else:
        domains = [d.lower() for d in args.domain]
        
    target_dbs = [d.lower() for d in args.db] if args.db else None
    
    # Domain mapped results
    domain_results = {}
    
    for dom in domains:
        print(f"\n================================================================================")
        print(f"🔥 STARTING EVALUATION SWEEP FOR DOMAIN: {dom.upper()}")
        print(f"================================================================================")
        
        try:
            corpus, queries = evaluator.load_domain_dataset(dom)
        except Exception as err:
            print(f"❌ Failed to load dataset for domain '{dom}': {err}")
            continue
            
        # Precompute embeddings once per domain
        print(f"   Embedding {len(corpus)} corpus documents...")
        t0 = time.time()
        embeddings = evaluator.model.encode(corpus, show_progress_bar=False, batch_size=64).tolist()
        embed_time = time.time() - t0
        print(f"   Embedded corpus in {embed_time:.2f}s (Speed: {len(corpus)/embed_time:.1f} docs/s)")
        
        print(f"   Embedding {len(queries)} queries...")
        t0 = time.time()
        q_texts = [q["question"] for q in queries]
        q_embeddings = evaluator.model.encode(q_texts, show_progress_bar=False).tolist()
        q_embed_time = time.time() - t0
        print(f"   Embedded queries in {q_embed_time:.2f}s (Speed: {len(queries)/q_embed_time:.1f} queries/s)")
        
        results = []
        
        # Initialize SimpleBM25 index for the domain if hybrid search is enabled
        bm25_index = None
        if evaluator.hybrid:
            print("   Building client-side BM25 index for hybrid search...")
            bm25_index = SimpleBM25(corpus)
        
        # Hyperspace collection reuse variables
        hs_collection_created = False
        hs_data_inserted = False
        coll_name_hs = f"rag_standards_{dom}"
        
        # ── HYPERSPACE ──
        if (not target_dbs or "hyperspace" in target_dbs) and HYPERSPACE_AVAILABLE:
            try:
                def hs_setup():
                    nonlocal hs_collection_created
                    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
                    if not hs_collection_created:
                        try: client.delete_collection(coll_name_hs)
                        except: pass
                        client.create_collection(coll_name_hs, dimension=evaluator.dim, metric="cosine")
                        dyn_ef = min(evaluator.ef_search, 32 if len(corpus) < 1000 else (64 if len(corpus) < 10000 else 100))
                        client.configure(ef_search=dyn_ef, ef_construction=200, collection=coll_name_hs)
                        hs_collection_created = True
                    return client
                
                def hs_ins(client, embs, texts):
                    nonlocal hs_data_inserted
                    if not hs_data_inserted:
                        ids = list(range(len(embs)))
                        metas = [{"text": t} for t in texts]
                        batch_size = 2000
                        for i in range(0, len(embs), batch_size):
                            client.batch_insert(
                                embs[i:i+batch_size],
                                ids[i:i+batch_size],
                                metas[i:i+batch_size],
                                collection=coll_name_hs
                            )
                        hs_data_inserted = True
                    else:
                        print("   [Hyperspace] Collection already populated, skipping ingestion.")
                
                def hs_srch(client, q_emb, limit, q_text=None):
                    if evaluator.hybrid and q_text:
                        alpha = get_hybrid_weight(dom)
                        res = client.search(
                            vector=q_emb,
                            query_text=q_text,
                            hybrid_alpha=alpha,
                            bm25={"fusion_method": "weighted", "method": "lucene", "k1": 1.5, "b": 0.75},
                            top_k=limit,
                            collection=coll_name_hs
                        )
                    else:
                        res = client.search(q_emb, top_k=limit, collection=coll_name_hs)
                    return [{"text": h["metadata"]["text"]} for h in res if "metadata" in h and "text" in h["metadata"]]
                
                def hs_cleanup(client):
                    client.close()
                    
                res = evaluator.run_eval("HyperspaceDB", corpus, queries, embeddings, q_embeddings, hs_setup, hs_ins, hs_srch, hs_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Hyperspace evaluation failed: {e}")
 
        # ── HYPERSPACE (WAVE CLIENT) ──
        if (not target_dbs or "hyperspace-wave" in target_dbs or "hyperspace_wave" in target_dbs) and HYPERSPACE_AVAILABLE:
            try:
                def hs_setup():
                    nonlocal hs_collection_created
                    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
                    if not hs_collection_created:
                        try: client.delete_collection(coll_name_hs)
                        except: pass
                        client.create_collection(coll_name_hs, dimension=evaluator.dim, metric="cosine")
                        dyn_ef = min(evaluator.ef_search, 32 if len(corpus) < 1000 else (64 if len(corpus) < 10000 else 100))
                        client.configure(ef_search=dyn_ef, ef_construction=200, collection=coll_name_hs)
                        hs_collection_created = True
                    return client
                
                def hs_ins(client, embs, texts):
                    nonlocal hs_data_inserted
                    if not hs_data_inserted:
                        ids = list(range(len(embs)))
                        metas = [{"text": t} for t in texts]
                        batch_size = 2000
                        for i in range(0, len(embs), batch_size):
                            client.batch_insert(
                                embs[i:i+batch_size],
                                ids[i:i+batch_size],
                                metas[i:i+batch_size],
                                collection=coll_name_hs
                            )
                        hs_data_inserted = True
                    else:
                        print("   [Hyperspace] Collection already populated, skipping ingestion.")
                
                def hs_srch_wave(client, q_emb, limit, q_text=None):
                    if evaluator.hybrid and q_text:
                        alpha = get_hybrid_weight(dom)
                        seeds = client.search(
                            vector=q_emb,
                            query_text=q_text,
                            hybrid_alpha=alpha,
                            bm25={"fusion_method": "weighted", "method": "lucene", "k1": 1.5, "b": 0.75},
                            top_k=5,
                            collection=coll_name_hs,
                            use_wave=False
                        )
                    else:
                        seeds = client.search(q_emb, top_k=5, collection=coll_name_hs, use_wave=False)
                    if not seeds:
                        return []
                    
                    node_scores = {}
                    node_metas = {}
                    
                    for seed in seeds:
                        seed_dist = seed.get("distance", 1.0)
                        seed_weight = 1.0 / (1.0 + seed_dist)
                        
                        traversed = client.traverse(
                            start_id=seed["id"],
                            max_depth=3,
                            max_nodes=limit * 3,
                            traversal_mode=1,
                            collection=coll_name_hs
                        )
                        
                        for rank_n, node in enumerate(traversed):
                            node_id = node["id"]
                            if "text" not in node["metadata"]:
                                continue
                            
                            node_score = seed_weight * (1.0 / (1.0 + rank_n))
                            node_scores[node_id] = node_scores.get(node_id, 0.0) + node_score
                            node_metas[node_id] = node["metadata"]
                            
                    sorted_nodes = sorted(node_scores.items(), key=lambda x: x[1], reverse=True)
                    return [{"text": node_metas[node_id]["text"]} for node_id, _ in sorted_nodes[:limit]]
                
                def hs_cleanup(client):
                    client.close()
                    
                res = evaluator.run_eval("HyperspaceDB-Wave", corpus, queries, embeddings, q_embeddings, hs_setup, hs_ins, hs_srch_wave, hs_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ HyperspaceDB-Wave evaluation failed: {e}")
 
        # ── HYPERSPACE (WAVE SERVER) ──
        if (not target_dbs or "hyperspace-waveserver" in target_dbs or "hyperspace_waveserver" in target_dbs) and HYPERSPACE_AVAILABLE:
            try:
                def hs_setup():
                    nonlocal hs_collection_created
                    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
                    if not hs_collection_created:
                        try: client.delete_collection(coll_name_hs)
                        except: pass
                        client.create_collection(coll_name_hs, dimension=evaluator.dim, metric="cosine")
                        dyn_ef = min(evaluator.ef_search, 32 if len(corpus) < 1000 else (64 if len(corpus) < 10000 else 100))
                        client.configure(ef_search=dyn_ef, ef_construction=200, collection=coll_name_hs)
                        hs_collection_created = True
                    return client
                
                def hs_ins(client, embs, texts):
                    nonlocal hs_data_inserted
                    if not hs_data_inserted:
                        ids = list(range(len(embs)))
                        metas = [{"text": t} for t in texts]
                        batch_size = 2000
                        for i in range(0, len(embs), batch_size):
                            client.batch_insert(
                                embs[i:i+batch_size],
                                ids[i:i+batch_size],
                                metas[i:i+batch_size],
                                collection=coll_name_hs
                            )
                        hs_data_inserted = True
                    else:
                        print("   [Hyperspace] Collection already populated, skipping ingestion.")
                
                def hs_srch_waveserver(client, q_emb, limit, q_text=None):
                    # Dynamic parameter tuning based on domain characteristics
                    if dom in ["cuad", "frames", "scidocs", "maud"]:
                        # Deep diffusion (structural/citations)
                        rf = 0.25
                        mass = 0.15
                        sigma = 0.6
                        qp = 0.8
                    else:
                        # Focused diffusion (factual/lookup QA)
                        rf = 0.8
                        mass = 0.4
                        sigma = 0.4
                        qp = 1.2
                    
                    filter_opts = {
                        "wave_restart_factor": str(rf),
                        "wave_mass_sq": str(mass),
                        "wave_sigma": str(sigma),
                        "wave_qp_strength": str(qp)
                    }
                    
                    if evaluator.hybrid and q_text:
                        alpha = get_hybrid_weight(dom)
                        results = client.search(
                            vector=q_emb,
                            query_text=q_text,
                            hybrid_alpha=alpha,
                            bm25={"fusion_method": "weighted", "method": "lucene", "k1": 1.5, "b": 0.75},
                            top_k=limit,
                            filter=filter_opts,
                            collection=coll_name_hs,
                            use_wave=True,
                            restart_factor=rf
                        )
                    else:
                        results = client.search(
                            vector=q_emb,
                            top_k=limit,
                            filter=filter_opts,
                            collection=coll_name_hs,
                            use_wave=True,
                            restart_factor=rf
                        )
                    return [{"text": r["metadata"]["text"]} for r in results if "metadata" in r and "text" in r["metadata"]]
                
                def hs_cleanup(client):
                    client.close()
                    
                res = evaluator.run_eval("HyperspaceDB-WaveServer", corpus, queries, embeddings, q_embeddings, hs_setup, hs_ins, hs_srch_waveserver, hs_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ HyperspaceDB-WaveServer evaluation failed: {e}")
 
        # ── QDRANT ──
        if (not target_dbs or "qdrant" in target_dbs) and QDRANT_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                def qd_setup():
                    client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.create_collection(
                        coll_name,
                        vectors_config=VectorParams(size=evaluator.dim, distance=Distance.COSINE)
                    )
                    # Bind the actual search method to avoid hasattr checking on every query
                    if hasattr(client, "search"):
                        client._search_fn = lambda q_emb, limit: [{"text": h.payload["text"], "score": h.score} for h in client.search(coll_name, query_vector=q_emb, limit=limit)]
                    else:
                        client._search_fn = lambda q_emb, limit: [{"text": h.payload["text"], "score": h.score} for h in client.query_points(coll_name, query=q_emb, limit=limit).points]
                    return client
                
                def qd_ins(client, embs, texts):
                    points = [PointStruct(id=i, vector=v, payload={"text": texts[i]}) for i, v in enumerate(embs)]
                    batch_size = 500
                    for i in range(0, len(points), batch_size):
                        client.upsert(coll_name, points[i:i+batch_size], wait=True)
                
                def qd_srch(client, q_emb, limit, q_text=None):
                    if evaluator.hybrid and q_text and bm25_index:
                        vec_hits = client._search_fn(q_emb, limit=50)
                        bm25_scores = bm25_index.get_scores(q_text)
                        top_bm25_idx = np.argsort(bm25_scores)[::-1][:50]
                        text_hits = [{"text": corpus[idx], "score": bm25_scores[idx]} for idx in top_bm25_idx if bm25_scores[idx] > 0]
                        alpha = get_hybrid_weight(dom)
                        return run_score_fusion(vec_hits, text_hits, alpha, limit=limit)
                    else:
                        return client._search_fn(q_emb, limit)
                
                def qd_cleanup(client):
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.close()
                    
                res = evaluator.run_eval("Qdrant", corpus, queries, embeddings, q_embeddings, qd_setup, qd_ins, qd_srch, qd_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Qdrant evaluation failed: {e}")
 
        # ── CHROMADB ──
        if (not target_dbs or "chroma" in target_dbs) and CHROMA_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                
                class ChromaContext:
                    def __init__(self, col, client):
                        self.col = col
                        self.client = client
                        
                def chr_setup():
                    client = None
                    col = None
                    class UniversalNoopTelemetry:
                        def __init__(self, *args, **kwargs): pass
                        def capture(self, *args, **kwargs): pass
                        def context(self, *args, **kwargs): pass
                        def dependencies(self): return set()
                        def start(self): pass
                        def stop(self): pass
 
                    try:
                        client = chromadb.HttpClient(
                            host="localhost",
                            port=8000,
                            settings=Settings(anonymized_telemetry=False),
                        )
                        if hasattr(client, "_telemetry"): client._telemetry = UniversalNoopTelemetry()
                        try: client.delete_collection(coll_name)
                        except: pass
                        col = client.create_collection(coll_name, metadata={"hnsw:space": "cosine"})
                    except Exception:
                        client = None
                        col = None
 
                    if col is None:
                        db_dir = os.path.abspath(".chroma_rag_data")
                        client = chromadb.PersistentClient(
                            path=db_dir,
                            settings=Settings(anonymized_telemetry=False),
                        )
                        if hasattr(client, "_telemetry"): client._telemetry = UniversalNoopTelemetry()
                        try: client.delete_collection(coll_name)
                        except: pass
                        col = client.create_collection(coll_name, metadata={"hnsw:space": "cosine"})
                    
                    return ChromaContext(col, client)
                
                def chr_ins(ctx, embs, texts):
                    ids = [str(i) for i in range(len(embs))]
                    batch_size = 500
                    for i in range(0, len(embs), batch_size):
                        ctx.col.add(
                            embeddings=embs[i:i+batch_size],
                            ids=ids[i:i+batch_size],
                            documents=texts[i:i+batch_size]
                        )
                
                def chr_srch(ctx, q_emb, limit, q_text=None):
                    if evaluator.hybrid and q_text and bm25_index:
                        res = ctx.col.query(query_embeddings=[q_emb], n_results=50)
                        documents = res.get("documents", [[]])[0]
                        distances = res.get("distances", [[]])[0]
                        vec_hits = [{"text": doc, "score": dist} for doc, dist in zip(documents, distances)]
                        
                        bm25_scores = bm25_index.get_scores(q_text)
                        top_bm25_idx = np.argsort(bm25_scores)[::-1][:50]
                        text_hits = [{"text": corpus[idx], "score": bm25_scores[idx]} for idx in top_bm25_idx if bm25_scores[idx] > 0]
                        
                        alpha = get_hybrid_weight(dom)
                        return run_score_fusion(vec_hits, text_hits, alpha, limit=limit)
                    else:
                        res = ctx.col.query(query_embeddings=[q_emb], n_results=limit)
                        documents = res.get("documents", [[]])[0]
                        return [{"text": doc} for doc in documents]
                
                def chr_cleanup(ctx):
                    try:
                        ctx.client.delete_collection(coll_name)
                    except: pass
                    
                res = evaluator.run_eval("ChromaDB", corpus, queries, embeddings, q_embeddings, chr_setup, chr_ins, chr_srch, chr_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ ChromaDB evaluation failed: {e}")
 
        # ── MILVUS ──
        if (not target_dbs or "milvus" in target_dbs) and MILVUS_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                def mil_setup():
                    connections.connect(host="localhost", port="19530")
                    if utility.has_collection(coll_name):
                        utility.drop_collection(coll_name)
                    schema = CollectionSchema([
                        FieldSchema("id", DataType.INT64, is_primary=True, auto_id=False),
                        FieldSchema("vec", DataType.FLOAT_VECTOR, dim=evaluator.dim),
                        FieldSchema("text", DataType.VARCHAR, max_length=65535)
                    ])
                    col = Collection(coll_name, schema)
                    col.create_index("vec", {"metric_type": "COSINE", "index_type": "IVF_FLAT", "params": {"nlist": 64}})
                    col.load()
                    return col
                
                def mil_ins(col, embs, texts):
                    ids = list(range(len(embs)))
                    def truncate_bytes(s, max_bytes=65530):
                        b = s.encode('utf-8')
                        return b[:max_bytes].decode('utf-8', errors='ignore') if len(b) > max_bytes else s
                    truncated_texts = [truncate_bytes(t) for t in texts]
                    col.insert([ids, embs, truncated_texts])
                    col.flush()
                
                def mil_srch(col, q_emb, limit, q_text=None):
                    if evaluator.hybrid and q_text and bm25_index:
                        res = col.search([q_emb], "vec", {"metric_type": "COSINE", "params": {"nprobe": 10}}, limit=50, output_fields=["text"])
                        vec_hits = [{"text": h.entity.get("text"), "score": h.distance} for h in res[0]]
                        bm25_scores = bm25_index.get_scores(q_text)
                        top_bm25_idx = np.argsort(bm25_scores)[::-1][:50]
                        text_hits = [{"text": corpus[idx], "score": bm25_scores[idx]} for idx in top_bm25_idx if bm25_scores[idx] > 0]
                        alpha = get_hybrid_weight(dom)
                        return run_score_fusion(vec_hits, text_hits, alpha, limit=limit)
                    else:
                        res = col.search([q_emb], "vec", {"metric_type": "COSINE", "params": {"nprobe": 10}}, limit=limit, output_fields=["text"])
                        return [{"text": h.entity.get("text")} for h in res[0]]
                
                def mil_cleanup(col):
                    try: utility.drop_collection(coll_name)
                    except: pass
                    
                res = evaluator.run_eval("Milvus", corpus, queries, embeddings, q_embeddings, mil_setup, mil_ins, mil_srch, mil_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Milvus evaluation failed: {e}")
 
        # ── WEAVIATE ──
        if (not target_dbs or "weaviate" in target_dbs) and WEAVIATE_AVAILABLE:
            try:
                coll_name = f"RagStandards{dom.capitalize()}"
                
                if hasattr(weaviate, "connect_to_local"):
                    import weaviate.classes.config as wvc
                    
                    def weav_setup():
                        client = weaviate.connect_to_local(port=8080, grpc_port=50052)
                        try:
                            client.collections.delete(coll_name)
                        except:
                            pass
                        collection = client.collections.create(
                            name=coll_name,
                            vectorizer_config=None,
                            properties=[
                                wvc.Property(name="text", data_type=wvc.DataType.TEXT)
                            ],
                            vector_index_config=wvc.Configure.VectorIndex.hnsw(
                                distance_metric=wvc.VectorDistances.COSINE
                            )
                        )
                        client._cached_collection = collection
                        return client
                    
                    def weav_ins(client, embs, texts):
                        collection = client._cached_collection
                        with collection.batch.dynamic() as batch:
                            for emb, text in zip(embs, texts):
                                batch.add_object(
                                    properties={"text": text},
                                    vector=emb
                                )
                    
                    def weav_srch(client, q_emb, limit, q_text=None):
                        collection = client._cached_collection
                        if evaluator.hybrid and q_text:
                            alpha = get_hybrid_weight(dom)
                            res = collection.query.hybrid(
                                query=q_text,
                                vector=q_emb,
                                alpha=alpha,
                                limit=limit,
                                return_properties=["text"]
                            )
                        else:
                            res = collection.query.near_vector(
                                near_vector=q_emb,
                                limit=limit,
                                return_properties=["text"]
                            )
                        return [{"text": obj.properties["text"]} for obj in res.objects if obj]
                    
                    def weav_cleanup(client):
                        try:
                            client.collections.delete(coll_name)
                        except:
                            pass
                        client.close()
                        
                else:
                    def weav_setup():
                        client = weaviate.Client(url="http://localhost:8080")
                        if client.schema.exists(coll_name):
                            client.schema.delete_class(coll_name)
                        client.schema.create_class({
                            "class": coll_name,
                            "vectorizer": "none",
                            "vectorIndexConfig": {"distance": "cosine"},
                            "properties": [
                                {"name": "text", "dataType": ["text"]}
                            ]
                        })
                        return client
                    
                    def weav_ins(client, embs, texts):
                        client.batch.configure(batch_size=100)
                        with client.batch as batch:
                            for i, (emb, text) in enumerate(zip(embs, texts)):
                                batch.add_data_object(
                                    data_object={"text": text},
                                    class_name=coll_name,
                                    vector=emb
                                )
                    
                    def weav_srch(client, q_emb, limit, q_text=None):
                        if evaluator.hybrid and q_text:
                            alpha = get_hybrid_weight(dom)
                            res = client.query.get(coll_name, ["text"]).with_hybrid(
                                query=q_text,
                                vector=q_emb,
                                alpha=alpha
                            ).with_limit(limit).do()
                        else:
                            res = client.query.get(coll_name, ["text"]).with_near_vector({"vector": q_emb}).with_limit(limit).do()
                        if "data" in res and "Get" in res["data"] and coll_name in res["data"]["Get"] and res["data"]["Get"][coll_name]:
                            return [{"text": obj["text"]} for obj in res["data"]["Get"][coll_name] if obj]
                        return []
                    
                    def weav_cleanup(client):
                        try: client.schema.delete_class(coll_name)
                        except: pass
                    
                res = evaluator.run_eval("Weaviate", corpus, queries, embeddings, q_embeddings, weav_setup, weav_ins, weav_srch, weav_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Weaviate evaluation failed: {e}")
                
        # Cleanup Hyperspace collection for this domain if it was created during the sweep
        if hs_collection_created:
            try:
                client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=1)
                client.delete_collection(coll_name_hs)
                client.close()
                print(f"🧹 Cleaned up Hyperspace collection '{coll_name_hs}'")
            except Exception as e:
                print(f"⚠️ Error cleaning up Hyperspace collection '{coll_name_hs}': {e}")
                
        if results:
            domain_results[dom] = results
            
    if domain_results:
        evaluator.generate_html_report(domain_results)
    else:
        print("\n❌ No databases were successfully evaluated across any domains. Please check database server statuses.")


if __name__ == "__main__":
    main()
