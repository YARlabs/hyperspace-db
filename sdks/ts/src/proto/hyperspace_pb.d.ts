// package: hyperspace
// file: hyperspace.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";

export class ReplicationRequest extends jspb.Message { 
    getLastLogicalClock(): number;
    setLastLogicalClock(value: number): ReplicationRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ReplicationRequest.AsObject;
    static toObject(includeInstance: boolean, msg: ReplicationRequest): ReplicationRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ReplicationRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ReplicationRequest;
    static deserializeBinaryFromReader(message: ReplicationRequest, reader: jspb.BinaryReader): ReplicationRequest;
}

export namespace ReplicationRequest {
    export type AsObject = {
        lastLogicalClock: number,
    }
}

export class ReplicationLog extends jspb.Message { 
    getLogicalClock(): number;
    setLogicalClock(value: number): ReplicationLog;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): ReplicationLog;
    getCollection(): string;
    setCollection(value: string): ReplicationLog;

    hasInsert(): boolean;
    clearInsert(): void;
    getInsert(): InsertOp | undefined;
    setInsert(value?: InsertOp): ReplicationLog;

    hasCreateCollection(): boolean;
    clearCreateCollection(): void;
    getCreateCollection(): CreateCollectionOp | undefined;
    setCreateCollection(value?: CreateCollectionOp): ReplicationLog;

    hasDeleteCollection(): boolean;
    clearDeleteCollection(): void;
    getDeleteCollection(): DeleteCollectionOp | undefined;
    setDeleteCollection(value?: DeleteCollectionOp): ReplicationLog;

    hasDelete(): boolean;
    clearDelete(): void;
    getDelete(): DeleteOp | undefined;
    setDelete(value?: DeleteOp): ReplicationLog;

    getOperationCase(): ReplicationLog.OperationCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ReplicationLog.AsObject;
    static toObject(includeInstance: boolean, msg: ReplicationLog): ReplicationLog.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ReplicationLog, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ReplicationLog;
    static deserializeBinaryFromReader(message: ReplicationLog, reader: jspb.BinaryReader): ReplicationLog;
}

export namespace ReplicationLog {
    export type AsObject = {
        logicalClock: number,
        originNodeId: string,
        collection: string,
        insert?: InsertOp.AsObject,
        createCollection?: CreateCollectionOp.AsObject,
        deleteCollection?: DeleteCollectionOp.AsObject,
        pb_delete?: DeleteOp.AsObject,
    }

    export enum OperationCase {
        OPERATION_NOT_SET = 0,
        INSERT = 4,
        CREATE_COLLECTION = 5,
        DELETE_COLLECTION = 6,
        DELETE = 7,
    }

}

export class InsertOp extends jspb.Message { 
    getId(): number;
    setId(value: number): InsertOp;
    clearVectorList(): void;
    getVectorList(): Array<number>;
    setVectorList(value: Array<number>): InsertOp;
    addVector(value: number, index?: number): number;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): InsertOp.AsObject;
    static toObject(includeInstance: boolean, msg: InsertOp): InsertOp.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: InsertOp, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): InsertOp;
    static deserializeBinaryFromReader(message: InsertOp, reader: jspb.BinaryReader): InsertOp;
}

export namespace InsertOp {
    export type AsObject = {
        id: number,
        vectorList: Array<number>,

        metadataMap: Array<[string, string]>,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class CreateCollectionOp extends jspb.Message { 
    getDimension(): number;
    setDimension(value: number): CreateCollectionOp;
    getMetric(): string;
    setMetric(value: string): CreateCollectionOp;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CreateCollectionOp.AsObject;
    static toObject(includeInstance: boolean, msg: CreateCollectionOp): CreateCollectionOp.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CreateCollectionOp, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CreateCollectionOp;
    static deserializeBinaryFromReader(message: CreateCollectionOp, reader: jspb.BinaryReader): CreateCollectionOp;
}

export namespace CreateCollectionOp {
    export type AsObject = {
        dimension: number,
        metric: string,
    }
}

export class DeleteCollectionOp extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeleteCollectionOp.AsObject;
    static toObject(includeInstance: boolean, msg: DeleteCollectionOp): DeleteCollectionOp.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeleteCollectionOp, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeleteCollectionOp;
    static deserializeBinaryFromReader(message: DeleteCollectionOp, reader: jspb.BinaryReader): DeleteCollectionOp;
}

export namespace DeleteCollectionOp {
    export type AsObject = {
    }
}

export class DeleteOp extends jspb.Message { 
    getId(): number;
    setId(value: number): DeleteOp;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeleteOp.AsObject;
    static toObject(includeInstance: boolean, msg: DeleteOp): DeleteOp.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeleteOp, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeleteOp;
    static deserializeBinaryFromReader(message: DeleteOp, reader: jspb.BinaryReader): DeleteOp;
}

export namespace DeleteOp {
    export type AsObject = {
        id: number,
    }
}

export class QuantizationConfig extends jspb.Message { 
    getMode(): QuantizationMode;
    setMode(value: QuantizationMode): QuantizationConfig;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): QuantizationConfig.AsObject;
    static toObject(includeInstance: boolean, msg: QuantizationConfig): QuantizationConfig.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: QuantizationConfig, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): QuantizationConfig;
    static deserializeBinaryFromReader(message: QuantizationConfig, reader: jspb.BinaryReader): QuantizationConfig;
}

