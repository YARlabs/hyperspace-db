#!/usr/bin/env python3
"""
Comprehensive Quantization & Reranking Parameter Sweep for HyperspaceDB v4.0.0
Tests all quantization levels:
  - 'none'        (FP32/FP64 Full Precision)
  - 'medium'      (ScalarI8, 8-bit)
  - 'medium_plus' (ScalarI4, 4-bit Block-wise)
  - 'turbo'       (TurboQuant 4-bit Spherical)
  - 'ternary'     (BitNet 1.58-bit {-1, 0, +1} Dual Bitmask)
  - 'opq'         (Optimized Product Quantization, 64-byte FWHT Rotated ADC)
  - 'pq'          (Naive Product Quantization, 64-byte ADC)
  - 'extreme'     (Binary ADC 1-bit)
  - 'rabitq'      (RaBitQ 1-bit + unbiased estimator, popcount bit-planes)   [v4.2.5+]
  - 'pq4'         (4-bit PQ fast-scan, L1-resident u8 LUT)                  [v4.2.5+]

Code-based modes (pq, opq, pq4, rabitq) are calibrated before searching: after ingest
the script calls RebuildIndex, which trains codebooks / the RaBitQ centroid on the
exact vectors (same protocol for every engine version, so comparisons stay fair).

Before/after comparisons:
  HS_BENCH_SERVER_BIN=/path/to/hyperspace-server   engine binary under test
  HS_BENCH_LABEL="v4.2.4 (before)"                 label used in the report
  HS_BENCH_REPORT=/path/report.md                  Markdown output path
  HS_BENCH_RESULT=/path/results.json               JSON output path

Along with:
  - Reranking variants (Disabled, Server Rerank 4x, 8x, 16x)
  - Two-Pass Exact Raw Vector Rerank (Top-40, Top-80)
  - Graph traversal depth (ef_search 100 vs 400)

Metrics measured:
  - Ingest Time & Ingest Throughput (vec/s)
  - Index RAM Footprint & Theoretical Vector Size
  - Search Throughput (QPS)
  - Latency: Mean, P50, P95, P99 (ms)
  - Accuracy: Recall@10 vs Exact Brute-Force Ground Truth (%)
"""

import os
import sys
import time
import json
import shutil
import socket
import subprocess
from typing import Dict, List, Tuple, Any, Optional
import numpy as np

# Ensure Python SDK is on path
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SDK_DIR = os.environ.get("HS_BENCH_SDK_DIR", os.path.join(BASE_DIR, "sdks", "python"))
sys.path.insert(0, SDK_DIR)

from hyperspace import HyperspaceClient

SERVER_BIN = os.path.join(BASE_DIR, "target", "release", "hyperspace-server-v4")
if not os.path.exists(SERVER_BIN):
    _alt = os.path.join(BASE_DIR, "target", "release", "hyperspace-server")
    if os.path.exists(_alt):
        SERVER_BIN = _alt
SERVER_BIN = os.environ.get("HS_BENCH_SERVER_BIN", SERVER_BIN)
ENGINE_LABEL = os.environ.get("HS_BENCH_LABEL", "v4.2.5")
DATA_DIR   = os.path.join(BASE_DIR, "data_quant_sweep")
REPORT_MD  = os.environ.get("HS_BENCH_REPORT", os.path.join(BASE_DIR, "BENCHMARK_QUANTIZATION_SWEEP.md"))
RESULT_JSON = os.environ.get("HS_BENCH_RESULT", os.path.join(BASE_DIR, "benchmarks", "quantization_sweep_results.json"))

# Modes whose codebooks / centroid must be trained on real data before searching.
CALIBRATED_MODES = {"pq", "opq", "pq4", "rabitq"}
CALIBRATE = os.environ.get("HS_BENCH_CALIBRATE", "1") not in ("0", "false", "no", "off")

API_KEY = "I_LOVE_HYPERSPACEDB"
PORT_GRPC = int(os.environ.get("HS_BENCH_GRPC_PORT", "50051"))
PORT_HTTP = int(os.environ.get("HS_BENCH_HTTP_PORT", "50050"))

# Benchmark Scale
N_VECTORS = 20_000
DIMENSION = 1024
N_CLUSTERS = 50
N_QUERIES = 200
TOP_K = 10
INGEST_BATCH = 1000

def is_port_open(port: int) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(('127.0.0.1', port)) == 0

