# LlamaIndex Hyperspace Integration: Spatial AI Memory Infrastructure

[![PyPI Version](https://img.shields.io/pypi/v/llama-index-vector-stores-hyperspace?style=for-the-badge)](https://pypi.org/project/llama-index-vector-stores-hyperspace/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

**Building the Episodic Memory for the AGI Era.**

This is the official LlamaIndex integration for **HyperspaceDB** — the world's first **Spatial AI Engine**. It models information exactly how the physical world and human cognition are structured: as hierarchical, spatial, and dynamic graphs.

## 🧠 Why Spatial AI for LlamaIndex? (Beyond RAG)

Traditional vector databases were built to search static PDF files for chatbots. **HyperspaceDB** provides the primitives for autonomous agents and robotics:

*   **Fractal Knowledge Graphs**: Euclidean vectors fail at hierarchies. Our Poincaré & Lorentz models compress massive trees (like codebases or medical taxonomies) into low-dimensional spaces, reducing RAM usage by 50x without losing semantic context.
*   **Continuous Reconsolidation**: AI agents need to "sleep" and organize memories. With our **Fast Upsert Path** and **Riemannian Math SDK** (Fréchet mean, parallel transport), your indexers can continuously shift and prune vectors dynamically.
*   **Heterogeneous Tribunal Framework**: Natively support the confrontational model of LLM routing (Architect vs. Tribunal) directly on the vector graph. Calculate a **Geometric Trust Score** to verify logical path lengths and detect hallucinations.
*   **Edge-to-Cloud Delta Sync**: Drones and humanoid robots can't wait for cloud latency. HyperspaceDB runs directly on Edge hardware, using **Merkle Tree Delta Sync** to asynchronously handshake and sync memory chunks with the Cloud.

## 📦 Installation

```bash
pip install llama-index-vector-stores-hyperspace hyperspacedb
```

## 🛠 Usage

### Hyperbolic Memory Initialization

```python
from llama_index.vector_stores.hyperspace import HyperspaceVectorStore
from llama_index.core import StorageContext, VectorStoreIndex
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", "API_KEY")

vector_store = HyperspaceVectorStore(
    client=client,
    collection_name="agent_memory",
    metric="lorentz",  # Use hyperbolic geometry for complex hierarchies
    dimension=64
)
```

### Hallucination Detection (Tribunal Framework)

Evaluate the structural trust of an LLM claim by verifying the logical path length between concepts in latent hyperbolic space:

```python
from hyperspace.agents import TribunalContext

# 1.0 = Truth (Identical), 0.0 = Hallucination (Disconnected)
score = client.evaluate_claim(concept_a_id=12, concept_b_id=45)
print(f"Geometric Trust Score: {score}")
```

### Multi-Geometry Spatial Filters

Prune search results by geometric regions:

```python
from llama_index.core.vector_stores import MetadataFilters

filters = MetadataFilters(
    filters=[
        # Spatial Sphere (Ball) Pruning
        {"key": "location", "value": {
            "$in_ball": {"center": [0,0,0, ...], "radius": 0.5}
        }}
    ]
)
```

---

## ⚡ Performance: Reflex-Level Speed

Built on Nightly Rust. Our **ArcSwap Lock-Free architecture** and **SIMD f32** intrinsics deliver up to **12,000 Search QPS** and **60,000 Ingest QPS** for real-time robotic memory.

## 📖 Documentation

*   [The Case for Hyperbolic Memory](https://yar.ink/manifesto)
*   [HyperspaceDB Official Documentation](https://yar.ink/docs)

## 📄 License

Apache-2.0. Copyright © 2026 **YARlabs**.
