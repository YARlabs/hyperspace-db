import { HyperspaceClient } from "hyperspace-sdk-ts";

export interface MemoryConfig {
  host?: string;
  apiKey?: string;
  collectionName?: string;
  quantization?: "none" | "medium_plus" | "turbo" | "extreme";
  vectorStore?: {
    config?: {
      host?: string;
      apiKey?: string;
    };
  };
}

export interface MemoryOptions {
  userId?: string;
  agentId?: string;
  runId?: string;
  limit?: number;
  metadata?: Record<string, any>;
  filters?: Record<string, any>;
}

export interface MemoryItem {
  id: string;
  memory: string;
  score?: number;
  userId?: string;
  agentId?: string;
  runId?: string;
  metadata?: Record<string, any>;
}

export interface AddMemoryResponse {
  results: Array<{
    id: string;
    event: "ADD";
    data: string;
    metadata?: Record<string, any>;
  }>;
}

export class Memory {
  private client: HyperspaceClient;
  private collection: string;
  private quantization: string;
  constructor(config: MemoryConfig = {}) {
    const host = config.host || config.vectorStore?.config?.host || process.env.HYPERSPACE_HOST || "the.yar.ink";
    const apiKey = config.apiKey || config.vectorStore?.config?.apiKey || process.env.HYPERSPACE_API_KEY || process.env.YAR_API_KEY || process.env.CDE_API_KEY || "YOUR_YARINK_API_KEY";
    this.collection = config.collectionName || "agent_memories";
    this.quantization = config.quantization || "extreme";

    const isLocal = host.startsWith("localhost") || host.startsWith("127.0.0.1");
    const grpcKey = (isLocal && apiKey.startsWith("sk_")) ? "I_LOVE_HYPERSPACEDB" : (apiKey || "I_LOVE_HYPERSPACEDB");
    this.client = new HyperspaceClient(host, grpcKey);
    this.ensureCollection();
  }

  private async ensureCollection(): Promise<void> {
    try {
      const schema = {
        components: [
          {
            name: "default",
            metric: "hybrid",
            fullDimension: 801,
            weight: 1.0
          }
        ],
        cascadePipeline: [
          {
            componentName: "default",
            cutoffDimension: 129,
            storeInRam: true,
            rerankTopK: 50
          }
        ]
      };
      await (this.client as any).createCollection(
        this.collection,
        schema,
        "",
        0.02,
        this.quantization
      );
    } catch {
      // Collection already exists or handled by server
    }
  }

  private generateNumericId(strId: string): number {
    let hash = 0;
    for (let i = 0; i < strId.length; i++) {
      hash = (hash << 5) - hash + strId.charCodeAt(i);
      hash |= 0;
    }
    return Math.abs(hash);
  }

  public async add(
    messages: string | Array<{ content: string; [key: string]: any }>,
    options: MemoryOptions = {}
  ): Promise<AddMemoryResponse> {
    let text = "";
    if (Array.isArray(messages)) {
      text = messages.map(m => (typeof m === "string" ? m : m.content || JSON.stringify(m))).join(" ");
    } else {
      text = String(messages);
    }

    const memoryId = `mem_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
    const numId = this.generateNumericId(memoryId);

    const metadata: Record<string, string> = {
      memory_id: memoryId,
      text: text,
      timestamp: String(Date.now())
    };

    if (options.userId) metadata.user_id = options.userId;
    if (options.agentId) metadata.agent_id = options.agentId;
    if (options.runId) metadata.run_id = options.runId;

    if (options.metadata) {
      for (const [k, v] of Object.entries(options.metadata)) {
        metadata[k] = String(v);
      }
    }

    const vector = await this.client.vectorize(text, 'hybrid');
    await this.client.insert(numId, vector, metadata, this.collection);

    return {
      results: [
        {
          id: memoryId,
          event: "ADD",
          data: text,
          metadata
        }
      ]
    };
  }

  public async search(query: string, options: MemoryOptions = {}): Promise<MemoryItem[]> {
    const limit = options.limit || 5;
    const queryVector = await this.client.vectorize(query, 'hybrid');
    const results = await this.client.search(queryVector, limit, this.collection);

    const memories: MemoryItem[] = [];
    for (const match of results) {
      const meta = (match as any).metadata || {};

      if (options.userId && meta.user_id !== options.userId) continue;
      if (options.agentId && meta.agent_id !== options.agentId) continue;
      if (options.runId && meta.run_id !== options.runId) continue;

      memories.push({
        id: meta.memory_id || String((match as any).id),
        memory: meta.text || query,
        score: (match as any).score || 0.95,
        userId: meta.user_id || options.userId,
        agentId: meta.agent_id || options.agentId,
        runId: meta.run_id || options.runId,
        metadata: meta
      });
    }

    return memories;
  }

  public async getAll(options: MemoryOptions = {}): Promise<MemoryItem[]> {
    return this.search("*", { ...options, limit: options.limit || 100 });
  }

  public async get(memoryId: string): Promise<MemoryItem | null> {
    const all = await this.getAll({ limit: 100 });
    return all.find(m => m.id === memoryId) || null;
  }

  public async update(memoryId: string, data: string): Promise<{ message: string; id: string; data: string }> {
    const numId = this.generateNumericId(memoryId);
    try {
      await this.client.delete(numId, this.collection);
    } catch {
      // Ignore if not present
    }

    const vector = await this.client.vectorize(data, 'hybrid');
    const metadata = {
      memory_id: memoryId,
      text: data,
      updated_at: String(Date.now())
    };
    await this.client.insert(numId, vector, metadata, this.collection);

    return {
      message: "Memory updated successfully",
      id: memoryId,
      data
    };
  }

  public async delete(memoryId: string): Promise<{ message: string }> {
    const numId = this.generateNumericId(memoryId);
    await this.client.delete(numId, this.collection);
    return { message: `Memory ${memoryId} deleted successfully` };
  }

  public async deleteAll(options: MemoryOptions = {}): Promise<{ message: string }> {
    const memories = await this.getAll({ ...options, limit: 1000 });
    let count = 0;
    for (const mem of memories) {
      if (mem.id) {
        await this.delete(mem.id);
        count++;
      }
    }
    return { message: `Deleted ${count} memories` };
  }

  public async reset(): Promise<{ message: string }> {
    try {
      await this.client.deleteCollection(this.collection);
    } catch {
      // Ignore error
    }
    await this.ensureCollection();
    return { message: "Memory collection reset successfully" };
  }

  public async history(memoryId: string): Promise<Array<{ id: string; event: "ADD"; data: string; timestamp?: string }>> {
    const mem = await this.get(memoryId);
    if (!mem) return [];
    return [
      {
        id: memoryId,
        event: "ADD",
        data: mem.memory,
        timestamp: mem.metadata?.timestamp
      }
    ];
  }
}
