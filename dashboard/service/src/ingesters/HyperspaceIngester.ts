import { HyperspaceClient } from 'hyperspace-sdk-ts';
import { VectorData, CollectionSchema } from '../types';
import pRetry from 'p-retry';

export class HyperspaceIngester {
    private client: HyperspaceClient;

    constructor(endpoint: string, apiKey?: string) {
        this.client = new HyperspaceClient(endpoint, apiKey);
    }

    async prepareTarget(collectionName: string, schema: CollectionSchema) {
        try {
            const collections = await this.client.listCollections();
            const exists = collections.find(c => c.name === collectionName);

            if (exists) {
                console.log(`Collection ${collectionName} already exists. Skipping creation.`);
                return;
            }

            console.log(`Creating collection ${collectionName} with dim=${schema.dimension}, metric=${schema.metric}`);
            await this.client.createCollection(
                collectionName, 
                schema.dimension, 
                schema.metric === 'ip' ? 'cosine' : schema.metric // Map IP to cosine if not directly supported
            );
        } catch (error) {
            console.error(`Failed to prepare target collection: ${error}`);
            throw error;
        }
    }

    async ingestBatch(collectionName: string, batch: VectorData[]) {
        const items = batch.map(item => ({
            id: typeof item.id === 'string' ? this.hashStringToInt(item.id) : item.id,
            vector: item.vector,
            metadata: item.metadata
        }));

        await pRetry(
            async () => {
                await this.client.batchInsert(items, collectionName);
            },
            {
                retries: 5,
                onFailedAttempt: error => {
                    console.warn(`Ingest attempt ${error.attemptNumber} failed. ${error.retriesLeft} retries left.`);
                }
            }
        );
    }

    private hashStringToInt(s: string): number {
        let hash = 0;
        for (let i = 0; i < s.length; i++) {
            const char = s.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32bit integer
        }
        return Math.abs(hash);
    }
}
