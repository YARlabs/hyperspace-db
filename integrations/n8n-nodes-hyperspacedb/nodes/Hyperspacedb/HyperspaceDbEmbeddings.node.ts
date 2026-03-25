import {
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
	ISupplyDataFunctions,
	SupplyData,
} from 'n8n-workflow';

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
		properties: [{ displayName: 'Metric', name: 'metric', type: 'string', default: 'lorentz' }],
	};
	async supplyData(this: ISupplyDataFunctions, i: number): Promise<SupplyData> {
        const credentials = await this.getCredentials('hyperspacedbApi') as any;
        const { HyperspaceClient } = require('hyperspace-sdk-ts');
        const host = credentials.host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');
        const client = new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey) as any;
		return { response: new HS_Embeddings(client, this.getNodeParameter('metric', i) as string) };
	}
}
