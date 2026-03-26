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
     * @default 1536
     */
    dimension?: number;
    /**
     * @default "l2"
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
        
        this._ensureCollection(args.dimension, args.metric);
    }

    private async _ensureCollection(desiredDim?: number, desiredMetric?: string) {
        try {
            const collections = await this.client.listCollections();
            const existing = collections.find(c => c.name === this.collectionName);
            if (existing) {
                 // Auto-populate properties if they were not explicitly locked in the store
                 console.log(`Using existing collection ${this.collectionName}: ${existing.dimension}d, ${existing.metric}`);
            } else if (desiredDim && desiredMetric) {
                await this.client.createCollection(this.collectionName, desiredDim, desiredMetric);
            }
        } catch (e) {
            console.error("Failed to check/create collection", e);
        }
    }


    _vectorstoreType(): string {
        return "hyperspace";
    }

    async addDocuments(documents: Document[], options?: string[] | { ids?: string[] }): Promise<string[]> {
        const ids = Array.isArray(options) ? options : options?.ids;
        if (this.useServerSideEmbedding) {
            const resultIds: string[] = [];
            for (let i = 0; i < documents.length; i++) {
                const doc = documents[i];
                let idNum: number;
                if (ids && ids[i]) {
                    idNum = parseInt(ids[i]) || this.computeContentHash(ids[i]);
                } else {
                    idNum = this.computeContentHash(doc.pageContent);
                }
                const metadata = { ...doc.metadata, text: doc.pageContent };
                
                const typed_metadata: Record<string, string> = {};
                for (const [key, value] of Object.entries(metadata)) {
                    typed_metadata[key] = String(value);
                }

                await this.client.insertText(doc.pageContent, idNum, typed_metadata, this.collectionName);
                resultIds.push(idNum.toString());
            }
            return resultIds;
        }

        if (!this.embeddings) {
            throw new Error("Embeddings required when useServerSideEmbedding is false");
        }

        const texts = documents.map(({ pageContent }) => pageContent);
        return this.addVectors(
            await this.embeddings.embedDocuments(texts),
            documents,
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
            console.warn("MMR search falls back to similarity search when using server-side embeddings.");
            return this.similaritySearch(query, options.k, options.filter);
        }
        
        const { k, fetchK = 20, lambda = 0.5, filter } = options;
        const queryEmbedding = await this.embeddings.embedQuery(query);
        const filters = this.parseFilters(filter);
        
        const results = await this.client.search(queryEmbedding, fetchK, this.collectionName, { filters });
        
        // MMR requires vectors. We currently don't expose vectors in search results for security/bandwidth.
        // So for now, MMR in 'community' mode will return top-K.
        return this.resultsToDocuments(results).slice(0, k).map(([doc]) => doc);
    }

    private parseFilters(filter?: Record<string, any>): any[] {
        if (!filter) return [];
        const filters: any[] = [];
        for (const [key, value] of Object.entries(filter)) {
            if (typeof value === "object" && value !== null) {
                // Handle spatial filters
                if ("$in_ball" in value) {
                    filters.push({ ball: { key, ...(value.$in_ball as any) } });
                } else if ("$in_box" in value) {
                    filters.push({ box: { key, ...(value.$in_box as any) } });
                } else if ("$in_cone" in value) {
                    filters.push({ cone: { key, ...(value.$in_cone as any) } });
                } else {
                    // Standard range filters $gte, $lte
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
