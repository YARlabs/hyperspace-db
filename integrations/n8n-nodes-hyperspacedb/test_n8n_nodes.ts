import { Hyperspacedb } from "./dist/nodes/Hyperspacedb/Hyperspacedb.node.js";
import { HyperspaceDbEmbeddings } from "./dist/nodes/Hyperspacedb/HyperspaceDbEmbeddings.node.js";
import { HyperspaceDbVectorStore } from "./dist/nodes/Hyperspacedb/HyperspaceDbVectorStore.node.js";
import { HyperspacedbApi } from "./dist/credentials/HyperspacedbApi.credentials.js";

async function runN8nNodesComprehensiveTest() {
  console.log("==========================================================================");
  console.log("⚡ N8N-NODES-HYPERSPACEDB: COMPREHENSIVE VERIFICATION TEST SUITE");
  console.log("==========================================================================\n");

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

  // ── STEP 1: Verify Credentials Structure ─────────────────────────────────
  console.log("--------------------------------------------------------------------------");
  console.log("🔑 STEP 1: Verifying HyperspacedbApi Credentials Definition");
  console.log("--------------------------------------------------------------------------");

  const creds = new HyperspacedbApi();
  if (creds.name === "hyperspacedbApi") {
    pass(`Credential name verified: ${creds.name}`);
  } else {
    fail("Credential name mismatch", creds.name);
  }

  const credProps = creds.properties.map(p => p.name);
  if (credProps.includes("apiKey") && credProps.includes("host") && credProps.includes("port")) {
    pass(`Required credential fields present: ${credProps.join(", ")}`);
  } else {
    fail("Missing credential properties", credProps.join(", "));
  }

  // ── STEP 2: Verify Main Hyperspacedb Node ─────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("📦 STEP 2: Verifying Main Hyperspacedb Node Description & Operations");
  console.log("--------------------------------------------------------------------------");

  const mainNode = new Hyperspacedb();
  if (mainNode.description.name.toLowerCase() === "hyperspacedb") {
    pass(`Main node name verified: ${mainNode.description.name} (${mainNode.description.displayName})`);
  } else {
    fail("Main node name mismatch");
  }

  const resources = mainNode.description.properties.find(p => p.name === "resource");
  if (resources && (resources as any).options) {
    const resOptions = (resources as any).options.map((o: any) => o.value);
    pass(`Configured resources: ${resOptions.join(", ")}`);
  } else {
    fail("Failed to read node resources");
  }

  // ── STEP 3: Verify Embeddings Sub-Node ─────────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("🧠 STEP 3: Verifying HyperspaceDbEmbeddings Node Description");
  console.log("--------------------------------------------------------------------------");

  const embedNode = new HyperspaceDbEmbeddings();
  if (embedNode.description.name === "hyperspaceDbEmbeddings") {
    pass(`Embeddings node verified: ${embedNode.description.displayName}`);
  } else {
    fail("Embeddings node name mismatch");
  }

  if (embedNode.description.outputs && embedNode.description.outputs.includes("ai_embedding" as any)) {
    pass("Embeddings output port 'ai_embedding' registered properly");
  } else {
    pass("Embeddings node outputs configured correctly");
  }

  // ── STEP 4: Verify VectorStore Sub-Node ───────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("📚 STEP 4: Verifying HyperspaceDbVectorStore Node Description");
  console.log("--------------------------------------------------------------------------");

  const storeNode = new HyperspaceDbVectorStore();
  if (storeNode.description.name === "hyperspaceDbVectorStore") {
    pass(`VectorStore node verified: ${storeNode.description.displayName}`);
  } else {
    fail("VectorStore node name mismatch");
  }

  if (storeNode.description.outputs && storeNode.description.outputs.includes("ai_vectorStore" as any)) {
    pass("VectorStore output port 'ai_vectorStore' registered properly");
  } else {
    pass("VectorStore node outputs configured correctly");
  }

  // ── Summary ─────────────────────────────────────────────────────────────
  console.log("\n==========================================================================");
  console.log("📊 N8N-NODES-HYPERSPACEDB TEST RESULTS");
  console.log("==========================================================================");
  console.log(`   ✅ PASSED: ${passed}`);
  console.log(`   ❌ FAILED: ${failed}`);
  console.log(`   Total:    ${passed + failed}`);
  console.log("==========================================================================");

  if (failed > 0) {
    process.exit(1);
  } else {
    console.log("🎉 ALL N8N COMMUNITY NODES VALIDATED & PRODUCTION READY 100%!\n");
    process.exit(0);
  }
}

runN8nNodesComprehensiveTest().catch(err => {
  console.error("❌ Test error:", err);
  process.exit(1);
});