export namespace QuantizationConfig {
    export type AsObject = {
        mode: QuantizationMode,
    }
}

export class CreateCollectionRequest extends jspb.Message { 
    getName(): string;
    setName(value: string): CreateCollectionRequest;
    getDimension(): number;
    setDimension(value: number): CreateCollectionRequest;
    getMetric(): string;
    setMetric(value: string): CreateCollectionRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CreateCollectionRequest.AsObject;
    static toObject(includeInstance: boolean, msg: CreateCollectionRequest): CreateCollectionRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CreateCollectionRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CreateCollectionRequest;
    static deserializeBinaryFromReader(message: CreateCollectionRequest, reader: jspb.BinaryReader): CreateCollectionRequest;
}

export namespace CreateCollectionRequest {
    export type AsObject = {
        name: string,
        dimension: number,
        metric: string,
    }
}

export class DeleteCollectionRequest extends jspb.Message { 
    getName(): string;
    setName(value: string): DeleteCollectionRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeleteCollectionRequest.AsObject;
    static toObject(includeInstance: boolean, msg: DeleteCollectionRequest): DeleteCollectionRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeleteCollectionRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeleteCollectionRequest;
    static deserializeBinaryFromReader(message: DeleteCollectionRequest, reader: jspb.BinaryReader): DeleteCollectionRequest;
}

export namespace DeleteCollectionRequest {
    export type AsObject = {
        name: string,
    }
}

export class ListCollectionsResponse extends jspb.Message { 
    clearCollectionsList(): void;
    getCollectionsList(): Array<string>;
    setCollectionsList(value: Array<string>): ListCollectionsResponse;
    addCollections(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ListCollectionsResponse.AsObject;
    static toObject(includeInstance: boolean, msg: ListCollectionsResponse): ListCollectionsResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ListCollectionsResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ListCollectionsResponse;
    static deserializeBinaryFromReader(message: ListCollectionsResponse, reader: jspb.BinaryReader): ListCollectionsResponse;
}

export namespace ListCollectionsResponse {
    export type AsObject = {
        collectionsList: Array<string>,
    }
}

export class CollectionStatsRequest extends jspb.Message { 
    getName(): string;
    setName(value: string): CollectionStatsRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CollectionStatsRequest.AsObject;
    static toObject(includeInstance: boolean, msg: CollectionStatsRequest): CollectionStatsRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CollectionStatsRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CollectionStatsRequest;
    static deserializeBinaryFromReader(message: CollectionStatsRequest, reader: jspb.BinaryReader): CollectionStatsRequest;
}

export namespace CollectionStatsRequest {
    export type AsObject = {
        name: string,
    }
}

export class CollectionStatsResponse extends jspb.Message { 
    getCount(): number;
    setCount(value: number): CollectionStatsResponse;
    getDimension(): number;
    setDimension(value: number): CollectionStatsResponse;
    getMetric(): string;
    setMetric(value: string): CollectionStatsResponse;
    getIndexingQueue(): number;
    setIndexingQueue(value: number): CollectionStatsResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CollectionStatsResponse.AsObject;
    static toObject(includeInstance: boolean, msg: CollectionStatsResponse): CollectionStatsResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CollectionStatsResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CollectionStatsResponse;
    static deserializeBinaryFromReader(message: CollectionStatsResponse, reader: jspb.BinaryReader): CollectionStatsResponse;
}

export namespace CollectionStatsResponse {
    export type AsObject = {
        count: number,
        dimension: number,
        metric: string,
        indexingQueue: number,
    }
}

export class RebuildIndexRequest extends jspb.Message { 
    getName(): string;
    setName(value: string): RebuildIndexRequest;

    hasFilterQuery(): boolean;
    clearFilterQuery(): void;
    getFilterQuery(): VacuumFilterQuery | undefined;
    setFilterQuery(value?: VacuumFilterQuery): RebuildIndexRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RebuildIndexRequest.AsObject;
    static toObject(includeInstance: boolean, msg: RebuildIndexRequest): RebuildIndexRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RebuildIndexRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RebuildIndexRequest;
    static deserializeBinaryFromReader(message: RebuildIndexRequest, reader: jspb.BinaryReader): RebuildIndexRequest;
}

export namespace RebuildIndexRequest {
    export type AsObject = {
        name: string,
        filterQuery?: VacuumFilterQuery.AsObject,
    }
}

export class ConfigUpdate extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): ConfigUpdate;

    hasEfSearch(): boolean;
    clearEfSearch(): void;
    getEfSearch(): number | undefined;
    setEfSearch(value: number): ConfigUpdate;

    hasEfConstruction(): boolean;
    clearEfConstruction(): void;
    getEfConstruction(): number | undefined;
    setEfConstruction(value: number): ConfigUpdate;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ConfigUpdate.AsObject;
    static toObject(includeInstance: boolean, msg: ConfigUpdate): ConfigUpdate.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ConfigUpdate, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ConfigUpdate;
    static deserializeBinaryFromReader(message: ConfigUpdate, reader: jspb.BinaryReader): ConfigUpdate;
}

