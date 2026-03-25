import type {
	ICredentialType,
	INodeProperties,
} from 'n8n-workflow';

export class HyperspacedbApi implements ICredentialType {
	name = 'hyperspacedbApi';

	displayName = 'HyperspaceDB API';

	documentationUrl = 'https://yar.ink/docs';

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

	// @ts-ignore: Custom gRPC test function (using legacy method name to avoid declarative URL validation)
	async test(this: any): Promise<any> {
		let host = this.getCredentialData('host') as string;
		const port = this.getCredentialData('port') as number;
		const apiKey = this.getCredentialData('apiKey') as string;

		// Strip http:// or https:// if present
		host = host.replace(/^(http|https):\/\//, '').replace(/\/$/, '');

		const { HyperspaceClient } = await import('hyperspace-sdk-ts');
		// Handle port as string or number for the SDK
		const client = new HyperspaceClient(`${host}:${port}`, apiKey);

		try {
			await client.getDigest("");
			return [{ json: { success: true } }];
		} catch (error: any) {
			throw new Error(`Connection failed: ${error.message}`);
		}
	}
}
