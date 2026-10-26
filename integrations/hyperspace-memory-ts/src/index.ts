import { HyperspaceClient, CognitiveMathExport as CognitiveMath, SearchResult } from "hyperspace-sdk-ts";

export interface MemoryConfig {
  host?: string;
  apiKey?: string;
  collectionName?: string;
  quantization?: "none" | "medium_plus" | "turbo" | "extreme";
  encryptionKey?: string;
  decayLambda?: number;
  autoSupersede?: boolean;
  vectorStore?: {
    config?: {
      host?: string;
      apiKey?: string;
      encryptionKey?: string;
    };
  };
}

export interface MemoryOptions {
  userId?: string;
  agentId?: string;
  runId?: string;
  sessionId?: string;
  tags?: string[];
  importance?: number;
  memoryType?: "semantic" | "episodic" | "procedural";
  limit?: number;
  minScore?: number;
  applyDecay?: boolean;
  metadata?: Record<string, any>;
  filters?: Record<string, any>;
}

export interface MemoryItem {
  id: string;
  memory: string;
  score?: number;
  rawSimilarity?: number;
  isSuperseded?: boolean;
  userId?: string;
  agentId?: string;
  runId?: string;
  sessionId?: string;
  tags?: string[];
  importance?: number;
  metadata?: Record<string, any>;
}

export interface AddMemoryResponse {
  results: Array<{
    id: string;
    event: "ADD";
    data: string;
    metadata?: Record<string, any>;
    supersededMemories?: string[];
  }>;
}

export interface ConsolidateOptions {
  limit?: number;
  summaryText?: string;
  persist?: boolean;
  archiveSources?: boolean;
}

export interface ConsolidateResult {
  status: string;
  topic: string;
  consolidatedId?: string;
  summary: string;
  sourceCount: number;
  sourceIds: string[];
  persisted: boolean;
  archivedSources: boolean;
  consolidatedDimension: number;
  frechetMeanPreview?: number[];
}

export interface ClaimVerificationResult {
  status: "VERIFIED" | "REJECTED";
  trustScore: number;
  threshold: number;
  lorentzDistance: number;
  cosineDistance: number;
  totalDistance: number;
  reason: string;
  vectorDimension: number;
}

export interface MemoryStatsResult {
  collection: string;
  host: string;
  status: string;
  approxVectorCount: number;
  schema: any;
}

export interface MemoryStats {
  collection: string;
  host: string;
  status: string;
  approxVectorCount: number;
  encrypted?: boolean;
  schema?: any;
}

/** MRL Truncation to 129D + upper-sheet Lorentz normalization */
function mrlTruncateAndNormalize(v: number[]): number[] {
  if (v.length <= 129) return v;
  const truncated = v.slice(0, 129);

  // Lorentz part (first 33 elements) — enforce upper-sheet constraint
  const lorentz = truncated.slice(0, 33);
  let spatialNormSq = 0;
  for (let i = 1; i < 33; i++) spatialNormSq += lorentz[i] * lorentz[i];
  lorentz[0] = Math.sqrt(1.0 + spatialNormSq);

  // Euclidean part (remaining 96 elements) — L2 normalize
  const euclidean = truncated.slice(33);
  let eucNormSq = 0;
  for (const x of euclidean) eucNormSq += x * x;
  const eucNorm = Math.sqrt(eucNormSq);
  if (eucNorm > 0) {
    for (let i = 0; i < euclidean.length; i++) euclidean[i] /= eucNorm;
  }

  return [...lorentz, ...euclidean];
}

export class Memory {
  private client: HyperspaceClient;
  private collection: string;
  private quantization: string;
  private host: string;
  private encryptionKey?: string;
  private decayLambda: number;
  private autoSupersede: boolean;

  constructor(config: MemoryConfig = {}) {
    const host = config.host || config.vectorStore?.config?.host || process.env.HYPERSPACE_HOST || "the.yar.ink";
    const apiKey = config.apiKey || config.vectorStore?.config?.apiKey || process.env.HYPERSPACE_API_KEY || process.env.YAR_API_KEY || process.env.CDE_API_KEY || "YOUR_YARINK_API_KEY";
    this.host = host;
    this.collection = config.collectionName || "agent_memories";
    this.quantization = config.quantization || "extreme";
    this.encryptionKey = config.encryptionKey || config.vectorStore?.config?.encryptionKey || process.env.HYPERSPACE_ENCRYPTION_KEY;
    this.decayLambda = config.decayLambda ?? 0.005;
    this.autoSupersede = config.autoSupersede ?? true;

    const isLocal = host.startsWith("localhost") || host.startsWith("127.0.0.1");
    const grpcKey = (isLocal && apiKey.startsWith("sk_")) ? "I_LOVE_HYPERSPACEDB" : (apiKey || "I_LOVE_HYPERSPACEDB");
    this.client = new HyperspaceClient(host, grpcKey);
    this.ensureCollection();
  }

