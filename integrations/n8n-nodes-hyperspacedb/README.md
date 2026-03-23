# n8n-nodes-hyperspacedb

This is an n8n community node for [HyperspaceDB](https://github.com/yarlabs/hyperspace-db) — a high-performance, multi-geometry vector database designed for advanced Agentic AI and cognitive modeling.

## 🚀 Features

- **Multi-Geometry Support**: L2, Cosine, Poincaré, and Lorentz distances.
- **Cognitive Graph Ops**: HNSW graph traversal, neighbors retrieval, and semantic clustering.
- **Wait-Free Indexing**: Real-time vector insertions with optional background indexing.
- **System Control**: Metrics, memory vacuuming, and instance status.
- **[H] Branding**: Official cyan logo integration.

## 📦 Installation

Follow the [community nodes installation guide](https://docs.n8n.io/integrations/community-nodes/installation/) in n8n.

```bash
npm install n8n-nodes-hyperspacedb
```

## 🛠️ Configuration

### Default Ports
- **REST API (n8n Node)**: `http://localhost:50050/api`
- **gRPC API (SDK)**: `localhost:50051`

### Authentication
The node requires an **API Key** (default: `I_LOVE_HYPERSPACEDB`) and a **Base URL**.

## 🧩 Supported Operations

### Collection
- **Get All**: List all managed collections.
- **Create**: Setup new collections with geometry-specific dimensions.
- **Get Stats**: Real-time telemetry (vector count, indexing queue).
- **Delete**: Remove a collection and all its data.
- **Rebuild Index**: Trigger Hot Vacuum optimization.

### Vector
- **Insert**: Store vectors with numeric IDs and JSON metadata.
- **Search**: Similarity search with top-k and optional Optimal Transport (Wasserstein).

### Graph (Agentic AI)
- **Get Node**: Retrieve node metadata and neighbors.
- **Get Neighbors**: Explore vector proximity in the HNSW graph.
- **Traverse**: BFS/DFS graph exploration.
- **Find Clusters**: Automatic semantic region detection.

### System
- **Status**: Instance configuration and health.
- **Metrics**: Performance metrics (RAM, CPU usage).
- **Usage**: Detailed quota/usage report.
- **Vacuum**: Force memory reclamation to the OS.

## 🤝 SDK Integration
This node works alongside the [hyperspace-sdk-ts](https://www.npmjs.com/package/hyperspace-sdk-ts). While the n8n node uses the REST API (port 50050), the SDK provides full gRPC (port 50051) support for high-throughput applications.

## 📜 License
MIT
