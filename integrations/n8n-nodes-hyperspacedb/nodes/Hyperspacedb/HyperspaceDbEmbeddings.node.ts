import {
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
	ISupplyDataFunctions,
	SupplyData,
} from 'n8n-workflow';
import {
	getHyperspaceClient,
} from './HyperspaceDb.utils';

class HS_Embeddings {
    constructor(private client: any, private metric: string) {}
    async embedDocuments(docs: string[]) {
        const res = [];
        for (const t of docs) res.push(await this.client.vectorize(t, this.metric));
        return res;
    }
    async embedQuery(t: string) { return await this.client.vectorize(t, this.metric); }
}

export class HyperspaceDbEmbeddings implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB Embeddings',
		name: 'hyperspaceDbEmbeddings',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1,
		description: 'HyperspaceDB Native Hyperbolic Embeddings',
		defaults: { name: 'HyperspaceDB Embeddings' },
		inputs: [],
		outputs: [{ displayName: 'Embeddings', maxConnections: 1, type: NodeConnectionTypes.AiEmbedding }],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		properties: [
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
				description: 'The spatial metric used for embeddings',
			},
		],
	};
	async supplyData(this: ISupplyDataFunctions, i: number): Promise<SupplyData> {
        const client = await getHyperspaceClient(this);
		return { response: new HS_Embeddings(client, this.getNodeParameter('metric', i) as string) };
	}
}

