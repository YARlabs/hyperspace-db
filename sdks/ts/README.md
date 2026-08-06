# HyperspaceDB TypeScript SDK

Official TypeScript client for HyperspaceDB gRPC API v3.1.1.

Use this SDK for:
- collection lifecycle management
- vector insert and search
- high-throughput batched search (`searchBatch`)
- bulk insertion (`batchInsert`)
- advanced filtering and hybrid search
- recursive logical filters (`AND`, `OR`, `NOT`)
- bulk point retrieval (`getPoints`)
- metadata updates (`updatePayload`)
- paginated scanning (`scroll`)
- filtered point counting (`count`)
- health monitoring (`healthCheck`)
- typed metadata (`string | number | boolean`)
- graph traversal APIs (`getNode`, `getNeighbors`, `getSubsumptionTree`, `getConceptParents`, `traverse`, `exploreGraph`, `findSemanticClusters`)
- rebuild with metadata pruning (`rebuildIndexWithFilter`)
- multi-tenant authentication headers (`x-api-key`, `x-hyperspace-user-id`)

## Requirements

- Node.js 18+
- Running HyperspaceDB server (default gRPC endpoint: `localhost:50051`)

## Installation

```bash
npm install hyperspace-sdk-ts
```

## Quick Start

```ts
import { HyperspaceClient } from "hyperspace-sdk-ts";

async function main() {
  const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");
  const collection = "docs_ts";

  await client.deleteCollection(collection).catch(() => {});
  await client.createCollection(collection, {
    components: [
      { name: "primary", metric: "cosine", full_dimension: 3, weight: 1.0 }
    ],
    cascade_pipeline: []
  });

  await client.insert(1, [0.1, 0.2, 0.3], { source: "demo" }, collection);
  await client.insert(2, [0.2, 0.1, 0.4], { source: "demo" }, collection);

  // Delete vector by ID
  await client.delete(1);

  const results = await client.search([0.1, 0.2, 0.3], 5, collection);
  console.log(results);

  client.close();
}

main().catch(console.error);
```

## API Overview

### `new HyperspaceClient(host?, apiKey?, userId?)`

- `host`: gRPC endpoint, default `localhost:50051`
- `apiKey`: optional API key
- `userId`: optional tenant/user ID

### `createCollection(name, schema)`

Create a new collection using a `CollectionSchema`.

- `metric`: `"l2" | "cosine" | "poincare" | "lorentz" | "hybrid"`

```ts
await client.createCollection("my_coll", {
  components: [
    { name: "primary", metric: "lorentz", full_dimension: 129, weight: 1.0 }
  ],
  cascade_pipeline: [
    { component_name: "primary", cutoff_dimension: 17, store_in_ram: true, rerank_top_k: 100 }
  ]
});
```

### `deleteCollection(name)`

Delete collection and all its data.

### `listCollections()`

Retrieve all active collections for the current tenant.
Returns `Promise<CollectionInfo[]>`.

```ts
const collections = await client.listCollections();
for (const col of collections) {
  console.log(`${col.name}: count=${col.count}, schema=${JSON.stringify(col.schema)}`);
}
```


### `insert(id, vector, meta?, collection?, durability?)`

Insert one vector. Accepts `number[]`, `Float32Array`, `Float64Array`.
Optional `typedMetadata` supports typed values for range/boolean filters.

### `insertText(id, text, meta?, collection?, durability?)`

Insert text to be vectorized and stored on the server side (Server-Side Embedding).

### `vectorize(text, metric?)`

Convert text to a dense vector using the server's embedding engine.
- `metric`: defaults to `"l2"`.

### `batchInsert(items, collection?, durability?)`

Efficient bulk insertion.
```ts
await client.batchInsert([
  { id: 10, vector: [0.1, 0.1, 0.1], metadata: { tag: "a" } },
  { id: 11, vector: [0.2, 0.2, 0.2], metadata: { tag: "b" } }
], "my_collection");
```

