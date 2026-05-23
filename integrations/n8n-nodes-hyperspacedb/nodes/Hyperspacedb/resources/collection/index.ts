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
			{
				name: 'Cache Stats',
				value: 'cacheStats',
				action: 'Get cache stats',
				description: 'Retrieve cache statistics for a specific collection\'s L0 Hot Tier',
			},
			{
				name: 'Clear Cache',
				value: 'cacheClear',
				action: 'Clear cache',
				description: 'Purge all hot items in the L0 Cache for a specific collection',
			},
			{
				name: 'Cache Config',
				value: 'cacheConfig',
				action: 'Update cache config',
				description: 'Update L0 Cache configuration (eviction policy, ANN threshold)',
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
				operation: ['create', 'getStats', 'delete', 'cacheStats', 'cacheClear', 'cacheConfig'],
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
	{
		displayName: 'Cache Eviction Policy',
		name: 'policy',
		type: 'options',
		options: [
			{ name: 'LRU (Least Recently Used)', value: 'lru' },
			{ name: 'LFU (Least Frequently Used)', value: 'lfu' },
			{ name: 'TTL (Time To Live)', value: 'ttl' },
		],
		required: true,
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['cacheConfig'],
			},
		},
		default: 'lru',
		description: 'The cache eviction policy to use',
	},
	{
		displayName: 'ANN Threshold',
		name: 'annThreshold',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['collection'],
				operation: ['cacheConfig'],
			},
		},
		default: 0.8,
		description: 'ANN similarity distance threshold for cache lookup filtering',
	},
];
