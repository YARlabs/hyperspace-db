import { VectorStore } from "@langchain/core/vectorstores";
import { Embeddings } from "@langchain/core/embeddings";
import { Document } from "@langchain/core/documents";
import * as crypto from "crypto";

// Minimal interface for gRPC client (placeholder for generated code)
interface HyperspaceClient {
    insert(collection: string, id: number, vector: number[], metadata: Record<string, string>): Promise<void>;
    search(collection: string, vector: number[], k: number): Promise<Array<{ id: number; score: number; metadata: Record<string, string> }>>;
}

export interface HyperspaceStoreArgs {
    client: any; // Using any for now until we have generated gRPC types
    collectionName?: string;
    enableDeduplication?: boolean;
}

export class HyperspaceStore extends VectorStore {
    declare FilterType: object;
    private client: any;
    private collectionName: string;
    private enableDeduplication: boolean;

    constructor(embeddings: Embeddings, args: HyperspaceStoreArgs) {
        super(embeddings, args);
        this.client = args.client;
        this.collectionName = args.collectionName ?? "default";
        this.enableDeduplication = args.enableDeduplication ?? true;
    }

    _vectorstoreType(): string {
        return "hyperspace";
    }

    async addDocuments(documents: Document[]): Promise<string[]> {
        const texts = documents.map(({ pageContent }) => pageContent);
        const metadatas = documents.map(({ metadata }) => metadata);
        return this.addVectors(
            await this.embeddings.embedDocuments(texts),
            documents,
            { metadatas } // Passing metadata in options/kwargs equivalent if needed, but VectorStore method signature varies. 
            // Actually standard addVectors takes documents as 2nd arg usually for metadata extraction? 
            // Let's implement addVectors directly.
        );
    }

    async addVectors(
        vectors: number[][],
        documents: Document[],
        options?: { ids?: string[]; metadatas?: Record<string, any>[] }
    ): Promise<string[]> {
        const ids = options?.ids || [];
        const resultIds: string[] = [];

        // User provided metadata or extract from documents
        const metadatas = options?.metadatas || documents.map((d) => d.metadata);

        for (let i = 0; i < vectors.length; i++) {
            const text = documents[i].pageContent;
            const vector = vectors[i];
            const metadata = metadatas[i] || {};

            // Add text to metadata for retrieval
            const fullMetadata: Record<string, string> = {};
            for (const [key, value] of Object.entries(metadata)) {
                fullMetadata[key] = String(value);
            }
            fullMetadata["text"] = text;

            let idStr = ids[i];
            let idNum: number;

            if (!idStr && this.enableDeduplication) {
                // Content-based deduplication
                idNum = this.computeContentHash(text);
                idStr = idNum.toString();
            } else if (idStr) {
                idNum = parseInt(idStr) || this.computeContentHash(idStr);
            } else {
                // Random ID if no deduplication and no ID provided
                idNum = Math.floor(Math.random() * 4294967295);
                idStr = idNum.toString();
            }

            try {
                // Mocking gRPC call for now until we generate clients
                // await this.client.insert(this.collectionName, idNum, vector, fullMetadata);
                // console.log(`[HyperspaceDB] Inserted ${idNum} into ${this.collectionName}`);

                // In a real implementation this would be:
                // await new Promise((resolve, reject) => {
                //   this.client.insert({ ... }, (err, response) => ...)
                // })
            } catch (e) {
                console.error(`Failed to insert vector ${idNum}:`, e);
                throw e;
            }

            resultIds.push(idStr);
        }

        return resultIds;
    }

    async similaritySearchVectorWithScore(
        query: number[],
        k: number,
        filter?: this["FilterType"]
    ): Promise<[Document, number][]> {
        // Mock search result
        // In reality: await this.client.search(...)

        const results: [Document, number][] = [];

        return results;
    }

    private computeContentHash(text: string): number {
        const hash = crypto.createHash("sha256").update(text).digest();
        // Use first 4 bytes as u32
        return hash.readUInt32BE(0);
    }

    static async fromTexts(
        texts: string[],
        metadatas: object[] | object,
        embeddings: Embeddings,
        dbConfig: HyperspaceStoreArgs
    ): Promise<HyperspaceStore> {
        const docs: Document[] = [];
        for (let i = 0; i < texts.length; i += 1) {
            const metadata = Array.isArray(metadatas) ? metadatas[i] : metadatas;
            const newDoc = new Document({
                pageContent: texts[i],
                metadata,
            });
            docs.push(newDoc);
        }
        const store = new HyperspaceStore(embeddings, dbConfig);
        await store.addDocuments(docs);
        return store;
    }
}
