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
				routing: {
					request: {
						method: 'GET',
						url: '/status',
					},
				},
			},
			{
				name: 'Get Metrics',
				value: 'getMetrics',
				action: 'Get system metrics',
				description: 'Get internal engine metrics (RAM, CPU, vector count)',
				routing: {
					request: {
						method: 'GET',
						url: '/metrics',
					},
				},
			},
			{
				name: 'Get Usage Report',
				value: 'getUsage',
				action: 'Get usage report',
				description: 'Get detailed usage report for admin purposes',
				routing: {
					request: {
						method: 'GET',
						url: '/admin/usage',
					},
				},
			},
			{
				name: 'Vacuum Memory',
				value: 'vacuum',
				action: 'Vacuum system memory',
				description: 'Trigger memory reclamation and return it to the OS',
				routing: {
					request: {
						method: 'POST',
						url: '/admin/vacuum',
					},
				},
			},
		],
		default: 'getStatus',
	},
];
