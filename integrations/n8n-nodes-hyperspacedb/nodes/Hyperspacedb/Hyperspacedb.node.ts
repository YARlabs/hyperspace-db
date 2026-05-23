import {
	IExecuteFunctions,
	INodeExecutionData,
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
} from 'n8n-workflow';
import { HyperspaceClient } from 'hyperspace-sdk-ts';
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
		defaults: { name: 'HyperspaceDB' },
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
					{ name: 'Collection', value: 'collection' },
					{ name: 'Vector', value: 'vector' },
					{ name: 'System', value: 'system' },
					{ name: 'Graph', value: 'graph' },
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
		const credentials = await this.getCredentials('hyperspacedbApi') as any;
		const host = credentials.host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');
		const client = new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey) as any;

		const resource = this.getNodeParameter('resource', 0) as string;
		const operation = this.getNodeParameter('operation', 0) as string;

		for (let i = 0; i < items.length; i++) {
			try {
				if (resource === 'vector') {
					const collectionName = this.getNodeParameter('collectionName', i) as string;

					if (operation === 'insert') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;

						const vector = typeof vectorStr === 'string' ? JSON.parse(vectorStr) : vectorStr;
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insert(vector, id, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'insertText') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const text = this.getNodeParameter('textContent', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insertText(id, text, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'search') {
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const topK = this.getNodeParameter('topK', i) as number;
						const vector = typeof vectorStr === 'string' ? JSON.parse(vectorStr) : vectorStr;

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
						
						let httpStats: any = {};
						try {
							const clientHost = (client as any).host || "";
							const ip = clientHost.split(':')[0];
							const url = `http://${ip}:50050/api/collections/${name}/stats`;
							const headers: Record<string, string> = {};
							if ((client as any).apiKey) headers['x-api-key'] = (client as any).apiKey;
							if ((client as any).userId) headers['x-hyperspace-user-id'] = (client as any).userId;

							const res = await fetch(url, { headers });
							if (res.ok) {
								httpStats = await res.json();
							}
						} catch (e) {
							console.error('Failed to fetch HTTP stats for collection:', e);
						}

						const digest = await client.getDigest(name);
						const stats = await client.getCollectionStats(name);
						
						returnData.push({
							json: {
								...digest,
								...stats,
								...httpStats,
							},
						});
					} else if (operation === 'delete') {
						const name = this.getNodeParameter('name', i) as string;
						await client.deleteCollection(name);
						returnData.push({ json: { success: true, collection: name } });
					} else if (operation === 'cacheStats') {
						const name = this.getNodeParameter('name', i) as string;
						const cacheStats = await client.getCacheStats(name);
						returnData.push({ json: cacheStats });
					} else if (operation === 'cacheClear') {
						const name = this.getNodeParameter('name', i) as string;
						const success = await client.clearCache(name);
						returnData.push({ json: { success } });
					} else if (operation === 'cacheConfig') {
						const name = this.getNodeParameter('name', i) as string;
						const policy = this.getNodeParameter('policy', i) as string;
						const annThreshold = this.getNodeParameter('annThreshold', i) as number;
						const success = await client.updateCacheConfig(name, policy, annThreshold);
						returnData.push({ json: { success } });
					}
				} else if (resource === 'system') {
					if (operation === 'getStatus') {
						const status = await client.getDigest('');
						returnData.push({ json: status });
					}
				} else if (resource === 'graph') {
					if (operation === 'traverse') {
						const collectionName = this.getNodeParameter('collectionName', i) as string;
						const startId = this.getNodeParameter('startId', i, 0) as number;
						const maxDepth = this.getNodeParameter('maxDepth', i, 3) as number;
						const maxNodes = this.getNodeParameter('maxNodes', i, 256) as number;
						const nodes = await client.traverse(startId, 0, maxDepth, maxNodes, collectionName);
						returnData.push({ json: { nodes } });
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
