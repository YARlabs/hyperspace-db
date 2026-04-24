import { MigrationPipeline } from './MigrationPipeline';
import { HyperspaceIngester } from './ingesters/HyperspaceIngester';
import { QdrantExtractor } from './extractors/QdrantExtractor';
import { ChromaExtractor } from './extractors/ChromaExtractor';
import { PineconeExtractor } from './extractors/PineconeExtractor';

async function runMigration() {
    console.log('🚀 Starting Migration Pipeline...');

    // 1. Setup Source (Example: Qdrant)
    const source = new QdrantExtractor('http://localhost:6333');
    
    // 2. Setup Target (HyperspaceDB)
    const target = new HyperspaceIngester('http://localhost:50051', 'MY_API_KEY');

    // 3. Setup Pipeline
    const pipeline = new MigrationPipeline(source, target, (progress) => {
        const percent = progress.totalVectors > 0 
            ? ((progress.migratedVectors / progress.totalVectors) * 100).toFixed(2)
            : '0';
        
        console.log(`[Progress] ${percent}% | Migrated: ${progress.migratedVectors}/${progress.totalVectors} | Batches: ${progress.batchesProcessed}`);
        
        if (progress.status === 'failed') {
            console.error(`❌ Migration failed: ${progress.error}`);
        }
    });

    // 4. Execute
    try {
        await pipeline.run('old_qdrant_collection', 'new_hyperspace_collection', 500);
        console.log('✅ Migration completed successfully!');
    } catch (error) {
        console.error('💥 Fatal error during migration:', error);
    }
}

// Check for command line arguments or environment variables if needed
if (require.main === module) {
    runMigration();
}
