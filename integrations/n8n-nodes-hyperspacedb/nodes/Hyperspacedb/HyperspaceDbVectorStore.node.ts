import {
    createVectorStoreNode,
} from '@n8n/ai-utilities';
import {
    IExecuteFunctions,
    INodeType,
    INodeTypeDescription,
    ISupplyDataFunctions,
    SupplyData,
    INodeExecutionData,
    ILoadOptionsFunctions,
    INodePropertyOptions,
} from 'n8n-workflow';
import {
    getHyperspaceClient,
} from './HyperspaceDb.utils';
import { HyperspaceStore } from './HyperspaceStore';

export class HyperspaceDbVectorStore implements INodeType {
    description: INodeTypeDescription;
    execute: (this: IExecuteFunctions) => Promise<INodeExecutionData[][]>;
    supplyData: (this: ISupplyDataFunctions, itemIndex: number) => Promise<SupplyData>;

    constructor() {
        const VectorStoreNodeClass = createVectorStoreNode({
            meta: {
                displayName: 'HyperspaceDB Vector Store',
                name: 'hyperspaceDbVectorStore',
                description: 'The world\'s first Hyperbolic Vector Database integration for Spatial AI',
                icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' } as any,
                docsUrl: 'https://docs.yar.ink',
                categories: ['AI'],
                subcategories: {
                    AI: ['Vector Stores', 'Root Nodes'],
                },
                credentials: [{ name: 'hyperspaceDbApi', required: true }],
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
                    options: [
                        { name: 'Lorentz (Hyperbolic)', value: 'lorentz' },
                        { name: 'Poincaré (Hyperbolic)', value: 'poincare' },
                        { name: 'Cosine Similarity', value: 'cosine' },
                        { name: 'Euclidean (L2)', value: 'l2' },
                    ],
                    default: 'lorentz',
                    description: 'The spatial metric used - should match your collection settings',
                },
                {
                    displayName: 'Dimension',
                    name: 'dimension',
                    type: 'number',
                    default: 1024,
                    description: 'Number of dimensions (should match your collection settings)',
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

        const instance = new VectorStoreNodeClass();
        instance.description.usableAsTool = true;
        this.description = instance.description;
        this.execute = instance.execute;
        this.supplyData = instance.supplyData;
    }

    methods = {
        loadOptions: {
            async getCollections(this: ILoadOptionsFunctions): Promise<INodePropertyOptions[]> {
                try {
                    const client = await getHyperspaceClient(this as any);
                    const collections = await client.listCollections();
                    return collections.map((name) => ({
                        name,
                        value: name,
                    }));
                } catch (error: any) {
                    throw new Error(`Failed to load collections: ${error.message}`);
                }
            },
        },
    };
}

