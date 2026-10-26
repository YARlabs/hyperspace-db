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
    encryptionKey?: string;
  }) {
    this._client = config.client;
    this.collectionName = config.collectionName;
    if (config.encryptionKey) {
      this._client.registerCollectionKey(this.collectionName, config.encryptionKey);
    }
    this._ensureCollection();
  }

  private async _ensureCollection() {
    try {
      const collections = (await this._client.listCollections()) as any[];
      const existing = collections.find(c => c.name === this.collectionName);
      if (existing) {
        const comp = existing.schema?.components?.[0];
        const dim = comp?.fullDimension;
        const metric = comp?.metric;
        console.log(`Using existing collection ${this.collectionName}: ${dim}d, ${metric}`);
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
      
      await this._client.insert(id, node.embedding, metadata, this.collectionName);
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
    if (query.mode !== VectorStoreQueryMode.DEFAULT && query.mode !== VectorStoreQueryMode.HYBRID) {
      throw new Error("Only default and hybrid query modes are supported");
    }

    const results = await this._client.search(
      query.queryEmbedding!,
      query.similarityTopK || 10,
      this.collectionName,
      {
        hybridAlpha: query.mode === VectorStoreQueryMode.HYBRID ? (query.alpha ?? 0.5) : undefined,
        hybridQuery: query.mode === VectorStoreQueryMode.HYBRID ? query.queryStr : undefined,
      }
    );

    return {
      nodes: results.map((r: any) => ({
        id_: r.id.toString(),
        metadata: r.metadata,
      } as any)),
      similarities: results.map((r: any) => r.distance),
      ids: results.map((r: any) => r.id.toString()),
    };
  }

  async getCacheStats(): Promise<any> {
    return this._client.getCacheStats(this.collectionName);
  }

  async clearCache(): Promise<boolean> {
    return this._client.clearCache(this.collectionName);
  }

  async updateCacheConfig(policy: string, annThreshold?: number): Promise<boolean> {
    return this._client.updateCacheConfig(this.collectionName, policy, annThreshold);
  }

  async link(sourceId: number, targetId: number, relation: string = "OWNS", weight: number = 1.0): Promise<boolean> {
    return this._client.link(sourceId, targetId, relation, weight, this.collectionName);
  }

  async linkBatch(edges: Array<{ sourceId: number; targetId: number; relation?: string; weight?: number }>): Promise<number> {
    return this._client.linkBatch(edges, this.collectionName);
  }

  async unlink(sourceId: number, targetId: number, relation: string = "OWNS"): Promise<boolean> {
    return this._client.unlink(sourceId, targetId, relation, this.collectionName);
  }

  async induceEdgesFromMetadata(
    rules: Array<{ field: string; ruleType: string; targetField?: string; relation?: string; weight?: number }>
  ): Promise<number> {
    return this._client.induceEdgesFromMetadata(rules, this.collectionName);
  }
}

export {
  HyperspaceQueryPlanner,
  evaluateQueryLocally,
  clarifyAndSplitIntent,
  QUERY_PLANNER_TOOL_SCHEMA,
  QueryPlannerInput,
  QueryPlannerResult,
  ClarificationPayload,
  ExecutionStrategy
} from "./query_planner";

