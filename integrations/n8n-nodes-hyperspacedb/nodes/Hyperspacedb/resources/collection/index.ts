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
				name: 'Get Many',
				value: 'getAll',
				action: 'Get all collections',
				description: 'Retrieve a list of all collections',
				routing: {
					request: {
						method: 'GET',
						url: '/collections',
					},
				},
			},
			{
				name: 'Create',
				value: 'create',
				action: 'Create a collection',
				description: 'Create a new collection with specified dimension and metric',
				routing: {
					request: {
						method: 'POST',
						url: '/collections',
					},
				},
			},
			{
				name: 'Get Stats',
				value: 'getStats',
				action: 'Get collection stats',
				description: 'Retrieve statistics for a specific collection',
				routing: {
					request: {
						method: 'GET',
						url: '=/collections/{{$parameter["collectionName"]}}/stats',
					},
				},
			},
			{
				name: 'Delete',
				value: 'delete',
				action: 'Delete a collection',
				description: 'Delete an existing collection',
				routing: {
					request: {
						method: 'DELETE',
						url: '=/collections/{{$parameter["collectionName"]}}',
					},
				},
			},
			{
				name: 'Rebuild Index',
				value: 'rebuild',
				action: 'Rebuild collection index',
				description: 'Trigger an optimization (Hot Vacuum) on a collection',
				routing: {
					request: {
						method: 'POST',
						url: '=/collections/{{$parameter["collectionName"]}}/rebuild',
					},
				},
			},
		],
		default: 'getAll',
	},
	{
		displayName: 'Collection Name',
		name: 'collectionName',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['create', 'getStats', 'delete', 'rebuild'],
			},
		},
		default: '',
		description: 'The name of the collection',
		routing: {
			send: {
				type: 'body',
				property: 'name',
			},
		},
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
		default: 128,
		description: 'The number of dimensions in the vector',
		routing: {
			send: {
				type: 'body',
				property: 'dimension',
			},
		},
	},
	{
		displayName: 'Metric',
		name: 'metric',
		type: 'options',
		options: [
			{ name: 'L2 (Euclidean)', value: 'l2' },
			{ name: 'Cosine', value: 'cosine' },
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
		default: 'l2',
		description: 'The distance metric to use',
		routing: {
			send: {
				type: 'body',
				property: 'metric',
			},
		},
	},
];
