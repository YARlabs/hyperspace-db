import { NodeConnectionTypes, type INodeType, type INodeTypeDescription } from 'n8n-workflow';
import { collectionDescription } from './resources/collection';
import { vectorDescription } from './resources/vector';
import { systemDescription } from './resources/system';
import { graphDescription } from './resources/graph';

export class Hyperspacedb implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'Hyperspacedb',
		name: 'hyperspacedb',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1,
		subtitle: '={{$parameter["operation"] + ": " + $parameter["resource"]}}',
		description: 'Interact with the Hyperspacedb API',
		defaults: {
			name: 'Hyperspacedb',
		},
		usableAsTool: true,
		inputs: [NodeConnectionTypes.Main],
		outputs: [NodeConnectionTypes.Main],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		requestDefaults: {
			baseURL: '={{$credentials.baseUrl}}',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json',
			},
		},
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
}
