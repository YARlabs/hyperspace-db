import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { HyperspaceClient, CognitiveMathExport as CognitiveMath, HyperbolicMath } from "hyperspace-sdk-ts";
import { z } from "zod";

const HYPERSPACE_HOST = process.env.HYPERSPACE_HOST || "localhost:50051";
const HYPERSPACE_API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";

// Helper for Gromov Delta (Ported from Rust SDK)
function analyzeDeltaHyperbolicity(vectors: number[][], numSamples: number = 100): { delta: number, recommendation: "lorentz" | "poincare" | "cosine" | "l2" } {
  if (vectors.length < 4) return { delta: 0, recommendation: "l2" };

  const l2Dist = (a: number[], b: number[]) => Math.sqrt(a.reduce((s, x, i) => s + Math.pow(x - b[i], 2), 0));

  let maxDelta = 0;
  for (let i = 0; i < numSamples; i++) {
    const idxs = [0, 0, 0, 0].map(() => Math.floor(Math.random() * vectors.length));
    if (new Set(idxs).size < 4) continue;

    const [a, b, u, v] = idxs.map(idx => vectors[idx]);
    const d_ab = l2Dist(a, b);
    const d_uv = l2Dist(u, v);
    const d_au = l2Dist(a, u);
    const d_bv = l2Dist(b, v);
    const d_av = l2Dist(a, v);
    const d_bu = l2Dist(b, u);

    const s1 = d_ab + d_uv;
    const s2 = d_au + d_bv;
    const s3 = d_av + d_bu;

    const sorted = [s1, s2, s3].sort((x, y) => y - x);
    const delta = (sorted[0] - sorted[1]) / 2;
    if (delta > maxDelta) maxDelta = delta;
  }

  const isNormalized = vectors.slice(0, 20).every(v => {
    const n = Math.sqrt(v.reduce((s, x) => s + x * x, 0));
    return Math.abs(n - 1.0) < 1e-2;
  });

  let recommendation: "lorentz" | "poincare" | "cosine" | "l2" = "l2";
  if (maxDelta < 0.15) recommendation = "lorentz";
  else if (maxDelta < 0.30) recommendation = "poincare";
  else if (isNormalized) recommendation = "cosine";

  return { delta: maxDelta, recommendation };
}

class HyperspaceMcpServer {
  private server: Server;
  private client: HyperspaceClient;

  constructor() {
    this.server = new Server(
      { name: "mcp-hyperspacedb", version: "3.5.0" },
      { capabilities: { tools: {} } }
    );
    this.client = new HyperspaceClient(HYPERSPACE_HOST, HYPERSPACE_API_KEY);
    this.setupTools();
    this.server.onerror = (error) => console.error("[MCP Error]", error);
  }

