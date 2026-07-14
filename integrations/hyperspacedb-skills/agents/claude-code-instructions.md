# HyperspaceDB — Claude Code CLAUDE.md Instructions
# Place in your project root as CLAUDE.md or include in your system prompt

## Project Context
This project uses **HyperspaceDB** (v3.1.2+) as its vector database.
HyperspaceDB communicates via **gRPC** on port 50051.

## Setup Instructions for Claude Code
When asked to add HyperspaceDB functionality:

1. **TypeScript**: Install `hyperspace-sdk-ts@^3.1.4`
2. **Python**: Install `hyperspacedb>=3.1.4`
3. **Always read env vars** for host/key — never hardcode

## Architecture Constraints
- HyperspaceDB runs as a separate gRPC server process
- The Rust server handles HNSW indexing, billing, and replication
- TypeScript/Python SDKs are thin gRPC clients
- MCP server (`mcp-hyperspacedb`) bridges the database to AI tools

## Code Patterns

### TypeScript
```typescript
import { HyperspaceClient, CollectionSchema } from 'hyperspace-sdk-ts';

const db = new HyperspaceClient(
  process.env.HYPERSPACE_HOST!,
  process.env.HYPERSPACE_API_KEY!
);

// createCollection requires CollectionSchema (NOT flat params):
const schema: CollectionSchema = {
  components: [
    { name: "main", metric: "cosine", fullDimension: 1536, weight: 1.0 }
    // metric options: "cosine" | "l2" | "lorentz" | "poincare" | "hybrid"
  ],
  cascadePipeline: [
    { componentName: "main", cutoffDimension: 1536, storeInRam: true, rerankTopK: 100 }
  ]
};
await db.createCollection("my_collection", schema);

// Hybrid search (vector + BM25):
const results = await db.search(queryVec, 10, "my_collection", {
  hybridQuery: "search terms",   // BM25 text
  hybridAlpha: 0.5,              // 0.0=BM25 only, 1.0=vector only
});
```

### Python
```python
from hyperspacedb import HyperspaceClient
import os

db = HyperspaceClient(
    host=os.environ["HYPERSPACE_HOST"],
    api_key=os.environ["HYPERSPACE_API_KEY"]
)
```

## When to Use Which Metric
- Text embeddings → `cosine`
- Numerical features → `l2`
- Ontologies / taxonomies → `lorentz`
- Hyperbolic manifolds → `poincare`
- Hierarchy + semantic text mixed → `hybrid` (first 33 dims = Lorentz, rest = Euclidean; total dim e.g. 801 = 33+768)

## Cognitive AI Features
HyperspaceDB has unique features for agentic memory:
- **Lyapunov stability** (`analyzeThoughtStability`): detect hallucination
- **Koopman momentum** (`predictMomentum`): forecast reasoning direction
- **Trust score** (`getTrustScore`): quantify reasoning confidence

These are used in production agentic pipelines to validate CoT before acting.

## DePIN Notes
- DePIN module is **alpha v0.0.1a** — use testnet only
- Always set `replicationFactor >= 2` in DePIN deployments
- Billing auto-evicts data when balance = 0

## MCP Tools Available
If `mcp-hyperspacedb` is configured as an MCP server, the following tools are available:
hyperspace_list_collections, hyperspace_create_collection, hyperspace_delete_collection,
hyperspace_insert_text, hyperspace_search_text, hyperspace_search_wasserstein,
hyperspace_get_neighbors, hyperspace_graph_traverse, hyperspace_explore_graph,
hyperspace_get_subsumption_tree, hyperspace_get_concept_parents,
hyperspace_find_clusters, hyperspace_analyze_thought_stability,
hyperspace_predict_momentum, hyperspace_get_trust_score, hyperspace_analyze_geometry,
hyperspace_get_stats, hyperspace_rebuild_index, hyperspace_vacuum,
hyperspace_trigger_reconsolidation, hyperspace_freeze_collection,
hyperspace_unfreeze_collection, hyperspace_cache_stats, hyperspace_cache_clear,
hyperspace_cache_config, hyperspace_delete_points, hyperspace_get_points
