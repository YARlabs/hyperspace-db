import {
	IExecuteFunctions,
	INodeExecutionData,
	INodeType,
	INodeTypeDescription,
	NodeConnectionTypes,
} from 'n8n-workflow';
import { HyperspaceClient } from 'hyperspace-sdk-ts';
import { collectionDescription } from './resources/collection';
import { vectorDescription } from './resources/vector';
import { systemDescription } from './resources/system';
import { graphDescription } from './resources/graph';
import { queryPlannerDescription } from './resources/queryPlanner';

export class Hyperspacedb implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'HyperspaceDB',
		name: 'hyperspaceDb',
		icon: { light: 'file:hyperspacedb.svg', dark: 'file:hyperspacedb.dark.svg' },
		group: ['transform'],
		version: 1.1,
		subtitle: '={{$parameter["operation"] + ": " + $parameter["resource"]}}',
		description: 'Interact with the HyperspaceDB gRPC API',
		defaults: { name: 'HyperspaceDB' },
		usableAsTool: true,
		inputs: [NodeConnectionTypes.Main],
		outputs: [NodeConnectionTypes.Main],
		credentials: [{ name: 'hyperspacedbApi', required: true }],
		properties: [
			{
				displayName: 'Resource',
				name: 'resource',
				type: 'options',
				noDataExpression: true,
				options: [
					{ name: 'Collection', value: 'collection' },
					{ name: 'Vector', value: 'vector' },
					{ name: 'Query Planner', value: 'queryPlanner' },
					{ name: 'System', value: 'system' },
					{ name: 'Graph', value: 'graph' },
				],
				default: 'collection',
			},
			...collectionDescription,
			...vectorDescription,
			...queryPlannerDescription,
			...systemDescription,
			...graphDescription,
		],
	};

	async execute(this: IExecuteFunctions): Promise<INodeExecutionData[][]> {
		const items = this.getInputData();
		const returnData: INodeExecutionData[] = [];
		const credentials = await this.getCredentials('hyperspacedbApi') as any;
		const host = credentials.host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');
		const client = new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey) as any;

		const resource = this.getNodeParameter('resource', 0) as string;
		const operation = this.getNodeParameter('operation', 0) as string;

		for (let i = 0; i < items.length; i++) {
			try {
				if (resource === 'vector') {
					const collectionName = this.getNodeParameter('collectionName', i) as string;

					if (operation === 'insert') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;

						const vector = typeof vectorStr === 'string' ? JSON.parse(vectorStr) : vectorStr;
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insert(vector, id, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'insertText') {
						const id = this.getNodeParameter('vectorId', i) as number;
						const text = this.getNodeParameter('textContent', i) as string;
						const metadataStr = this.getNodeParameter('metadata', i, '{}') as string;
						const metadata = typeof metadataStr === 'string' ? JSON.parse(metadataStr) : metadataStr;

						await client.insertText(id, text, metadata, collectionName);
						returnData.push({ json: { success: true, id, collection: collectionName } });
					} else if (operation === 'search') {
						const vectorStr = this.getNodeParameter('vectorData', i) as string;
						const topK = this.getNodeParameter('topK', i) as number;
						const vector = typeof vectorStr === 'string' ? JSON.parse(vectorStr) : vectorStr;

						const results = await client.search(vector, topK, collectionName);
						returnData.push({ json: { results } });
					} else if (operation === 'searchText') {
						const text = this.getNodeParameter('textContent', i) as string;
						const topK = this.getNodeParameter('topK', i) as number;

						const results = await client.searchText(text, topK, collectionName);
						returnData.push({ json: { results } });
					} else if (operation === 'verifyGeneration') {
						const llmAnswer = this.getNodeParameter('llmAnswer', i) as string;
						const contextIdsParam = this.getNodeParameter('contextIds', i) as string;
						const maxDistanceThreshold = this.getNodeParameter('maxDistanceThreshold', i, undefined) as number | undefined;
						const checkGeodesic = this.getNodeParameter('checkGeodesic', i, true) as boolean;

						let contextIds: string[] = [];
						if (typeof contextIdsParam === 'string') {
							try {
								const parsed = JSON.parse(contextIdsParam);
								contextIds = Array.isArray(parsed) ? parsed.map(String) : [contextIdsParam];
							} catch {
								contextIds = contextIdsParam.split(',').map(s => s.trim()).filter(Boolean);
							}
						}

						const verification = await client.verifyGeneration(
							llmAnswer,
							contextIds,
							collectionName,
							maxDistanceThreshold,
							checkGeodesic
						);
						returnData.push({ json: verification as any });
					}
				} else if (resource === 'queryPlanner') {
					if (operation === 'planAndSplit') {
						const rawUserQuery = this.getNodeParameter('rawUserQuery', i) as string;
						const context = this.getNodeParameter('context', i, '') as string;
						const collectionsParam = this.getNodeParameter('collections', i, '') as string;
						const maxSubQueries = this.getNodeParameter('maxSubQueries', i, 5) as number;

						const collections = collectionsParam
							? collectionsParam.split(',').map(s => s.trim()).filter(Boolean)
							: ['default'];

						const VAGUE_PATTERNS = [
							/^(how|what|tell me)\s+(is|about)\s+(it|this|that|them)\??$/i,
							/^(compare|diff|difference)\s+(them|both|these|it)\??$/i,
							/^(is it good|how is performance|is it fast|does it work)\??$/i,
							/^(help|info|information|details)\??$/i,
							/^(what should i do|what next|where to start)\??$/i,
						];
						const SPLIT_CONNECTORS = [
							/\s+and\s+(?:also\s+)?(?:what\s+about|how\s+about|tell\s+me\s+about)?\s+/i,
							/\s*;\s*/,
							/\?\s+(?:also,?\s*)?/i,
							/\s+as\s+well\s+as\s+/i,
							/\s+in\s+addition\s+to\s+(?:that,?\s*)?/i,
							/\s+plus\s+/i,
							/\s*,\s*and\s+/i,
						];

						const raw = (rawUserQuery || '').trim();
						if (!raw || raw.length < 4) {
							returnData.push({
								json: {
									status: 'clarification_needed',
									action: 'ask_user_clarification',
									is_intent_clear: false,
									has_multiple_questions: false,
									search_queries: [],
									clarification: {
										question: 'Your query is brief. What specific topic would you like to search?',
										options: [
											'System architecture & performance',
											'API setup & integration',
											'Vector indexing & geometric metrics',
											'Compliance & policies'
										],
										reason: 'Input query is empty or too short.'
									},
									reason: 'Query too brief for accurate vector retrieval.',
									execution_strategy: { parallel: false, target_collections: collections, top_k_per_query: 5 }
								}
							});
						} else {
							const isVague = VAGUE_PATTERNS.some(pat => pat.test(raw));
							if (isVague && (!context || context.trim().length === 0)) {
								returnData.push({
									json: {
										status: 'clarification_needed',
										action: 'ask_user_clarification',
										is_intent_clear: false,
										has_multiple_questions: false,
										search_queries: [],
										clarification: {
											question: `Could you clarify what aspect of '${raw}' you are interested in?`,
											options: [
												'Benchmark latency & performance',
												'Configuration examples',
												'Geometric hierarchy & light cones',
												'Quantization & memory footprint'
											],
											reason: 'Query contains ambiguous pronouns or generic concepts without antecedent context.'
										},
										reason: 'Pronoun/concept ambiguity without conversation context.',
										execution_strategy: { parallel: false, target_collections: collections, top_k_per_query: 5 }
									}
								});
							} else {
								let subQueries = [raw];
								for (const regex of SPLIT_CONNECTORS) {
									const nextList: string[] = [];
									for (const q of subQueries) {
										if (regex.test(q)) {
											const parts = q.split(regex).map(p => p.trim()).filter(p => p.length > 3);
											if (parts.length > 1) {
												nextList.push(...parts);
												continue;
											}
										}
										nextList.push(q);
									}
									subQueries = nextList;
								}

								const cleanSubQueries = Array.from(new Set(subQueries.map(sq => sq.replace(/\?+$/, '').trim())))
									.filter(sq => sq.length > 3)
									.slice(0, maxSubQueries);

								const hasMultiple = cleanSubQueries.length > 1;
								returnData.push({
									json: {
										status: 'ready_for_search',
										action: hasMultiple ? 'parallel_search' : 'direct_search',
										is_intent_clear: true,
										has_multiple_questions: hasMultiple,
										search_queries: cleanSubQueries.length > 0 ? cleanSubQueries : [raw],
										clarification: null,
										reason: hasMultiple
											? `Detected ${cleanSubQueries.length} distinct questions. Split for parallel vector retrieval.`
											: 'Intent is clear and isolated.',
										execution_strategy: {
											parallel: hasMultiple,
											target_collections: collections,
											top_k_per_query: hasMultiple ? 3 : 5
										}
									}
								});
							}
						}
					}
				} else if (resource === 'collection') {
					if (operation === 'create') {
						const name = this.getNodeParameter('name', i) as string;
						const dimension = this.getNodeParameter('dimension', i) as number;
						const metric = this.getNodeParameter('metric', i) as string;

						await client.createCollection(name, dimension, metric);
						returnData.push({ json: { success: true, name } });
					} else if (operation === 'list') {
						const collections = await client.listCollections();
						returnData.push({ json: { collections } });
					} else if (operation === 'getStats') {
						const name = this.getNodeParameter('name', i) as string;
						
						let httpStats: any = {};
						try {
							const clientHost = (client as any).host || "";
							const ip = clientHost.split(':')[0];
							const url = `http://${ip}:50050/api/collections/${name}/stats`;
							const headers: Record<string, string> = {};
							if ((client as any).apiKey) headers['x-api-key'] = (client as any).apiKey;
							if ((client as any).userId) headers['x-hyperspace-user-id'] = (client as any).userId;

							const res = await fetch(url, { headers });
							if (res.ok) {
								httpStats = await res.json();
							}
						} catch (e) {
							console.error('Failed to fetch HTTP stats for collection:', e);
						}

						const digest = await client.getDigest(name);
						const stats = await client.getCollectionStats(name);
						
						returnData.push({
							json: {
								...digest,
								...stats,
								...httpStats,
							},
						});
					} else if (operation === 'delete') {
						const name = this.getNodeParameter('name', i) as string;
						await client.deleteCollection(name);
						returnData.push({ json: { success: true, collection: name } });
					} else if (operation === 'cacheStats') {
						const name = this.getNodeParameter('name', i) as string;
						const cacheStats = await client.getCacheStats(name);
						returnData.push({ json: cacheStats });
					} else if (operation === 'cacheClear') {
						const name = this.getNodeParameter('name', i) as string;
						const success = await client.clearCache(name);
						returnData.push({ json: { success } });
					} else if (operation === 'cacheConfig') {
						const name = this.getNodeParameter('name', i) as string;
						const policy = this.getNodeParameter('policy', i) as string;
						const annThreshold = this.getNodeParameter('annThreshold', i) as number;
						const success = await client.updateCacheConfig(name, policy, annThreshold);
						returnData.push({ json: { success } });
					}
				} else if (resource === 'system') {
					if (operation === 'getStatus') {
						const status = await client.getDigest('');
						returnData.push({ json: status });
					}
				} else if (resource === 'graph') {
					if (operation === 'traverse') {
						const collectionName = this.getNodeParameter('collectionName', i) as string;
						const startId = this.getNodeParameter('startId', i, 0) as number;
						const maxDepth = this.getNodeParameter('maxDepth', i, 3) as number;
						const maxNodes = this.getNodeParameter('maxNodes', i, 256) as number;
						const nodes = await client.traverse(startId, 0, maxDepth, maxNodes, collectionName);
						returnData.push({ json: { nodes } });
					}
				}
			} catch (error: any) {
				if (this.continueOnFail()) {
					returnData.push({ json: { error: error.message } });
					continue;
				}
				throw error;
			}
		}
		return [returnData];
	}
}
