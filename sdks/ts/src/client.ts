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
import * as CognitiveMath from './math';

export * as CognitiveMathExport from './math';
export { TribunalContext } from './agents';
export { DurabilityLevel };
export type TypedMetadataValue = string | number | boolean;

export interface Filter {
    match?: { key: string, value: string };
    prefix?: { key: string, prefix: string };
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

export interface VectorComponent {
    name: string;
    metric: string;
    fullDimension: number;
    weight: number;
}

export interface MrlLayer {
    componentName: string;
    cutoffDimension: number;
    storeInRam: boolean;
    rerankTopK: number;
}

export interface CollectionSchema {
    components: VectorComponent[];
    cascadePipeline: MrlLayer[];
}

export interface CollectionInfo {
    name: string;
    count: number;
    schema?: CollectionSchema;
}

export interface CollectionStats {
    count: number;
    schema?: CollectionSchema;
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
    private host: string;
    private apiKey?: string;
    private userId?: string;
    public embedder?: { encode: (text: string) => Promise<number[]> | number[] };
    private collectionKeys: { [key: string]: string } = {};
    private encryptionContexts: { [key: string]: any } = {};
    private collectionMetrics: { [key: string]: string } = {};
    private collectionNoiseSigmas: { [key: string]: number } = {};
    private collectionSchemas: { [key: string]: CollectionSchema } = {};
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
        } else if (f.prefix) {
            const p = new hyperspace_pb.Prefix();
            p.setKey(f.prefix.key);
            p.setPrefix(f.prefix.prefix);
            pf.setPrefix(p);
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
        this.host = host;
        this.apiKey = apiKey;
        this.userId = userId;
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

    public registerCollectionKey(collectionName: string, key: string, metric: string = "l2", noiseSigma: number = 0.02, schema?: CollectionSchema) {
        this.collectionKeys[collectionName] = key;
        this.collectionMetrics[collectionName] = metric;
        this.collectionNoiseSigmas[collectionName] = noiseSigma;
        if (schema) {
            this.collectionSchemas[collectionName] = schema;
        }
        if (this.encryptionContexts[collectionName]) {
            delete this.encryptionContexts[collectionName];
        }
    }

    private deriveKeys(password: string, collectionName: string): { aesKey: Buffer, hmacKey: Buffer } {
        const crypto = require('crypto');
        const salt = crypto.createHash('sha256').update(collectionName).digest();
        const aesKey = crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
        const hmacKey = crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
        return { aesKey, hmacKey };
    }

    private encryptPayload(plaintext: Buffer, aesKey: Buffer): Buffer {
        const crypto = require('crypto');
        const iv = crypto.randomBytes(12);
        const pbkdf2Salt = crypto.randomBytes(16);
        const derivedAesKey = crypto.pbkdf2Sync(aesKey, pbkdf2Salt, 100000, 32, 'sha256');
        const cipherPayload = crypto.createCipheriv('aes-256-gcm', derivedAesKey, iv);
        const encryptedPayload = Buffer.concat([cipherPayload.update(plaintext), cipherPayload.final()]);
        const payloadTag = cipherPayload.getAuthTag();
        return Buffer.concat([pbkdf2Salt, iv, encryptedPayload, payloadTag]);
    }

    private decryptPayload(data: Buffer, aesKey: Buffer): Buffer {
        const crypto = require('crypto');
        if (data.length < 16 + 12 + 16) {
            throw new Error("Invalid encrypted payload size");
        }
        const pbkdf2Salt = data.subarray(0, 16);
        const iv = data.subarray(16, 28);
        const ciphertext = data.subarray(28, data.length - 16);
        const tag = data.subarray(data.length - 16);
        
        const derivedAesKey = crypto.pbkdf2Sync(aesKey, pbkdf2Salt, 100000, 32, 'sha256');
        const decipher = crypto.createDecipheriv('aes-256-gcm', derivedAesKey, iv);
        decipher.setAuthTag(tag);
        return Buffer.concat([decipher.update(ciphertext), decipher.final()]);
    }

    private hashMetadataKey(key: string, hmacKey: Buffer): string {
        const crypto = require('crypto');
        const hash = crypto.createHmac('sha256', hmacKey).update(key).digest('hex');
        return "tag_" + hash.slice(0, 16);
    }

    private hashMetadataValue(value: string, hmacKey: Buffer): string {
        const crypto = require('crypto');
        const hash = crypto.createHmac('sha256', hmacKey).update(value).digest('hex');
        return "val_" + hash;
    }

    private async _getEncryptionContext(collection: string, vectorDim?: number, metric: string = "l2"): Promise<any | null> {
        if (!collection) return null;
        const key = this.collectionKeys[collection];
        if (!key) return null;

        if (!this.collectionSchemas[collection]) {
            try {
                const stats = await this.getCollectionStats(collection);
                if (stats && stats.schema) {
                    this.collectionSchemas[collection] = stats.schema;
                }
            } catch (e) {}
        }

        if (!this.encryptionContexts[collection]) {
            const { aesKey, hmacKey } = this.deriveKeys(key, collection);
            this.encryptionContexts[collection] = {
                aesKey,
                hmacKey,
                projectionMatrices: {}
            };
        }

        const context = this.encryptionContexts[collection];

        if (vectorDim !== undefined) {
            if (!context.projectionMatrices[vectorDim]) {
                const isLorentz = ["lorentz", "poincare"].includes(metric.toLowerCase());
                const matrixDim = metric.toLowerCase() === "poincare" ? vectorDim + 1 : vectorDim;
                if (isLorentz) {
                    context.projectionMatrices[vectorDim] = CognitiveMath.generateLorentzMatrix(matrixDim, context.hmacKey);
                } else {
                    context.projectionMatrices[vectorDim] = CognitiveMath.generateOrthogonalMatrix(matrixDim, context.hmacKey);
                }
            }
        }

        return context;
    }

    private _projectSingleBlock(subVec: number[], metric: string, context: any, blockId?: string): number[] {
        const dim = subVec.length;
        if (dim === 0) return [];

        const cacheKey = blockId ? `${dim}_${blockId}` : `${dim}`;

        if (!context.projectionMatrices[cacheKey]) {
            const isLorentz = ["lorentz", "poincare"].includes(metric.toLowerCase());
            const matrixDim = metric.toLowerCase() === "poincare" ? dim + 1 : dim;

            const crypto = require('crypto');
            let seed = context.hmacKey;
            if (blockId) {
                seed = crypto.createHash('sha256').update(seed).update(blockId).digest();
            }

            if (isLorentz) {
                context.projectionMatrices[cacheKey] = CognitiveMath.generateLorentzMatrix(matrixDim, seed);
            } else {
                context.projectionMatrices[cacheKey] = CognitiveMath.generateOrthogonalMatrix(matrixDim, seed);
            }
        }

        const matrix = context.projectionMatrices[cacheKey];

        if (metric.toLowerCase() === "poincare") {
            const lorentzVec = CognitiveMath.poincareToLorentz(subVec);
            const projLorentz = CognitiveMath.projectVector(lorentzVec, matrix);
            return CognitiveMath.lorentzToPoincare(projLorentz);
        } else {
            return CognitiveMath.projectVector(subVec, matrix);
        }
    }

    private _projectCollectionVector(collection: string, vector: number[], context: any, metric: string = "l2"): number[] {
        const schema = this.collectionSchemas[collection];
        if (!schema || !schema.components || schema.components.length === 0) {
            return this._projectSingleBlock(vector, metric, context);
        }

        const components = schema.components;
        const cascade = schema.cascadePipeline || [];
        const componentCutoffs: { [key: string]: number[] } = {};

        for (const layer of cascade) {
            const compName = layer.componentName;
            const cutoff = layer.cutoffDimension;
            if (compName && cutoff) {
                if (!componentCutoffs[compName]) {
                    componentCutoffs[compName] = [];
                }
                componentCutoffs[compName].push(cutoff);
            }
        }

        for (const compName in componentCutoffs) {
            componentCutoffs[compName] = Array.from(new Set(componentCutoffs[compName])).sort((a, b) => a - b);
        }

        const projectedParts: number[] = [];
        let currentOffset = 0;

        for (const comp of components) {
            const compName = comp.name;
            const compMetric = comp.metric;
            const compDim = comp.fullDimension;

            if (currentOffset >= vector.length) {
                break;
            }

            let subVec = vector.slice(currentOffset, currentOffset + compDim);
            if (subVec.length < compDim) {
                subVec = subVec.concat(new Array(compDim - subVec.length).fill(0));
            }

            const cutoffs = componentCutoffs[compName] || [];
            const validCutoffs = cutoffs.filter(c => c < compDim);

            let projSub: number[] = [];
            if (validCutoffs.length === 0) {
                projSub = this._projectSingleBlock(subVec, compMetric, context);
            } else {
                let blockStart = 0;
                for (const cutoff of validCutoffs) {
                    const blockData = subVec.slice(blockStart, cutoff);
                    const projBlock = this._projectSingleBlock(blockData, compMetric, context, `${compName}_block_${blockStart}_${cutoff}`);
                    projSub = projSub.concat(projBlock);
                    blockStart = cutoff;
                }

                if (blockStart < compDim) {
                    const blockData = subVec.slice(blockStart, compDim);
                    const projBlock = this._projectSingleBlock(blockData, compMetric, context, `${compName}_block_${blockStart}_${compDim}`);
                    projSub = projSub.concat(projBlock);
                }
            }

            projectedParts.push(...projSub);
            currentOffset += compDim;
        }

        if (currentOffset < vector.length) {
            projectedParts.push(...vector.slice(currentOffset));
        }

        return projectedParts;
    }

    private _encryptFilters(filters: Filter[], context: any): Filter[] {
        if (!filters) return filters;
        return filters.map(f => {
            const nf = { ...f };
            if (nf.match) {
                nf.match = {
                    key: this.hashMetadataKey(nf.match.key, context.hmacKey),
                    value: this.hashMetadataValue(nf.match.value, context.hmacKey)
                };
            }
            if (nf.prefix) {
                nf.prefix = {
                    key: this.hashMetadataKey(nf.prefix.key, context.hmacKey),
                    prefix: this.hashMetadataValue(nf.prefix.prefix, context.hmacKey)
                };
            }
            if (nf.and) {
                nf.and = this._encryptFilters(nf.and, context);
            }
            if (nf.or) {
                nf.or = this._encryptFilters(nf.or, context);
            }
            if (nf.not) {
                nf.not = this._encryptFilters([nf.not], context)[0];
            }
            return nf;
        });
    }

    // ... (create/delete unchanged) ...

    public createCollection(name: string, schema: CollectionSchema, encryptionKey: string = '', noiseSigma: number = 0.02): Promise<boolean> {
        const metric = (schema.components && schema.components[0]) ? schema.components[0].metric : "l2";
        if (encryptionKey) {
            this.registerCollectionKey(name, encryptionKey, metric, noiseSigma, schema);
        }
        return new Promise((resolve, reject) => {
            const req = new CreateCollectionRequest();
            req.setName(name);
            
            const protoSchema = new hyperspace_pb.CollectionSchema();
            
            const components = schema.components.map(c => {
                const comp = new hyperspace_pb.VectorComponent();
                comp.setName(c.name);
                comp.setMetric(c.metric);
                comp.setFullDimension(c.fullDimension);
                comp.setWeight(c.weight);
                return comp;
            });
            protoSchema.setComponentsList(components);
            
            const pipeline = schema.cascadePipeline.map(l => {
                const layer = new hyperspace_pb.MrlLayer();
                layer.setComponentName(l.componentName);
                layer.setCutoffDimension(l.cutoffDimension);
                layer.setStoreInRam(l.storeInRam);
                layer.setRerankTopK(l.rerankTopK);
                return layer;
            });
            protoSchema.setCascadePipelineList(pipeline);
            
            req.setSchema(protoSchema);

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

    public freezeCollection(name: string): Promise<string> {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.FreezeCollectionRequest();
            req.setName(name);

            this.client.freezeCollection(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus());
            });
        });
    }

