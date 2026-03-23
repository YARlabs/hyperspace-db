import {
  BaseVectorStore,
  VectorStoreQuery,
  VectorStoreQueryResult,
  VectorStoreQueryMode,
} from "llamaindex";
import { HyperspaceClient } from "hyperspace-sdk-ts";

export class HyperspaceVectorStore extends BaseVectorStore {
  storesText: boolean = true;
  private client: HyperspaceClient;
  private collectionName: string;

  constructor(config: {
    client: HyperspaceClient;
    collectionName: string;
  }) {
    super();
    this.client = config.client;
    this.collectionName = config.collectionName;
  }

  async add(nodes: any[]): Promise<string[]> {
    for (const node of nodes) {
      const metadata = node.metadata || {};
      const id = parseInt(node.id_) || Math.floor(Math.random() * 2**32);
      
      await this.client.insert(id, node.embedding, metadata, this.collectionName);
    }
    return nodes.map((n) => n.id_);
  }

  async delete(refDocId: string): Promise<void> {
    const id = parseInt(refDocId);
    if (!isNaN(id)) {
      await this.client.delete(id, this.collectionName);
    }
  }

  async query(query: VectorStoreQuery): Promise<VectorStoreQueryResult> {
    if (query.mode !== VectorStoreQueryMode.DEFAULT) {
      throw new Error("Only default query mode is supported");
    }

    const results = await this.client.search(
      query.queryEmbedding!,
      query.similarityTopK || 10,
      this.collectionName
    );

    return {
      nodes: results.map((r: any) => ({
        id_: r.id.toString(),
        metadata: r.metadata,
        embedding: r.vector,
      })),
      similarities: results.map((r: any) => r.score),
      ids: results.map((r: any) => r.id.toString()),
    };
  }
}
