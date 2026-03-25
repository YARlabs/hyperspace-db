import {
	IExecuteFunctions,
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
	ISupplyDataFunctions,
	SupplyData,
	ILoadOptionsFunctions,
	INodePropertyOptions,
} from 'n8n-workflow';
import { getHyperspaceClient } from './HyperspaceDb.utils';
import { HyperspaceStore } from './HyperspaceStore';

export class HyperspaceDbVectorStore implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB Vector Store',
		name: 'hyperspaceDbVectorStore',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1,
		description: 'The world\'s first Hyperbolic Vector Database integration for Spatial AI',
		defaults: {
			name: 'HyperspaceDB Vector Store',
		},
		codex: {
			categories: ['AI'],
			subcategories: {
				AI: ['Vector Stores', 'Root Nodes'],
			},
			resources: {
				primaryDocumentation: [
					{
						url: 'https://docs.yar.ink',
					},
				],
			},
		},
		inputs: [NodeConnectionTypes.Main, NodeConnectionTypes.AiEmbedding as any],
		outputs: [NodeConnectionTypes.Main],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		properties: [
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
				default: 'cosine',
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
	};

	async supplyData(this: ISupplyDataFunctions, itemIndex: number): Promise<SupplyData> {
		const collectionName = this.getNodeParameter('collectionName', itemIndex) as string;
		const metric = this.getNodeParameter('metric', itemIndex, 'lorentz') as 'lorentz' | 'poincare' | 'cosine' | 'l2';
		const dimension = this.getNodeParameter('dimension', itemIndex, 1024) as number;

		const client = await getHyperspaceClient(this as any);
		const embeddings = (await this.getInputConnectionData(
			NodeConnectionTypes.AiEmbedding,
			itemIndex,
		)) as any;

		return {
			response: new HyperspaceStore(embeddings, {
				client: client as any,
				collectionName,
				dimension,
				metric,
			}),
		};
	}

	async execute(this: IExecuteFunctions): Promise<any[][]> {
		return [this.getInputData()];
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