### `search(vector, topK, collection?, options?)`

Run nearest-neighbor search with a raw vector. 

### `searchText(text, topK, collection?, options?)`

Run nearest-neighbor search using text input. The text is vectorized on the server before searching.

```ts
const results = await client.searchText("How to use HyperspaceDB?", 10, "coll", {
  filters: [
    { match: { key: "category", value: "docs" } }
  ]
});
```

### Geometric Filters

HyperspaceDB introduces advanced spatial filters that run on the engine level:

```ts
// 1. Proximity Search (Ball)
const ballFilter = {
  inBall: { center: [0.1, 0.2, 0.3], radius: 0.5 }
};

// 2. Workspace Constraints (Box)
const boxFilter = {
  inBox: { minBounds: [-1, -1, -1], maxBounds: [1, 1, 1] }
};

// 3. Field of View / Angular Search (Cone)
const coneFilter = {
  inCone: { axes: [1.0, 0.0, 0.0], apertures: [0.5], cen: 0.01 }
};

const results = await client.search([0.1, 0.2, 0.3], 10, "coll", {
  filters: [ballFilter, boxFilter]
});

// 4. Recursive Logical Filters
const logicFilter = {
  and: [
    { match: { key: "status", value: "active" } },
    { or: [
        { range: { key: "score", gte: 0.8 } },
        { match: { key: "priority", value: "high" } }
      ]
    }
  ]
};
```

### `getPoints(ids, collection?)`

Retrieve multiple points by their IDs.
Returns `Promise<Point[]>`.

### `updatePayload(id, metadata, collection?)`

Patch metadata for an existing point.
```ts
await client.updatePayload(1, { status: "processed", tags: "updated" });
```

### `scroll(limit?, offset?, filters?, collection?)`

Paginated retrieval of points with optional filtering.
```ts
const points = await client.scroll(50, 0, [{ match: { key: "category", value: "news" } }]);
```

### `count(filters?, collection?)`

Count points matching filters.
```ts
const total = await client.count([{ range: { key: "price", lte: 100 } }]);
```

### `healthCheck()`

Check server connectivity. Returns `"ONLINE"` or throws.

### Hybrid & Lexical Search (BM25)

HyperspaceDB supports combined lexical and vector ranking.

```ts
// Hybrid Search (Semantic Vector + BM25 Lexical)
const results = await client.search([0.1, -0.2, 0.5], 10, "coll", {
  hybridQuery: "hybrid search implementation",
  hybridAlpha: 0.7, // 70% vector weight
  bm25: {
    method: "bm25plus",
    language: "english"
  }
});

// Or Pure Lexical Search via searchText
const lexicalResults = await client.searchText("full-text query", 10, "coll", {
  bm25: { method: "lucene" }
});
```

### `searchBatch(vectors, topK, collection?)`

Run multiple searches in one gRPC request to reduce RPC overhead.

### `searchWasserstein(vector, topK, collection?)`

Execute O(N) Cross-Feature Match (1D L1 CDF distance) instead of generic Poincare/L2. Ideal for comparing distributions.

### `searchMultiCollection(vector, collections, topK)`

Submit one vector and run parallel searches across multiple collections in one batch request (e.g. for Multi-Geometry benchmarks comparing L2, Cosine, Poincare, Lorentz).

### `getDigest(collection?)`

Retrieve collection stats and logical clock.

### `close()`

Close underlying gRPC channel.

### `subscribeToEvents(options, onEvent, onError?)`

Subscribe to CDC stream events from server:

```ts
const stream = client.subscribeToEvents(
  { types: ["insert", "delete"], collection: "docs_ts" },
  (event) => console.log("event:", event.toObject()),
  (err) => console.error(err),
);
```

### `rebuildIndex(collection)`

Trigger index rebuild/vacuum for a collection.

### `triggerReconsolidation(collection, targetVector, learningRate)`

