#!/usr/bin/env python3
import os
import sys
import json
import numpy as np
import argparse
from typing import List, Dict, Tuple
from tqdm import tqdm

# Ensure we can import from legacy the core calculations
_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import run_benchmark_legacy as legacy
from db_plugins.registry import load_plugins, select_plugins
from plugin_runtime import BenchmarkContext, Result

def load_cached_vectors(cache_dir):
    print(f"📂 Loading cached vectors from {cache_dir}...")
    
    # Qwen (Euclidean) - kept as float16 to save time
    qwen_p = np.fromfile(os.path.join(cache_dir, "qwen_p_embs.npy"), dtype='float16').reshape(-1, 1024).astype('float32')
    
    # YAR (Lorentz 129D) - Ultra-High Precision float64
    yar_p_full = np.fromfile(os.path.join(cache_dir, "yar_p_embs.npy"), dtype='float64').reshape(-1, 129)
    
    # --- Data Sanitation ---
    print("   [Fix] Cleaning vectors (NaN/Inf check)...")
    qwen_p = np.nan_to_num(qwen_p, nan=0.0, posinf=1e4, neginf=-1e4)
    # YAR is now in f64, so it should be much cleaner, but we keep it safe
    yar_p_full = np.nan_to_num(yar_p_full, nan=0.0, posinf=1e9, neginf=-1e9)
    
    num_samples = len(qwen_p)
    print(f"✅ Detected {num_samples:,} samples in cache.")
    
    return qwen_p, yar_p_full, num_samples

def calculate_lorentz_gt(q_vecs, doc_vecs, doc_ids, k=10):
    """Exact Lorentz nearest neighbor search using Minkowski inner product"""
    # Distance d(u,v) = arccosh(-B(u,v)) where B(u,v) = -u0*v0 + sum(ui*vi)
    # Minimizing distance == Maximizing B(u,v) (since arccosh is monotonic 
    # and B is always <= -1)
    gt = []
    for q in tqdm(q_vecs, desc="Brute-force GT (Lorentz)"):
        # Minkowski Inner Product: -q0*d0 + q_spatial @ d_spatial.T
        minkowski_ip = -q[0] * doc_vecs[:, 0] + (doc_vecs[:, 1:] @ q[1:])
        # Top-K indices with LARGEST minkowski_ip
        top_idx = np.argpartition(-minkowski_ip, k - 1)[:k]
        top_idx = top_idx[np.argsort(-minkowski_ip[top_idx])]
        gt.append([doc_ids[idx] for idx in top_idx])
    return gt

