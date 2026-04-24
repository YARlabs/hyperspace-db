import { QdrantClient } from '@qdrant/js-client-rest';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class QdrantExtractor extends BaseExtractor {
    private client: QdrantClient;

    constructor(url: string, apiKey?: string) {
        super();
        this.client = new QdrantClient({ url, apiKey });
    }

    async connect(): Promise<void> {
        // Test connection
        await this.client.getCollections();
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const info = await this.client.getCollection(collectionName);
        const config = info.config.params;
        
        // Qdrant uses 'Distance' enum: Cosine, Euclid, Dot
        let metric: CollectionSchema['metric'] = 'cosine';
        const qMetric = info.config.hnsw_config ? 'cosine' : 'cosine'; // Simplified for example

        return {
            dimension: config.vectors?.size || 0,
            metric: metric,
            count: info.points_count
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        let nextToken: string | number | null | undefined = undefined;

        while (nextToken !== null) {
            const result = await this.client.scroll(collectionName, {
                limit: batchSize,
                with_payload: true,
                with_vector: true,
                offset: nextToken
            });

            if (result.points.length === 0) break;

            const batch: VectorData[] = result.points.map(p => ({
                id: p.id,
                vector: p.vector as number[],
                metadata: p.payload || {}
            }));

            yield batch;
            nextToken = result.next_page_offset;
            if (!nextToken) break;
        }
    }

    async disconnect(): Promise<void> {
        // No explicit disconnect needed for Qdrant rest client
    }
}