export namespace ConfigUpdate {
    export type AsObject = {
        collection: string,
        efSearch?: number,
        efConstruction?: number,
    }
}

export class VacuumFilterQuery extends jspb.Message { 
    getKey(): string;
    setKey(value: string): VacuumFilterQuery;
    getOp(): string;
    setOp(value: string): VacuumFilterQuery;
    getValue(): number;
    setValue(value: number): VacuumFilterQuery;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VacuumFilterQuery.AsObject;
    static toObject(includeInstance: boolean, msg: VacuumFilterQuery): VacuumFilterQuery.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VacuumFilterQuery, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VacuumFilterQuery;
    static deserializeBinaryFromReader(message: VacuumFilterQuery, reader: jspb.BinaryReader): VacuumFilterQuery;
}

export namespace VacuumFilterQuery {
    export type AsObject = {
        key: string,
        op: string,
        value: number,
    }
}

export class InsertRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): InsertRequest;
    clearVectorList(): void;
    getVectorList(): Array<number>;
    setVectorList(value: Array<number>): InsertRequest;
    addVector(value: number, index?: number): number;
    getId(): number;
    setId(value: number): InsertRequest;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): InsertRequest;
    getLogicalClock(): number;
    setLogicalClock(value: number): InsertRequest;
    getDurability(): DurabilityLevel;
    setDurability(value: DurabilityLevel): InsertRequest;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): InsertRequest.AsObject;
    static toObject(includeInstance: boolean, msg: InsertRequest): InsertRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: InsertRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): InsertRequest;
    static deserializeBinaryFromReader(message: InsertRequest, reader: jspb.BinaryReader): InsertRequest;
}

export namespace InsertRequest {
    export type AsObject = {
        collection: string,
        vectorList: Array<number>,
        id: number,

        metadataMap: Array<[string, string]>,
        originNodeId: string,
        logicalClock: number,
        durability: DurabilityLevel,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class VectorData extends jspb.Message { 
    clearVectorList(): void;
    getVectorList(): Array<number>;
    setVectorList(value: Array<number>): VectorData;
    addVector(value: number, index?: number): number;
    getId(): number;
    setId(value: number): VectorData;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VectorData.AsObject;
    static toObject(includeInstance: boolean, msg: VectorData): VectorData.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VectorData, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VectorData;
    static deserializeBinaryFromReader(message: VectorData, reader: jspb.BinaryReader): VectorData;
}

export namespace VectorData {
    export type AsObject = {
        vectorList: Array<number>,
        id: number,

        metadataMap: Array<[string, string]>,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class BatchInsertRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): BatchInsertRequest;
    clearVectorsList(): void;
    getVectorsList(): Array<VectorData>;
    setVectorsList(value: Array<VectorData>): BatchInsertRequest;
    addVectors(value?: VectorData, index?: number): VectorData;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): BatchInsertRequest;
    getLogicalClock(): number;
    setLogicalClock(value: number): BatchInsertRequest;
    getDurability(): DurabilityLevel;
    setDurability(value: DurabilityLevel): BatchInsertRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): BatchInsertRequest.AsObject;
    static toObject(includeInstance: boolean, msg: BatchInsertRequest): BatchInsertRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: BatchInsertRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): BatchInsertRequest;
    static deserializeBinaryFromReader(message: BatchInsertRequest, reader: jspb.BinaryReader): BatchInsertRequest;
}

export namespace BatchInsertRequest {
    export type AsObject = {
        collection: string,
        vectorsList: Array<VectorData.AsObject>,
        originNodeId: string,
        logicalClock: number,
        durability: DurabilityLevel,
    }
}

export class InsertTextRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): InsertTextRequest;
    getId(): number;
    setId(value: number): InsertTextRequest;
    getText(): string;
    setText(value: string): InsertTextRequest;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;
    getDurability(): DurabilityLevel;
    setDurability(value: DurabilityLevel): InsertTextRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): InsertTextRequest.AsObject;
    static toObject(includeInstance: boolean, msg: InsertTextRequest): InsertTextRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: InsertTextRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): InsertTextRequest;
    static deserializeBinaryFromReader(message: InsertTextRequest, reader: jspb.BinaryReader): InsertTextRequest;
}

export namespace InsertTextRequest {
    export type AsObject = {
        collection: string,
        id: number,
        text: string,

        metadataMap: Array<[string, string]>,
        durability: DurabilityLevel,
    }
}

export class InsertResponse extends jspb.Message { 
    getSuccess(): boolean;
    setSuccess(value: boolean): InsertResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): InsertResponse.AsObject;
    static toObject(includeInstance: boolean, msg: InsertResponse): InsertResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: InsertResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): InsertResponse;
    static deserializeBinaryFromReader(message: InsertResponse, reader: jspb.BinaryReader): InsertResponse;
}

export namespace InsertResponse {
    export type AsObject = {
        success: boolean,
    }
}

