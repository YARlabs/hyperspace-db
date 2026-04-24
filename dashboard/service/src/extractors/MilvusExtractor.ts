import { MilvusClient } from '@zilliz/milvus2-sdk-node';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class MilvusExtractor extends BaseExtractor {
    private client: MilvusClient;

    constructor(address: string, username?: string, password?: string) {
        super();
        this.client = new MilvusClient({ address, username, password });
    }

    async connect(): Promise<void> {
        await this.client.checkHealth();
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const desc = await this.client.describeCollection({ collection_name: collectionName });
        const vectorField = desc.schema.fields.find(f => f.data_type === 'FloatVector');
        
        const dimStr = vectorField?.type_params.find(p => p.key === 'dim')?.value;
        const dimension = dimStr ? parseInt(dimStr) : 0;

        const stats = await this.client.getCollectionStatistics({ collection_name: collectionName });
        const count = parseInt(stats.stats.find(s => s.key === 'row_count')?.value || '0');

        return {
            dimension,
            metric: 'cosine', // Defaulting for example
            count
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        let offset = 0;
        
        while (true) {
            // Using query with offset/limit
            const res = await this.client.query({
                collection_name: collectionName,
                limit: batchSize,
                offset: offset,
                output_fields: ['*'],
                // Milvus requires a filter for query, using 'id >= 0' or similar
                filter: 'id >= 0' 
            });

            if (!res.data || res.data.length === 0) break;

            const batch: VectorData[] = res.data.map((item: any) => ({
                id: item.id,
                vector: item.vector, // Ensure vector field is included in output_fields
                metadata: item
            }));

            yield batch;
            offset += res.data.length;
            if (res.data.length < batchSize) break;
        }
    }

    async disconnect(): Promise<void> {
        await this.client.closeConnection();
    }
}
