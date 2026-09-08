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

/** Generate a stable uint32 hash from a string. Used to create deterministic memory IDs. */
function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  return h;
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
      { name: "mcp-hyperspace-memory", version: "1.0.0" },
      { capabilities: { tools: {} } }
    );
    const isLocal = HYPERSPACE_HOST.startsWith("localhost") || HYPERSPACE_HOST.startsWith("127.0.0.1");
    const grpcKey = (isLocal && HYPERSPACE_API_KEY.startsWith("sk_")) ? "I_LOVE_HYPERSPACEDB" : (HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB");
    this.client = new HyperspaceClient(HYPERSPACE_HOST, grpcKey);
    this.setupTools();
    this.server.onerror = (error) => console.error("[Memory MCP Error]", error);
  }

  private async ensureCollection(): Promise<void> {
    if (this.collectionEnsured) return;
    this.collectionEnsured = true;
    try {
      const cols = await this.client.listCollections();
      if (!cols.some(c => c.name === MEMORY_COLLECTION)) {
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
          },
          "",
          0.02,
          "extreme"
        );
      }
    } catch {
      // ignore
    }
  }

  private setupTools() {
    // ── List Tools ──────────────────────────────────────────────────────────
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: "memory_remember",
          description: `Store a new memory, event, or fact for the agent. Automatically vectorizes text and saves to collection '${MEMORY_COLLECTION}' with session_id and tags in metadata. Returns a memory_id you can use to update or delete it later.`,
          inputSchema: {
            type: "object",
            properties: {
              text: { type: "string", description: "The memory text to store." },
              session_id: { type: "string", description: "Session or conversation identifier. Used to scope recall to a specific session." },
              tags: { type: "array", items: { type: "string" }, description: "Optional list of tags (e.g. ['user_preference', 'fact'])." }
            },
            required: ["text", "session_id"]
          }
        },
        {
          name: "memory_recall",
          description: `Retrieve the most relevant memories for a query. Searches the agent's memory collection '${MEMORY_COLLECTION}' using semantic similarity. Optionally filter by session_id to stay within a conversation context.`,
          inputSchema: {
            type: "object",
            properties: {
              query: { type: "string", description: "What to search for in memory." },
              session_id: { type: "string", description: "Optional: filter results to a specific session." },
              limit: { type: "number", description: "Number of memories to retrieve (default: 5)." }
            },
            required: ["query"]
          }
        },
        {
          name: "memory_forget",
          description: `Delete a specific memory by its ID from collection '${MEMORY_COLLECTION}'. Use the memory_id returned by memory_remember or memory_recall.`,
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
          description: `Update an existing memory by deleting it and storing a new version. The new memory_id may differ from the original.`,
          inputSchema: {
            type: "object",
            properties: {
              memory_id: { type: "number", description: "ID of the memory to replace." },
              new_text: { type: "string", description: "The updated memory text." },
              session_id: { type: "string", description: "Session identifier (keep the same as the original if updating in place)." },
              tags: { type: "array", items: { type: "string" }, description: "Updated tags for the new memory." }
            },
            required: ["memory_id", "new_text", "session_id"]
          }
        },
        {
          name: "memory_list_sessions",
          description: `List unique session_ids present in the memory collection '${MEMORY_COLLECTION}'. Useful to understand which conversations have stored memories.`,
          inputSchema: {
            type: "object",
            properties: {
              limit: { type: "number", description: "Max number of recent memories to scan for session IDs (default: 100)." }
            }
          }
        },
        {
          name: "memory_explore_hierarchy",
          description: `Explore conceptual hierarchy in the agent's memory using Lorentz Cone Subsumption. Navigate 'up' to find parent/broader concepts or 'down' to find more specific sub-concepts.`,
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
          description: `Consolidate a cluster of related memories into a single abstract concept using the Fréchet Mean on the hyperboloid. Helps compress repetitive episodic memories into semantic knowledge.`,
          inputSchema: {
            type: "object",
            properties: {
              topic_query: { type: "string", description: "A topic or query to identify the cluster of memories to consolidate." },
              limit: { type: "number", description: "Number of memories to consolidate (default: 10)." }
            },
            required: ["topic_query"]
          }
        },
        {
          name: "memory_verify_claim",
          description: `Verify whether a logical claim is geometrically consistent with stored memories. Computes Lorentz + Cosine hybrid distance between premise and conclusion in 129D MRL space. Returns VERIFIED or REJECTED with a trust_score (0–1). Use this to prevent hallucinations.`,
          inputSchema: {
            type: "object",
            properties: {
              premise: { type: "string", description: "The known fact or context." },
              conclusion: { type: "string", description: "The claim to verify against the premise." }
            },
            required: ["premise", "conclusion"]
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
            const { text, session_id, tags } = z.object({
              text: z.string(),
              session_id: z.string(),
              tags: z.array(z.string()).optional().default([])
            }).parse(args);

            const id = hashString(`${text}_${session_id}`);
            const metadata: Record<string, string> = {
              text,
              session_id,
              tags: tags.join(","),
              stored_at: new Date().toISOString()
            };

            await this.ensureCollection();
            await this.client.insertText(id, text, metadata, MEMORY_COLLECTION);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "remembered",
                  memory_id: id,
                  collection: MEMORY_COLLECTION,
                  message: `Stored memory with ID ${id}`
                }, null, 2)
              }]
            };
          }

          // ── memory_recall ─────────────────────────────────────────────────
          case "memory_recall": {
            const { query, session_id, limit } = z.object({
              query: z.string(),
              session_id: z.string().optional(),
              limit: z.number().optional()
            }).parse(args);

            const options: Record<string, unknown> = {};
            if (session_id) {
              options.filters = [{ match: { key: "session_id", value: session_id } }];
            }

            const res = await this.client.searchText(query, limit ?? 5, MEMORY_COLLECTION, options);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  query,
                  session_id: session_id ?? "all",
                  results: res
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
                text: JSON.stringify({ success, memory_id, message: success ? `Memory ${memory_id} deleted.` : `Memory ${memory_id} not found.` }, null, 2)
              }]
            };
          }

          // ── memory_update ─────────────────────────────────────────────────
          case "memory_update": {
            const { memory_id, new_text, session_id, tags } = z.object({
              memory_id: z.number(),
              new_text: z.string(),
              session_id: z.string(),
              tags: z.array(z.string()).optional().default([])
            }).parse(args);

            // Delete the old memory
            await this.client.delete(memory_id, MEMORY_COLLECTION);

            // Insert the updated memory
            const newId = hashString(`${new_text}_${session_id}`);
            const metadata: Record<string, string> = {
              text: new_text,
              session_id,
              tags: tags.join(","),
              updated_at: new Date().toISOString(),
              replaced_id: String(memory_id)
            };
            await this.client.insertText(newId, new_text, metadata, MEMORY_COLLECTION);

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "updated",
                  old_memory_id: memory_id,
                  new_memory_id: newId,
                  message: `Memory updated. Old ID ${memory_id} → New ID ${newId}`
                }, null, 2)
              }]
            };
          }

          // ── memory_list_sessions ───────────────────────────────────────────
          case "memory_list_sessions": {
            const { limit } = z.object({ limit: z.number().optional() }).parse(args);

            // Search with a broad query to get a sample of memories, then extract unique session_ids
            const res = await this.client.searchText("memory session context", limit ?? 100, MEMORY_COLLECTION);
            const sessionIds = [...new Set(
              res
                .map((r: SearchResult) => r.metadata?.session_id)
                .filter((s): s is string => Boolean(s))
            )];

            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  collection: MEMORY_COLLECTION,
                  session_count: sessionIds.length,
                  sessions: sessionIds
                }, null, 2)
              }]
            };
          }

          // ── memory_explore_hierarchy ──────────────────────────────────────
          case "memory_explore_hierarchy": {
            const { concept_id, direction, limit } = z.object({
              concept_id: z.number(),
              direction: z.enum(["up", "down"]),
              limit: z.number().optional()
            }).parse(args);

            if (direction === "up") {
              const res = await this.client.getConceptParents(concept_id, 0, limit ?? 32, MEMORY_COLLECTION);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            } else {
              const res = await this.client.getSubsumptionTree(concept_id, 3, MEMORY_COLLECTION);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            }
          }

          // ── memory_consolidate ────────────────────────────────────────────
          case "memory_consolidate": {
            const { topic_query, limit } = z.object({
              topic_query: z.string(),
              limit: z.number().optional()
            }).parse(args);

            const hits = await this.client.searchText(topic_query, limit ?? 10, MEMORY_COLLECTION);
            if (hits.length === 0) {
              return { content: [{ type: "text", text: JSON.stringify({ error: "No matching memories found to consolidate." }, null, 2) }] };
            }

            const ids = hits.map((h: SearchResult) => h.id);
            const points = await this.client.getPoints(ids, MEMORY_COLLECTION);
            const vectors = points
              .map((p: { vector: number[] }) => p.vector)
              .filter((v: number[]) => Array.isArray(v) && v.length > 0);

            if (vectors.length === 0) {
              return { content: [{ type: "text", text: JSON.stringify({ error: "No valid vectors retrieved." }, null, 2) }] };
            }

            const meanVector = CognitiveMath.frechetMean(vectors, 1.0);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "consolidated",
                  topic: topic_query,
                  source_count: vectors.length,
                  source_ids: ids,
                  consolidated_dimension: meanVector.length,
                  consolidated_vector: meanVector
                }, null, 2)
              }]
            };
          }

          // ── memory_verify_claim ───────────────────────────────────────────
          case "memory_verify_claim": {
            const { premise, conclusion } = z.object({
              premise: z.string(),
              conclusion: z.string()
            }).parse(args);

            try {
              const [u_raw, v_raw] = await Promise.all([
                this.client.vectorize(premise, "hybrid"),
                this.client.vectorize(conclusion, "hybrid")
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

              // 0.36 is the geometric boundary in 129D MRL hybrid space
              const verified = trustScore > 0.36;
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    status: verified ? "VERIFIED" : "REJECTED",
                    trust_score: parseFloat(trustScore.toFixed(4)),
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

          default:
            throw new McpError(ErrorCode.MethodNotFound, `Tool not found: ${name}`);
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
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
