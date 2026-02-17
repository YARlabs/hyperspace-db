# HyperspaceDB TypeScript SDK

Official TypeScript client for HyperspaceDB gRPC API.

Use this SDK for:
- collection lifecycle management
- vector insert and search
- high-throughput batched search (`searchBatch`)
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

Insert one vector.

- accepts `number[]`, `Float32Array`, `Float64Array`
- `durability` defaults to `DurabilityLevel.DEFAULT_LEVEL`

### `search(vector, topK, collection?)`

Run nearest-neighbor search and return:

```ts
{ id: number, distance: number, metadata: Record<string, string> }[]
```

### `searchBatch(vectors, topK, collection?)`

Run multiple searches in one gRPC request:

```ts
{ id: number, distance: number, metadata: Record<string, string> }[][]
```

Use `searchBatch` for concurrency-heavy workloads to reduce RPC overhead.

### `close()`

Close underlying gRPC channel.

## Performance Notes

- Prefer `searchBatch` for throughput benchmarks and high-QPS services.
- Reuse one client instance per process or worker.
- For large inserts, send vectors in chunks.

## Error Handling

All methods reject on transport/protocol errors.

Example:

```ts
try {
  const res = await client.search([0.1, 0.2, 0.3], 10, "my_collection");
  console.log(res);
} catch (err) {
  console.error("Hyperspace request failed:", err);
}
```

## Notes for Package Consumers

- This SDK targets gRPC data plane operations.
- Control plane endpoints (`/api/*`) are HTTP and are not part of this package.

