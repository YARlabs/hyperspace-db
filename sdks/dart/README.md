# HyperspaceDB Dart SDK

Official Dart/Flutter SDK for [HyperspaceDB](https://github.com/yarlabs/hyperspace-db).

## Installation

Add to `pubspec.yaml`:

```yaml
dependencies:
  hyperspacedb:
    path: ./sdks/dart # Or from pub.dev once published
```

## Quick Start

```dart
import 'package:hyperspacedb/hyperspacedb.dart';

void main() async {
  final client = HyperspaceClient('localhost', 50051, apiKey: 'I_LOVE_HYPERSPACEDB');
  
  await client.createCollection('docs_dart', 128, 'cosine');
  
  await client.insert(1, [0.1, 0.2, 0.3], collection: 'docs_dart');
  
  final results = await client.search([0.1, 0.2, 0.3], 5, collection: 'docs_dart');
  print(results);
}
```

## Hybrid & Lexical Search (BM25)

Combine semantic vector search with BM25 lexical ranking:

```dart
final results = await client.search(
  vector, 
  10, 
  collection: 'docs',
  hybridQuery: 'advanced spatial indexing',
  hybridAlpha: 0.7, // 70% vector, 30% BM25
);

// Pure Lexical Search
final lexicalResults = await client.searchText(
  'hyperspatial retrieval', 
  10, 
  collection: 'docs',
  bm25Options: Bm25Options(method: 'bm25plus'),
);
```

## Cognitive Math (Spatial AI)

```dart
import 'package:hyperspacedb/src/math.dart';

final entropy = localEntropy(thoughtVector, neighbors, 1.0);
final stability = lyapunovConvergence(thoughtChain, 1.0);
```
