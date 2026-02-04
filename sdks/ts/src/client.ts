import * as grpc from '@grpc/grpc-js';
import { DatabaseClient } from './proto/hyperspace_grpc_pb';
import {
    InsertRequest, SearchRequest,
    CreateCollectionRequest, DeleteCollectionRequest, Empty
} from './proto/hyperspace_pb';

export class HyperspaceClient {
    private client: DatabaseClient;
    private metadata: grpc.Metadata;

    constructor(host: string = 'localhost:50051', apiKey?: string) {
        this.client = new DatabaseClient(host, grpc.credentials.createInsecure());
        this.metadata = new grpc.Metadata();
        if (apiKey) {
            this.metadata.add('x-api-key', apiKey);
        }
    }

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

    public insert(id: number, vector: number[], meta?: { [key: string]: string }, collection: string = ''): Promise<boolean> {
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

            this.client.insert(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                resolve(resp.getSuccess());
            });
        });
    }

    public search(vector: number[], topK: number, collection: string = ''): Promise<{ id: number, distance: number }[]> {
        return new Promise((resolve, reject) => {
            const req = new SearchRequest();
            req.setVectorList(vector);
            req.setTopK(topK);
            req.setCollection(collection);

            this.client.search(req, this.metadata, (err, resp) => {
                if (err) return reject(err);
                const results = resp.getResultsList().map(r => ({
                    id: r.getId(),
                    distance: r.getDistance()
                }));
                resolve(results);
            });
        });
    }

    public close() {
        this.client.close();
    }
}