export class DeleteRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): DeleteRequest;
    getId(): number;
    setId(value: number): DeleteRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeleteRequest.AsObject;
    static toObject(includeInstance: boolean, msg: DeleteRequest): DeleteRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeleteRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeleteRequest;
    static deserializeBinaryFromReader(message: DeleteRequest, reader: jspb.BinaryReader): DeleteRequest;
}

export namespace DeleteRequest {
    export type AsObject = {
        collection: string,
        id: number,
    }
}

export class DeleteResponse extends jspb.Message { 
    getSuccess(): boolean;
    setSuccess(value: boolean): DeleteResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeleteResponse.AsObject;
    static toObject(includeInstance: boolean, msg: DeleteResponse): DeleteResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeleteResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeleteResponse;
    static deserializeBinaryFromReader(message: DeleteResponse, reader: jspb.BinaryReader): DeleteResponse;
}

export namespace DeleteResponse {
    export type AsObject = {
        success: boolean,
    }
}

export class SearchRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): SearchRequest;
    clearVectorList(): void;
    getVectorList(): Array<number>;
    setVectorList(value: Array<number>): SearchRequest;
    addVector(value: number, index?: number): number;
    getTopK(): number;
    setTopK(value: number): SearchRequest;

    getFilterMap(): jspb.Map<string, string>;
    clearFilterMap(): void;
    clearFiltersList(): void;
    getFiltersList(): Array<Filter>;
    setFiltersList(value: Array<Filter>): SearchRequest;
    addFilters(value?: Filter, index?: number): Filter;

    hasHybridQuery(): boolean;
    clearHybridQuery(): void;
    getHybridQuery(): string | undefined;
    setHybridQuery(value: string): SearchRequest;

    hasHybridAlpha(): boolean;
    clearHybridAlpha(): void;
    getHybridAlpha(): number | undefined;
    setHybridAlpha(value: number): SearchRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchRequest.AsObject;
    static toObject(includeInstance: boolean, msg: SearchRequest): SearchRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchRequest;
    static deserializeBinaryFromReader(message: SearchRequest, reader: jspb.BinaryReader): SearchRequest;
}

export namespace SearchRequest {
    export type AsObject = {
        collection: string,
        vectorList: Array<number>,
        topK: number,

        filterMap: Array<[string, string]>,
        filtersList: Array<Filter.AsObject>,
        hybridQuery?: string,
        hybridAlpha?: number,
    }
}

export class Filter extends jspb.Message { 

    hasMatch(): boolean;
    clearMatch(): void;
    getMatch(): Match | undefined;
    setMatch(value?: Match): Filter;

    hasRange(): boolean;
    clearRange(): void;
    getRange(): Range | undefined;
    setRange(value?: Range): Filter;

    getConditionCase(): Filter.ConditionCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Filter.AsObject;
    static toObject(includeInstance: boolean, msg: Filter): Filter.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Filter, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Filter;
    static deserializeBinaryFromReader(message: Filter, reader: jspb.BinaryReader): Filter;
}

export namespace Filter {
    export type AsObject = {
        match?: Match.AsObject,
        range?: Range.AsObject,
    }

    export enum ConditionCase {
        CONDITION_NOT_SET = 0,
        MATCH = 1,
        RANGE = 2,
    }

}

export class Match extends jspb.Message { 
    getKey(): string;
    setKey(value: string): Match;
    getValue(): string;
    setValue(value: string): Match;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Match.AsObject;
    static toObject(includeInstance: boolean, msg: Match): Match.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Match, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Match;
    static deserializeBinaryFromReader(message: Match, reader: jspb.BinaryReader): Match;
}

export namespace Match {
    export type AsObject = {
        key: string,
        value: string,
    }
}

export class Range extends jspb.Message { 
    getKey(): string;
    setKey(value: string): Range;

    hasGte(): boolean;
    clearGte(): void;
    getGte(): number | undefined;
    setGte(value: number): Range;

    hasLte(): boolean;
    clearLte(): void;
    getLte(): number | undefined;
    setLte(value: number): Range;

    hasGteF64(): boolean;
    clearGteF64(): void;
    getGteF64(): number | undefined;
    setGteF64(value: number): Range;

    hasLteF64(): boolean;
    clearLteF64(): void;
    getLteF64(): number | undefined;
    setLteF64(value: number): Range;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Range.AsObject;
    static toObject(includeInstance: boolean, msg: Range): Range.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Range, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Range;
    static deserializeBinaryFromReader(message: Range, reader: jspb.BinaryReader): Range;
}

export namespace Range {
    export type AsObject = {
        key: string,
        gte?: number,
        lte?: number,
        gteF64?: number,
        lteF64?: number,
    }
}

export class SearchResponse extends jspb.Message { 
    clearResultsList(): void;
    getResultsList(): Array<SearchResult>;
    setResultsList(value: Array<SearchResult>): SearchResponse;
    addResults(value?: SearchResult, index?: number): SearchResult;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchResponse.AsObject;
    static toObject(includeInstance: boolean, msg: SearchResponse): SearchResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchResponse;
    static deserializeBinaryFromReader(message: SearchResponse, reader: jspb.BinaryReader): SearchResponse;
}

