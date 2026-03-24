import type { INodeProperties } from 'n8n-workflow';

const showForSystem = {
	resource: ['system'],
};

export const systemDescription: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		noDataExpression: true,
		displayOptions: {
			show: showForSystem,
		},
		options: [
			{
				name: 'Get Status',
				value: 'getStatus',
				action: 'Get system status',
				description: 'Retrieve real-time status and configuration of the HyperspaceDB instance',
			},
		],
		default: 'getStatus',
	},
];
