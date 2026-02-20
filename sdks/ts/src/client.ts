import * as grpc from '@grpc/grpc-js';
import { DatabaseClient } from './proto/hyperspace_grpc_pb';
import {
    BatchSearchRequest,
    InsertRequest, SearchRequest,
    CreateCollectionRequest, DeleteCollectionRequest, Empty,
    DurabilityLevel,
    BatchInsertRequest, // New
    VectorData,
    GetNodeRequest,
    GetNeighborsRequest,
    TraverseRequest,
    FindSemanticClustersRequest,
    GetConceptParentsRequest,
    RebuildIndexRequest,
    VacuumFilterQuery,
    EventSubscriptionRequest,
    EventType,
    EventMessage
} from './proto/hyperspace_pb';
import * as hyperspace_pb from './proto/hyperspace_pb'; // New, for direct access to types

export { DurabilityLevel };
export type TypedMetadataValue = string | number | boolean;

export interface Filter {
    match?: { key: string, value: string };
    range?: { key: string, gte?: number, lte?: number };
}

export interface SearchResult {
    id: number;
    distance: number;
    metadata: { [key: string]: string };
    typedMetadata: { [key: string]: TypedMetadataValue };
}

export interface GraphNode {
    id: number;
    layer: number;
    neighbors: number[];
    metadata: { [key: string]: string };
    typedMetadata: { [key: string]: TypedMetadataValue };
}

export interface VacuumFilter {
    key: string;
    op: 'lt' | 'lte' | 'gt' | 'gte' | 'eq' | 'ne';
    value: number;
}

export type EventTypeName = 'insert' | 'delete';
export interface SubscribeOptions {
    types?: EventTypeName[];
    collection?: string;
}

