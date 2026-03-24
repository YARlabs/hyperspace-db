# n8n-nodes-hyperspacedb

This is an n8n community node for [HyperspaceDB](https://github.com/yarlabs/hyperspace-db) — a high-performance, multi-geometry vector database designed for advanced Agentic AI and cognitive modeling.
# HyperspaceDB n8n Integration

The world's first Hyperbolic Vector Database integration for n8n. Build advanced Spatial AI workflows with native Poincaré and Lorentz geometry support.

## Features

- **Hyperbolic & Euclidean Spaces**: Support for Lorentz, Poincaré, Cosine, and L2 metrics.
- **Standalone Architecture**: No external LangChain library conflicts. Logic is internalized for stability.
- **Auto-Syncing**: Automatically detects collection dimensions and metrics - no more manual configuration errors.
- **Universal SDK**: Powered by the official `hyperspace-sdk-ts`.
- **RAG Ready**: Seamlessly integrates with n8n AI Agent and Chains.

## Nodes

### 1. HyperspaceDB Vector Store
The primary node for document storage and similarity search.
- **Dynamic Collection Selection**: Select your database from a dropdown list.
- **Zero-Config Metadata**: Dimensions and geometry are fetched automatically from the server.

### 2. HyperspaceDB Embeddings
Server-side vectorization node.
- **Geometry Matching**: Automatically matches the space of your target collection.

### 3. HyperspaceDB (Standalone)
Direct API access for management tasks:
- Create/Delete Collections
- Get Stats
- Direct Vector/Text Insertion
- Graph Traversals

## Installation

Install via the n8n Community Nodes panel:
`n8n-nodes-hyperspacedb`

## Credentials

Requires `HyperspaceDB API` credentials:
- **Host**: Your server address
- **Port**: gRPC port (default 50051)
- **API Key**: Your secure access key

---
Built with ❤️ by the YARlabs.

## 🤝 SDK Integration
This node works alongside the [hyperspace-sdk-ts](https://www.npmjs.com/package/hyperspace-sdk-ts). While the n8n node uses the REST API (port 50050), the SDK provides full gRPC (port 50051) support for high-throughput applications.

## 📜 License
MIT
