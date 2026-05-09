// This is a generated file - do not edit.
//
// Generated from hyperspace.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'hyperspace.pb.dart' as $0;

export 'hyperspace.pb.dart';

@$pb.GrpcServiceName('hyperspace.Database')
class DatabaseClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  DatabaseClient(super.channel, {super.options, super.interceptors});

  /// Collection Management
  $grpc.ResponseFuture<$0.StatusResponse> createCollection(
    $0.CreateCollectionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createCollection, request, options: options);
  }

  $grpc.ResponseFuture<$0.StatusResponse> deleteCollection(
    $0.DeleteCollectionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteCollection, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListCollectionsResponse> listCollections(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listCollections, request, options: options);
  }

  $grpc.ResponseFuture<$0.CollectionStatsResponse> getCollectionStats(
    $0.CollectionStatsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getCollectionStats, request, options: options);
  }

  /// Insert vectors
  $grpc.ResponseFuture<$0.InsertResponse> insert(
    $0.InsertRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$insert, request, options: options);
  }

  $grpc.ResponseFuture<$0.InsertResponse> batchInsert(
    $0.BatchInsertRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$batchInsert, request, options: options);
  }

  $grpc.ResponseFuture<$0.InsertResponse> insertText(
    $0.InsertTextRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$insertText, request, options: options);
  }

  $grpc.ResponseFuture<$0.VectorizeResponse> vectorize(
    $0.VectorizeRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$vectorize, request, options: options);
  }

  $grpc.ResponseFuture<$0.SearchResponse> searchText(
    $0.SearchTextRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$searchText, request, options: options);
  }

  /// Delete vectors
  $grpc.ResponseFuture<$0.DeleteResponse> delete(
    $0.DeleteRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$delete, request, options: options);
  }

  /// Extended Data Ops
  $grpc.ResponseFuture<$0.GetPointsResponse> getPoints(
    $0.GetPointsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getPoints, request, options: options);
  }

  $grpc.ResponseFuture<$0.StatusResponse> updatePayload(
    $0.UpdatePayloadRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updatePayload, request, options: options);
  }

  $grpc.ResponseFuture<$0.ScrollResponse> scroll(
    $0.ScrollRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$scroll, request, options: options);
  }

  $grpc.ResponseFuture<$0.CountResponse> count(
    $0.CountRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$count, request, options: options);
  }

  /// Search (ANN)
  $grpc.ResponseFuture<$0.SearchResponse> search(
    $0.SearchRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$search, request, options: options);
  }

  /// Batch Search (ANN)
  $grpc.ResponseFuture<$0.BatchSearchResponse> searchBatch(
    $0.BatchSearchRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$searchBatch, request, options: options);
  }

  /// Multi-Geometry Search (v3.0)
  $grpc.ResponseFuture<$0.SearchMultiCollectionResponse> searchMultiCollection(
    $0.SearchMultiCollectionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$searchMultiCollection, request, options: options);
  }

  /// Graph Traversal API (v2.3)
  $grpc.ResponseFuture<$0.GraphNode> getNode(
    $0.GetNodeRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getNode, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetNeighborsResponse> getNeighbors(
    $0.GetNeighborsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getNeighbors, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetConceptParentsResponse> getConceptParents(
    $0.GetConceptParentsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getConceptParents, request, options: options);
  }

  $grpc.ResponseFuture<$0.TraverseResponse> traverse(
    $0.TraverseRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$traverse, request, options: options);
  }

  $grpc.ResponseFuture<$0.FindSemanticClustersResponse> findSemanticClusters(
    $0.FindSemanticClustersRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$findSemanticClusters, request, options: options);
  }

  /// Stream statistics for TUI (Global or Collection tailored)
  $grpc.ResponseStream<$0.SystemStats> monitor(
    $0.MonitorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$monitor, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// Admin Controls
  $grpc.ResponseFuture<$0.StatusResponse> triggerSnapshot(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$triggerSnapshot, request, options: options);
  }

  $grpc.ResponseFuture<$0.StatusResponse> triggerVacuum(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$triggerVacuum, request, options: options);
  }

  $grpc.ResponseFuture<$0.StatusResponse> triggerReconsolidation(
    $0.ReconsolidationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$triggerReconsolidation, request,
        options: options);
  }

  /// Dynamic Configuration
  $grpc.ResponseFuture<$0.StatusResponse> configure(
    $0.ConfigUpdate request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$configure, request, options: options);
  }

  /// Replication (Leader -> Follower)
  $grpc.ResponseStream<$0.ReplicationLog> replicate(
    $0.ReplicationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$replicate, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// CDC/Event Stream (External subscribers)
  $grpc.ResponseStream<$0.EventMessage> subscribeToEvents(
    $0.EventSubscriptionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$subscribeToEvents, $async.Stream.fromIterable([request]),
        options: options);
  }

  $grpc.ResponseFuture<$0.DigestResponse> getDigest(
    $0.DigestRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getDigest, request, options: options);
  }

  $grpc.ResponseFuture<$0.StatusResponse> rebuildIndex(
    $0.RebuildIndexRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$rebuildIndex, request, options: options);
  }

  /// Delta Sync (Merkle Tree — Task 2.1)
  /// Step 1: Client sends its digest, server returns which buckets differ.
  $grpc.ResponseFuture<$0.SyncHandshakeResponse> syncHandshake(
    $0.SyncHandshakeRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$syncHandshake, request, options: options);
  }

  /// Step 2: Client requests vectors from specific buckets, server streams them.
  $grpc.ResponseStream<$0.SyncVectorData> syncPull(
    $0.SyncPullRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$syncPull, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// Step 3 (optional): Client pushes its unique vectors to the server.
  $grpc.ResponseFuture<$0.SyncPushResponse> syncPush(
    $async.Stream<$0.SyncVectorData> request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$syncPush, request, options: options).single;
  }

  /// Health Status
  $grpc.ResponseFuture<$0.HealthCheckResponse> healthCheck(
    $0.Empty request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$healthCheck, request, options: options);
  }

  // method descriptors

  static final _$createCollection =
      $grpc.ClientMethod<$0.CreateCollectionRequest, $0.StatusResponse>(
          '/hyperspace.Database/CreateCollection',
          ($0.CreateCollectionRequest value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$deleteCollection =
      $grpc.ClientMethod<$0.DeleteCollectionRequest, $0.StatusResponse>(
          '/hyperspace.Database/DeleteCollection',
          ($0.DeleteCollectionRequest value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$listCollections =
      $grpc.ClientMethod<$0.Empty, $0.ListCollectionsResponse>(
          '/hyperspace.Database/ListCollections',
          ($0.Empty value) => value.writeToBuffer(),
          $0.ListCollectionsResponse.fromBuffer);
  static final _$getCollectionStats =
      $grpc.ClientMethod<$0.CollectionStatsRequest, $0.CollectionStatsResponse>(
          '/hyperspace.Database/GetCollectionStats',
          ($0.CollectionStatsRequest value) => value.writeToBuffer(),
          $0.CollectionStatsResponse.fromBuffer);
  static final _$insert =
      $grpc.ClientMethod<$0.InsertRequest, $0.InsertResponse>(
          '/hyperspace.Database/Insert',
          ($0.InsertRequest value) => value.writeToBuffer(),
          $0.InsertResponse.fromBuffer);
  static final _$batchInsert =
      $grpc.ClientMethod<$0.BatchInsertRequest, $0.InsertResponse>(
          '/hyperspace.Database/BatchInsert',
          ($0.BatchInsertRequest value) => value.writeToBuffer(),
          $0.InsertResponse.fromBuffer);
  static final _$insertText =
      $grpc.ClientMethod<$0.InsertTextRequest, $0.InsertResponse>(
          '/hyperspace.Database/InsertText',
          ($0.InsertTextRequest value) => value.writeToBuffer(),
          $0.InsertResponse.fromBuffer);
  static final _$vectorize =
      $grpc.ClientMethod<$0.VectorizeRequest, $0.VectorizeResponse>(
          '/hyperspace.Database/Vectorize',
          ($0.VectorizeRequest value) => value.writeToBuffer(),
          $0.VectorizeResponse.fromBuffer);
  static final _$searchText =
      $grpc.ClientMethod<$0.SearchTextRequest, $0.SearchResponse>(
          '/hyperspace.Database/SearchText',
          ($0.SearchTextRequest value) => value.writeToBuffer(),
          $0.SearchResponse.fromBuffer);
  static final _$delete =
      $grpc.ClientMethod<$0.DeleteRequest, $0.DeleteResponse>(
          '/hyperspace.Database/Delete',
          ($0.DeleteRequest value) => value.writeToBuffer(),
          $0.DeleteResponse.fromBuffer);
  static final _$getPoints =
      $grpc.ClientMethod<$0.GetPointsRequest, $0.GetPointsResponse>(
          '/hyperspace.Database/GetPoints',
          ($0.GetPointsRequest value) => value.writeToBuffer(),
          $0.GetPointsResponse.fromBuffer);
  static final _$updatePayload =
      $grpc.ClientMethod<$0.UpdatePayloadRequest, $0.StatusResponse>(
          '/hyperspace.Database/UpdatePayload',
          ($0.UpdatePayloadRequest value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$scroll =
      $grpc.ClientMethod<$0.ScrollRequest, $0.ScrollResponse>(
          '/hyperspace.Database/Scroll',
          ($0.ScrollRequest value) => value.writeToBuffer(),
          $0.ScrollResponse.fromBuffer);
  static final _$count = $grpc.ClientMethod<$0.CountRequest, $0.CountResponse>(
      '/hyperspace.Database/Count',
      ($0.CountRequest value) => value.writeToBuffer(),
      $0.CountResponse.fromBuffer);
  static final _$search =
      $grpc.ClientMethod<$0.SearchRequest, $0.SearchResponse>(
          '/hyperspace.Database/Search',
          ($0.SearchRequest value) => value.writeToBuffer(),
          $0.SearchResponse.fromBuffer);
  static final _$searchBatch =
      $grpc.ClientMethod<$0.BatchSearchRequest, $0.BatchSearchResponse>(
          '/hyperspace.Database/SearchBatch',
          ($0.BatchSearchRequest value) => value.writeToBuffer(),
          $0.BatchSearchResponse.fromBuffer);
  static final _$searchMultiCollection = $grpc.ClientMethod<
          $0.SearchMultiCollectionRequest, $0.SearchMultiCollectionResponse>(
      '/hyperspace.Database/SearchMultiCollection',
      ($0.SearchMultiCollectionRequest value) => value.writeToBuffer(),
      $0.SearchMultiCollectionResponse.fromBuffer);
  static final _$getNode = $grpc.ClientMethod<$0.GetNodeRequest, $0.GraphNode>(
      '/hyperspace.Database/GetNode',
      ($0.GetNodeRequest value) => value.writeToBuffer(),
      $0.GraphNode.fromBuffer);
  static final _$getNeighbors =
      $grpc.ClientMethod<$0.GetNeighborsRequest, $0.GetNeighborsResponse>(
          '/hyperspace.Database/GetNeighbors',
          ($0.GetNeighborsRequest value) => value.writeToBuffer(),
          $0.GetNeighborsResponse.fromBuffer);
  static final _$getConceptParents = $grpc.ClientMethod<
          $0.GetConceptParentsRequest, $0.GetConceptParentsResponse>(
      '/hyperspace.Database/GetConceptParents',
      ($0.GetConceptParentsRequest value) => value.writeToBuffer(),
      $0.GetConceptParentsResponse.fromBuffer);
  static final _$traverse =
      $grpc.ClientMethod<$0.TraverseRequest, $0.TraverseResponse>(
          '/hyperspace.Database/Traverse',
          ($0.TraverseRequest value) => value.writeToBuffer(),
          $0.TraverseResponse.fromBuffer);
  static final _$findSemanticClusters = $grpc.ClientMethod<
          $0.FindSemanticClustersRequest, $0.FindSemanticClustersResponse>(
      '/hyperspace.Database/FindSemanticClusters',
      ($0.FindSemanticClustersRequest value) => value.writeToBuffer(),
      $0.FindSemanticClustersResponse.fromBuffer);
  static final _$monitor =
      $grpc.ClientMethod<$0.MonitorRequest, $0.SystemStats>(
          '/hyperspace.Database/Monitor',
          ($0.MonitorRequest value) => value.writeToBuffer(),
          $0.SystemStats.fromBuffer);
  static final _$triggerSnapshot =
      $grpc.ClientMethod<$0.Empty, $0.StatusResponse>(
          '/hyperspace.Database/TriggerSnapshot',
          ($0.Empty value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$triggerVacuum =
      $grpc.ClientMethod<$0.Empty, $0.StatusResponse>(
          '/hyperspace.Database/TriggerVacuum',
          ($0.Empty value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$triggerReconsolidation =
      $grpc.ClientMethod<$0.ReconsolidationRequest, $0.StatusResponse>(
          '/hyperspace.Database/TriggerReconsolidation',
          ($0.ReconsolidationRequest value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$configure =
      $grpc.ClientMethod<$0.ConfigUpdate, $0.StatusResponse>(
          '/hyperspace.Database/Configure',
          ($0.ConfigUpdate value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$replicate =
      $grpc.ClientMethod<$0.ReplicationRequest, $0.ReplicationLog>(
          '/hyperspace.Database/Replicate',
          ($0.ReplicationRequest value) => value.writeToBuffer(),
          $0.ReplicationLog.fromBuffer);
  static final _$subscribeToEvents =
      $grpc.ClientMethod<$0.EventSubscriptionRequest, $0.EventMessage>(
          '/hyperspace.Database/SubscribeToEvents',
          ($0.EventSubscriptionRequest value) => value.writeToBuffer(),
          $0.EventMessage.fromBuffer);
  static final _$getDigest =
      $grpc.ClientMethod<$0.DigestRequest, $0.DigestResponse>(
          '/hyperspace.Database/GetDigest',
          ($0.DigestRequest value) => value.writeToBuffer(),
          $0.DigestResponse.fromBuffer);
  static final _$rebuildIndex =
      $grpc.ClientMethod<$0.RebuildIndexRequest, $0.StatusResponse>(
          '/hyperspace.Database/RebuildIndex',
          ($0.RebuildIndexRequest value) => value.writeToBuffer(),
          $0.StatusResponse.fromBuffer);
  static final _$syncHandshake =
      $grpc.ClientMethod<$0.SyncHandshakeRequest, $0.SyncHandshakeResponse>(
          '/hyperspace.Database/SyncHandshake',
          ($0.SyncHandshakeRequest value) => value.writeToBuffer(),
          $0.SyncHandshakeResponse.fromBuffer);
  static final _$syncPull =
      $grpc.ClientMethod<$0.SyncPullRequest, $0.SyncVectorData>(
          '/hyperspace.Database/SyncPull',
          ($0.SyncPullRequest value) => value.writeToBuffer(),
          $0.SyncVectorData.fromBuffer);
  static final _$syncPush =
      $grpc.ClientMethod<$0.SyncVectorData, $0.SyncPushResponse>(
          '/hyperspace.Database/SyncPush',
          ($0.SyncVectorData value) => value.writeToBuffer(),
          $0.SyncPushResponse.fromBuffer);
  static final _$healthCheck =
      $grpc.ClientMethod<$0.Empty, $0.HealthCheckResponse>(
          '/hyperspace.Database/HealthCheck',
          ($0.Empty value) => value.writeToBuffer(),
          $0.HealthCheckResponse.fromBuffer);
}

@$pb.GrpcServiceName('hyperspace.Database')
abstract class DatabaseServiceBase extends $grpc.Service {
  $core.String get $name => 'hyperspace.Database';

  DatabaseServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.CreateCollectionRequest, $0.StatusResponse>(
            'CreateCollection',
            createCollection_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateCollectionRequest.fromBuffer(value),
            ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteCollectionRequest, $0.StatusResponse>(
            'DeleteCollection',
            deleteCollection_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteCollectionRequest.fromBuffer(value),
            ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.ListCollectionsResponse>(
        'ListCollections',
        listCollections_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.ListCollectionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CollectionStatsRequest,
            $0.CollectionStatsResponse>(
        'GetCollectionStats',
        getCollectionStats_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CollectionStatsRequest.fromBuffer(value),
        ($0.CollectionStatsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.InsertRequest, $0.InsertResponse>(
        'Insert',
        insert_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.InsertRequest.fromBuffer(value),
        ($0.InsertResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.BatchInsertRequest, $0.InsertResponse>(
        'BatchInsert',
        batchInsert_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.BatchInsertRequest.fromBuffer(value),
        ($0.InsertResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.InsertTextRequest, $0.InsertResponse>(
        'InsertText',
        insertText_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.InsertTextRequest.fromBuffer(value),
        ($0.InsertResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.VectorizeRequest, $0.VectorizeResponse>(
        'Vectorize',
        vectorize_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.VectorizeRequest.fromBuffer(value),
        ($0.VectorizeResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SearchTextRequest, $0.SearchResponse>(
        'SearchText',
        searchText_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SearchTextRequest.fromBuffer(value),
        ($0.SearchResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteRequest, $0.DeleteResponse>(
        'Delete',
        delete_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteRequest.fromBuffer(value),
        ($0.DeleteResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetPointsRequest, $0.GetPointsResponse>(
        'GetPoints',
        getPoints_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetPointsRequest.fromBuffer(value),
        ($0.GetPointsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdatePayloadRequest, $0.StatusResponse>(
        'UpdatePayload',
        updatePayload_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdatePayloadRequest.fromBuffer(value),
        ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ScrollRequest, $0.ScrollResponse>(
        'Scroll',
        scroll_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ScrollRequest.fromBuffer(value),
        ($0.ScrollResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CountRequest, $0.CountResponse>(
        'Count',
        count_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CountRequest.fromBuffer(value),
        ($0.CountResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SearchRequest, $0.SearchResponse>(
        'Search',
        search_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SearchRequest.fromBuffer(value),
        ($0.SearchResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.BatchSearchRequest, $0.BatchSearchResponse>(
            'SearchBatch',
            searchBatch_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.BatchSearchRequest.fromBuffer(value),
            ($0.BatchSearchResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SearchMultiCollectionRequest,
            $0.SearchMultiCollectionResponse>(
        'SearchMultiCollection',
        searchMultiCollection_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SearchMultiCollectionRequest.fromBuffer(value),
        ($0.SearchMultiCollectionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetNodeRequest, $0.GraphNode>(
        'GetNode',
        getNode_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetNodeRequest.fromBuffer(value),
        ($0.GraphNode value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetNeighborsRequest, $0.GetNeighborsResponse>(
            'GetNeighbors',
            getNeighbors_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetNeighborsRequest.fromBuffer(value),
            ($0.GetNeighborsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetConceptParentsRequest,
            $0.GetConceptParentsResponse>(
        'GetConceptParents',
        getConceptParents_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetConceptParentsRequest.fromBuffer(value),
        ($0.GetConceptParentsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.TraverseRequest, $0.TraverseResponse>(
        'Traverse',
        traverse_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.TraverseRequest.fromBuffer(value),
        ($0.TraverseResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.FindSemanticClustersRequest,
            $0.FindSemanticClustersResponse>(
        'FindSemanticClusters',
        findSemanticClusters_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.FindSemanticClustersRequest.fromBuffer(value),
        ($0.FindSemanticClustersResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.MonitorRequest, $0.SystemStats>(
        'Monitor',
        monitor_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.MonitorRequest.fromBuffer(value),
        ($0.SystemStats value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.StatusResponse>(
        'TriggerSnapshot',
        triggerSnapshot_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.StatusResponse>(
        'TriggerVacuum',
        triggerVacuum_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ReconsolidationRequest, $0.StatusResponse>(
            'TriggerReconsolidation',
            triggerReconsolidation_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ReconsolidationRequest.fromBuffer(value),
            ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ConfigUpdate, $0.StatusResponse>(
        'Configure',
        configure_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ConfigUpdate.fromBuffer(value),
        ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ReplicationRequest, $0.ReplicationLog>(
        'Replicate',
        replicate_Pre,
        false,
        true,
        ($core.List<$core.int> value) =>
            $0.ReplicationRequest.fromBuffer(value),
        ($0.ReplicationLog value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.EventSubscriptionRequest, $0.EventMessage>(
            'SubscribeToEvents',
            subscribeToEvents_Pre,
            false,
            true,
            ($core.List<$core.int> value) =>
                $0.EventSubscriptionRequest.fromBuffer(value),
            ($0.EventMessage value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DigestRequest, $0.DigestResponse>(
        'GetDigest',
        getDigest_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DigestRequest.fromBuffer(value),
        ($0.DigestResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RebuildIndexRequest, $0.StatusResponse>(
        'RebuildIndex',
        rebuildIndex_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RebuildIndexRequest.fromBuffer(value),
        ($0.StatusResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.SyncHandshakeRequest, $0.SyncHandshakeResponse>(
            'SyncHandshake',
            syncHandshake_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.SyncHandshakeRequest.fromBuffer(value),
            ($0.SyncHandshakeResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SyncPullRequest, $0.SyncVectorData>(
        'SyncPull',
        syncPull_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.SyncPullRequest.fromBuffer(value),
        ($0.SyncVectorData value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SyncVectorData, $0.SyncPushResponse>(
        'SyncPush',
        syncPush,
        true,
        false,
        ($core.List<$core.int> value) => $0.SyncVectorData.fromBuffer(value),
        ($0.SyncPushResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.HealthCheckResponse>(
        'HealthCheck',
        healthCheck_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.HealthCheckResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.StatusResponse> createCollection_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateCollectionRequest> $request) async {
    return createCollection($call, await $request);
  }

  $async.Future<$0.StatusResponse> createCollection(
      $grpc.ServiceCall call, $0.CreateCollectionRequest request);

  $async.Future<$0.StatusResponse> deleteCollection_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteCollectionRequest> $request) async {
    return deleteCollection($call, await $request);
  }

  $async.Future<$0.StatusResponse> deleteCollection(
      $grpc.ServiceCall call, $0.DeleteCollectionRequest request);

  $async.Future<$0.ListCollectionsResponse> listCollections_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return listCollections($call, await $request);
  }

  $async.Future<$0.ListCollectionsResponse> listCollections(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.CollectionStatsResponse> getCollectionStats_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CollectionStatsRequest> $request) async {
    return getCollectionStats($call, await $request);
  }

  $async.Future<$0.CollectionStatsResponse> getCollectionStats(
      $grpc.ServiceCall call, $0.CollectionStatsRequest request);

  $async.Future<$0.InsertResponse> insert_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.InsertRequest> $request) async {
    return insert($call, await $request);
  }

  $async.Future<$0.InsertResponse> insert(
      $grpc.ServiceCall call, $0.InsertRequest request);

  $async.Future<$0.InsertResponse> batchInsert_Pre($grpc.ServiceCall $call,
      $async.Future<$0.BatchInsertRequest> $request) async {
    return batchInsert($call, await $request);
  }

  $async.Future<$0.InsertResponse> batchInsert(
      $grpc.ServiceCall call, $0.BatchInsertRequest request);

  $async.Future<$0.InsertResponse> insertText_Pre($grpc.ServiceCall $call,
      $async.Future<$0.InsertTextRequest> $request) async {
    return insertText($call, await $request);
  }

  $async.Future<$0.InsertResponse> insertText(
      $grpc.ServiceCall call, $0.InsertTextRequest request);

  $async.Future<$0.VectorizeResponse> vectorize_Pre($grpc.ServiceCall $call,
      $async.Future<$0.VectorizeRequest> $request) async {
    return vectorize($call, await $request);
  }

  $async.Future<$0.VectorizeResponse> vectorize(
      $grpc.ServiceCall call, $0.VectorizeRequest request);

  $async.Future<$0.SearchResponse> searchText_Pre($grpc.ServiceCall $call,
      $async.Future<$0.SearchTextRequest> $request) async {
    return searchText($call, await $request);
  }

  $async.Future<$0.SearchResponse> searchText(
      $grpc.ServiceCall call, $0.SearchTextRequest request);

  $async.Future<$0.DeleteResponse> delete_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.DeleteRequest> $request) async {
    return delete($call, await $request);
  }

  $async.Future<$0.DeleteResponse> delete(
      $grpc.ServiceCall call, $0.DeleteRequest request);

  $async.Future<$0.GetPointsResponse> getPoints_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetPointsRequest> $request) async {
    return getPoints($call, await $request);
  }

  $async.Future<$0.GetPointsResponse> getPoints(
      $grpc.ServiceCall call, $0.GetPointsRequest request);

  $async.Future<$0.StatusResponse> updatePayload_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdatePayloadRequest> $request) async {
    return updatePayload($call, await $request);
  }

  $async.Future<$0.StatusResponse> updatePayload(
      $grpc.ServiceCall call, $0.UpdatePayloadRequest request);

  $async.Future<$0.ScrollResponse> scroll_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.ScrollRequest> $request) async {
    return scroll($call, await $request);
  }

  $async.Future<$0.ScrollResponse> scroll(
      $grpc.ServiceCall call, $0.ScrollRequest request);

  $async.Future<$0.CountResponse> count_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.CountRequest> $request) async {
    return count($call, await $request);
  }

  $async.Future<$0.CountResponse> count(
      $grpc.ServiceCall call, $0.CountRequest request);

  $async.Future<$0.SearchResponse> search_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.SearchRequest> $request) async {
    return search($call, await $request);
  }

  $async.Future<$0.SearchResponse> search(
      $grpc.ServiceCall call, $0.SearchRequest request);

  $async.Future<$0.BatchSearchResponse> searchBatch_Pre($grpc.ServiceCall $call,
      $async.Future<$0.BatchSearchRequest> $request) async {
    return searchBatch($call, await $request);
  }

  $async.Future<$0.BatchSearchResponse> searchBatch(
      $grpc.ServiceCall call, $0.BatchSearchRequest request);

  $async.Future<$0.SearchMultiCollectionResponse> searchMultiCollection_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SearchMultiCollectionRequest> $request) async {
    return searchMultiCollection($call, await $request);
  }

  $async.Future<$0.SearchMultiCollectionResponse> searchMultiCollection(
      $grpc.ServiceCall call, $0.SearchMultiCollectionRequest request);

  $async.Future<$0.GraphNode> getNode_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetNodeRequest> $request) async {
    return getNode($call, await $request);
  }

  $async.Future<$0.GraphNode> getNode(
      $grpc.ServiceCall call, $0.GetNodeRequest request);

  $async.Future<$0.GetNeighborsResponse> getNeighbors_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetNeighborsRequest> $request) async {
    return getNeighbors($call, await $request);
  }

  $async.Future<$0.GetNeighborsResponse> getNeighbors(
      $grpc.ServiceCall call, $0.GetNeighborsRequest request);

  $async.Future<$0.GetConceptParentsResponse> getConceptParents_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetConceptParentsRequest> $request) async {
    return getConceptParents($call, await $request);
  }

  $async.Future<$0.GetConceptParentsResponse> getConceptParents(
      $grpc.ServiceCall call, $0.GetConceptParentsRequest request);

  $async.Future<$0.TraverseResponse> traverse_Pre($grpc.ServiceCall $call,
      $async.Future<$0.TraverseRequest> $request) async {
    return traverse($call, await $request);
  }

  $async.Future<$0.TraverseResponse> traverse(
      $grpc.ServiceCall call, $0.TraverseRequest request);

  $async.Future<$0.FindSemanticClustersResponse> findSemanticClusters_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.FindSemanticClustersRequest> $request) async {
    return findSemanticClusters($call, await $request);
  }

  $async.Future<$0.FindSemanticClustersResponse> findSemanticClusters(
      $grpc.ServiceCall call, $0.FindSemanticClustersRequest request);

  $async.Stream<$0.SystemStats> monitor_Pre($grpc.ServiceCall $call,
      $async.Future<$0.MonitorRequest> $request) async* {
    yield* monitor($call, await $request);
  }

  $async.Stream<$0.SystemStats> monitor(
      $grpc.ServiceCall call, $0.MonitorRequest request);

  $async.Future<$0.StatusResponse> triggerSnapshot_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return triggerSnapshot($call, await $request);
  }

  $async.Future<$0.StatusResponse> triggerSnapshot(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.StatusResponse> triggerVacuum_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return triggerVacuum($call, await $request);
  }

  $async.Future<$0.StatusResponse> triggerVacuum(
      $grpc.ServiceCall call, $0.Empty request);

  $async.Future<$0.StatusResponse> triggerReconsolidation_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ReconsolidationRequest> $request) async {
    return triggerReconsolidation($call, await $request);
  }

  $async.Future<$0.StatusResponse> triggerReconsolidation(
      $grpc.ServiceCall call, $0.ReconsolidationRequest request);

  $async.Future<$0.StatusResponse> configure_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.ConfigUpdate> $request) async {
    return configure($call, await $request);
  }

  $async.Future<$0.StatusResponse> configure(
      $grpc.ServiceCall call, $0.ConfigUpdate request);

  $async.Stream<$0.ReplicationLog> replicate_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ReplicationRequest> $request) async* {
    yield* replicate($call, await $request);
  }

  $async.Stream<$0.ReplicationLog> replicate(
      $grpc.ServiceCall call, $0.ReplicationRequest request);

  $async.Stream<$0.EventMessage> subscribeToEvents_Pre($grpc.ServiceCall $call,
      $async.Future<$0.EventSubscriptionRequest> $request) async* {
    yield* subscribeToEvents($call, await $request);
  }

  $async.Stream<$0.EventMessage> subscribeToEvents(
      $grpc.ServiceCall call, $0.EventSubscriptionRequest request);

  $async.Future<$0.DigestResponse> getDigest_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.DigestRequest> $request) async {
    return getDigest($call, await $request);
  }

  $async.Future<$0.DigestResponse> getDigest(
      $grpc.ServiceCall call, $0.DigestRequest request);

  $async.Future<$0.StatusResponse> rebuildIndex_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RebuildIndexRequest> $request) async {
    return rebuildIndex($call, await $request);
  }

  $async.Future<$0.StatusResponse> rebuildIndex(
      $grpc.ServiceCall call, $0.RebuildIndexRequest request);

  $async.Future<$0.SyncHandshakeResponse> syncHandshake_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SyncHandshakeRequest> $request) async {
    return syncHandshake($call, await $request);
  }

  $async.Future<$0.SyncHandshakeResponse> syncHandshake(
      $grpc.ServiceCall call, $0.SyncHandshakeRequest request);

  $async.Stream<$0.SyncVectorData> syncPull_Pre($grpc.ServiceCall $call,
      $async.Future<$0.SyncPullRequest> $request) async* {
    yield* syncPull($call, await $request);
  }

  $async.Stream<$0.SyncVectorData> syncPull(
      $grpc.ServiceCall call, $0.SyncPullRequest request);

  $async.Future<$0.SyncPushResponse> syncPush(
      $grpc.ServiceCall call, $async.Stream<$0.SyncVectorData> request);

  $async.Future<$0.HealthCheckResponse> healthCheck_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.Empty> $request) async {
    return healthCheck($call, await $request);
  }

  $async.Future<$0.HealthCheckResponse> healthCheck(
      $grpc.ServiceCall call, $0.Empty request);
}
