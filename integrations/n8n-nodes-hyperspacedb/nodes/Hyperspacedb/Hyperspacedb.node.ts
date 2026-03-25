import {
	IExecuteFunctions,
	INodeExecutionData,
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
} from 'n8n-workflow';
import { getHyperspaceClient } from './hyperspacedb-utils';
import { collectionDescription } from './resources/collection';
import { vectorDescription } from './resources/vector';
import { systemDescription } from './resources/system';
import { graphDescription } from './resources/graph';

export class Hyperspacedb implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB',
		name: 'hyperspaceDb',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1.1,
		subtitle: '={{$parameter["operation"] + ": " + $parameter["resource"]}}',
		description: 'Interact with the HyperspaceDB gRPC API',
		defaults: {
			name: 'HyperspaceDB',
		},
		usableAsTool: true,
		inputs: [NodeConnectionTypes.Main],
		outputs: [NodeConnectionTypes.Main],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		properties: [
			{
				displayName: 'Resource',
				name: 'resource',
				type: 'options',
				noDataExpression: true,
				options: [
					{
						name: 'Collection',
						value: 'collection',
					},
					{
						name: 'Vector',
						value: 'vector',
					},
					{
						name: 'System',
						value: 'system',
					},
					{
						name: 'Graph',
						value: 'graph',
					},
				],
				default: 'collection',
			},
			...collectionDescription,
			...vectorDescription,
			...systemDescription,
			...graphDescription,
		],
	};

	async execute(this: IExecuteFunctions): Promise<INodeExecutionData[][]> {
		const items = this.getInputData();
		const returnData: INodeExecutionData[] = [];
		const resource = this.getNodeParameter('resource', 0) as string;
		const operation = this.getNodeParameter('operation', 0) as string;

		const client = await getHyperspaceClient(this);

		for (let i = 0; i < items.length; i++) {
			try {
				if (resource === 'vector') {
					const collectionName = this.getNodeParameter('collectionName', i) as string;

					if (operation === 'insert') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;

						const vector = JSON.parse(vectorStr);
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insert(vector, id, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'insertText') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const text = this.getNodeParameter('textContent', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insertText(text, id, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'search') {
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const topK = this.getNodeParameter('topK', i) as number;
						const vector = JSON.parse(vectorStr);

						const results = await client.search(vector, topK, collectionName);
						returnData.push({ json: { results } });
					} else if (operation === 'searchText') {
						const text = this.getNodeParameter('textContent', i) as string;
						const topK = this.getNodeParameter('topK', i) as number;

						const results = await client.searchText(text, topK, collectionName);
						returnData.push({ json: { results } });
					}
				} else if (resource === 'collection') {
					if (operation === 'create') {
						const name = this.getNodeParameter('name', i) as string;
						const dimension = this.getNodeParameter('dimension', i) as number;
						const metric = this.getNodeParameter('metric', i) as string;

						await client.createCollection(name, dimension, metric);
						returnData.push({ json: { success: true, name } });
					} else if (operation === 'list') {
						const collections = await client.listCollections();
						returnData.push({ json: { collections } });
					} else if (operation === 'getStats') {
						const name = this.getNodeParameter('name', i) as string;
						const stats = await client.getDigest(name);
						returnData.push({ json: stats });
					} else if (operation === 'delete') {
						const name = this.getNodeParameter('name', i) as string;
						await client.deleteCollection(name);
						returnData.push({ json: { success: true, collection: name } });
					}
				} else if (resource === 'system') {
					if (operation === 'getStatus') {
						const status = await client.getDigest("");
						returnData.push({ json: status });
					}
				}
			} catch (error: any) {
				if (this.continueOnFail()) {
					returnData.push({ json: { error: error.message } });
					continue;
				}
				throw error;
			}
		}

		return [returnData];
	}
}

// @ts-ignore
exports.Hyperspacedb = Hyperspacedb;