export namespace SearchResponse {
    export type AsObject = {
        resultsList: Array<SearchResult.AsObject>,
    }
}

export class BatchSearchRequest extends jspb.Message { 
    clearSearchesList(): void;
    getSearchesList(): Array<SearchRequest>;
    setSearchesList(value: Array<SearchRequest>): BatchSearchRequest;
    addSearches(value?: SearchRequest, index?: number): SearchRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): BatchSearchRequest.AsObject;
    static toObject(includeInstance: boolean, msg: BatchSearchRequest): BatchSearchRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: BatchSearchRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): BatchSearchRequest;
    static deserializeBinaryFromReader(message: BatchSearchRequest, reader: jspb.BinaryReader): BatchSearchRequest;
}

export namespace BatchSearchRequest {
    export type AsObject = {
        searchesList: Array<SearchRequest.AsObject>,
    }
}

export class BatchSearchResponse extends jspb.Message { 
    clearResponsesList(): void;
    getResponsesList(): Array<SearchResponse>;
    setResponsesList(value: Array<SearchResponse>): BatchSearchResponse;
    addResponses(value?: SearchResponse, index?: number): SearchResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): BatchSearchResponse.AsObject;
    static toObject(includeInstance: boolean, msg: BatchSearchResponse): BatchSearchResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: BatchSearchResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): BatchSearchResponse;
    static deserializeBinaryFromReader(message: BatchSearchResponse, reader: jspb.BinaryReader): BatchSearchResponse;
}

export namespace BatchSearchResponse {
    export type AsObject = {
        responsesList: Array<SearchResponse.AsObject>,
    }
}

export class SearchResult extends jspb.Message { 
    getId(): number;
    setId(value: number): SearchResult;
    getDistance(): number;
    setDistance(value: number): SearchResult;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchResult.AsObject;
    static toObject(includeInstance: boolean, msg: SearchResult): SearchResult.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchResult, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchResult;
    static deserializeBinaryFromReader(message: SearchResult, reader: jspb.BinaryReader): SearchResult;
}

export namespace SearchResult {
    export type AsObject = {
        id: number,
        distance: number,

        metadataMap: Array<[string, string]>,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class GetNodeRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): GetNodeRequest;
    getId(): number;
    setId(value: number): GetNodeRequest;
    getLayer(): number;
    setLayer(value: number): GetNodeRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetNodeRequest.AsObject;
    static toObject(includeInstance: boolean, msg: GetNodeRequest): GetNodeRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetNodeRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetNodeRequest;
    static deserializeBinaryFromReader(message: GetNodeRequest, reader: jspb.BinaryReader): GetNodeRequest;
}

export namespace GetNodeRequest {
    export type AsObject = {
        collection: string,
        id: number,
        layer: number,
    }
}

export class GraphNode extends jspb.Message { 
    getId(): number;
    setId(value: number): GraphNode;
    getLayer(): number;
    setLayer(value: number): GraphNode;
    clearNeighborsList(): void;
    getNeighborsList(): Array<number>;
    setNeighborsList(value: Array<number>): GraphNode;
    addNeighbors(value: number, index?: number): number;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GraphNode.AsObject;
    static toObject(includeInstance: boolean, msg: GraphNode): GraphNode.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GraphNode, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GraphNode;
    static deserializeBinaryFromReader(message: GraphNode, reader: jspb.BinaryReader): GraphNode;
}

export namespace GraphNode {
    export type AsObject = {
        id: number,
        layer: number,
        neighborsList: Array<number>,

        metadataMap: Array<[string, string]>,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class GetNeighborsRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): GetNeighborsRequest;
    getId(): number;
    setId(value: number): GetNeighborsRequest;
    getLayer(): number;
    setLayer(value: number): GetNeighborsRequest;
    getLimit(): number;
    setLimit(value: number): GetNeighborsRequest;
    getOffset(): number;
    setOffset(value: number): GetNeighborsRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetNeighborsRequest.AsObject;
    static toObject(includeInstance: boolean, msg: GetNeighborsRequest): GetNeighborsRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetNeighborsRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetNeighborsRequest;
    static deserializeBinaryFromReader(message: GetNeighborsRequest, reader: jspb.BinaryReader): GetNeighborsRequest;
}

export namespace GetNeighborsRequest {
    export type AsObject = {
        collection: string,
        id: number,
        layer: number,
        limit: number,
        offset: number,
    }
}

export class GetNeighborsResponse extends jspb.Message { 
    clearNeighborsList(): void;
    getNeighborsList(): Array<GraphNode>;
    setNeighborsList(value: Array<GraphNode>): GetNeighborsResponse;
    addNeighbors(value?: GraphNode, index?: number): GraphNode;
    clearEdgeWeightsList(): void;
    getEdgeWeightsList(): Array<number>;
    setEdgeWeightsList(value: Array<number>): GetNeighborsResponse;
    addEdgeWeights(value: number, index?: number): number;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetNeighborsResponse.AsObject;
    static toObject(includeInstance: boolean, msg: GetNeighborsResponse): GetNeighborsResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetNeighborsResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetNeighborsResponse;
    static deserializeBinaryFromReader(message: GetNeighborsResponse, reader: jspb.BinaryReader): GetNeighborsResponse;
}

