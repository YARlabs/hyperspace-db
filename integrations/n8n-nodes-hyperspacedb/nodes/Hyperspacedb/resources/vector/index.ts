import type { INodeProperties } from 'n8n-workflow';

const showForVector = {
	resource: ['vector'],
};

export const vectorDescription: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		noDataExpression: true,
		displayOptions: {
			show: showForVector,
		},
		options: [
			{
				name: 'Insert',
				value: 'insert',
				action: 'Insert a vector',
				description: 'Insert a new vector into a collection',
				routing: {
					request: {
						method: 'POST',
						url: '=/collections/{{$parameter["collectionName"]}}/insert',
					},
				},
			},
			{
				name: 'Search',
				value: 'search',
				action: 'Search vectors',
				description: 'Perform a vector similarity search',
				routing: {
					request: {
						method: 'POST',
						url: '=/collections/{{$parameter["collectionName"]}}/search',
					},
				},
			},
		],
		default: 'insert',
	},
	{
		displayName: 'Collection Name',
		name: 'collectionName',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
			},
		},
		default: '',
		description: 'The name of the collection to interact with',
	},
	{
		displayName: 'Vector ID',
		name: 'vectorId',
		type: 'number',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insert'],
			},
		},
		default: 0,
		description: 'The unique numeric ID for the vector',
		routing: {
			send: {
				type: 'body',
				property: 'id',
			},
		},
	},
	{
		displayName: 'Vector Data (JS Array or String)',
		name: 'vectorData',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
			},
		},
		default: '',
		placeholder: '[0.1, 0.2, 0.3, ...]',
		description: 'The vector embedding as a JSON array',
		routing: {
			send: {
				type: 'body',
				property: 'vector',
				// We might need to transform this from string to array if user passes string
                                // In declarative style, transformation is done via expressions or custom methods
			},
		},
	},
	{
		displayName: 'Metadata',
		name: 'metadata',
		type: 'json',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insert'],
			},
		},
		default: '{}',
		description: 'Optional metadata as a JSON object',
		routing: {
			send: {
				type: 'body',
				property: 'metadata',
			},
		},
	},
	{
		displayName: 'Top K',
		name: 'topK',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['search'],
			},
		},
		default: 10,
		description: 'The number of results to return',
		routing: {
			send: {
				type: 'body',
				property: 'top_k',
			},
		},
	},
        {
		displayName: 'Use Wasserstein',
		name: 'useWasserstein',
		type: 'boolean',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['search'],
			},
		},
		default: false,
		description: 'Whether to use Optimal Transport distance (Wasserstein HNSW)',
		routing: {
			send: {
				type: 'body',
				property: 'use_wasserstein',
			},
		},
	},
];
