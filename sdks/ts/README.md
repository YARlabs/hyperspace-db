# HyperspaceDB TypeScript SDK

Official TypeScript client for HyperspaceDB gRPC API v3.1.0.

Use this SDK for:
- collection lifecycle management
- vector insert and search
- high-throughput batched search (`searchBatch`)
- bulk insertion (`batchInsert`)
- advanced filtering and hybrid search
- typed metadata (`string | number | boolean`)
- graph traversal APIs (`getNode`, `getNeighbors`, `getConceptParents`, `traverse`, `findSemanticClusters`)
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
  await client.createCollection(collection, 3, "cosine");

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

### `createCollection(name, dimension, metric)`

Create a new collection.

- `metric`: `"l2" | "cosine" | "poincare" | "lorentz"`

### `deleteCollection(name)`

Delete collection and all its data.

### `listCollections()`

Retrieve all active collections for the current tenant.
Returns `Promise<CollectionInfo[]>`.

```ts
const collections = await client.listCollections();
for (const col of collections) {
  console.log(`${col.name}: dim=${col.dimension}, metric=${col.metric}, count=${col.count}`);
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

### Geometric Filters (New in v3.0)

HyperspaceDB v3.0 introduces advanced spatial filters that run on the engine level:

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
```

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
const entropy = CognitiveMath.localEntropy(candidateThought, neighbors, 1.0);

// 2. Proof of Convergence (Negative derivative = convergence)
const stability = CognitiveMath.lyapunovConvergence(chainOfThought, 1.0);

// 3. Extrapolate next thought (Koopman linearization)
const nextThought = CognitiveMath.koopmanExtrapolate(past, current, 1.0, 1.0);

// 4. Phase-Locked Loop for topic tracking
const syncedThought = CognitiveMath.contextResonance(thought, globalContext, 0.5, 1.0);
```

## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** — each geometry (`l2`, `cosine`, `poincare`, `lorentz`) can have its own backend independently.

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