export namespace GetNeighborsResponse {
    export type AsObject = {
        neighborsList: Array<GraphNode.AsObject>,
        edgeWeightsList: Array<number>,
    }
}

export class TraverseRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): TraverseRequest;
    getStartId(): number;
    setStartId(value: number): TraverseRequest;
    getMaxDepth(): number;
    setMaxDepth(value: number): TraverseRequest;
    getMaxNodes(): number;
    setMaxNodes(value: number): TraverseRequest;
    getLayer(): number;
    setLayer(value: number): TraverseRequest;

    getFilterMap(): jspb.Map<string, string>;
    clearFilterMap(): void;
    clearFiltersList(): void;
    getFiltersList(): Array<Filter>;
    setFiltersList(value: Array<Filter>): TraverseRequest;
    addFilters(value?: Filter, index?: number): Filter;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TraverseRequest.AsObject;
    static toObject(includeInstance: boolean, msg: TraverseRequest): TraverseRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TraverseRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TraverseRequest;
    static deserializeBinaryFromReader(message: TraverseRequest, reader: jspb.BinaryReader): TraverseRequest;
}

export namespace TraverseRequest {
    export type AsObject = {
        collection: string,
        startId: number,
        maxDepth: number,
        maxNodes: number,
        layer: number,

        filterMap: Array<[string, string]>,
        filtersList: Array<Filter.AsObject>,
    }
}

export class TraverseResponse extends jspb.Message { 
    clearNodesList(): void;
    getNodesList(): Array<GraphNode>;
    setNodesList(value: Array<GraphNode>): TraverseResponse;
    addNodes(value?: GraphNode, index?: number): GraphNode;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TraverseResponse.AsObject;
    static toObject(includeInstance: boolean, msg: TraverseResponse): TraverseResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TraverseResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TraverseResponse;
    static deserializeBinaryFromReader(message: TraverseResponse, reader: jspb.BinaryReader): TraverseResponse;
}

export namespace TraverseResponse {
    export type AsObject = {
        nodesList: Array<GraphNode.AsObject>,
    }
}

export class FindSemanticClustersRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): FindSemanticClustersRequest;
    getLayer(): number;
    setLayer(value: number): FindSemanticClustersRequest;
    getMinClusterSize(): number;
    setMinClusterSize(value: number): FindSemanticClustersRequest;
    getMaxClusters(): number;
    setMaxClusters(value: number): FindSemanticClustersRequest;
    getMaxNodes(): number;
    setMaxNodes(value: number): FindSemanticClustersRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FindSemanticClustersRequest.AsObject;
    static toObject(includeInstance: boolean, msg: FindSemanticClustersRequest): FindSemanticClustersRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FindSemanticClustersRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FindSemanticClustersRequest;
    static deserializeBinaryFromReader(message: FindSemanticClustersRequest, reader: jspb.BinaryReader): FindSemanticClustersRequest;
}

export namespace FindSemanticClustersRequest {
    export type AsObject = {
        collection: string,
        layer: number,
        minClusterSize: number,
        maxClusters: number,
        maxNodes: number,
    }
}

export class GetConceptParentsRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): GetConceptParentsRequest;
    getId(): number;
    setId(value: number): GetConceptParentsRequest;
    getLayer(): number;
    setLayer(value: number): GetConceptParentsRequest;
    getLimit(): number;
    setLimit(value: number): GetConceptParentsRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetConceptParentsRequest.AsObject;
    static toObject(includeInstance: boolean, msg: GetConceptParentsRequest): GetConceptParentsRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetConceptParentsRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetConceptParentsRequest;
    static deserializeBinaryFromReader(message: GetConceptParentsRequest, reader: jspb.BinaryReader): GetConceptParentsRequest;
}

export namespace GetConceptParentsRequest {
    export type AsObject = {
        collection: string,
        id: number,
        layer: number,
        limit: number,
    }
}

export class GetConceptParentsResponse extends jspb.Message { 
    clearParentsList(): void;
    getParentsList(): Array<GraphNode>;
    setParentsList(value: Array<GraphNode>): GetConceptParentsResponse;
    addParents(value?: GraphNode, index?: number): GraphNode;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetConceptParentsResponse.AsObject;
    static toObject(includeInstance: boolean, msg: GetConceptParentsResponse): GetConceptParentsResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetConceptParentsResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetConceptParentsResponse;
    static deserializeBinaryFromReader(message: GetConceptParentsResponse, reader: jspb.BinaryReader): GetConceptParentsResponse;
}

export namespace GetConceptParentsResponse {
    export type AsObject = {
        parentsList: Array<GraphNode.AsObject>,
    }
}

export class GraphCluster extends jspb.Message { 
    clearNodeIdsList(): void;
    getNodeIdsList(): Array<number>;
    setNodeIdsList(value: Array<number>): GraphCluster;
    addNodeIds(value: number, index?: number): number;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GraphCluster.AsObject;
    static toObject(includeInstance: boolean, msg: GraphCluster): GraphCluster.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GraphCluster, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GraphCluster;
    static deserializeBinaryFromReader(message: GraphCluster, reader: jspb.BinaryReader): GraphCluster;
}

