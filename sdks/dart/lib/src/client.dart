import 'dart:async';
import 'dart:math';
import 'package:grpc/grpc.dart';
import 'generated/hyperspace.pbgrpc.dart';
import 'generated/hyperspace.pb.dart';

class HyperspaceClient {
  final ClientChannel _channel;
  late final DatabaseClient _stub;
  final String apiKey;
  final String? tenantId;

  HyperspaceClient(String address, int port, {this.apiKey = '', this.tenantId})
      : _channel = ClientChannel(
          address,
          port: port,
          options: const ChannelOptions(credentials: ChannelCredentials.insecure()),
        ) {
    _stub = DatabaseClient(_channel, options: _callOptions());
  }

  CallOptions _callOptions() {
    final metadata = <String, String>{
      if (apiKey.isNotEmpty) 'x-api-key': apiKey,
      if (tenantId != null) 'x-hyperspace-user-id': tenantId!,
    };
    return CallOptions(metadata: metadata);
  }

  Future<void> close() async {
    await _channel.shutdown();
  }

  Future<bool> createCollection(String name, int dimension, String metric) async {
    final req = CreateCollectionRequest(name: name, dimension: dimension, metric: metric);
    final resp = await _stub.createCollection(req);
    return resp.status.isNotEmpty;
  }

  Future<bool> deleteCollection(String name) async {
    final req = DeleteCollectionRequest(name: name);
    final resp = await _stub.deleteCollection(req);
    return resp.status.isNotEmpty;
  }

  Future<List<CollectionSummary>> listCollections() async {
    final req = Empty();
    final resp = await _stub.listCollections(req);
    return resp.collections;
  }


  Future<bool> insert(int id, List<double> vector, {String collection = ''}) async {
    final req = InsertRequest(
      id: id,
      vector: vector,
      collection: collection,
    );
    final resp = await _stub.insert(req);
    return resp.success;
  }

  Future<bool> delete(int id, {String collection = ''}) async {
    final req = DeleteRequest(
      id: id,
      collection: collection,
    );
    final resp = await _stub.delete(req);
    return resp.success;
  }

  Future<bool> insertText(int id, String text, {String collection = ''}) async {
    final req = InsertTextRequest(
      id: id,
      text: text,
      collection: collection,
    );
    final resp = await _stub.insertText(req);
    return resp.success;
  }

  Future<List<double>> vectorize(String text, String metric) async {
    final req = VectorizeRequest(text: text, metric: metric);
    final resp = await _stub.vectorize(req);
    return resp.vector;
  }

  Future<List<SearchResult>> search(
    List<double> vector, 
    int topK, {
    String collection = '', 
    String? hybridQuery, 
    double? hybridAlpha,
    Bm25Options? bm25Options,
    int? mrlDimension,
    bool? useWasserstein,
  }) async {
    final req = SearchRequest(
      vector: vector,
      topK: topK,
      collection: collection,
    );
    if (hybridQuery != null) req.hybridQuery = hybridQuery;
    if (hybridAlpha != null) req.hybridAlpha = hybridAlpha;
    if (bm25Options != null) req.bm25Options = bm25Options;
    if (mrlDimension != null) req.mrlDimension = mrlDimension;
    if (useWasserstein != null) req.useWasserstein = useWasserstein;

    final resp = await _stub.search(req);
    return resp.results;
  }

  Future<List<SearchResult>> searchText(
    String text, 
    int topK, {
    String collection = '', 
    double? hybridAlpha,
    Bm25Options? bm25Options,
  }) async {
    final req = SearchTextRequest(
      text: text,
      topK: topK,
      collection: collection,
    );
    if (hybridAlpha != null) req.hybridAlpha = hybridAlpha;
    if (bm25Options != null) req.bm25Options = bm25Options;

    final resp = await _stub.searchText(req);
    return resp.results;
  }

  // Delta Sync API
  Future<SyncHandshakeResponse> syncHandshake(String collection, List<Int64> clientBuckets, {Int64? clientLogicalClock, Int64? clientCount}) async {
    final req = SyncHandshakeRequest(
      collection: collection,
      clientBuckets: clientBuckets,
      clientLogicalClock: clientLogicalClock ?? Int64.ZERO,
      clientCount: clientCount ?? Int64.ZERO,
    );
    return _stub.syncHandshake(req);
  }

  Stream<SyncVectorData> syncPull(String collection, List<int> bucketIndices) {
    final req = SyncPullRequest(
      collection: collection,
      bucketIndices: bucketIndices,
    );
    return _stub.syncPull(req);
  }

  Future<SyncPushResponse> syncPush(Stream<SyncVectorData> vectors) {
    return _stub.syncPush(vectors);
  }

