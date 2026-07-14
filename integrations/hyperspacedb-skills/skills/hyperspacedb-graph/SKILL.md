---
name: hyperspacedb-graph
description: >
  Graph traversal, Lorentz hierarchy, and concept relationship operations for HyperspaceDB.
  Use this skill whenever working with hierarchical data, knowledge graphs, ontologies,
  concept taxonomies, or multi-hop reasoning chains.
  Trigger on: "knowledge graph", "hierarchy", "traverse", "parent concepts",
  "subsumption", "explore graph", "Lorentz embedding", "concept tree", "ontology".
---

# HyperspaceDB Graph Operations

HyperspaceDB's HNSW index is a navigable graph — you can traverse it directly for
**knowledge graph exploration**, **ontology navigation**, and **multi-hop reasoning**.

The **Lorentz metric** is specifically designed for **hierarchical data**: taxonomies,
organizational charts, biological hierarchies, and knowledge bases.

---

## 1. Get Neighbors (Local Graph Connectivity)

Returns the nearest neighbors of a point in the HNSW graph — its direct connections.

```typescript
const neighbors = await client.getNeighbors(
  pointId,      // number
  16,           // k — number of neighbors
  0,            // layer — HNSW layer (0 = most connected base layer)
  "my_collection"
);
// returns: Array<{ id: number, distance: number }>
```

**Use case**: Explore direct semantic relationships without doing a full ANN search.

---

## 2. Graph Traversal (Multi-hop BFS/DFS)

Traverse the knowledge graph starting from a node, following edges for N hops.

```typescript
const result = await client.traverse(
  startId,    // starting node
  3,          // max hops
  "my_collection"
);
// returns visited node IDs and traversal paths
```

**Use case**: "What concepts are reachable from 'quantum entanglement' within 2 hops?"

---

## 3. Explore Graph (Visualization-ready)

Returns nodes and edges in a structured format ready for graph visualization (D3, Cytoscape, etc.).

```typescript
const graph = await client.exploreGraph(
  startId,   // number
  2,         // max_depth
  256,       // max_nodes
  "my_collection"
);
// returns: { nodes: [{id, metadata}], edges: [{source, target, weight}] }
```

---

## 4. Subsumption Tree (Lorentz Hierarchy)

Retrieves the **full hierarchical tree** of concepts rooted at a given ID.
Only meaningful for collections using the **Lorentz metric**.

```typescript
const tree = await client.getSubsumptionTree(
  rootId,      // root concept ID
  3,           // max_depth
  "ontology_collection"
);
```

**Use case**: "Show me the complete taxonomy under the concept 'Mammal'."

---

## 5. Concept Parents

Retrieve the parent concepts of a node in the hierarchy.

```typescript
const parents = await client.getConceptParents(
  conceptId,   // number
  0,           // HNSW layer
  32,          // max parents to return
  "ontology_collection"
);
```

**Use case**: "What is the parent category of 'Golden Retriever'?" → Dog → Canine → Mammal

---

## 6. Semantic Clusters

Detects emergent thematic regions within the vector space — unsupervised conceptual grouping.

```typescript
const clusters = await client.findSemanticClusters(
  "my_collection",
  8           // number of clusters
);
// returns: Array<{ centroid: number[], member_ids: number[], label?: string }>
```

**Use case**: "What are the main topic clusters in this document corpus?"

---

## Best Practices for Hierarchical Data

### Setting Up a Lorentz Collection

```typescript
import { CollectionSchema } from 'hyperspace-sdk-ts';

const schema: CollectionSchema = {
  components: [
    { name: "ontology", metric: "lorentz", fullDimension: 128, weight: 1.0 }
  ],
  cascadePipeline: [
    { componentName: "ontology", cutoffDimension: 128, storeInRam: true, rerankTopK: 64 }
  ]
};
await client.createCollection("knowledge_graph", schema);
```

### Setting Up a Hybrid Collection (Hierarchy + Semantics)

Use `hybrid` when your data has **both** hierarchical structure **and** dense semantic content.
The fixed layout is: **33 Lorentz dims** (hierarchy) + **N Euclidean dims** (semantics).

```typescript
// 33 Lorentz + 768 Euclidean = 801 total dimensions
const hybridSchema: CollectionSchema = {
  components: [
    { name: "hybrid", metric: "hybrid", fullDimension: 801, weight: 1.0 }
  ],
  cascadePipeline: [
    { componentName: "hybrid", cutoffDimension: 801, storeInRam: false, rerankTopK: 200 }
  ]
};
await client.createCollection("knowledge_with_semantics", hybridSchema);
```

### Embedding Hierarchical Data

When inserting hierarchical data, use embeddings that preserve **hyperbolic distance**:
- Poincaré embeddings
- Lorentzian embeddings (e.g., from `hyperspace-sdk-ts` math utilities)
- Standard embeddings from models fine-tuned on ontological data

```typescript
import { HyperbolicMath } from 'hyperspace-sdk-ts';

// Project a Euclidean embedding onto the Lorentz hyperboloid
const lorentzVector = HyperbolicMath.toLorentz(euclideanEmbedding);

// For hybrid: concatenate Lorentz (33d) + Euclidean (Nd)
const hybridVector = [...lorentzVector.slice(0, 33), ...semanticEmbedding];
```

### Traversal vs. ANN Search

| Need | Use |
|------|-----|
| "Find similar vectors" | `search()` or `searchText()` |
| "Follow edges in the graph" | `traverse()` |
| "Show concept hierarchy" | `getSubsumptionTree()` |
| "What is this concept's parent?" | `getConceptParents()` |
| "Visualize the graph" | `exploreGraph()` |
| "Find topic clusters" | `findSemanticClusters()` |

---

## See Also

- [hyperspacedb-core](../hyperspacedb-core/SKILL.md) — CRUD and search operations
- [hyperspacedb-cognitive](../hyperspacedb-cognitive/SKILL.md) — reasoning stability and momentum