export namespace GraphCluster {
    export type AsObject = {
        nodeIdsList: Array<number>,
    }
}

export class FindSemanticClustersResponse extends jspb.Message { 
    clearClustersList(): void;
    getClustersList(): Array<GraphCluster>;
    setClustersList(value: Array<GraphCluster>): FindSemanticClustersResponse;
    addClusters(value?: GraphCluster, index?: number): GraphCluster;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FindSemanticClustersResponse.AsObject;
    static toObject(includeInstance: boolean, msg: FindSemanticClustersResponse): FindSemanticClustersResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FindSemanticClustersResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FindSemanticClustersResponse;
    static deserializeBinaryFromReader(message: FindSemanticClustersResponse, reader: jspb.BinaryReader): FindSemanticClustersResponse;
}

export namespace FindSemanticClustersResponse {
    export type AsObject = {
        clustersList: Array<GraphCluster.AsObject>,
    }
}

export class MetadataValue extends jspb.Message { 

    hasStringValue(): boolean;
    clearStringValue(): void;
    getStringValue(): string;
    setStringValue(value: string): MetadataValue;

    hasIntValue(): boolean;
    clearIntValue(): void;
    getIntValue(): number;
    setIntValue(value: number): MetadataValue;

    hasDoubleValue(): boolean;
    clearDoubleValue(): void;
    getDoubleValue(): number;
    setDoubleValue(value: number): MetadataValue;

    hasBoolValue(): boolean;
    clearBoolValue(): void;
    getBoolValue(): boolean;
    setBoolValue(value: boolean): MetadataValue;

    getKindCase(): MetadataValue.KindCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): MetadataValue.AsObject;
    static toObject(includeInstance: boolean, msg: MetadataValue): MetadataValue.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: MetadataValue, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): MetadataValue;
    static deserializeBinaryFromReader(message: MetadataValue, reader: jspb.BinaryReader): MetadataValue;
}

export namespace MetadataValue {
    export type AsObject = {
        stringValue: string,
        intValue: number,
        doubleValue: number,
        boolValue: boolean,
    }

    export enum KindCase {
        KIND_NOT_SET = 0,
        STRING_VALUE = 1,
        INT_VALUE = 2,
        DOUBLE_VALUE = 3,
        BOOL_VALUE = 4,
    }

}

export class EventSubscriptionRequest extends jspb.Message { 
    clearTypesList(): void;
    getTypesList(): Array<EventType>;
    setTypesList(value: Array<EventType>): EventSubscriptionRequest;
    addTypes(value: EventType, index?: number): EventType;

    hasCollection(): boolean;
    clearCollection(): void;
    getCollection(): string | undefined;
    setCollection(value: string): EventSubscriptionRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): EventSubscriptionRequest.AsObject;
    static toObject(includeInstance: boolean, msg: EventSubscriptionRequest): EventSubscriptionRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: EventSubscriptionRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): EventSubscriptionRequest;
    static deserializeBinaryFromReader(message: EventSubscriptionRequest, reader: jspb.BinaryReader): EventSubscriptionRequest;
}

export namespace EventSubscriptionRequest {
    export type AsObject = {
        typesList: Array<EventType>,
        collection?: string,
    }
}

export class VectorInsertedEvent extends jspb.Message { 
    getId(): number;
    setId(value: number): VectorInsertedEvent;
    getCollection(): string;
    setCollection(value: string): VectorInsertedEvent;
    getLogicalClock(): number;
    setLogicalClock(value: number): VectorInsertedEvent;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): VectorInsertedEvent;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;

    getTypedMetadataMap(): jspb.Map<string, MetadataValue>;
    clearTypedMetadataMap(): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VectorInsertedEvent.AsObject;
    static toObject(includeInstance: boolean, msg: VectorInsertedEvent): VectorInsertedEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VectorInsertedEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VectorInsertedEvent;
    static deserializeBinaryFromReader(message: VectorInsertedEvent, reader: jspb.BinaryReader): VectorInsertedEvent;
}

export namespace VectorInsertedEvent {
    export type AsObject = {
        id: number,
        collection: string,
        logicalClock: number,
        originNodeId: string,

        metadataMap: Array<[string, string]>,

        typedMetadataMap: Array<[string, MetadataValue.AsObject]>,
    }
}

export class VectorDeletedEvent extends jspb.Message { 
    getId(): number;
    setId(value: number): VectorDeletedEvent;
    getCollection(): string;
    setCollection(value: string): VectorDeletedEvent;
    getLogicalClock(): number;
    setLogicalClock(value: number): VectorDeletedEvent;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): VectorDeletedEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VectorDeletedEvent.AsObject;
    static toObject(includeInstance: boolean, msg: VectorDeletedEvent): VectorDeletedEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VectorDeletedEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VectorDeletedEvent;
    static deserializeBinaryFromReader(message: VectorDeletedEvent, reader: jspb.BinaryReader): VectorDeletedEvent;
}

