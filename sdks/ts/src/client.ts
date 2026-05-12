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
    EventMessage,
    VectorizeRequest,
    InsertTextRequest,
    SearchTextRequest,
    CollectionSummary as ProtoCollectionSummary,
    CollectionStatsRequest,
    ConfigUpdate,
    QuantizationConfig,
    QuantizationMode,
    SearchMultiCollectionRequest,
    MonitorRequest,
    GetPointsRequest,
    UpdatePayloadRequest,
    ScrollRequest,
    CountRequest,
    HealthCheckResponse,
    Bm25Options as ProtoBm25Options,
    ReconsolidationRequest
} from './proto/hyperspace_pb';

import * as hyperspace_pb from './proto/hyperspace_pb'; // New, for direct access to types

export * as CognitiveMath from './math';
export { TribunalContext } from './agents';
export { DurabilityLevel };
export type TypedMetadataValue = string | number | boolean;

export interface Filter {
    match?: { key: string, value: string };
    range?: { key: string, gte?: number, lte?: number };
    inCone?: { axes: number[], apertures: number[], cen: number[] };
    inBall?: { center: number[], radius: number };
    inBox?: { minBounds: number[], maxBounds: number[] };
    and?: Filter[];
    or?: Filter[];
    not?: Filter;
}

export interface SearchResult {
    id: number;
    distance: number;
    metadata: { [key: string]: string };
    typedMetadata: { [key: string]: TypedMetadataValue };
    payload?: Uint8Array; // Sidecar Payload Storage (v3.2)
}

export interface CollectionInfo {
    name: string;
    count: number;
    dimension: number;
    metric: string;
}

export interface CollectionStats {
    count: number;
    dimension: number;
    metric: string;
    indexingQueue: number;
    diskUsageBytes: number;
    ramUsageBytes: number;
    activeTasks: number;
}

export interface SystemMetrics {
    cpuUsage: number;
    ramUsageMb: number;
    totalVectors: number;
    activeCollections: number;
    uptimeSeconds: number;
    networkRxBytes: number;
    networkTxBytes: number;
}

export interface CollectionConfig {
    quantizationMode?: 'NONE' | 'SCALAR_I8';
    efSearch?: number;
    efConstruction?: number;
    m?: number;
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

export interface Bm25Options {
    method?: "bm25plus" | "bm25l" | "robertson" | "lucene" | "atire";
    k1?: number;
    b?: number;
    delta?: number;
    language?: string;
    ngrams?: number;
    fusionMethod?: "rrf" | "weighted";
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
    },
    analyzeDeltaHyperbolicity(vectors: number[][], numSamples: number = 1000): { delta: number, recommendation: string } {
        if (vectors.length < 4) return { delta: 0, recommendation: 'euclidean' };
        
        const l2Dist = (a: number[], b: number[]) => Math.sqrt(a.reduce((s, x, i) => s + (x - b[i]) ** 2, 0));
        let maxDelta = 0;

        for (let s = 0; s < numSamples; s++) {
            const indices = new Set<number>();
            while (indices.size < 4) indices.add(Math.floor(Math.random() * vectors.length));
            const [i, j, k, l] = Array.from(indices);

            const d_ij = l2Dist(vectors[i], vectors[j]);
            const d_kl = l2Dist(vectors[k], vectors[l]);
            const d_ik = l2Dist(vectors[i], vectors[k]);
            const d_jl = l2Dist(vectors[j], vectors[l]);
            const d_il = l2Dist(vectors[i], vectors[l]);
            const d_jk = l2Dist(vectors[j], vectors[k]);

            const s1 = d_ij + d_kl;
            const s2 = d_ik + d_jl;
            const s3 = d_il + d_jk;

            const sums = [s1, s2, s3].sort((a, b) => b - a);
            const delta = (sums[0] - sums[1]) / 2.0;
            if (delta > maxDelta) maxDelta = delta;
        }

        const recommendation = maxDelta < 0.15 ? 'lorentz' : (maxDelta < 0.30 ? 'poincare' : 'l2');
        return { delta: maxDelta, recommendation };
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

    private toProtoFilter(f: Filter): hyperspace_pb.Filter {
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
        } else if (f.inCone) {
            const c = new hyperspace_pb.InCone();
            c.setAxesList(f.inCone.axes);
            c.setAperturesList(f.inCone.apertures);
            c.setCen(f.inCone.cen[0] || 0);
            pf.setInCone(c);
        } else if (f.inBall) {
            const b = new hyperspace_pb.InBall();
            b.setCenterList(f.inBall.center);
            b.setRadius(f.inBall.radius);
            pf.setInBall(b);
        } else if (f.inBox) {
            const b = new hyperspace_pb.InBox();
            b.setMinBoundsList(f.inBox.minBounds);
            b.setMaxBoundsList(f.inBox.maxBounds);
            pf.setInBox(b);
        } else if (f.and) {
            const andOp = new hyperspace_pb.FilterAnd();
            andOp.setConditionsList(f.and.map(cond => this.toProtoFilter(cond)));
            pf.setAndOp(andOp);
        } else if (f.or) {
            const orOp = new hyperspace_pb.FilterOr();
            orOp.setConditionsList(f.or.map(cond => this.toProtoFilter(cond)));
            pf.setOrOp(orOp);
        } else if (f.not) {
            const notOp = new hyperspace_pb.FilterNot();
            notOp.setCondition(this.toProtoFilter(f.not));
            pf.setNotOp(notOp);
        }
        return pf;
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

    public listCollections(): Promise<CollectionInfo[]> {
        return new Promise((resolve, reject) => {
            const req = new Empty();

            this.client.listCollections(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const list = (resp.getCollectionsList() as any as ProtoCollectionSummary[]).map(c => ({
                    name: c.getName(),
                    count: c.getCount(),
                    dimension: c.getDimension(),
                    metric: c.getMetric()
                }));
                resolve(list);
            });
        });
    }

