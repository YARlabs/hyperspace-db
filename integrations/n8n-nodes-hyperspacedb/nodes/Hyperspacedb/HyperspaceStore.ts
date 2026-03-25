import { VectorStore } from '@langchain/core/vectorstores';
import { Embeddings } from '@langchain/core/embeddings';
import { Document } from '@langchain/core/documents';

export interface HyperspaceStoreArgs {
	client: any;
	collectionName: string;
	dimension: number;
	metric: 'lorentz' | 'poincare' | 'cosine' | 'l2';
}

export class HyperspaceStore extends VectorStore {
	declare FilterType: any;

	constructor(
		embeddings: Embeddings,
		private args: HyperspaceStoreArgs,
	) {
		super(embeddings, args);
	}

	_vectorstoreType(): string {
		return 'hyperspace';
	}

	async addVectors(vectors: number[][], documents: Document[]): Promise<void> {
		const points = vectors.map((vector, idx) => ({
			id: Math.floor(Math.random() * 1000000), // In production, use UUID or consistent IDs
			vector,
			metadata: {
				...documents[idx].metadata,
				text: documents[idx].pageContent,
			},
		}));

		for (const point of points) {
			await this.args.client.insert(
                point.id,
                point.vector,
                point.metadata,
                this.args.collectionName
            );
		}
	}

	async addDocuments(documents: Document[]): Promise<void> {
		const texts = documents.map(({ pageContent }) => pageContent);
		const vectors = await this.embeddings.embedDocuments(texts);
		return this.addVectors(vectors, documents);
	}

	async similaritySearchVectorWithScore(
		query: number[],
		k: number,
		_filter?: this['FilterType'],
	): Promise<[Document, number][]> {
		const results = await this.args.client.search(
            query,
            k,
            this.args.collectionName
        );

		return results.map((res: any) => [
			new Document({
				pageContent: res.metadata?.text || '',
				metadata: res.metadata || {},
			}),
			res.distance,
		]);
	}

	static async fromTexts(
		texts: string[],
		metadatas: object[] | object,
		embeddings: Embeddings,
		dbConfig: HyperspaceStoreArgs,
	): Promise<HyperspaceStore> {
		const docs = texts.map((text, i) => {
			const metadata = Array.isArray(metadatas) ? metadatas[i] : metadatas;
			return new Document({ pageContent: text, metadata });
		});
		const instance = new this(embeddings, dbConfig);
		await instance.addDocuments(docs);
		return instance;
	}
}
