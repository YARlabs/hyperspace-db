import express from 'express';
import cors from 'cors';
import { MigrationPipeline } from './MigrationPipeline';
import { HyperspaceIngester } from './ingesters/HyperspaceIngester';
import { QdrantExtractor } from './extractors/QdrantExtractor';
import { ChromaExtractor } from './extractors/ChromaExtractor';
import { PineconeExtractor } from './extractors/PineconeExtractor';
import { MigrationProgress } from './types';

const app = express();
app.use(cors());
app.use(express.json());

const activeMigrations = new Map<string, MigrationPipeline>();

app.post('/api/migration/start', async (req, res) => {
    const { 
        sourceType, 
        url, 
        apiKey, 
        sourceCollection, 
        targetCollection, 
        batchSize,
        hyperspaceUrl 
    } = req.body;

    const migrationId = `${Date.now()}-${sourceCollection}`;
    
    let extractor;
    switch (sourceType) {
        case 'qdrant': extractor = new QdrantExtractor(url, apiKey); break;
        case 'chroma': extractor = new ChromaExtractor(url); break;
        case 'pinecone': extractor = new PineconeExtractor(apiKey); break;
        default: return res.status(400).json({ error: 'Unsupported source type' });
    }

    const ingester = new HyperspaceIngester(hyperspaceUrl || 'http://localhost:50051');
    const pipeline = new MigrationPipeline(extractor, ingester);

    activeMigrations.set(migrationId, pipeline);

    // Fire and forget (it runs in background)
    pipeline.run(sourceCollection, targetCollection, batchSize).catch(console.error);

    res.json({ migrationId, status: 'started' });
});

app.get('/api/migration/status/:id', (req, res) => {
    const pipeline = activeMigrations.get(req.params.id);
    if (!pipeline) return res.status(404).json({ error: 'Migration not found' });

    res.json(pipeline.getProgress());
});

app.get('/api/migration/list', (req, res) => {
    const list = Array.from(activeMigrations.entries()).map(([id, p]) => ({
        id,
        progress: p.getProgress()
    }));
    res.json(list);
});

const PORT = process.env.PORT || 3001;
app.listen(PORT, () => {
    console.log(`🚀 Migration Service running on port ${PORT}`);
});
