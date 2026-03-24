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
} from 'n8n-workflow';
import { getHyperspaceStore } from './HyperspaceDb.utils';

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
                docsUrl: 'https://docs.hyperspace.systems',
                categories: ['AI'],
                subcategories: {
                    AI: ['Vector Stores', 'Root Nodes'],
                },
            },
            sharedFields: [
                {
                    displayName: 'Collection Name',
                    name: 'collectionName',
                    type: 'string',
                    default: 'default',
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
                    description: 'The spatial metric used in your collection',
                },
                {
                    displayName: 'Dimension',
                    name: 'dimension',
                    type: 'number',
                    default: 1536,
                    description: 'Number of dimensions (OpenAI=1536, Hyperspace DEFAULT=1024)',
                },
            ],
            async getVectorStoreClient(context: IExecuteFunctions | ISupplyDataFunctions, _filter: any, _embeddings: any, itemIndex: number) {
                return await getHyperspaceStore(context, itemIndex) as any;
            },
            async populateVectorStore(context: IExecuteFunctions | ISupplyDataFunctions, _embeddings: any, documents: any[], itemIndex: number) {
                const store = await getHyperspaceStore(context, itemIndex);
                await store.addDocuments(documents);
            },
        });

        const instance = new VectorStoreNodeClass();
        instance.description.usableAsTool = true;
        this.description = instance.description;
        this.execute = instance.execute;
        this.supplyData = instance.supplyData;
    }
}