Trigger AI Sleep Mode natively: updates parameters using Flow Matching (Riemannian SGD) instantly via the database engine.

### `rebuildIndexWithFilter(collection, filter)`

Rebuild with metadata pruning for sleep/reconsolidation workflows.

```ts
await client.rebuildIndexWithFilter("docs_ts", {
  key: "energy",
  op: "lt",
  value: 0.1,
});
```

### `HyperbolicMath`

```ts
import { HyperbolicMath } from "hyperspace-sdk-ts";

const z = HyperbolicMath.mobiusAdd([0.1, 0.0], [0.2, 0.0]);
```

Provided utilities:
- `mobiusAdd(x, y, c?)`
- `expMap(x, v, c?)`
- `logMap(x, y, c?)`
- `riemannianGradient(x, euclideanGrad, c?)`
- `parallelTransport(x, y, v, c?)`
- `frechetMean(points, c?, maxIter?, tol?)`

### `CognitiveMath` (Spatial AI Engine)

Provides advanced tools for Agentic AI, running entirely on the client side:

```ts
import { CognitiveMath } from "hyperspace-sdk-ts";

// 1. Detect Hallucinations (Entropy approaches 1.0)
const entropy = client.localEntropy(candidateThought, neighbors, 1.0);

// 2. Proof of Convergence (Negative derivative = convergence)
const stability = await client.getTrustScore([1, 2, 3]);

// 3. Extrapolate next thought (Koopman linearization)
const nextThought = await client.predictMomentum([10, 11], 1.0);

// 4. Phase-Locked Loop for topic tracking
const syncedThought = CognitiveMath.contextResonance(thought, globalContext, 0.5, 1.0);

// 5. Predict Semantic Relation (A + R ≈ B)
const relation = await client.predictRelation(1, 2);
```

## Implicit Graph Engine (v3.1.1)

HyperspaceDB treats your vectors as nodes in a dynamic graph. Relationships are inferred from the geometry:
- **Lorentz / Poincare**: Hierarchy and subsumption (light cones).
- **L2 / Cosine**: Semantic similarity and adjacency.

### Subsumption Trees
Extract directed hierarchies from Lorentz-encoded data:
```ts
const tree = await client.getSubsumptionTree(1, 5);
```

### Advanced Traversal
Navigate the graph using physical kernels:
```ts
const results = await client.traverse({
    startId: 1,
    traversalMode: 2, // 0: GREEDY, 1: DIFFUSIVE, 2: MOMENTUM
    breadthLimit: 5
});
```
```

## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** — each geometry (`l2`, `cosine`, `poincare`, `lorentz`, `hybrid`) can have its own backend independently.

### Server-Side Config (`.env`)

```env
HYPERSPACE_EMBED=true

# Cosine via OpenAI
HS_EMBED_COSINE_PROVIDER=openai
HS_EMBED_COSINE_EMBED_MODEL=text-embedding-3-small
HS_EMBED_COSINE_API_KEY=sk-...

# Poincaré via HuggingFace Hub (downloads model.onnx + tokenizer.json)
HS_EMBED_POINCARE_PROVIDER=huggingface
HS_EMBED_POINCARE_HF_MODEL_ID=your-org/cde-spatial-poincare-128d
HS_EMBED_POINCARE_DIM=128
HF_TOKEN=hf_...          # Optional — for gated/private models

# Lorentz via local ONNX file
HS_EMBED_LORENTZ_PROVIDER=local
HS_EMBED_LORENTZ_MODEL_PATH=./models/lorentz_128d.onnx
HS_EMBED_LORENTZ_TOKENIZER_PATH=./models/lorentz_128d_tokenizer.json
HS_EMBED_LORENTZ_DIM=129  # spatial_dim + 1 for time component
```

### Client-Side Embedder

```ts
import { OpenAIEmbedder, HuggingFaceEmbedder, LocalOnnxEmbedder } from "hyperspace-sdk-ts";

