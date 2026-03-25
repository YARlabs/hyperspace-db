import { ISupplyDataFunctions, IExecuteFunctions, NodeConnectionTypes } from 'n8n-workflow';
import { HyperspaceClient } from 'hyperspace-sdk-ts';
import { HyperspaceStore } from './HyperspaceStore';

export interface HyperspaceNodeCredentials {
	host: string;
	port: number;
	apiKey: string;
}

function cleanHost(host: string): string {
    return host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');
}

export async function getHyperspaceClient(
	context: ISupplyDataFunctions | IExecuteFunctions,
): Promise<HyperspaceClient> {
	const credentials = await context.getCredentials('hyperspacedbApi') as any;
    const host = cleanHost(credentials.host);
	return new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey) as any;
}

export async function getHyperspaceStore(
	context: ISupplyDataFunctions | IExecuteFunctions,
	itemIndex: number,
): Promise<HyperspaceStore> {
	const credentials = await context.getCredentials('hyperspacedbApi') as any;
    const host = cleanHost(credentials.host);
    const client = new HyperspaceClient(`${host}:${credentials.port}`, credentials.apiKey);
	const collectionName = context.getNodeParameter('collectionName', itemIndex) as string;
	const metric = context.getNodeParameter('metric', itemIndex) as 'lorentz' | 'poincare' | 'cosine' | 'l2';
	const dimension = context.getNodeParameter('dimension', itemIndex, 1024) as number;
    
	const embeddings = await context.getInputConnectionData(NodeConnectionTypes.AiEmbedding, itemIndex);

	return new HyperspaceStore(embeddings as any, {
		client: client as any, // Cast to any to avoid type mismatch between different SDK versions
		collectionName,
		dimension,
		metric,
	});
}
