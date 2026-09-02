import { Memory } from "./index.js";

const HOST = process.env.HYPERSPACE_HOST || "localhost:50051";
const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const TEST_COLLECTION = `test_mem0_ts_${Date.now()}`;

async function runMemoryComprehensiveTestSuite() {
  console.log("==========================================================================");
  console.log("🧠 HYPERSPACEDB MEMORY SDK (TypeScript): COMPREHENSIVE E2E TEST SUITE");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Test Collection: ${TEST_COLLECTION}\n`);

  const memory = new Memory({
    host: HOST,
    apiKey: API_KEY,
    collectionName: TEST_COLLECTION,
    quantization: "extreme"
  });

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

  // ── STEP 1: Mem0 API Interface Compliance ──────────────────────────────────
  console.log("--------------------------------------------------------------------------");
  console.log("📋 STEP 1: Mem0 Interface Method Signatures Verification");
  console.log("--------------------------------------------------------------------------");
  const methods = ["add", "search", "getAll", "get", "update", "delete", "deleteAll", "reset", "history"];
  for (const m of methods) {
    if (typeof (memory as any)[m] === "function") {
      pass(`Method present: ${m}`);
    } else {
      fail(`Missing required Mem0 method: ${m}`);
    }
  }

  // ── STEP 2: Multi-Domain Memory Ingestion ──────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("💾 STEP 2: Ingesting Multi-Domain Memories (Medical, Finance, Infra)");
  console.log("--------------------------------------------------------------------------");

  // Alice: Medical Domain
  const addAlice1 = await memory.add(
    "Patient blood glucose is 14.2 mmol/L with rapid rising trend at 14:30.",
    { userId: "user_alice", agentId: "agent_medical", runId: "run_shift_01", metadata: { category: "glucose", severity: "high" } }
  );
  const memAlice1 = addAlice1.results[0].id;
  pass(`Added Alice Medical Memory 1: ${memAlice1}`);

  const addAlice2 = await memory.add(
    "Target glucose range before meals is 4.0 - 7.0 mmol/L.",
    { userId: "user_alice", agentId: "agent_medical", runId: "run_shift_01", metadata: { category: "target", severity: "normal" } }
  );
  const memAlice2 = addAlice2.results[0].id;
  pass(`Added Alice Medical Memory 2: ${memAlice2}`);

  // Bob: Finance Domain
  const addBob1 = await memory.add(
    "Invoice #9042 for SaaS subscription $99 paid via Stripe webhook at 15:00.",
    { userId: "user_bob", agentId: "agent_finance", runId: "run_billing_02", metadata: { category: "billing", amount: "99" } }
  );
  const memBob1 = addBob1.results[0].id;
  pass(`Added Bob Finance Memory 1: ${memBob1}`);

  // Charlie: DevOps Domain
  const addCharlie1 = await memory.add(
    "Kubernetes cluster upgraded to v1.30 on primary us-east-1 region.",
    { userId: "user_charlie", agentId: "agent_devops", runId: "run_infra_03", metadata: { category: "infrastructure" } }
  );
  const memCharlie1 = addCharlie1.results[0].id;
  pass(`Added Charlie DevOps Memory 1: ${memCharlie1}`);

  // ── STEP 3: Multi-User Isolation & Leakage Verification ───────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("🔍 STEP 3: Multi-User Isolation (0% Cross-User Leakage)");
  console.log("--------------------------------------------------------------------------");

  const searchAlice = await memory.search("glucose target insulin", { userId: "user_alice", limit: 5 });
  if (searchAlice.length > 0 && searchAlice.every(m => m.userId === "user_alice")) {
    pass(`Alice search returned ${searchAlice.length} memories, all scoped to user_alice`);
  } else {
    fail("Alice search failed or returned cross-user memories");
  }

  const hasLeakBob = searchAlice.some(m => m.userId === "user_bob" || m.memory.includes("Invoice #9042"));
  if (!hasLeakBob) {
    pass("Zero cross-user leakage: Bob finance memory absent from Alice search");
  } else {
    fail("CRITICAL: Bob finance memory leaked into Alice search results!");
  }

  const searchBob = await memory.search("Stripe invoice payment", { userId: "user_bob", limit: 5 });
  if (searchBob.length > 0 && searchBob.every(m => m.userId === "user_bob")) {
    pass(`Bob search returned ${searchBob.length} memories, all scoped to user_bob`);
  } else {
    fail("Bob search failed or returned cross-user memories");
  }

  const searchGhost = await memory.search("glucose invoice", { userId: "user_ghost", limit: 5 });
  if (searchGhost.length === 0) {
    pass("Non-existent user search correctly returned 0 memories");
  } else {
    fail("Non-existent user search returned unexpected results");
  }

  // ── STEP 4: Semantic Ranking Accuracy ─────────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("📊 STEP 4: Semantic Ranking & Relevance Accuracy (v5_Light Embeddings)");
  console.log("--------------------------------------------------------------------------");

  const rankResults = await memory.search("What is the targeted blood glucose level before meals?", { userId: "user_alice", limit: 5 });
  if (rankResults.length > 0) {
    const topHit = rankResults[0];
    if (topHit.memory.includes("Target glucose range")) {
      pass(`Semantic ranking #1 hit is exact target memory (Score: ${topHit.score})`);
    } else {
      fail(`Unexpected top hit: ${topHit.memory}`);
    }
  } else {
    fail("Semantic ranking search returned no results");
  }

  // ── STEP 5: Agent-ID & Run-ID Scoping ─────────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("🎯 STEP 5: Agent-ID and Run-ID Scoping Verification");
  console.log("--------------------------------------------------------------------------");

  const agentResults = await memory.search("blood glucose", { agentId: "agent_medical", limit: 5 });
  if (agentResults.length > 0 && agentResults.every(m => m.agentId === "agent_medical")) {
    pass(`Agent scoping verified: all ${agentResults.length} hits match agent_medical`);
  } else {
    fail("Agent scoping failed");
  }

  const runResults = await memory.search("glucose", { runId: "run_shift_01", limit: 5 });
  if (runResults.length > 0 && runResults.every(m => m.runId === "run_shift_01")) {
    pass(`Run scoping verified: all ${runResults.length} hits match run_shift_01`);
  } else {
    fail("Run scoping failed");
  }

  // ── STEP 6: Update Memory & History Lineage ───────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("✏️  STEP 6: Memory Update & Historical Revisions (update, history)");
  console.log("--------------------------------------------------------------------------");

  const updateRes = await memory.update(memAlice1, "Patient blood glucose normalized to 6.2 mmol/L after 4 units Humalog.");
  if (updateRes.message.includes("updated successfully")) {
    pass(`Updated Alice Memory ${memAlice1}: ${updateRes.message}`);
  } else {
    fail("Memory update failed", JSON.stringify(updateRes));
  }

  const hist = await memory.history(memAlice1);
  if (Array.isArray(hist)) {
    pass(`Retrieved history for ${memAlice1}: ${hist.length} record(s)`);
  } else {
    fail("History retrieval returned non-array");
  }

  // ── STEP 7: Granular Retrieval (get, getAll) ──────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("📜 STEP 7: Granular Retrieval (get, getAll)");
  console.log("--------------------------------------------------------------------------");

  const aliceAll = await memory.getAll({ userId: "user_alice" });
  pass(`getAll for user_alice returned ${aliceAll.length} items`);

  const bobItem = await memory.get(memBob1);
  if (bobItem && bobItem.userId === "user_bob") {
    pass(`get(${memBob1}) retrieved Bob's memory: "${bobItem.memory.substring(0, 40)}..."`);
  } else {
    fail(`Failed to retrieve memory by ID ${memBob1}`);
  }

  // ── STEP 8: Granular Delete & Isolation ───────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("🗑️  STEP 8: Granular Delete & Unaffected Isolation (delete)");
  console.log("--------------------------------------------------------------------------");

  const delRes = await memory.delete(memBob1);
  pass(`Deleted Bob memory ${memBob1}: ${delRes.message}`);

  const aliceAfterDel = await memory.search("glucose", { userId: "user_alice", limit: 5 });
  if (aliceAfterDel.length > 0) {
    pass(`Alice memories intact after Bob deletion (${aliceAfterDel.length} memories retained)`);
  } else {
    fail("Alice memories were unexpectedly lost after Bob deletion");
  }

  // ── STEP 9: Collection Reset & Tear Down ──────────────────────────────────
  console.log("\n--------------------------------------------------------------------------");
  console.log("🧹 STEP 9: Reset Memory Collection & Tear Down (reset)");
  console.log("--------------------------------------------------------------------------");

  const resetRes = await memory.reset();
  pass(`Reset collection: ${resetRes.message}`);

  // ── Final Summary ─────────────────────────────────────────────────────────
  console.log("\n==========================================================================");
  console.log("📊 TYPESCRIPT MEMORY SDK TEST RESULTS");
  console.log("==========================================================================");
  console.log(`   ✅ PASSED: ${passed}`);
  console.log(`   ❌ FAILED: ${failed}`);
  console.log(`   Total:    ${passed + failed}`);
  console.log("==========================================================================");

  if (failed > 0) {
    throw new Error(`${failed} test(s) failed in TypeScript Memory SDK suite`);
  }
  console.log("🎉 ALL TYPESCRIPT COMPREHENSIVE MEMORY TESTS PASSED!\n");
}

runMemoryComprehensiveTestSuite().catch(err => {
  console.error("❌ Test failed:", err);
  process.exit(1);
});