def wait_for_server(port: int = PORT_GRPC, timeout: float = 20.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if is_port_open(port):
            return True
        time.sleep(0.3)
    return False

def generate_clustered_dataset(n: int, dim: int, num_clusters: int, seed: int = 42) -> np.ndarray:
    """Generate normalized clustered embeddings mimicking real semantic manifolds."""
    rng = np.random.default_rng(seed)
    centroids = rng.standard_normal((num_clusters, dim)).astype(np.float32)
    centroids /= np.maximum(np.linalg.norm(centroids, axis=1, keepdims=True), 1e-9)

    # Assign vectors to clusters
    cluster_ids = rng.integers(0, num_clusters, size=n)
    noise = rng.normal(0, 0.08, size=(n, dim)).astype(np.float32)
    vectors = centroids[cluster_ids] + noise
    vectors /= np.maximum(np.linalg.norm(vectors, axis=1, keepdims=True), 1e-9)
    return vectors

def generate_queries(centroids: np.ndarray, n_queries: int, dim: int, seed: int = 999) -> np.ndarray:
    """Generate realistic queries around cluster manifolds."""
    rng = np.random.default_rng(seed)
    num_clusters = centroids.shape[0]
    c_indices = rng.integers(0, num_clusters, size=n_queries)
    noise = rng.normal(0, 0.07, size=(n_queries, dim)).astype(np.float32)
    queries = centroids[c_indices] + noise
    queries /= np.maximum(np.linalg.norm(queries, axis=1, keepdims=True), 1e-9)
    return queries

def compute_ground_truth(vectors: np.ndarray, queries: np.ndarray, top_k: int = 10) -> List[List[int]]:
    """Compute exact top_k nearest neighbors using Cosine similarity (Brute-Force)."""
    print(f"📊 Computing exact Brute-Force Ground Truth for {len(queries)} queries across {len(vectors):,} vectors...")
    t0 = time.time()
    # Cosine distance = 1 - dot_product for unit-normalized vectors
    # Best matches = largest dot product
    sims = np.dot(queries, vectors.T)
    # Get top_k highest dot products
    gt = []
    for row in sims:
        top_indices = np.argsort(-row)[:top_k]
        gt.append(top_indices.tolist())
    print(f"✅ Ground truth computed in {time.time() - t0:.2f}s")
    return gt

def calculate_recall(ground_truth: List[List[int]], results: List[List[int]], top_k: int = 10) -> float:
    if not results or not ground_truth:
        return 0.0
    recalls = []
    for gt_row, res_row in zip(ground_truth, results):
        if not gt_row:
            continue
        gt_set = set(gt_row[:top_k])
        res_set = set(res_row[:top_k])
        recalls.append(len(gt_set.intersection(res_set)) / float(top_k))
    return float(np.mean(recalls)) * 100.0 if recalls else 0.0

def get_process_rss_mb(pid: int) -> float:
    try:
        out = subprocess.check_output(["ps", "-o", "rss=", "-p", str(pid)]).decode().strip()
        if out:
            return float(out) / 1024.0
    except Exception:
        pass
    return 0.0

_RUN_COUNTER = 0

def run_single_config(
    cfg: Dict[str, Any],
    vectors: np.ndarray,
    queries: np.ndarray,
    ground_truth: List[List[int]]
) -> Dict[str, Any]:
    global PORT_GRPC, PORT_HTTP, _RUN_COUNTER
    name = cfg["name"]
    quant = cfg["quant"]
    rerank = cfg.get("rerank", False)
    oversample = cfg.get("oversample", 4)
    ef_search = cfg.get("ef_search", 100)
    is_two_pass_raw = cfg.get("two_pass_raw", False)
    raw_fetch_k = cfg.get("raw_fetch_k", 40)

    print(f"\n======================================================================")
    print(f"▶ [{name.upper()}] Quant: {quant} | Rerank: {rerank} (x{oversample}) | ef_search: {ef_search}")
    print(f"  Description: {cfg.get('desc', '')}")
    print(f"======================================================================")

    # 1. Clean data directory
    if os.path.exists(DATA_DIR):
        shutil.rmtree(DATA_DIR, ignore_errors=True)
    os.makedirs(DATA_DIR, exist_ok=True)

    # 2. Setup environment
    env = os.environ.copy()
    env["HS_DATA_DIR"] = DATA_DIR
    env["HS_V4_ENGINE"] = "1"
    env["HS_QUANTIZATION_LEVEL"] = quant
    env["HS_RERANK_ENABLED"] = "1" if rerank else "0"
    env["HS_RERANK_OVERSAMPLE"] = str(oversample)
    env["HS_HNSW_EF_SEARCH"] = str(ef_search)
    env["HS_HNSW_M"] = "64"
    env["HS_HNSW_EF_CONSTRUCT"] = "400"
    env["HS_PORT"] = str(PORT_GRPC)
    env["HS_HTTP_PORT"] = str(PORT_HTTP)
    env["RUST_LOG"] = "warn"

    # Fresh port pair per config so a slow-to-exit previous server never blocks the next run.
    base_grpc = int(os.environ.get("HS_BENCH_GRPC_PORT", "50051"))
    PORT_GRPC = base_grpc + 2 * _RUN_COUNTER
    PORT_HTTP = PORT_GRPC - 1
    _RUN_COUNTER += 1

    # The previous config's server may still be releasing the port.
    deadline = time.time() + 20.0
    while is_port_open(PORT_GRPC) and time.time() < deadline:
        time.sleep(0.5)
    if is_port_open(PORT_GRPC):
        raise RuntimeError(f"Port {PORT_GRPC} is already in use (another server running?) — set HS_BENCH_GRPC_PORT")

    proc = subprocess.Popen(
        [SERVER_BIN, "--port", str(PORT_GRPC), "--http-port", str(PORT_HTTP)],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )

    try:
        if not wait_for_server(PORT_GRPC):
            raise RuntimeError(f"Server failed to bind on port {PORT_GRPC}")

        client = HyperspaceClient(f"localhost:{PORT_GRPC}", api_key=API_KEY)
        col_name = f"col_{name}"

        # Create collection
        client.create_collection(col_name, dimension=DIMENSION, metric="cosine", quantization=quant)

        # Ingestion
        print(f"   [ingest] Ingesting {N_VECTORS:,} vectors (dim={DIMENSION})...", flush=True)
        t_ingest_start = time.perf_counter()
        for i in range(0, N_VECTORS, INGEST_BATCH):
            batch = vectors[i : i + INGEST_BATCH]
            ids = list(range(i, i + len(batch)))
            client.batch_insert(batch.tolist(), ids, collection=col_name)
        ingest_time = time.perf_counter() - t_ingest_start
        ingest_qps = N_VECTORS / max(ingest_time, 1e-6)
        print(f"   [ingest] Done in {ingest_time:.2f}s ({ingest_qps:,.0f} vec/s)")

        # Wait for indexing queue to drain
        t_drain_start = time.perf_counter()
        while time.perf_counter() - t_drain_start < 25.0:
            stats = client.get_collection_stats(col_name)
            if stats:
                count = stats.get("count", 0)
                queue = stats.get("indexing_queue", 0)
                if count >= N_VECTORS and queue == 0:
                    break
            time.sleep(0.3)
        drain_time = time.perf_counter() - t_drain_start

        # Calibrate code-based quantizers (trains codebooks / centroid on exact vectors).
        calibrate_time = 0.0
        if CALIBRATE and quant in CALIBRATED_MODES:
            t_cal = time.perf_counter()
            ok = client.rebuild_index(col_name)
            calibrate_time = time.perf_counter() - t_cal
            print(f"   [calibrate] RebuildIndex {'ok' if ok else 'FAILED'} in {calibrate_time:.2f}s")

        # Measure RAM
        rss_mb = get_process_rss_mb(proc.pid)
        col_stats = client.get_collection_stats(col_name)
        reported_ram = col_stats.get("usage", {}).get("ram_bytes", 0) if col_stats else 0
        reported_ram_mb = reported_ram / (1024 * 1024)

        # Theoretical bytes per vector
        theor_bytes = {
            "none": DIMENSION * 4 + 4,         # 4,100 B
            "medium": DIMENSION + 4,           # 1,028 B
            "medium_plus": (DIMENSION // 16) * 12 + 4, # 772 B
            "turbo": (DIMENSION // 2) + 8,      # 520 B
            "ternary": (DIMENSION // 8) * 2 + 8 + 4, # 268 B (128B nz + 128B sign + 4B scale + 4B alpha + 4B ID)
            "opq": (DIMENSION // 16) + 4,       # 68 B (64 subvectors = 64 bytes + 4B ID)
            "pq": (DIMENSION // 16) + 4,        # 68 B
            "extreme": (DIMENSION // 8) + 4,    # 132 B
            "rabitq": ((DIMENSION + 63) // 64) * 8 + 8 + 4,  # 140 B (bits + norm + ip + ID)
            "pq4": (DIMENSION // 4 + 1) // 2 + 4,              # 132 B (D/4 nibbles + ID)
        }.get(quant, 1024)
        theor_mb = (theor_bytes * N_VECTORS) / (1024 * 1024)

        # Warmup search
        client.search(queries[0].tolist(), top_k=TOP_K, collection=col_name)

        # Search benchmark
        latencies_ms = []
        retrieved_ids = []

        t_search_start = time.perf_counter()
        for q in queries:
            q_list = q.tolist()
            t0 = time.perf_counter()
            if is_two_pass_raw:
                # Step 1: candidate fetch from server (e.g. 40 or 80)
                cand_res = client.search(q_list, top_k=raw_fetch_k, collection=col_name)
                cand_ids = [hit["id"] for hit in cand_res]
                # Step 2: exact re-ranking against raw vector coordinates
                if cand_ids:
                    cand_matrix = vectors[cand_ids]
                    sims = np.dot(cand_matrix, q)
                    top_local = np.argsort(-sims)[:TOP_K]
                    final_ids = [cand_ids[idx] for idx in top_local]
                else:
                    final_ids = []
            else:
                res = client.search(q_list, top_k=TOP_K, collection=col_name)
                final_ids = [hit["id"] for hit in res]

            t1 = time.perf_counter()
            latencies_ms.append((t1 - t0) * 1000.0)
            retrieved_ids.append(final_ids)

        total_search_time = time.perf_counter() - t_search_start
        search_qps = len(queries) / max(total_search_time, 1e-6)

        mean_lat = float(np.mean(latencies_ms))
        p50_lat = float(np.percentile(latencies_ms, 50))
        p95_lat = float(np.percentile(latencies_ms, 95))
        p99_lat = float(np.percentile(latencies_ms, 99))

        recall_at_10 = calculate_recall(ground_truth, retrieved_ids, top_k=TOP_K)

        print(f"   [search] QPS: {search_qps:,.0f} | Recall@10: {recall_at_10:.2f}% | Latency (avg/p95): {mean_lat:.2f}ms / {p95_lat:.2f}ms")
        print(f"   [memory] Process RSS: {rss_mb:.1f} MB | Vector Index Theor: {theor_mb:.2f} MB ({theor_bytes} B/vec)")

        client.close()
        return {
            "name": name,
            "desc": cfg.get("desc", ""),
            "quant": quant,
            "rerank": rerank,
            "oversample": oversample,
            "ef_search": ef_search,
            "two_pass_raw": is_two_pass_raw,
            "raw_fetch_k": raw_fetch_k if is_two_pass_raw else None,
            "ingest_time_s": ingest_time,
            "ingest_qps": ingest_qps,
            "drain_time_s": drain_time,
            "calibrate_time_s": calibrate_time,
            "search_qps": search_qps,
            "mean_latency_ms": mean_lat,
            "p50_latency_ms": p50_lat,
            "p95_latency_ms": p95_lat,
            "p99_latency_ms": p99_lat,
            "recall_at_10": recall_at_10,
            "rss_mb": rss_mb,
            "reported_ram_mb": reported_ram_mb,
            "theor_bytes_per_vec": theor_bytes,
            "theor_mb": theor_mb,
        }
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except Exception:
            proc.kill()
        if os.path.exists(DATA_DIR):
            shutil.rmtree(DATA_DIR, ignore_errors=True)

def generate_markdown_report(results: List[Dict[str, Any]], baseline_res: Dict[str, Any]):
    print(f"\n📝 Generating comprehensive Markdown report: {REPORT_MD}...")

    base_theor_mb = baseline_res.get("theor_mb", 78.12)
    base_qps = baseline_res.get("search_qps", 1.0)
    base_recall = baseline_res.get("recall_at_10", 100.0)

    with open(REPORT_MD, "w", encoding="utf-8") as f:
        f.write(f"# HyperspaceDB {ENGINE_LABEL} Quantization & Rerank Sweep Benchmark Report\n\n")
        f.write("> **Environment & Test Parameters**\n")
        f.write(f"> - **Engine**: HyperspaceDB {ENGINE_LABEL} (`{os.path.basename(SERVER_BIN)}`)\n")
        f.write(f"> - **Code-mode calibration**: {'RebuildIndex after ingest (pq, opq, pq4, rabitq)' if CALIBRATE else 'disabled'}\n")
        f.write(f"> - **Dataset**: {N_VECTORS:,} vectors, {DIMENSION}D Dense Semantic Clustered Embedding ({N_CLUSTERS} clusters)\n")
        f.write(f"> - **Metric**: Cosine Distance (Normalized Euclidean)\n")
        f.write(f"> - **Queries**: {N_QUERIES:,} queries evaluated against Exact Brute-Force Ground Truth\n")
        f.write(f"> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search={baseline_res.get('ef_search', 100)}\n")
        f.write(f"> - **Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")

        f.write("## 1. Executive Summary & Comparison Matrix\n\n")
        f.write("| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |\n")
        f.write("| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |\n")

        for r in results:
            comp_ratio = base_theor_mb / max(r["theor_mb"], 1e-4)
            if r["two_pass_raw"]:
                rerank_str = f"Client 2-Pass Raw (k={r['raw_fetch_k']})"
            elif r["rerank"]:
                rerank_str = f"Server ADC (x{r['oversample']})"
            else:
                rerank_str = "None"

            f.write(f"| **{r['name']}** | `{r['quant']}` | {rerank_str} | **{r['recall_at_10']:.2f}%** | {r['search_qps']:,.0f} | {r['mean_latency_ms']:.2f}ms | {r['p95_latency_ms']:.2f}ms | {r['theor_mb']:.2f} MB | **{comp_ratio:.1f}×** |\n")

        f.write("\n---\n\n")

        f.write("## 2. Ingestion & Storage Footprint Analysis\n\n")
        f.write(f"| Configuration | Quant Mode | Bytes / Vec | {N_VECTORS:,} Vecs RAM | Ingest Time | Ingest QPS | Process RSS |\n")
        f.write("| :--- | :--- | :---: | :---: | :---: | :---: | :---: |\n")
        for r in results:
            f.write(f"| **{r['name']}** | `{r['quant']}` | {r['theor_bytes_per_vec']} B | {r['theor_mb']:.2f} MB | {r['ingest_time_s']:.2f}s | {r['ingest_qps']:,.0f} v/s | {r['rss_mb']:.1f} MB |\n")

        f.write("\n---\n\n")

        f.write("## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking\n\n")
        f.write("Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).\n\n")

        extreme_runs = [r for r in results if r["quant"] == "extreme"]
        if extreme_runs:
            f.write("| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |\n")
            f.write("| :--- | :---: | :---: | :---: | :---: | :--- |\n")
            for er in extreme_runs:
                if not er["rerank"] and not er["two_pass_raw"]:
                    char = "Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening."
                elif er["rerank"]:
                    char = f"HNSW fetches top {TOP_K * er['oversample']} candidates; server re-scores via ADC."
                elif er["two_pass_raw"]:
                    char = f"HNSW fetches top {er['raw_fetch_k']} candidates; exact raw vector cosine re-ranking."
                f.write(f"| **{er['name']}** | **{er['recall_at_10']:.2f}%** | {er['search_qps']:,.0f} | {er['mean_latency_ms']:.2f}ms | {er['p99_latency_ms']:.2f}ms | {char} |\n")

        f.write("\n### Key Takeaways for Extreme Mode:\n")
        f.write("1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.\n")
        f.write("2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.\n")
        f.write("\n---\n\n")

        f.write("## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)\n\n")
        f.write("OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.\n")
        f.write("By applying Fast Walsh-Hadamard Transform (FWHT $\\mathcal{O}(D \\log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.\n\n")

        opq_runs = [r for r in results if r["quant"] in ("opq", "pq")]
        if opq_runs:
            f.write("| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |\n")
            f.write("| :--- | :---: | :---: | :---: | :---: | :--- |\n")
            for opr in opq_runs:
                if not opr["rerank"] and not opr["two_pass_raw"]:
                    char = "Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec)."
                elif opr["rerank"]:
                    char = f"Server ADC candidate expansion (x{opr['oversample']})."
                elif opr["two_pass_raw"]:
                    char = f"Client raw vector cosine re-ranking (Top-{opr['raw_fetch_k']})."
                f.write(f"| **{opr['name']}** | **{opr['recall_at_10']:.2f}%** | {opr['search_qps']:,.0f} | {opr['mean_latency_ms']:.2f}ms | {opr['p99_latency_ms']:.2f}ms | {char} |\n")

        f.write("\n---\n\n")

        f.write("## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking\n\n")
        f.write("BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\\gamma = 0.67 \\cdot \\mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.\n")
        f.write("Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.\n\n")

        ternary_runs = [r for r in results if r["quant"] == "ternary"]
        if ternary_runs:
            f.write("| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |\n")
            f.write("| :--- | :---: | :---: | :---: | :---: | :--- |\n")
            for tr in ternary_runs:
                if not tr["rerank"] and not tr["two_pass_raw"]:
                    char = "Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount."
                elif tr["rerank"]:
                    char = f"Server ADC candidate expansion (x{tr['oversample']})."
                elif tr["two_pass_raw"]:
                    char = f"Client raw vector cosine re-ranking (Top-{tr['raw_fetch_k']})."
                f.write(f"| **{tr['name']}** | **{tr['recall_at_10']:.2f}%** | {tr['search_qps']:,.0f} | {tr['mean_latency_ms']:.2f}ms | {tr['p99_latency_ms']:.2f}ms | {char} |\n")

        f.write("\n---\n\n")

        f.write("## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)\n\n")
        f.write("RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor "
                "⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. "
                "PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.\n\n")
        new_runs = [r for r in results if r["quant"] in ("rabitq", "pq4")]
        if new_runs:
            f.write("| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |\n")
            f.write("| :--- | :---: | :---: | :---: | :---: | :---: |\n")
            for nr in new_runs:
                f.write(f"| **{nr['name']}** | **{nr['recall_at_10']:.2f}%** | {nr['search_qps']:,.0f} | {nr['mean_latency_ms']:.2f}ms | {nr['p99_latency_ms']:.2f}ms | {nr.get('calibrate_time_s', 0.0):.1f}s |\n")
        f.write("\n---\n\n")

        f.write("## 7. Architectural Recommendation Matrix\n\n")
        f.write("| Workload / Use Case | Recommended Config | Expected Recall | Compression | Trade-off Description |\n")
        f.write("| :--- | :--- | :---: | :---: | :--- |\n")
        f.write("| **Max Precision (Medical / Financial / Legal)** | `none` or `medium_rerank4` | 98.5 - 100.0% | 1× - 4× | Full precision guarantee, highest RAM consumption. |\n")
        f.write("| **Production Standard (RAG / Enterprise Search)** | `medium_plus_rerank4` | 94.0 - 97.5% | 5.3× | Ideal sweet spot between RAM savings and high accuracy. |\n")
        f.write("| **Balanced Edge / Cloud Reasoning (BitNet 1.58-Bit)** | `ternary_rerank4` or `ternary_twopass_raw40` | 95.0 - 99.2% | **15.3×** | 2-bit dual bitmask {-1, 0, +1} with SIMD popcount; 30.6× vs FP64. |\n")
        f.write("| **Ultra High-Density Vector Index** | `opq` or `opq_rerank4` | 89.0 - 98.5% | **60.3×** | 64B subvector codebook with FWHT rotation + 64KB L1 ADC LUT. |\n")
        f.write("| **Ultra High-Scale / Edge / RAM-Constrained** | `extreme` + Two-Pass Rerank | 96.0 - 99.5% | **31×** | Extreme 128B index in memory; exact vectors in disk/payload/S3. |\n")
        f.write("| **Volatile Context / Agent Memory** | `turbo` | 92.0 - 96.0% | 7.9× | Spherical Lloyd-Max preserving semantic clusters. |\n\n")

    print(f"✅ Benchmark report saved to {REPORT_MD}")

def parse_recall_gates(spec: Optional[str]) -> Dict[str, float]:
    """'a=97,b=95.5' -> {'a': 97.0, 'b': 95.5} (recall@10 in percent)."""
    gates: Dict[str, float] = {}
    if not spec:
        return gates
    for item in spec.split(","):
        item = item.strip()
        if not item:
            continue
        name, sep, value = item.partition("=")
        if not sep or not name.strip():
            raise SystemExit(f"--min-recall: expected 'config=percent', got {item!r}")
        try:
            gates[name.strip()] = float(value)
        except ValueError:
            raise SystemExit(f"--min-recall: bad threshold in {item!r}")
    return gates


def check_recall_gates(results: List[Dict[str, Any]], gates: Dict[str, float]) -> List[str]:
    """Returns one message per violated (or missing) gate."""
    by_name = {r["name"]: r for r in results}
    failures = []
    for name, minimum in gates.items():
        r = by_name.get(name)
        if r is None:
            failures.append(f"{name}: not run (check --filter)")
        elif r["recall_at_10"] < minimum:
            failures.append(f"{name}: recall@10 {r['recall_at_10']:.2f}% < {minimum:.2f}%")
    return failures


def main():
    global N_VECTORS, N_QUERIES

    import argparse
    parser = argparse.ArgumentParser(description="HyperspaceDB Quantization Sweep Benchmark")
    parser.add_argument("--filter", type=str, default=None, help="Filter config names or quantization modes (comma-separated, e.g. 'opq,none')")
    parser.add_argument("--n-vectors", type=int, default=N_VECTORS, help="Number of vectors to benchmark")
    parser.add_argument("--n-queries", type=int, default=N_QUERIES, help="Number of queries to run")
    parser.add_argument("--quick", action="store_true", help="Quick mode (5,000 vectors, 50 queries) for rapid validation")
    parser.add_argument("--exclude", type=str, default=None, help="Skip configs whose name or quant mode matches (comma-separated, e.g. 'rabitq,pq4' for engines that predate them)")
    parser.add_argument("--min-recall", type=str, default=None,
                        help="Recall gate: comma-separated 'config=min_recall_percent' pairs, e.g. "
                             "'none_ef100=97,medium_rerank4=95'. Exits with status 1 if a config "
                             "falls below its threshold (used by CI to catch recall regressions).")
    parser.add_argument("--no-report", action="store_true", help="Skip the Markdown report (CI / gate runs)")
    args = parser.parse_args()
    gates = parse_recall_gates(args.min_recall)

    if args.quick:
        N_VECTORS = 5000
        N_QUERIES = 50
    else:
        N_VECTORS = args.n_vectors
        N_QUERIES = args.n_queries

    print("======================================================================")
    print(f"  HYPERSPACE DB {ENGINE_LABEL} — FULL CONFIGURATION & QUANTIZATION SWEEP")
    print("======================================================================")
    print(f"Engine Binary:   {SERVER_BIN}")
    print(f"Vectors:         {N_VECTORS:,} vectors, dim={DIMENSION}")
    print(f"Queries:         {N_QUERIES} queries")
    print(f"Ground Truth:    Exact Brute-Force Cosine Distance")
    print("======================================================================")

    # 1. Define Configurations Matrix
    configs = [
        # Full Precision Baselines
        {
            "name": "none_ef100",
            "quant": "none",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "Full Precision FP32/64 Baseline (ef_search=100)"
        },
        {
            "name": "none_ef400",
            "quant": "none",
            "rerank": False,
            "oversample": 1,
            "ef_search": 400,
            "desc": "Full Precision FP32/64 Baseline (ef_search=400)"
        },

        # Medium (ScalarI8, 8-bit)
        {
            "name": "medium",
            "quant": "medium",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "ScalarI8 8-bit (No Rerank)"
        },
        {
            "name": "medium_rerank4",
            "quant": "medium",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "ScalarI8 8-bit (Server ADC Rerank 4x)"
        },

        # Medium Plus (ScalarI4, 4-bit)
        {
            "name": "medium_plus",
            "quant": "medium_plus",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "ScalarI4 4-bit Block-wise (No Rerank)"
        },
        {
            "name": "medium_plus_ef400",
            "quant": "medium_plus",
            "rerank": False,
            "oversample": 1,
            "ef_search": 400,
            "desc": "ScalarI4 4-bit (ef_search=400)"
        },
        {
            "name": "medium_plus_rerank4",
            "quant": "medium_plus",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "ScalarI4 4-bit (Server ADC Rerank 4x)"
        },
        {
            "name": "medium_plus_rerank8",
            "quant": "medium_plus",
            "rerank": True,
            "oversample": 8,
            "ef_search": 100,
            "desc": "ScalarI4 4-bit (Server ADC Rerank 8x)"
        },

        # Turbo (TurboQuant 4-bit spherical)
        {
            "name": "turbo",
            "quant": "turbo",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "TurboQuant 4-bit Spherical (No Rerank)"
        },
        {
            "name": "turbo_ef400",
            "quant": "turbo",
            "rerank": False,
            "oversample": 1,
            "ef_search": 400,
            "desc": "TurboQuant 4-bit (ef_search=400)"
        },
        {
            "name": "turbo_rerank4",
            "quant": "turbo",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "TurboQuant 4-bit (Server ADC Rerank 4x)"
        },
        {
            "name": "turbo_twopass_raw40",
            "quant": "turbo",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 40,
            "desc": "TurboQuant 4-bit + Client Two-Pass Exact Raw Rerank (Top-40)"
        },

        # Ternary (BitNet 1.58-bit {-1, 0, +1}, 15.3x compression vs FP32)
        {
            "name": "ternary",
            "quant": "ternary",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "BitNet 1.58-Bit Ternary {-1, 0, +1} (Single Pass, No Rerank)"
        },
        {
            "name": "ternary_ef400",
            "quant": "ternary",
            "rerank": False,
            "oversample": 1,
            "ef_search": 400,
            "desc": "BitNet 1.58-Bit Ternary (ef_search=400)"
        },
        {
            "name": "ternary_rerank4",
            "quant": "ternary",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "BitNet 1.58-Bit Ternary (Server ADC Rerank 4x)"
        },
        {
            "name": "ternary_rerank8",
            "quant": "ternary",
            "rerank": True,
            "oversample": 8,
            "ef_search": 100,
            "desc": "BitNet 1.58-Bit Ternary (Server ADC Rerank 8x)"
        },
        {
            "name": "ternary_twopass_raw40",
            "quant": "ternary",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 40,
            "desc": "BitNet 1.58-Bit Ternary + Client Two-Pass Exact Raw Rerank (Top-40)"
        },

        # OPQ / PQ (Optimized Product Quantization, 64-byte, 60.3x compression)
        {
            "name": "opq",
            "quant": "opq",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "OPQ 64-byte FWHT Rotated ADC (Single Pass, No Rerank)"
        },
        {
            "name": "opq_ef400",
            "quant": "opq",
            "rerank": False,
            "oversample": 1,
            "ef_search": 400,
            "desc": "OPQ 64-byte FWHT Rotated (ef_search=400)"
        },
        {
            "name": "opq_rerank4",
            "quant": "opq",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "OPQ 64-byte (Server ADC Rerank 4x)"
        },
        {
            "name": "opq_rerank8",
            "quant": "opq",
            "rerank": True,
            "oversample": 8,
            "ef_search": 100,
            "desc": "OPQ 64-byte (Server ADC Rerank 8x)"
        },
        {
            "name": "opq_twopass_raw40",
            "quant": "opq",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 40,
            "desc": "OPQ 64-byte + Client Two-Pass Exact Raw Rerank (Top-40)"
        },
        {
            "name": "pq",
            "quant": "pq",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "Naive Product Quantization 64-byte (Single Pass)"
        },
        {
            "name": "pq_rerank4",
            "quant": "pq",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "Naive PQ 64-byte (Server ADC Rerank 4x)"
        },
        {
            "name": "pq_rerank8",
            "quant": "pq",
            "rerank": True,
            "oversample": 8,
            "ef_search": 100,
            "desc": "Naive PQ 64-byte (Server ADC Rerank 8x)"
        },
        {
            "name": "pq_twopass_raw40",
            "quant": "pq",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 40,
            "desc": "Naive PQ 64-byte + Client Two-Pass Exact Raw Rerank (Top-40)"
        },

        # Extreme (Binary ADC, 1-bit)
        {
            "name": "extreme",
            "quant": "extreme",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "desc": "1-Bit Binary ADC (Single Pass, No Rerank)"
        },
        {
            "name": "extreme_rerank4",
            "quant": "extreme",
            "rerank": True,
            "oversample": 4,
            "ef_search": 100,
            "desc": "1-Bit Binary ADC (Server Rerank 4x)"
        },
        {
            "name": "extreme_rerank8",
            "quant": "extreme",
            "rerank": True,
            "oversample": 8,
            "ef_search": 100,
            "desc": "1-Bit Binary ADC (Server Rerank 8x)"
        },
        {
            "name": "extreme_rerank16",
            "quant": "extreme",
            "rerank": True,
            "oversample": 16,
            "ef_search": 100,
            "desc": "1-Bit Binary ADC (Server Rerank 16x)"
        },
        {
            "name": "extreme_twopass_raw40",
            "quant": "extreme",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 40,
            "desc": "1-Bit Binary + Client Two-Pass Exact Raw Rerank (Top-40)"
        },
        {
            "name": "extreme_twopass_raw80",
            "quant": "extreme",
            "rerank": False,
            "oversample": 1,
            "ef_search": 100,
            "two_pass_raw": True,
            "raw_fetch_k": 80,
            "desc": "1-Bit Binary + Client Two-Pass Exact Raw Rerank (Top-80)"
        },

        # RaBitQ (1-bit + unbiased estimator, v4.2.5+)
        {"name": "rabitq", "quant": "rabitq", "rerank": False, "oversample": 1, "ef_search": 100,
         "desc": "RaBitQ 1-bit + correction factor (Single Pass, No Rerank)"},
        {"name": "rabitq_rerank4", "quant": "rabitq", "rerank": True, "oversample": 4, "ef_search": 100,
         "desc": "RaBitQ (Server Rerank 4x)"},
        {"name": "rabitq_rerank8", "quant": "rabitq", "rerank": True, "oversample": 8, "ef_search": 100,
         "desc": "RaBitQ (Server Rerank 8x)"},
        {"name": "rabitq_twopass_raw40", "quant": "rabitq", "rerank": False, "oversample": 1, "ef_search": 100,
         "two_pass_raw": True, "raw_fetch_k": 40, "desc": "RaBitQ + Client Two-Pass Exact Raw Rerank (Top-40)"},

        # PQ4 fast-scan (4-bit PQ, v4.2.5+)
        {"name": "pq4", "quant": "pq4", "rerank": False, "oversample": 1, "ef_search": 100,
         "desc": "PQ4 Fast-Scan 4-bit (Single Pass, No Rerank)"},
        {"name": "pq4_rerank4", "quant": "pq4", "rerank": True, "oversample": 4, "ef_search": 100,
         "desc": "PQ4 Fast-Scan (Server Rerank 4x)"},
        {"name": "pq4_rerank8", "quant": "pq4", "rerank": True, "oversample": 8, "ef_search": 100,
         "desc": "PQ4 Fast-Scan (Server Rerank 8x)"},
    ]

    if args.filter:
        filter_tokens = [t.strip().lower() for t in args.filter.split(",") if t.strip()]
        filtered_configs = [
            c for c in configs
            if any(tok in c["name"].lower() or tok == c["quant"].lower() for tok in filter_tokens)
        ]
        if not filtered_configs:
            print(f"⚠️  No configs matched filter '{args.filter}'. Available: {[c['name'] for c in configs]}")
            return
        configs = filtered_configs

    if args.exclude:
        ex_tokens = [t.strip().lower() for t in args.exclude.split(",") if t.strip()]
        configs = [
            c for c in configs
            if not any(tok == c["quant"].lower() or c["name"].lower().startswith(tok) for tok in ex_tokens)
        ]

    print(f"Configs To Run:  {len(configs)} configuration(s)")

    # 2. Generate Dataset
    print(f"\n[data] Generating {N_VECTORS:,} clustered vectors across {N_CLUSTERS} manifolds...")
    vectors = generate_clustered_dataset(N_VECTORS, DIMENSION, N_CLUSTERS, seed=42)
    centroids = vectors[:N_CLUSTERS]
    queries = generate_queries(centroids, N_QUERIES, DIMENSION, seed=999)

    # 3. Compute Ground Truth
    ground_truth = compute_ground_truth(vectors, queries, top_k=TOP_K)

    all_results = []
    baseline_res = None

    for i, cfg in enumerate(configs, 1):
        print(f"\n[{i}/{len(configs)}] Running benchmark for {cfg['name']}...")
        res = run_single_config(cfg, vectors, queries, ground_truth)
        all_results.append(res)
        if cfg["name"] == "none_ef100":
            baseline_res = res

    # 4. Save JSON results
    os.makedirs(os.path.dirname(RESULT_JSON), exist_ok=True)
    with open(RESULT_JSON, "w", encoding="utf-8") as f:
        json.dump(all_results, f, indent=2)
    print(f"\n💾 Saved raw results to {RESULT_JSON}")

    # 5. Generate Markdown Report
    if not args.no_report:
        if baseline_res is None:
            baseline_res = all_results[0]
        generate_markdown_report(all_results, baseline_res)

    failures = check_recall_gates(all_results, gates)
    if failures:
        print("\n❌ RECALL GATE FAILED:")
        for line in failures:
            print(f"   {line}")
        sys.exit(1)
    if gates:
        print(f"\n✅ Recall gate passed ({len(gates)} threshold(s)).")

    print("\n🎉 ALL CONFIGURATIONS TESTED SUCCESSFULLY!")

if __name__ == "__main__":
    main()
