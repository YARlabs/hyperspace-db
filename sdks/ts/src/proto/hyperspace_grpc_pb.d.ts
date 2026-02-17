// package: hyperspace
// file: hyperspace.proto

/* tslint:disable */
/* eslint-disable */

import * as grpc from "@grpc/grpc-js";
import * as hyperspace_pb from "./hyperspace_pb";

interface IDatabaseService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    createCollection: IDatabaseService_ICreateCollection;
    deleteCollection: IDatabaseService_IDeleteCollection;
    listCollections: IDatabaseService_IListCollections;
    getCollectionStats: IDatabaseService_IGetCollectionStats;
    insert: IDatabaseService_IInsert;
    batchInsert: IDatabaseService_IBatchInsert;
    insertText: IDatabaseService_IInsertText;
    delete: IDatabaseService_IDelete;
    search: IDatabaseService_ISearch;
    searchBatch: IDatabaseService_ISearchBatch;
    monitor: IDatabaseService_IMonitor;
    triggerSnapshot: IDatabaseService_ITriggerSnapshot;
    triggerVacuum: IDatabaseService_ITriggerVacuum;
    configure: IDatabaseService_IConfigure;
    replicate: IDatabaseService_IReplicate;
    getDigest: IDatabaseService_IGetDigest;
    rebuildIndex: IDatabaseService_IRebuildIndex;
}

interface IDatabaseService_ICreateCollection extends grpc.MethodDefinition<hyperspace_pb.CreateCollectionRequest, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/CreateCollection";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.CreateCollectionRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.CreateCollectionRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}
interface IDatabaseService_IDeleteCollection extends grpc.MethodDefinition<hyperspace_pb.DeleteCollectionRequest, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/DeleteCollection";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.DeleteCollectionRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.DeleteCollectionRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}
interface IDatabaseService_IListCollections extends grpc.MethodDefinition<hyperspace_pb.Empty, hyperspace_pb.ListCollectionsResponse> {
    path: "/hyperspace.Database/ListCollections";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.Empty>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.Empty>;
    responseSerialize: grpc.serialize<hyperspace_pb.ListCollectionsResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.ListCollectionsResponse>;
}
interface IDatabaseService_IGetCollectionStats extends grpc.MethodDefinition<hyperspace_pb.CollectionStatsRequest, hyperspace_pb.CollectionStatsResponse> {
    path: "/hyperspace.Database/GetCollectionStats";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.CollectionStatsRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.CollectionStatsRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.CollectionStatsResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.CollectionStatsResponse>;
}
interface IDatabaseService_IInsert extends grpc.MethodDefinition<hyperspace_pb.InsertRequest, hyperspace_pb.InsertResponse> {
    path: "/hyperspace.Database/Insert";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.InsertRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.InsertRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.InsertResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.InsertResponse>;
}
interface IDatabaseService_IBatchInsert extends grpc.MethodDefinition<hyperspace_pb.BatchInsertRequest, hyperspace_pb.InsertResponse> {
    path: "/hyperspace.Database/BatchInsert";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.BatchInsertRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.BatchInsertRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.InsertResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.InsertResponse>;
}
interface IDatabaseService_IInsertText extends grpc.MethodDefinition<hyperspace_pb.InsertTextRequest, hyperspace_pb.InsertResponse> {
    path: "/hyperspace.Database/InsertText";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.InsertTextRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.InsertTextRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.InsertResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.InsertResponse>;
}
interface IDatabaseService_IDelete extends grpc.MethodDefinition<hyperspace_pb.DeleteRequest, hyperspace_pb.DeleteResponse> {
    path: "/hyperspace.Database/Delete";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.DeleteRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.DeleteRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.DeleteResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.DeleteResponse>;
}
interface IDatabaseService_ISearch extends grpc.MethodDefinition<hyperspace_pb.SearchRequest, hyperspace_pb.SearchResponse> {
    path: "/hyperspace.Database/Search";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.SearchRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.SearchRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.SearchResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.SearchResponse>;
}
interface IDatabaseService_ISearchBatch extends grpc.MethodDefinition<hyperspace_pb.BatchSearchRequest, hyperspace_pb.BatchSearchResponse> {
    path: "/hyperspace.Database/SearchBatch";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.BatchSearchRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.BatchSearchRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.BatchSearchResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.BatchSearchResponse>;
}
interface IDatabaseService_IMonitor extends grpc.MethodDefinition<hyperspace_pb.MonitorRequest, hyperspace_pb.SystemStats> {
    path: "/hyperspace.Database/Monitor";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<hyperspace_pb.MonitorRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.MonitorRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.SystemStats>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.SystemStats>;
}
interface IDatabaseService_ITriggerSnapshot extends grpc.MethodDefinition<hyperspace_pb.Empty, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/TriggerSnapshot";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.Empty>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.Empty>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}
interface IDatabaseService_ITriggerVacuum extends grpc.MethodDefinition<hyperspace_pb.Empty, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/TriggerVacuum";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.Empty>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.Empty>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}
interface IDatabaseService_IConfigure extends grpc.MethodDefinition<hyperspace_pb.ConfigUpdate, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/Configure";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.ConfigUpdate>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.ConfigUpdate>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}
interface IDatabaseService_IReplicate extends grpc.MethodDefinition<hyperspace_pb.ReplicationRequest, hyperspace_pb.ReplicationLog> {
    path: "/hyperspace.Database/Replicate";
    requestStream: false;
    responseStream: true;
    requestSerialize: grpc.serialize<hyperspace_pb.ReplicationRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.ReplicationRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.ReplicationLog>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.ReplicationLog>;
}
interface IDatabaseService_IGetDigest extends grpc.MethodDefinition<hyperspace_pb.DigestRequest, hyperspace_pb.DigestResponse> {
    path: "/hyperspace.Database/GetDigest";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.DigestRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.DigestRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.DigestResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.DigestResponse>;
}
interface IDatabaseService_IRebuildIndex extends grpc.MethodDefinition<hyperspace_pb.RebuildIndexRequest, hyperspace_pb.StatusResponse> {
    path: "/hyperspace.Database/RebuildIndex";
    requestStream: false;
    responseStream: false;
    requestSerialize: grpc.serialize<hyperspace_pb.RebuildIndexRequest>;
    requestDeserialize: grpc.deserialize<hyperspace_pb.RebuildIndexRequest>;
    responseSerialize: grpc.serialize<hyperspace_pb.StatusResponse>;
    responseDeserialize: grpc.deserialize<hyperspace_pb.StatusResponse>;
}

