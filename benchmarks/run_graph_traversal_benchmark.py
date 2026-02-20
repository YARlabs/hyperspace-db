#!/usr/bin/env python3
"""
Graph Traversal Benchmark: server-side traverse vs client-side baseline BFS.
"""

import argparse
import statistics
import time
from collections import deque
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PY_SDK = os.path.join(ROOT, "sdks", "python")
if PY_SDK not in sys.path:
    sys.path.insert(0, PY_SDK)

from hyperspace.client import HyperspaceClient


def baseline_bfs(client, collection: str, start_id: int, depth: int, max_nodes: int):
    visited = set([start_id])
    queue = deque([(start_id, 0)])
    out = []
    while queue and len(out) < max_nodes:
        node_id, d = queue.popleft()
        out.append(node_id)
        if d >= depth:
            continue
        neighbors = client.get_neighbors(
            id=node_id, collection=collection, layer=0, limit=64, offset=0
        )
        for n in neighbors:
            nxt = int(n["id"])
            if nxt not in visited:
                visited.add(nxt)
                queue.append((nxt, d + 1))
    return out


def timed(fn, rounds: int):
    lat_ms = []
    sizes = []
    for _ in range(rounds):
        t0 = time.perf_counter()
        result = fn()
        dt = (time.perf_counter() - t0) * 1000.0
        lat_ms.append(dt)
        sizes.append(len(result))
    return lat_ms, sizes


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="localhost:50051")
    parser.add_argument("--collection", default="default")
    parser.add_argument("--start-id", type=int, default=1)
    parser.add_argument("--depth", type=int, default=2)
    parser.add_argument("--max-nodes", type=int, default=256)
    parser.add_argument("--rounds", type=int, default=50)
    parser.add_argument("--api-key", default=None)
    parser.add_argument("--user-id", default=None)
    args = parser.parse_args()

    client = HyperspaceClient(
        host=args.host, api_key=args.api_key, user_id=args.user_id
    )

    trav_lat, trav_sizes = timed(
        lambda: client.traverse(
            start_id=args.start_id,
            max_depth=args.depth,
            max_nodes=args.max_nodes,
            layer=0,
            collection=args.collection,
        ),
        args.rounds,
    )

    bfs_lat, bfs_sizes = timed(
        lambda: baseline_bfs(
            client,
            args.collection,
            args.start_id,
            args.depth,
            args.max_nodes,
        ),
        args.rounds,
    )

    print("=== Graph Traversal Benchmark ===")
    print(f"Rounds: {args.rounds}")
    print(
        f"Traverse RPC: mean={statistics.mean(trav_lat):.2f}ms p95={statistics.quantiles(trav_lat, n=100)[94]:.2f}ms avg_nodes={statistics.mean(trav_sizes):.1f}"
    )
    print(
        f"Baseline BFS: mean={statistics.mean(bfs_lat):.2f}ms p95={statistics.quantiles(bfs_lat, n=100)[94]:.2f}ms avg_nodes={statistics.mean(bfs_sizes):.1f}"
    )
    print(f"Speedup (mean): {statistics.mean(bfs_lat) / max(statistics.mean(trav_lat), 1e-9):.2f}x")


if __name__ == "__main__":
    main()
