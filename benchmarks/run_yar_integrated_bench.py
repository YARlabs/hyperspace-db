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

def load_cached_vectors(cache_dir, model_name, dtype='float32'):
    """Load vectors and immediately sterilize them for numerical stability"""
    q_path = os.path.join(cache_dir, f"{model_name}_q_embs.npy")
    p_path = os.path.join(cache_dir, f"{model_name}_p_embs.npy")
    
    dim = 129 if model_name == "yar" else 1024
    
    # Load raw
    q_vecs = np.fromfile(q_path, dtype=dtype).reshape(-1, dim).astype(np.float64)
    p_vecs = np.fromfile(p_path, dtype=dtype).reshape(-1, dim).astype(np.float64)
    
    # FORCED STERILIZATION: Remove any non-finite values before they touch the math engine
    q_vecs = np.nan_to_num(q_vecs, nan=0.0, posinf=0.0, neginf=0.0)
    p_vecs = np.nan_to_num(p_vecs, nan=0.0, posinf=0.0, neginf=0.0)
    
    # Pre-normalize Qwen for Cosine to match legacy exactly
    if model_name == "qwen":
        norms = np.linalg.norm(p_vecs, axis=1, keepdims=True)
        norms[norms < 1e-12] = 1.0
        p_vecs = p_vecs / norms
        
    return q_vecs, p_vecs

def calculate_lorentz_gt(q_vecs, doc_vecs, doc_ids, k=10):
    """Exact Lorentz nearest neighbor search with total silence and precision"""
    gt = []
    # Already in f64 from loader, but ensure it here too
    q_vecs = q_vecs.astype(np.float64)
    doc_vecs = doc_vecs.astype(np.float64)
    
    # Lorentz search batching for speed and safety
    spatial_docs = doc_vecs[:, 1:]
    t_docs = doc_vecs[:, 0]
    
    for q in tqdm(q_vecs, desc="Brute-force GT (Lorentz)"):
        # Minkowski IP formula: -q0*d0 + <q_x, d_x>
        # Add tiny eps only if we had to handle arccosh (here we don't need it for ranking)
        minkowski_ip = (-q[0] * t_docs) + (spatial_docs @ q[1:])
        
        # Ranking is stable here. Closest point = point with largest Minkowski IP.
        top_idx = np.argpartition(-minkowski_ip, k - 1)[:k]
        top_idx = top_idx[np.argsort(-minkowski_ip[top_idx])]
        gt.append([str(doc_ids[idx]) for idx in top_idx])
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

    # 1. Load Vectors correctly
    try:
        q_vecs_euc, doc_vecs_euc = load_cached_vectors(args.cache_dir, "qwen", dtype='float32')
        print("✅ Loaded Qwen vectors (float32)")
    except Exception:
        q_vecs_euc, doc_vecs_euc = load_cached_vectors(args.cache_dir, "qwen", dtype='float16')
        print("✅ Loaded Qwen vectors (float16 fallback)")
        
    q_vecs_yar, doc_vecs_yar = load_cached_vectors(args.cache_dir, "yar", dtype='float64')
    print("✅ Loaded YAR vectors (float64)")
    
    # 2. Preparation
    actual_q = min(len(q_vecs_euc), args.query_limit)
    actual_d = len(doc_vecs_euc)
    
    print(f"📊 Dataset: {actual_q} queries (limited), {actual_d} documents.")
    
    q_vecs_euc = q_vecs_euc[:actual_q]
    q_vecs_yar = q_vecs_yar[:actual_q]
    
    # INTERNAL IDS: Use integers for database insertion logic, but strings for results comparison
    corpus_ids = [str(i) for i in range(actual_d)]
    test_query_ids = [str(i) for i in range(actual_q)]

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
                
                # Mode A: Cosine (for Qwen)
                print("\n🚀 Running Hyperspace [COSINE] (Qwen-1024D)...")
                cfg.HYPER_MODE = "cosine"; cfg.dim_base = 1024
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