export const DatabaseService: IDatabaseService;

export interface IDatabaseServer extends grpc.UntypedServiceImplementation {
    createCollection: grpc.handleUnaryCall<hyperspace_pb.CreateCollectionRequest, hyperspace_pb.StatusResponse>;
    deleteCollection: grpc.handleUnaryCall<hyperspace_pb.DeleteCollectionRequest, hyperspace_pb.StatusResponse>;
    listCollections: grpc.handleUnaryCall<hyperspace_pb.Empty, hyperspace_pb.ListCollectionsResponse>;
    getCollectionStats: grpc.handleUnaryCall<hyperspace_pb.CollectionStatsRequest, hyperspace_pb.CollectionStatsResponse>;
    insert: grpc.handleUnaryCall<hyperspace_pb.InsertRequest, hyperspace_pb.InsertResponse>;
    batchInsert: grpc.handleUnaryCall<hyperspace_pb.BatchInsertRequest, hyperspace_pb.InsertResponse>;
    insertText: grpc.handleUnaryCall<hyperspace_pb.InsertTextRequest, hyperspace_pb.InsertResponse>;
    delete: grpc.handleUnaryCall<hyperspace_pb.DeleteRequest, hyperspace_pb.DeleteResponse>;
    search: grpc.handleUnaryCall<hyperspace_pb.SearchRequest, hyperspace_pb.SearchResponse>;
    searchBatch: grpc.handleUnaryCall<hyperspace_pb.BatchSearchRequest, hyperspace_pb.BatchSearchResponse>;
    monitor: grpc.handleServerStreamingCall<hyperspace_pb.MonitorRequest, hyperspace_pb.SystemStats>;
    triggerSnapshot: grpc.handleUnaryCall<hyperspace_pb.Empty, hyperspace_pb.StatusResponse>;
    triggerVacuum: grpc.handleUnaryCall<hyperspace_pb.Empty, hyperspace_pb.StatusResponse>;
    configure: grpc.handleUnaryCall<hyperspace_pb.ConfigUpdate, hyperspace_pb.StatusResponse>;
    replicate: grpc.handleServerStreamingCall<hyperspace_pb.ReplicationRequest, hyperspace_pb.ReplicationLog>;
    getDigest: grpc.handleUnaryCall<hyperspace_pb.DigestRequest, hyperspace_pb.DigestResponse>;
    rebuildIndex: grpc.handleUnaryCall<hyperspace_pb.RebuildIndexRequest, hyperspace_pb.StatusResponse>;
}