export const HyperbolicMath = {
    projectToBall(x: number[], c: number = 1.0): number[] {
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        const n = Math.sqrt(Math.max(normSq(x), 0));
        const maxN = (1 / Math.sqrt(c)) - 1e-9;
        if (n <= maxN || n <= 1e-15) return [...x];
        const scale = maxN / n;
        return x.map((v) => v * scale);
    },
    mobiusAdd(x: number[], y: number[], c: number = 1.0): number[] {
        if (x.length !== y.length) throw new Error('Dimension mismatch');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const dot = (a: number[], b: number[]) => a.reduce((s, v, i) => s + v * b[i], 0);
        const x2 = dot(x, x);
        const y2 = dot(y, y);
        const xy = dot(x, y);
        const left = 1 + 2 * c * xy + c * y2;
        const right = 1 - c * x2;
        const den = 1 + 2 * c * xy + c * c * x2 * y2;
        if (Math.abs(den) < 1e-15) throw new Error('Mobius denominator too small');
        return x.map((xi, i) => (left * xi + right * y[i]) / den);
    },
    expMap(x: number[], v: number[], c: number = 1.0): number[] {
        if (x.length !== v.length) throw new Error('Dimension mismatch');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        const vNorm = Math.sqrt(Math.max(normSq(v), 0));
        if (vNorm < 1e-15) return [...x];
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const scale = Math.tanh(Math.sqrt(c) * lambdaX * vNorm / 2) / (Math.sqrt(c) * vNorm);
        return HyperbolicMath.mobiusAdd(x, v.map((vi) => scale * vi), c);
    },
    logMap(x: number[], y: number[], c: number = 1.0): number[] {
        if (x.length !== y.length) throw new Error('Dimension mismatch');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        const delta = HyperbolicMath.mobiusAdd(x.map((xi) => -xi), y, c);
        const deltaNorm = Math.sqrt(Math.max(normSq(delta), 0));
        if (deltaNorm < 1e-15) return new Array(x.length).fill(0);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const arg = Math.min(Math.sqrt(c) * deltaNorm, 1 - 1e-15);
        const factor = (2 / (lambdaX * Math.sqrt(c))) * Math.atanh(arg);
        return delta.map((di) => factor * di / deltaNorm);
    },
    riemannianGradient(x: number[], euclideanGrad: number[], c: number = 1.0): number[] {
        if (x.length !== euclideanGrad.length) throw new Error('Dimension mismatch');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const scale = 1 / (lambdaX * lambdaX);
        return euclideanGrad.map((g) => scale * g);
    },
    parallelTransport(x: number[], y: number[], v: number[], c: number = 1.0): number[] {
        if (x.length !== y.length || x.length !== v.length) throw new Error('Dimension mismatch');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        const gyro = (u: number[], w: number[], z: number[]): number[] => {
            const uw = HyperbolicMath.mobiusAdd(u, w, c);
            const wz = HyperbolicMath.mobiusAdd(w, z, c);
            const left = HyperbolicMath.mobiusAdd(u, wz, c);
            return HyperbolicMath.mobiusAdd(uw.map((k) => -k), left, c);
        };
        const g = gyro(y, x.map((xi) => -xi), v);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const lambdaY = 2 / Math.max(1 - c * normSq(y), 1e-15);
        const scale = lambdaX / lambdaY;
        return g.map((gi) => scale * gi);
    },
    frechetMean(points: number[][], c: number = 1.0, maxIter: number = 64, tol: number = 1e-8): number[] {
        if (!points.length) throw new Error('Points set cannot be empty');
        if (c <= 0) throw new Error('Curvature c must be > 0');
        const dim = points[0].length;
        if (points.some((p) => p.length !== dim)) throw new Error('Dimension mismatch');
        const normSq = (u: number[]) => u.reduce((s, z) => s + z * z, 0);
        let mu = HyperbolicMath.projectToBall(points[0], c);
        for (let iter = 0; iter < Math.max(1, maxIter); iter++) {
            const grad = new Array(dim).fill(0);
            for (const p of points) {
                const lg = HyperbolicMath.logMap(mu, p, c);
                for (let i = 0; i < dim; i++) grad[i] += lg[i];
            }
            for (let i = 0; i < dim; i++) grad[i] /= points.length;
            const gNorm = Math.sqrt(Math.max(normSq(grad), 0));
            if (gNorm <= Math.max(tol, 1e-15)) break;
            mu = HyperbolicMath.expMap(mu, grad, c);
            mu = HyperbolicMath.projectToBall(mu, c);
        }
        return mu;
    }
};

