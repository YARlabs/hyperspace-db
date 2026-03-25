# LlamaIndex Hyperspace Integration: Spatial AI Memory Suite (JS/TS)

[![NPM Version](https://img.shields.io/npm/v/llamaindex-hyperspace?style=for-the-badge)](https://www.npmjs.com/package/llamaindex-hyperspace)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

**Give your JS/TS agents a high-performance spatial memory.**

This is the official LlamaIndex integration for **HyperspaceDB** — the high-performance multi-geometry vector database designed for Autonomous Agents, Robotics, and Continuous Learning.

## 🧠 Beyond RAG: Spatial AI Engine

Traditional vector databases were built for static chatbots. **HyperspaceDB** is built to model human cognition and episodic memory:

*   **Fractal Knowledge Graphs**: Euclidean vectors fail at hierarchies. Our native Poincaré & Lorentz models (Hyperbolic geometry) compress massive trees (like codebases or taxonomies) into small, semantically dense vectors, reducing RAM usage by 50x.
*   **Memory Reconsolidation**: AI agents need to "sleep" and organize memories. Use our built-in **Flow Matching** and **Riemannian Math** (Fréchet mean, parallel transport) natively to dynamically shift and prune vectors.
*   **Edge-to-Cloud Integration**: Web-agents and robotics and humanoid robots can't wait for cloud latency. Use the **Merkle Tree Delta Sync** protocol to handshakes episodic memory chunks between the client (WASM/Local) and Cloud.
*   **Lock-Free Performance**: Built with Rust. Achievement up to 12,000 Search QPS with near-zero latency, even under massive concurrent agent workloads.

## 📦 Installation

```bash
npm install llamaindex-hyperspace llamaindex hyperspace-sdk-ts
```

## 🛠 Usage

### Hyperbolic Memory Initialization (Poincaré Ball)

```typescript
import { HyperspaceVectorStore } from "llamaindex-hyperspace";
import { HyperspaceClient } from "hyperspace-sdk-ts";

const client = new HyperspaceClient("localhost:50051", "YOUR_API_KEY");

const vectorStore = new HyperspaceVectorStore({
    client,
    collectionName: "agent_spatial_memory",
    metric: "lorentz", // Use hyperbolic geometry for hierarhical data
    dimension: 64,      // semantically dense
});
```

### Advanced Spatial Pruning (Geometric Search)

Go beyond similarity. Prune memories using spatial regions:

```typescript
// Use in ball (center + radius) to prune semantic space
const results = await vectorStore.query({
    queryVector: [0.1, -0.4, ...],
    filters: {
        location: {
            $in_ball: {
                center: [0.12, -0.45, ...],
                radius: 0.15
            }
        }
    }
});
```

## 📡 Edge-Cloud Delta Sync Handshake

Identify memory drift and sync with the Cloud using Merkle-XOR buckets:

```typescript
// 1. Handshake: Send local 256 bucket hashes
const { diffBuckets } = await client.syncHandshake(collection, localBuckets);

if (diffBuckets.length > 0) {
    // 2. Pull only the modified/missing buckets
    const stream = client.syncPull(collection, diffBuckets);
}
```

---

## 📖 Documentation

*   [Why Spatial AI? (YARlabs Manifesto)](https://yar.ink/manifesto)
*   [HyperspaceDB Official Docs](https://yar.ink/docs)

## 📄 License

Apache-2.0. Copyright © 2026 **YARlabs**.
