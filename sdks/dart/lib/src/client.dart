import 'dart:async';
import 'dart:math';
import 'package:grpc/grpc.dart';
import 'generated/hyperspace.pbgrpc.dart' hide Filter;
import 'generated/hyperspace.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';

/// Extension to support Sidecar Payload Storage (v3.2) without requiring immediate stub regeneration
extension SearchResultPayload on pb.SearchResult {
  List<int>? get payload {
    if (hasField(5)) {
      return getField(5) as List<int>;
    }
    return null;
  }
}

/// Helper for building complex filters in HyperspaceDB.
class Filter {
  final pb.Filter _proto;
  Filter(this._proto);

  static Filter match(String key, String value) {
    final f = pb.Filter()..match = (Match()..key = key..value = value);
    return Filter(f);
  }

  static Filter range(String key, {num? gte, num? lte}) {
    final r = Range()..key = key;
    if (gte != null) {
      if (gte is int) r.gte = Int64(gte); else r.gteF64 = gte.toDouble();
    }
    if (lte != null) {
      if (lte is int) r.lte = Int64(lte); else r.lteF64 = lte.toDouble();
    }
    return Filter(pb.Filter()..range = r);
  }

  static Filter inCone(List<double> axes, List<double> apertures, double cen) {
    final c = InCone()..axes.addAll(axes)..apertures.addAll(apertures)..cen = cen;
    return Filter(pb.Filter()..inCone = c);
  }

  static Filter inBall(List<double> center, double radius) {
    final b = InBall()..center.addAll(center)..radius = radius;
    return Filter(pb.Filter()..inBall = b);
  }

  static Filter inBox(List<double> minBounds, List<double> maxBounds) {
    final b = InBox()..minBounds.addAll(minBounds)..maxBounds.addAll(maxBounds);
    return Filter(pb.Filter()..inBox = b);
  }

  static Filter and(List<Filter> filters) {
    final a = FilterAnd()..conditions.addAll(filters.map((f) => f._proto));
    return Filter(pb.Filter()..andOp = a);
  }

  static Filter or(List<Filter> filters) {
    final o = FilterOr()..conditions.addAll(filters.map((f) => f._proto));
    return Filter(pb.Filter()..orOp = o);
  }

