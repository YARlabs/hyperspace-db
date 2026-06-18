"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.HyperspaceClient = exports.HyperbolicMath = exports.DurabilityLevel = exports.TribunalContext = exports.CognitiveMath = void 0;
const grpc = __importStar(require("@grpc/grpc-js"));
const hyperspace_grpc_pb_1 = require("./proto/hyperspace_grpc_pb");
const hyperspace_pb_1 = require("./proto/hyperspace_pb");
Object.defineProperty(exports, "DurabilityLevel", { enumerable: true, get: function () { return hyperspace_pb_1.DurabilityLevel; } });
const hyperspace_pb = __importStar(require("./proto/hyperspace_pb")); // New, for direct access to types
exports.CognitiveMath = __importStar(require("./math"));
var agents_1 = require("./agents");
Object.defineProperty(exports, "TribunalContext", { enumerable: true, get: function () { return agents_1.TribunalContext; } });
exports.HyperbolicMath = {
    projectToBall(x, c = 1.0) {
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        const n = Math.sqrt(Math.max(normSq(x), 0));
        const maxN = (1 / Math.sqrt(c)) - 1e-9;
        if (n <= maxN || n <= 1e-15)
            return [...x];
        const scale = maxN / n;
        return x.map((v) => v * scale);
    },
    mobiusAdd(x, y, c = 1.0) {
        if (x.length !== y.length)
            throw new Error('Dimension mismatch');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const dot = (a, b) => a.reduce((s, v, i) => s + v * b[i], 0);
        const x2 = dot(x, x);
        const y2 = dot(y, y);
        const xy = dot(x, y);
        const left = 1 + 2 * c * xy + c * y2;
        const right = 1 - c * x2;
        const den = 1 + 2 * c * xy + c * c * x2 * y2;
        if (Math.abs(den) < 1e-15)
            throw new Error('Mobius denominator too small');
        return x.map((xi, i) => (left * xi + right * y[i]) / den);
    },
    expMap(x, v, c = 1.0) {
        if (x.length !== v.length)
            throw new Error('Dimension mismatch');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        const vNorm = Math.sqrt(Math.max(normSq(v), 0));
        if (vNorm < 1e-15)
            return [...x];
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const scale = Math.tanh(Math.sqrt(c) * lambdaX * vNorm / 2) / (Math.sqrt(c) * vNorm);
        return exports.HyperbolicMath.mobiusAdd(x, v.map((vi) => scale * vi), c);
    },
    logMap(x, y, c = 1.0) {
        if (x.length !== y.length)
            throw new Error('Dimension mismatch');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        const delta = exports.HyperbolicMath.mobiusAdd(x.map((xi) => -xi), y, c);
        const deltaNorm = Math.sqrt(Math.max(normSq(delta), 0));
        if (deltaNorm < 1e-15)
            return new Array(x.length).fill(0);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const arg = Math.min(Math.sqrt(c) * deltaNorm, 1 - 1e-15);
        const factor = (2 / (lambdaX * Math.sqrt(c))) * Math.atanh(arg);
        return delta.map((di) => factor * di / deltaNorm);
    },
    riemannianGradient(x, euclideanGrad, c = 1.0) {
        if (x.length !== euclideanGrad.length)
            throw new Error('Dimension mismatch');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const scale = 1 / (lambdaX * lambdaX);
        return euclideanGrad.map((g) => scale * g);
    },
    parallelTransport(x, y, v, c = 1.0) {
        if (x.length !== y.length || x.length !== v.length)
            throw new Error('Dimension mismatch');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        const gyro = (u, w, z) => {
            const uw = exports.HyperbolicMath.mobiusAdd(u, w, c);
            const wz = exports.HyperbolicMath.mobiusAdd(w, z, c);
            const left = exports.HyperbolicMath.mobiusAdd(u, wz, c);
            return exports.HyperbolicMath.mobiusAdd(uw.map((k) => -k), left, c);
        };
        const g = gyro(y, x.map((xi) => -xi), v);
        const lambdaX = 2 / Math.max(1 - c * normSq(x), 1e-15);
        const lambdaY = 2 / Math.max(1 - c * normSq(y), 1e-15);
        const scale = lambdaX / lambdaY;
        return g.map((gi) => scale * gi);
    },
    frechetMean(points, c = 1.0, maxIter = 64, tol = 1e-8) {
        if (!points.length)
            throw new Error('Points set cannot be empty');
        if (c <= 0)
            throw new Error('Curvature c must be > 0');
        const dim = points[0].length;
        if (points.some((p) => p.length !== dim))
            throw new Error('Dimension mismatch');
        const normSq = (u) => u.reduce((s, z) => s + z * z, 0);
        let mu = exports.HyperbolicMath.projectToBall(points[0], c);
        for (let iter = 0; iter < Math.max(1, maxIter); iter++) {
            const grad = new Array(dim).fill(0);
            for (const p of points) {
                const lg = exports.HyperbolicMath.logMap(mu, p, c);
                for (let i = 0; i < dim; i++)
                    grad[i] += lg[i];
            }
            for (let i = 0; i < dim; i++)
                grad[i] /= points.length;
            const gNorm = Math.sqrt(Math.max(normSq(grad), 0));
            if (gNorm <= Math.max(tol, 1e-15))
                break;
            mu = exports.HyperbolicMath.expMap(mu, grad, c);
            mu = exports.HyperbolicMath.projectToBall(mu, c);
        }
        return mu;
    },
    analyzeDeltaHyperbolicity(vectors, numSamples = 1000) {
        if (vectors.length < 4)
            return { delta: 0, recommendation: 'euclidean' };
        const l2Dist = (a, b) => Math.sqrt(a.reduce((s, x, i) => s + (x - b[i]) ** 2, 0));
        let maxDelta = 0;
        for (let s = 0; s < numSamples; s++) {
            const indices = new Set();
            while (indices.size < 4)
                indices.add(Math.floor(Math.random() * vectors.length));
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
            if (delta > maxDelta)
                maxDelta = delta;
        }
        const recommendation = maxDelta < 0.15 ? 'lorentz' : (maxDelta < 0.30 ? 'poincare' : 'l2');
        return { delta: maxDelta, recommendation };
    }
};
class HyperspaceClient {
    static toVectorList(vector) {
        if (Array.isArray(vector)) {
            return vector;
        }
        return Array.from(vector);
    }
    static toProtoMetadataValue(value) {
        const out = new hyperspace_pb.MetadataValue();
        if (typeof value === 'string')
            out.setStringValue(value);
        else if (typeof value === 'boolean')
            out.setBoolValue(value);
        else if (Number.isInteger(value))
            out.setIntValue(Number(value));
        else
            out.setDoubleValue(Number(value));
        return out;
    }
    static parseTypedMetadata(metaMap) {
        const out = {};
        if (metaMap.getLength() === 0)
            return out;
        metaMap.forEach((value, key) => {
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
    toProtoFilter(f) {
        const pf = new hyperspace_pb.Filter();
        if (f.match) {
            const m = new hyperspace_pb.Match();
            m.setKey(f.match.key);
            m.setValue(f.match.value);
            pf.setMatch(m);
        }
        else if (f.prefix) {
            const p = new hyperspace_pb.Prefix();
            p.setKey(f.prefix.key);
            p.setPrefix(f.prefix.prefix);
            pf.setPrefix(p);
        }
        else if (f.range) {
            const r = new hyperspace_pb.Range();
            r.setKey(f.range.key);
            if (f.range.gte !== undefined) {
                if (Number.isInteger(f.range.gte))
                    r.setGte(f.range.gte);
                else
                    r.setGteF64(f.range.gte);
            }
            if (f.range.lte !== undefined) {
                if (Number.isInteger(f.range.lte))
                    r.setLte(f.range.lte);
                else
                    r.setLteF64(f.range.lte);
            }
            pf.setRange(r);
        }
        else if (f.inCone) {
            const c = new hyperspace_pb.InCone();
            c.setAxesList(f.inCone.axes);
            c.setAperturesList(f.inCone.apertures);
            c.setCen(f.inCone.cen[0] || 0);
            pf.setInCone(c);
        }
        else if (f.inBall) {
            const b = new hyperspace_pb.InBall();
            b.setCenterList(f.inBall.center);
            b.setRadius(f.inBall.radius);
            pf.setInBall(b);
        }
        else if (f.inBox) {
            const b = new hyperspace_pb.InBox();
            b.setMinBoundsList(f.inBox.minBounds);
            b.setMaxBoundsList(f.inBox.maxBounds);
            pf.setInBox(b);
        }
        else if (f.and) {
            const andOp = new hyperspace_pb.FilterAnd();
            andOp.setConditionsList(f.and.map(cond => this.toProtoFilter(cond)));
            pf.setAndOp(andOp);
        }
        else if (f.or) {
            const orOp = new hyperspace_pb.FilterOr();
            orOp.setConditionsList(f.or.map(cond => this.toProtoFilter(cond)));
            pf.setOrOp(orOp);
        }
        else if (f.not) {
            const notOp = new hyperspace_pb.FilterNot();
            notOp.setCondition(this.toProtoFilter(f.not));
            pf.setNotOp(notOp);
        }
        return pf;
    }
    constructor(host = 'localhost:50051', apiKey, userId) {
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
        this.client = new hyperspace_grpc_pb_1.DatabaseClient(host, grpc.credentials.createInsecure(), options);
        this.metadata = new grpc.Metadata();
        if (apiKey) {
            this.metadata.add('x-api-key', apiKey);
        }
        if (userId) {
            this.metadata.add('x-hyperspace-user-id', userId);
        }
    }
    // ... (create/delete unchanged) ...
    createCollection(name, schema) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.CreateCollectionRequest();
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
                if (err)
                    return reject(err);
                resolve(true);
            });
        });
    }
    deleteCollection(name) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.DeleteCollectionRequest();
            req.setName(name);
            this.client.deleteCollection(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(true);
            });
        });
    }
    freezeCollection(name) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.FreezeCollectionRequest();
            req.setName(name);
            this.client.freezeCollection(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus());
            });
        });
    }
    unfreezeCollection(name) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.UnfreezeCollectionRequest();
            req.setName(name);
            this.client.unfreezeCollection(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus());
            });
        });
    }
    listCollections() {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.Empty();
            this.client.listCollections(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const list = resp.getCollectionsList().map(c => {
                    const info = {
                        name: c.getName(),
                        count: c.getCount()
                    };
                    const protoSchema = c.getSchema();
                    if (protoSchema) {
                        info.schema = {
                            components: protoSchema.getComponentsList().map((comp) => ({
                                name: comp.getName(),
                                metric: comp.getMetric(),
                                fullDimension: comp.getFullDimension(),
                                weight: comp.getWeight()
                            })),
                            cascadePipeline: protoSchema.getCascadePipelineList().map((layer) => ({
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
    getPoints(ids, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.GetPointsRequest();
            req.setCollection(collection);
            req.setIdsList(ids);
            this.client.getPoints(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const points = resp.getPointsList().map((p) => ({
                    id: p.getId(),
                    vector: p.getVectorList(),
                    metadata: p.getMetadataMap().toObject()
                }));
                resolve(points);
            });
        });
    }
    delete(id, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.DeleteRequest();
            req.setCollection(collection);
            req.setId(id);
            this.client.delete(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }
    insert(id, vector, meta, collection = '', durability = hyperspace_pb_1.DurabilityLevel.DEFAULT_LEVEL, typedMetadata, payload // Sidecar Payload Storage (v3.2)
    ) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.InsertRequest();
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            req.setId(id);
            if (meta) {
                const map = req.getMetadataMap();
                for (const k in meta)
                    map.set(k, meta[k]);
            }
            if (typedMetadata) {
                const map = req.getTypedMetadataMap();
                for (const k in typedMetadata)
                    map.set(k, HyperspaceClient.toProtoMetadataValue(typedMetadata[k]));
            }
            req.setCollection(collection);
            req.setOriginNodeId('');
            req.setLogicalClock(0);
            req.setDurability(durability);
            if (payload) {
                req.setPayload(payload);
            }
            this.client.insert(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }
    insertText(id, text, meta, collection = '', durability = hyperspace_pb_1.DurabilityLevel.DEFAULT_LEVEL) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.InsertTextRequest();
            req.setText(text);
            req.setId(id);
            if (meta) {
                const map = req.getMetadataMap();
                for (const k in meta)
                    map.set(k, meta[k]);
            }
            req.setCollection(collection);
            req.setDurability(durability);
            this.client.insertText(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }
    vectorize(text, metric = 'l2') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.VectorizeRequest();
            req.setText(text);
            req.setMetric(metric);
            this.client.vectorize(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getVectorList());
            });
        });
    }
    batchInsert(items, collection = '', durability = hyperspace_pb_1.DurabilityLevel.DEFAULT_LEVEL) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.BatchInsertRequest();
            req.setCollection(collection);
            req.setDurability(durability);
            const vectors = items.map(item => {
                const v = new hyperspace_pb_1.VectorData();
                v.setId(item.id);
                v.setVectorList(HyperspaceClient.toVectorList(item.vector));
                if (item.metadata) {
                    const map = v.getMetadataMap();
                    for (const k in item.metadata)
                        map.set(k, item.metadata[k]);
                }
                if (item.typedMetadata) {
                    const map = v.getTypedMetadataMap();
                    for (const k in item.typedMetadata)
                        map.set(k, HyperspaceClient.toProtoMetadataValue(item.typedMetadata[k]));
                }
                return v;
            });
            req.setVectorsList(vectors);
            this.client.batchInsert(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }
    search(vector, topK, collection = '', options) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.SearchRequest();
            req.setVectorList(HyperspaceClient.toVectorList(vector));
            req.setTopK(topK);
            req.setCollection(collection);
            if (options?.filter) {
                const map = req.getFilterMap();
                for (const k in options.filter) {
                    map.set(k, options.filter[k]);
                }
            }
            if (options?.restartFactor !== undefined) {
                const map = req.getFilterMap();
                map.set('wave_restart_factor', options.restartFactor.toString());
            }
            if (options?.useWave !== undefined) {
                req.setUseWave(options.useWave);
            }
            if (options?.filters) {
                req.setFiltersList(options.filters.map(f => this.toProtoFilter(f)));
            }
            if (options?.hybridQuery)
                req.setHybridQuery(options.hybridQuery);
            if (options?.hybridAlpha !== undefined)
                req.setHybridAlpha(options.hybridAlpha);
            if (options?.mrlDimension !== undefined)
                req.setMrlDimension(options.mrlDimension);
            if (options?.useWasserstein !== undefined)
                req.setUseWasserstein(options.useWasserstein);
            if (options?.includePayload !== undefined)
                req.setIncludePayload(options.includePayload);
            if (options?.componentWeights) {
                const map = req.getComponentWeightsMap();
                for (const k in options.componentWeights) {
                    map.set(k, options.componentWeights[k]);
                }
            }
            if (options?.bm25) {
                const bm25Msg = new hyperspace_pb_1.Bm25Options();
                if (options.bm25.method !== undefined)
                    bm25Msg.setMethod(options.bm25.method);
                if (options.bm25.k1 !== undefined)
                    bm25Msg.setK1(options.bm25.k1);
                if (options.bm25.b !== undefined)
                    bm25Msg.setB(options.bm25.b);
                if (options.bm25.delta !== undefined)
                    bm25Msg.setDelta(options.bm25.delta);
                if (options.bm25.language !== undefined)
                    bm25Msg.setLanguage(options.bm25.language);
                if (options.bm25.ngrams !== undefined)
                    bm25Msg.setNgrams(options.bm25.ngrams);
                if (options.bm25.fusionMethod !== undefined)
                    bm25Msg.setFusionMethod(options.bm25.fusionMethod);
                req.setBm25Options(bm25Msg);
            }
            this.client.search(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const results = resp.getResultsList().map(r => {
                    const metaMap = r.getMetadataMap();
                    const meta = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
    searchText(text, topK, collection = '', options) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.SearchTextRequest();
            req.setText(text);
            req.setTopK(topK);
            req.setCollection(collection);
            if (options?.filters) {
                req.setFiltersList(options.filters.map(f => this.toProtoFilter(f)));
            }
            if (options?.bm25) {
                const bm25Msg = new hyperspace_pb_1.Bm25Options();
                if (options.bm25.method !== undefined)
                    bm25Msg.setMethod(options.bm25.method);
                if (options.bm25.k1 !== undefined)
                    bm25Msg.setK1(options.bm25.k1);
                if (options.bm25.b !== undefined)
                    bm25Msg.setB(options.bm25.b);
                if (options.bm25.delta !== undefined)
                    bm25Msg.setDelta(options.bm25.delta);
                if (options.bm25.language !== undefined)
                    bm25Msg.setLanguage(options.bm25.language);
                if (options.bm25.ngrams !== undefined)
                    bm25Msg.setNgrams(options.bm25.ngrams);
                if (options.bm25.fusionMethod !== undefined)
                    bm25Msg.setFusionMethod(options.bm25.fusionMethod);
                req.setBm25Options(bm25Msg);
            }
            if (options?.hybridAlpha !== undefined) {
                req.setHybridAlpha(options.hybridAlpha);
            }
            if (options?.includePayload !== undefined) {
                req.setIncludePayload(options.includePayload);
            }
            this.client.searchText(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const results = resp.getResultsList().map(r => {
                    const metaMap = r.getMetadataMap();
                    const meta = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
    searchBatch(vectors, topK, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.BatchSearchRequest();
            req.setSearchesList(vectors.map((vector) => {
                const s = new hyperspace_pb_1.SearchRequest();
                s.setVectorList(HyperspaceClient.toVectorList(vector));
                s.setTopK(topK);
                s.setCollection(collection);
                return s;
            }));
            this.client.searchBatch(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const batch = resp.getResponsesList().map((searchResp) => searchResp.getResultsList().map((r) => {
                    const metaMap = r.getMetadataMap();
                    const meta = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
                }));
                resolve(batch);
            });
        });
    }
    getDigest(collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.DigestRequest();
            req.setCollection(collection);
            this.client.getDigest(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve({
                    logicalClock: resp.getLogicalClock(),
                    stateHash: resp.getStateHash(),
                    count: resp.getCount()
                });
            });
        });
    }
    rebuildIndex(collection) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.RebuildIndexRequest();
            req.setName(collection);
            this.client.rebuildIndex(req, this.metadata, (err) => {
                if (err)
                    return reject(err);
                resolve(true);
            });
        });
    }
    rebuildIndexWithFilter(collection, filter) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.RebuildIndexRequest();
            req.setName(collection);
            const fq = new hyperspace_pb_1.VacuumFilterQuery();
            fq.setKey(filter.key);
            fq.setOp(filter.op);
            fq.setValue(filter.value);
            req.setFilterQuery(fq);
            this.client.rebuildIndex(req, this.metadata, (err) => {
                if (err)
                    return reject(err);
                resolve(true);
            });
        });
    }
    getNode(id, layer = 0, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.GetNodeRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);
            this.client.getNode(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const metaMap = resp.getMetadataMap();
                const metadata = {};
                if (metaMap.getLength() > 0) {
                    metaMap.forEach((entry, key) => {
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
    getNeighbors(id, layer = 0, limit = 64, offset = 0, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.GetNeighborsRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);
            req.setLimit(limit);
            req.setOffset(offset);
            this.client.getNeighbors(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const nodes = resp.getNeighborsList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
    getConceptParents(id, layer = 0, limit = 32, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.GetConceptParentsRequest();
            req.setCollection(collection);
            req.setId(id);
            req.setLayer(layer);
            req.setLimit(limit);
            this.client.getConceptParents(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const nodes = resp.getParentsList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
    traverse(startId, layer = 0, maxDepth = 2, maxNodes = 256, collection = '', options) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.TraverseRequest();
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
                if (err)
                    return reject(err);
                const nodes = resp.getNodesList().map((n) => {
                    const metaMap = n.getMetadataMap();
                    const metadata = {};
                    if (metaMap.getLength() > 0) {
                        metaMap.forEach((entry, key) => {
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
    findSemanticClusters(layer = 0, minClusterSize = 3, maxClusters = 32, maxNodes = 10000, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.FindSemanticClustersRequest();
            req.setCollection(collection);
            req.setLayer(layer);
            req.setMinClusterSize(minClusterSize);
            req.setMaxClusters(maxClusters);
            req.setMaxNodes(maxNodes);
            this.client.findSemanticClusters(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getClustersList().map((c) => c.getNodeIdsList()));
            });
        });
    }
    subscribeToEvents(options, onEvent, onError) {
        const req = new hyperspace_pb_1.EventSubscriptionRequest();
        if (options.collection) {
            req.setCollection(options.collection);
        }
        const requested = options.types || [];
        if (requested.length > 0) {
            const mapped = requested.map((t) => t === 'insert' ? hyperspace_pb_1.EventType.VECTOR_INSERTED : hyperspace_pb_1.EventType.VECTOR_DELETED);
            req.setTypesList(mapped);
        }
        const stream = this.client.subscribeToEvents(req, this.metadata);
        stream.on('data', onEvent);
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }
    syncHandshake(collection, clientBuckets, clientLogicalClock = 0, clientCount = 0) {
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
                if (err)
                    return reject(err);
                resolve(res.toObject());
            });
        });
    }
    syncPull(collection, bucketIndices, onData, onError) {
        const req = new hyperspace_pb.SyncPullRequest();
        req.setCollection(collection);
        req.setBucketIndicesList(bucketIndices);
        const stream = this.client.syncPull(req, this.metadata);
        stream.on('data', (data) => {
            onData(data.toObject());
        });
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }
    getCollectionStats(name) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.CollectionStatsRequest();
            req.setName(name);
            this.client.getCollectionStats(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const stats = {
                    count: resp.getCount(),
                    indexingQueue: resp.getIndexingQueue(),
                    diskUsageBytes: resp.getDiskUsageBytes(),
                    ramUsageBytes: resp.getRamUsageBytes(),
                    activeTasks: resp.getActiveTasks()
                };
                const protoSchema = resp.getSchema();
                if (protoSchema) {
                    stats.schema = {
                        components: protoSchema.getComponentsList().map((comp) => ({
                            name: comp.getName(),
                            metric: comp.getMetric(),
                            fullDimension: comp.getFullDimension(),
                            weight: comp.getWeight()
                        })),
                        cascadePipeline: protoSchema.getCascadePipelineList().map((layer) => ({
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
    async exists(name) {
        try {
            await this.getCollectionStats(name);
            return true;
        }
        catch (e) {
            if (e.code === grpc.status.NOT_FOUND || (e.message && e.message.includes('not found'))) {
                return false;
            }
            throw e;
        }
    }
    async getCacheStats(name) {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/stats`;
        const headers = {};
        if (this.apiKey)
            headers['x-api-key'] = this.apiKey;
        if (this.userId)
            headers['x-hyperspace-user-id'] = this.userId;
        const res = await fetch(url, { headers });
        if (!res.ok) {
            throw new Error(`Failed to get cache stats: ${res.statusText} (${await res.text()})`);
        }
        return res.json();
    }
    async clearCache(name) {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/clear`;
        const headers = { 'Content-Type': 'application/json' };
        if (this.apiKey)
            headers['x-api-key'] = this.apiKey;
        if (this.userId)
            headers['x-hyperspace-user-id'] = this.userId;
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
    async updateCacheConfig(name, policy, annThreshold) {
        const ip = this.host.split(':')[0];
        const url = `http://${ip}:50050/api/collections/${name}/cache/config`;
        const headers = { 'Content-Type': 'application/json' };
        if (this.apiKey)
            headers['x-api-key'] = this.apiKey;
        if (this.userId)
            headers['x-hyperspace-user-id'] = this.userId;
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
    updateCollection(name, config) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.ConfigUpdate();
            req.setCollection(name);
            if (config.efSearch !== undefined)
                req.setEfSearch(config.efSearch);
            if (config.efConstruction !== undefined)
                req.setEfConstruction(config.efConstruction);
            if (config.m !== undefined)
                req.setM(config.m);
            this.client.configure(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus() === 'success' || (!!resp.getStatus() && resp.getStatus().includes('updated')));
            });
        });
    }
    createSnapshot() {
        return new Promise((resolve, reject) => {
            this.client.triggerSnapshot(new hyperspace_pb_1.Empty(), this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }
    vacuum() {
        return new Promise((resolve, reject) => {
            this.client.triggerVacuum(new hyperspace_pb_1.Empty(), this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }
    getMetrics(onData, onError) {
        const req = new hyperspace_pb_1.MonitorRequest();
        const stream = this.client.monitor(req, this.metadata);
        stream.on('data', onData);
        if (onError) {
            stream.on('error', onError);
        }
        return stream;
    }
    searchMultiCollection(collections, query) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.SearchMultiCollectionRequest();
            req.setCollectionsList(collections);
            req.setVectorList(HyperspaceClient.toVectorList(query));
            req.setTopK(10);
            this.client.searchMultiCollection(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.toObject());
            });
        });
    }
    triggerReconsolidation(collection, targetVector, learningRate = 0.01) {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb_1.ReconsolidationRequest();
            req.setCollection(collection);
            req.setTargetVectorList(targetVector);
            req.setLearningRate(learningRate);
            this.client.triggerReconsolidation(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                resolve(resp.getStatus() === 'success');
            });
        });
    }
    getSubsumptionTree(rootId, maxDepth = 3, collection = '') {
        return new Promise((resolve, reject) => {
            const req = new hyperspace_pb.GetSubsumptionTreeRequest();
            req.setRootId(rootId);
            req.setMaxDepth(maxDepth);
            req.setCollection(collection);
            this.client.getSubsumptionTree(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const nodes = resp.getNodesList().map((n) => ({
                    id: n.getId(),
                    metadata: n.getMetadataMap().toObject(),
                    edgeTypes: n.getEdgeTypesList()
                }));
                resolve(nodes);
            });
        });
    }
    async exploreGraph(startId, maxDepth = 2, maxNodes = 256, collection = '') {
        const req = new hyperspace_pb_1.TraverseRequest();
        req.setStartId(startId);
        req.setMaxDepth(maxDepth);
        req.setMaxNodes(maxNodes);
        req.setCollection(collection);
        req.setTraversalMode(1); // DIFFUSIVE
        req.setBreadthLimit(10);
        return new Promise((resolve, reject) => {
            this.client.traverse(req, this.metadata, (err, resp) => {
                if (err)
                    return reject(err);
                const nodes = resp.getNodesList().map((n) => ({
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
    async predictMomentum(trajectoryIds, steps = 1.0, collection = '', curvature = 1.0) {
        if (trajectoryIds.length < 2)
            return [];
        const pts = await this.getPoints(trajectoryIds, collection);
        const idToVec = new Map();
        pts.forEach(p => idToVec.set(p.id, p.vector));
        const vectors = trajectoryIds.map(id => idToVec.get(id)).filter(v => !!v);
        if (vectors.length < 2)
            return [];
        const past = vectors[vectors.length - 2];
        const current = vectors[vectors.length - 1];
        const { koopmanExtrapolate } = require('./math');
        return koopmanExtrapolate(past, current, steps, curvature);
    }
    async getTrustScore(trajectoryIds, collection = '', curvature = 1.0) {
        const pts = await this.getPoints(trajectoryIds, collection);
        const idToVec = new Map();
        pts.forEach(p => idToVec.set(p.id, p.vector));
        const vectors = trajectoryIds.map(id => idToVec.get(id)).filter(v => !!v);
        if (vectors.length < 3)
            return 0.0;
        const { lyapunovConvergence } = require('./math');
        try {
            return lyapunovConvergence(vectors, curvature);
        }
        catch (e) {
            return 1.0;
        }
    }
    close() {
        this.client.close();
    }
}
exports.HyperspaceClient = HyperspaceClient;
