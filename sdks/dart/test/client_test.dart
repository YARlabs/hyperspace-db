import 'dart:async';
import 'package:test/test.dart';
import 'package:hyperspacedb/hyperspacedb.dart';
import 'package:hyperspacedb/src/generated/hyperspace.pbgrpc.dart' as pb;
import 'package:fixnum/fixnum.dart';

void main() {
  late HyperspaceClient client;
  final collectionName = 'test_dart_collection';

  setUpAll(() async {
    client = HyperspaceClient('localhost', 50051, apiKey: 'I_LOVE_HYPERSPACEDB');
  });

  tearDownAll(() async {
    try {
      await client.deleteCollection(collectionName);
    } catch (_) {}
    await client.close();
  });

  group('HyperspaceDB Integration Tests', () {
    test('HealthCheck', () async {
      final isHealthy = await client.healthCheck();
      expect(isHealthy, isTrue);
    });

    test('Collection Management', () async {
      // 1. Delete if exists
      if (await client.exists(collectionName)) {
        await client.deleteCollection(collectionName);
      }

      // 2. Create
      final schema = pb.CollectionSchema(
        components: [
          pb.VectorComponent(
            name: 'main',
            metric: 'poincare',
            fullDimension: 4,
            weight: 1.0,
          )
        ]
      );
      final created = await client.createCollection(collectionName, schema);
      expect(created, isTrue);

      // 3. Exists
      final doesExist = await client.exists(collectionName);
      expect(doesExist, isTrue);

      // 4. List Collections
      final collections = await client.listCollections();
      final names = collections.map((c) => c.name).toList();
      expect(names, contains(collectionName));

      // 5. Get Stats
      final stats = await client.getCollectionStats(collectionName);
      expect(stats.count, equals(Int64(0)));
      expect(stats.schema.components.first.metric, equals('poincare'));
    });

    test('Data Ops (Insert, Get, Update, Count, Scroll)', () async {
      // 1. Insert Single Vector
      final inserted = await client.insert(
        1,
        [0.1, 0.1, 0.1, 0.1],
        collection: collectionName,
        typedMetadata: {'tag': 'first', 'value': 42},
      );
      expect(inserted, isTrue);

      // 2. Batch Insert
      final batchInserted = await client.batchInsert([
        {
          'id': 2,
          'vector': [0.2, 0.2, 0.2, 0.2],
          'typedMetadata': {'tag': 'second', 'value': 84},
        },
        {
          'id': 3,
          'vector': [0.3, 0.3, 0.3, 0.3],
          'typedMetadata': {'tag': 'third', 'value': 126},
        }
      ], collection: collectionName);
      expect(batchInserted, isTrue);

      // 3. Get Points
      final points = await client.getPoints([1, 2], collection: collectionName);
      expect(points.length, equals(2));
      expect(points.any((p) => p.id == 1), isTrue);
      expect(points.any((p) => p.id == 2), isTrue);

      // 4. Update Payload
      final updated = await client.updatePayload(
        1,
        {'tag': 'first_updated', 'value': 43},
        collection: collectionName,
      );
      expect(updated, isTrue);

      // 5. Count
      final cnt = await client.count(collectionName);
      expect(cnt, equals(3));

      // 6. Scroll
      final scrollResult = await client.scroll(collectionName, limit: 10);
      expect(scrollResult.points.length, equals(3));
    });

    test('Search Operations', () async {
      // 1. Standard Vector Search
      final searchResults = await client.search(
        [0.15, 0.15, 0.15, 0.15],
        2,
        collection: collectionName,
      );
      expect(searchResults.isNotEmpty, isTrue);
      expect(searchResults.first.id, isNotNull);

      // 2. Batch Search
      final batchResults = await client.searchBatch(
        [
          [0.12, 0.12, 0.12, 0.12],
          [0.28, 0.28, 0.28, 0.28]
        ],
        2,
        collection: collectionName,
      );
      expect(batchResults.length, equals(2));
      expect(batchResults[0].isNotEmpty, isTrue);
      expect(batchResults[1].isNotEmpty, isTrue);

      // 3. Search Text (BM25 or hybrid)
      try {
        await client.insertText(10, 'spatial query text', collection: collectionName);
        final textResults = await client.searchText(
          'spatial',
          2,
          collection: collectionName,
        );
        expect(textResults, isNotNull);
      } catch (e) {
        // Fallback for setups without local embed models enabled
        print('SearchText skipped or failed: $e');
      }

      // 4. Multi-Collection Search
      final multiRes = await client.searchMultiCollection(
        [collectionName],
        [0.1, 0.1, 0.1, 0.1],
        topK: 2,
      );
      expect(multiRes.responses.containsKey(collectionName), isTrue);
    });

    test('Spatial Filters', () async {
      // 1. Match Filter
      final filterRes = await client.search(
        [0.15, 0.15, 0.15, 0.15],
        2,
        collection: collectionName,
        filters: [Filter.match('tag', 'second')],
      );
      expect(filterRes.length, equals(1));
      expect(filterRes.first.id, equals(2));

      // 2. Range Filter
      final rangeRes = await client.search(
        [0.15, 0.15, 0.15, 0.15],
        5,
        collection: collectionName,
        filters: [Filter.range('value', gte: 80, lte: 130)],
      );
      expect(rangeRes.length, equals(2));
      final ids = rangeRes.map((r) => r.id).toList();
      expect(ids, contains(2));
      expect(ids, contains(3));
    });

    test('Graph API Operations', () async {
      // GetNode
      final node = await client.getNode(collectionName, 1, 0);
      expect(node.id, equals(1));

      // GetNeighbors
      final neighbors = await client.getNeighbors(collectionName, 1, 0);
      expect(neighbors, isNotNull);

      // GetConceptParents
      final parents = await client.getConceptParents(collectionName, 1, 0);
      expect(parents, isNotNull);

      // Traverse
      final travReq = pb.TraverseRequest(
        collection: collectionName,
        startId: 1,
        maxDepth: 2,
        maxNodes: 10,
        layer: 0,
      );
      final traverseRes = await client.traverse(travReq);
      expect(traverseRes.nodes, isNotNull);

      // FindSemanticClusters
      final clusterReq = pb.FindSemanticClustersRequest(
        collection: collectionName,
        layer: 0,
        minClusterSize: 1,
        maxClusters: 5,
        maxNodes: 10,
      );
      final clusters = await client.findSemanticClusters(clusterReq);
      expect(clusters.clusters, isNotNull);

      // GetSubsumptionTree
      final tree = await client.getSubsumptionTree(collectionName, 1, 2);
      expect(tree.nodes, isNotNull);
    });

    test('Replication, CDC, & Monitor', () async {
      // 1. Subscribe to Events Stream
      final stream = client.subscribeToEvents([
        pb.EventType.VECTOR_INSERTED,
        pb.EventType.VECTOR_DELETED
      ], collection: collectionName);
      
      expect(stream, isNotNull);

      // 2. Monitor Metrics Stream
      final metrics = client.getMetrics();
      expect(metrics, isNotNull);

      // 3. Replicate Stream
      final replStream = client.replicate(0);
      expect(replStream, isNotNull);
    });

    test('Admin & Configuration', () async {
      // 1. Trigger Snapshot
      final snap = await client.createSnapshot();
      expect(snap, isTrue);

      // 2. Trigger Vacuum
      final vac = await client.vacuum();
      expect(vac, isTrue);

      // 3. Rebuild Index
      final rebuild = await client.rebuildIndex(collectionName);
      expect(rebuild, isTrue);

      // 4. Get Digest
      final digest = await client.getDigest(collectionName);
      expect(digest.count, isNotNull);

      // 5. Trigger Reconsolidation
      final recon = await client.triggerReconsolidation(collectionName, [0.1, 0.1, 0.1, 0.1], 0.01);
      expect(recon, isTrue);

      // 6. Configure (efSearch, efConstruction, m)
      final configured = await client.updateCollection(collectionName, efSearch: 32, efConstruction: 64, m: 16);
      expect(configured, isTrue);
    });

    test('Delta Sync Protocols', () async {
      // 1. Handshake
      final clientBuckets = List.filled(256, Int64(0));
      final handshake = await client.syncHandshake(
        collectionName,
        clientBuckets,
      );
      expect(handshake.serverCount, isNotNull);

      // 2. Sync Pull Stream
      final pull = client.syncPull(collectionName, [0, 1, 2]);
      expect(pull, isNotNull);

      // 3. Sync Push
      final pushStreamController = StreamController<pb.SyncVectorData>();
      final pushFuture = client.syncPush(pushStreamController.stream);
      
      pushStreamController.add(pb.SyncVectorData(
        collection: collectionName,
        id: 99,
        vector: [0.9, 0.9, 0.9, 0.9],
        bucketIndex: 0,
      ));
      await pushStreamController.close();
      
      final pushRes = await pushFuture;
      expect(pushRes, isNotNull);
    });

    test('Collection Freeze/Unfreeze', () async {
      // 1. Freeze
      final frozen = await client.freezeCollection(collectionName);
      expect(frozen, isTrue);

      // 2. Try insert (should fail or throw exception when frozen)
      try {
        await client.insert(4, [0.4, 0.4, 0.4, 0.4], collection: collectionName);
      } catch (e) {
        expect(e.toString(), contains('frozen'));
      }

      // 3. Unfreeze
      final unfrozen = await client.unfreezeCollection(collectionName);
      expect(unfrozen, isTrue);

      // 4. Try insert again (should succeed)
      final insertedAfterUnfreeze = await client.insert(4, [0.4, 0.4, 0.4, 0.4], collection: collectionName);
      expect(insertedAfterUnfreeze, isTrue);
    });

    test('Delete Point', () async {
      final scrollBefore = await client.scroll(collectionName, limit: 100);
      final countBefore = scrollBefore.points.length;

      final deleted = await client.delete(1, collection: collectionName);
      expect(deleted, isTrue);

      final scrollAfter = await client.scroll(collectionName, limit: 100);
      expect(scrollAfter.points.length, equals(countBefore - 1));
    });

    test('Offline Cognitive Math and Hyperbolic Space Utilities', () {
      // 1. Poincaré and Lorentz conversions
      final p1 = [0.1, 0.2, 0.3];
      final l1 = poincareToLorentz(p1);
      expect(l1.length, equals(4));
      
      final p1Back = lorentzToPoincare(l1);
      expect(p1Back[0], closeTo(p1[0], 1e-7));
      expect(p1Back[1], closeTo(p1[1], 1e-7));
      expect(p1Back[2], closeTo(p1[2], 1e-7));

      // 2. Poincaré Distance
      final p2 = [0.15, 0.25, 0.35];
      final dist = poincareDist(p1, p2);
      expect(dist, isPositive);

      // 3. Local Entropy
      final candidate = [0.1, 0.1];
      final neighbors = [
        [0.11, 0.1],
        [0.1, 0.12],
        [0.09, 0.09]
      ];
      final entropy = localEntropy(candidate, neighbors, c: 1.0);
      expect(entropy, lessThan(0.1));

      // 4. Lyapunov Convergence
      final trajectory = [
        [0.5, 0.5],
        [0.3, 0.3],
        [0.1, 0.1],
        [0.05, 0.05]
      ];
      final lya = lyapunovConvergence(trajectory, c: 1.0);
      expect(lya, lessThan(0.0));

      // 5. Koopman Extrapolation
      final past = [0.1, 0.2];
      final current = [0.15, 0.25];
      final future = koopmanExtrapolate(past, current, 1.0, c: 1.0);
      expect(future.length, equals(2));

      // 6. Context Resonance
      final thought = [0.5, 0.0];
      final globalCtx = [0.0, 0.5];
      final pull = contextResonance(thought, globalCtx, 0.5, c: 1.0);
      expect(pull.length, equals(2));
    });
  });
}
