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
			{
				name: 'Link',
				value: 'link',
				action: 'Create directed edge',
				description: 'Create an explicit directed, weighted, labeled edge between two nodes',
			},
			{
				name: 'Link Batch',
				value: 'linkBatch',
				action: 'Batch link edges',
				description: 'Atomically link multiple edges in CSR graph',
			},
			{
				name: 'Unlink',
				value: 'unlink',
				action: 'Delete directed edge',
				description: 'Delete an explicit directed edge with zero-lag invalidation',
			},
			{
				name: 'Induce Edges',
				value: 'induceEdges',
				action: 'Induce edges from metadata',
				description: 'Declaratively build edges from metadata properties (path_hierarchy, foreign_key, exact_match)',
			},
		],
		default: 'traverse',
	},
	{
		displayName: 'Collection Name',
		name: 'collectionName',
		type: 'string',
		default: 'default',
		required: true,
		displayOptions: {
			show: showForGraph,
		},
		description: 'The name of the collection',
	},
	{
		displayName: 'Source ID',
		name: 'sourceId',
		type: 'number',
		default: 1,
		required: true,
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['link', 'unlink'],
			},
		},
		description: 'Numeric ID of the source entity',
	},
	{
		displayName: 'Target ID',
		name: 'targetId',
		type: 'number',
		default: 2,
		required: true,
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['link', 'unlink'],
			},
		},
		description: 'Numeric ID of the target entity',
	},
	{
		displayName: 'Relation',
		name: 'relation',
		type: 'string',
		default: 'OWNS',
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['link', 'unlink'],
			},
		},
		description: 'Semantic relation label (e.g. OWNS, DEPENDS_ON)',
	},
	{
		displayName: 'Weight',
		name: 'weight',
		type: 'number',
		default: 1.0,
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['link'],
			},
		},
		description: 'Edge weight',
	},
	{
		displayName: 'Edges JSON',
		name: 'edgesJson',
		type: 'string',
		default: '[{"sourceId": 1, "targetId": 2, "relation": "OWNS", "weight": 1.0}]',
		required: true,
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['linkBatch'],
			},
		},
		description: 'JSON array of edges to link: [{ sourceId, targetId, relation, weight }]',
	},
	{
		displayName: 'Rules JSON',
		name: 'rulesJson',
		type: 'string',
		default: '[{"field": "file_path", "ruleType": "path_hierarchy", "relation": "CONTAINS"}]',
		required: true,
		displayOptions: {
			show: {
				...showForGraph,
				operation: ['induceEdges'],
			},
		},
		description: 'JSON array of induction rules: [{ field, ruleType, targetField?, relation?, weight? }]',
	},
];
