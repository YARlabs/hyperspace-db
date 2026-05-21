#!/usr/bin/env python3
"""
HyperspaceDB Cache vs Redis — Real Performance Benchmark
=========================================================
Tests the following scenarios on Cosine (1024d) and YAR Hybrid (801d):

  1. HyperspaceDB  — Cache Disabled  (baseline: pure HNSW from disk/RAM)
  2. HyperspaceDB  — Cache Enabled, L1 Exact Hits  (using id filter)
  3. HyperspaceDB  — Cache Enabled, L2 ANN Hits    (full ANN, no filter)
  4. HyperspaceDB  — search_batch API (single gRPC call for 32 queries)
  5. Redis         — HSET + cosine brute-force (reference: fast KV store)

Key improvements over the previous stress test:
  - Uses search_batch() for true multi-query throughput measurement
  - Sets HYPERSPACE_CACHE_REBUILD_COOLDOWN_MS=500 for faster L2 warmup
  - Sets HYPERSPACE_CACHE_L2_EF_SEARCH=100 for better recall
  - Sets HYPERSPACE_CACHE_READ_THROUGH=true to auto-populate cache on misses
  - Runs concurrent workers at 16 / 32 / 64 levels to find saturation point
  - Compares directly against Redis for vector-like workloads
"""

import os
import sys
import time
import shutil
import subprocess
import socket
import struct
import threading
import numpy as np
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple

# ── SDK path ──────────────────────────────────────────────────────────────────
sdk_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python"))
sys.path.append(sdk_path)

from hyperspace import HyperspaceClient

# ── Redis (optional) ──────────────────────────────────────────────────────────
try:
    import redis
    REDIS_AVAILABLE = True
except ImportError:
    REDIS_AVAILABLE = False
    print("⚠️  redis-py not found (pip install redis). Redis comparison will be skipped.")

# ─────────────────────────────────────────────────────────────────────────────
# Config
# ─────────────────────────────────────────────────────────────────────────────

SCENARIOS = [
    # (name,         metric,    dim,   ann_threshold,  collection)
    ("Cosine-1024d", "cosine",  1024,  0.30,           "bench_cosine"),
    ("Hybrid-801d",  "hybrid",  801,   2.0,            "bench_hybrid"),
]

NUM_VECTORS   = 5_000   # dataset size per collection
NUM_QUERIES   = 1_000   # search queries per phase
BATCH_SIZE    = 32      # search_batch chunk size
WARMUP_RATIO  = 0.05    # fraction of queries used for server warmup
CONCURRENCIES = [16, 32]  # concurrent thread counts to test

SERVER_BIN = os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/release/hyperspace-server"))
DATA_DIR   = os.path.abspath(os.path.join(os.path.dirname(__file__), "../data"))

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