    public delete(id: number, collection: string = ''): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.DeleteRequest();
            req.setCollection(collection);
            req.setId(id);

            this.client.delete(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public insert(
        id: number,
        vector: number[] | Float32Array | Float64Array,
        meta?: { [key: string]: string },
        collection: string = '',
        durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL,
        typedMetadata?: { [key: string]: TypedMetadataValue },
        payload?: Uint8Array // Sidecar Payload Storage (v3.2)
    ): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new InsertRequest();
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            req.setId(id);
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
            if (payload) {
                req.setPayload(payload);
            }

            this.client.insert(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public insertText(
        id: number,
        text: string,
        meta?: { [key: string]: string },
        collection: string = '',
        durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL
    ): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new InsertTextRequest();
            req.setText(text);
            req.setId(id);
            if (meta) {
                const map = req.getMetadataMap();
                for (const k in meta) map.set(k, meta[k]);
            }
            req.setCollection(collection);
            req.setDurability(durability);

            this.client.insertText(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public vectorize(text: string, metric: string = 'l2'): Promise<number[]> {
        return new Promise((resolve, reject) => {
            const req = new VectorizeRequest();
            req.setText(text);
            req.setMetric(metric);

            this.client.vectorize(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getVectorList());
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
            hybridAlpha?: number,
            bm25?: Bm25Options,
            mrlDimension?: number,
            useWasserstein?: boolean,
            includePayload?: boolean
        }
    ): Promise<SearchResult[]> {
        return new Promise((resolve, reject) => {
            const req = new SearchRequest();
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            req.setTopK(topK);
            req.setCollection(collection);

            if (options?.filters) {
                req.setFiltersList(options.filters.map(f => this.toProtoFilter(f)));
            }

            if (options?.hybridQuery) req.setHybridQuery(options.hybridQuery);
            if (options?.hybridAlpha !== undefined) req.setHybridAlpha(options.hybridAlpha);
            if (options?.mrlDimension !== undefined) req.setMrlDimension(options.mrlDimension);
            if (options?.useWasserstein !== undefined) req.setUseWasserstein(options.useWasserstein);
            if (options?.includePayload !== undefined) req.setIncludePayload(options.includePayload);
            
            if (options?.bm25) {
                const bm25Msg = new ProtoBm25Options();
                if (options.bm25.method !== undefined) bm25Msg.setMethod(options.bm25.method);
                if (options.bm25.k1 !== undefined) bm25Msg.setK1(options.bm25.k1);
                if (options.bm25.b !== undefined) bm25Msg.setB(options.bm25.b);
                if (options.bm25.delta !== undefined) bm25Msg.setDelta(options.bm25.delta);
                if (options.bm25.language !== undefined) bm25Msg.setLanguage(options.bm25.language);
                if (options.bm25.ngrams !== undefined) bm25Msg.setNgrams(options.bm25.ngrams);
                if (options.bm25.fusionMethod !== undefined) bm25Msg.setFusionMethod(options.bm25.fusionMethod);
                req.setBm25Options(bm25Msg);
            }

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
                        typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap()),
                        payload: r.getPayload_asU8()
                    };
                });
                resolve(results);
            });
        });
    }

    public searchText(
        text: string,
        topK: number,
        collection: string = '',
        options?: {
            filters?: Filter[],
            bm25?: Bm25Options,
            hybridAlpha?: number,
            includePayload?: boolean
        }
    ): Promise<SearchResult[]> {
        return new Promise((resolve, reject) => {
            const req = new SearchTextRequest();
            req.setText(text);
            req.setTopK(topK);
            req.setCollection(collection);

            if (options?.filters) {
                req.setFiltersList(options.filters.map(f => this.toProtoFilter(f)));
            }

            if (options?.bm25) {
                const bm25Msg = new ProtoBm25Options();
                if (options.bm25.method !== undefined) bm25Msg.setMethod(options.bm25.method);
                if (options.bm25.k1 !== undefined) bm25Msg.setK1(options.bm25.k1);
                if (options.bm25.b !== undefined) bm25Msg.setB(options.bm25.b);
                if (options.bm25.delta !== undefined) bm25Msg.setDelta(options.bm25.delta);
                if (options.bm25.language !== undefined) bm25Msg.setLanguage(options.bm25.language);
                if (options.bm25.ngrams !== undefined) bm25Msg.setNgrams(options.bm25.ngrams);
                if (options.bm25.fusionMethod !== undefined) bm25Msg.setFusionMethod(options.bm25.fusionMethod);
                req.setBm25Options(bm25Msg);
            }
            if (options?.hybridAlpha !== undefined) {
                req.setHybridAlpha(options.hybridAlpha);
            }
            if (options?.includePayload !== undefined) {
                req.setIncludePayload(options.includePayload);
            }

            this.client.searchText(req, this.metadata, (err, resp) => {
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
                        typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap()),
                        payload: r.getPayload_asU8()
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
                            typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap()),
                            payload: r.getPayload_asU8()
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
                req.setFiltersList(options.filters.map(f => this.toProtoFilter(f)));
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

    public syncHandshake(
        collection: string,
        clientBuckets: number[],
        clientLogicalClock: number = 0,
        clientCount: number = 0
    ): Promise<hyperspace_pb.SyncHandshakeResponse.AsObject> {
        return new Promise((resolve, reject) => {
            if (clientBuckets.length !== 256) {
                return reject(new Error("clientBuckets must contain exactly 256 elements"));
            }
            const req = new hyperspace_pb.SyncHandshakeRequest();
            req.setCollection(collection);
            req.setClientBucketsList(clientBuckets);
            req.setClientLogicalClock(clientLogicalClock);
            req.setClientCount(clientCount);

            this.client.syncHandshake(req, this.metadata, (err, res) => {
                if (err) return reject(err);
                resolve(res.toObject());
            });
        });
    }

    public syncPull(
        collection: string,
        bucketIndices: number[],
        onData: (data: hyperspace_pb.SyncVectorData.AsObject) => void,
        onError?: (err: Error) => void
    ): grpc.ClientReadableStream<hyperspace_pb.SyncVectorData> {
        const req = new hyperspace_pb.SyncPullRequest();
        req.setCollection(collection);
        req.setBucketIndicesList(bucketIndices);

        const stream = this.client.syncPull(req, this.metadata);
        stream.on('data', (data: hyperspace_pb.SyncVectorData) => {
            onData(data.toObject());
        });
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }

    public getCollectionStats(name: string): Promise<CollectionStats> {
        return new Promise((resolve, reject) => {
            const req = new CollectionStatsRequest();
            req.setName(name);

            this.client.getCollectionStats(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve({
                    count: resp.getCount(),
                    dimension: resp.getDimension(),
                    metric: resp.getMetric(),
                    indexingQueue: resp.getIndexingQueue(),
                    diskUsageBytes: resp.getDiskUsageBytes(),
                    ramUsageBytes: resp.getRamUsageBytes(),
                    activeTasks: resp.getActiveTasks()
                });
            });
        });
    }

    public async exists(name: string): Promise<boolean> {
        try {
            await this.getCollectionStats(name);
            return true;
        } catch (e: any) {
            if (e.code === grpc.status.NOT_FOUND || (e.message && e.message.includes('not found'))) {
                return false;
            }
            throw e;
        }
    }

    public updateCollection(name: string, config: CollectionConfig): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new ConfigUpdate();
            req.setCollection(name);
            if (config.efSearch !== undefined) req.setEfSearch(config.efSearch);
            if (config.efConstruction !== undefined) req.setEfConstruction(config.efConstruction);
            if (config.m !== undefined) req.setM(config.m);

            this.client.configure(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus() === 'success' || (!!resp.getStatus() && resp.getStatus().includes('updated')));
            });
        });
    }

    public createSnapshot(): Promise<boolean> {
        return new Promise((resolve, reject) => {
            this.client.triggerSnapshot(new Empty(), this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }

    public vacuum(): Promise<boolean> {
        return new Promise((resolve, reject) => {
            this.client.triggerVacuum(new Empty(), this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }

    public getMetrics(onData: (data: any) => void, onError?: (err: Error) => void): grpc.ClientReadableStream<any> {
        const req = new MonitorRequest();
        const stream = this.client.monitor(req, this.metadata);
        stream.on('data', onData);
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }

    public searchMultiCollection(collections: string[], query: number[]): Promise<any> {
        return new Promise((resolve, reject) => {
            const req = new SearchMultiCollectionRequest();
            req.setCollectionsList(collections);
            req.setVectorList(HyperspaceClient.toVectorList(query));
            req.setTopK(10);

            this.client.searchMultiCollection(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.toObject());
            });
        });
    }

    public triggerReconsolidation(collection: string, targetVector: number[], learningRate: number = 0.01): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new ReconsolidationRequest();
            req.setCollection(collection);
            req.setTargetVectorList(targetVector);
            req.setLearningRate(learningRate);

            this.client.triggerReconsolidation(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }


    public close() {
        this.client.close();
    }
}
