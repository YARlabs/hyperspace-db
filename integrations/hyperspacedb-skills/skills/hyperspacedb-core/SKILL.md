---
name: hyperspacedb-core
description: >
  Core operations for HyperspaceDB — a multi-geometry vector database.
  Use this skill whenever you need to: create or manage collections, insert vectors
  or text, perform semantic search, retrieve points by ID, or delete data.
  Trigger on: "create collection", "insert vector", "search", "find similar",
  "vector database", "semantic memory", "HyperspaceDB", "embed and store".
---

# HyperspaceDB Core Operations

HyperspaceDB is a high-performance, multi-geometry vector database with built-in AI capabilities.
The server communicates over **gRPC** (port `50051` by default).

## Connection

```typescript
import { HyperspaceClient } from 'hyperspace-sdk-ts';

const client = new HyperspaceClient(
  process.env.HYPERSPACE_HOST ?? 'localhost:50051',
  process.env.HYPERSPACE_API_KEY ?? 'I_LOVE_HYPERSPACEDB'
);
```

```python
from hyperspacedb import HyperspaceClient

client = HyperspaceClient(
    host=os.environ.get("HYPERSPACE_HOST", "localhost:50051"),
    api_key=os.environ.get("HYPERSPACE_API_KEY", "I_LOVE_HYPERSPACEDB")
)
```

> **Always use environment variables** for `HYPERSPACE_HOST` and `HYPERSPACE_API_KEY`.
> Never hardcode connection strings in application code.

---

## 1. Collections

### Create a Collection

`createCollection` takes a `CollectionSchema` — a **multi-component** descriptor that
defines vector geometry and the MRL (Multi-Resolution Layered) cascade pipeline.

```typescript
import { HyperspaceClient, CollectionSchema } from 'hyperspace-sdk-ts';

// Simple single-component collection
const schema: CollectionSchema = {
  components: [
    {
      name: "main",
      metric: "cosine",   // cosine | l2 | lorentz | poincare | hybrid
      fullDimension: 1536,
      weight: 1.0
    }
  ],
  cascadePipeline: [
    { componentName: "main", cutoffDimension: 1536, storeInRam: true, rerankTopK: 100 }
  ]
};

await client.createCollection("my_memory", schema);

// Hybrid collection: 33 Lorentz dims + 768 Euclidean dims (total 801 dims)
const hybridSchema: CollectionSchema = {
  components: [
    { name: "hybrid_main", metric: "hybrid", fullDimension: 801, weight: 1.0 }
  ],
  cascadePipeline: [
    { componentName: "hybrid_main", cutoffDimension: 801, storeInRam: false, rerankTopK: 200 }
  ]
};
await client.createCollection("ontology_and_text", hybridSchema);
```

**Geometry selection guide:**
| Data type | Recommended metric |
|-----------|-------------------|
| NLP embeddings (normalized) | `cosine` |
| Dense numeric features | `l2` |
| Hierarchical / taxonomic data | `lorentz` |
| Hyperbolic manifolds | `poincare` |
| Mixed: hierarchy + semantic text | `hybrid` (33 Lorentz + N Euclidean dims) |

Use `hyperspace_analyze_geometry` (via MCP) to auto-detect the best metric for your data.

### List Collections

```typescript
const collections = await client.listCollections();
```

### Delete a Collection

```typescript
await client.deleteCollection("my_memory"); // irreversible!
```

---

## 2. Insert Data

### Insert Raw Vector

```typescript
const id = await client.insert(
  [0.1, 0.2, ..., 0.9],  // float32 array, length = fullDimension
  { source: "user_chat", timestamp: "2026-07-14" },  // metadata
  "my_memory"
);
```

### Insert Text (auto-embed on server)

```typescript
const id = await client.insertText(
  "The user asked about quantum entanglement.",
  { user_id: "u123", session: "sess_abc" },  // metadata
  "my_memory"
);
```

### Batch Insert

```typescript
// batchInsert takes an array of {vector, metadata} objects
const ids = await client.batchInsert(
  [
    { vector: [...], metadata: { key: "val1" } },
    { vector: [...], metadata: { key: "val2" } },
  ],
  "my_memory"
);
```

---

## 3. Search

### Semantic Vector Search

```typescript
const results = await client.search(
  queryVector,   // number[] | Float32Array | Float64Array
  10,            // top-k
  "my_memory",
  {
    filters: [...],    // optional Filter[]
    mrlDimension: 256, // optional: use truncated MRL dimension for faster search
    useWasserstein: false,
    includePayload: false,
  }
);
// results: Array<{ id, distance, metadata, typedMetadata, payload? }>
```

### Hybrid Search (Vector + BM25 Full-text)

For collections with **`hybrid`** metric or when you need keyword+semantic fusion:

```typescript
const results = await client.search(
  queryVector,
  10,
  "my_memory",
  {
    hybridQuery: "quantum entanglement",  // BM25 text query
    hybridAlpha: 0.5,  // 0.0 = pure BM25, 1.0 = pure vector, 0.5 = balanced
  }
);
```

### Search by Text (server-side embedding)

```typescript
const results = await client.searchText(
  "quantum physics papers",
  10,
  "my_memory",
  {
    hybridAlpha: 0.7,  // optional: blend vector score with BM25
  }
);
```

### Filtering

```typescript
const results = await client.search(queryVector, 10, "my_memory", {
  filters: [
    { and: [
      { match: { key: "source", value: "research_paper" } },
      { range: { key: "year", gte: 2020 } }
    ]}
  ]
});
```

---

## 4. Point Operations

### Get Points by IDs

```typescript
const points = await client.getPoints([1, 42, 99], "my_memory");
```

### Delete a Point

```typescript
await client.delete(pointId, "my_memory");
```

---

## 5. Maintenance

```typescript
await client.rebuildIndex("my_memory");  // optimize HNSW graph
await client.vacuum();                   // purge deleted vectors, reclaim disk
```

---

## Common Pitfalls

- **Dimension mismatch**: The vector dimension must exactly match the collection's configured dimension.
- **Wrong metric for data**: Use `lorentz` for hierarchical/ontological data, not `l2`.
- **Forgetting to vacuum**: Deleted points are soft-deleted; call `vacuum()` to reclaim disk.
- **Replication Factor 1 in production**: Always use RF ≥ 2 for fault tolerance in DePIN deployments.

## See Also

- [hyperspacedb-graph](../hyperspacedb-graph/SKILL.md) — graph traversal and Lorentz hierarchy
- [hyperspacedb-cognitive](../hyperspacedb-cognitive/SKILL.md) — CoT stability, momentum, trust
- [hyperspacedb-mcp](../hyperspacedb-mcp/SKILL.md) — MCP server tool reference
