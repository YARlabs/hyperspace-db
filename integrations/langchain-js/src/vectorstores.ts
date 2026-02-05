import { VectorStore } from "@langchain/core/vectorstores";
import { Embeddings } from "@langchain/core/embeddings";
import { Document } from "@langchain/core/documents";
import * as crypto from "crypto";
import { HyperspaceClient } from "hyperspace-sdk-ts";

export interface HyperspaceStoreArgs {
    client: HyperspaceClient;
    collectionName?: string;
    enableDeduplication?: boolean;
}

export class HyperspaceStore extends VectorStore {
    declare FilterType: object;
    private client: HyperspaceClient;
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
        return this.addVectors(
            await this.embeddings.embedDocuments(texts),
            documents
        );
    }

    async addVectors(
        vectors: number[][],
        documents: Document[],
        options?: { ids?: string[]; metadatas?: Record<string, any>[] }
    ): Promise<string[]> {
        const ids = options?.ids || [];
        const resultIds: string[] = [];
        const metadatas = options?.metadatas || documents.map((d) => d.metadata);

        for (let i = 0; i < vectors.length; i++) {
            const text = documents[i].pageContent;
            const vector = vectors[i];
            const metadata = metadatas[i] || {};

            const fullMetadata: Record<string, string> = {};
            for (const [key, value] of Object.entries(metadata)) {
                fullMetadata[key] = String(value);
            }
            fullMetadata["text"] = text;

            let idStr = ids[i];
            let idNum: number;

            if (!idStr && this.enableDeduplication) {
                idNum = this.computeContentHash(text);
                idStr = idNum.toString();
            } else if (idStr) {
                idNum = parseInt(idStr) || this.computeContentHash(idStr);
            } else {
                idNum = Math.floor(Math.random() * 4294967295);
                idStr = idNum.toString();
            }

            try {
                await this.client.insert(idNum, vector, fullMetadata, this.collectionName);
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
        const results = await this.client.search(query, k, this.collectionName);

        const output: [Document, number][] = results.map(r => {
            // r is { id, distance, metadata } which we added in SDK
            // @ts-ignore - metadata is added in our modified SDK
            const metadata = r.metadata || {};
            const text = metadata["text"] || "";
            // Remove text from metadata to avoid duplication
            const docMeta = { ...metadata };
            // delete docMeta["text"]; // Optional: keep it or remove it

            return [
                new Document({
                    pageContent: text,
                    metadata: docMeta
                }),
                r.distance
            ];
        });

        return output;
    }

    private computeContentHash(text: string): number {
        const hash = crypto.createHash("sha256").update(text).digest();
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
