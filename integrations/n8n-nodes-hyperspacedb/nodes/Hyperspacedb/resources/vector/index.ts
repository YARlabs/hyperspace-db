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
				name: 'Insert Vector',
				value: 'insert',
				action: 'Insert a raw vector',
				description: 'Insert a new raw vector into a collection',
			},
			{
				name: 'Insert Text (Auto-embed)',
				value: 'insertText',
				action: 'Insert text with auto-embedding',
				description: 'Insert text that will be automatically vectorized by the server',
			},
			{
				name: 'Search Vector',
				value: 'search',
				action: 'Search by raw vector',
				description: 'Perform a raw vector similarity search',
			},
			{
				name: 'Search Text (Auto-embed)',
				value: 'searchText',
				action: 'Search by natural language',
				description: 'Search using a text query that will be vectorized by the server',
			},
			{
				name: 'Verify Generation (Anti-Hallucination)',
				value: 'verifyGeneration',
				action: 'Mathematically verify LLM answer against context boundaries',
				description: 'Verifies whether generated answer semantically breaks geometric boundaries of source context',
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
		displayName: 'ID (Numeric)',
		name: 'vectorId',
		type: 'number',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insert', 'insertText'],
			},
		},
		default: 0,
		description: 'The unique numeric ID for the entry',
	},
	{
		displayName: 'Text Content',
		name: 'textContent',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insertText', 'searchText'],
			},
		},
		default: '',
		description: 'The text content to be vectorized and stored/searched',
	},
	{
		displayName: 'Vector Data (JS Array)',
		name: 'vectorData',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insert', 'search'],
			},
		},
		default: '',
		placeholder: '[0.1, 0.2, 0.3, ...]',
		description: 'The raw vector embedding as a JSON array',
	},
	{
		displayName: 'Metadata',
		name: 'metadata',
		type: 'json',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['insert', 'insertText'],
			},
		},
		default: '{}',
		description: 'Optional metadata as a JSON object',
	},
	{
		displayName: 'Top K',
		name: 'topK',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['search', 'searchText'],
			},
		},
		default: 10,
		description: 'The number of results to return',
	},
	{
		displayName: 'LLM Answer',
		name: 'llmAnswer',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['verifyGeneration'],
			},
		},
		default: '',
		description: 'The text drafted by the LLM to verify against the source context boundaries',
	},
	{
		displayName: 'Context IDs',
		name: 'contextIds',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['verifyGeneration'],
			},
		},
		default: '',
		description: 'Comma-separated string or JSON array of source context chunk identifiers',
	},
	{
		displayName: 'Max Distance Threshold',
		name: 'maxDistanceThreshold',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['verifyGeneration'],
			},
		},
		default: 0.85,
		description: 'Optional maximum geometric distance threshold',
	},
	{
		displayName: 'Check Geodesic',
		name: 'checkGeodesic',
		type: 'boolean',
		displayOptions: {
			show: {
				resource: ['vector'],
				operation: ['verifyGeneration'],
			},
		},
		default: true,
		description: 'Whether to verify geodesic graph connectivity between chunks',
	},
];

