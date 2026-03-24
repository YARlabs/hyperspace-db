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
				name: 'Traverse',
				value: 'traverse',
				action: 'Traverse graph',
				description: 'Perform a BFS/DFS traversal with filtering',
			},
		],
		default: 'traverse',
	},
];
