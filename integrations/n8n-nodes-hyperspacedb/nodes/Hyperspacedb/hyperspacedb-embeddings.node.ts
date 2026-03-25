import {
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
	ISupplyDataFunctions,
	SupplyData,
	ILoadOptionsFunctions,
	INodePropertyOptions,
} from 'n8n-workflow';
import { getHyperspaceClient } from './hyperspacedb-utils';
import { Embeddings, EmbeddingsParams } from '@langchain/core/embeddings';

// LangChain wrapper for Hyperspace Server-side embeddings
class HyperspaceEmbeddings extends Embeddings {
	private client: any;
	private metric: string;

	constructor(fields: EmbeddingsParams & { client: any; metric: string }) {
		super(fields);
		this.client = fields.client;
		this.metric = fields.metric;
	}

	async embedDocuments(documents: string[]): Promise<number[][]> {
		const result: number[][] = [];
		for (const text of documents) {
			const vector = await this.client.vectorize(text, this.metric);
			result.push(vector);
		}
		return result;
	}

	async embedQuery(document: string): Promise<number[]> {
		return await this.client.vectorize(document, this.metric);
	}
}

export class HyperspaceDbEmbeddings implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB Embeddings',
		name: 'hyperspaceDbEmbeddings',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1,
		description: 'HyperspaceDB Native Hyperbolic Embeddings',
		defaults: {
			name: 'HyperspaceDB Embeddings',
		},
		codex: {
			categories: ['AI'],
			subcategories: {
				AI: ['Embeddings'],
			},
		},
		inputs: [],
		outputs: [
			{
				displayName: 'Embeddings',
				maxConnections: 1,
				type: NodeConnectionTypes.AiEmbedding,
			},
		],
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
				description: 'The collection you are working with',
			},
			{
				displayName: 'Model Geometry',
				name: 'metric',
				type: 'options',
				options: [
					{ name: 'Cosine (Euclidean)', value: 'cosine' },
					{ name: 'L2 (Euclidean)', value: 'l2' },
					{ name: 'Poincaré (Hyperbolic)', value: 'poincare' },
					{ name: 'Lorentz (Hyperbolic)', value: 'lorentz' },
				],
				default: 'cosine',
				description: 'The geometry space for vectorization',
			},
		],
	};

	async supplyData(this: ISupplyDataFunctions, itemIndex: number): Promise<SupplyData> {
		const metric = this.getNodeParameter('metric', itemIndex) as string;
		const client = await getHyperspaceClient(this);

		const embeddings = new HyperspaceEmbeddings({
			client,
			metric,
		});

		return {
			response: embeddings
		};
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

// @ts-ignore
exports.HyperspaceDbEmbeddings = HyperspaceDbEmbeddings;

