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
    /**
     * @default 1024
     */
    dimension?: number;
    /**
     * @default "lorentz"
     */
    metric?: string;
}

export class HyperspaceStore extends VectorStore {
    private client: HyperspaceClient;
    private collectionName: string;
    private enableDeduplication: boolean;
    private useServerSideEmbedding: boolean;

    constructor(embeddings: Embeddings | undefined, args: HyperspaceStoreArgs) {
        // Use a dummy embeddings if server-side is enabled but none provided
        super(embeddings || ({} as any), args);
        this.client = args.client;
        this.collectionName = args.collectionName ?? "default";
        this.enableDeduplication = args.enableDeduplication ?? true;
        this.useServerSideEmbedding = args.useServerSideEmbedding ?? false;
    }

    _vectorstoreType(): string {
        return "hyperspace";
    }

    async addDocuments(documents: Document[], options?: string[] | { ids?: string[] }): Promise<string[]> {
        if (!documents || !Array.isArray(documents) || documents.length === 0) {
            return [];
        }

        const ids = Array.isArray(options) ? options : options?.ids;
        if (this.useServerSideEmbedding) {
            const resultIds: string[] = [];
            for (let i = 0; i < documents.length; i++) {
                const doc = documents[i];
                if (!doc) continue;
                let idNum: number;
                if (ids && ids[i]) {
                    idNum = parseInt(ids[i]) || this.computeContentHash(ids[i]);
                } else {
                    idNum = this.computeContentHash(doc.pageContent || "");
                }
                const metadata = { ...doc.metadata, text: doc.pageContent || "" };
                
                const typed_metadata: Record<string, string> = {};
                for (const [key, value] of Object.entries(metadata)) {
                    typed_metadata[key] = String(value);
                }

                await this.client.insertText(doc.pageContent || "", idNum, typed_metadata, this.collectionName);
                resultIds.push(idNum.toString());
            }
            return resultIds;
        }

        if (!this.embeddings) {
            throw new Error("Embeddings required when useServerSideEmbedding is false");
        }

        const texts = documents.filter(d => d).map(({ pageContent }) => pageContent || "");
        return this.addVectors(
            await this.embeddings.embedDocuments(texts),
            documents.filter(d => d),
            { ids }
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
        
        const itemsToInsert: { id: number; vector: number[]; metadata: Record<string, string> }[] = [];

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

            itemsToInsert.push({
                id: idNum,
                vector: vector,
                metadata: fullMetadata
            });
            resultIds.push(idStr);
        }

        try {
            await this.client.batchInsert(
                itemsToInsert,
                this.collectionName
            );
        } catch (e) {
            console.error(`Failed to batch insert vectors:`, e);
            throw e;
        }

        return resultIds;
    }

    async delete(params: { ids?: string[] }): Promise<void> {
        const ids = params.ids || [];
        for (const idStr of ids) {
            const idNum = parseInt(idStr) || this.computeContentHash(idStr);
            await this.client.delete(idNum, this.collectionName);
        }
    }

    async similaritySearchVectorWithScore(
        query: number[],
        k: number,
        filter?: Record<string, any>
    ): Promise<[Document, number][]> {
        const filters = this.parseFilters(filter);
        const results = await this.client.search(query, k, this.collectionName, { filters });
        return this.resultsToDocuments(results);
    }

    async similaritySearch(
        query: string,
        k: number,
        filter?: Record<string, any>
    ): Promise<Document[]> {
        if (this.useServerSideEmbedding) {
            const filters = this.parseFilters(filter);
            const results = await this.client.searchText(query, k, this.collectionName, { filters });
            const docsWithScore = this.resultsToDocuments(results);
            return docsWithScore.map(([doc]) => doc);
        }
        if (!this.embeddings) {
            throw new Error("Embeddings required when useServerSideEmbedding is false");
        }
        return super.similaritySearch(query, k, filter);
    }

    async maxMarginalRelevanceSearch(
        query: string,
        options: { k: number; fetchK?: number; lambda?: number; filter?: Record<string, any> }
    ): Promise<Document[]> {
        if (this.useServerSideEmbedding || !this.embeddings) {
            return this.similaritySearch(query, options.k, options.filter);
        }
        
        const { k, fetchK = 20, filter } = options;
        const queryEmbedding = await this.embeddings.embedQuery(query);
        const filters = this.parseFilters(filter);
        
        const results = await this.client.search(queryEmbedding, fetchK, this.collectionName, { filters });
        return this.resultsToDocuments(results).slice(0, k).map(([doc]) => doc);
    }

    private parseFilters(filter?: Record<string, any>): any[] {
        if (!filter) return [];
        const filters: any[] = [];
        for (const [key, value] of Object.entries(filter)) {
            if (typeof value === "object" && value !== null) {
                // Handle spatial filters first
                if ("$in_ball" in value) {
                    filters.push({ ball: { key, ...(value.$in_ball as any) } });
                } else if ("$in_box" in value) {
                    filters.push({ box: { key, ...(value.$in_box as any) } });
                } else if ("$in_cone" in value) {
                    filters.push({ cone: { key, ...(value.$in_cone as any) } });
                } else {
                    // Regular range filters
                    const range: any = { key };
                    if ("$gte" in value) range.gte = value.$gte;
                    if ("$lte" in value) range.lte = value.$lte;
                    filters.push({ range });
                }
            } else {
                filters.push({ match: { key, value: String(value) } });
            }
        }
        return filters;
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
}
