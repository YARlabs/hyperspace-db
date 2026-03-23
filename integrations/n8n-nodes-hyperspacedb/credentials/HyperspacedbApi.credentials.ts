import type {
	IAuthenticateGeneric,
	ICredentialTestRequest,
	ICredentialType,
	INodeProperties,
} from 'n8n-workflow';

export class HyperspacedbApi implements ICredentialType {
	name = 'hyperspacedbApi';

	displayName = 'Hyperspacedb API';

	// Link to your community node's README
	documentationUrl = 'https://github.com/yarlabs/hyperspace-db';

	properties: INodeProperties[] = [
		{
			displayName: 'Base URL',
			name: 'baseUrl',
			type: 'string',
			default: 'http://localhost:50050/api',
			placeholder: 'http://localhost:50050/api',
			required: true,
			description: 'The base URL of your HyperspaceDB instance (including /api)',
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

	authenticate: IAuthenticateGeneric = {
		type: 'generic',
		properties: {
			headers: {
				'x-api-key': '={{$credentials.apiKey}}',
			},
		},
	};

	test: ICredentialTestRequest = {
		request: {
			baseURL: '={{$credentials.baseUrl}}',
			url: '/status',
		},
	};
}
