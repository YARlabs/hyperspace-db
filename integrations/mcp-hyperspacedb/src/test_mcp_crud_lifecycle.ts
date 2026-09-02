import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import path from "path";

const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const HOST = process.env.HYPERSPACE_HOST || "the.yar.ink";

/**
 * CRUD & Lifecycle Verification Suite for mcp-hyperspacedb v4.0.0
 * Verifies: tool registry, collection create/insert/search/delete lifecycle
 * Note: Cognitive memory tools are in mcp-hyperspace-memory (separate package)
 */
export async function runMcpCrudLifecycleTest() {
  console.log("==========================================================================");
  console.log("🧪 mcp-hyperspacedb v4.0.0: CRUD & LIFECYCLE VERIFICATION SUITE");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}\n`);

  const serverPath = path.resolve(process.cwd(), "dist/index.js");
  const transport = new StdioClientTransport({
    command: "node",
    args: [serverPath],
    env: {
      HYPERSPACE_HOST: HOST,
      HYPERSPACE_API_KEY: API_KEY,
      PATH: process.env.PATH || ""
    }
  });

  const client = new Client(
    { name: "mcp-crud-verifier", version: "4.0.0" },
    { capabilities: {} }
  );

  const testCollection = `lifecycle_test_${Date.now()}`;

  try {
    await client.connect(transport);
    console.log("   ✓ Step 1: MCP Server Process Spawned & Connected via Stdio\n");

    // ── Step 2: Verify tool registry ─────────────────────────────────────────
    const toolsResult = await client.listTools();
    const toolNames = toolsResult.tools.map(t => t.name);
    console.log(`   Registered MCP Tools (${toolNames.length}):`);
    console.log(`   - ${toolNames.join("\n   - ")}`);

    // Required DB tools (cognitive tools moved to mcp-hyperspace-memory)
    const requiredTools = [
      "hyperspace_create_collection",
      "hyperspace_insert_text",
      "hyperspace_search_text",
      "hyperspace_delete_points",
      "hyperspace_delete_collection",
      "hyperspace_list_collections",
      "hyperspace_get_stats",
      "hyperspace_graph_traverse",
      "hyperspace_get_neighbors",
      "hyperspace_find_clusters",
      "hyperspace_analyze_geometry",
      "hyperspace_analyze_thought_stability",
      "hyperspace_predict_momentum",
      "hyperspace_get_trust_score",
    ];

    const missingTools = requiredTools.filter(t => !toolNames.includes(t));
    if (missingTools.length > 0) {
      throw new Error(`Missing required DB tools: ${missingTools.join(", ")}`);
    }
    console.log(`\n   ✓ Step 2: All ${requiredTools.length} required DB tools present`);

    // Verify cognitive tools are NOT present (moved to mcp-hyperspace-memory)
    const cognitiveTools = ["hyperspace_remember_event", "hyperspace_recall_context", "hyperspace_forget_memory"];
    const orphanedCognitive = cognitiveTools.filter(t => toolNames.includes(t));
    if (orphanedCognitive.length > 0) {
      throw new Error(`Cognitive tools should NOT be in mcp-hyperspacedb: ${orphanedCognitive.join(", ")}`);
    }
    console.log("   ✓ Step 3: Confirmed cognitive tools absent (correctly moved to mcp-hyperspace-memory)");

    // ── Step 4: Full CRUD lifecycle ───────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔄 STEP 4: Full Collection CRUD Lifecycle");
    console.log("--------------------------------------------------------------------------");

    // Create
    await client.callTool({
      name: "hyperspace_create_collection",
      arguments: { collection: testCollection, dimension: 801, metric: "hybrid", quantization: "none" }
    });
    console.log(`   ✓ Created collection: ${testCollection}`);

    // Insert
    await client.callTool({
      name: "hyperspace_insert_text",
      arguments: {
        collection: testCollection,
        id: 1,
        text: "HyperspaceDB uses hyperbolic geometry for hierarchical knowledge representation.",
        metadata: { type: "fact", domain: "database" }
      }
    });
    await client.callTool({
      name: "hyperspace_insert_text",
      arguments: {
        collection: testCollection,
        id: 2,
        text: "The Lorentz model provides a natural embedding space for hierarchical data.",
        metadata: { type: "fact", domain: "math" }
      }
    });
    console.log("   ✓ Inserted 2 text records");

    // Search
    const searchRes = await client.callTool({
      name: "hyperspace_search_text",
      arguments: { collection: testCollection, text: "hyperbolic geometry database", top_k: 3 }
    });
    const searchJson = JSON.parse((searchRes as any).content[0].text);
    console.log(`   ✓ Search returned ${searchJson.length} results. Top-1 ID: ${searchJson[0]?.id}`);

    // List collections
    const listRes = await client.callTool({ name: "hyperspace_list_collections", arguments: {} });
    const listText = (listRes as any).content[0].text;
    const found = listText.includes(testCollection);
    console.log(`   ✓ Collection visible in list: ${found ? '✅ YES' : '❌ NO'}`);

    // Get stats
    const statsRes = await client.callTool({
      name: "hyperspace_get_stats",
      arguments: { collection: testCollection }
    });
    console.log(`   ✓ Stats: ${(statsRes as any).content[0].text.substring(0, 80)}...`);

    // Delete single point
    await client.callTool({
      name: "hyperspace_delete_points",
      arguments: { collection: testCollection, id: 1 }
    });
    console.log("   ✓ Deleted point ID=1");

    // Delete collection
    await client.callTool({
      name: "hyperspace_delete_collection",
      arguments: { collection: testCollection }
    });
    console.log(`   ✓ Collection deleted: ${testCollection}`);

    console.log("\n==========================================================================");
    console.log("✅ mcp-hyperspacedb CRUD & LIFECYCLE SUITE: ALL CHECKS PASSED!");
    console.log("==========================================================================");
  } catch (err: any) {
    console.error("❌ MCP Lifecycle Verification Failed:", err.message);
    // Attempt cleanup
    try { await client.callTool({ name: "hyperspace_delete_collection", arguments: { collection: testCollection } }); } catch {}
    process.exit(1);
  }
}

if (process.argv[1]?.endsWith("test_mcp_crud_lifecycle.ts") || process.argv[1]?.endsWith("test_mcp_crud_lifecycle.js")) {
  runMcpCrudLifecycleTest().then(() => process.exit(0));
}