export interface IDatabaseClient {
    createCollection(request: hyperspace_pb.CreateCollectionRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    createCollection(request: hyperspace_pb.CreateCollectionRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    createCollection(request: hyperspace_pb.CreateCollectionRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    listCollections(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    listCollections(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    listCollections(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    insert(request: hyperspace_pb.InsertRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    insert(request: hyperspace_pb.InsertRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    insert(request: hyperspace_pb.InsertRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    batchInsert(request: hyperspace_pb.BatchInsertRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    batchInsert(request: hyperspace_pb.BatchInsertRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    batchInsert(request: hyperspace_pb.BatchInsertRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    insertText(request: hyperspace_pb.InsertTextRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    insertText(request: hyperspace_pb.InsertTextRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    insertText(request: hyperspace_pb.InsertTextRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    delete(request: hyperspace_pb.DeleteRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    delete(request: hyperspace_pb.DeleteRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    delete(request: hyperspace_pb.DeleteRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    search(request: hyperspace_pb.SearchRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    search(request: hyperspace_pb.SearchRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    search(request: hyperspace_pb.SearchRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    searchBatch(request: hyperspace_pb.BatchSearchRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    searchBatch(request: hyperspace_pb.BatchSearchRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    searchBatch(request: hyperspace_pb.BatchSearchRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    monitor(request: hyperspace_pb.MonitorRequest, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.SystemStats>;
    monitor(request: hyperspace_pb.MonitorRequest, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.SystemStats>;
    triggerSnapshot(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    triggerSnapshot(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    triggerSnapshot(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    triggerVacuum(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    triggerVacuum(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    triggerVacuum(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    configure(request: hyperspace_pb.ConfigUpdate, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    configure(request: hyperspace_pb.ConfigUpdate, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    configure(request: hyperspace_pb.ConfigUpdate, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    replicate(request: hyperspace_pb.ReplicationRequest, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.ReplicationLog>;
    replicate(request: hyperspace_pb.ReplicationRequest, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.ReplicationLog>;
    getDigest(request: hyperspace_pb.DigestRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    getDigest(request: hyperspace_pb.DigestRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    getDigest(request: hyperspace_pb.DigestRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
}

export class DatabaseClient extends grpc.Client implements IDatabaseClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: Partial<grpc.ClientOptions>);
    public createCollection(request: hyperspace_pb.CreateCollectionRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public createCollection(request: hyperspace_pb.CreateCollectionRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public createCollection(request: hyperspace_pb.CreateCollectionRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public deleteCollection(request: hyperspace_pb.DeleteCollectionRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public listCollections(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    public listCollections(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    public listCollections(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.ListCollectionsResponse) => void): grpc.ClientUnaryCall;
    public getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    public getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    public getCollectionStats(request: hyperspace_pb.CollectionStatsRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.CollectionStatsResponse) => void): grpc.ClientUnaryCall;
    public insert(request: hyperspace_pb.InsertRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public insert(request: hyperspace_pb.InsertRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public insert(request: hyperspace_pb.InsertRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public batchInsert(request: hyperspace_pb.BatchInsertRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public batchInsert(request: hyperspace_pb.BatchInsertRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public batchInsert(request: hyperspace_pb.BatchInsertRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public insertText(request: hyperspace_pb.InsertTextRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public insertText(request: hyperspace_pb.InsertTextRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public insertText(request: hyperspace_pb.InsertTextRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.InsertResponse) => void): grpc.ClientUnaryCall;
    public delete(request: hyperspace_pb.DeleteRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    public delete(request: hyperspace_pb.DeleteRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    public delete(request: hyperspace_pb.DeleteRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DeleteResponse) => void): grpc.ClientUnaryCall;
    public search(request: hyperspace_pb.SearchRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    public search(request: hyperspace_pb.SearchRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    public search(request: hyperspace_pb.SearchRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.SearchResponse) => void): grpc.ClientUnaryCall;
    public searchBatch(request: hyperspace_pb.BatchSearchRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    public searchBatch(request: hyperspace_pb.BatchSearchRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    public searchBatch(request: hyperspace_pb.BatchSearchRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.BatchSearchResponse) => void): grpc.ClientUnaryCall;
    public monitor(request: hyperspace_pb.MonitorRequest, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.SystemStats>;
    public monitor(request: hyperspace_pb.MonitorRequest, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.SystemStats>;
    public triggerSnapshot(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public triggerSnapshot(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public triggerSnapshot(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public triggerVacuum(request: hyperspace_pb.Empty, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public triggerVacuum(request: hyperspace_pb.Empty, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public triggerVacuum(request: hyperspace_pb.Empty, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public configure(request: hyperspace_pb.ConfigUpdate, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public configure(request: hyperspace_pb.ConfigUpdate, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public configure(request: hyperspace_pb.ConfigUpdate, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public replicate(request: hyperspace_pb.ReplicationRequest, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.ReplicationLog>;
    public replicate(request: hyperspace_pb.ReplicationRequest, metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientReadableStream<hyperspace_pb.ReplicationLog>;
    public getDigest(request: hyperspace_pb.DigestRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    public getDigest(request: hyperspace_pb.DigestRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    public getDigest(request: hyperspace_pb.DigestRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.DigestResponse) => void): grpc.ClientUnaryCall;
    public rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
    public rebuildIndex(request: hyperspace_pb.RebuildIndexRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: hyperspace_pb.StatusResponse) => void): grpc.ClientUnaryCall;
}