export class HyperspaceClient {
    private client: DatabaseClient;
    private metadata: grpc.Metadata;
    private static toVectorList(vector: number[] | Float32Array | Float64Array): number[] {
        if (Array.isArray(vector)) {
            return vector;
        }
        return Array.from(vector);
    }
    private static toProtoMetadataValue(value: TypedMetadataValue): hyperspace_pb.MetadataValue {
        const out = new hyperspace_pb.MetadataValue();
        if (typeof value === 'string') out.setStringValue(value);
        else if (typeof value === 'boolean') out.setBoolValue(value);
        else if (Number.isInteger(value)) out.setIntValue(Number(value));
        else out.setDoubleValue(Number(value));
        return out;
    }
    private static parseTypedMetadata(metaMap: any): { [key: string]: TypedMetadataValue } {
        const out: { [key: string]: TypedMetadataValue } = {};
        if (metaMap.getLength() === 0) return out;
        metaMap.forEach((value: hyperspace_pb.MetadataValue, key: string) => {
            switch (value.getKindCase()) {
                case hyperspace_pb.MetadataValue.KindCase.STRING_VALUE:
                    out[key] = value.getStringValue();
                    break;
                case hyperspace_pb.MetadataValue.KindCase.INT_VALUE:
                    out[key] = value.getIntValue();
                    break;
                case hyperspace_pb.MetadataValue.KindCase.DOUBLE_VALUE:
                    out[key] = value.getDoubleValue();
                    break;
                case hyperspace_pb.MetadataValue.KindCase.BOOL_VALUE:
                    out[key] = value.getBoolValue();
                    break;
                default:
                    break;
            }
        });
        return out;
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

    public insert(
        id: number,
        vector: number[] | Float32Array | Float64Array,
        meta?: { [key: string]: string },
        collection: string = '',
        durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL,
        typedMetadata?: { [key: string]: TypedMetadataValue }
    ): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new InsertRequest();
            req.setId(id);
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            if (meta) {
                const map = req.getMetadataMap();
                for (const k in meta) map.set(k, meta[k]);
            }
            if (typedMetadata) {
                const map = req.getTypedMetadataMap();
                for (const k in typedMetadata) map.set(k, HyperspaceClient.toProtoMetadataValue(typedMetadata[k]));
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

    public batchInsert(
        items: {
            id: number,
            vector: number[] | Float32Array | Float64Array,
            metadata?: { [key: string]: string },
            typedMetadata?: { [key: string]: TypedMetadataValue }
        }[],
        collection: string = '',
        durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL
    ): Promise<boolean> {
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
                if (item.typedMetadata) {
                    const map = v.getTypedMetadataMap();
                    for (const k in item.typedMetadata) map.set(k, HyperspaceClient.toProtoMetadataValue(item.typedMetadata[k]));
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
                        if (f.range.gte !== undefined) {
                            if (Number.isInteger(f.range.gte)) r.setGte(f.range.gte);
                            else r.setGteF64(f.range.gte);
                        }
                        if (f.range.lte !== undefined) {
                            if (Number.isInteger(f.range.lte)) r.setLte(f.range.lte);
                            else r.setLteF64(f.range.lte);
                        }
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
                        metadata: meta,
                        typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap())
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
                            metadata: meta,
                            typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap())
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

    public rebuildIndex(collection: string): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new RebuildIndexRequest();
            req.setName(collection);
            this.client.rebuildIndex(req, this.metadata, (err) => {
                if (err) return reject(err);
                resolve(true);
            });
        });
    }

    public rebuildIndexWithFilter(collection: string, filter: VacuumFilter): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new RebuildIndexRequest();
            req.setName(collection);
            const fq = new VacuumFilterQuery();
            fq.setKey(filter.key);
            fq.setOp(filter.op);
            fq.setValue(filter.value);
            req.setFilterQuery(fq);
            this.client.rebuildIndex(req, this.metadata, (err) => {
                if (err) return reject(err);
                resolve(true);
            });
        });
    }

    public getNode(id: number, layer: number = 0, collection: string = ''): Promise<GraphNode> {
        return new Promise((resolve, reject) => {
            const req = new GetNodeRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);

            this.client.getNode(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const metaMap = resp.getMetadataMap();
                const metadata: { [key: string]: string } = {};
                if (metaMap.getLength() > 0) {
                    metaMap.forEach((entry: string, key: string) => {
                        metadata[key] = entry;
                    });
                }
                resolve({
                    id: resp.getId(),
                    layer: resp.getLayer(),
                    neighbors: resp.getNeighborsList(),
                    metadata,
                    typedMetadata: HyperspaceClient.parseTypedMetadata(resp.getTypedMetadataMap())
                });
            });
        });
    }

    public getNeighbors(id: number, layer: number = 0, limit: number = 64, offset: number = 0, collection: string = ''): Promise<GraphNode[]> {
        return new Promise((resolve, reject) => {
            const req = new GetNeighborsRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);
            req.setLimit(limit);
            req.setOffset(offset);

            this.client.getNeighbors(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const nodes = resp.getNeighborsList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata: { [key: string]: string } = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry: string, key: string) => {
                            metadata[key] = entry;
                        });
                    }
                    return {
                        id: n.getId(),
                        layer: n.getLayer(),
                        neighbors: n.getNeighborsList(),
                        metadata,
                        typedMetadata: HyperspaceClient.parseTypedMetadata(n.getTypedMetadataMap())
                    };
                });
                resolve(nodes);
            });
        });
    }

    public getConceptParents(id: number, layer: number = 0, limit: number = 32, collection: string = ''): Promise<GraphNode[]> {
        return new Promise((resolve, reject) => {
            const req = new GetConceptParentsRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);
            req.setLimit(limit);

            this.client.getConceptParents(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const nodes = resp.getParentsList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata: { [key: string]: string } = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry: string, key: string) => {
                            metadata[key] = entry;
                        });
                    }
                    return {
                        id: n.getId(),
                        layer: n.getLayer(),
                        neighbors: n.getNeighborsList(),
                        metadata,
                        typedMetadata: HyperspaceClient.parseTypedMetadata(n.getTypedMetadataMap())
                    };
                });
                resolve(nodes);
            });
        });
    }

    public traverse(
        startId: number,
        layer: number = 0,
        maxDepth: number = 2,
        maxNodes: number = 256,
        collection: string = '',
        options?: { filter?: { [key: string]: string }, filters?: Filter[] }
    ): Promise<GraphNode[]> {
        return new Promise((resolve, reject) => {
            const req = new TraverseRequest();
            req.setCollection(collection);
            req.setStartId(startId);
            req.setLayer(layer);
            req.setMaxDepth(maxDepth);
            req.setMaxNodes(maxNodes);
            if (options?.filter) {
                const map = req.getFilterMap();
                for (const k in options.filter) {
                    map.set(k, options.filter[k]);
                }
            }
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
                        if (f.range.gte !== undefined) {
                            if (Number.isInteger(f.range.gte)) r.setGte(f.range.gte);
                            else r.setGteF64(f.range.gte);
                        }
                        if (f.range.lte !== undefined) {
                            if (Number.isInteger(f.range.lte)) r.setLte(f.range.lte);
                            else r.setLteF64(f.range.lte);
                        }
                        pf.setRange(r);
                    }
                    return pf;
                });
                req.setFiltersList(protoFilters);
            }

            this.client.traverse(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const nodes = resp.getNodesList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata: { [key: string]: string } = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry: string, key: string) => {
                            metadata[key] = entry;
                        });
                    }
                    return {
                        id: n.getId(),
                        layer: n.getLayer(),
                        neighbors: n.getNeighborsList(),
                        metadata,
                        typedMetadata: HyperspaceClient.parseTypedMetadata(n.getTypedMetadataMap())
                    };
                });
                resolve(nodes);
            });
        });
    }

    public findSemanticClusters(layer: number = 0, minClusterSize: number = 3, maxClusters: number = 32, maxNodes: number = 10000, collection: string = ''): Promise<number[][]> {
        return new Promise((resolve, reject) => {
            const req = new FindSemanticClustersRequest();
            req.setCollection(collection);
            req.setLayer(layer);
            req.setMinClusterSize(minClusterSize);
            req.setMaxClusters(maxClusters);
            req.setMaxNodes(maxNodes);

            this.client.findSemanticClusters(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getClustersList().map((c) => c.getNodeIdsList()));
            });
        });
    }

    public subscribeToEvents(
        options: SubscribeOptions,
        onEvent: (event: EventMessage) => void,
        onError?: (err: Error) => void
    ): grpc.ClientReadableStream<EventMessage> {
        const req = new EventSubscriptionRequest();
        if (options.collection) {
            req.setCollection(options.collection);
        }
        const requested = options.types || [];
        if (requested.length > 0) {
            const mapped = requested.map((t) => t === 'insert' ? EventType.VECTOR_INSERTED : EventType.VECTOR_DELETED);
            req.setTypesList(mapped);
        }
        const stream = this.client.subscribeToEvents(req, this.metadata);
        stream.on('data', onEvent);
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }

    public close() {
        this.client.close();
    }
}
