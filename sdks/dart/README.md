# HyperspaceDB Dart SDK

Official Dart/Flutter SDK for [HyperspaceDB](https://github.com/yarlabs/hyperspace-db). A hyper-fast, lock-free Vector Database built in Rust with native support for Hyperbolic Space and Agentic AI workflows.

## 🚀 Features

- **100% Feature Parity**: Supports all gRPC v3.1.0 operations (Parity with TS/Python).
- **Multi-Geometry**: Native support for `Euclidean`, `Cosine`, `Poincaré`, and `Lorentz` metrics.
- **Hybrid Retrieval**: Seamlessly combine semantic vector search with BM25 lexical ranking.
- **Recursive Filtering**: Complex logical filters (`AND`, `OR`, `NOT`) and spatial constraints (`InCone`, `InBall`, `InBox`).
- **High Throughput**: Optimized `batchInsert` and `searchBatch` for production workloads.
- **Cognitive Math**: Built-in tools for Agentic AI (Hallucination detection, Proof of convergence).
- **Delta Sync**: Merkle-tree based state synchronization for distributed systems.
- **Real-time CDC**: Subscribe to live data change events from the server.
- **Typed Metadata**: Native mapping for `String`, `int`, `double`, and `bool` values.

## 📦 Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  hyperspacedb:
    path: ./sdks/dart # Path to the SDK
  grpc: ^3.2.4
  fixnum: ^1.1.0
```

## 🛠 Quick Start

```dart
import 'package:hyperspacedb/hyperspacedb.dart';

void main() async {
  // 1. Initialize Client
  final client = HyperspaceClient(
    'localhost', 
    50051, 
    apiKey: 'I_LOVE_HYPERSPACEDB',
    tenantId: 'agent_alpha'
  );
  
  const col = 'knowledge_graph';

  // 2. Create Collection with Schema (Multi-Vector & MRL support)
  await client.createCollection(col, pb.CollectionSchema(
    components: [
      pb.VectorComponent(name: 'primary', metric: 'poincare', fullDimension: 128, weight: 1.0)
    ]
  ));
  
  // 3. Batch Insert Vectors with Typed Metadata
  await client.batchInsert([
    {
      'id': 1,
      'vector': List.filled(128, 0.1),
      'typedMetadata': {'type': 'concept', 'level': 1}
    },
    {
      'id': 2,
      'vector': List.filled(128, 0.2),
      'typedMetadata': {'type': 'concept', 'level': 2}
    }
  ], collection: col);
  
  // 4. Hybrid Search with Recursive Filters
  final results = await client.search(
    List.filled(128, 0.15), 
    10, 
    collection: col,
    hybridQuery: 'semantic resonance',
    hybridAlpha: 0.5,
    filters: [
      Filter.and([
        Filter.match('type', 'concept'),
        Filter.range('level', gte: 1, lte: 5),
      ])
    ]
  );
  
  print('Found ${results.length} results');
  await client.close();
}
```

## 🧠 Matryoshka Representation Learning (MRL)

HyperspaceDB supports MRL through its **Cascade Pipeline**. This allows you to perform initial fast search on a truncated low-dimensional vector (e.g., 64D) and then rerank the results using the full vector (e.g., 1024D).

```dart
final schema = pb.CollectionSchema(
  components: [
    pb.VectorComponent(name: 'main', metric: 'lorentz', fullDimension: 1025)
  ],
  cascadePipeline: [
    pb.MrlLayer(
      componentName: 'main',
      cutoffDimension: 129, // 128D (+1) fast search
      storeInRam: true,
      rerankTopK: 100
    )
  ]
);
await client.createCollection('mrl_docs', schema);
```

## 🔍 API Overview

### Collection Management
- `createCollection(name, schema)`: Create a new space with [CollectionSchema].
- `deleteCollection(name)`: Remove a space.
- `listCollections()`: List all available collections.
- `getCollectionStats(name)`: Get real-time stats including **Disk**, **RAM** and **Schema** details.
- `exists(name)`: Check if a collection exists.

### Data Operations
- `insert(id, vector, {metadata, typedMetadata})`: Single point ingestion.
- `batchInsert(items)`: Bulk ingestion for high throughput.
- `getPoints(ids)`: Retrieve points by ID in one call.
- `updatePayload(id, typedMetadata)`: Partial update of metadata (patching).
- `delete(id)`: Remove a vector by ID.
- `scroll({limit, offset, filters})`: Paginated scanning of the database.
- `count({filters})`: Get the number of points matching criteria.

### Advanced Search
- `search(vector, topK, {filters, hybridQuery, hybridAlpha, componentWeights, ...})`: Standard ANN search.
- `searchText(text, topK, {filters, ...})`: Server-side vectorized search.
- `searchBatch(vectors, topK)`: Multiple searches in one RPC call.
- `searchMultiCollection(collections, vector, topK)`: Parallel search across multiple metrics.

### Spatial Filtering (New in v3.1)
Build complex filters using the `Filter` class:
```dart
final filter = Filter.or([
  Filter.inBall([0, 0, 0], 0.5), // Points within radius
  Filter.inBox([-1, -1], [1, 1]), // Points within bounds
  Filter.inCone(axis, apertures, cen), // Angular field of view
]);
```

### Admin & Maintenance
- `createSnapshot()`: Trigger a database snapshot.
- `vacuum()`: Manually trigger garbage collection and index optimization.
- `rebuildIndex(name)`: Fully restructure the HNSW index.
- `healthCheck()`: Verify if the database is ready to serve.

### Real-time & Sync
- `subscribeToEvents(types, {collection})`: Listen to `Insert`, `Delete`, `Snapshot` events via Stream.
- `syncHandshake/Pull/Push`: Merkle-tree based delta synchronization protocols.

## 🧠 Cognitive Math & Analytics

The SDK includes offline tools for analyzing Agentic AI behavior in Hyperbolic space:

```dart
import 'package:hyperspacedb/src/math.dart';

// 1. Analyze Dataset Geometry
final analysis = HyperspaceClient.analyzeDeltaHyperbolicity(vectors);
print('Recommended Metric: ${analysis['recommendation']}'); // poincare, lorentz, or l2

// 2. Stability Analysis
final stability = lyapunovConvergence(chainOfThoughtTrajectory);
if (stability < 0) print('AI reasoning is converging stable.');

// 3. Hallucination Detection
final entropy = localEntropy(thoughtVector, neighborVectors);
if (entropy > 0.8) print('Potential hallucination detected (high geometric entropy)');
```

## ⚡ Performance Tips

- **Reuse Client**: Initialize one `HyperspaceClient` and reuse it across your application.
- **Batching**: Always use `batchInsert` for loading more than 100 points.
- **Typed Metadata**: Use `typedMetadata` instead of `metadata` (string-only) for efficient range filtering.
- **Streams**: Use `subscribeToEvents` for real-time UI updates in Flutter.

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
