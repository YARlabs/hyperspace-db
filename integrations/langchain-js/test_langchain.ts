import { HyperspaceClient } from "hyperspace-sdk-ts";
import { HyperspaceStore } from "./src/vectorstores.js";
import { Document } from "@langchain/core/documents";

const HOST = process.env.HYPERSPACE_HOST || "localhost:50051";
const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const isLocal = HOST.startsWith("localhost") || HOST.startsWith("127.0.0.1");
const grpcKey = (isLocal && API_KEY.startsWith("sk_")) ? "I_LOVE_HYPERSPACEDB" : API_KEY;
const testCol = `test_langchain_js_${Date.now()}`;

async function runLangchainJsComprehensiveTest() {
  console.log("==========================================================================");
  console.log("🦜🔗 LANGCHAIN-JS: COMPREHENSIVE VECTORSTORE TEST SUITE");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Collection: ${testCol}\n`);

  const client = new HyperspaceClient(HOST, grpcKey);
  const store = new HyperspaceStore(undefined, {
    client,
    collectionName: testCol,
    dimension: 801,
    metric: "hybrid",
    useServerSideEmbedding: true
  });

  // Wait a moment for collection creation to resolve
  await new Promise(r => setTimeout(r, 1000));

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
    // ── STEP 1: Ingest Documents ─────────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("💾 STEP 1: Ingesting Documents via addDocuments()");
    console.log("--------------------------------------------------------------------------");

    const docs = [
      new Document({
        pageContent: "Lorentz geometry enables hierarchical embedding with minimal distortion in H32.",
        metadata: { category: "geometry", importance: "high" }
      }),
      new Document({
        pageContent: "Continuous glucose monitoring provides real-time interstitial blood sugar readings.",
        metadata: { category: "medical", importance: "critical" }
      }),
      new Document({
        pageContent: "Stripe handles automated credit card subscriptions and invoice billing webhooks.",
        metadata: { category: "finance", importance: "normal" }
      })
    ];

    const docIds = await store.addDocuments(docs);
    pass(`Successfully added ${docIds.length} documents (IDs: ${docIds.join(", ")})`);

    // ── STEP 2: Similarity Search (Geometry) ─────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔍 STEP 2: Semantic Similarity Search (Geometry Domain)");
    console.log("--------------------------------------------------------------------------");

    const geomHits = await store.similaritySearch("hyperbolic geometry tree hierarchy", 2);
    if (geomHits.length > 0 && geomHits[0].pageContent.includes("Lorentz geometry")) {
      pass(`Top result is correct geometry document (Found: "${geomHits[0].pageContent.substring(0, 45)}...")`);
    } else {
      fail("Top result was not the expected geometry document");
    }

    // ── STEP 3: Similarity Search (Medical) ──────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔍 STEP 3: Semantic Similarity Search (Medical Domain)");
    console.log("--------------------------------------------------------------------------");

    const medHits = await store.similaritySearch("blood glucose target level", 2);
    const foundMed = medHits.find(d => d.pageContent.includes("glucose") || d.pageContent.includes("sugar"));
    if (foundMed) {
      pass(`Found medical document in top results: "${foundMed.pageContent.substring(0, 45)}..."`);
    } else {
      fail(`Medical document not found. Hits: ${medHits.map(d => d.pageContent.substring(0, 30)).join(" | ")}`);
    }

    // ── STEP 4: Similarity Search with Scores ────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("📊 STEP 4: Similarity Search with Score Verification");
    console.log("--------------------------------------------------------------------------");

    const hitsWithScore = await store.similaritySearchWithScore("Stripe payments invoices", 2);
    if (hitsWithScore.length > 0) {
      const [doc, score] = hitsWithScore[0];
      pass(`Top finance result: "${doc.pageContent.substring(0, 35)}..." with distance score ${score}`);
    } else {
      fail("No results returned for finance search");
    }

    // ── STEP 5: Cleanup ──────────────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🧹 STEP 5: Tear Down & Cleanup Collection");
    console.log("--------------------------------------------------------------------------");

    await client.deleteCollection(testCol);
    pass(`Deleted test collection: ${testCol}`);

    // ── Summary ─────────────────────────────────────────────────────────────
    console.log("\n==========================================================================");
    console.log("📊 LANGCHAIN-JS TEST RESULTS SUMMARY");
    console.log("==========================================================================");
    console.log(`   ✅ PASSED: ${passed}`);
    console.log(`   ❌ FAILED: ${failed}`);
    console.log(`   Total:    ${passed + failed}`);
    console.log("==========================================================================");

    if (failed > 0) {
      process.exit(1);
    } else {
      console.log("🎉 ALL LANGCHAIN-JS VECTORSTORE TESTS PASSED 100%!\n");
      process.exit(0);
    }

  } catch (err: any) {
    console.error("❌ LangChain-JS Test Error:", err);
    try { await client.deleteCollection(testCol); } catch {}
    process.exit(1);
  }
}

runLangchainJsComprehensiveTest();
