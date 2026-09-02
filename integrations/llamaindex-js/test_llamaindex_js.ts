import { HyperspaceClient } from "hyperspace-sdk-ts";
import { HyperspaceVectorStore } from "./src/index.js";
import { VectorStoreQueryMode } from "llamaindex";

const HOST = process.env.HYPERSPACE_HOST || "localhost:50051";
const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const isLocal = HOST.startsWith("localhost") || HOST.startsWith("127.0.0.1");
const grpcKey = (isLocal && API_KEY.startsWith("sk_")) ? "I_LOVE_HYPERSPACEDB" : API_KEY;
const TEST_COLLECTION = `test_llamaindex_js_${Date.now()}`;

async function runLlamaIndexJsComprehensiveTest() {
  console.log("==========================================================================");
  console.log("🦙 LLAMINDEX-JS: COMPREHENSIVE VECTORSTORE TEST SUITE");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Collection: ${TEST_COLLECTION}\n`);

  const client = new HyperspaceClient(HOST, grpcKey);

  let passed = 0;
  let failed = 0;

  function pass(label: string) {
    console.log(`   ✅ ${label}`);
    passed++;
  }

  function fail(label: string, detail?: string) {
    console.error(`   ❌ ${label}${detail ? `: ${detail}` : ""}`);
    failed++;
  }

  try {
    // ── STEP 1: Collection Creation ──────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("📦 STEP 1: Creating Collection with Hybrid Schema");
    console.log("--------------------------------------------------------------------------");

    const schema = {
      components: [
        {
          name: "default",
          metric: "hybrid",
          fullDimension: 801,
          weight: 1.0
        }
      ]
    };
    await client.createCollection(TEST_COLLECTION, schema as any);
    pass(`Collection ${TEST_COLLECTION} created successfully`);

    const vectorStore = new HyperspaceVectorStore({
      client,
      collectionName: TEST_COLLECTION
    });

    // ── STEP 2: Ingest Nodes ─────────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("💾 STEP 2: Adding Multiple Domain Nodes (Medical, Financial, Math)");
    console.log("--------------------------------------------------------------------------");

    const vecMedical = await client.vectorize("Patient exhibits acute hyperglycemia with blood glucose 14.5 mmol/L.", "hybrid");
    const vecFinance = await client.vectorize("Quarterly recurring revenue expanded by 34% driven by enterprise contracts.", "hybrid");
    const vecMath = await client.vectorize("The Poincaré ball and hyperboloid model are isometric representations of hyperbolic space.", "hybrid");

    const nodeMedical: any = {
      id_: "101",
      getContent: () => "Patient exhibits acute hyperglycemia with blood glucose 14.5 mmol/L.",
      embedding: vecMedical,
      metadata: { domain: "medical", severity: "high" }
    };
    const nodeFinance: any = {
      id_: "102",
      getContent: () => "Quarterly recurring revenue expanded by 34% driven by enterprise contracts.",
      embedding: vecFinance,
      metadata: { domain: "finance", type: "revenue" }
    };
    const nodeMath: any = {
      id_: "103",
      getContent: () => "The Poincaré ball and hyperboloid model are isometric representations of hyperbolic space.",
      embedding: vecMath,
      metadata: { domain: "math", geometry: "hyperbolic" }
    };

    const addedIds = await vectorStore.add([nodeMedical, nodeFinance, nodeMath]);
    pass(`Successfully added ${addedIds.length} nodes to VectorStore (IDs: ${addedIds.join(", ")})`);

    // ── STEP 3: Vector Query (Semantic Search) ────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔍 STEP 3: Vector Similarity Query (Standard Vector Search)");
    console.log("--------------------------------------------------------------------------");

    const queryVec = await client.vectorize("What are the characteristics of hyperbolic space geometry?", "hybrid");
    const queryResult = await vectorStore.query({
      queryEmbedding: queryVec,
      similarityTopK: 2,
      mode: VectorStoreQueryMode.DEFAULT
    });

    if (queryResult.nodes && queryResult.nodes.length > 0) {
      pass(`Query returned ${queryResult.nodes.length} nodes. Top hit ID: ${queryResult.nodes[0].id_}`);
      if (queryResult.nodes[0].id_ === "103") {
        pass("Top result is exact expected math/hyperbolic geometry node");
      } else {
        fail(`Expected top hit ID 103, got ${queryResult.nodes[0].id_}`);
      }
    } else {
      fail("No nodes returned for semantic query");
    }

    // ── STEP 4: Delete Node ──────────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🗑️  STEP 4: Delete Node by Reference ID");
    console.log("--------------------------------------------------------------------------");

    await vectorStore.delete("102");
    pass("Deleted node 102 (finance)");

    // ── STEP 5: Verify Deletion ──────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("✅ STEP 5: Verify Node 102 is Removed from Results");
    console.log("--------------------------------------------------------------------------");

    const checkRes = await vectorStore.query({
      queryEmbedding: vecFinance,
      similarityTopK: 5,
      mode: VectorStoreQueryMode.DEFAULT
    });
    const stillPresent = checkRes.nodes?.some((n: any) => n.id_ === "102");
    if (!stillPresent) {
      pass("Confirmed node 102 is no longer in search results");
    } else {
      fail("Deleted node 102 still appeared in search results");
    }

    // ── STEP 6: Clean Up ─────────────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🧹 STEP 6: Clean Up Collection");
    console.log("--------------------------------------------------------------------------");

    await client.deleteCollection(TEST_COLLECTION);
    pass(`Deleted collection: ${TEST_COLLECTION}`);

    // ── Summary ─────────────────────────────────────────────────────────────
    console.log("\n==========================================================================");
    console.log("📊 LLAMINDEX-JS TEST RESULTS");
    console.log("==========================================================================");
    console.log(`   ✅ PASSED: ${passed}`);
    console.log(`   ❌ FAILED: ${failed}`);
    console.log(`   Total:    ${passed + failed}`);
    console.log("==========================================================================");

    if (failed > 0) {
      process.exit(1);
    } else {
      console.log("🎉 ALL LLAMINDEX-JS TESTS PASSED 100%!\n");
      process.exit(0);
    }

  } catch (err: any) {
    console.error("❌ LlamaIndex-JS Test Error:", err);
    try { await client.deleteCollection(TEST_COLLECTION); } catch {}
    process.exit(1);
  }
}

runLlamaIndexJsComprehensiveTest();
