# HyperspaceDB Cursor Rules
# Place this content in your project's .cursorrules file

You have access to HyperspaceDB — a multi-geometry vector database designed for
advanced AI memory, semantic search, and graph-based knowledge systems.

## Connection
Always use environment variables for connection:
```typescript
const client = new HyperspaceClient(
  process.env.HYPERSPACE_HOST ?? 'localhost:50051',
  process.env.HYPERSPACE_API_KEY ?? 'I_LOVE_HYPERSPACEDB'
);
```
Never hardcode host or API keys.

## Geometry Selection
- `cosine` → normalized NLP embeddings (OpenAI, Cohere, etc.)
- `l2` → dense numeric feature vectors
- `lorentz` → hierarchical data, taxonomies, ontologies
- `poincare` → hyperbolic manifold data
- `hybrid` → mixed data: 33 Lorentz dims (hierarchy) + N Euclidean dims (semantics), total e.g. 801 dims (33+768)
- When unsure, use `hyperspace_analyze_geometry` to auto-detect.

## Replication
- Development: `replicationFactor: 1`
- Production/DePIN: `replicationFactor: 3` minimum

## TypeScript SDK — createCollection
`createCollection` takes a **CollectionSchema** (NOT a flat object):
```typescript
import { HyperspaceClient, CollectionSchema } from 'hyperspace-sdk-ts';
// Version: ^3.1.4

const schema: CollectionSchema = {
  components: [
    { name: "main", metric: "cosine", fullDimension: 1536, weight: 1.0 }
    // metric: "cosine" | "l2" | "lorentz" | "poincare" | "hybrid"
  ],
  cascadePipeline: [
    { componentName: "main", cutoffDimension: 1536, storeInRam: true, rerankTopK: 100 }
  ]
};
await client.createCollection("my_collection", schema);
```

## Python SDK
```python
from hyperspacedb import HyperspaceClient
# Version: >=3.1.4
```

## Hybrid Search
Combine vector similarity with BM25 full-text in a single query:
```typescript
await client.search(queryVector, 10, "collection", {
  hybridQuery: "search text",  // BM25 text query
  hybridAlpha: 0.5,            // 0.0=BM25 only, 1.0=vector only, 0.5=balanced
});
// searchText also supports hybridAlpha parameter
```

## Key Operations
- Insert: `client.insertText(text, metadata, collection)`
- Search: `client.searchText(query, topK, collection)`
- Graph: `client.traverse(startId, hops, collection)`
- Stability: `client.analyzeThoughtStability(ids, curvature, collection)`

## Cognitive Tools — When to Use
- **analyzeThoughtStability**: After building a reasoning chain, verify it converges
- **predictMomentum**: Pre-fetch context before the agent needs it
- **getTrustScore**: Gate decisions on reasoning confidence (threshold: 0.75)

## Maintenance
Always call `vacuum()` after bulk deletions to reclaim disk space.
Call `rebuildIndex(collection)` after major schema changes.

## MCP Integration
If HyperspaceDB MCP server is connected, prefer using MCP tools over direct SDK calls
for interactive sessions. Direct SDK is better for production application code.
