import { HyperspaceClient } from 'hyperspace-sdk-ts';
import { BaseExtractor } from './BaseExtractor';
import { VectorData, CollectionSchema } from '../types';

export class LegacyHyperspaceExtractor extends BaseExtractor {
    private client: HyperspaceClient;

    constructor(endpoint: string, apiKey?: string) {
        super();
        this.client = new HyperspaceClient(endpoint, apiKey);
    }

    async connect(): Promise<void> {
        await this.client.listCollections();
    }

    async inspectSchema(collectionName: string): Promise<CollectionSchema> {
        const collections = await this.client.listCollections();
        const info = collections.find(c => c.name === collectionName);
        
        if (!info) throw new Error(`Collection ${collectionName} not found`);

        return {
            dimension: info.dimension,
            metric: info.metric as any,
            count: info.count
        };
    }

    async *streamBatches(collectionName: string, batchSize: number): AsyncGenerator<VectorData[]> {
        // HyperspaceDB export API (simulated via search if no explicit export exists, 
        // but typically a vector store should have a way to scroll).
        // Since HyperspaceDB handles IDs explicitly, we can use id ranges or offset-based search.
        
        // This is a placeholder for actual export API if available, 
        // otherwise we fall back to a "scroll" like behavior.
        
        // For this example, we'll assume a 'scroll' or similar paginated read exists.
        // In HyperspaceDB, we can use the Search API with high top_k and filters if needed, 
        // but for a true extractor, we'd use a more efficient internal stream.
        
        console.warn('Hyperspace-to-Hyperspace migration uses simulated scrolling.');
        
        // Placeholder for real streaming logic
        let offset = 0;
        while (true) {
            // Simulated: Fetching by ID ranges or offset
            const results = await (this.client as any).exportBatch(collectionName, offset, batchSize);
            if (!results || results.length === 0) break;
            
            yield results.map((r: any) => ({
                id: r.id,
                vector: r.vector,
                metadata: r.metadata
            }));
            
            offset += results.length;
            if (results.length < batchSize) break;
        }
    }

    async disconnect(): Promise<void> {
        // Close client if necessary
    }
}