def write_ultimate_story(final_results: List[Result], num_docs: int, num_queries: int):
    path = "BENCHMARK_YAR_ULTIMATE.md"
    with open(path, "w") as f:
        f.write("# 📐 YAR vs Qwen3: Ultimate VectorDB Benchmark\n\n")
        f.write(f"Testing on **MS MARCO 50K** subset  \n")
        f.write(f"**Doc count:** {num_docs:,}  \n")
        f.write(f"**Query count:** {num_queries:,}  \n\n")
        
        f.write("| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall@10 | MRR | NDCG | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        
        # Sort so Hyperspace Lorentz is at the top
        sorted_res = sorted(final_results, key=lambda x: (0 if "Lorentz" in x.metric else 1, -x.search_qps))
        
        for r in sorted_res:
            if r.status == "Success":
                f.write(f"| **{r.database}** | {r.dimension} | {r.geometry} | {r.metric} | {r.insert_qps:,.0f} | {r.search_qps:,.0f} | {r.p99:.2f}ms | {r.recall:.1%} | {r.mrr:.2f} | {r.ndcg:.2f} | {r.disk_usage} |\n")
            else:
                f.write(f"| **{r.database}** | {r.dimension} | Error | {r.metric} | - | - | - | - | - | - | {r.status} |\n")
    print(f"📝 Markdown report saved to: {path}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cache_dir", type=str, required=True)
    parser.add_argument("--target_db", type=str, default=None)
    parser.add_argument("--query_limit", type=int, default=1000)
    args = parser.parse_args()

    # 1. Load Vectors
    qwen_vecs, yar_vecs_full, total_loaded = load_cached_vectors(args.cache_dir)
    
    # 2. Sequential Partitioning
    actual_q = min(args.query_limit, total_loaded // 10)
    actual_d = total_loaded - actual_q
    
    doc_ids = [str(i) for i in range(total_loaded)]
    test_query_ids = doc_ids[:actual_q]
    corpus_ids = doc_ids[actual_q:]
    
    q_vecs_euc = qwen_vecs[:actual_q]
    doc_vecs_euc = qwen_vecs[actual_q:]
    q_vecs_yar = yar_vecs_full[:actual_q]
    doc_vecs_yar = yar_vecs_full[actual_q:]

    q_vecs_poincare = q_vecs_yar[:, 1:]
    doc_vecs_poincare = doc_vecs_yar[:, 1:]

    # 3. Build Config
    cfg = legacy.Config()
    cfg.dataset_name = "Cached_YAR_Comparison_MSMARCO"
    cfg.doc_limit = actual_d
    cfg.query_limit = actual_q
    
    # 4. Brute-force GT Calculation
    print("\n🧮 Calculating Ground Truth...")
    # Euclidean GT
    math_gt_euc = legacy.calculate_brute_force_gt(q_vecs_euc, doc_vecs_euc, corpus_ids, k=10, metric="cosine")
    
    # Real Lorentz GT
    math_gt_lorentz = calculate_lorentz_gt(q_vecs_yar, doc_vecs_yar, corpus_ids, k=10)
    
    valid_qrels_euc = {tid: math_gt_euc[i] for i, tid in enumerate(test_query_ids)}
    valid_qrels_lorentz = {tid: math_gt_lorentz[i] for i, tid in enumerate(test_query_ids)}

    # 5. Run Plugins
    plugins = load_plugins()
    selected_plugins = select_plugins(plugins, args.target_db)
    final_results = []
    
    # Shared Context
    common_ctx = BenchmarkContext(
        cfg=cfg, docs=[""]*actual_d, doc_ids=corpus_ids,
        test_queries=[""]*actual_q, test_query_ids=test_query_ids,
        valid_qrels=valid_qrels_euc, # Defaults to Euclidean for baseline comparison
        doc_vecs_euc=doc_vecs_euc, q_vecs_euc=q_vecs_euc,
        doc_vecs_hyp=doc_vecs_poincare, q_vecs_hyp=q_vecs_poincare,
        math_gt_euc=math_gt_euc, math_gt_hyp=math_gt_lorentz # use Lorentz as Hyperbolic GT
    )

    for plugin in selected_plugins:
        try:
            if plugin.name != "hyper":
                if not plugin.is_available(): continue
                print(f"\n🚀 Running {plugin.name.upper()} [Euclidean] (Qwen-1024D)...")
                cfg.HYPER_MODE = "euclidean"; cfg.dim_base = 1024
                final_results.append(plugin.run(common_ctx))
            else:
                if not plugin.is_available(): continue
                
                # Mode A: Euclidean
                print("\n🚀 Running Hyperspace [EUCLIDEAN] (Qwen-1024D)...")
                cfg.HYPER_MODE = "euclidean"; cfg.dim_base = 1024
                # use euc qrels for euc run
                common_ctx.valid_qrels = valid_qrels_euc
                final_results.append(plugin.run(common_ctx))

                # Mode C: Lorentz
                print("\n🚀 Running Hyperspace [LORENTZ] (YAR-129D)...")
                cfg.HYPER_MODE = "lorentz"; cfg.dim_hyp = 129
                # update qrels to lorentz for lorentz run
                common_ctx.valid_qrels = valid_qrels_lorentz
                
                lorentz_ctx = BenchmarkContext(
                    cfg=cfg, docs=common_ctx.docs, doc_ids=common_ctx.doc_ids,
                    test_queries=common_ctx.test_queries, test_query_ids=common_ctx.test_query_ids,
                    valid_qrels=valid_qrels_lorentz, doc_vecs_euc=doc_vecs_euc, q_vecs_euc=q_vecs_euc,
                    doc_vecs_hyp=doc_vecs_yar, q_vecs_hyp=q_vecs_yar,
                    math_gt_euc=math_gt_euc, math_gt_hyp=math_gt_lorentz
                )
                final_results.append(plugin.run(lorentz_ctx))
        except Exception as e:
            print(f"❌ Error in {plugin.name}: {e}")

    # 6. Reporting
    legacy.print_table(final_results)
    legacy.generate_benchmark_html_report(final_results, "YAR_VS_QWEN_50K", actual_d, actual_q)
    write_ultimate_story(final_results, actual_d, actual_q)
    
if __name__ == "__main__":
    main()
