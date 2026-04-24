# HyperspaceDB Migration Worker

High-performance ETL pipeline for migrating vector data from various vector databases to HyperspaceDB.

## Features
- **Strategy Pattern**: Extensible architecture for adding new source databases.
- **Streaming**: Asynchronous generators ensure low memory footprint during large migrations.
- **Automatic Schema Mapping**: Detects dimension and metric from the source collection.
- **Robust Ingestion**: Built-in exponential backoff and retries.
- **Progress Monitoring**: Real-time progress reporting for dashboard integration.

## Supported Sources
- **Qdrant**: via `scroll()` API.
- **Pinecone**: via `listPaginated()` and `fetch()`.
- **Chroma**: via `get()` with limit/offset.
- **Weaviate**: via GraphQL Cursor API (`after`).
- **Milvus**: via `query()` with limit/offset.
- **HyperspaceDB (Legacy)**: Cross-instance or version migration.

## Quick Start

### Installation
```bash
cd integrations/migration-worker
npm install
```

### Usage Example
```typescript
import { 
    MigrationPipeline, 
    HyperspaceIngester, 
    QdrantExtractor 
} from './src';

async function main() {
    const source = new QdrantExtractor('http://qdrant-server:6333');
    const target = new HyperspaceIngester('http://hyperspace-server:50051');

    const pipeline = new MigrationPipeline(source, target, (progress) => {
        console.log(`Migrated ${progress.migratedVectors}/${progress.totalVectors} vectors...`);
    });

    await pipeline.run('source_collection', 'target_collection', 500);
}
```

## Architecture
The worker follows the **ETL (Extract, Transform, Load)** pattern:
1. **Extract**: `BaseExtractor` implementations fetch batches from source.
2. **Transform**: `MigrationPipeline` ensures IDs are compatible (hashes strings to uint32) and maps schemas.
3. **Load**: `HyperspaceIngester` handles batch insertion with retries.