// OpenAI API
const embedder = new OpenAIEmbedder({ apiKey: "sk-...", model: "text-embedding-3-small" });
const vector = await embedder.encode("my text");

// HuggingFace Hub — downloads model.onnx + tokenizer.json on first use
const embedder = new HuggingFaceEmbedder({
  modelId: "BAAI/bge-small-en-v1.5",
  geometry: "cosine",
  hfToken: process.env.HF_TOKEN,  // Optional
});
const vector = await embedder.encode("my text");

// Local ONNX file
const embedder = new LocalOnnxEmbedder({
  modelPath: "./models/bge-small.onnx",
  tokenizerPath: "./models/bge-small-tokenizer.json",
  geometry: "cosine",
});
const vector = await embedder.encode("my text");
```

### Supported Geometries

| Geometry | Post-Processing | Best For |
|---|---|---|
| `cosine` | Unit normalize | Semantic similarity |
| `l2` | Unit normalize | Euclidean distance tasks |
| `poincare` | Clamp to unit ball | Hierarchical data (trees, ontologies) |
| `lorentz` | None (model handles it) | Mixed hierarchical + semantic |

## Performance Notes

- Prefer `searchBatch` and `batchInsert` for throughput-heavy services.
- Reuse one client instance per process or worker.
- For `lorentz` geometry, dimension = spatial_dim + 1 (the time component x₀).
- For `huggingface` provider, models are cached locally after first download.

## Error Handling

All methods reject on transport/protocol errors. Targets gRPC data plane operations.
For control plane endpoints (`/api/*`), use regular HTTP requests to the server's HTTP port.

## Zero-Knowledge Client-Side Encryption (ZK-Privacy)

HyperspaceDB v3.1.1 introduces Zero-Knowledge client-side encryption (ZK-Privacy). All private data (vectors, metadata, payloads) are encrypted/obfuscated *before* they leave the client. The database server never sees the raw vectors or plaintext data, ensuring maximum security even in public or untrusted DePIN environments.

### Key Features
1. **Vector Projection**: High-dimensional vectors are projected using a deterministic orthogonal matrix (or Lorentz boost matrix for hyperbolic spaces) generated from the collection key. This preserves distances (L2, Cosine, Lorentz) while hiding the vector coordinates.
2. **Anisotropic Noise Injection**: Injecting subtle deterministic noise into the vectors to prevent reconstruction attacks.
3. **Payload Encryption**: Sidecar payloads are encrypted client-side using AES-256-GCM before being sent to the database.
4. **Metadata Hashing**: Metadata keys and values are obfuscated using HMAC-SHA256.

### Usage Example

```ts
import { HyperspaceClient } from "hyperspace-sdk-ts";

async function main() {
  const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");
  
  const collection = "encrypted_docs";
  const secretKey = "my-super-secret-key";

  // Register collection key to enable automatic client-side encryption/decryption
  // noiseSigma defaults to 0.02 (2% anisotropic noise)
  client.registerCollectionKey(collection, secretKey, "cosine", 0.02);

  // 1. Insert vector (will be projected, noise injected, payload encrypted, metadata hashed)
  await client.insert(
    1, 
    [0.1, 0.2, 0.3], 
    { category: "confidential" }, 
    collection,
    undefined,
    undefined,
    Buffer.from("This is a highly secret document payload", "utf-8")
  );

  // 2. Search (search vector is projected and noise-injected; results are decrypted locally)
  const results = await client.search([0.1, 0.2, 0.3], 5, collection, {
    // Filters are automatically hashed client-side
    filter: { category: "confidential" }
  });

  for (const res of results) {
    console.log(`ID: ${res.id}, Distance: ${res.distance}`);
    if (res.payload) {
      console.log(`Decrypted Payload: ${Buffer.from(res.payload).toString("utf-8")}`);
    }
  }

  client.close();
}

main().catch(console.error);
```

