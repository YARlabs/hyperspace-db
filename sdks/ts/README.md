# HyperspaceDB TypeScript SDK

Official TypeScript client for HyperspaceDB gRPC API.

Use this SDK for:
- collection lifecycle management
- vector insert and search
- high-throughput batched search (`searchBatch`)
- bulk insertion (`batchInsert`)
- advanced filtering and hybrid search
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

- `metric`: `"l2" | "cosine" | "poincare"`

### `deleteCollection(name)`

Delete collection and all its data.

### `insert(id, vector, meta?, collection?, durability?)`

Insert one vector. Accepts `number[]`, `Float32Array`, `Float64Array`.

### `batchInsert(items, collection?, durability?)`

Efficient bulk insertion.
```ts
await client.batchInsert([
  { id: 10, vector: [0.1, 0.1, 0.1], metadata: { tag: "a" } },
  { id: 11, vector: [0.2, 0.2, 0.2], metadata: { tag: "b" } }
], "my_collection");
```

### `search(vector, topK, collection?, options?)`

Run nearest-neighbor search. 
Options include `filters`, `hybridQuery`, and `hybridAlpha`.

```ts
const results = await client.search(vector, 10, "coll", {
  filters: [
    { match: { key: "category", value: "electronics" } },
    { range: { key: "price", gte: 100, lte: 500 } }
  ],
  hybridQuery: "latest smartphone",
  hybridAlpha: 0.5
});
```

### `searchBatch(vectors, topK, collection?)`

Run multiple searches in one gRPC request to reduce RPC overhead.

### `getDigest(collection?)`

Retrieve collection stats and logical clock.

### `close()`

Close underlying gRPC channel.

## Performance Notes

- Prefer `searchBatch` and `batchInsert` for throughput-heavy services.
- Reuse one client instance per process or worker.

## Error Handling

All methods reject on transport/protocol errors. Targets gRPC data plane operations.
For control plane endpoints (`/api/*`), use regular HTTP requests to the server's HTTP port.

