import 'dart:async';
import 'dart:math';
import 'dart:typed_data';
import 'dart:convert';
import 'package:grpc/grpc.dart';
import 'generated/hyperspace.pbgrpc.dart' hide Filter;
import 'generated/hyperspace.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:crypto/crypto.dart';
import 'package:pointycastle/export.dart' as pc;
import 'math.dart' as hsMath;

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

class EncryptionContext {
  final List<int> aesKey;
  final List<int> hmacKey;
  final Map<String, List<List<double>>> projectionMatrices;
  EncryptionContext(this.aesKey, this.hmacKey) : projectionMatrices = {};
}

class HyperspaceClient {
  final ClientChannel _channel;
  late final DatabaseClient _stub;
  final String apiKey;
  final String? tenantId;

  final _collectionKeys = <String, String>{};
  final _encryptionContexts = <String, EncryptionContext>{};
  final _collectionMetrics = <String, String>{};
  final _collectionNoiseSigmas = <String, double>{};
  final _collectionSchemas = <String, pb.CollectionSchema>{};

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

  Future<bool> createCollection(String name, pb.CollectionSchema schema, {String encryptionKey = '', double noiseSigma = 0.02}) async {
    final metric = schema.components.isNotEmpty ? schema.components[0].metric : "l2";
    if (encryptionKey.isNotEmpty) {
      registerCollectionKey(name, encryptionKey, metric: metric, noiseSigma: noiseSigma, schema: schema);
    }
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
    final metric = _collectionMetrics[collection] ?? "l2";
    final context = await _getEncryptionContext(collection, vectorDim: vector.length, metric: metric);

    List<double> finalVector = vector;
    Map<String, String>? finalMetadata = metadata;
    Map<String, pb.MetadataValue>? finalTypedMetadata;
    List<int>? finalPayload = payload;

    if (context != null) {
      // 1. Noise injection
      final sigma = _collectionNoiseSigmas[collection] ?? 0.02;
      if (sigma > 0.0) {
        finalVector = hsMath.injectAnisotropicNoise(vector, context.hmacKey, sigma);
      } else {
        finalVector = List.from(vector);
      }

      // 2. Vector projection
      finalVector = _projectCollectionVector(collection, finalVector, context, metric);

      // 3. Payload Encryption
      if (payload != null && payload.isNotEmpty) {
        finalPayload = encryptPayload(payload, context.aesKey);
      }

      // 4. Metadata Hashing
      if (metadata != null) {
        finalMetadata = {};
        metadata.forEach((k, v) {
          final ek = hashMetadataKey(k, context.hmacKey);
          final ev = hashMetadataValue(v, context.hmacKey);
          finalMetadata![ek] = ev;
        });
      }

      if (typedMetadata != null) {
        finalTypedMetadata = {};
        typedMetadata.forEach((k, v) {
          final ek = hashMetadataKey(k, context.hmacKey);
          final String evStr = v is String ? v : v.toString();
          final ev = hashMetadataValue(evStr, context.hmacKey);
          finalTypedMetadata![ek] = pb.MetadataValue()..stringValue = ev;
        });
      }
    }

    final req = InsertRequest(
      id: id,
      vector: finalVector,
      collection: collection,
    );
    if (finalMetadata != null) req.metadata.addAll(finalMetadata);
    if (finalTypedMetadata != null) {
      req.typedMetadata.addAll(finalTypedMetadata);
    } else if (typedMetadata != null) {
      typedMetadata.forEach((k, v) => req.typedMetadata[k] = _toProtoMetadataValue(v));
    }
    if (finalPayload != null) req.payload = finalPayload;

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
    return resp.status.isNotEmpty;
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
    bool useWave = false,
    double? restartFactor,
  }) async {
    final metric = _collectionMetrics[collection] ?? "l2";
    final context = await _getEncryptionContext(collection, vectorDim: vector.length, metric: metric);

    List<double> finalVector = vector;
    Map<String, String>? finalFilter = filter;
    List<pb.Filter>? finalFilters;
    bool finalIncludePayload = includePayload;

    if (context != null) {
      // 1. Noise injection
      final sigma = _collectionNoiseSigmas[collection] ?? 0.02;
      if (sigma > 0.0) {
        finalVector = hsMath.injectAnisotropicNoise(vector, context.hmacKey, sigma);
      } else {
        finalVector = List.from(vector);
      }

      // 2. Vector projection
      finalVector = _projectCollectionVector(collection, finalVector, context, metric);

      // 3. Encrypt filters
      if (filter != null) {
        finalFilter = {};
        filter.forEach((k, v) {
          final ek = hashMetadataKey(k, context.hmacKey);
          final ev = hashMetadataValue(v, context.hmacKey);
          finalFilter![ek] = ev;
        });
      }

      if (filters != null) {
        finalFilters = _encryptFilters(filters.map((f) => f._proto).toList(), context);
      }

      // 4. Force payload inclusion so we can decrypt locally
      finalIncludePayload = true;
    }

    final req = SearchRequest(
      vector: finalVector,
      topK: topK,
      collection: collection,
    );
    if (finalIncludePayload) req.includePayload = finalIncludePayload;
    if (hybridQuery != null) req.hybridQuery = hybridQuery;
    if (hybridAlpha != null) req.hybridAlpha = hybridAlpha;
    if (bm25Options != null) req.bm25Options = bm25Options;
    if (mrlDimension != null) req.mrlDimension = mrlDimension;
    if (useWasserstein != null) req.useWasserstein = useWasserstein;
    if (finalFilter != null) req.filter.addAll(finalFilter);
    if (restartFactor != null) {
      req.filter['wave_restart_factor'] = restartFactor.toString();
    }
    if (finalFilters != null) {
      req.filters.addAll(finalFilters);
    } else if (filters != null) {
      req.filters.addAll(filters.map((f) => f._proto));
    }
    if (componentWeights != null) req.componentWeights.addAll(componentWeights);
    if (useWave) req.useWave = useWave;

    final resp = await _stub.search(req);

    if (context != null) {
      for (var r in resp.results) {
        if (r.payload.isNotEmpty) {
          try {
            final dec = decryptPayload(r.payload, context.aesKey);
            r.setField(5, dec);
          } catch (_) {}
        }
      }
    }

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
      return resp.status == 'ready' || resp.status == 'SERVING';
    } catch (e) {
      return false;
    }
  }

  Future<bool> createSnapshot() async {
    final resp = await _stub.triggerSnapshot(Empty());
    return resp.status.isNotEmpty;
  }

  Future<bool> vacuum() async {
    final resp = await _stub.triggerVacuum(Empty());
    return resp.status.isNotEmpty;
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

  Future<GetSubsumptionTreeResponse> getSubsumptionTree(String collection, int rootId, int maxDepth) {
    final req = GetSubsumptionTreeRequest(
      collection: collection,
      rootId: rootId,
      maxDepth: maxDepth,
    );
    return _stub.getSubsumptionTree(req);
  }

  // Admin & Sync API
  Future<bool> rebuildIndex(String name) async {
    final req = RebuildIndexRequest(name: name);
    final resp = await _stub.rebuildIndex(req);
    return resp.status.isNotEmpty;
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
    return resp.status.isNotEmpty;
  }

  Future<bool> freezeCollection(String name) async {
    final req = FreezeCollectionRequest(name: name);
    final resp = await _stub.freezeCollection(req);
    return resp.status.isNotEmpty;
  }

  Future<bool> unfreezeCollection(String name) async {
    final req = UnfreezeCollectionRequest(name: name);
    final resp = await _stub.unfreezeCollection(req);
    return resp.status.isNotEmpty;
  }

  Future<GetConceptParentsResponse> getConceptParents(String collection, int id, int layer, {int limit = 10}) {
    final req = GetConceptParentsRequest(collection: collection, id: id, layer: layer, limit: limit);
    return _stub.getConceptParents(req);
  }

  Stream<ReplicationLog> replicate(int lastLogicalClock) {
    final req = ReplicationRequest(lastLogicalClock: Int64(lastLogicalClock));
    return _stub.replicate(req);
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

  void registerCollectionKey(String collectionName, String key, {String metric = "l2", double noiseSigma = 0.02, pb.CollectionSchema? schema}) {
    _collectionKeys[collectionName] = key;
    _collectionMetrics[collectionName] = metric;
    _collectionNoiseSigmas[collectionName] = noiseSigma;
    if (schema != null) {
      _collectionSchemas[collectionName] = schema;
    }
    _encryptionContexts.remove(collectionName);
  }

  List<int> pbkdf2Derive(List<int> passwordBytes, List<int> salt, int iterations, int keyLength) {
    final kdf = pc.PBKDF2KeyDerivator(pc.HMac(pc.SHA256Digest(), 64));
    kdf.init(pc.Pbkdf2Parameters(Uint8List.fromList(salt), iterations, keyLength));
    return kdf.process(Uint8List.fromList(passwordBytes));
  }

  Map<String, List<int>> deriveKeys(String password, String collectionName) {
    final salt = sha256.convert(utf8.encode(collectionName)).bytes;
    final aesKey = pbkdf2Derive(utf8.encode(password), salt, 100000, 32);
    final hmacKey = pbkdf2Derive(utf8.encode(password), salt, 100000, 32);
    return {'aesKey': aesKey, 'hmacKey': hmacKey};
  }

  List<int> encryptPayload(List<int> plaintext, List<int> aesKey) {
    final pbkdf2Salt = Uint8List(16);
    final rng = Random.secure();
    for (int i = 0; i < 16; i++) pbkdf2Salt[i] = rng.nextInt(256);

    final derivedKey = pbkdf2Derive(aesKey, pbkdf2Salt, 100000, 32);

    final iv = Uint8List(12);
    for (int i = 0; i < 12; i++) iv[i] = rng.nextInt(256);

    final cipher = pc.GCMBlockCipher(pc.AESEngine());
    cipher.init(true, pc.AEADParameters(pc.KeyParameter(Uint8List.fromList(derivedKey)), 128, iv, Uint8List(0)));

    final ciphertextAndTag = cipher.process(Uint8List.fromList(plaintext));

    final builder = BytesBuilder();
    builder.add(pbkdf2Salt);
    builder.add(iv);
    builder.add(ciphertextAndTag);
    return builder.toBytes();
  }

  List<int> decryptPayload(List<int> data, List<int> aesKey) {
    if (data.length < 16 + 12 + 16) {
      throw ArgumentError("Invalid encrypted payload size");
    }
    final pbkdf2Salt = data.sublist(0, 16);
    final iv = data.sublist(16, 28);
    final ciphertextAndTag = data.sublist(28);

    final derivedKey = pbkdf2Derive(aesKey, pbkdf2Salt, 100000, 32);

    final cipher = pc.GCMBlockCipher(pc.AESEngine());
    cipher.init(false, pc.AEADParameters(pc.KeyParameter(Uint8List.fromList(derivedKey)), 128, Uint8List.fromList(iv), Uint8List(0)));

    return cipher.process(Uint8List.fromList(ciphertextAndTag));
  }

  String hashMetadataKey(String key, List<int> hmacKey) {
    final hmac = Hmac(sha256, hmacKey);
    final hash = hmac.convert(utf8.encode(key)).toString();
    return "tag_" + hash.substring(0, 16);
  }

  String hashMetadataValue(String value, List<int> hmacKey) {
    final hmac = Hmac(sha256, hmacKey);
    final hash = hmac.convert(utf8.encode(value)).toString();
    return "val_" + hash;
  }

  Future<EncryptionContext?> _getEncryptionContext(String collection, {int? vectorDim, String metric = "l2"}) async {
    if (collection.isEmpty) return null;
    final key = _collectionKeys[collection];
    if (key == null) return null;

    if (!_collectionSchemas.containsKey(collection)) {
      try {
        final stats = await getCollectionStats(collection);
        if (stats.hasSchema()) {
          _collectionSchemas[collection] = stats.schema;
        }
      } catch (_) {}
    }

    if (!_encryptionContexts.containsKey(collection)) {
      final keys = deriveKeys(key, collection);
      _encryptionContexts[collection] = EncryptionContext(keys['aesKey']!, keys['hmacKey']!);
    }

    final context = _encryptionContexts[collection]!;

    if (vectorDim != null) {
      final cacheKey = vectorDim.toString();
      if (!context.projectionMatrices.containsKey(cacheKey)) {
        final isLorentz = ["lorentz", "poincare"].contains(metric.toLowerCase());
        final matrixDim = metric.toLowerCase() == "poincare" ? vectorDim + 1 : vectorDim;
        if (isLorentz) {
          context.projectionMatrices[cacheKey] = hsMath.generateLorentzMatrix(matrixDim, context.hmacKey);
        } else {
          context.projectionMatrices[cacheKey] = hsMath.generateOrthogonalMatrix(matrixDim, context.hmacKey);
        }
      }
    }

    return context;
  }

  List<double> _projectSingleBlock(List<double> subVec, String metric, EncryptionContext context, {String? blockId}) {
    final dim = subVec.length;
    if (dim == 0) return [];

    final cacheKey = blockId != null ? "${dim}_$blockId" : "$dim";

    if (!context.projectionMatrices.containsKey(cacheKey)) {
      final isLorentz = ["lorentz", "poincare"].contains(metric.toLowerCase());
      final matrixDim = metric.toLowerCase() == "poincare" ? dim + 1 : dim;

      var seed = context.hmacKey;
      if (blockId != null) {
        seed = sha256.convert([...seed, ...utf8.encode(blockId)]).bytes;
      }

      if (isLorentz) {
        context.projectionMatrices[cacheKey] = hsMath.generateLorentzMatrix(matrixDim, seed);
      } else {
        context.projectionMatrices[cacheKey] = hsMath.generateOrthogonalMatrix(matrixDim, seed);
      }
    }

    final matrix = context.projectionMatrices[cacheKey]!;

    if (metric.toLowerCase() == "poincare") {
      final lorentzVec = hsMath.poincareToLorentz(subVec);
      final projLorentz = hsMath.projectVector(lorentzVec, matrix);
      return hsMath.lorentzToPoincare(projLorentz);
    } else {
      return hsMath.projectVector(subVec, matrix);
    }
  }

  List<double> _projectCollectionVector(String collection, List<double> vector, EncryptionContext context, String metric) {
    final schema = _collectionSchemas[collection];
    if (schema == null || schema.components.isEmpty) {
      return _projectSingleBlock(vector, metric, context);
    }

    final componentCutoffs = <String, List<int>>{};
    for (var layer in schema.cascadePipeline) {
      final compName = layer.componentName;
      final cutoff = layer.cutoffDimension;
      if (compName.isNotEmpty && cutoff > 0) {
        componentCutoffs.putIfAbsent(compName, () => []).add(cutoff);
      }
    }

    for (var compName in componentCutoffs.keys) {
      final cutoffs = componentCutoffs[compName]!.toSet().toList()..sort();
      componentCutoffs[compName] = cutoffs;
    }

    final projectedParts = <double>[];
    var currentOffset = 0;

    for (var comp in schema.components) {
      final compName = comp.name;
      final compMetric = comp.metric;
      final compDim = comp.fullDimension;

      if (currentOffset >= vector.length) {
        break;
      }

      var end = currentOffset + compDim;
      if (end > vector.length) end = vector.length;
      var subVec = vector.sublist(currentOffset, end);
      if (subVec.length < compDim) {
        subVec = [...subVec, ...List<double>.filled(compDim - subVec.length, 0.0)];
      }

      final cutoffs = componentCutoffs[compName] ?? [];
      final validCutoffs = cutoffs.where((c) => c < compDim).toList();

      List<double> projSub;
      if (validCutoffs.isEmpty) {
        projSub = _projectSingleBlock(subVec, compMetric, context);
      } else {
        projSub = [];
        var blockStart = 0;
        for (var cutoff in validCutoffs) {
          final blockData = subVec.sublist(blockStart, cutoff);
          final projBlock = _projectSingleBlock(blockData, compMetric, context, blockId: "${compName}_block_${blockStart}_$cutoff");
          projSub.addAll(projBlock);
          blockStart = cutoff;
        }
        if (blockStart < compDim) {
          final blockData = subVec.sublist(blockStart, compDim);
          final projBlock = _projectSingleBlock(blockData, compMetric, context, blockId: "${compName}_block_${blockStart}_$compDim");
          projSub.addAll(projBlock);
        }
      }

      projectedParts.addAll(projSub);
      currentOffset += compDim;
    }

    if (currentOffset < vector.length) {
      projectedParts.addAll(vector.sublist(currentOffset));
    }

    return projectedParts;
  }

  List<pb.Filter> _encryptFilters(List<pb.Filter> filters, EncryptionContext context) {
    return filters.map((f) {
      final nf = pb.Filter();
      if (f.hasMatch()) {
        nf.match = pb.Match()
          ..key = hashMetadataKey(f.match.key, context.hmacKey)
          ..value = hashMetadataValue(f.match.value, context.hmacKey);
      } else if (f.hasPrefix()) {
        nf.prefix = pb.Prefix()
          ..key = hashMetadataKey(f.prefix.key, context.hmacKey)
          ..prefix = hashMetadataValue(f.prefix.prefix, context.hmacKey);
      } else if (f.hasAndOp()) {
        nf.andOp = pb.FilterAnd()..conditions.addAll(_encryptFilters(f.andOp.conditions, context));
      } else if (f.hasOrOp()) {
        nf.orOp = pb.FilterOr()..conditions.addAll(_encryptFilters(f.orOp.conditions, context));
      } else if (f.hasNotOp()) {
        nf.notOp = pb.FilterNot()..condition = _encryptFilters([f.notOp.condition], context)[0];
      } else {
        nf.mergeFromMessage(f);
      }
      return nf;
    }).toList();
  }
}
