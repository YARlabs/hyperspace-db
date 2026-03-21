import { VectorStore } from "@langchain/core/vectorstores";
import { Embeddings } from "@langchain/core/embeddings";
import { Document } from "@langchain/core/documents";
import * as crypto from "crypto";
import { HyperspaceClient } from "hyperspace-sdk-ts";

export interface HyperspaceStoreArgs {
    client: HyperspaceClient;
    collectionName?: string;
    enableDeduplication?: boolean;
    /**
     * If true, server will handle embedding (via insertText/searchText).
     * @default false
     */
    useServerSideEmbedding?: boolean;
}

export class HyperspaceStore extends VectorStore {
    private client: HyperspaceClient;
    private collectionName: string;
    private enableDeduplication: boolean;
    private useServerSideEmbedding: boolean;

    constructor(embeddings: Embeddings, args: HyperspaceStoreArgs) {
        super(embeddings, args);
        this.client = args.client;
        this.collectionName = args.collectionName ?? "default";
        this.enableDeduplication = args.enableDeduplication ?? true;
        this.useServerSideEmbedding = args.useServerSideEmbedding ?? false;
    }

    _vectorstoreType(): string {
        return "hyperspace";
    }

    async addDocuments(documents: Document[]): Promise<string[]> {
        if (this.useServerSideEmbedding) {
            const resultIds: string[] = [];
            for (const doc of documents) {
                const idNum = this.computeContentHash(doc.pageContent);
                const metadata = { ...doc.metadata, text: doc.pageContent };
                
                const typed_metadata: Record<string, string> = {};
                for (const [key, value] of Object.entries(metadata)) {
                    typed_metadata[key] = String(value);
                }

                await this.client.insertText(idNum, doc.pageContent, typed_metadata, this.collectionName);
                resultIds.push(idNum.toString());
            }
            return resultIds;
        }

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
        const idsToInsert: number[] = [];
        const vectorsToInsert: number[][] = [];
        const metadatasToInsert: Record<string, string>[] = [];

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

            idsToInsert.push(idNum);
            vectorsToInsert.push(vector);
            metadatasToInsert.push(fullMetadata);
            resultIds.push(idStr);
        }

        const items = idsToInsert.map((idNum, idx) => ({
            id: idNum,
            vector: vectorsToInsert[idx],
            metadata: metadatasToInsert[idx]
        }));

        try {
            await this.client.batchInsert(
                items,
                this.collectionName
            );
        } catch (e) {
            console.error(`Failed to batch insert vectors:`, e);
            throw e;
        }

        return resultIds;
    }

    async similaritySearchVectorWithScore(
        query: number[],
        k: number,
        filter?: this["FilterType"]
    ): Promise<[Document, number][]> {
        // This is only called when useServerSideEmbedding is false or via low-level
        const results = await this.client.search(query, k, this.collectionName);
        return this.resultsToDocuments(results);
    }

    async similaritySearch(
        query: string,
        k: number,
        filter?: this["FilterType"]
    ): Promise<Document[]> {
        if (this.useServerSideEmbedding) {
            const results = await this.client.searchText(query, k, this.collectionName);
            const docsWithScore = this.resultsToDocuments(results);
            return docsWithScore.map(([doc]) => doc);
        }
        return super.similaritySearch(query, k, filter);
    }

    private resultsToDocuments(results: any[]): [Document, number][] {
        return results.map((r: any) => {
            const metadata = r.metadata || {};
            const text = metadata["text"] || "";
            const docMeta = { ...metadata };
            return [
                new Document({
                    pageContent: text,
                    metadata: docMeta
                }),
                r.distance
            ];
        });
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
