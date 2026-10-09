# HyperspaceDB MCP Server

[![MCP](https://img.shields.io/badge/MCP-Protocol-blue)](https://modelcontextprotocol.io)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue)](https://www.typescriptlang.org/)
[![HyperspaceDB](https://img.shields.io/badge/HyperspaceDB-v4.0-cyan)](https://github.com/yarlabs/hyperspace-db)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The **HyperspaceDB MCP Server** (`mcp-hyperspacedb`) exposes the complete low-level data plane, cognitive geometry engine, graph navigation, and administration interface of **HyperspaceDB** to Large Language Models (LLMs) via the [Model Context Protocol (MCP)](https://modelcontextprotocol.io).

> [!NOTE]
> **Looking for drop-in conversational agent memory?**  
> If you are setting up plug-and-play long-term memory for Claude Desktop, Cursor, Windsurf, or OpenWebUI, use [`mcp-hyperspace-memory`](../mcp-hyperspace-memory) instead (`npx -y mcp-hyperspace-memory@latest`).  
> `mcp-hyperspacedb` is the **full-featured database and geometry server** designed for developers, data pipelines, and advanced cognitive agents requiring fine-grained control over vector spaces, collections, graph topologies, and cache systems.

---

## 🚀 Key Architectural Capabilities

### 1. Geometric Diagnostics & Curvature Analysis
Diagnose latent data topology using **Gromov $\delta$-hyperbolicity** over Riemannian and pseudo-Riemannian manifolds:
- **`hyperspace_analyze_geometry`**: Employs the 4-point Gromov condition to recommend the ideal manifold metric (`lorentz`, `poincare`, `cosine`, or `l2`).

### 2. Cognitive Agent Diagnostics (Chain-of-Thought Stability)
Evaluate the geometric stability of multi-step reasoning traces:
- **`hyperspace_analyze_thought_stability`**: Calculates **Lyapunov Convergence** of a reasoning trajectory. Distinguishes whether the model's reasoning converges toward a stable attractor or diverges into chaotic hallucinations.
- **`hyperspace_predict_momentum`**: Forecasts future thought trajectories via Koopman momentum extrapolation.
- **`hyperspace_get_trust_score`**: Computes geodesic trust scores along thought sequences.

### 3. Hyperbolic Hierarchy & Graph Traversal
- **`hyperspace_get_subsumption_tree`**: Traces taxonomic subsumption trees in hyperbolic space ($H^{33}$).
- **`hyperspace_get_concept_parents`**: Identifies hypernym/parent concepts along the Lorentz cone.
- **`hyperspace_graph_traverse`**: Multi-hop BFS/DFS path exploration in the HNSW knowledge graph.
- **`hyperspace_explore_graph`**: Extracts interactive subgraphs (nodes and links) for 2D/3D visualization.
- **`hyperspace_find_clusters`**: Detects emergent semantic clusters directly in the vector space.

### 4. Advanced Search & Vector Data Plane
- **`hyperspace_search_text`**: Hybrid semantic + BM25 lexical search with adjustable fusion weight (`hybrid_alpha`).
- **`hyperspace_search_wasserstein`**: Optimal Transport (Earth Mover's Distance) semantic distribution search.
- **`hyperspace_insert_text`** / **`hyperspace_get_points`** / **`hyperspace_delete_points`**: Point-level CRUD operations.

### 5. Collection Lifecycle & Hardware Acceleration Maintenance
- Full control over collection creation with quantization schemes (`none`, `medium`, `medium_plus`, `turbo`, `extreme`).
- L0 Hot Tier cache administration (`cache_stats`, `cache_clear`, `cache_config`).
- Server-side flow matching reconsolidation (`trigger_reconsolidation`), collection freezing, HNSW index rebuilding, and disk vacuuming.

---

## 🛠️ Installation & MCP Host Configuration

### Quick Start with npx

```bash
npx -y mcp-hyperspacedb@latest
```

### Host Configurations

Add to your MCP host configuration (e.g. `claude_desktop_config.json`, `~/.codeium/windsurf/mcp_config.json`, or Cursor Settings):

#### ☁️ 1. YAR.INK Managed SaaS

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb@latest"],
      "env": {
        "HYPERSPACE_HOST": "the.yar.ink",
        "HYPERSPACE_API_KEY": "sk_YOUR_API_KEY"
      }
    }
  }
}
```

#### 🏠 2. Local Self-Hosted Instance (Default Auth)

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb@latest"],
      "env": {
        "HYPERSPACE_HOST": "localhost:50051",
        "HYPERSPACE_API_KEY": "I_LOVE_HYPERSPACEDB"
      }
    }
  }
}
```

#### 🔒 3. Local DB + Cloud v5 Embedding API Key (Two-Key Setup)

When running a secured local engine while leveraging continuous cloud embeddings:

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb@latest"],
      "env": {
        "HYPERSPACE_HOST": "localhost:50051",
        "HYPERSPACE_API_KEY": "my_local_secret",
        "YAR_API_KEY": "sk_YOUR_YAR_API_KEY"
      }
    }
  }
}
```

* `HYPERSPACE_API_KEY`: Authenticates with local gRPC engine (`localhost:50051`).
* `YAR_API_KEY` (or legacy `CDE_API_KEY`): Authenticates with cloud embeddings endpoint.

---

## 🧩 Complete Tool Inventory (27 Tools)

### 📊 Data Plane Tools

| Tool | Required Parameters | Optional Parameters | Description & Return Value |
|---|---|---|---|
| `hyperspace_search_text` | `collection`, `text` | `top_k`, `hybrid_alpha`, `bm25_options` | Semantic + lexical hybrid search. Returns array of matching records with `id`, `score`, and `payload`. |
| `hyperspace_search_wasserstein` | `collection`, `text` | `top_k` | Optimal Transport (Wasserstein) search. Returns array of top matches ranked by Wasserstein distance. |
| `hyperspace_insert_text` | `collection`, `id`, `text` | `metadata` | Inserts text under numeric `id` with auto-vectorization. Returns status string or error. |
| `hyperspace_get_points` | `collection`, `ids` | — | Retrieves vector coordinates and metadata for a list of point IDs. Returns JSON array of points. |
| `hyperspace_delete_points` | `collection`, `id` | — | Deletes a point by its numeric ID. Returns `{ success }` object. |
| `hyperspace_create_collection` | `collection` | `dimension`, `metric`, `quantization` | Creates a new vector space with chosen geometry and quantization. Returns `{ success }` object. |
| `hyperspace_delete_collection` | `collection` | — | Permanently deletes a collection and its vectors. Returns `{ success }` object. |
| `hyperspace_list_collections` | — | — | Lists all active collections. Returns array of `{ name, dimension, metric, count }`. |

### 🕸️ Graph & Hierarchical Navigation Tools

| Tool | Required Parameters | Optional Parameters | Description & Return Value |
|---|---|---|---|
| `hyperspace_graph_traverse` | `collection`, `start_id` | `max_depth`, `max_nodes` | Multi-hop BFS graph traversal starting from `start_id`. Returns graph paths and visited nodes. |
| `hyperspace_explore_graph` | `collection`, `start_id` | `max_depth`, `max_nodes` | Traverses graph and outputs nodes and links formatted for visual renderers. |
| `hyperspace_get_neighbors` | `collection`, `id` | `layer`, `limit` | Returns local connectivity and neighboring nodes at a given HNSW graph layer. |
| `hyperspace_get_subsumption_tree` | `collection`, `root_id` | `max_depth` | Returns hierarchical Lorentz cone subsumption tree starting from `root_id`. |
| `hyperspace_get_concept_parents` | `collection`, `id` | `layer`, `limit` | Traverses upward along the Lorentz cone to retrieve parent concepts. |
| `hyperspace_find_clusters` | `collection` | `min_cluster_size` | Detects emergent semantic clusters in vector space. Returns cluster centroids and member IDs. |

### 🧠 Geometry & Cognitive AI Tools

| Tool | Required Parameters | Optional Parameters | Description & Return Value |
|---|---|---|---|
| `hyperspace_analyze_geometry` | `vectors` | `samples` | Computes Gromov $\delta$-hyperbolicity. Returns `{ delta, recommendation }` (`lorentz`, `poincare`, `cosine`, `l2`). |
| `hyperspace_analyze_thought_stability` | `trajectory` | `curvature` | Computes Lyapunov exponent along a Chain-of-Thought path. Classifies as `STABLE` or `CHAOTIC`. |
| `hyperspace_predict_momentum` | `collection`, `trajectory_ids` | `steps`, `curvature` | Forecasts next cognitive steps using Koopman momentum extrapolation. |
| `hyperspace_get_trust_score` | `collection`, `trajectory_ids` | `curvature` | Evaluates geodesic path consistency and returns trust score (0.0–1.0). |
| `hyperspace_trigger_reconsolidation` | `collection` | `learning_rate` | AI Sleep Mode: Triggers Flow Matching server optimization for concepts in collection. |

### ⚙️ System, Cache & Maintenance Tools

| Tool | Required Parameters | Optional Parameters | Description & Return Value |
|---|---|---|---|
| `hyperspace_get_stats` | `collection` | — | Merges gRPC telemetry, collection stats, logical clock, and HTTP cache stats into a JSON object. |
| `hyperspace_cache_stats` | `collection` | — | Retrieves hit rate, miss rate, and memory usage for the L0 Hot Tier Cache. |
| `hyperspace_cache_clear` | `collection` | — | Flushes all entries from the L0 Cache for the target collection. Returns `{ success }`. |
| `hyperspace_cache_config` | `collection`, `policy` | `ann_threshold` | Updates L0 Cache eviction policy (`lru`, `lfu`, `ttl`) and similarity threshold. |
| `hyperspace_freeze_collection` | `collection` | — | Locks collection into read-only mode to prevent new inserts. |
| `hyperspace_unfreeze_collection` | `collection` | — | Unlocks a frozen collection to permit modifications. |
| `hyperspace_rebuild_index` | `collection` | — | Triggers server-side HNSW index rebuild and optimization. |
| `hyperspace_vacuum` | — | — | Purges deleted tombstones and reclaims fragmented disk space. |

---

## 💻 Local Development

```bash
cd integrations/mcp-hyperspacedb
npm install
npm run build
npm run dev
```

## 📜 License

MIT © [YARlabs](https://yar.ink)
