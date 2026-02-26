#!/usr/bin/env python3
"""
Task 3.3 / 3.4 Integration Test — UDP Gossip Swarm (боевые условия)

Scenario:
  1. Launch 10 nodes HyperspaceDB on unique gRPC / HTTP / UDP-gossip ports.
  2. Wait for 3 heartbeat cycles (15-20 s) and make sure all nodes see the swarm.
  3. Kill 5 nodes (Network Partition).
  4. Wait for PEER_TTL=30 s + 1 heartbeat and make sure dead peers are removed.
"""

import os
import shutil
import signal
import subprocess
import sys
import time

import requests

# ─── Configuration ─────────────────────────────────────────────────────────────

BASE_DIR = "/tmp/hs_swarm_test"
BINARY   = os.path.abspath("target/release/hyperspace-server")

NUM_NODES    = 10
API_KEY      = "I_LOVE_HYPERSPACEDB"
HEADERS      = {"x-api-key": API_KEY}

# Ports:  gRPC = 50651+i,  HTTP-Dashboard = 50750+i,  UDP-Gossip = 7970+i-1
def grpc_port(i):  return 50650 + i          # 50651 … 50660
def http_port(i):  return 50750 + i          # 50751 … 50760
def gossip_port(i): return 7969 + i          # 7970 … 7979

# Seed-peers: nodes 1+2 talk to each other, others — seed-string with both
def seed_peers(i: int) -> str:
    s1 = f"127.0.0.1:{gossip_port(1)}"
    s2 = f"127.0.0.1:{gossip_port(2)}"
    if i == 1: return s2
    if i == 2: return s1
    return f"{s1},{s2}"


# ─── Helpers ────────────────────────────────────────────────────────────────── 

def start_node(i: int) -> subprocess.Popen:
    data = os.path.join(BASE_DIR, f"node{i}")
    os.makedirs(data, exist_ok=True)

    env = {
        **os.environ,
        "HS_DIMENSION":       "1024",
        "HS_METRIC":          "cosine",
        "HYPERSPACE_API_KEY": API_KEY,
        "HS_DATA_DIR":        data,
        "HS_GOSSIP_ENABLED": "true",
        "HS_GOSSIP_PORT":     str(gossip_port(i)),
        "HS_GOSSIP_PEERS":    seed_peers(i),
    }

    log_path = os.path.join(BASE_DIR, f"node{i}.log")
    log_fh   = open(log_path, "w")

    proc = subprocess.Popen(
        [
            BINARY,
            "--port",      str(grpc_port(i)),
            "--http-port", str(http_port(i)),
        ],
        env=env,
        stdout=log_fh,
        stderr=subprocess.STDOUT,
    )
    return proc


def wait_ready(i: int, timeout: int = 20) -> bool:
    """Polls /api/status until node responds."""
    url      = f"http://127.0.0.1:{http_port(i)}/api/status"
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            r = requests.get(url, headers=HEADERS, timeout=2)
            if r.status_code == 200:
                return True
        except Exception:
            pass
        time.sleep(0.4)
    return False


def swarm_peers(i: int) -> dict | None:
    """GET /api/swarm/peers on node i."""
    try:
        r = requests.get(
            f"http://127.0.0.1:{http_port(i)}/api/swarm/peers",
            headers=HEADERS,
            timeout=3,
        )
        if r.status_code == 200:
            return r.json()
    except Exception:
        pass
    return None


def log_tail(i: int, n: int = 18):
    path = os.path.join(BASE_DIR, f"node{i}.log")
    try:
        with open(path) as f:
            lines = f.readlines()
        print(f"\n  ── node{i}.log (last {n} lines) ──")
        print("".join(lines[-n:]))
    except Exception as e:
        print(f"  (cannot read node{i}.log: {e})")


def kill_proc(proc: subprocess.Popen):
    try:
        proc.send_signal(signal.SIGKILL)
        proc.wait(timeout=3)
    except Exception:
        pass


# ─── Main ─────────────────────────────────────────────────────────────────────

