import axios from 'axios';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class WeaviateExtractor extends BaseExtractor {
    constructor(private url: string, private apiKey?: string) {
        super();
    }

    async connect(): Promise<void> {
        await axios.get(`${this.url}/v1/meta`);
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const res = await axios.get(`${this.url}/v1/schema/${collectionName}`);
        const schema = res.data;
        
        // Weaviate doesn't return dimension in schema usually, 
        // need to check meta or fetch one record
        const sampleRes = await this.queryWeaviate(`
            {
                Get {
                    ${collectionName}(limit: 1) {
                        _additional {
                            vector
                        }
                    }
                }
            }
        `);
        
        const dimension = sampleRes.data.Get[collectionName][0]?._additional.vector.length || 0;

        return {
            dimension,
            metric: 'cosine',
            count: 0 // Weaviate doesn't return count easily in schema
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        let after: string | null = null;

        while (true) {
            const query = `
                {
                    Get {
                        ${collectionName}(
                            limit: ${batchSize}
                            ${after ? `after: "${after}"` : ''}
                        ) {
                            _additional {
                                id
                                vector
                            }
                            [ALL_PROPERTIES_PLACEHOLDER]
                        }
                    }
                }
            `;
            
            // In a real implementation, we'd fetch actual property names
            const res = await this.queryWeaviate(query);
            const objects = res.data.Get[collectionName];

            if (!objects || objects.length === 0) break;

            const batch: VectorData[] = objects.map((obj: any) => ({
                id: obj._additional.id,
                vector: obj._additional.vector,
                metadata: obj
            }));

            yield batch;
            after = objects[objects.length - 1]._additional.id;
        }
    }

    private async queryWeaviate(query: string) {
        const res = await axios.post(`${this.url}/v1/graphql`, { query }, {
            headers: this.apiKey ? { 'Authorization': `Bearer ${this.apiKey}` } : {}
        });
        return res.data;
    }

    async disconnect(): Promise<void> {}
}
