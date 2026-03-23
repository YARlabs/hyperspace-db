import type { INodeProperties } from 'n8n-workflow';

const showForGraph = {
	resource: ['graph'],
};

export const graphDescription: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		noDataExpression: true,
		displayOptions: {
			show: showForGraph,
		},
		options: [
			{
				name: 'Get Node',
				value: 'getNode',
				action: 'Get graph node',
				description: 'Retrieve a specific node and its immediate metadata from the graph',
				routing: {
					request: {
						method: 'GET',
						url: '=/collections/{{$parameter["collectionName"]}}/graph/node',
						qs: {
							id: '={{$parameter["id"]}}',
							layer: '={{$parameter["layer"]}}',
						},
					},
				},
			},
			{
				name: 'Get Neighbors',
				value: 'getNeighbors',
				action: 'Get neighbors',
				description: 'Retrieve neighbors of a specific node in a graph layer',
				routing: {
					request: {
						method: 'GET',
						url: '=/collections/{{$parameter["collectionName"]}}/graph/neighbors',
						qs: {
							id: '={{$parameter["id"]}}',
							layer: '={{$parameter["layer"]}}',
							limit: '={{$parameter["limit"]}}',
						},
					},
				},
			},
			{
				name: 'Traverse',
				value: 'traverse',
				action: 'Traverse graph',
				description: 'Perform a BFS/DFS traversal with filtering',
				routing: {
					request: {
						method: 'POST',
						url: '=/collections/{{$parameter["collectionName"]}}/graph/traverse',
					},
				},
			},
			{
				name: 'Find Clusters',
				value: 'clusters',
				action: 'Find clusters',
				description: 'Detect semantic clusters within the graph',
				routing: {
					request: {
						method: 'POST',
						url: '=/collections/{{$parameter["collectionName"]}}/graph/clusters',
					},
				},
			},
		],
		default: 'getNode',
	},
	{
		displayName: 'Collection Name',
		name: 'collectionName',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['graph'],
			},
		},
		default: '',
	},
	{
		displayName: 'Node ID',
		name: 'id',
		type: 'number',
		required: true,
		displayOptions: {
			show: {
				resource: ['graph'],
				operation: ['getNode', 'getNeighbors'],
			},
		},
		default: 0,
	},
	{
		displayName: 'Start ID',
		name: 'startId',
		type: 'number',
		required: true,
		displayOptions: {
			show: {
				resource: ['graph'],
				operation: ['traverse'],
			},
		},
		default: 0,
		routing: {
			send: {
				type: 'body',
				property: 'start_id',
			},
		},
	},
	{
		displayName: 'Layer',
		name: 'layer',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['graph'],
			},
		},
		default: 0,
		routing: {
			send: {
				type: 'body',
				property: 'layer',
			},
		},
	},
	{
		displayName: 'Limit',
		name: 'limit',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['graph'],
				operation: ['getNeighbors', 'traverse', 'clusters'],
			},
		},
		default: 64,
		routing: {
			send: {
				type: 'body',
				property: 'limit',
			},
		},
	},
	{
		displayName: 'Max Depth',
		name: 'maxDepth',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['graph'],
				operation: ['traverse'],
			},
		},
		default: 2,
		routing: {
			send: {
				type: 'body',
				property: 'max_depth',
			},
		},
	},
];