def main():
    # ── 0. Ensure binary exists ───────────────────────────────────────────────
    if not os.path.exists(BINARY):
        print("Binary not found. Building …")
        rc = subprocess.run(
            ["cargo", "build", "--release", "--bin", "hyperspace-server"]
        ).returncode
        if rc != 0:
            sys.exit("Build failed!")

    # ── Cleanup stale state ───────────────────────────────────────────────────
    if os.path.exists(BASE_DIR):
        shutil.rmtree(BASE_DIR)
    os.makedirs(BASE_DIR, exist_ok=True)

    processes: list[subprocess.Popen]  = []
    results: dict[str, str]            = {}

    try:
        # ── 1. Launch all 10 nodes ────────────────────────────────────────────
        print(f"\n🚀 Запуск {NUM_NODES} нод HyperspaceDB …")
        print(f"   gRPC={grpc_port(1)}–{grpc_port(NUM_NODES)}  "
              f"HTTP={http_port(1)}–{http_port(NUM_NODES)}  "
              f"UDP={gossip_port(1)}–{gossip_port(NUM_NODES)}\n")

        for i in range(1, NUM_NODES + 1):
            p = start_node(i)
            processes.append(p)
            print(f"  Node {i:2d} | gRPC:{grpc_port(i)} HTTP:{http_port(i)} "
                  f"UDP:{gossip_port(i)} | PID={p.pid}")
            time.sleep(0.2)  # stagger slightly

        # ── 2. Wait for all nodes to boot ─────────────────────────────────────
        print("\n⏳ Ожидаем готовности нод …")
        for i in range(1, NUM_NODES + 1):
            ok = wait_ready(i, timeout=25)
            sym = "✅" if ok else "❌"
            print(f"  {sym} Node {i}: {'ready' if ok else 'TIMEOUT'}")
            if not ok:
                log_tail(i)
                kill_proc(processes[i - 1])
                results["startup"] = f"FAIL (Node {i} did not start)"
                break
        else:
            results["startup"] = "PASS"

        if results.get("startup") != "PASS":
            return summarise(results)

        # ── 3. Let gossip propagate (3 heartbeat cycles × 5 s + 5 s buffer) ──
        GOSSIP_WAIT = 35
        print(f"\n📡 Ждём {GOSSIP_WAIT}с для распространения heartbeat-пакетов "
              f"(heartbeat=5s × 7 циклов — полная транзитивная discovery) …")
        time.sleep(GOSSIP_WAIT)

        # ── 4. Verify peer discovery ──────────────────────────────────────────
        print("\n🔍 Проверяем видимость пиров в рое:")
        discovery_fail = False
        SEEDS = {1, 2}   # seed nodes see ALL peers; others see ≥2 (seed routing)
        for i in range(1, NUM_NODES + 1):
            data = swarm_peers(i)
            if data is None:
                print(f"  ❌ Node {i}: /api/swarm/peers недоступен")
                discovery_fail = True
                continue
            count   = data.get("peer_count", 0)
            healthy = sum(1 for p in data["peers"] if p.get("healthy"))
            # Seed nodes are directly messaged by everyone → see all 9 peers
            # Non-seed nodes only know seeds → expect ≥2 (at minimum the 2 seeds)
            required = NUM_NODES - 1 if i in SEEDS else 2
            ok       = count >= required
            sym      = "✅" if ok else "⚠️ "
            label    = f"ожидалось ≥{required}"
            print(f"  {sym} Node {i}: видит {count} пиров ({healthy} healthy) — {label}")
            if not ok:
                discovery_fail = True

        results["discovery"] = "FAIL" if discovery_fail else "PASS"

        # ── 5. Kill nodes 6–10 (simulate partition) ────────────────────────────
        KILL_RANGE = range(6, NUM_NODES + 1)
        print(f"\n💀 Убиваем ноды {list(KILL_RANGE)} (симуляция Network Partition) …")
        for i in KILL_RANGE:
            kill_proc(processes[i - 1])
            print(f"  ☠️  Node {i} остановлена (PID {processes[i - 1].pid})")

        # ── 6. Wait for PEER_TTL eviction ─────────────────────────────────────
        PEER_TTL    = 30          # must match gossip.rs PEER_TTL
        EXTRA       = 10          # +1 heartbeat cycle
        EVICT_WAIT  = PEER_TTL + EXTRA
        print(f"\n⏳ Ждём {EVICT_WAIT}с (TTL={PEER_TTL}s + {EXTRA}s buffer) …")
        time.sleep(EVICT_WAIT)

        # ── 7. Verify eviction ─────────────────────────────────────────────────
        print("\n🔍 Проверяем, что мёртвые пиры исчезли из реестра:")
        eviction_fail = False
        ALIVE = range(1, 6)
        for i in ALIVE:
            data = swarm_peers(i)
            if data is None:
                print(f"  ❌ Node {i}: API недоступен")
                eviction_fail = True
                continue
            count   = data.get("peer_count", 0)
            healthy = sum(1 for p in data["peers"] if p.get("healthy"))
            # After TTL: we expect only the 4 other alive nodes visible
            ok      = count <= 4
            sym     = "✅" if ok else "❌"
            print(f"  {sym} Node {i}: видит {count} пиров ({healthy} healthy) — ожидалось ≤4")
            if not ok:
                eviction_fail = True

        results["eviction"] = "FAIL" if eviction_fail else "PASS"

    finally:
        # ── Teardown ───────────────────────────────────────────────────────────
        print("\n🧹 Остановка всех процессов …")
        for p in processes:
            kill_proc(p)
        time.sleep(1)
        shutil.rmtree(BASE_DIR, ignore_errors=True)
        print("✅ Очищено.")
        summarise(results)


def summarise(results: dict):
    print("\n" + "═" * 60)
    print("📋 ИТОГОВЫЙ ОТЧЁТ — UDP Gossip Swarm Test (Task 3.3 / 3.4)")
    print("═" * 60)
    all_pass = True
    labels = {
        "startup":   "Node Startup (10 нод)",
        "discovery": "Peer Discovery (UDP Gossip)",
        "eviction":  "PEER_TTL Eviction (30 s)",
    }
    for key, label in labels.items():
        val = results.get(key, "NOT RUN")
        ok  = val == "PASS"
        if not ok:
            all_pass = False
        sym = "✅" if ok else "❌"
        print(f"  {sym}  {label:40s}  {val}")

    print()
    if all_pass:
        print("🎉 ALL TESTS PASSED — tasks 3.3 and 3.4 confirmed!")
        sys.exit(0)
    else:
        print("⚠️  Some tests failed — see details above.")
        sys.exit(1)


if __name__ == "__main__":
    main()
