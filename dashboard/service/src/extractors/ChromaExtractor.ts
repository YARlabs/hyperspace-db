import { ChromaClient } from 'chromadb';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class ChromaExtractor extends BaseExtractor {
    private client: ChromaClient;

    constructor(path: string) {
        super();
        this.client = new ChromaClient({ path });
    }

    async connect(): Promise<void> {
        await this.client.heartbeat();
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const collection = await this.client.getCollection({ name: collectionName });
        const count = await collection.count();
        
        // Chroma metadata usually doesn't explicitly store dimension in the schema call, 
        // we might need to fetch one record to check.
        const sample = await collection.get({ limit: 1, include: ['embeddings' as any] });
        const dimension = sample.embeddings?.[0]?.length || 0;

        return {
            dimension,
            metric: 'cosine', // Chroma default
            count
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        const collection = await this.client.getCollection({ name: collectionName });
        const total = await collection.count();
        let offset = 0;

        while (offset < total) {
            const result = await collection.get({
                limit: batchSize,
                offset: offset,
                include: ['embeddings', 'metadatas', 'documents'] as any
            });

            if (!result.ids.length) break;

            const batch: VectorData[] = result.ids.map((id, i) => ({
                id,
                vector: result.embeddings?.[i] as number[],
                metadata: {
                    ...(result.metadatas?.[i] as object),
                    document: result.documents?.[i]
                }
            }));

            yield batch;
            offset += result.ids.length;
        }
    }

    async disconnect(): Promise<void> {
        // No explicit disconnect
    }
}
