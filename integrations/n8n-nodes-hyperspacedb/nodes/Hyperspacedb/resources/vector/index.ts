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
];
