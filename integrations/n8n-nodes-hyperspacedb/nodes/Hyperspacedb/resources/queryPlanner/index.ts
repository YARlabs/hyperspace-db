import type { INodeProperties } from 'n8n-workflow';

const showForQueryPlanner = {
	resource: ['queryPlanner'],
};

export const queryPlannerDescription: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		noDataExpression: true,
		displayOptions: {
			show: showForQueryPlanner,
		},
		options: [
			{
				name: 'Plan & Clarify Intent',
				value: 'planAndSplit',
				action: 'Evaluate query clarity and split compound questions',
				description: 'Evaluates user query before vector search: checks intent clarity and splits into parallel sub-queries',
			},
		],
		default: 'planAndSplit',
	},
	{
		displayName: 'Raw User Query',
		name: 'rawUserQuery',
		type: 'string',
		required: true,
		displayOptions: {
			show: {
				resource: ['queryPlanner'],
				operation: ['planAndSplit'],
			},
		},
		default: '',
		description: 'The natural language user prompt to evaluate and decompose',
	},
	{
		displayName: 'Conversation Context',
		name: 'context',
		type: 'string',
		displayOptions: {
			show: {
				resource: ['queryPlanner'],
				operation: ['planAndSplit'],
			},
		},
		default: '',
		description: 'Optional previous conversation history to resolve pronouns or coreferences',
	},
	{
		displayName: 'Target Collections',
		name: 'collections',
		type: 'string',
		displayOptions: {
			show: {
				resource: ['queryPlanner'],
				operation: ['planAndSplit'],
			},
		},
		default: '',
		description: 'Comma-separated list of target collection names',
	},
	{
		displayName: 'Max Sub-Queries',
		name: 'maxSubQueries',
		type: 'number',
		displayOptions: {
			show: {
				resource: ['queryPlanner'],
				operation: ['planAndSplit'],
			},
		},
		default: 5,
		description: 'Maximum number of parallel sub-queries to split',
	},
];
