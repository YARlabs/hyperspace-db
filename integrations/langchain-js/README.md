# LangChain Hyperspace Integration: Spatial AI Memory

[![NPM Version](https://img.shields.io/npm/v/langchain-hyperspace?style=for-the-badge)](https://www.npmjs.com/package/langchain-hyperspace)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

**Give your agents reflex-level speed and spatial reasoning.** 

This is the official LangChain integration for **HyperspaceDB** — the world's first spatial AI engine designed for Autonomous Agents, Robotics, and Continuous Learning.

## 🧠 Beyond Vector Search: Spatial AI Memory

Traditional vector databases are built to search static text. **HyperspaceDB** is built to model human cognition and physical world hierarchies:

*   **Fractal Knowledge Graphs**: Euclidean vectors fail at hierarchies. Our native Hyperbolic engine (Poincaré & Lorentz models) compresses complex codebases or taxonomies into low-dimensional spaces, reducing RAM usage by 50x without losing semantic context.
*   **Continuous Reconsolidation**: Transform raw information into episodic memory. Use built-in **Flow Matching** and **Riemannian Math** (Fréchet mean, parallel transport) natively within your LangChain chains.
*   **Edge-to-Cloud Sync**: Robots and web-agents can't wait for cloud latency. Use the **Merkle Tree Delta Sync** protocol to handshakes episodic memory chunks between local devices and the cloud.
*   **Lock-Free ArcSwap Architecture**: Built on Rust. Achieve up to 12,000 Search QPS and 60,000 Ingest QPS with near-zero latency, even under extreme agent concurrency.

## 📦 Installation

```bash
npm install langchain-hyperspace hyperspace-sdk-ts @langchain/core
```

## 🛠 Usage

### Hyperbolic Memory Initialization (Poincaré Ball)

```typescript
import { HyperspaceStore } from "langchain-hyperspace";
import { HyperspaceClient } from "hyperspace-sdk-ts";

const client = new HyperspaceClient("localhost:50051", "YOUR_API_KEY");

const vectorStore = new HyperspaceStore(
    embeddings, // Your favorite embeddings or useServerSideEmbedding
    {
        client,
        collectionName: "agent_spatial_memory",
        metric: "lorentz", // Use hyperbolic geometry for hierarchical knowledge
        dimension: 64,      // High semantic compression
    }
);
```

### Advanced Spatial Pruning (Geometric Search)

Go beyond simple similarity. Prune memories using spatial regions:

```typescript
const results = await vectorStore.similaritySearch("Find drone flight patterns", 5, {
    spatial_region: {
        $in_ball: {
            center: [0.12, -0.45, 0.88, ...],
            radius: 0.15
        }
    }
});
```

## 📡 Edge-Cloud Handshake

Synchronize episodic memory between local robot/agent and the fleet:

```typescript
// Handshake hashes to identify memory drift
const digest = await client.getDigest("agent_memories");
await syncWithCloud(digest.state_hash, digest.buckets);
```

---

## 📖 Documentation

*   [Why Spatial AI? (YARlabs Manifesto)](https://yar.ink/manifesto)
*   [HyperspaceDB Official Docs](https://yar.ink/docs)

## 📄 License

Apache-2.0. Copyright © 2026 **YARlabs**.
