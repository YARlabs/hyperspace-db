---
name: hyperspacedb-mcp
description: >
  Model Context Protocol (MCP) server for HyperspaceDB. Use this skill to configure,
  connect, and use HyperspaceDB tools in Claude Desktop, Cursor, Windsurf, Antigravity,
  or any MCP-compatible AI host. Covers all 30+ available MCP tools.
  Trigger on: "MCP", "Claude Desktop", "Cursor MCP", "hyperspace_search",
  "hyperspace_insert", "model context protocol", "mcp-hyperspacedb", "tool use".
---

# HyperspaceDB MCP Server

The `mcp-hyperspacedb` server exposes all HyperspaceDB capabilities as MCP tools,
letting any MCP-compatible AI host (Claude, Cursor, Windsurf, Antigravity, etc.)
directly interact with the database — **without writing any integration code**.

---

## Setup

### Quick Start (Recommended)

```bash
npx mcp-hyperspacedb
```

### MCP Host Configuration

Add to your MCP config file (e.g., `claude_desktop_config.json`, `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb"],
      "env": {
        "HYPERSPACE_HOST": "your-node.example.com:50051",
        "HYPERSPACE_API_KEY": "your_secret_key"
      }
    }
  }
}
```

**Config file locations by host:**
| Host | Config path |
|------|-------------|
| Claude Desktop (macOS) | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Claude Desktop (Windows) | `%APPDATA%\Claude\claude_desktop_config.json` |
| Cursor | `.cursor/mcp.json` in project root |
| Windsurf | `~/.windsurf/mcp.json` |
| Antigravity / custom | Per host documentation |

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HYPERSPACE_HOST` | `localhost:50051` | gRPC address of HyperspaceDB node |
| `HYPERSPACE_API_KEY` | `I_LOVE_HYPERSPACEDB` | Authentication key |

> **Never hardcode these values.** Always pass via environment variables.

---

## Complete Tool Reference

### 📁 Collection Management

| Tool | Description |
|------|-------------|
| `hyperspace_list_collections` | List all collections with stats |
| `hyperspace_create_collection` | Create a new collection (name, dimension, metric: cosine/l2/lorentz/poincare/**hybrid**, cascadePipeline) |
| `hyperspace_delete_collection` | **Permanently** delete a collection and all vectors |
| `hyperspace_freeze_collection` | Make collection read-only |
| `hyperspace_unfreeze_collection` | Re-enable inserts |
| `hyperspace_rebuild_index` | Rebuild and optimize the HNSW index |
| `hyperspace_vacuum` | Purge soft-deleted vectors, reclaim disk space |

### 📥 Data Operations

| Tool | Description |
|------|-------------|
| `hyperspace_insert_text` | Insert text (auto-embedded server-side) |
| `hyperspace_delete_points` | Delete a vector by ID |
| `hyperspace_get_points` | Retrieve vectors by IDs |

### 🔍 Search

| Tool | Description |
|------|-------------|
| `hyperspace_search_text` | Semantic search by text query; supports `hybrid_alpha` for BM25 fusion |
| `hyperspace_search_wasserstein` | Optimal Transport cross-distribution search |

### 🕸️ Graph & Hierarchy

| Tool | Description |
|------|-------------|
| `hyperspace_get_neighbors` | Direct HNSW neighbors of a node |
| `hyperspace_graph_traverse` | BFS/DFS multi-hop traversal |
| `hyperspace_explore_graph` | Visualization-ready graph data (nodes + edges) |
| `hyperspace_get_subsumption_tree` | Lorentz hierarchy tree from a root concept |
| `hyperspace_get_concept_parents` | Parent concepts in the hierarchy |
| `hyperspace_find_clusters` | Unsupervised cluster detection |

### 🧠 Cognitive AI

| Tool | Description |
|------|-------------|
| `hyperspace_analyze_thought_stability` | Lyapunov CoT convergence analysis (returns `{ lyapunov_exponent, is_stable }`) |
| `hyperspace_predict_momentum` | Koopman trajectory momentum forecast (returns `number[]` — next vector) |
| `hyperspace_get_trust_score` | Composite reasoning trust score (returns `number` 0–1) |
| `hyperspace_analyze_geometry` | Gromov Delta analysis → optimal metric (cosine/l2/lorentz/poincare/hybrid) |

### ⚙️ System & Cache

| Tool | Description |
|------|-------------|
| `hyperspace_get_stats` | Node telemetry, vector counts, clock |
| `hyperspace_trigger_reconsolidation` | Trigger Flow Matching sleep-mode optimization |
| `hyperspace_cache_stats` | L0 cache hit/miss statistics |
| `hyperspace_cache_clear` | Clear L0 cache for a collection |
| `hyperspace_cache_config` | Update cache eviction policy and ANN threshold |

---

## Example Prompts

Once connected, you can say to your AI agent:

```
"Create a collection called 'team_memory' with cosine metric and 1536 dimensions."

"Search 'team_memory' for documents about Kubernetes deployments."

"Analyze the geometry of these vectors and tell me the best metric to use."

"Check if my reasoning chain [ids: 1,2,3,4,5] is stable or diverging."

"Show me the parent concepts of node 42 in 'knowledge_graph'."

"What are the main topic clusters in 'research_papers'?"
```

---

## How Cognitive Tools Work

The MCP server implements cognitive tools as **client-side computations**:
1. `getPoints()` fetches the stored vectors for the given trajectory IDs
2. Mathematical analysis (Lyapunov, Koopman, trust) runs inside the MCP server process
3. Results are returned without any server-side RPC for the math itself

This means cognitive tools work against **any version** of the HyperspaceDB server.

**Geometry awareness**: When collection metric is `hybrid`, momentum extrapolation
splits the vector into Lorentz (first 33 dims) and Euclidean (remaining dims) parts,
extrapolates each independently, then recombines.

---

## See Also

- [hyperspacedb-core](../hyperspacedb-core/SKILL.md) — direct SDK usage
- [hyperspacedb-cognitive](../hyperspacedb-cognitive/SKILL.md) — cognitive primitives explained
- [hyperspacedb-depin](../hyperspacedb-depin/SKILL.md) — DePIN node setup
