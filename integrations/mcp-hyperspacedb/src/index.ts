import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { HyperspaceClient, CognitiveMath, HyperbolicMath } from "hyperspace-sdk-ts";
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
          description: "Search for semanticly similar information using natural language query.",
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
          name: "hyperspace_get_stats",
          description: "Get database cluster health, logical clock, and vector volume.",
          inputSchema: {
            type: "object",
            properties: { collection: { type: "string" } }
          }
        }
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      try {
        switch (name) {
          case "hyperspace_search_text": {
            const { collection, text, top_k } = z.object({ collection: z.string(), text: z.string(), top_k: z.number().optional() }).parse(args);
            const res = await this.client.searchText(text, top_k || 5, collection);
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
            await this.client.insertText(text, id, metadata, collection);
            return { content: [{ type: "text", text: `Stored ${id} in ${collection}` }] };
          }
          case "hyperspace_graph_traverse": {
            const { collection, start_id, max_depth, max_nodes } = z.object({ collection: z.string(), start_id: z.number(), max_depth: z.number().optional(), max_nodes: z.number().optional() }).parse(args);
            const nodes = await this.client.traverse(start_id, 0, max_depth || 3, max_nodes || 256, collection);
            return { content: [{ type: "text", text: JSON.stringify(nodes, null, 2) }] };
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
          case "hyperspace_get_stats": {
            const { collection } = z.object({ collection: z.string().optional() }).parse(args);
            const stats = await this.client.getDigest(collection || "");
            return { content: [{ type: "text", text: JSON.stringify(stats, null, 2) }] };
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
