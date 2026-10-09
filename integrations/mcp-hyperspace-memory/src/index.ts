import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { HyperspaceClient, CognitiveMathExport as CognitiveMath, SearchResult } from "hyperspace-sdk-ts";
import { z } from "zod";

// ─── Configuration ────────────────────────────────────────────────────────────
// SaaS endpoint is the default — no HYPERSPACE_HOST needed for most users.
const rawHost = process.env.HYPERSPACE_HOST || "the.yar.ink";
const HYPERSPACE_HOST = rawHost.replace(/^https?:\/\//, "").replace(/\/$/, "");
const HYPERSPACE_API_KEY = process.env.HYPERSPACE_API_KEY || process.env.YAR_API_KEY || process.env.CDE_API_KEY || "";

// MEMORY_COLLECTION: set once in env, agent never needs to pass it per-call.
// Defaults to "agent_cognitive_memories_129" on SaaS for zero-config experience.
const MEMORY_COLLECTION = process.env.MEMORY_COLLECTION || "agent_cognitive_memories_129";

if (!HYPERSPACE_API_KEY) {
  console.error("[mcp-hyperspace-memory] WARNING: HYPERSPACE_API_KEY is not set. Set it in your MCP config env.");
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Generate a stable uint32 hash from a string. Clamped to [1, 10_000_000] for memory safety. */
function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  return (h % 10000000) + 1;
}

/** MRL Truncation to 129D + re-normalize for hybrid (Lorentz + Euclidean) vectors. */
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

// ─── MCP Server ───────────────────────────────────────────────────────────────

class HyperspaceMemoryServer {
  private server: Server;
  private client: HyperspaceClient;

  private collectionEnsured: boolean = false;

  constructor() {
    this.server = new Server(
      { name: "mcp-hyperspace-memory", version: "4.0.0" },
      { capabilities: { tools: {} } }
    );
    const isLocal = HYPERSPACE_HOST.startsWith("localhost") || HYPERSPACE_HOST.startsWith("127.0.0.1");
    const grpcKey = (isLocal && process.env.HYPERSPACE_LOCAL_ADMIN_KEY)
      ? process.env.HYPERSPACE_LOCAL_ADMIN_KEY
      : (HYPERSPACE_API_KEY || "");
    this.client = new HyperspaceClient(HYPERSPACE_HOST, grpcKey);
    this.setupTools();
    this.server.onerror = (error) => console.error("[Memory MCP Error]", error);
  }

  private async withRetry<T>(fn: () => Promise<T>, retries = 3, delayMs = 300): Promise<T> {
    let lastError: any;
    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        return await fn();
      } catch (err: any) {
        lastError = err;
        const msg = String(err?.message || err);
        const isTransient = msg.includes("502") || msg.includes("UNAVAILABLE") || msg.includes("transport error") || msg.includes("connection reset");
        if (!isTransient || attempt === retries) {
          throw err;
        }
        await new Promise((resolve) => setTimeout(resolve, delayMs * Math.pow(2, attempt)));
      }
    }
    throw lastError;
  }

  private async ensureCollection(): Promise<void> {
    if (this.collectionEnsured) return;
    this.collectionEnsured = true;
    const isLocal = HYPERSPACE_HOST.startsWith("localhost") || HYPERSPACE_HOST.startsWith("127.0.0.1");
    if (!isLocal) {
      // On SaaS / remote cloud, user collections are managed and pre-provisioned
      return;
    }
    try {
      const cols = await this.client.listCollections().catch(() => []);
      if (cols.some(c => c.name === MEMORY_COLLECTION)) {
        return;
      }
      await this.client.createCollection(
        MEMORY_COLLECTION,
        {
          components: [{ name: "default", metric: "hybrid", fullDimension: 801, weight: 1.0 }],
          cascadePipeline: [
            {
              componentName: "default",
              cutoffDimension: 129,
              storeInRam: true,
              rerankTopK: 50
            }
          ]
        }
      ).catch(() => { });
    } catch {
      // Ignore
    }
  }

  /**
   * Fast & robust vectorization: uses the continuous cloud embeddings API directly
   * on SaaS or delegates to local server when running on localhost.
   */
  private async vectorize(text: string): Promise<number[]> {
    const apiKey = HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
    if (apiKey.startsWith("sk_")) {
      try {
        const res = await fetch("https://the.yar.ink/v1/embeddings", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${apiKey}`
          },
          body: JSON.stringify({ model: "v5_Light", input: text })
        });
        if (res.ok) {
          const json: any = await res.json();
          if (json.data && json.data[0] && json.data[0].embedding) {
            return json.data[0].embedding;
          }
        }
      } catch { }
    }
    try {
      return await this.client.vectorize(text, "hybrid");
    } catch { }
    return this.withRetry(async () => {
      const res = await fetch("https://the.yar.ink/v1/embeddings", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${apiKey}`
        },
        body: JSON.stringify({ model: "v5_Light", input: text })
      });
      const json: any = await res.json();
      return json.data[0].embedding;
    });
  }

  /**
   * Resilient memory insertion helper with automatic dual-mode fallback and retry:
   * Uses fast cloud vectorization + payload attachment for optimal latency and persistence.
   */
  private async storeMemoryText(id: number, text: string, metadata: Record<string, string>): Promise<void> {
    await this.ensureCollection();
    const vec = await this.vectorize(text);
    await this.withRetry(() =>
      this.client.insert(id, vec, metadata, MEMORY_COLLECTION, 0, undefined, Buffer.from(text, "utf-8"))
    );
  }

  /**
   * Resilient memory search helper using direct vector similarity with retry.
   */
  private async searchMemoryText(query: string, limit: number, options?: any): Promise<SearchResult[]> {
    await this.ensureCollection();
    const vec = await this.vectorize(query);
    return await this.withRetry(() =>
      this.client.search(vec, limit, MEMORY_COLLECTION, options)
    );
  }

  private setupTools() {
    // ── List Tools ──────────────────────────────────────────────────────────
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: "memory_remember",
          description: `Store a new memory, event, fact, or procedure for the agent in collection '${MEMORY_COLLECTION}'. Automatically vectorizes text and stores with session_id, tags, memory_type (semantic, episodic, procedural), and importance (1-10). Returns a JSON object with memory_id, status, and metadata summary.`,
          inputSchema: {
            type: "object",
            properties: {
              text: { type: "string", description: "The memory text to store." },
              session_id: { type: "string", description: "Session or conversation identifier. Scopes recall to a specific conversation." },
              tags: { type: "array", items: { type: "string" }, description: "Optional list of tags (e.g. ['user_preference', 'fact'])." },
              memory_type: {
                type: "string",
                enum: ["semantic", "episodic", "procedural"],
                description: "Type of memory: 'semantic' (facts, preferences), 'episodic' (events, context), 'procedural' (instructions, rules). Default: 'episodic'."
              },
              importance: {
                type: "number",
                description: "Importance level from 1 (minor detail) to 10 (critical core fact). Default: 5."
              }
            },
            required: ["text", "session_id"]
          }
        },
        {
          name: "memory_recall",
          description: `Retrieve the most relevant memories for a query from collection '${MEMORY_COLLECTION}' using semantic similarity. Filter by session_id, memory_type ('semantic'|'episodic'|'procedural'), or min_importance (1-10). Returns a JSON object with query, filters, and ranked memories with cognitive scores.`,
          inputSchema: {
            type: "object",
            properties: {
              query: { type: "string", description: "What to search for in memory." },
              session_id: { type: "string", description: "Optional: filter results to a specific session identifier." },
              memory_type: {
                type: "string",
                enum: ["semantic", "episodic", "procedural"],
                description: "Optional: filter by memory type."
              },
              min_importance: {
                type: "number",
                description: "Optional: filter memories with importance >= min_importance (1-10)."
              },
              limit: { type: "number", description: "Number of memories to retrieve (default: 5)." }
            },
            required: ["query"]
          }
        },
        {
          name: "memory_forget",
          description: `Delete a specific memory by numeric memory_id from collection '${MEMORY_COLLECTION}'. Returns a JSON object with success status and confirmation message.`,
          inputSchema: {
            type: "object",
            properties: {
              memory_id: { type: "number", description: "The memory ID to delete." }
            },
            required: ["memory_id"]
          }
        },
        {
          name: "memory_update",
          description: `Update an existing memory by deleting memory_id and storing new_text with refreshed embeddings and metadata. Returns a JSON object with old_memory_id, new_memory_id, and status.`,
          inputSchema: {
            type: "object",
            properties: {
              memory_id: { type: "number", description: "ID of the memory to replace." },
              new_text: { type: "string", description: "The updated memory text." },
              session_id: { type: "string", description: "Session identifier (keep the same as the original if updating in place)." },
              tags: { type: "array", items: { type: "string" }, description: "Updated tags for the new memory." },
              memory_type: {
                type: "string",
                enum: ["semantic", "episodic", "procedural"],
                description: "Updated memory type. Default: 'episodic'."
              },
              importance: {
                type: "number",
                description: "Updated importance level (1-10). Default: 5."
              }
            },
            required: ["memory_id", "new_text", "session_id"]
          }
        },
        {
          name: "memory_list_sessions",
          description: `Discover unique session_ids and active conversation threads present in collection '${MEMORY_COLLECTION}'. Uses multi-probe vector queries to ensure comprehensive session discovery. Returns a JSON object with session_count and list of sessions with memory counts and recent sample previews.`,
          inputSchema: {
            type: "object",
            properties: {
              limit: { type: "number", description: "Max number of candidate memories to scan across domain probes (default: 100)." }
            }
          }
        },
        {
          name: "memory_explore_hierarchy",
          description: `Explore conceptual hierarchy in memory using Lorentz Cone Subsumption. 'up' navigates to parent/hypernym concepts; 'down' returns the subsumption tree of descendant concepts. Returns a JSON structure of the hierarchy.`,
          inputSchema: {
            type: "object",
            properties: {
              concept_id: { type: "number", description: "The memory/concept ID to start from." },
              direction: { type: "string", enum: ["up", "down"], description: "'up' → broader parent concepts. 'down' → specific descendant concepts." },
              limit: { type: "number", description: "Max results (default: 32)." }
            },
            required: ["concept_id", "direction"]
          }
        },
        {
          name: "memory_consolidate",
          description: `Consolidate a cluster of related episodic memories into a single abstract semantic concept using the Fréchet Mean on the hyperboloid. Optionally stores the synthesized summary_text back into memory (persist: true) and cleans up source memories (archive_sources: true). Returns a JSON report with consolidated_id, mean vector, and source IDs.`,
          inputSchema: {
            type: "object",
            properties: {
              topic_query: { type: "string", description: "A topic or query to identify the cluster of memories to consolidate." },
              limit: { type: "number", description: "Number of memories to consolidate (default: 10)." },
              summary_text: { type: "string", description: "Optional: synthesized summary or abstract statement representing the consolidated concept." },
              persist: { type: "boolean", description: "Whether to store the consolidated concept into memory (default: true)." },
              archive_sources: { type: "boolean", description: "Whether to delete/purge the source episodic memories after consolidation (default: false)." }
            },
            required: ["topic_query"]
          }
        },
        {
          name: "memory_verify_claim",
          description: `Verify whether a logical claim (conclusion) is geometrically consistent with a premise in 129D MRL hybrid space (Lorentz + Cosine). Returns a JSON object with status ('VERIFIED'|'REJECTED'), trust_score (0.0-1.0), geodesic distances, and geometric explanation to prevent hallucinations.`,
          inputSchema: {
            type: "object",
            properties: {
              premise: { type: "string", description: "The known fact or context." },
              conclusion: { type: "string", description: "The claim to verify against the premise." },
              threshold: { type: "number", description: "Trust score threshold (default: 0.36)." }
            },
            required: ["premise", "conclusion"]
          }
        },
        {
          name: "memory_stats",
          description: `Get operational statistics and health diagnostics for the memory collection '${MEMORY_COLLECTION}'. Returns a JSON object with total vector count, dimension, metric, cascade settings, and host information.`,
          inputSchema: {
            type: "object",
            properties: {}
          }
        }
      ]
    }));

    // ── Call Tool ───────────────────────────────────────────────────────────
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      try {
        switch (name) {
          // ── memory_remember ──────────────────────────────────────────────
          case "memory_remember": {
            const { text, session_id, tags, memory_type, importance } = z.object({
              text: z.string(),
              session_id: z.string(),
              tags: z.array(z.string()).optional().default([]),
              memory_type: z.enum(["semantic", "episodic", "procedural"]).optional().default("episodic"),
              importance: z.number().min(1).max(10).optional().default(5)
            }).parse(args);

            const id = hashString(`${text}_${session_id}`);
            const now = new Date().toISOString();
            const metadata: Record<string, string> = {
              text,
              session_id,
              tags: tags.join(","),
              memory_type,
              importance: String(importance),
              stored_at: now,
              last_accessed_at: now,
              access_count: "0"
            };

            await this.storeMemoryText(id, text, metadata);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "remembered",
                  memory_id: id,
                  collection: MEMORY_COLLECTION,
                  memory_type,
                  importance,
                  session_id,
                  message: `Stored ${memory_type} memory with ID ${id}`
                }, null, 2)
              }]
            };
          }

          // ── memory_recall ─────────────────────────────────────────────────
          case "memory_recall": {
            const { query, session_id, memory_type, min_importance, limit } = z.object({
              query: z.string(),
              session_id: z.string().optional(),
              memory_type: z.enum(["semantic", "episodic", "procedural"]).optional(),
              min_importance: z.number().min(1).max(10).optional(),
              limit: z.number().optional().default(5)
            }).parse(args);

            const hasFilters = Boolean(session_id || memory_type || min_importance !== undefined);
            const candidateLimit = hasFilters ? Math.max(limit * 8, 50) : limit;

            const options: Record<string, unknown> = {};
            if (session_id) {
              options.filter = { session_id };
            }

            const rawResults = await this.searchMemoryText(query, candidateLimit, options);

            // Filter by session_id, memory_type, and min_importance
            const filteredResults = rawResults.filter((r: SearchResult) => {
              if (session_id && r.metadata?.session_id && r.metadata.session_id !== session_id) return false;
              if (memory_type && r.metadata?.memory_type && r.metadata.memory_type !== memory_type) return false;
              if (min_importance !== undefined) {
                const imp = parseFloat(r.metadata?.importance || "5");
                if (imp < min_importance) return false;
              }
              return true;
            });

            // Compute cognitive score with importance boost and recency decay
            const cognitiveResults = filteredResults.map((r: SearchResult) => {
              const imp = parseFloat(r.metadata?.importance || "5");
              const storedAt = r.metadata?.stored_at ? new Date(r.metadata.stored_at).getTime() : Date.now();
              const hoursElapsed = Math.max(0, (Date.now() - storedAt) / (1000 * 3600));
              // Subtle half-life decay over 1 week (168h), minimum floor 0.85
              const recencyFactor = Math.max(0.85, Math.exp(-0.005 * hoursElapsed));
              const rawSimilarity = 1.0 / (1.0 + Math.max(0, r.distance));
              const importanceBoost = 1.0 + 0.05 * (imp - 5);
              const cognitiveScore = parseFloat((rawSimilarity * importanceBoost * recencyFactor).toFixed(4));

              return {
                id: r.id,
                distance: parseFloat(r.distance.toFixed(4)),
                cognitive_score: cognitiveScore,
                text: r.metadata?.text || "",
                memory_type: r.metadata?.memory_type || "episodic",
                importance: imp,
                session_id: r.metadata?.session_id || "",
                tags: r.metadata?.tags ? r.metadata.tags.split(",").filter(Boolean) : [],
                stored_at: r.metadata?.stored_at,
                metadata: r.metadata
              };
            });

            // Sort by cognitive_score descending and slice to requested limit
            cognitiveResults.sort((a, b) => b.cognitive_score - a.cognitive_score);
            const finalResults = cognitiveResults.slice(0, limit);

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  query,
                  session_id: session_id ?? "all",
                  memory_type: memory_type ?? "all",
                  total_found: finalResults.length,
                  results: finalResults
                }, null, 2)
              }]
            };
          }

          // ── memory_forget ─────────────────────────────────────────────────
          case "memory_forget": {
            const { memory_id } = z.object({ memory_id: z.number() }).parse(args);
            const success = await this.client.delete(memory_id, MEMORY_COLLECTION);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  success,
                  memory_id,
                  message: success ? `Memory ${memory_id} deleted.` : `Memory ${memory_id} not found.`
                }, null, 2)
              }]
            };
          }

          // ── memory_update ─────────────────────────────────────────────────
          case "memory_update": {
            const { memory_id, new_text, session_id, tags, memory_type, importance } = z.object({
              memory_id: z.number(),
              new_text: z.string(),
              session_id: z.string(),
              tags: z.array(z.string()).optional().default([]),
              memory_type: z.enum(["semantic", "episodic", "procedural"]).optional().default("episodic"),
              importance: z.number().min(1).max(10).optional().default(5)
            }).parse(args);

            // Delete the old memory
            await this.client.delete(memory_id, MEMORY_COLLECTION);

            // Insert the updated memory
            const newId = hashString(`${new_text}_${session_id}`);
            const now = new Date().toISOString();
            const metadata: Record<string, string> = {
              text: new_text,
              session_id,
              tags: tags.join(","),
              memory_type,
              importance: String(importance),
              updated_at: now,
              stored_at: now,
              replaced_id: String(memory_id),
              last_accessed_at: now,
              access_count: "0"
            };
            await this.storeMemoryText(newId, new_text, metadata);

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "updated",
                  old_memory_id: memory_id,
                  new_memory_id: newId,
                  memory_type,
                  importance,
                  message: `Memory updated. Old ID ${memory_id} → New ID ${newId}`
                }, null, 2)
              }]
            };
          }

          // ── memory_list_sessions ───────────────────────────────────────────
          case "memory_list_sessions": {
            const { limit } = z.object({ limit: z.number().optional().default(100) }).parse(args);

            // Multi-probe domain sampling across diverse semantic clusters for robust session discovery
            const probes = [
              "memory session conversation",
              "user interaction topic",
              "agent decision action",
              "system knowledge fact"
            ];

            const probeResults = await Promise.all(
              probes.map(q =>
                this.searchMemoryText(q, Math.min(limit, 50))
                  .catch(() => [] as SearchResult[])
              )
            );

            const sessionMap = new Map<string, { count: number; sample?: string; last_stored?: string }>();
            for (const hit of probeResults.flat()) {
              const sid = hit.metadata?.session_id;
              if (sid) {
                const existing = sessionMap.get(sid);
                if (!existing) {
                  sessionMap.set(sid, {
                    count: 1,
                    sample: hit.metadata?.text ? hit.metadata.text.slice(0, 80) : undefined,
                    last_stored: hit.metadata?.stored_at
                  });
                } else {
                  existing.count += 1;
                }
              }
            }

            const sessions = Array.from(sessionMap.entries()).map(([session_id, data]) => ({
              session_id,
              observed_count: data.count,
              sample: data.sample,
              last_stored: data.last_stored
            }));

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  collection: MEMORY_COLLECTION,
                  session_count: sessions.length,
                  sessions
                }, null, 2)
              }]
            };
          }

          // ── memory_explore_hierarchy ──────────────────────────────────────
          case "memory_explore_hierarchy": {
            const { concept_id, direction, limit } = z.object({
              concept_id: z.number(),
              direction: z.enum(["up", "down"]),
              limit: z.number().optional().default(32)
            }).parse(args);

            if (direction === "up") {
              const res = await this.client.getConceptParents(concept_id, 0, limit, MEMORY_COLLECTION);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            } else {
              const res = await this.client.getSubsumptionTree(concept_id, 3, MEMORY_COLLECTION);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            }
          }

          // ── memory_consolidate ────────────────────────────────────────────
          case "memory_consolidate": {
            const { topic_query, limit, summary_text, persist, archive_sources } = z.object({
              topic_query: z.string(),
              limit: z.number().optional().default(10),
              summary_text: z.string().optional(),
              persist: z.boolean().optional().default(true),
              archive_sources: z.boolean().optional().default(false)
            }).parse(args);

            const hits = await this.searchMemoryText(topic_query, limit);
            if (hits.length === 0) {
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({ error: "No matching memories found to consolidate." }, null, 2)
                }]
              };
            }

            const ids = hits.map((h: SearchResult) => h.id);
            const points = await this.client.getPoints(ids, MEMORY_COLLECTION);
            const vectors = points
              .map((p: { vector: number[] }) => p.vector)
              .filter((v: number[]) => Array.isArray(v) && v.length > 0);

            if (vectors.length === 0) {
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({ error: "No valid vectors retrieved." }, null, 2)
                }]
              };
            }

            const meanVector = CognitiveMath.frechetMean(vectors, 1.0);
            const consolidatedSummary = summary_text || `[Consolidated] ${topic_query}: synthesized from ${vectors.length} memories`;
            let consolidatedId: number | undefined;

            if (persist) {
              consolidatedId = hashString(`consolidated_${topic_query}_${Date.now()}`);
              const now = new Date().toISOString();
              const metadata: Record<string, string> = {
                text: consolidatedSummary,
                session_id: "consolidated_global",
                tags: `consolidated,${topic_query}`,
                memory_type: "semantic",
                importance: "8",
                source_count: String(vectors.length),
                source_ids: ids.join(","),
                stored_at: now,
                last_accessed_at: now,
                access_count: "0"
              };
              await this.storeMemoryText(consolidatedId, consolidatedSummary, metadata);
            }

            if (archive_sources) {
              await Promise.all(ids.map(id => this.client.delete(id, MEMORY_COLLECTION).catch(() => { })));
            }

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "consolidated",
                  topic: topic_query,
                  consolidated_id: consolidatedId,
                  summary: consolidatedSummary,
                  source_count: vectors.length,
                  source_ids: ids,
                  persisted: persist,
                  archived_sources: archive_sources,
                  consolidated_dimension: meanVector.length,
                  frechet_mean_preview: meanVector.slice(0, 6)
                }, null, 2)
              }]
            };
          }

          // ── memory_verify_claim ───────────────────────────────────────────
          case "memory_verify_claim": {
            const { premise, conclusion, threshold } = z.object({
              premise: z.string(),
              conclusion: z.string(),
              threshold: z.number().optional().default(0.30)
            }).parse(args);

            try {
              const [u_raw, v_raw] = await Promise.all([
                this.vectorize(premise),
                this.vectorize(conclusion)
              ]);

              const u = mrlTruncateAndNormalize(u_raw);
              const v = mrlTruncateAndNormalize(v_raw);

              // Lorentz distance (hyperbolic, first 33D)
              const u_l = u.slice(0, 33);
              const v_l = v.slice(0, 33);
              let prod = -u_l[0] * v_l[0];
              for (let i = 1; i < 33; i++) prod += u_l[i] * v_l[i];
              const lorentz_dist = Math.acosh(Math.max(-prod, 1.0));

              // Cosine distance (Euclidean, remaining 96D)
              const u_e = u.slice(33);
              const v_e = v.slice(33);
              let dot = 0, norm_u = 0, norm_v = 0;
              for (let i = 0; i < u_e.length; i++) {
                dot += u_e[i] * v_e[i];
                norm_u += u_e[i] * u_e[i];
                norm_v += v_e[i] * v_e[i];
              }
              const cosine_dist = 1.0 - dot / (Math.sqrt(norm_u) * Math.sqrt(norm_v) + 1e-9);

              const dist = lorentz_dist + cosine_dist;
              const trustScore = 1.0 / (1.0 + dist);
              const verified = trustScore > threshold;

              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    status: verified ? "VERIFIED" : "REJECTED",
                    trust_score: parseFloat(trustScore.toFixed(4)),
                    threshold,
                    lorentz_distance: parseFloat(lorentz_dist.toFixed(4)),
                    cosine_distance: parseFloat(cosine_dist.toFixed(4)),
                    total_distance: parseFloat(dist.toFixed(4)),
                    reason: verified
                      ? "Claim is geometrically consistent with the premise in hyperbolic space."
                      : `Geodesic violation: distance ${dist.toFixed(4)} exceeds threshold. Concepts are in disconnected sub-cones.`,
                    vector_dimension: u.length
                  }, null, 2)
                }]
              };
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    status: "REJECTED",
                    trust_score: 0.0,
                    reason: `Vectorization failed: ${msg}`
                  }, null, 2)
                }]
              };
            }
          }

          // ── memory_stats ──────────────────────────────────────────────────
          case "memory_stats": {
            await this.ensureCollection();
            const cols = await this.client.listCollections().catch(() => []);
            const target = cols.find(c => c.name === MEMORY_COLLECTION);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  collection: MEMORY_COLLECTION,
                  host: HYPERSPACE_HOST,
                  status: target ? "ready" : "not_initialized",
                  approx_vector_count: target ? target.count : 0,
                  schema: target?.schema || {
                    metric: "hybrid",
                    fullDimension: 801,
                    ramMrlDimension: 129
                  }
                }, null, 2)
              }]
            };
          }

          default:
            throw new McpError(ErrorCode.MethodNotFound, `Tool not found: ${name}`);
        }
      } catch (err: unknown) {
        console.error(`[MCP Error in ${name}]:`, err);
        const msg = err instanceof Error ? err.stack || err.message : String(err);
        throw new McpError(ErrorCode.InternalError, msg);
      }
    });
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error(
      `[mcp-hyperspace-memory] Online. Collection: '${MEMORY_COLLECTION}' | Host: ${HYPERSPACE_HOST}`
    );
  }
}

const server = new HyperspaceMemoryServer();
server.run().catch(console.error);
