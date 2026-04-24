import { Pinecone } from '@pinecone-database/pinecone';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class PineconeExtractor extends BaseExtractor {
    private client: Pinecone;

    constructor(apiKey: string) {
        super();
        this.client = new Pinecone({ apiKey });
    }

    async connect(): Promise<void> {
        await this.client.listIndexes();
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const index = this.client.index(collectionName);
        const stats = await index.describeIndexStats();
        
        // We need to get the actual index metadata for dimension
        const indexList = await this.client.listIndexes();
        const meta = indexList.indexes?.find(i => i.name === collectionName);

        return {
            dimension: meta?.dimension || 0,
            metric: (meta?.metric as any) || 'cosine',
            count: stats.totalRecordCount
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        const index = this.client.index(collectionName);
        
        // Pinecone doesn't have a simple 'scroll all'. 
        // Newer versions have 'listPaginated', older ones require 'list' or querying with filters.
        // Assuming modern Pinecone SDK usage.
        
        let paginationToken: string | undefined = undefined;

        while (true) {
            const listResult = await index.listPaginated({ 
                limit: batchSize, 
                paginationToken 
            });

            if (!listResult.vectors || listResult.vectors.length === 0) break;

            const ids = listResult.vectors.map(v => v.id!);
            const fetchResult = await index.fetch(ids);

            const batch: VectorData[] = Object.values(fetchResult.records).map(r => ({
                id: r.id,
                vector: r.values,
                metadata: r.metadata || {}
            }));

            yield batch;

            paginationToken = listResult.pagination?.next;
            if (!paginationToken) break;
        }
    }

    async disconnect(): Promise<void> {
        // No explicit disconnect
    }
}
