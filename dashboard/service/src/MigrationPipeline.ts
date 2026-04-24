import { BaseExtractor } from './extractors/BaseExtractor';
import { HyperspaceIngester } from './ingesters/HyperspaceIngester';
import { MigrationProgress, ProgressCallback } from './types';

export class MigrationPipeline {
    private progress: MigrationProgress = {
        totalVectors: 0,
        migratedVectors: 0,
        batchesProcessed: 0,
        status: 'idle'
    };

    constructor(
        private extractor: BaseExtractor,
        private ingester: HyperspaceIngester,
        private onProgress?: ProgressCallback
    ) {}

    async run(sourceCollection: string, targetCollection: string, batchSize: number = 1000) {
        this.updateStatus('running');
        this.progress.startTime = new Date();

        try {
            // 1. Connect
            await this.extractor.connect();

            // 2. Inspect Schema
            const schema = await this.extractor.inspectSchema(sourceCollection);
            this.progress.totalVectors = schema.count || 0;
            this.reportProgress();

            // 3. Prepare Target
            await this.ingester.prepareTarget(targetCollection, schema);

            // 4. Stream and Ingest
            for await (const batch of this.extractor.streamBatches(sourceCollection, batchSize)) {
                await this.ingester.ingestBatch(targetCollection, batch);
                
                this.progress.migratedVectors += batch.length;
                this.progress.batchesProcessed += 1;
                this.reportProgress();
            }

            this.updateStatus('completed');
            this.progress.endTime = new Date();
            this.reportProgress();

        } catch (error: any) {
            this.updateStatus('failed', error.message);
            throw error;
        } finally {
            await this.extractor.disconnect();
        }
    }

    private updateStatus(status: MigrationProgress['status'], error?: string) {
        this.progress.status = status;
        this.progress.error = error;
    }

    private reportProgress() {
        if (this.onProgress) {
            this.onProgress({ ...this.progress });
        }
    }

    getProgress(): MigrationProgress {
        return { ...this.progress };
    }
}