    public unfreezeCollection(name: string): Promise<string> {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.UnfreezeCollectionRequest();
            req.setName(name);

            this.client.unfreezeCollection(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getStatus());
            });
        });
    }

    public listCollections(): Promise<CollectionInfo[]> {
        return new Promise((resolve, reject) => {
            const req = new Empty();

            this.client.listCollections(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const list = (resp.getCollectionsList() as any as ProtoCollectionSummary[]).map(c => {
                    const info: CollectionInfo = {
                        name: c.getName(),
                        count: c.getCount()
                    };
                    const protoSchema = c.getSchema();
                    if (protoSchema) {
                        info.schema = {
                            components: protoSchema.getComponentsList().map((comp: any) => ({
                                name: comp.getName(),
                                metric: comp.getMetric(),
                                fullDimension: comp.getFullDimension(),
                                weight: comp.getWeight()
                            })),
                            cascadePipeline: protoSchema.getCascadePipelineList().map((layer: any) => ({
                                componentName: layer.getComponentName(),
                                cutoffDimension: layer.getCutoffDimension(),
                                storeInRam: layer.getStoreInRam(),
                                rerankTopK: layer.getRerankTopK()
                            }))
                        };
                    }
                    return info;
                });
                resolve(list);
            });
        });
    }

    public getPoints(ids: number[], collection: string = ''): Promise<{id: number, vector: number[], metadata: {[key: string]: string}}[]> {
        return new Promise((resolve, reject) => {
            const req = new GetPointsRequest();
            req.setCollection(collection);
            req.setIdsList(ids);

            this.client.getPoints(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const points = resp.getPointsList().map((p: any) => ({
                    id: p.getId(),
                    vector: p.getVectorList(),
                    metadata: p.getMetadataMap().toObject()
                }));
                resolve(points);
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
        return new Promise(async (resolve, reject) => {
            try {
                const req = new InsertRequest();
                let vectorList = HyperspaceClient.toVectorList(vector);
                
                const metric = this.collectionMetrics[collection] || "l2";
                const context = await this._getEncryptionContext(collection, vectorList.length, metric);
                
                if (context) {
                    const sigma = this.collectionNoiseSigmas[collection] ?? 0.02;
                    if (sigma > 0.0) {
                        vectorList = CognitiveMath.injectAnisotropicNoise(vectorList, context.hmacKey, sigma);
                    }
                    vectorList = this._projectCollectionVector(collection, vectorList, context, metric);
                    
                    let rawPayload: Buffer | null = null;
                    if (payload) {
                        rawPayload = Buffer.from(payload);
                    }
                    if (rawPayload) {
                        const encrypted = this.encryptPayload(rawPayload, context.aesKey);
                        req.setPayload(new Uint8Array(encrypted));
                    }
                    
                    if (meta) {
                        const map = req.getMetadataMap();
                        for (const k in meta) {
                            const ek = this.hashMetadataKey(k, context.hmacKey);
                            const ev = this.hashMetadataValue(meta[k], context.hmacKey);
                            map.set(ek, ev);
                        }
                    }
                    if (typedMetadata) {
                        const map = req.getTypedMetadataMap();
                        for (const k in typedMetadata) {
                            const ek = this.hashMetadataKey(k, context.hmacKey);
                            const ev = this.hashMetadataValue(String(typedMetadata[k]), context.hmacKey);
                            map.set(ek, HyperspaceClient.toProtoMetadataValue(ev));
                        }
                    }
                } else {
                    if (meta) {
                        const map = req.getMetadataMap();
                        for (const k in meta) map.set(k, meta[k]);
                    }
                    if (typedMetadata) {
                        const map = req.getTypedMetadataMap();
                        for (const k in typedMetadata) map.set(k, HyperspaceClient.toProtoMetadataValue(typedMetadata[k]));
                    }
                    if (payload) {
                        req.setPayload(payload);
                    }
                }
                
                req.setVectorList(vectorList);
                req.setId(id);
                req.setCollection(collection);
                req.setOriginNodeId('');
                req.setLogicalClock(0);
                req.setDurability(durability);

                this.client.insert(req, this.metadata, (err, resp) => {
                    if (err) return reject(err);
                    resolve(resp.getSuccess());
                });
            } catch (e) {
                reject(e);
            }
        });
    }

    public insertText(
        id: number,
        text: string,
        meta?: { [key: string]: string },
        collection: string = '',
        durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL
    ): Promise<boolean> {
        if (this.collectionKeys[collection]) {
            if (!this.embedder) {
                return Promise.reject(new Error("An embedder must be configured to use insertText on encrypted collections."));
            }
            return Promise.resolve(this.embedder.encode(text)).then(vector => {
                return this.insert(id, vector, meta, collection, durability, undefined, Buffer.from(text, 'utf-8'));
            });
        }
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
            filter?: { [key: string]: string },
            filters?: Filter[],
            hybridQuery?: string,
            hybridAlpha?: number,
            bm25?: Bm25Options,
            mrlDimension?: number,
            useWasserstein?: boolean,
            includePayload?: boolean,
            componentWeights?: { [key: string]: number },
            useWave?: boolean,
            restartFactor?: number
        }
    ): Promise<SearchResult[]> {
        return new Promise(async (resolve, reject) => {
            try {
                const req = new SearchRequest();
                let vectorList = HyperspaceClient.toVectorList(vector);
                req.setTopK(topK);
                req.setCollection(collection);
                
                const metric = this.collectionMetrics[collection] || "l2";
                const context = await this._getEncryptionContext(collection, vectorList.length, metric);
                
                let optFilters = options?.filters;
                let optFilter = options?.filter;
                let includePayload = options?.includePayload;
                
                if (context) {
                    const sigma = this.collectionNoiseSigmas[collection] ?? 0.02;
                    if (sigma > 0.0) {
                        vectorList = CognitiveMath.injectAnisotropicNoise(vectorList, context.hmacKey, sigma);
                    }
                    vectorList = this._projectCollectionVector(collection, vectorList, context, metric);
                    
                    if (optFilter) {
                        const hashedFilter: { [key: string]: string } = {};
                        for (const k in optFilter) {
                            const ek = this.hashMetadataKey(k, context.hmacKey);
                            const ev = this.hashMetadataValue(optFilter[k], context.hmacKey);
                            hashedFilter[ek] = ev;
                        }
                        optFilter = hashedFilter;
                    }
                    if (optFilters) {
                        optFilters = this._encryptFilters(optFilters, context);
                    }
                    
                    includePayload = true;
                }
                
                req.setVectorList(vectorList);
                
                if (optFilter) {
                    const map = req.getFilterMap();
                    for (const k in optFilter) {
                        map.set(k, optFilter[k]);
                    }
                }
                
                if (options?.restartFactor !== undefined) {
                    const map = req.getFilterMap();
                    map.set('wave_restart_factor', options.restartFactor.toString());
                }
                if (options?.useWave !== undefined) {
                    req.setUseWave(options.useWave);
                }
                if (optFilters) {
                    req.setFiltersList(optFilters.map(f => this.toProtoFilter(f)));
                }
                if (options?.hybridQuery) req.setHybridQuery(options.hybridQuery);
                if (options?.hybridAlpha !== undefined) req.setHybridAlpha(options.hybridAlpha);
                if (options?.mrlDimension !== undefined) req.setMrlDimension(options.mrlDimension);
                if (options?.useWasserstein !== undefined) req.setUseWasserstein(options.useWasserstein);
                if (includePayload !== undefined) req.setIncludePayload(includePayload);
                
                if (options?.componentWeights) {
                    const map = req.getComponentWeightsMap();
                    for (const k in options.componentWeights) {
                        map.set(k, options.componentWeights[k]);
                    }
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
                        
                        let payloadBytes = r.getPayload_asU8();
                        if (context && payloadBytes && payloadBytes.length > 0) {
                            try {
                                const decrypted = this.decryptPayload(Buffer.from(payloadBytes), context.aesKey);
                                payloadBytes = new Uint8Array(decrypted);
                            } catch (e) {
                                // Decryption error
                            }
                        }
                        
                        return {
                            id: r.getId(),
                            distance: r.getDistance(),
                            metadata: meta,
                            typedMetadata: HyperspaceClient.parseTypedMetadata(r.getTypedMetadataMap()),
                            payload: payloadBytes
                        };
                    });
                    resolve(results);
                });
            } catch (e) {
                reject(e);
            }
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
        if (this.collectionKeys[collection]) {
            if (!this.embedder) {
                return Promise.reject(new Error("An embedder must be configured to use searchText on encrypted collections."));
            }
            return Promise.resolve(this.embedder.encode(text)).then(vector => {
                return this.search(vector, topK, collection, options);
            });
        }
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
                const stats: CollectionStats = {
                    count: resp.getCount(),
                    indexingQueue: resp.getIndexingQueue(),
                    diskUsageBytes: resp.getDiskUsageBytes(),
                    ramUsageBytes: resp.getRamUsageBytes(),
                    activeTasks: resp.getActiveTasks()
                };
                const protoSchema = resp.getSchema();
                if (protoSchema) {
                    stats.schema = {
                        components: protoSchema.getComponentsList().map((comp: any) => ({
                            name: comp.getName(),
                            metric: comp.getMetric(),
                            fullDimension: comp.getFullDimension(),
                            weight: comp.getWeight()
                        })),
                        cascadePipeline: protoSchema.getCascadePipelineList().map((layer: any) => ({
                            componentName: layer.getComponentName(),
                            cutoffDimension: layer.getCutoffDimension(),
                            storeInRam: layer.getStoreInRam(),
                            rerankTopK: layer.getRerankTopK()
                        }))
                    };
                }
                resolve(stats);
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

    public async getCacheStats(name: string): Promise<any> {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/stats`;
        const headers: { [key: string]: string } = {};
        if (this.apiKey) headers['x-api-key'] = this.apiKey;
        if (this.userId) headers['x-hyperspace-user-id'] = this.userId;

        const res = await fetch(url, { headers });
        if (!res.ok) {
            throw new Error(`Failed to get cache stats: ${res.statusText} (${await res.text()})`);
        }
        return res.json();
    }

    public async clearCache(name: string): Promise<boolean> {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/clear`;
        const headers: { [key: string]: string } = { 'Content-Type': 'application/json' };
        if (this.apiKey) headers['x-api-key'] = this.apiKey;
        if (this.userId) headers['x-hyperspace-user-id'] = this.userId;

        const res = await fetch(url, {
            method: 'POST',
            headers,
            body: JSON.stringify({})
        });
        if (!res.ok) {
            throw new Error(`Failed to clear cache: ${res.statusText} (${await res.text()})`);
        }
        const data = await res.json();
        return data.status === 'success';
    }

    public async updateCacheConfig(name: string, policy: string, annThreshold?: number): Promise<boolean> {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/config`;
        const headers: { [key: string]: string } = { 'Content-Type': 'application/json' };
        if (this.apiKey) headers['x-api-key'] = this.apiKey;
        if (this.userId) headers['x-hyperspace-user-id'] = this.userId;

        const res = await fetch(url, {
            method: 'POST',
            headers,
            body: JSON.stringify({
                policy,
                ann_threshold: annThreshold
            })
        });
        if (!res.ok) {
            throw new Error(`Failed to update cache config: ${res.statusText} (${await res.text()})`);
        }
        const data = await res.json();
        return data.status === 'success';
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

    public getSubsumptionTree(rootId: number, maxDepth: number = 3, collection: string = ''): Promise<GraphNode[]> {
        return new Promise((resolve, reject) => {
            const req = new (hyperspace_pb as any).GetSubsumptionTreeRequest();
            req.setRootId(rootId);
            req.setMaxDepth(maxDepth);
            req.setCollection(collection);

            (this.client as any).getSubsumptionTree(req, this.metadata, (err: any, resp: any) => {
                if (err) return reject(err);
                const nodes = resp.getNodesList().map((n: any) => ({
                    id: n.getId(),
                    metadata: n.getMetadataMap().toObject(),
                    edgeTypes: n.getEdgeTypesList()
                }));
                resolve(nodes);
            });
        });
    }

    public async exploreGraph(startId: number, maxDepth: number = 2, maxNodes: number = 256, collection: string = ''): Promise<any> {
        const req = new TraverseRequest();
        req.setStartId(startId);
        req.setMaxDepth(maxDepth);
        req.setMaxNodes(maxNodes);
        req.setCollection(collection);
        (req as any).setTraversalMode(1); // DIFFUSIVE
        (req as any).setBreadthLimit(10);

        return new Promise((resolve, reject) => {
            this.client.traverse(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const nodes = resp.getNodesList().map((n: any) => ({
                    id: n.getId(),
                    metadata: n.getMetadataMap().toObject(),
                    edgeTypes: n.getEdgeTypesList(),
                    neighbors: n.getNeighborsList()
                }));
                resolve({
                    nodes,
                    centerId: startId,
                    count: nodes.length
                });
            });
        });
    }

    public async predictMomentum(trajectoryIds: number[], steps: number = 1.0, collection: string = '', curvature: number = 1.0): Promise<number[]> {
        if (trajectoryIds.length < 2) return [];

        const pts = await this.getPoints(trajectoryIds, collection);
        const idToVec = new Map<number, number[]>();
        pts.forEach(p => idToVec.set(p.id, p.vector));

        const vectors = trajectoryIds.map(id => idToVec.get(id)).filter(v => !!v) as number[][];
        if (vectors.length < 2) return [];

        const past = vectors[vectors.length - 2];
        const current = vectors[vectors.length - 1];

        const { koopmanExtrapolate } = require('./math');
        return koopmanExtrapolate(past, current, steps, curvature);
    }

    public async getTrustScore(trajectoryIds: number[], collection: string = '', curvature: number = 1.0): Promise<number> {
        const pts = await this.getPoints(trajectoryIds, collection);
        const idToVec = new Map<number, number[]>();
        pts.forEach(p => idToVec.set(p.id, p.vector));

        const vectors = trajectoryIds.map(id => idToVec.get(id)).filter(v => !!v) as number[][];
        if (vectors.length < 3) return 0.0;

        const { lyapunovConvergence } = require('./math');
        try {
            return lyapunovConvergence(vectors, curvature);
        } catch (e) {
            return 1.0;
        }
    }


    public close() {
        this.client.close();
    }
}