  private async ensureCollection(): Promise<void> {
    try {
      const schema: any = {
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
        ],
        encrypted: !!this.encryptionKey
      };
      if (this.encryptionKey) {
        this.client.registerCollectionKey(this.collection, this.encryptionKey, "hybrid", 0.0, schema);
      }
      await (this.client as any).createCollection(
        this.collection,
        schema,
        this.encryptionKey || "",
        0.0,
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
    const nowTs = Date.now();

    const metadata: Record<string, string> = {
      memory_id: memoryId,
      text: text,
      timestamp: String(nowTs),
      created_at: String(nowTs),
      last_accessed_at: String(nowTs),
      access_count: "0",
      is_superseded: "false",
      stored_at: new Date(nowTs).toISOString()
    };

    if (options.userId) metadata.user_id = options.userId;
    if (options.agentId) metadata.agent_id = options.agentId;
    if (options.runId) metadata.run_id = options.runId;
    if (options.sessionId) metadata.session_id = options.sessionId;
    if (options.memoryType) metadata.memory_type = options.memoryType;
    if (options.importance !== undefined) metadata.importance = String(options.importance);
    if (options.tags && options.tags.length > 0) metadata.tags = options.tags.join(",");

    if (options.metadata) {
      for (const [k, v] of Object.entries(options.metadata)) {
        metadata[k] = String(v);
      }
    }

    const vector = await this.client.vectorize(text, 'hybrid');
    await this.client.insert(numId, vector, metadata, this.collection, 0, undefined, Buffer.from(text, "utf-8"));

    // Autonomous conflict resolution: find previous conflicting/outdated memories
    const supersededIds: string[] = [];
    if (this.autoSupersede && text.length > 10) {
      try {
        const candidates = await this.search(text, { userId: options.userId, sessionId: options.sessionId, limit: 3 });
        for (const cand of candidates) {
          if (cand.id && cand.id !== memoryId && (cand.score ?? 0) >= 0.82) {
            const candNumId = this.generateNumericId(cand.id);
            await this.link(numId, candNumId, "SUPERSEDES", cand.score ?? 1.0);
            supersededIds.push(cand.id);
          }
        }
      } catch {
        // Silently skip if induction fails
      }
    }

    return {
      results: [
        {
          id: memoryId,
          event: "ADD",
          data: text,
          metadata,
          supersededMemories: supersededIds
        }
      ]
    };
  }

  public async search(query: string, options: MemoryOptions = {}): Promise<MemoryItem[]> {
    const limit = options.limit || 5;
    const queryVector = await this.client.vectorize(query, 'hybrid');
    // Oversample candidates for cognitive decay re-ranking
    const candidateLimit = Math.max(limit * 4, 25);
    const results = await this.client.search(queryVector, candidateLimit, this.collection);

    const nowTs = Date.now();
    const applyDecay = options.applyDecay ?? true;
    const memories: MemoryItem[] = [];

    for (const match of results) {
      const meta = (match as any).metadata || {};

      if (options.userId && meta.user_id !== options.userId) continue;
      if (options.agentId && meta.agent_id !== options.agentId) continue;
      if (options.runId && meta.run_id !== options.runId) continue;
      if (options.sessionId && meta.session_id !== options.sessionId) continue;

      const rawSimilarity = (match as any).score ?? (
        (match as any).distance !== undefined
          ? 1.0 / (1.0 + Math.max(0, (match as any).distance))
          : 0.95
      );

      let finalScore = rawSimilarity;
      if (applyDecay) {
        const storedTs = parseFloat(meta.last_accessed_at || meta.timestamp || String(nowTs));
        const hoursElapsed = Math.max(0, (nowTs - storedTs) / (3600 * 1000));
        const recencyDecay = Math.exp(-this.decayLambda * hoursElapsed);
        const accessCount = parseInt(meta.access_count || "0", 10);
        const accessBoost = 1.0 + 0.05 * Math.log(1.0 + accessCount);
        const imp = parseFloat(meta.importance || "5.0");
        const importanceBoost = 1.0 + 0.05 * (imp - 5.0);
        const isSuperseded = meta.is_superseded === "true";
        const supersedePenalty = isSuperseded ? 0.20 : 1.0;

        finalScore = rawSimilarity * recencyDecay * accessBoost * importanceBoost * supersedePenalty;
      }

      if (options.minScore !== undefined && finalScore < options.minScore) continue;

      memories.push({
        id: meta.memory_id || String((match as any).id),
        memory: meta.text || query,
        score: parseFloat(finalScore.toFixed(4)),
        rawSimilarity: parseFloat(rawSimilarity.toFixed(4)),
        isSuperseded: meta.is_superseded === "true",
        userId: meta.user_id || options.userId,
        agentId: meta.agent_id || options.agentId,
        runId: meta.run_id || options.runId,
        sessionId: meta.session_id || options.sessionId,
        tags: meta.tags ? meta.tags.split(",") : undefined,
        importance: meta.importance ? parseFloat(meta.importance) : undefined,
        metadata: meta
      });
    }

    // Rank by cognitive score
    memories.sort((a, b) => (b.score ?? 0) - (a.score ?? 0));
    return memories.slice(0, limit);
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
    await this.client.insert(numId, vector, metadata, this.collection, 0, undefined, Buffer.from(data, "utf-8"));

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

  /**
   * Enumerate active sessions observed in the memory collection.
   */
  public async listSessions(limit: number = 100): Promise<Array<{ sessionId: string; observedCount: number; sample?: string }>> {
    const probes = [
      "memory session conversation",
      "user interaction topic",
      "agent decision action",
      "system knowledge fact"
    ];

    const probeResults = await Promise.all(
      probes.map(q =>
        this.search(q, { limit: Math.min(limit, 50) }).catch(() => [] as MemoryItem[])
      )
    );

    const sessionMap = new Map<string, { count: number; sample?: string }>();
    for (const mem of probeResults.flat()) {
      const sid = mem.sessionId;
      if (sid) {
        const existing = sessionMap.get(sid);
        if (!existing) {
          sessionMap.set(sid, {
            count: 1,
            sample: mem.memory ? mem.memory.slice(0, 80) : undefined
          });
        } else {
          existing.count += 1;
        }
      }
    }

    return Array.from(sessionMap.entries()).map(([sessionId, data]) => ({
      sessionId,
      observedCount: data.count,
      sample: data.sample
    }));
  }

  /**
   * Consolidate a cluster of related episodic memories into a single abstract semantic concept
   * using the Fréchet Mean on the hyperboloid.
   */
  public async consolidate(topicQuery: string, options: ConsolidateOptions = {}): Promise<ConsolidateResult> {
    const limit = options.limit || 10;
    const persist = options.persist ?? true;
    const archiveSources = options.archiveSources ?? false;

    const hits = await this.search(topicQuery, { limit });
    if (hits.length === 0) {
      throw new Error(`No matching memories found to consolidate for topic: "${topicQuery}"`);
    }

    const ids = hits.map(h => this.generateNumericId(h.id));
    const points = await this.client.getPoints(ids, this.collection);
    const vectors = points
      .map((p: any) => p.vector)
      .filter((v: number[]) => Array.isArray(v) && v.length > 0);

    if (vectors.length === 0) {
      throw new Error("No valid vectors retrieved for consolidation.");
    }

    const meanVector = CognitiveMath.frechetMean(vectors, 1.0);
    const summary = options.summaryText || `[Consolidated] ${topicQuery}: synthesized from ${vectors.length} memories`;
    let consolidatedId: string | undefined;

    if (persist) {
      consolidatedId = `consolidated_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
      const numId = this.generateNumericId(consolidatedId);
      const metadata: Record<string, string> = {
        memory_id: consolidatedId,
        text: summary,
        session_id: "consolidated_global",
        tags: `consolidated,${topicQuery}`,
        memory_type: "semantic",
        importance: "8",
        source_count: String(vectors.length),
        source_ids: hits.map(h => h.id).join(","),
        stored_at: new Date().toISOString()
      };
      await this.client.insert(numId, meanVector, metadata, this.collection, 0, undefined, Buffer.from(summary, "utf-8"));
    }

    if (archiveSources) {
      await Promise.all(hits.map(h => this.delete(h.id).catch(() => {})));
    }

    return {
      status: "consolidated",
      topic: topicQuery,
      consolidatedId,
      summary,
      sourceCount: vectors.length,
      sourceIds: hits.map(h => h.id),
      persisted: persist,
      archivedSources: archiveSources,
      consolidatedDimension: meanVector.length,
      frechetMeanPreview: meanVector.slice(0, 6)
    };
  }

  /**
   * Verify whether a logical claim is geometrically consistent with a premise in 129D MRL hybrid space
   * (Lorentz + Cosine) to block hallucinations.
   */
  public async verifyClaim(premise: string, conclusion: string, threshold: number = 0.30): Promise<ClaimVerificationResult> {
    const [uRaw, vRaw] = await Promise.all([
      this.client.vectorize(premise, 'hybrid'),
      this.client.vectorize(conclusion, 'hybrid')
    ]);

    const u = mrlTruncateAndNormalize(uRaw);
    const v = mrlTruncateAndNormalize(vRaw);

    // Lorentz distance (hyperbolic, first 33D)
    const uL = u.slice(0, 33);
    const vL = v.slice(0, 33);
    let prod = -uL[0] * vL[0];
    for (let i = 1; i < 33; i++) prod += uL[i] * vL[i];
    const lorentzDist = Math.acosh(Math.max(-prod, 1.0));

    // Cosine distance (Euclidean, remaining 96D)
    const uE = u.slice(33);
    const vE = v.slice(33);
    let dot = 0, normU = 0, normV = 0;
    for (let i = 0; i < uE.length; i++) {
      dot += uE[i] * vE[i];
      normU += uE[i] * uE[i];
      normV += vE[i] * vE[i];
    }
    const cosineDist = 1.0 - dot / (Math.sqrt(normU) * Math.sqrt(normV) + 1e-9);

    const dist = lorentzDist + cosineDist;
    const trustScore = 1.0 / (1.0 + dist);
    const verified = trustScore > threshold;

    return {
      status: verified ? "VERIFIED" : "REJECTED",
      trustScore: parseFloat(trustScore.toFixed(4)),
      threshold,
      lorentzDistance: parseFloat(lorentzDist.toFixed(4)),
      cosineDistance: parseFloat(cosineDist.toFixed(4)),
      totalDistance: parseFloat(dist.toFixed(4)),
      reason: verified
        ? "Claim is geometrically consistent with premise in hyperbolic space."
        : `Geodesic violation: distance ${dist.toFixed(4)} exceeds threshold.`,
      vectorDimension: u.length
    };
  }

  /**
   * Traverse concept taxonomy in Lorentz space.
   */
  public async exploreHierarchy(conceptId: number, direction: "up" | "down" = "down", limit: number = 32): Promise<any> {
    if (direction === "up") {
      return await (this.client as any).getConceptParents(conceptId, 0, limit, this.collection);
    } else {
      return await (this.client as any).getSubsumptionTree(conceptId, 3, this.collection);
    }
  }

  /**
   * Create explicit directed edge between memories or concepts in the native KAG graph.
   */
  public async link(
    sourceId: number,
    targetId: number,
    relation: string = 'OWNS',
    weight: number = 1.0
  ): Promise<boolean> {
    await this.ensureCollection();
    try {
      return await (this.client as any).link(sourceId, targetId, relation, weight, this.collection);
    } catch {
      return false;
    }
  }

  /**
   * Batch-insert explicit directed edges between memories/entities in CSR topology.
   */
  public async linkBatch(
    edges: Array<{ sourceId: number; targetId: number; relation?: string; weight?: number }>
  ): Promise<number> {
    await this.ensureCollection();
    try {
      return await (this.client as any).linkBatch(edges, this.collection);
    } catch {
      return 0;
    }
  }

  /**
   * Delete an explicit directed edge with zero-lag invalidation.
   */
  public async unlink(
    sourceId: number,
    targetId: number,
    relation: string = 'OWNS'
  ): Promise<boolean> {
    await this.ensureCollection();
    try {
      return await (this.client as any).unlink(sourceId, targetId, relation, this.collection);
    } catch {
      return false;
    }
  }

  /**
   * Induce topological edges from metadata properties across memory entries.
   */
  public async induceEdges(
    rules: Array<{ field: string; ruleType: string; targetField?: string; relation?: string; weight?: number }>
  ): Promise<number> {
    await this.ensureCollection();
    try {
      return await (this.client as any).induceEdgesFromMetadata(rules, this.collection);
    } catch {
      return 0;
    }
  }

  /**
   * Get operational statistics and diagnostics for the memory collection.
   */
  public async stats(): Promise<MemoryStatsResult> {
    await this.ensureCollection();
    const cols = await this.client.listCollections().catch(() => []);
    const target = cols.find((c: any) => c.name === this.collection);
    return {
      collection: this.collection,
      host: this.host,
      status: target ? "ready" : "not_initialized",
      approxVectorCount: target ? (target as any).count : 0,
      schema: (target as any)?.schema || {
        metric: "hybrid",
        fullDimension: 801,
        ramMrlDimension: 129
      }
    };
  }
}

export * from "./query_planner.js";
