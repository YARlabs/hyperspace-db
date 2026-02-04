// package: hyperspace
// file: hyperspace.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";

export class ReplicationLog extends jspb.Message { 
    getId(): number;
    setId(value: number): ReplicationLog;
    clearVectorList(): void;
    getVectorList(): Array<number>;
    setVectorList(value: Array<number>): ReplicationLog;
    addVector(value: number, index?: number): number;

    getMetadataMap(): jspb.Map<string, string>;
    clearMetadataMap(): void;
    getCollection(): string;
    setCollection(value: string): ReplicationLog;
    getOriginNodeId(): string;
    setOriginNodeId(value: string): ReplicationLog;
    getLogicalClock(): number;
    setLogicalClock(value: number): ReplicationLog;

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
        id: number,
        vectorList: Array<number>,

        metadataMap: Array<[string, string]>,
        collection: string,
        originNodeId: string,
        logicalClock: number,
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

export class SearchResult extends jspb.Message { 
    getId(): number;
    setId(value: number): SearchResult;
    getDistance(): number;
    setDistance(value: number): SearchResult;

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

export enum QuantizationMode {
    NONE = 0,
    SCALAR_I8 = 1,
}
