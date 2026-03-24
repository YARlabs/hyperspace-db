# HyperspaceDB MCP Server

[![MCP](https://img.shields.io/badge/MCP-Protocol-blue)](https://modelcontextprotocol.io)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue)](https://www.typescriptlang.org/)
[![HyperspaceDB](https://img.shields.io/badge/HyperspaceDB-v3.0-cyan)](https://github.com/yarlabs/hyperspace-db)

The **HyperspaceDB MCP Server** acts as a high-performance cognitive bridge, enabling Large Language Models (LLMs) to interact with **HyperspaceDB** — a multi-geometry vector database designed for advanced AI memory and spatial computing.

This server implements the [Model Context Protocol (MCP)](https://modelcontextprotocol.io), exposing a comprehensive suite of tools for **geometrical data analysis**, **graph traversal**, and **cognitive AI metrics** directly to models in Claude Desktop, Cursor, and other MCP hosts.

## 🚀 Key Features

### 1. Geometric Data Diagnostics
Identify the optimal geometry for your data using **Gromov Delta-hyperbolicity** analysis.
- **`hyperspace_analyze_geometry`**: Uses the 4-point condition to recommend `Lorentz`, `Poincare`, `Cosine`, or `L2` metrics for your datasets.

### 2. Cognitive AI Tools (Agentic Logic)
Track and manage the model's internal reasoning stability.
- **`hyperspace_analyze_thought_stability`**: Calculates **Lyapunov Convergence** of a trajectory (Chain of Thought). Detects if a model is "hallucinating" or converging on a stable logical attractor.
- **`hyperspace_find_clusters`**: Detects emergent semantic regions in the database knowledge graph to help the model synthesize higher-level concepts.

### 3. High-Performance Knowledge Retrieval
- **`hyperspace_search_text`**: Natural language semantic search using server-side embeddings.
- **`hyperspace_search_wasserstein`**: Optimal Transport (OT) based search for comparing complex distributions and finding non-obvious conceptual overlaps.
- **`hyperspace_insert_text`**: Asynchronous storage of factual claims or system logs with automatic vectorization.

### 4. Graph Memory Navigation
- **`hyperspace_graph_traverse`**: Perform deep BFS/DFS traversal through the HNSW knowledge graph. Allows the model to "follow paths" between disparate facts to build complex reasoning chains.

---

## 🛠️ Installation & Setup

### Prerequisites
- Node.js 18+
- Running instance of [HyperspaceDB](https://github.com/yarlabs/hyperspace-db) (default: `localhost:50051`)

### 1. Run directly with npx (Recommended)
You don't need to install anything. Just run:

```bash
npx mcp-hyperspacedb
```

### 2. Configuration for MCP Hosts

Add the following to your MCP configuration file (e.g., `claude_desktop_config.json` or Cursor settings):

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb"],
      "env": {
        "HYPERSPACE_HOST": "localhost:50051",
        "HYPERSPACE_API_KEY": "I_LOVE_HYPERSPACEDB"
      }
    }
  }
}
```

---

## 🧩 Available Tools

### Data Tools
- **`hyperspace_list_collections`**: Get all active collections.
- **`hyperspace_create_collection`**: Setup new memory spaces with specific geometry.
- **`hyperspace_insert_text`**: Store new facts into the DB.
- **`hyperspace_search_text`**: Query the DB using semantic similarity.
- **`hyperspace_search_wasserstein`**: Advanced cross-feature distribution search.

### Graph & AI Tools
- **`hyperspace_get_neighbors`**: Explore local connectivity in the vector graph.
- **`hyperspace_graph_traverse`**: Perform multi-hop logical exploration.
- **`hyperspace_find_clusters`**: Identify thematic regions in vector space.
- **`hyperspace_analyze_thought_stability`**: Validate Chain of Thought (CoT) stability.
- **`hyperspace_analyze_geometry`**: Run Gromov Delta analysis on raw vectors.

### System Tools
- **`hyperspace_get_stats`**: Telemetry on cluster health, clocks, and vector volume.
- **`hyperspace_trigger_reconsolidation`**: Manually trigger "AI Sleep Mode" (Flow Matching optimization) for a collection.

---

## 👨‍💻 Development
To run in development mode with live logs:
```bash
npm run dev
```

## 📜 License
MIT