  static Filter not(Filter filter) {
    final n = FilterNot()..condition = filter._proto;
    return Filter(pb.Filter()..notOp = n);
  }
}

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

  MetadataValue _toProtoMetadataValue(dynamic value) {
    final mv = MetadataValue();
    if (value is String) {
      mv.stringValue = value;
    } else if (value is int) {
      mv.intValue = Int64(value);
    } else if (value is double) {
      mv.doubleValue = value;
    } else if (value is bool) {
      mv.boolValue = value;
    } else {
      mv.stringValue = value.toString();
    }
    return mv;
  }

  Future<void> close() async {
    await _channel.shutdown();
  }

  Future<bool> createCollection(String name, pb.CollectionSchema schema) async {
    final req = CreateCollectionRequest(name: name, schema: schema);
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


  Future<bool> insert(int id, List<double> vector, {
    String collection = '', 
    Map<String, String>? metadata,
    Map<String, dynamic>? typedMetadata,
    List<int>? payload,
  }) async {
    final req = InsertRequest(
      id: id,
      vector: vector,
      collection: collection,
    );
    if (metadata != null) req.metadata.addAll(metadata);
    if (typedMetadata != null) {
      typedMetadata.forEach((k, v) => req.typedMetadata[k] = _toProtoMetadataValue(v));
    }
    if (payload != null) req.payload = payload;
    final resp = await _stub.insert(req);
    return resp.success;
  }

  Future<bool> batchInsert(List<Map<String, dynamic>> points, {String collection = ''}) async {
    final req = BatchInsertRequest(collection: collection);
    for (var p in points) {
      final vd = VectorData(
        id: p['id'] as int,
        vector: p['vector'] as List<double>,
      );
      if (p['metadata'] != null) vd.metadata.addAll(Map<String, String>.from(p['metadata']));
      if (p['typedMetadata'] != null) {
        (p['typedMetadata'] as Map<String, dynamic>).forEach((k, v) => vd.typedMetadata[k] = _toProtoMetadataValue(v));
      }
      req.vectors.add(vd);
    }
    final resp = await _stub.batchInsert(req);
    return resp.success;
  }

  Future<List<VectorData>> getPoints(List<int> ids, {String collection = ''}) async {
    final req = GetPointsRequest(collection: collection, ids: ids);
    final resp = await _stub.getPoints(req);
    return resp.points;
  }

  Future<bool> updatePayload(int id, Map<String, dynamic> typedMetadata, {String collection = ''}) async {
    final req = UpdatePayloadRequest(collection: collection, id: id);
    typedMetadata.forEach((k, v) => req.typedMetadata[k] = _toProtoMetadataValue(v));
    final resp = await _stub.updatePayload(req);
    return resp.status == 'success';
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
    bool includePayload = false,
    Bm25Options? bm25Options,
    int? mrlDimension,
    bool? useWasserstein,
    Map<String, String>? filter,
    List<Filter>? filters,
    Map<String, double>? componentWeights,
  }) async {
    final req = SearchRequest(
      vector: vector,
      topK: topK,
      collection: collection,
    );
    if (includePayload) req.includePayload = includePayload;
    if (hybridQuery != null) req.hybridQuery = hybridQuery;
    if (hybridAlpha != null) req.hybridAlpha = hybridAlpha;
    if (bm25Options != null) req.bm25Options = bm25Options;
    if (mrlDimension != null) req.mrlDimension = mrlDimension;
    if (useWasserstein != null) req.useWasserstein = useWasserstein;
    if (filter != null) req.filter.addAll(filter);
    if (filters != null) req.filters.addAll(filters.map((f) => f._proto));
    if (componentWeights != null) req.componentWeights.addAll(componentWeights);

    final resp = await _stub.search(req);
    return resp.results;
  }

  Future<List<List<SearchResult>>> searchBatch(List<List<double>> vectors, int topK, {String collection = ''}) async {
    final req = BatchSearchRequest();
    for (var v in vectors) {
      req.searches.add(SearchRequest(vector: v, topK: topK, collection: collection));
    }
    final resp = await _stub.searchBatch(req);
    return resp.responses.map((r) => r.results).toList();
  }

  Future<List<SearchResult>> searchText(
    String text, 
    int topK, {
    String collection = '', 
    double? hybridAlpha,
    Bm25Options? bm25Options,
    Map<String, String>? filter,
    List<Filter>? filters,
    bool includePayload = false,
  }) async {
    final req = SearchTextRequest(
      text: text,
      topK: topK,
      collection: collection,
    );
    if (includePayload) req.includePayload = includePayload;
    if (hybridAlpha != null) req.hybridAlpha = hybridAlpha;
    if (bm25Options != null) req.bm25Options = bm25Options;
    if (filter != null) req.filter.addAll(filter);
    if (filters != null) req.filters.addAll(filters.map((f) => f._proto));

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

  Future<bool> updateCollection(String name, {int? efSearch, int? efConstruction, int? m}) async {
    final req = ConfigUpdate(collection: name);
    if (efSearch != null) req.efSearch = efSearch;
    if (efConstruction != null) req.efConstruction = efConstruction;
    if (m != null) req.m = m;
    final resp = await _stub.configure(req);
    return resp.status.contains('success') || resp.status.contains('updated');
  }

  Future<ScrollResponse> scroll(String collection, {int limit = 100, int offset = 0, List<Filter>? filters}) async {
    final req = ScrollRequest(collection: collection, limit: limit, offset: offset);
    if (filters != null) req.filters.addAll(filters.map((f) => f._proto));
    return _stub.scroll(req);
  }

  Future<int> count(String collection, {List<Filter>? filters}) async {
    final req = CountRequest(collection: collection);
    if (filters != null) req.filters.addAll(filters.map((f) => f._proto));
    final resp = await _stub.count(req);
    return resp.count.toInt();
  }

  Future<bool> healthCheck() async {
    try {
      final resp = await _stub.healthCheck(Empty());
      return resp.status == 'ready';
    } catch (e) {
      return false;
    }
  }

  Future<bool> createSnapshot() async {
    final resp = await _stub.triggerSnapshot(Empty());
    return resp.status == 'success';
  }

  Future<bool> vacuum() async {
    final resp = await _stub.triggerVacuum(Empty());
    return resp.status == 'success';
  }

  Stream<SystemStats> getMetrics() {
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
      types: types,
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
