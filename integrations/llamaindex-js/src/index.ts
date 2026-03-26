import {
  VectorStore,
  VectorStoreQuery,
  VectorStoreQueryResult,
  VectorStoreQueryMode,
} from "llamaindex";
import { HyperspaceClient } from "hyperspace-sdk-ts";

export class HyperspaceVectorStore implements VectorStore {
  storesText: boolean = true;
  private _client: HyperspaceClient;
  private collectionName: string;

  constructor(config: {
    client: HyperspaceClient;
    collectionName: string;
  }) {
    this._client = config.client;
    this.collectionName = config.collectionName;
    this._ensureCollection();
  }

  private async _ensureCollection() {
    try {
      const collections = (await this._client.listCollections()) as any[];
      const existing = collections.find(c => c.name === this.collectionName);
      if (existing) {
        console.log(`Using existing collection ${this.collectionName}: ${existing.dimension}d, ${existing.metric}`);
      }

    } catch (e) {
      // Ignore
    }
  }


  client(): any {
    return this._client;
  }

  async add(nodes: any[]): Promise<string[]> {
    for (const node of nodes) {
      const metadata = node.metadata || {};
      const id = parseInt(node.id_) || Math.floor(Math.random() * 2**32);
      
      await this._client.insert(node.embedding, id, metadata, this.collectionName);
    }
    return nodes.map((n) => n.id_);
  }

  async delete(refDocId: string): Promise<void> {
    const id = parseInt(refDocId);
    if (!isNaN(id)) {
      await this._client.delete(id, this.collectionName);
    }
  }

  async query(query: VectorStoreQuery): Promise<VectorStoreQueryResult> {
    if (query.mode !== VectorStoreQueryMode.DEFAULT) {
      throw new Error("Only default query mode is supported");
    }

    const results = await this._client.search(
      query.queryEmbedding!,
      query.similarityTopK || 10,
      this.collectionName
    );

    return {
      nodes: results.map((r: any) => ({
        id_: r.id.toString(),
        metadata: r.metadata,
        embedding: r.vector,
      } as any)),
      similarities: results.map((r: any) => r.score),
      ids: results.map((r: any) => r.id.toString()),
    };
  }
}
