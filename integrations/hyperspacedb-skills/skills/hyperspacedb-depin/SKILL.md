---
name: hyperspacedb-depin
description: >
  DePIN (Decentralized Physical Infrastructure Network) node operations for HyperspaceDB.
  Use this skill when working with distributed node deployment, billing, storage economics,
  node registration, earning mechanics, and the DePIN SDK.
  Trigger on: "DePIN", "node", "miner", "staking", "billing", "storage fee",
  "decentralized storage", "node registration", "earn tokens", "hyperspace node".
  
  WARNING: DePIN is alpha software (v0.0.1a). Use at your own risk.
---

# HyperspaceDB DePIN — Decentralized Node Operations

> **⚠️ ALPHA SOFTWARE — v0.0.1a**
> DePIN support is in early alpha. APIs may change. Use on testnet only.
> Run on mainnet at your own risk.

---

## What is HyperspaceDB DePIN?

DePIN nodes are regular machines (VPS, bare metal, or home servers) that contribute
storage and compute to the HyperspaceDB network in exchange for economic rewards.

**How it works:**
1. Node operator runs `hyperspace-node` binary
2. Node registers itself on the p2p network via mDNS/gossip
3. Clients route vector operations to available nodes
4. Billing module tracks write, read, and storage fees per collection
5. Node operators earn rewards proportional to their contribution

---

## 1. Running a Node

### Installation

```bash
# Using the node launcher binary (Linux/macOS)
curl -sSf https://install.hyperspacedb.io | sh

# Or build from source
git clone https://github.com/yarlabs/hyperspace-db
cd hyperspace-db
cargo build --release -p hyperspace-server
```

### Launch

```bash
./hyperspace-node \
  --host 0.0.0.0:50051 \        # gRPC listen address
  --node-id my-node-01 \        # unique node identifier
  --data-dir /data/hyperspace \  # persistent storage path
  --api-key YOUR_SECRET_KEY \   # authentication
  --replication-factor 3        # replicate data to 3 nodes
```

### Environment Variables

```bash
export HYPERSPACE_NODE_ID="my-node-01"
export HYPERSPACE_DATA_DIR="/data/hyperspace"
export HYPERSPACE_API_KEY="your_secret_key"
export HYPERSPACE_HOST="0.0.0.0:50051"
export HYPERSPACE_REPLICATION_FACTOR="3"
```

---

## 2. Billing & Storage Economics

HyperspaceDB charges for three types of operations:

| Fee Type | Description | Unit |
|----------|-------------|------|
| **Write Fee** | Charged per vector insert | per vector |
| **Read Fee** | Charged per search/query | per query |
| **Storage Fee** | Charged continuously per stored vector | per vector/day |

### Storage Fee & Deletion Logic

If a client's balance drops to zero, the billing module **automatically evicts their
data** from the network. This prevents freeloading:

```rust
// Billing is enforced server-side via gRPC interceptors
// Client must maintain positive balance to keep data alive
```

### Checking Balance via DePIN SDK (Python)

```python
from hyperspace_depin import DePINClient

depin = DePINClient(
    host=os.environ["HYPERSPACE_HOST"],
    api_key=os.environ["HYPERSPACE_API_KEY"]
)

# Check current balance
balance = depin.get_balance()
print(f"Balance: {balance.tokens} tokens")
print(f"Storage used: {balance.bytes_stored} bytes")
print(f"Estimated days remaining: {balance.days_remaining}")

# Top up balance
depin.deposit(amount=100)  # 100 tokens
```

---

## 3. gRPC Signed Ticket Pipeline

All DePIN operations use **signed tickets** — cryptographically signed gRPC
metadata that authorizes billing deductions.

```python
# SignedTicket is automatically attached by the DePIN SDK
# as gRPC interceptor metadata on every request

from hyperspace_depin import DePINClient, SignedTicketInterceptor

# The interceptor signs each RPC call automatically
depin = DePINClient(
    host="my-node.example.com:50051",
    api_key="my_key",
    signing_key="my_ed25519_private_key"  # for DePIN ticket signing
)
```

---

## 4. Node Registration & P2P Discovery

Nodes discover each other via **mDNS** (local) and **gossip protocol** (WAN).

```bash
# Check registered peers from your node
hyperspace-node peers list

# Force bootstrap from a known seed
hyperspace-node peers add --seed seed1.hyperspacedb.io:50051
```

---

## 5. Replication Factor

**Always set Replication Factor ≥ 2** in production:

```typescript
await client.createCollection({
  name: "production_memory",
  dimension: 1536,
  metric: "cosine",
  replicationFactor: 3,   // data exists on 3 nodes; survives 2 node failures
});
```

| RF | Fault tolerance | Storage overhead |
|----|----------------|-----------------|
| 1 | None — single node failure = data loss | 1× |
| 2 | Survives 1 node failure | 2× |
| 3 | Survives 2 node failures (recommended) | 3× |

---

## 6. Monitoring

```python
# Node health and economic stats
stats = depin.get_node_stats()
print(f"Vectors stored: {stats.vector_count}")
print(f"Earnings (24h): {stats.earnings_24h} tokens")
print(f"Uptime: {stats.uptime_pct}%")
```

---

## Alpha Limitations (v0.0.1a)

- Token economy is simulated — no real token transfers yet
- mDNS discovery works on LAN; WAN bootstrap is manual
- No automatic rebalancing when nodes join/leave
- ZK privacy proofs are experimental

## See Also

- [hyperspacedb-core](../hyperspacedb-core/SKILL.md) — database operations
- [hyperspacedb-mcp](../hyperspacedb-mcp/SKILL.md) — MCP integration
