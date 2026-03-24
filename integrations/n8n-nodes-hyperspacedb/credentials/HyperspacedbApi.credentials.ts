import type {
	ICredentialType,
	INodeProperties,
} from 'n8n-workflow';

export class HyperspaceDbApi implements ICredentialType {
	name = 'hyperspaceDbApi';

	displayName = 'HyperspaceDB API';

	documentationUrl = 'https://docs.hyperspace.systems';

	properties: INodeProperties[] = [
		{
			displayName: 'Host',
			name: 'host',
			type: 'string',
			default: 'localhost',
			required: true,
			description: 'The host of your HyperspaceDB instance (e.g., localhost or host.docker.internal)',
		},
		{
			displayName: 'Port (gRPC)',
			name: 'port',
			type: 'number',
			default: 50051,
			required: true,
			description: 'The gRPC port of your HyperspaceDB instance (usually 50051)',
		},
		{
			displayName: 'API Key',
			name: 'apiKey',
			type: 'string',
			typeOptions: { password: true },
			required: true,
			default: 'I_LOVE_HYPERSPACEDB',
		},
	];
}