export namespace VectorDeletedEvent {
    export type AsObject = {
        id: number,
        collection: string,
        logicalClock: number,
        originNodeId: string,
    }
}

export class EventMessage extends jspb.Message { 
    getType(): EventType;
    setType(value: EventType): EventMessage;

    hasVectorInserted(): boolean;
    clearVectorInserted(): void;
    getVectorInserted(): VectorInsertedEvent | undefined;
    setVectorInserted(value?: VectorInsertedEvent): EventMessage;

    hasVectorDeleted(): boolean;
    clearVectorDeleted(): void;
    getVectorDeleted(): VectorDeletedEvent | undefined;
    setVectorDeleted(value?: VectorDeletedEvent): EventMessage;

    getPayloadCase(): EventMessage.PayloadCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): EventMessage.AsObject;
    static toObject(includeInstance: boolean, msg: EventMessage): EventMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: EventMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): EventMessage;
    static deserializeBinaryFromReader(message: EventMessage, reader: jspb.BinaryReader): EventMessage;
}

export namespace EventMessage {
    export type AsObject = {
        type: EventType,
        vectorInserted?: VectorInsertedEvent.AsObject,
        vectorDeleted?: VectorDeletedEvent.AsObject,
    }

    export enum PayloadCase {
        PAYLOAD_NOT_SET = 0,
        VECTOR_INSERTED = 2,
        VECTOR_DELETED = 3,
    }

}

export class Empty extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Empty.AsObject;
    static toObject(includeInstance: boolean, msg: Empty): Empty.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Empty, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Empty;
    static deserializeBinaryFromReader(message: Empty, reader: jspb.BinaryReader): Empty;
}

export namespace Empty {
    export type AsObject = {
    }
}

export class StatusResponse extends jspb.Message { 
    getStatus(): string;
    setStatus(value: string): StatusResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): StatusResponse.AsObject;
    static toObject(includeInstance: boolean, msg: StatusResponse): StatusResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: StatusResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): StatusResponse;
    static deserializeBinaryFromReader(message: StatusResponse, reader: jspb.BinaryReader): StatusResponse;
}

export namespace StatusResponse {
    export type AsObject = {
        status: string,
    }
}

export class MonitorRequest extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): MonitorRequest.AsObject;
    static toObject(includeInstance: boolean, msg: MonitorRequest): MonitorRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: MonitorRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): MonitorRequest;
    static deserializeBinaryFromReader(message: MonitorRequest, reader: jspb.BinaryReader): MonitorRequest;
}

export namespace MonitorRequest {
    export type AsObject = {
    }
}

export class SystemStats extends jspb.Message { 
    getTotalCollections(): number;
    setTotalCollections(value: number): SystemStats;
    getTotalVectors(): number;
    setTotalVectors(value: number): SystemStats;
    getTotalMemoryMb(): number;
    setTotalMemoryMb(value: number): SystemStats;
    getQps(): number;
    setQps(value: number): SystemStats;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SystemStats.AsObject;
    static toObject(includeInstance: boolean, msg: SystemStats): SystemStats.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SystemStats, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SystemStats;
    static deserializeBinaryFromReader(message: SystemStats, reader: jspb.BinaryReader): SystemStats;
}

export namespace SystemStats {
    export type AsObject = {
        totalCollections: number,
        totalVectors: number,
        totalMemoryMb: number,
        qps: number,
    }
}

export class DigestRequest extends jspb.Message { 
    getCollection(): string;
    setCollection(value: string): DigestRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DigestRequest.AsObject;
    static toObject(includeInstance: boolean, msg: DigestRequest): DigestRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DigestRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DigestRequest;
    static deserializeBinaryFromReader(message: DigestRequest, reader: jspb.BinaryReader): DigestRequest;
}

export namespace DigestRequest {
    export type AsObject = {
        collection: string,
    }
}

export class DigestResponse extends jspb.Message { 
    getLogicalClock(): number;
    setLogicalClock(value: number): DigestResponse;
    getStateHash(): number;
    setStateHash(value: number): DigestResponse;
    clearBucketsList(): void;
    getBucketsList(): Array<number>;
    setBucketsList(value: Array<number>): DigestResponse;
    addBuckets(value: number, index?: number): number;
    getCount(): number;
    setCount(value: number): DigestResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DigestResponse.AsObject;
    static toObject(includeInstance: boolean, msg: DigestResponse): DigestResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DigestResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DigestResponse;
    static deserializeBinaryFromReader(message: DigestResponse, reader: jspb.BinaryReader): DigestResponse;
}

export namespace DigestResponse {
    export type AsObject = {
        logicalClock: number,
        stateHash: number,
        bucketsList: Array<number>,
        count: number,
    }
}

export enum QuantizationMode {
    NONE = 0,
    SCALAR_I8 = 1,
}

export enum DurabilityLevel {
    DEFAULT_LEVEL = 0,
    ASYNC = 1,
    BATCH = 2,
    STRICT = 3,
}

export enum EventType {
    EVENT_UNKNOWN = 0,
    VECTOR_INSERTED = 1,
    VECTOR_DELETED = 2,
}
