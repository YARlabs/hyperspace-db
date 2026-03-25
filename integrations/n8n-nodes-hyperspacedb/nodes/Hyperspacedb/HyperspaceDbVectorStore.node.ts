import {
	IExecuteFunctions,
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
	ISupplyDataFunctions,
	SupplyData,
} from 'n8n-workflow';

class HS_Store {
	constructor(private embeddings: any, private client: any, private col: string) {}
	_vectorstoreType() { return "hyperspace"; }
	async addDocuments(docs: any[]) { return docs.map(() => "ok"); }
    async similaritySearchVectorWithScore(q: any, k: any) {
        const res = await this.client.search(q, k, this.col);
        return res.map((r: any) => [{ pageContent: r.metadata.text || "", metadata: r.metadata }, r.distance]);
    }
}

export class HyperspaceDbVectorStore implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB Vector Store',
		name: 'hyperspaceDbVectorStore',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1,
		description: 'HyperspaceDB Hyperbolic Vector Store',
		defaults: { name: 'HyperspaceDB Vector Store' },
		inputs: [NodeConnectionTypes.Main, NodeConnectionTypes.AiEmbedding as any],
		outputs: [NodeConnectionTypes.Main],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		properties: [{ displayName: 'Collection', name: 'collectionName', type: 'string', default: '' }],
	};
	async supplyData(this: ISupplyDataFunctions, i: number): Promise<SupplyData> {
        const credentials = await this.getCredentials('hyperspacedbApi') as any;
        const { HyperspaceClient } = require('hyperspace-sdk-ts');
        const host = credentials.host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');
        const client = new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey) as any;
		const embeddings = await this.getInputConnectionData(NodeConnectionTypes.AiEmbedding, i) as any;
		return { response: new HS_Store(embeddings, client, this.getNodeParameter('collectionName', i) as string) };
	}
    async execute(this: IExecuteFunctions): Promise<any[][]> { return [this.getInputData()]; }
}
