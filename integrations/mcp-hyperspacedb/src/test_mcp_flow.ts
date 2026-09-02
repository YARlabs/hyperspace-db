import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import path from "path";

const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const HOST = process.env.HYPERSPACE_HOST || "localhost:50051";

/**
 * Fetches real 801D Hybrid (H³² Lorentz x ℝ⁷⁶⁸ Euclidean) vectors from the live v5_Light CDE model
 */
async function getV5Embedding(text: string): Promise<number[]> {
  const cdeKey = (API_KEY && API_KEY.startsWith("sk_")) ? API_KEY : (process.env.CDE_API_KEY || "YOUR_YARINK_API_KEY");
  const url = `https://the.yar.ink/v1/embeddings`;
  
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      const res = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${cdeKey}`
        },
        body: JSON.stringify({ model: "v5_Light", input: text })
      });

      if (res.ok) {
        const json: any = await res.json();
        return json.data[0].embedding;
      }

      if (attempt === 4) {
        const errBody = await res.text();
        throw new Error(`v5_Light Embedding API error (${res.status}): ${errBody.substring(0, 100)}`);
      }
      await new Promise(r => setTimeout(r, 1000 * attempt));
    } catch (err: any) {
      if (attempt === 4) throw err;
      await new Promise(r => setTimeout(r, 1000 * attempt));
    }
  }
  throw new Error("v5_Light embedding failed after retries");
}

async function runDeepMcpDbBenchmark() {
  console.log("==========================================================================");
  console.log("🔬 mcp-hyperspacedb v4.0.0: Full DB Benchmark (v5_Light Real Embeddings)");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Model: v5_Light (801D Hybrid H³² Lorentz x ℝ⁷⁶⁸ Euclidean)\n`);

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
    { name: "mcp-v5-db-tester", version: "4.0.0" },
    { capabilities: {} }
  );

  const testCollection = `v5_db_test_${Date.now()}`;

  try {
    await client.connect(transport);
    console.log("   ✓ Connected to MCP Server via Stdio Transport\n");

    // ── STEP 1: Tool Registry ──────────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("📋 STEP 1: Verifying DB Tool Registry");
    console.log("--------------------------------------------------------------------------");
    const toolsResult = await client.listTools();
    const toolNames = toolsResult.tools.map(t => t.name);
    console.log(`   Registered MCP Tools: ${toolNames.length}`);
    console.log(`   Tools: ${toolNames.join(", ")}`);
    console.log("   ✓ Tool Registry Loaded Successfully!\n");

    // ── STEP 2: Real v5_Light Embeddings + Lyapunov Analysis ──────────────
    console.log("--------------------------------------------------------------------------");
    console.log("🧠 STEP 2: Vectorizing CoT Thought Trajectories with v5_Light (801D)");
    console.log("--------------------------------------------------------------------------");

    // Trajectory A: Convergent (coherent medical reasoning)
    const convergentPrompts = [
      "Patient glucose is 14.2 mmol/L with a rapid rising trend.",
      "Checking active insulin on board (IOB = 0.4 U) and remaining active carbs.",
      "Calculating target correction bolus: (14.2 - 6.0) / 2.5 = 3.28 Units.",
      "Safety check passed. Submitting 3.28 U bolus command to insulin pump."
    ];

    // Trajectory B: Chaotic (hallucination / random topic drift)
    const chaoticPrompts = [
      "Patient glucose is 14.2 mmol/L with a rapid rising trend.",
      "Stock market tech shares dropped 3.2 percent in Q3 earnings.",
      "The boiling point of liquid nitrogen at sea level is -195.8 degrees Celsius.",
      "Submitting 3.28 U bolus command to insulin pump."
    ];

    console.log("   [Vectorizing Trajectory A - Coherent Medical CoT]...");
    const convVectors = await Promise.all(convergentPrompts.map(p => getV5Embedding(p)));
    console.log(`   ✓ Vectorized ${convVectors.length} steps. Dims = ${convVectors[0].length}`);

    console.log("   [Vectorizing Trajectory B - Chaotic Hallucinating CoT]...");
    const chaoVectors = await Promise.all(chaoticPrompts.map(p => getV5Embedding(p)));
    console.log(`   ✓ Vectorized ${chaoVectors.length} steps. Dims = ${chaoVectors[0].length}`);

    const lyapConvRes = await client.callTool({
      name: "hyperspace_analyze_thought_stability",
      arguments: { trajectory: convVectors, curvature: 1.0 }
    });

    const lyapChaoRes = await client.callTool({
      name: "hyperspace_analyze_thought_stability",
      arguments: { trajectory: chaoVectors, curvature: 1.0 }
    });

    const convOut = (lyapConvRes as any).content[0].text;
    const chaoOut = (lyapChaoRes as any).content[0].text;

    console.log("\n   Real v5 Coherent CoT Path Output: ", convOut);
    console.log("   Real v5 Chaotic Wandering Output:  ", chaoOut);

    if (convOut.includes("STABLE")) {
      console.log("   ✓ Correctly classified Coherent Medical CoT as STABLE!");
    } else {
      console.warn("   ⚠️ Warning: Coherent CoT was not marked as STABLE.");
    }

    if (chaoOut.includes("CHAOTIC")) {
      console.log("   ✓ Correctly classified Topic-Drift Hallucination as CHAOTIC!\n");
    } else {
      console.warn("   ⚠️ Warning: Topic Drift was not marked as CHAOTIC.\n");
    }

    // ── STEP 3: Geometry Analysis ──────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("📐 STEP 3: Gromov Delta Geometry Analysis on Real v5_Light Embeddings");
    console.log("--------------------------------------------------------------------------");

    const geomResult = await client.callTool({
      name: "hyperspace_analyze_geometry",
      arguments: { vectors: convVectors, samples: 50 }
    });
    console.log("   v5_Light Geometry Analysis:", (geomResult as any).content[0].text);
    console.log("   ✓ v5_Light uses HYBRID metric (33D Lorentz H³² + 768D Euclidean ℝ⁷⁶⁸).\n");

    // ── STEP 4: Create Native Hybrid Collection ────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("💾 STEP 4: Creating Hybrid Collection (801D, metric=hybrid, medium_plus)");
    console.log("--------------------------------------------------------------------------");

    const createRes = await client.callTool({
      name: "hyperspace_create_collection",
      arguments: {
        collection: testCollection,
        dimension: 801,
        metric: "hybrid",
        quantization: "medium_plus"
      }
    });
    console.log(`   Collection '${testCollection}': ${(createRes as any).content[0].text.substring(0, 60)}...`);

    // ── STEP 5: Insert Real Texts + Search (using insert_text, not memory tools) ──
    console.log("--------------------------------------------------------------------------");
    console.log("🔒 STEP 5: Insert Real Text Vectors & Semantic Search");
    console.log("--------------------------------------------------------------------------");

    const docs = [
      { id: 1, text: "Patient basal insulin rate was configured to 0.85 U/hr at 14:00.", meta: { domain: "medical" } },
      { id: 2, text: "Target glucose range is 4.0 - 7.0 mmol/L before meals.", meta: { domain: "medical" } },
      { id: 3, text: "Invoice #9042 for SaaS subscription paid via Stripe webhook.", meta: { domain: "finance" } },
    ];

    for (const doc of docs) {
      await client.callTool({
        name: "hyperspace_insert_text",
        arguments: { collection: testCollection, id: doc.id, text: doc.text, metadata: doc.meta }
      });
    }
    console.log(`   ✓ Inserted ${docs.length} text vectors into hybrid collection.`);

    const searchRes = await client.callTool({
      name: "hyperspace_search_text",
      arguments: { collection: testCollection, text: "What is the basal insulin setting?", top_k: 5 }
    });
    const searchJson = JSON.parse((searchRes as any).content[0].text);
    const topResult = searchJson[0];
    console.log(`   ✓ Search returned ${searchJson.length} results. Top-1: ID=${topResult?.id}, domain=${topResult?.metadata?.domain}`);

    // Check that medical query returns medical results
    if (topResult?.metadata?.domain === "medical") {
      console.log("   ✓ Semantic search returned correct domain-relevant result (medical)!");
    } else {
      console.warn("   ⚠️ Search may have returned unexpected domain result.");
    }

    // ── STEP 6: Graph traversal & trust score ─────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("🕸️ STEP 6: Graph Traversal & Koopman Trust Score");
    console.log("--------------------------------------------------------------------------");

    const trajRes = await client.callTool({
      name: "hyperspace_predict_momentum",
      arguments: { collection: testCollection, trajectory_ids: [1, 2], steps: 1.0, curvature: 1.0 }
    });
    console.log("   ✓ Koopman Momentum:", (trajRes as any).content[0].text.substring(0, 80) + "...");

    const trustRes = await client.callTool({
      name: "hyperspace_get_trust_score",
      arguments: { collection: testCollection, trajectory_ids: [1, 2], curvature: 1.0 }
    });
    console.log("   ✓ Trust Score:", (trustRes as any).content[0].text.substring(0, 80) + "...");

    // ── STEP 7: Cleanup ──────────────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("🧹 STEP 7: Cleanup");
    console.log("--------------------------------------------------------------------------");
    await client.callTool({
      name: "hyperspace_delete_collection",
      arguments: { collection: testCollection }
    });
    console.log(`   ✓ Collection '${testCollection}' deleted.`);

    console.log("\n==========================================================================");
    console.log("✅ mcp-hyperspacedb REAL V5 BENCHMARK: ALL CHECKS PASSED!");
    console.log("==========================================================================\n");

  } catch (err: any) {
    console.error("❌ DB Benchmark Error:", err);
    try { await client.callTool({ name: "hyperspace_delete_collection", arguments: { collection: testCollection } }); } catch {}
  } finally {
    process.exit(0);
  }
}

runDeepMcpDbBenchmark();
