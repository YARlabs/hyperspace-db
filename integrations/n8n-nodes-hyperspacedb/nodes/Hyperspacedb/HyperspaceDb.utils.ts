import { ISupplyDataFunctions, IExecuteFunctions, NodeConnectionTypes } from 'n8n-workflow';
import { HyperspaceClient } from 'hyperspace-sdk-ts';
import { HyperspaceStore } from './HyperspaceStore';

export interface HyperspaceNodeCredentials {
	host: string;
	port: number;
	apiKey: string;
}

export async function getHyperspaceClient(
	context: ISupplyDataFunctions | IExecuteFunctions,
): Promise<HyperspaceClient> {
	const credentials = await context.getCredentials('hyperspaceDbApi') as any;
	return new HyperspaceClient(`${credentials.host}:${credentials.port}`, credentials.apiKey) as any;
}

export async function getHyperspaceStore(
	context: ISupplyDataFunctions | IExecuteFunctions,
	itemIndex: number,
): Promise<HyperspaceStore> {
	const credentials = await context.getCredentials('hyperspaceDbApi') as any;
    const client = new HyperspaceClient(`${credentials.host}:${credentials.port}`, credentials.apiKey);
	const collectionName = context.getNodeParameter('collectionName', itemIndex) as string;
	const metric = context.getNodeParameter('metric', itemIndex) as string;
	const dimension = context.getNodeParameter('dimension', itemIndex, 1024) as number;
    
	const embeddings = await context.getInputConnectionData(NodeConnectionTypes.AiEmbedding, itemIndex);

	return new HyperspaceStore(embeddings as any, {
		client: client as any, // Cast to any to avoid type mismatch between different SDK versions
		collectionName,
		dimension,
		metric,
	});
}
