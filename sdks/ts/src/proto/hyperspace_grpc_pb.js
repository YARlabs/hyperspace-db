// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('@grpc/grpc-js');
var hyperspace_pb = require('./hyperspace_pb.js');

function serialize_hyperspace_BatchInsertRequest(arg) {
  if (!(arg instanceof hyperspace_pb.BatchInsertRequest)) {
    throw new Error('Expected argument of type hyperspace.BatchInsertRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_BatchInsertRequest(buffer_arg) {
  return hyperspace_pb.BatchInsertRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_BatchSearchRequest(arg) {
  if (!(arg instanceof hyperspace_pb.BatchSearchRequest)) {
    throw new Error('Expected argument of type hyperspace.BatchSearchRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_BatchSearchRequest(buffer_arg) {
  return hyperspace_pb.BatchSearchRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_BatchSearchResponse(arg) {
  if (!(arg instanceof hyperspace_pb.BatchSearchResponse)) {
    throw new Error('Expected argument of type hyperspace.BatchSearchResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_BatchSearchResponse(buffer_arg) {
  return hyperspace_pb.BatchSearchResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_CollectionStatsRequest(arg) {
  if (!(arg instanceof hyperspace_pb.CollectionStatsRequest)) {
    throw new Error('Expected argument of type hyperspace.CollectionStatsRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_CollectionStatsRequest(buffer_arg) {
  return hyperspace_pb.CollectionStatsRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_CollectionStatsResponse(arg) {
  if (!(arg instanceof hyperspace_pb.CollectionStatsResponse)) {
    throw new Error('Expected argument of type hyperspace.CollectionStatsResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_CollectionStatsResponse(buffer_arg) {
  return hyperspace_pb.CollectionStatsResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_ConfigUpdate(arg) {
  if (!(arg instanceof hyperspace_pb.ConfigUpdate)) {
    throw new Error('Expected argument of type hyperspace.ConfigUpdate');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_ConfigUpdate(buffer_arg) {
  return hyperspace_pb.ConfigUpdate.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_CreateCollectionRequest(arg) {
  if (!(arg instanceof hyperspace_pb.CreateCollectionRequest)) {
    throw new Error('Expected argument of type hyperspace.CreateCollectionRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_CreateCollectionRequest(buffer_arg) {
  return hyperspace_pb.CreateCollectionRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_DeleteCollectionRequest(arg) {
  if (!(arg instanceof hyperspace_pb.DeleteCollectionRequest)) {
    throw new Error('Expected argument of type hyperspace.DeleteCollectionRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_DeleteCollectionRequest(buffer_arg) {
  return hyperspace_pb.DeleteCollectionRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_DeleteRequest(arg) {
  if (!(arg instanceof hyperspace_pb.DeleteRequest)) {
    throw new Error('Expected argument of type hyperspace.DeleteRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_DeleteRequest(buffer_arg) {
  return hyperspace_pb.DeleteRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_DeleteResponse(arg) {
  if (!(arg instanceof hyperspace_pb.DeleteResponse)) {
    throw new Error('Expected argument of type hyperspace.DeleteResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_DeleteResponse(buffer_arg) {
  return hyperspace_pb.DeleteResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_DigestRequest(arg) {
  if (!(arg instanceof hyperspace_pb.DigestRequest)) {
    throw new Error('Expected argument of type hyperspace.DigestRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_DigestRequest(buffer_arg) {
  return hyperspace_pb.DigestRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_DigestResponse(arg) {
  if (!(arg instanceof hyperspace_pb.DigestResponse)) {
    throw new Error('Expected argument of type hyperspace.DigestResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_DigestResponse(buffer_arg) {
  return hyperspace_pb.DigestResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_Empty(arg) {
  if (!(arg instanceof hyperspace_pb.Empty)) {
    throw new Error('Expected argument of type hyperspace.Empty');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_Empty(buffer_arg) {
  return hyperspace_pb.Empty.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_InsertRequest(arg) {
  if (!(arg instanceof hyperspace_pb.InsertRequest)) {
    throw new Error('Expected argument of type hyperspace.InsertRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_InsertRequest(buffer_arg) {
  return hyperspace_pb.InsertRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_InsertResponse(arg) {
  if (!(arg instanceof hyperspace_pb.InsertResponse)) {
    throw new Error('Expected argument of type hyperspace.InsertResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_InsertResponse(buffer_arg) {
  return hyperspace_pb.InsertResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_InsertTextRequest(arg) {
  if (!(arg instanceof hyperspace_pb.InsertTextRequest)) {
    throw new Error('Expected argument of type hyperspace.InsertTextRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_InsertTextRequest(buffer_arg) {
  return hyperspace_pb.InsertTextRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_ListCollectionsResponse(arg) {
  if (!(arg instanceof hyperspace_pb.ListCollectionsResponse)) {
    throw new Error('Expected argument of type hyperspace.ListCollectionsResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_ListCollectionsResponse(buffer_arg) {
  return hyperspace_pb.ListCollectionsResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_MonitorRequest(arg) {
  if (!(arg instanceof hyperspace_pb.MonitorRequest)) {
    throw new Error('Expected argument of type hyperspace.MonitorRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_MonitorRequest(buffer_arg) {
  return hyperspace_pb.MonitorRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_RebuildIndexRequest(arg) {
  if (!(arg instanceof hyperspace_pb.RebuildIndexRequest)) {
    throw new Error('Expected argument of type hyperspace.RebuildIndexRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_RebuildIndexRequest(buffer_arg) {
  return hyperspace_pb.RebuildIndexRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_ReplicationLog(arg) {
  if (!(arg instanceof hyperspace_pb.ReplicationLog)) {
    throw new Error('Expected argument of type hyperspace.ReplicationLog');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_ReplicationLog(buffer_arg) {
  return hyperspace_pb.ReplicationLog.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_ReplicationRequest(arg) {
  if (!(arg instanceof hyperspace_pb.ReplicationRequest)) {
    throw new Error('Expected argument of type hyperspace.ReplicationRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_ReplicationRequest(buffer_arg) {
  return hyperspace_pb.ReplicationRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_SearchRequest(arg) {
  if (!(arg instanceof hyperspace_pb.SearchRequest)) {
    throw new Error('Expected argument of type hyperspace.SearchRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_SearchRequest(buffer_arg) {
  return hyperspace_pb.SearchRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_SearchResponse(arg) {
  if (!(arg instanceof hyperspace_pb.SearchResponse)) {
    throw new Error('Expected argument of type hyperspace.SearchResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_SearchResponse(buffer_arg) {
  return hyperspace_pb.SearchResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_StatusResponse(arg) {
  if (!(arg instanceof hyperspace_pb.StatusResponse)) {
    throw new Error('Expected argument of type hyperspace.StatusResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_StatusResponse(buffer_arg) {
  return hyperspace_pb.StatusResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_hyperspace_SystemStats(arg) {
  if (!(arg instanceof hyperspace_pb.SystemStats)) {
    throw new Error('Expected argument of type hyperspace.SystemStats');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_hyperspace_SystemStats(buffer_arg) {
  return hyperspace_pb.SystemStats.deserializeBinary(new Uint8Array(buffer_arg));
}


var DatabaseService = exports.DatabaseService = {
  // Collection Management
createCollection: {
    path: '/hyperspace.Database/CreateCollection',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.CreateCollectionRequest,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_CreateCollectionRequest,
    requestDeserialize: deserialize_hyperspace_CreateCollectionRequest,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
  deleteCollection: {
    path: '/hyperspace.Database/DeleteCollection',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.DeleteCollectionRequest,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_DeleteCollectionRequest,
    requestDeserialize: deserialize_hyperspace_DeleteCollectionRequest,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
  listCollections: {
    path: '/hyperspace.Database/ListCollections',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.Empty,
    responseType: hyperspace_pb.ListCollectionsResponse,
    requestSerialize: serialize_hyperspace_Empty,
    requestDeserialize: deserialize_hyperspace_Empty,
    responseSerialize: serialize_hyperspace_ListCollectionsResponse,
    responseDeserialize: deserialize_hyperspace_ListCollectionsResponse,
  },
  getCollectionStats: {
    path: '/hyperspace.Database/GetCollectionStats',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.CollectionStatsRequest,
    responseType: hyperspace_pb.CollectionStatsResponse,
    requestSerialize: serialize_hyperspace_CollectionStatsRequest,
    requestDeserialize: deserialize_hyperspace_CollectionStatsRequest,
    responseSerialize: serialize_hyperspace_CollectionStatsResponse,
    responseDeserialize: deserialize_hyperspace_CollectionStatsResponse,
  },
  // Insert vectors
insert: {
    path: '/hyperspace.Database/Insert',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.InsertRequest,
    responseType: hyperspace_pb.InsertResponse,
    requestSerialize: serialize_hyperspace_InsertRequest,
    requestDeserialize: deserialize_hyperspace_InsertRequest,
    responseSerialize: serialize_hyperspace_InsertResponse,
    responseDeserialize: deserialize_hyperspace_InsertResponse,
  },
  batchInsert: {
    path: '/hyperspace.Database/BatchInsert',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.BatchInsertRequest,
    responseType: hyperspace_pb.InsertResponse,
    requestSerialize: serialize_hyperspace_BatchInsertRequest,
    requestDeserialize: deserialize_hyperspace_BatchInsertRequest,
    responseSerialize: serialize_hyperspace_InsertResponse,
    responseDeserialize: deserialize_hyperspace_InsertResponse,
  },
  insertText: {
    path: '/hyperspace.Database/InsertText',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.InsertTextRequest,
    responseType: hyperspace_pb.InsertResponse,
    requestSerialize: serialize_hyperspace_InsertTextRequest,
    requestDeserialize: deserialize_hyperspace_InsertTextRequest,
    responseSerialize: serialize_hyperspace_InsertResponse,
    responseDeserialize: deserialize_hyperspace_InsertResponse,
  },
  // Delete vectors
delete: {
    path: '/hyperspace.Database/Delete',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.DeleteRequest,
    responseType: hyperspace_pb.DeleteResponse,
    requestSerialize: serialize_hyperspace_DeleteRequest,
    requestDeserialize: deserialize_hyperspace_DeleteRequest,
    responseSerialize: serialize_hyperspace_DeleteResponse,
    responseDeserialize: deserialize_hyperspace_DeleteResponse,
  },
  // Search (ANN)
search: {
    path: '/hyperspace.Database/Search',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.SearchRequest,
    responseType: hyperspace_pb.SearchResponse,
    requestSerialize: serialize_hyperspace_SearchRequest,
    requestDeserialize: deserialize_hyperspace_SearchRequest,
    responseSerialize: serialize_hyperspace_SearchResponse,
    responseDeserialize: deserialize_hyperspace_SearchResponse,
  },
  // Batch Search (ANN)
searchBatch: {
    path: '/hyperspace.Database/SearchBatch',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.BatchSearchRequest,
    responseType: hyperspace_pb.BatchSearchResponse,
    requestSerialize: serialize_hyperspace_BatchSearchRequest,
    requestDeserialize: deserialize_hyperspace_BatchSearchRequest,
    responseSerialize: serialize_hyperspace_BatchSearchResponse,
    responseDeserialize: deserialize_hyperspace_BatchSearchResponse,
  },
  // Stream statistics for TUI (Global or Collection tailored)
monitor: {
    path: '/hyperspace.Database/Monitor',
    requestStream: false,
    responseStream: true,
    requestType: hyperspace_pb.MonitorRequest,
    responseType: hyperspace_pb.SystemStats,
    requestSerialize: serialize_hyperspace_MonitorRequest,
    requestDeserialize: deserialize_hyperspace_MonitorRequest,
    responseSerialize: serialize_hyperspace_SystemStats,
    responseDeserialize: deserialize_hyperspace_SystemStats,
  },
  // Admin Controls
triggerSnapshot: {
    path: '/hyperspace.Database/TriggerSnapshot',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.Empty,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_Empty,
    requestDeserialize: deserialize_hyperspace_Empty,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
  triggerVacuum: {
    path: '/hyperspace.Database/TriggerVacuum',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.Empty,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_Empty,
    requestDeserialize: deserialize_hyperspace_Empty,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
  // Dynamic Configuration
configure: {
    path: '/hyperspace.Database/Configure',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.ConfigUpdate,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_ConfigUpdate,
    requestDeserialize: deserialize_hyperspace_ConfigUpdate,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
  // Replication (Leader -> Follower)
replicate: {
    path: '/hyperspace.Database/Replicate',
    requestStream: false,
    responseStream: true,
    requestType: hyperspace_pb.ReplicationRequest,
    responseType: hyperspace_pb.ReplicationLog,
    requestSerialize: serialize_hyperspace_ReplicationRequest,
    requestDeserialize: deserialize_hyperspace_ReplicationRequest,
    responseSerialize: serialize_hyperspace_ReplicationLog,
    responseDeserialize: deserialize_hyperspace_ReplicationLog,
  },
  getDigest: {
    path: '/hyperspace.Database/GetDigest',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.DigestRequest,
    responseType: hyperspace_pb.DigestResponse,
    requestSerialize: serialize_hyperspace_DigestRequest,
    requestDeserialize: deserialize_hyperspace_DigestRequest,
    responseSerialize: serialize_hyperspace_DigestResponse,
    responseDeserialize: deserialize_hyperspace_DigestResponse,
  },
  rebuildIndex: {
    path: '/hyperspace.Database/RebuildIndex',
    requestStream: false,
    responseStream: false,
    requestType: hyperspace_pb.RebuildIndexRequest,
    responseType: hyperspace_pb.StatusResponse,
    requestSerialize: serialize_hyperspace_RebuildIndexRequest,
    requestDeserialize: deserialize_hyperspace_RebuildIndexRequest,
    responseSerialize: serialize_hyperspace_StatusResponse,
    responseDeserialize: deserialize_hyperspace_StatusResponse,
  },
};

exports.DatabaseClient = grpc.makeGenericClientConstructor(DatabaseService, 'Database');