def kill_servers():
    subprocess.run(["pkill", "-f", "hyperspace-server"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.5)

def wait_for_port(port: int = 50051, timeout: float = 20.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with socket.create_connection(("localhost", port), timeout=1.0):
                return True
        except OSError:
            time.sleep(0.3)
    return False

def start_server(cache_enabled: bool, ann_threshold: float,
                 cooldown_ms: int = 500, ef_search: int = 100,
                 batch_inner_concurrency: int = 1) -> subprocess.Popen:
    """Start a fresh server instance with specified cache configuration."""
    kill_servers()
    if os.path.exists(DATA_DIR):
        try:
            shutil.rmtree(DATA_DIR)
        except Exception as e:
            print(f"   ⚠️  Could not remove data dir: {e}")

    env = os.environ.copy()
    env["HYPERSPACE_CACHE_ENABLED"]              = "true" if cache_enabled else "false"
    env["HYPERSPACE_CACHE_L1_CAPACITY"]          = "25000"
    env["HYPERSPACE_CACHE_REBUILD_BATCH"]        = "50"
    env["HYPERSPACE_CACHE_REBUILD_COOLDOWN_MS"]  = str(cooldown_ms)
    env["HYPERSPACE_CACHE_L2_EF_SEARCH"]         = str(ef_search)
    env["HYPERSPACE_CACHE_READ_THROUGH"]         = "true"
    env["HYPERSPACE_WAL_SYNC_MODE"]              = "async"
    env["HS_HNSW_EF_CONSTRUCT"]                  = "200"
    env["HS_HNSW_EF_SEARCH"]                     = "100"
    # Parallel inner execution of search_batch — key for real batch throughput!
    env["HS_SEARCH_BATCH_INNER_CONCURRENCY"]     = str(batch_inner_concurrency)

    if cache_enabled and ann_threshold > 0:
        env["HYPERSPACE_CACHE_ANN_THRESHOLD"] = str(ann_threshold)
    elif "HYPERSPACE_CACHE_ANN_THRESHOLD" in env:
        del env["HYPERSPACE_CACHE_ANN_THRESHOLD"]

    proc = subprocess.Popen([SERVER_BIN], env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if not wait_for_port():
        proc.terminate()
        sys.exit("❌  Server failed to start!")
    time.sleep(0.5)
    return proc

def wait_for_indexing(client: HyperspaceClient, collection: str, timeout: float = 90.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            stats = client.get_collection_stats(collection)
            if stats.get("indexing_queue", 1) == 0:
                return
        except Exception:
            pass
        time.sleep(0.3)

# ─────────────────────────────────────────────────────────────────────────────
# Vector generation
# ─────────────────────────────────────────────────────────────────────────────

def make_vectors(n: int, dim: int, metric: str) -> np.ndarray:
    """Generate n valid vectors for the given metric space."""
    rng = np.random.default_rng(42)
    if metric == "cosine":
        v = rng.standard_normal((n, dim)).astype(np.float64)
        v /= np.linalg.norm(v, axis=1, keepdims=True) + 1e-12
        return v
    elif metric == "hybrid":
        lorentz_dim, eucl_dim = 33, dim - 33
        spatial = rng.uniform(-0.05, 0.05, (n, lorentz_dim - 1))
        t = np.sqrt(1.0 + (spatial ** 2).sum(axis=1, keepdims=True))
        lorentz = np.concatenate([t, spatial], axis=1)
        eucl = rng.standard_normal((n, eucl_dim)).astype(np.float64)
        eucl /= np.linalg.norm(eucl, axis=1, keepdims=True) + 1e-12
        return np.concatenate([lorentz, eucl], axis=1)
    else:
        raise ValueError(f"Unknown metric: {metric}")

# ─────────────────────────────────────────────────────────────────────────────
# Measurement utilities
# ─────────────────────────────────────────────────────────────────────────────

@dataclass
class RunResult:
    qps: float = 0.0
    p50_ms: float = 0.0
    p99_ms: float = 0.0
    errors: int = 0

def measure_concurrent(fn, queries: List, concurrency: int, batch: int = 1) -> RunResult:
    """
    Run `fn(batch_of_queries)` in parallel using `concurrency` threads.
    Returns QPS, p50, p99.
    """
    latencies: List[float] = []
    lock = threading.Lock()
    errors = 0

    # Split queries into chunks of `batch`
    chunks = [queries[i:i+batch] for i in range(0, len(queries), batch)]

    def run_chunk(chunk):
        nonlocal errors
        t0 = time.perf_counter()
        try:
            fn(chunk)
        except Exception as e:
            with lock:
                errors += 1
        elapsed_ms = (time.perf_counter() - t0) * 1000.0
        with lock:
            latencies.append(elapsed_ms)

    t_start = time.perf_counter()
    with ThreadPoolExecutor(max_workers=concurrency) as ex:
        list(ex.map(run_chunk, chunks))
    total_s = time.perf_counter() - t_start

    if not latencies:
        return RunResult()

    total_queries = len(queries)
    qps = total_queries / total_s
    p50 = float(np.percentile(latencies, 50))
    p99 = float(np.percentile(latencies, 99))
    return RunResult(qps=qps, p50_ms=p50, p99_ms=p99, errors=errors)

# ─────────────────────────────────────────────────────────────────────────────
# HyperspaceDB Phases
# ─────────────────────────────────────────────────────────────────────────────

def hs_insert(client: HyperspaceClient, vectors: np.ndarray,
              ids: List[int], collection: str, batch_sz: int = 500):
    """Batch-insert all vectors."""
    for i in range(0, len(vectors), batch_sz):
        client.batch_insert(
            vectors=vectors[i:i+batch_sz].tolist(),
            ids=ids[i:i+batch_sz],
            collection=collection
        )

def run_hs_phase(label: str, client: HyperspaceClient, queries: np.ndarray,
                 collection: str, concurrency: int,
                 use_id_filter: bool = False,
                 ids: List[int] = None,
                 use_batch_api: bool = False) -> RunResult:
    """
    Single search-phase measurement.
    - use_id_filter=True  → L1 Exact Hit  (filter={"id": str(x)})
    - use_batch_api=True  → search_batch() for a chunk at once
    - otherwise           → concurrent single searches
    """
    warmup_n = max(1, int(len(queries) * WARMUP_RATIO))

    if use_batch_api:
        # Warmup
        for i in range(0, warmup_n, BATCH_SIZE):
            client.search_batch(queries[i:i+BATCH_SIZE].tolist(), top_k=10, collection=collection)

        def fn_batch(chunk):
            client.search_batch([q.tolist() for q in chunk], top_k=10, collection=collection)

        return measure_concurrent(fn_batch, list(queries), concurrency, batch=BATCH_SIZE)

    elif use_id_filter:
        # Warmup
        for i in range(warmup_n):
            client.search(vector=queries[i].tolist(), filter={"id": str(ids[i])},
                          top_k=10, collection=collection)

        def fn_l1(chunk):
            for j, q in enumerate(chunk):
                client.search(vector=q.tolist(), filter={"id": str(ids[j])},
                              top_k=10, collection=collection)

        return measure_concurrent(fn_l1, list(zip(queries, ids[:len(queries)])),
                                  concurrency, batch=1)
    else:
        # Warmup
        for i in range(warmup_n):
            client.search(vector=queries[i].tolist(), top_k=10, collection=collection)

        def fn_single(chunk):
            for q in chunk:
                client.search(vector=q.tolist() if not isinstance(q, list) else q,
                              top_k=10, collection=collection)

        return measure_concurrent(fn_single, queries.tolist(), concurrency, batch=1)

# ─────────────────────────────────────────────────────────────────────────────
# Redis Phase (brute-force ANN via HSET + numpy cosine)
# ─────────────────────────────────────────────────────────────────────────────

def run_redis_phase(vectors: np.ndarray, queries: np.ndarray,
                    dim: int, concurrency: int, n_top: int = 10) -> Optional[RunResult]:
    """
    Simulate vector ANN on Redis:
    - Store all vectors as HSET fields (packed float32 binary)
    - For each query: HGETALL → unpack → cosine similarity → top-K
    This represents what a Redis-based vector store would look like WITHOUT
    native RediSearch / RedisVSS. With RediSearch VSS the picture is different,
    but most users start with basic Redis.
    """
    if not REDIS_AVAILABLE:
        return None

    try:
        r = redis.Redis(host="localhost", port=6379, db=15)
        r.ping()
    except Exception as e:
        print(f"      ⚠️  Redis not available on localhost:6379 — {e}")
        return None

    # Upload
    print("      📤 Loading vectors into Redis...")
    pipe = r.pipeline(transaction=False)
    for i, v in enumerate(vectors):
        blob = struct.pack(f"{dim}f", *v.astype(np.float32))
        pipe.hset(f"vec:{i}", "data", blob)
        if (i + 1) % 500 == 0:
            pipe.execute()
            pipe = r.pipeline(transaction=False)
    pipe.execute()

    # Fetch all keys once (for brute-force reference)
    all_blobs = {i: r.hget(f"vec:{i}", "data") for i in range(len(vectors))}
    all_vecs = np.stack([
        np.frombuffer(blob, dtype=np.float32).astype(np.float64)
        for blob in all_blobs.values()
    ])  # shape: (N, dim)

    warmup_n = max(1, int(len(queries) * WARMUP_RATIO))
    for q in queries[:warmup_n]:
        sims = all_vecs @ q
        np.argpartition(-sims, n_top)[:n_top]

    def fn_redis(chunk):
        for q in chunk:
            sims = all_vecs @ (q if isinstance(q, np.ndarray) else np.array(q))
            np.argpartition(-sims, n_top)[:n_top]

    result = measure_concurrent(fn_redis, list(queries), concurrency, batch=1)

    # Cleanup
    try:
        for i in range(len(vectors)):
            r.delete(f"vec:{i}")
    except Exception:
        pass

    return result

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

def run_scenario(scenario_name: str, metric: str, dim: int,
                 ann_threshold: float, collection: str, concurrency: int):
    print(f"\n  {'─'*60}")
    print(f"  🧬  {scenario_name}  |  concurrency={concurrency}")
    print(f"  {'─'*60}")

    vectors = make_vectors(NUM_VECTORS, dim, metric)
    queries = vectors[:NUM_QUERIES]
    ids     = list(range(1, NUM_VECTORS + 1))

    results: Dict[str, RunResult] = {}

    # ── Phase A: Cache DISABLED ───────────────────────────────────────────────
    print("  [1/5] HyperspaceDB — Cache OFF...")
    srv = start_server(cache_enabled=False, ann_threshold=0.0)
    cli = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    cli.create_collection(collection, dimension=dim, metric=metric)
    hs_insert(cli, vectors, ids, collection)
    wait_for_indexing(cli, collection)
    results["hs_cache_off"] = run_hs_phase("Cache OFF", cli, queries, collection, concurrency)
    cli.delete_collection(collection)
    srv.terminate(); srv.wait()
    print(f"      → {results['hs_cache_off'].qps:.0f} QPS  |  p50={results['hs_cache_off'].p50_ms:.2f}ms  p99={results['hs_cache_off'].p99_ms:.2f}ms")

    # ── Phase B: Cache ON — L1 Exact Hits ────────────────────────────────────
    print("  [2/5] HyperspaceDB — Cache ON (L1 Exact Hits, id filter)...")
    srv = start_server(cache_enabled=True, ann_threshold=ann_threshold, cooldown_ms=500)
    cli = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    cli.create_collection(collection, dimension=dim, metric=metric)
    hs_insert(cli, vectors, ids, collection)
    wait_for_indexing(cli, collection)
    time.sleep(2.0)   # let L1 build
    results["hs_l1_hit"] = run_hs_phase("L1 Hit", cli, queries, collection, concurrency,
                                         use_id_filter=True, ids=ids)
    cli.delete_collection(collection)
    srv.terminate(); srv.wait()
    print(f"      → {results['hs_l1_hit'].qps:.0f} QPS  |  p50={results['hs_l1_hit'].p50_ms:.2f}ms  p99={results['hs_l1_hit'].p99_ms:.2f}ms")

    # ── Phase C: Cache ON — L2 ANN Hits ──────────────────────────────────────
    print("  [3/5] HyperspaceDB — Cache ON (L2 ANN)...")
    srv = start_server(cache_enabled=True, ann_threshold=ann_threshold, cooldown_ms=500, ef_search=100)
    cli = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    cli.create_collection(collection, dimension=dim, metric=metric)
    hs_insert(cli, vectors, ids, collection)
    wait_for_indexing(cli, collection)
    time.sleep(6.0)   # let L2 build (cooldown=500ms so multiple rebuilds can occur)
    results["hs_l2_ann"] = run_hs_phase("L2 ANN", cli, queries, collection, concurrency)
    cli.delete_collection(collection)
    srv.terminate(); srv.wait()
    print(f"      → {results['hs_l2_ann'].qps:.0f} QPS  |  p50={results['hs_l2_ann'].p50_ms:.2f}ms  p99={results['hs_l2_ann'].p99_ms:.2f}ms")

    # ── Phase D: search_batch API — NOW WITH PARALLEL SERVER EXECUTION ────────
    print(f"  [4/5] HyperspaceDB — search_batch (batch={BATCH_SIZE}, parallel inner={BATCH_SIZE}, Cache ON)...")
    srv = start_server(cache_enabled=True, ann_threshold=ann_threshold,
                       cooldown_ms=500, batch_inner_concurrency=BATCH_SIZE)
    cli = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    cli.create_collection(collection, dimension=dim, metric=metric)
    # Boost ef_search for best quality during batch benchmark
    cli.configure(ef_search=100, collection=collection)
    hs_insert(cli, vectors, ids, collection)
    wait_for_indexing(cli, collection)
    time.sleep(2.0)
    results["hs_batch"] = run_hs_phase("Batch API", cli, queries, collection, concurrency,
                                        use_batch_api=True)
    cli.delete_collection(collection)
    srv.terminate(); srv.wait()
    print(f"      → {results['hs_batch'].qps:.0f} QPS  |  p50={results['hs_batch'].p50_ms:.2f}ms  p99={results['hs_batch'].p99_ms:.2f}ms")

    # ── Phase E: Redis brute-force ────────────────────────────────────────────
    print("  [5/5] Redis — brute-force cosine (numpy in-memory)...")
    redis_res = run_redis_phase(vectors, queries, dim, concurrency)
    if redis_res:
        results["redis"] = redis_res
        print(f"      → {redis_res.qps:.0f} QPS  |  p50={redis_res.p50_ms:.2f}ms  p99={redis_res.p99_ms:.2f}ms")
    else:
        print("      → (skipped)")

    return results

def print_report(all_results: List[Tuple]):
    W = 130
    print("\n" + "=" * W)
    print("  📊  HYPERSPACEDB CACHE vs REDIS — PERFORMANCE REPORT")
    print("=" * W)
    header = f"  {'Scenario':<22}  {'Mode':<28}  {'QPS':>8}  {'p50 ms':>8}  {'p99 ms':>8}  {'vs Cache-OFF':>13}  {'vs Redis':>10}"
    print(header)
    print("  " + "─" * (W - 2))

    for (scenario_name, concurrency, results) in all_results:
        baseline_qps  = results.get("hs_cache_off", RunResult()).qps or 1
        redis_qps     = results.get("redis", RunResult()).qps or 0

        labels = [
            ("hs_cache_off", f"HyperspaceDB Cache OFF  (c={concurrency})"),
            ("hs_l1_hit",    f"HyperspaceDB L1 Hit     (c={concurrency})"),
            ("hs_l2_ann",    f"HyperspaceDB L2 ANN     (c={concurrency})"),
            ("hs_batch",     f"HyperspaceDB Batch×{BATCH_SIZE}  (c={concurrency})"),
            ("redis",        f"Redis brute-force numpy (c={concurrency})"),
        ]

        first = True
        for key, label in labels:
            r = results.get(key)
            if r is None:
                continue
            vs_off   = f"{r.qps / baseline_qps:.2f}x" if key != "hs_cache_off" else "baseline"
            vs_redis = f"{r.qps / redis_qps:.2f}x" if redis_qps > 0 else "N/A"
            row_name = scenario_name if first else ""
            print(f"  {row_name:<22}  {label:<28}  {r.qps:>8.0f}  {r.p50_ms:>8.2f}  {r.p99_ms:>8.2f}  {vs_off:>13}  {vs_redis:>10}")
            first = False
        print("  " + "─" * (W - 2))

    print("\n  ✨  Benchmark complete.")
    print("=" * W)

def main():
    print("=" * 70)
    print("🔥  HyperspaceDB Cache vs Redis — Real Performance Benchmark")
    print("=" * 70)
    print(f"\n   Scenarios  : {', '.join(s[0] for s in SCENARIOS)}")
    print(f"   Vectors    : {NUM_VECTORS:,}  |  Queries: {NUM_QUERIES:,}")
    print(f"   Batch size : {BATCH_SIZE}  |  Concurrencies: {CONCURRENCIES}")
    print(f"   Server     : {SERVER_BIN}")
    print(f"   Redis      : {'available' if REDIS_AVAILABLE else 'not installed (pip install redis)'}")
    print()

    all_results = []

    for (scenario_name, metric, dim, ann_threshold, collection) in SCENARIOS:
        for concurrency in CONCURRENCIES:
            try:
                results = run_scenario(scenario_name, metric, dim, ann_threshold, collection, concurrency)
                all_results.append((scenario_name, concurrency, results))
            except Exception as e:
                print(f"  ❌  Scenario {scenario_name} failed: {e}")
                kill_servers()

    print_report(all_results)
    kill_servers()

if __name__ == "__main__":
    main()
