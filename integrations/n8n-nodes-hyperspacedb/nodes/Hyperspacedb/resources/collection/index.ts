import type { INodeProperties } from 'n8n-workflow';

const showForCollection = {
	resource: ['collection'],
};

export const collectionDescription: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		noDataExpression: true,
		displayOptions: {
			show: showForCollection,
		},
		options: [
			{
				name: 'List',
				value: 'list',
				action: 'List all collections',
				description: 'Retrieve a list of all collections',
			},
			{
				name: 'Create',
				value: 'create',
				action: 'Create a collection',
				description: 'Create a new collection with specified dimension and metric',
			},
			{
				name: 'Get Stats',
				value: 'getStats',
				action: 'Get collection stats',
				description: 'Retrieve statistics for a specific collection',
			},
			{
				name: 'Delete',
				value: 'delete',
				action: 'Delete a collection',
				description: 'Delete an existing collection',
			},
		],
		default: 'list',
	},
	{
		displayName: 'Collection Name',
		name: 'name',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['create', 'getStats', 'delete'],
			},
		},
		default: '',
		description: 'The name of the collection',
	},
	{
		displayName: 'Dimension',
		name: 'dimension',
		type: 'number',
		required: true,
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['create'],
			},
		},
		default: 1024,
		description: 'Number of dimensions (e.g. 1536 for OpenAI, 1024 for Hyperspace DEFAULT)',
	},
	{
		displayName: 'Metric',
		name: 'metric',
		type: 'options',
		options: [
			{ name: 'Cosine', value: 'cosine' },
			{ name: 'L2 (Euclidean)', value: 'l2' },
			{ name: 'Poincare', value: 'poincare' },
			{ name: 'Lorentz', value: 'lorentz' },
		],
		required: true,
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['create'],
			},
		},
		default: 'cosine',
		description: 'Distance metric to use',
	},
];
