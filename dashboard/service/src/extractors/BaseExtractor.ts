import { VectorData, CollectionSchema } from './types';

export abstract class BaseExtractor {
    /**
     * Connect to the source database
     */
    abstract connect(): Promise<void>;

    /**
     * Get collection schema (dimension, metric, count)
     */
    abstract inspectSchema(collectionName: string): Promise<CollectionSchema>;

    /**
     * Stream batches of vectors as an async generator
     * @param collectionName Name of the collection to extract
     * @param batchSize Number of vectors per batch
     */
    abstract streamBatches(
        collectionName: string, 
        batchSize: number
    ): AsyncGenerator<VectorData[]>;

    /**
     * Close connection to the source database
     */
    abstract disconnect(): Promise<void>;
}