  Future<CollectionStatsResponse> getCollectionStats(String name) {
    final req = CollectionStatsRequest(name: name);
    return _stub.getCollectionStats(req);
  }

  Future<bool> exists(String name) async {
    try {
      await getCollectionStats(name);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<bool> updateCollection(String name) async {
    final req = ConfigUpdate(collection: name);
    final resp = await _stub.configure(req);
    return resp.status == 'success';
  }

  Future<bool> createSnapshot() async {
    final resp = await _stub.triggerSnapshot(Empty());
    return resp.status == 'success';
  }

  Future<bool> vacuum() async {
    final resp = await _stub.triggerVacuum(Empty());
    return resp.status == 'success';
  }

  Stream<MonitorResponse> getMetrics() {
    return _stub.monitor(MonitorRequest());
  }

  Future<SearchMultiCollectionResponse> searchMultiCollection(List<String> collections, List<double> query, {int topK = 10}) {
    final req = SearchMultiCollectionRequest(
      collections: collections,
      vector: query,
      topK: topK,
    );
    return _stub.searchMultiCollection(req);
  }

  // Graph API
  Future<GraphNode> getNode(String collection, int id, int layer) {
    final req = GetNodeRequest(collection: collection, id: id, layer: layer);
    return _stub.getNode(req);
  }

  Future<GetNeighborsResponse> getNeighbors(String collection, int id, int layer, {int limit = 10, int offset = 0}) {
    final req = GetNeighborsRequest(collection: collection, id: id, layer: layer, limit: limit, offset: offset);
    return _stub.getNeighbors(req);
  }

  Future<TraverseResponse> traverse(TraverseRequest req) {
    return _stub.traverse(req);
  }

  Future<FindSemanticClustersResponse> findSemanticClusters(FindSemanticClustersRequest req) {
    return _stub.findSemanticClusters(req);
  }

  // Admin & Sync API
  Future<bool> rebuildIndex(String name) async {
    final req = RebuildIndexRequest(name: name);
    final resp = await _stub.rebuildIndex(req);
    return resp.status == 'success';
  }

  Future<DigestResponse> getDigest(String collection) {
    final req = DigestRequest(collection: collection);
    return _stub.getDigest(req);
  }

  Stream<EventMessage> subscribeToEvents(List<EventType> types, {String? collection}) {
    final req = EventSubscriptionRequest(
      types: types.map((t) => t.value).toList(),
      collection: collection,
    );
    return _stub.subscribeToEvents(req);
  }

  Future<bool> triggerReconsolidation(String collection, List<double> target, double lr) async {
    final req = ReconsolidationRequest(
      collection: collection,
      targetVector: target,
      learningRate: lr,
    );
    final resp = await _stub.triggerReconsolidation(req);
    return resp.status == 'success';
  }


  /// Calculates Gromov Delta-hyperbolicity of a dataset.
  /// Used for geometric analytics.
  static Map<String, dynamic> analyzeDeltaHyperbolicity(List<List<double>> vectors, {int numSamples = 1000}) {
    if (vectors.length < 4) return {'delta': 0.0, 'recommendation': 'euclidean'};

    double l2Dist(List<double> a, List<double> b) {
      double sum = 0;
      for (int i = 0; i < a.length; i++) {
        sum += pow(a[i] - b[i], 2);
      }
      return sqrt(sum);
    }

    final rand = Random();
    double maxDelta = 0;

    for (int s = 0; s < numSamples; s++) {
      final idxs = <int>{};
      while (idxs.length < 4) idxs.add(rand.nextInt(vectors.length));
      final p = idxs.toList();
      
      final d_ij = l2Dist(vectors[p[0]], vectors[p[1]]);
      final d_kl = l2Dist(vectors[p[2]], vectors[p[3]]);
      final d_ik = l2Dist(vectors[p[0]], vectors[p[2]]);
      final d_jl = l2Dist(vectors[p[1]], vectors[p[3]]);
      final d_il = l2Dist(vectors[p[0]], vectors[p[3]]);
      final d_jk = l2Dist(vectors[p[1]], vectors[p[2]]);

      final s1 = d_ij + d_kl;
      final s2 = d_ik + d_jl;
      final s3 = d_il + d_jk;

      final sorted = [s1, s2, s3]..sort((a, b) => b.compareTo(a));
      final delta = (sorted[0] - sorted[1]) / 2.0;
      if (delta > maxDelta) maxDelta = delta;
    }

    final String recommendation = maxDelta < 0.15 ? 'lorentz' : (maxDelta < 0.30 ? 'poincare' : 'l2');
    return {'delta': maxDelta, 'recommendation': recommendation};
  }
}
