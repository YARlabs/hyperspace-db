import * as grpc from '@grpc/grpc-js';
import { DatabaseClient } from './proto/hyperspace_grpc_pb';
import {
    BatchSearchRequest,
    InsertRequest, SearchRequest,
    CreateCollectionRequest, DeleteCollectionRequest, Empty,
    DurabilityLevel,
    BatchInsertRequest, // New
    VectorData // New
} from './proto/hyperspace_pb';
import * as hyperspace_pb from './proto/hyperspace_pb'; // New, for direct access to types

export { DurabilityLevel };

export interface Filter {
    match?: { key: string, value: string };
    range?: { key: string, gte?: number, lte?: number };
}

export interface SearchResult {
    id: number;
    distance: number;
    metadata: { [key: string]: string };
}

export class HyperspaceClient {
    private client: DatabaseClient;
    private metadata: grpc.Metadata;
    private static toVectorList(vector: number[] | Float32Array | Float64Array): number[] {
        if (Array.isArray(vector)) {
            return vector;
        }
        return Array.from(vector);
    }

    constructor(host: string = 'localhost:50051', apiKey?: string, userId?: string) {
        const options = {
            'grpc.max_send_message_length': 64 * 1024 * 1024,
            'grpc.max_receive_message_length': 64 * 1024 * 1024,
            'grpc.keepalive_time_ms': 10000,
            'grpc.keepalive_timeout_ms': 5000,
            'grpc.keepalive_permit_without_calls': 1,
            'grpc.http2.min_time_between_pings_ms': 10000,
            'grpc.http2.min_ping_interval_without_data_ms': 5000,
        };
        this.client = new DatabaseClient(host, grpc.credentials.createInsecure(), options);
        this.metadata = new grpc.Metadata();
        if (apiKey) {
            this.metadata.add('x-api-key', apiKey);
        }
        if (userId) {
            this.metadata.add('x-hyperspace-user-id', userId);
        }
    }

    // ... (create/delete unchanged) ...

    public createCollection(name: string, dimension: number, metric: string): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new CreateCollectionRequest();
            req.setName(name);
            req.setDimension(dimension);
            req.setMetric(metric);

            this.client.createCollection(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(true);
            });
        });
    }

    public deleteCollection(name: string): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new DeleteCollectionRequest();
            req.setName(name);

            this.client.deleteCollection(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(true);
            });
        });
    }

    public insert(id: number, vector: number[] | Float32Array | Float64Array, meta?: { [key: string]: string }, collection: string = '', durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new InsertRequest();
            req.setId(id);
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            if (meta) {
                const map = req.getMetadataMap();
                for (const k in meta) map.set(k, meta[k]);
            }
            req.setCollection(collection);
            req.setOriginNodeId('');
            req.setLogicalClock(0);
            req.setDurability(durability);

            this.client.insert(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public batchInsert(items: { id: number, vector: number[] | Float32Array | Float64Array, metadata?: { [key: string]: string } }[], collection: string = '', durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new BatchInsertRequest();
            req.setCollection(collection);
            req.setDurability(durability);

            const vectors = items.map(item => {
                const v = new VectorData();
                v.setId(item.id);
                v.setVectorList(HyperspaceClient.toVectorList(item.vector));
                if (item.metadata) {
                    const map = v.getMetadataMap();
                    for (const k in item.metadata) map.set(k, item.metadata[k]);
                }
                return v;
            });
            req.setVectorsList(vectors);

            this.client.batchInsert(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public search(
        vector: number[] | Float32Array | Float64Array,
        topK: number,
        collection: string = '',
        options?: {
            filters?: Filter[],
            hybridQuery?: string,
            hybridAlpha?: number
        }
    ): Promise<SearchResult[]> {
        return new Promise((resolve, reject) => {
            const req = new SearchRequest();
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            req.setTopK(topK);
            req.setCollection(collection);

            if (options?.filters) {
                const protoFilters = options.filters.map(f => {
                    const pf = new hyperspace_pb.Filter();
                    if (f.match) {
                        const m = new hyperspace_pb.Match();
                        m.setKey(f.match.key);
                        m.setValue(f.match.value);
                        pf.setMatch(m);
                    } else if (f.range) {
                        const r = new hyperspace_pb.Range();
                        r.setKey(f.range.key);
                        if (f.range.gte !== undefined) r.setGte(f.range.gte);
                        if (f.range.lte !== undefined) r.setLte(f.range.lte);
                        pf.setRange(r);
                    }
                    return pf;
                });
                req.setFiltersList(protoFilters);
            }

            if (options?.hybridQuery) req.setHybridQuery(options.hybridQuery);
            if (options?.hybridAlpha !== undefined) req.setHybridAlpha(options.hybridAlpha);

            this.client.search(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const results = resp.getResultsList().map(r => {
                    const metaMap = r.getMetadataMap();
                    const meta: { [key: string]: string } = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry: string, key: string) => {
                            meta[key] = entry;
                        });
                    }
                    return {
                        id: r.getId(),
                        distance: r.getDistance(),
                        metadata: meta
                    };
                });
                resolve(results);
            });
        });
    }

    public searchBatch(vectors: Array<number[] | Float32Array | Float64Array>, topK: number, collection: string = ''): Promise<SearchResult[][]> {
        return new Promise((resolve, reject) => {
            const req = new BatchSearchRequest();
            req.setSearchesList(
                vectors.map((vector) => {
                    const s = new SearchRequest();
                    s.setVectorList(HyperspaceClient.toVectorList(vector));
                    s.setTopK(topK);
                    s.setCollection(collection);
                    return s;
                })
            );

            this.client.searchBatch(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const batch = resp.getResponsesList().map((searchResp) =>
                    searchResp.getResultsList().map((r) => {
                        const metaMap = r.getMetadataMap();
                        const meta: { [key: string]: string } = {};
                        if (metaMap.getLength() > 0) {
                            metaMap.forEach((entry: string, key: string) => {
                                meta[key] = entry;
                            });
                        }
                        return {
                            id: r.getId(),
                            distance: r.getDistance(),
                            metadata: meta
                        };
                    })
                );
                resolve(batch);
            });
        });
    }

    public getDigest(collection: string = ''): Promise<{ logicalClock: number, stateHash: number, count: number }> {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.DigestRequest();
            req.setCollection(collection);

            this.client.getDigest(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve({
                    logicalClock: resp.getLogicalClock(),
                    stateHash: resp.getStateHash(),
                    count: resp.getCount()
                });
            });
        });
    }

    public close() {
        this.client.close();
    }
}