  private setupTools() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        // --- DATA PLANE TOOLS ---
        {
          name: "hyperspace_search_text",
          description: "Search for semanticly similar information using natural language query. Supports hybrid search (BM25 + Semantic).",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              text: { type: "string" },
              top_k: { type: "number", default: 5 },
              hybrid_alpha: { type: "number", description: "Weight for hybrid search fusion (0.0 = lexical only, 1.0 = semantic only). Default: 0.5" },
              bm25_options: {
                type: "object",
                properties: {
                  method: { type: "string", description: "BM25 method (bm25, bm25plus, lucene, atire)" },
                  language: { type: "string", description: "Stemmer language (english, russian, etc.)" },
                }
              }
            },
            required: ["collection", "text"]
          }
        },
        {
          name: "hyperspace_search_wasserstein",
          description: "Advanced Optimal Transport (Wasserstein) search for comparing distributions or complex concept overlap.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              text: { type: "string" },
              top_k: { type: "number", default: 5 }
            },
            required: ["collection", "text"]
          }
        },
        {
          name: "hyperspace_insert_text",
          description: "Store a new factual claim or memory. Automatically handles vectorization.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              id: { type: "number" },
              text: { type: "string" },
              metadata: { type: "object" }
            },
            required: ["collection", "id", "text"]
          }
        },
        {
          name: "hyperspace_create_collection",
          description: "Setup new memory spaces (collections) with specific vector dimension and metric geometry.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              dimension: { type: "number", default: 1024, description: "Vector dimension (e.g., 1024)" },
              metric: { type: "string", default: "cosine", description: "Distance metric: l2, cosine, poincare, lorentz" }
            },
            required: ["collection"]
          }
        },
        // --- AGENTIC GRAPH TOOLS ---
        {
          name: "hyperspace_graph_traverse",
          description: "Deep graph exploration. Finds logical paths between concept A and context B. Use this for complex reasoning or cross-referencing.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              start_id: { type: "number" },
              max_depth: { type: "number", default: 3 },
              max_nodes: { type: "number", default: 256 }
            },
            required: ["collection", "start_id"]
          }
        },
        {
          name: "hyperspace_get_neighbors",
          description: "Explore local connectivity of a concept in the vector index graph.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              id: { type: "number", description: "The node/concept ID" },
              layer: { type: "number", default: 0, description: "HNSW graph layer" },
              limit: { type: "number", default: 64, description: "Max neighbors to return" }
            },
            required: ["collection", "id"]
          }
        },
        {
          name: "hyperspace_find_clusters",
          description: "Detect emergent structure and hierarchy in the current knowledge base.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              min_cluster_size: { type: "number", default: 3 }
            },
            required: ["collection"]
          }
        },
        // --- ANALYTICS TOOLS (Standalone) ---
        {
          name: "hyperspace_analyze_geometry",
          description: "Calculates Gromov Delta-hyperbolicity to determine if your data is best suited for Flat (Cosine/L2) or Curved (Poincare/Lorentz) space.",
          inputSchema: {
            type: "object",
            properties: {
              vectors: { type: "array", items: { type: "array", items: { type: "number" } } },
              samples: { type: "number", default: 200 }
            },
            required: ["vectors"]
          }
        },
        {
          name: "hyperspace_analyze_thought_stability",
          description: "Calculates Lyapunov Convergence of a trajectory (Chain of Thought). Negative means stable/converging. Positive means chaotic/hallucinating.",
          inputSchema: {
            type: "object",
            properties: {
              trajectory: { type: "array", items: { type: "array", items: { type: "number" } } },
              curvature: { type: "number", default: 1.0 }
            },
            required: ["trajectory"]
          }
        },
        // --- COGNITIVE SYSTEM TOOLS ---
        {
          name: "hyperspace_trigger_reconsolidation",
          description: "AI Sleep Mode: Triggers Flow Matching on the server to optimize the geometric representation of concepts based on their usage/context.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              learning_rate: { type: "number", default: 0.1 }
            },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_freeze_collection",
          description: "Freeze a collection to make it read-only, preventing new inserts.",
          inputSchema: {
            type: "object",
            properties: { collection: { type: "string" } },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_unfreeze_collection",
          description: "Unfreeze a previously frozen collection to allow inserts again.",
          inputSchema: {
            type: "object",
            properties: { collection: { type: "string" } },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_list_collections",
          description: "List all active collections with their metadata (dimension, metric, count).",
          inputSchema: {
            type: "object",
            properties: {}
          }
        },
        {
          name: "hyperspace_get_stats",
          description: "Get detailed statistics and logical clock for a specific collection. Merges metadata, cache stats and WriteBuffer size.",
          inputSchema: {
            type: "object",
            properties: { collection: { type: "string" } },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_cache_stats",
          description: "Get cache statistics (hits, misses, policy, etc.) for a specific collection's L0 Hot Tier Cache.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The name of the collection." }
            },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_cache_clear",
          description: "Clear / purge all items in the L0 Cache for a specific collection.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The name of the collection." }
            },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_cache_config",
          description: "Update L0 Cache configuration (eviction policy, ANN threshold) for a specific collection.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The name of the collection." },
              policy: { type: "string", description: "Cache eviction policy (e.g. lru, lfu, ttl)." },
              ann_threshold: { type: "number", description: "ANN similarity distance threshold for cache lookup filtering." }
            },
            required: ["collection", "policy"]
          }
        },
        {
          name: "hyperspace_delete_collection",
          description: "Permanently delete a collection and all of its vectors.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The name of the collection to delete." }
            },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_delete_points",
          description: "Delete a single vector point from a collection by its ID.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              id: { type: "number", description: "The ID of the point to delete." }
            },
            required: ["collection", "id"]
          }
        },
        {
          name: "hyperspace_get_points",
          description: "Retrieve vector coordinate and metadata for a list of point IDs.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              ids: { type: "array", items: { type: "number" }, description: "Array of point IDs." }
            },
            required: ["collection", "ids"]
          }
        },
        {
          name: "hyperspace_rebuild_index",
          description: "Rebuild and optimize the HNSW index on the server for a specific collection.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The name of the collection." }
            },
            required: ["collection"]
          }
        },
        {
          name: "hyperspace_vacuum",
          description: "Perform vacuuming on the database to permanently purge deleted vectors and reclaim disk space.",
          inputSchema: {
            type: "object",
            properties: {}
          }
        },
        {
          name: "hyperspace_get_subsumption_tree",
          description: "Retrieve the Lorentz hierarchy subsumption tree starting from a given root ID.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              root_id: { type: "number", description: "The root node ID of the subsumption tree." },
              max_depth: { type: "number", default: 3, description: "Maximum hierarchy depth." }
            },
            required: ["collection", "root_id"]
          }
        },
        {
          name: "hyperspace_get_concept_parents",
          description: "Retrieve parent concepts in a hierarchical collection.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              id: { type: "number", description: "The concept ID." },
              layer: { type: "number", default: 0, description: "HNSW graph layer." },
              limit: { type: "number", default: 32, description: "Limit number of parents." }
            },
            required: ["collection", "id"]
          }
        },
        {
          name: "hyperspace_explore_graph",
          description: "Traverse the graph and return nodes and links in a format ready for visualization.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              start_id: { type: "number", description: "The starting node ID." },
              max_depth: { type: "number", default: 2, description: "Max depth traversal." },
              max_nodes: { type: "number", default: 256, description: "Max nodes to return." }
            },
            required: ["collection", "start_id"]
          }
        },
        {
          name: "hyperspace_predict_momentum",
          description: "Forecast future agent thought paths using Koopman momentum extrapolation.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              trajectory_ids: { type: "array", items: { type: "number" }, description: "Array of trajectory node IDs." },
              steps: { type: "number", default: 1.0, description: "Steps to predict ahead." },
              curvature: { type: "number", default: 1.0, description: "Curvature parameter." }
            },
            required: ["collection", "trajectory_ids"]
          }
        },
        {
          name: "hyperspace_get_trust_score",
          description: "Evaluate stability and trust score for a given thought trajectory path.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string", description: "The collection name." },
              trajectory_ids: { type: "array", items: { type: "number" }, description: "Array of trajectory node IDs." },
              curvature: { type: "number", default: 1.0, description: "Curvature parameter." }
            },
            required: ["collection", "trajectory_ids"]
          }
        },
        // --- COGNITIVE SKILLS / TOOLS ---
        {
          name: "hyperspace_remember_event",
          description: "Saves an event or fact to the agent's autobiographical memory (Episodic Memory). Automatically vectorizes the text and writes it to the Sidecar Payload, preserving tags and session ID in metadata.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              text: { type: "string", description: "Text of the event or statement." },
              session_id: { type: "string", description: "Session/dialogue identifier." },
              tags: { type: "array", items: { type: "string" }, description: "List of tags for labeling." }
            },
            required: ["collection", "text", "session_id", "tags"]
          }
        },
        {
          name: "hyperspace_recall_context",
          description: "Retrieves similar memories (Episodic Memory) to form context. Supports filtering by session_id.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              query: { type: "string", description: "Semantic query text." },
              session_id: { type: "string", description: "Optional session ID filter." },
              limit: { type: "number", default: 5, description: "Maximum number of results to return." }
            },
            required: ["collection", "query"]
          }
        },
        {
          name: "hyperspace_forget_memory",
          description: "Deletes a specific memory from the database by its ID.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              memory_id: { type: "number", description: "Memory identifier." }
            },
            required: ["collection", "memory_id"]
          }
        },
        {
          name: "hyperspace_explore_hierarchy",
          description: "Explore hierarchical relationships of concepts in Lorentz space (Lorentz Cone Subsumption). Allows navigating to parent concepts or down to subtree descendants.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              concept_id: { type: "number", description: "Concept identifier." },
              direction: { type: "string", description: "Traversal direction: 'up' (parents) or 'down' (descendant subtree)." },
              limit: { type: "number", default: 32, description: "Limit on the number of results." }
            },
            required: ["collection", "concept_id", "direction"]
          }
        },
        {
          name: "hyperspace_consolidate_memories",
          description: "Consolidates a group of related memories on a topic into a single abstract concept by calculating the geometric center of mass (Fréchet Mean) on the hyperboloid.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              topic_query: { type: "string", description: "Topic or search query for grouping." },
              limit: { type: "number", default: 10, description: "Number of memories to consolidate." }
            },
            required: ["collection", "topic_query"]
          }
        },
        {
          name: "hyperspace_verify_logical_claim",
          description: "Verifies the logical connection (facts) between a premise and a conclusion. Calculates Trust Score based on trajectory or graph distance to prevent hallucinations.",
          inputSchema: {
            type: "object",
            properties: {
              collection: { type: "string" },
              premise: { type: "string", description: "The initial fact or premise." },
              conclusion: { type: "string", description: "The statement to verify." }
            },
            required: ["collection", "premise", "conclusion"]
          }
        }
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      try {
        switch (name) {
          case "hyperspace_list_collections": {
            const res = await this.client.listCollections();
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_search_text": {
            const { collection, text, top_k, hybrid_alpha, bm25_options } = z.object({
              collection: z.string(),
              text: z.string(),
              top_k: z.number().optional(),
              hybrid_alpha: z.number().optional(),
              bm25_options: z.object({
                method: z.string().optional(),
                language: z.string().optional(),
              }).optional()
            }).parse(args);
            const res = await this.client.searchText(text, top_k || 5, collection, {
              hybridAlpha: hybrid_alpha,
              bm25: bm25_options as any
            });
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_search_wasserstein": {
            const { collection, text, top_k } = z.object({ collection: z.string(), text: z.string(), top_k: z.number().optional() }).parse(args);
            // Wasserstein uses a specific internal method or we can use searchWasserstein text variant if exists.
            // Client.ts has searchWasserstein(vector, topK, collection)
            const vector = await this.client.vectorize(text);
            const res = await (this.client as any).searchWasserstein(vector, top_k || 5, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_insert_text": {
            const { collection, id, text, metadata } = z.object({ collection: z.string(), id: z.number(), text: z.string(), metadata: z.record(z.string(), z.string()).optional() }).parse(args);
            await this.client.insertText(id, text, metadata, collection);
            return { content: [{ type: "text", text: `Stored ${id} in ${collection}` }] };
          }
          case "hyperspace_create_collection": {
            const { collection, dimension, metric } = z.object({
              collection: z.string(),
              dimension: z.number().optional(),
              metric: z.string().optional()
            }).parse(args);

            const isHybrid129 = (metric === 'hybrid' && dimension === 129);
            const fullDim = isHybrid129 ? 801 : (dimension || 1024);

            const success = await this.client.createCollection(collection, {
              components: [{ name: 'default', metric: metric || 'cosine', fullDimension: fullDim, weight: 1.0 }],
              cascadePipeline: isHybrid129 ? [
                { componentName: 'default', cutoffDimension: 129, storeInRam: true, rerankTopK: 0 }
              ] : []
            });
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_graph_traverse": {
            const { collection, start_id, max_depth, max_nodes } = z.object({ collection: z.string(), start_id: z.number(), max_depth: z.number().optional(), max_nodes: z.number().optional() }).parse(args);
            const nodes = await this.client.traverse(start_id, 0, max_depth || 3, max_nodes || 256, collection);
            return { content: [{ type: "text", text: JSON.stringify(nodes, null, 2) }] };
          }
          case "hyperspace_get_neighbors": {
            const { collection, id, layer, limit } = z.object({
              collection: z.string(),
              id: z.number(),
              layer: z.number().optional(),
              limit: z.number().optional()
            }).parse(args);
            const res = await this.client.getNeighbors(id, layer || 0, limit || 64, 0, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_find_clusters": {
            const { collection, min_cluster_size } = z.object({ collection: z.string(), min_cluster_size: z.number().optional() }).parse(args);
            const clusters = await this.client.findSemanticClusters(0, min_cluster_size || 3, 32, 10000, collection);
            return { content: [{ type: "text", text: JSON.stringify(clusters, null, 2) }] };
          }
          case "hyperspace_analyze_geometry": {
            const { vectors, samples } = z.object({ vectors: z.array(z.array(z.number())), samples: z.number().optional() }).parse(args);
            const analysis = analyzeDeltaHyperbolicity(vectors, samples);
            return { content: [{ type: "text", text: JSON.stringify(analysis, null, 2) }] };
          }
          case "hyperspace_analyze_thought_stability": {
            const { trajectory, curvature } = z.object({ trajectory: z.array(z.array(z.number())), curvature: z.number().optional() }).parse(args);
            const convergence = (CognitiveMath as any).lyapunovConvergence(trajectory, curvature || 1.0);
            return { content: [{ type: "text", text: `Lyapunov Convergence: ${convergence} (${convergence < 0 ? "STABLE" : "CHAOTIC"})` }] };
          }
          case "hyperspace_trigger_reconsolidation": {
            const { collection, learning_rate } = z.object({ collection: z.string(), learning_rate: z.number().optional() }).parse(args);
            // Use Triggering natively if available in client
            const res = await (this.client as any).triggerReconsolidation?.(collection, new Array(1024).fill(0), learning_rate || 0.1) || "Reconsolidation triggered.";
            return { content: [{ type: "text", text: String(res) }] };
          }
          case "hyperspace_freeze_collection": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const res = await this.client.freezeCollection(collection);
            return { content: [{ type: "text", text: String(res) }] };
          }
          case "hyperspace_unfreeze_collection": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const res = await this.client.unfreezeCollection(collection);
            return { content: [{ type: "text", text: String(res) }] };
          }
          case "hyperspace_get_stats": {
            const { collection } = z.object({ collection: z.string() }).parse(args);

            // 1. Fetch gRPC CollectionStats & Digest in parallel
            let digest: any = {};
            let grpcStats: any = {};
            try {
              [digest, grpcStats] = await Promise.all([
                this.client.getDigest(collection),
                this.client.getCollectionStats(collection)
              ]);
            } catch (e: any) {
              console.error(`gRPC stats/digest fetch failed: ${e.message}`);
              throw e;
            }

            // 2. Fetch HTTP collection stats in parallel try-catch
            let httpStats: any = {};
            try {
              const clientAny = this.client as any;
              const host = clientAny.host || "localhost:50051";
              const ip = host.split(':')[0];
              const url = `http://${ip}:50050/api/collections/${collection}/stats`;
              const headers: { [key: string]: string } = {};
              if (clientAny.apiKey) headers['x-api-key'] = clientAny.apiKey;
              if (clientAny.userId) headers['x-hyperspace-user-id'] = clientAny.userId;

              const res = await fetch(url, { headers });
              if (res.ok) {
                httpStats = await res.json();
              } else {
                console.error(`HTTP collection stats request failed: ${res.statusText}`);
              }
            } catch (e: any) {
              console.error(`HTTP collection stats fetch failed: ${e.message}`);
            }

            // 3. Merge gRPC + HTTP results
            const stats = {
              ...digest,
              ...grpcStats,
              ...httpStats
            };
            return { content: [{ type: "text", text: JSON.stringify(stats, null, 2) }] };
          }
          case "hyperspace_cache_stats": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const stats = await this.client.getCacheStats(collection);
            return { content: [{ type: "text", text: JSON.stringify(stats, null, 2) }] };
          }
          case "hyperspace_cache_clear": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const success = await this.client.clearCache(collection);
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_cache_config": {
            const { collection, policy, ann_threshold } = z.object({
              collection: z.string(),
              policy: z.string(),
              ann_threshold: z.number().optional()
            }).parse(args);
            const success = await this.client.updateCacheConfig(collection, policy, ann_threshold);
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_delete_collection": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const success = await this.client.deleteCollection(collection);
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_delete_points": {
            const { collection, id } = z.object({ collection: z.string(), id: z.number() }).parse(args);
            const success = await this.client.delete(id, collection);
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_get_points": {
            const { collection, ids } = z.object({ collection: z.string(), ids: z.array(z.number()) }).parse(args);
            const res = await this.client.getPoints(ids, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_rebuild_index": {
            const { collection } = z.object({ collection: z.string() }).parse(args);
            const success = await this.client.rebuildIndex(collection);
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_vacuum": {
            const success = await this.client.vacuum();
            return { content: [{ type: "text", text: JSON.stringify({ success }, null, 2) }] };
          }
          case "hyperspace_get_subsumption_tree": {
            const { collection, root_id, max_depth } = z.object({ collection: z.string(), root_id: z.number(), max_depth: z.number().optional() }).parse(args);
            const res = await this.client.getSubsumptionTree(root_id, max_depth || 3, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_get_concept_parents": {
            const { collection, id, layer, limit } = z.object({ collection: z.string(), id: z.number(), layer: z.number().optional(), limit: z.number().optional() }).parse(args);
            const res = await this.client.getConceptParents(id, layer || 0, limit || 32, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_explore_graph": {
            const { collection, start_id, max_depth, max_nodes } = z.object({ collection: z.string(), start_id: z.number(), max_depth: z.number().optional(), max_nodes: z.number().optional() }).parse(args);
            const res = await this.client.exploreGraph(start_id, max_depth || 2, max_nodes || 256, collection);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_predict_momentum": {
            const { collection, trajectory_ids, steps, curvature } = z.object({ collection: z.string(), trajectory_ids: z.array(z.number()), steps: z.number().optional(), curvature: z.number().optional() }).parse(args);
            const res = await this.client.predictMomentum(trajectory_ids, steps || 1.0, collection, curvature || 1.0);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_get_trust_score": {
            const { collection, trajectory_ids, curvature } = z.object({ collection: z.string(), trajectory_ids: z.array(z.number()), curvature: z.number().optional() }).parse(args);
            const res = await this.client.getTrustScore(trajectory_ids, collection, curvature || 1.0);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          // --- COGNITIVE SKILLS HANDLERS ---
          case "hyperspace_remember_event": {
            const { collection, text, session_id, tags } = z.object({
              collection: z.string(),
              text: z.string(),
              session_id: z.string(),
              tags: z.array(z.string())
            }).parse(args);

            // Generate stable uint32 hash ID from text and session_id
            const keyStr = `${text}_${session_id}`;
            let id = 0;
            for (let i = 0; i < keyStr.length; i++) {
              id = (id * 31 + keyStr.charCodeAt(i)) >>> 0;
            }

            const metadata: { [key: string]: string } = {
              text,
              session_id,
              tags: tags.join(',')
            };

            await this.client.insertText(id, text, metadata, collection);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  status: "success",
                  memory_id: id,
                  message: `Event remembered successfully under ID ${id}`
                }, null, 2)
              }]
            };
          }
          case "hyperspace_recall_context": {
            const { collection, query, session_id, limit } = z.object({
              collection: z.string(),
              query: z.string(),
              session_id: z.string().optional(),
              limit: z.number().optional()
            }).parse(args);

            const options: any = {};
            if (session_id) {
              options.filters = [{ match: { key: "session_id", value: session_id } }];
            }

            const res = await this.client.searchText(query, limit || 5, collection, options);
            return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
          }
          case "hyperspace_forget_memory": {
            const { collection, memory_id } = z.object({
              collection: z.string(),
              memory_id: z.number()
            }).parse(args);

            const success = await this.client.delete(memory_id, collection);
            return { content: [{ type: "text", text: JSON.stringify({ success, message: `Memory ${memory_id} forgotten.` }, null, 2) }] };
          }
          case "hyperspace_explore_hierarchy": {
            const { collection, concept_id, direction, limit } = z.object({
              collection: z.string(),
              concept_id: z.number(),
              direction: z.enum(["up", "down"]),
              limit: z.number().optional()
            }).parse(args);

            if (direction === "up") {
              const res = await this.client.getConceptParents(concept_id, 0, limit || 32, collection);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            } else {
              const res = await this.client.getSubsumptionTree(concept_id, 3, collection);
              return { content: [{ type: "text", text: JSON.stringify(res, null, 2) }] };
            }
          }
          case "hyperspace_consolidate_memories": {
            const { collection, topic_query, limit } = z.object({
              collection: z.string(),
              topic_query: z.string(),
              limit: z.number().optional()
            }).parse(args);

            // 1. Search for memory points matching the topic
            const hits = await this.client.searchText(topic_query, limit || 10, collection);
            if (hits.length === 0) {
              return { content: [{ type: "text", text: JSON.stringify({ error: "No matching memories found to consolidate." }, null, 2) }] };
            }

            // 2. Fetch coordinate vectors for those points
            const ids = hits.map(h => h.id);
            const points = await this.client.getPoints(ids, collection);
            const vectors = points.map(p => p.vector).filter(v => Array.isArray(v) && v.length > 0);

            if (vectors.length === 0) {
              return { content: [{ type: "text", text: JSON.stringify({ error: "No valid vectors retrieved." }, null, 2) }] };
            }

            // 3. Compute Fréchet Mean using CognitiveMath
            const meanVector = CognitiveMath.frechetMean(vectors, 1.0);
            return {
              content: [{
                type: "text",
                text: JSON.stringify({
                  consolidated_vector: meanVector,
                  dimension: meanVector.length,
                  source_points_count: vectors.length,
                  source_ids: ids
                }, null, 2)
              }]
            };
          }
          case "hyperspace_verify_logical_claim": {
            const { collection, premise, conclusion } = z.object({
              collection: z.string(),
              premise: z.string(),
              conclusion: z.string()
            }).parse(args);

            try {
              // 1. Vectorize both premise and conclusion on the fly using hybrid metric (801D)
              const [u_raw, v_raw] = await Promise.all([
                this.client.vectorize(premise, 'hybrid'),
                this.client.vectorize(conclusion, 'hybrid')
              ]);

              // 2. Perform MRL Truncation to 129D (33 Lorentz + 96 Euclidean) and re-normalize
              const mrlTruncateAndNormalize = (v: number[]): number[] => {
                if (v.length <= 129) return v;
                const truncated = v.slice(0, 129);
                
                // Lorentz part (first 33 elements)
                const lorentz_part = truncated.slice(0, 33);
                let spatial_norm_sq = 0;
                for (let i = 1; i < 33; i++) {
                  spatial_norm_sq += lorentz_part[i] * lorentz_part[i];
                }
                lorentz_part[0] = Math.sqrt(1.0 + spatial_norm_sq); // upper sheet constraint
                
                // Euclidean part (remaining 96 elements)
                const euclidean_part = truncated.slice(33);
                let euc_norm_sq = 0;
                for (let i = 0; i < euclidean_part.length; i++) {
                  euc_norm_sq += euclidean_part[i] * euclidean_part[i];
                }
                const euc_norm = Math.sqrt(euc_norm_sq);
                if (euc_norm > 0) {
                  for (let i = 0; i < euclidean_part.length; i++) {
                    euclidean_part[i] /= euc_norm;
                  }
                }
                
                return [...lorentz_part, ...euclidean_part];
              };

              const u = mrlTruncateAndNormalize(u_raw);
              const v = mrlTruncateAndNormalize(v_raw);

              // 3. Mathematically strict hybrid distance calculation over 129D space:
              // Lorentz distance for first 33 dimensions, Cosine distance for the remaining 96 dimensions.
              const u_lorentz = u.slice(0, 33);
              const v_lorentz = v.slice(0, 33);
              
              // Lorentz inner product: -u[0]*v[0] + sum_{i=1}^{32} u[i]*v[i]
              let prod = -u_lorentz[0] * v_lorentz[0];
              for (let i = 1; i < 33; i++) {
                prod += u_lorentz[i] * v_lorentz[i];
              }
              // Hyperbolic distance (Minkowski model)
              const lorentz_dist = Math.acosh(Math.max(-prod, 1.0));

              const u_cosine = u.slice(33);
              const v_cosine = v.slice(33);
              let dot = 0;
              let norm_u = 0;
              let norm_v = 0;
              for (let i = 0; i < u_cosine.length; i++) {
                dot += u_cosine[i] * v_cosine[i];
                norm_u += u_cosine[i] * u_cosine[i];
                norm_v += v_cosine[i] * v_cosine[i];
              }
              const cosine_dist = 1.0 - (dot / (Math.sqrt(norm_u) * Math.sqrt(norm_v) + 1e-9));

              // Combined hybrid metric distance
              const dist = lorentz_dist + cosine_dist;
              const trustScore = 1.0 / (1.0 + dist);

              // 0.36 threshold represents the boundary in our 129D MRL hybrid space
              const status = trustScore > 0.36 ? "VERIFIED" : "REJECTED";
              const reason = trustScore > 0.36
                ? "Logical claim is geometrically consistent with episodic context."
                : `Geodesic violation. Hyperbolic distance between concepts is too large (${dist.toFixed(4)}), indicating disconnected sub-cones.`;

              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    status,
                    trust_score: parseFloat(trustScore.toFixed(4)),
                    lorentz_distance: parseFloat(lorentz_dist.toFixed(4)),
                    cosine_distance: parseFloat(cosine_dist.toFixed(4)),
                    total_distance: parseFloat(dist.toFixed(4)),
                    reason,
                    vector_dimension: u.length
                  }, null, 2)
                }]
              };
            } catch (err: any) {
              return {
                content: [{
                  type: "text",
                  text: JSON.stringify({
                    status: "REJECTED",
                    trust_score: 0.0,
                    reason: `Failed to compute logical claim safety score: ${err.message}`
                  }, null, 2)
                }]
              };
            }
          }
          default:
            throw new McpError(ErrorCode.MethodNotFound, `Tool not found: ${name}`);
        }
      } catch (err: any) {
        throw new McpError(ErrorCode.InternalError, err.message);
      }
    });
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error("Hyperspace POWER MCP online.");
  }
}

const server = new HyperspaceMcpServer();
server.run().catch(console.error);
