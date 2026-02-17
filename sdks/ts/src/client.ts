import * as grpc from '@grpc/grpc-js';
import { DatabaseClient } from './proto/hyperspace_grpc_pb';
import {
    InsertRequest, SearchRequest,
    CreateCollectionRequest, DeleteCollectionRequest, Empty,
    DurabilityLevel
} from './proto/hyperspace_pb';

export { DurabilityLevel };

export class HyperspaceClient {
    private client: DatabaseClient;
    private metadata: grpc.Metadata;

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

    public insert(id: number, vector: number[], meta?: { [key: string]: string }, collection: string = '', durability: DurabilityLevel = DurabilityLevel.DEFAULT_LEVEL): Promise<boolean> {
        return new Promise((resolve, reject) => {
            const req = new InsertRequest();
            req.setId(id);
            req.setVectorList(vector);
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

    public search(vector: number[], topK: number, collection: string = ''): Promise<{ id: number, distance: number, metadata: { [key: string]: string } }[]> {
        return new Promise((resolve, reject) => {
            const req = new SearchRequest();
            req.setVectorList(vector);
            req.setTopK(topK);
            req.setCollection(collection);

            this.client.search(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const results = resp.getResultsList().map(r => {
                    const metaMap = r.getMetadataMap();
                    const meta: { [key: string]: string } = {};
                    metaMap.forEach((entry: any, key: any) => {
                        meta[key as string] = entry as string;
                    });
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

    public close() {
        this.client.close();
    }
}
