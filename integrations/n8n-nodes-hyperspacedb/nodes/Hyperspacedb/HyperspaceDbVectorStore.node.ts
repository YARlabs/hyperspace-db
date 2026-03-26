import {
    createVectorStoreNode,
    N8nBinaryLoader,
    N8nJsonLoader,
} from '@n8n/ai-utilities';
import {
    IExecuteFunctions,
    ISupplyDataFunctions,
    ILoadOptionsFunctions,
    INodePropertyOptions,
    INodeType,
    INodeTypeDescription,
} from 'n8n-workflow';
import {
    getHyperspaceClient,
} from './HyperspaceDb.utils';
import { HyperspaceStore } from './HyperspaceStore';

/**
 * FIX FOR n8n Community Nodes `processedDocuments.map is not a function` error.
 * n8n executes community nodes in an isolated VM context. This causes `instanceof` 
 * checks to fail when comparing objects created by core n8n nodes (like Default Data Loader) 
 * against the classes loaded in the community node. We override the `Symbol.hasInstance` 
 * to use Duck Typing so that internal `processDocuments.js` succeeds.
 */
const duckTypeCheck = (instance: any) =>
    instance && typeof instance === 'object' && 'processItem' in instance && 'processAll' in instance;

Object.defineProperty(N8nBinaryLoader, Symbol.hasInstance, { value: duckTypeCheck });
Object.defineProperty(N8nJsonLoader, Symbol.hasInstance, { value: duckTypeCheck });

const VectorStoreNodeClass = createVectorStoreNode({
    meta: {
        displayName: 'HyperspaceDB Vector Store',
        name: 'hyperspaceDbVectorStore',
        description: 'The world\'s first Hyperbolic Vector Database integration for Spatial AI',
        icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' } as any,
        docsUrl: 'https://yar.ink/docs',
        categories: ['AI'],
        subcategories: {
            AI: ['Vector Stores', 'Root Nodes'],
        },
        credentials: [{ name: 'hyperspacedbApi', required: true }],
    },
    sharedFields: [
        {
            displayName: 'Collection Name',
            name: 'collectionName',
            type: 'options',
            typeOptions: {
                loadOptionsMethod: 'getCollections',
            },
            default: '',
            required: true,
        },
        {
            displayName: 'Metric / Geometry',
            name: 'metric',
            type: 'options',
            typeOptions: {
                loadOptionsMethod: 'getCollectionMetric',
                loadOptionsDependsOn: ['collectionName'],
            },
            default: 'lorentz',
            description: 'The spatial metric used - auto-fetched from collection',
        },
        {
            displayName: 'Dimension',
            name: 'dimension',
            type: 'options',
            typeOptions: {
                loadOptionsMethod: 'getCollectionDimension',
                loadOptionsDependsOn: ['collectionName'],
            },
            default: 1024,
            description: 'Number of dimensions - auto-fetched from collection',
        },
    ],
    retrieveFields: [
        {
            displayName: 'Name',
            name: 'toolName',
            type: 'string',
            default: 'hyperspace_vector_store',
            required: true,
            description: 'Name of the vector store tool for the AI Agent',
            placeholder: 'e.g. company_knowledge_base',
            displayOptions: {
                show: {
                    mode: ['retrieve-as-tool'],
                },
            },
        },
    ],
    async getVectorStoreClient(context: IExecuteFunctions | ISupplyDataFunctions, _filter: any, _embeddings: any, itemIndex: number) {
        const collectionName = context.getNodeParameter('collectionName', itemIndex) as string;
        const metric = context.getNodeParameter('metric', itemIndex, 'lorentz') as string;
        const dimension = context.getNodeParameter('dimension', itemIndex, 1024) as number;

        const client = await getHyperspaceClient(context);
        return new HyperspaceStore(_embeddings, {
            client,
            collectionName,
            dimension,
            metric,
        }) as any;
    },
    async populateVectorStore(context: IExecuteFunctions | ISupplyDataFunctions, _embeddings: any, documents: any[], itemIndex: number) {
        const collectionName = context.getNodeParameter('collectionName', itemIndex) as string;
        const metric = context.getNodeParameter('metric', itemIndex, 'lorentz') as string;
        const dimension = context.getNodeParameter('dimension', itemIndex, 1024) as number;

        const client = await getHyperspaceClient(context);
        const store = new HyperspaceStore(_embeddings, {
            client,
            collectionName,
            dimension,
            metric,
        });
        await store.addDocuments(documents);
    },
});

export class HyperspaceDbVectorStore extends (VectorStoreNodeClass as any) implements INodeType {
    declare description: INodeTypeDescription;

    methods = {
        loadOptions: {
            async getCollections(this: ILoadOptionsFunctions): Promise<INodePropertyOptions[]> {
                try {
                    const client = await getHyperspaceClient(this as any);
                    const collections = (await client.listCollections()) as any[];
                    return collections.map((col) => ({
                        name: `${col.name} (dim: ${col.dimension}, metric: ${col.metric})`,
                        value: col.name,
                    }));
                } catch (error: any) {
                    throw new Error(`Failed to load collections: ${error.message}`);
                }
            },
            async getCollectionMetric(this: ILoadOptionsFunctions): Promise<INodePropertyOptions[]> {
                try {
                    const collectionName = this.getCurrentNodeParameter('collectionName') as string;
                    if (!collectionName) {
                        return [
                            { name: 'Lorentz (Hyperbolic)', value: 'lorentz' },
                            { name: 'Poincaré (Hyperbolic)', value: 'poincare' },
                            { name: 'Cosine Similarity', value: 'cosine' },
                            { name: 'Euclidean (L2)', value: 'l2' },
                        ];
                    }
                    const client = await getHyperspaceClient(this as any);
                    const collections = (await client.listCollections()) as any[];
                    const col = collections.find((c) => c.name === collectionName);
                    if (col) {
                        return [{ name: col.metric, value: col.metric }];
                    }
                    return [
                        { name: 'Lorentz (Hyperbolic)', value: 'lorentz' },
                        { name: 'Poincaré (Hyperbolic)', value: 'poincare' },
                        { name: 'Cosine Similarity', value: 'cosine' },
                        { name: 'Euclidean (L2)', value: 'l2' },
                    ];
                } catch (error: any) {
                    return [{ name: 'lorentz', value: 'lorentz' }];
                }
            },
            async getCollectionDimension(this: ILoadOptionsFunctions): Promise<INodePropertyOptions[]> {
                try {
                    const collectionName = this.getCurrentNodeParameter('collectionName') as string;
                    if (!collectionName) return [{ name: '1024', value: 1024 }];
                    const client = await getHyperspaceClient(this as any);
                    const collections = (await client.listCollections()) as any[];
                    const col = collections.find((c) => c.name === collectionName);
                    if (col) {
                        return [{ name: String(col.dimension), value: col.dimension }];
                    }
                    return [{ name: '1024', value: 1024 }];
                } catch (error: any) {
                    return [{ name: '1024', value: 1024 }];
                }
            },
        },
    };

    constructor() {
        super();
        this.description.usableAsTool = true;
    }
}



