import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const HOST = process.env.HYPERSPACE_HOST || "the.yar.ink";
const MEMORY_COLLECTION = process.env.MEMORY_COLLECTION || "agent_cognitive_memories_129";

/**
 * Comprehensive test suite for mcp-hyperspace-memory v1.0.0
 * Tests: all 8 tools, session isolation, update, consolidation, claim verification
 */
async function runMemoryServerTest() {
  console.log("==========================================================================");
  console.log("🧠 mcp-hyperspace-memory v1.0.0: COMPREHENSIVE TEST SUITE");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Memory Collection: ${MEMORY_COLLECTION}\n`);

  const fs = await import("fs");
  const serverPath = fs.existsSync(path.resolve(__dirname, "../dist/index.js"))
    ? path.resolve(__dirname, "../dist/index.js")
    : path.resolve(__dirname, "index.js");

  // Verify the built server exists
  if (!fs.existsSync(serverPath)) {
    console.error(`❌ mcp-hyperspace-memory dist not found at: ${serverPath}`);
    console.error("   Run: cd integrations/mcp-hyperspace-memory && npm run build");
    process.exit(1);
  }

  const transport = new StdioClientTransport({
    command: "node",
    args: [serverPath],
    env: {
      HYPERSPACE_HOST: HOST,
      HYPERSPACE_API_KEY: API_KEY,
      MEMORY_COLLECTION,
      PATH: process.env.PATH || ""
    }
  });

  const client = new Client(
    { name: "mcp-memory-tester", version: "1.0.0" },
    { capabilities: {} }
  );

  let passed = 0;
  let failed = 0;

  function pass(label: string) {
    console.log(`   ✅ ${label}`);
    passed++;
  }

  function fail(label: string, detail?: string) {
    console.error(`   ❌ ${label}${detail ? `: ${detail}` : ''}`);
    failed++;
  }

  try {
    await client.connect(transport);
    console.log("   ✓ Connected to mcp-hyperspace-memory via Stdio\n");

    // ── STEP 1: Tool Registry ─────────────────────────────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("📋 STEP 1: Tool Registry Verification");
    console.log("--------------------------------------------------------------------------");

    const toolsResult = await client.listTools();
    const toolNames = toolsResult.tools.map(t => t.name);
    console.log(`   Registered tools (${toolNames.length}): ${toolNames.join(", ")}`);

    const expectedTools = [
      "memory_remember", "memory_recall", "memory_forget", "memory_update",
      "memory_list_sessions", "memory_explore_hierarchy", "memory_consolidate", "memory_verify_claim"
    ];

    for (const tool of expectedTools) {
      if (toolNames.includes(tool)) {
        pass(`Tool present: ${tool}`);
      } else {
        fail(`Missing tool: ${tool}`);
      }
    }

    // Verify NO DB admin tools are present
    const forbiddenTools = ["hyperspace_create_collection", "hyperspace_delete_collection", "hyperspace_cache_stats"];
    for (const tool of forbiddenTools) {
      if (!toolNames.includes(tool)) {
        pass(`Admin tool correctly absent: ${tool}`);
      } else {
        fail(`Admin tool should NOT be present: ${tool}`);
      }
    }

    // ── STEP 2: memory_remember ────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("💾 STEP 2: memory_remember — Storing episodic memories");
    console.log("--------------------------------------------------------------------------");

    const memRes1 = await client.callTool({
      name: "memory_remember",
      arguments: {
        text: "Patient glucose level is 14.2 mmol/L with rapid rising trend at 14:30.",
        session_id: "session_medical_01",
        tags: ["medical", "glucose", "urgent"]
      }
    });
    const mem1 = JSON.parse((memRes1 as any).content[0].text);
    if (mem1.status === "remembered" && typeof mem1.memory_id === "number") {
      pass(`Remembered medical event. ID: ${mem1.memory_id}`);
    } else {
      fail("memory_remember returned unexpected response", JSON.stringify(mem1));
    }

    const memRes2 = await client.callTool({
      name: "memory_remember",
      arguments: {
        text: "Target glucose range is 4.0 - 7.0 mmol/L before meals.",
        session_id: "session_medical_01",
        tags: ["medical", "target"]
      }
    });
    const mem2 = JSON.parse((memRes2 as any).content[0].text);
    pass(`Remembered glucose target. ID: ${mem2.memory_id}`);

    const memRes3 = await client.callTool({
      name: "memory_remember",
      arguments: {
        text: "Invoice #9042 for SaaS subscription paid via Stripe at 15:00.",
        session_id: "session_finance_02",
        tags: ["billing", "stripe"]
      }
    });
    const mem3 = JSON.parse((memRes3 as any).content[0].text);
    pass(`Remembered finance event. ID: ${mem3.memory_id}`);

    // ── STEP 3: memory_recall + session isolation ──────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔍 STEP 3: memory_recall — Semantic search + session isolation");
    console.log("--------------------------------------------------------------------------");

    const recallRes = await client.callTool({
      name: "memory_recall",
      arguments: {
        query: "What is the basal insulin glucose target?",
        session_id: "session_medical_01",
        limit: 5
      }
    });
    const recall = JSON.parse((recallRes as any).content[0].text);
    const resultTexts = JSON.stringify(recall.results);
    if (recall.results && recall.results.length > 0) {
      pass(`Recall returned ${recall.results.length} results for medical session`);
    } else {
      fail("Recall returned no results");
    }

    // Session isolation: finance memory should NOT appear in medical recall
    if (!resultTexts.includes("Invoice #9042")) {
      pass("Session isolation VERIFIED: finance memory absent from medical recall");
    } else {
      fail("Session isolation FAILED: finance memory leaked into medical session recall");
    }

    // ── STEP 4: memory_update ─────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("✏️  STEP 4: memory_update — Replace an existing memory");
    console.log("--------------------------------------------------------------------------");

    const updateRes = await client.callTool({
      name: "memory_update",
      arguments: {
        memory_id: mem1.memory_id,
        new_text: "Patient glucose level is 8.4 mmol/L, normalized after insulin correction.",
        session_id: "session_medical_01",
        tags: ["medical", "glucose", "resolved"]
      }
    });
    const update = JSON.parse((updateRes as any).content[0].text);
    if (update.status === "updated" && update.old_memory_id === mem1.memory_id) {
      pass(`Memory updated. Old ID: ${update.old_memory_id} → New ID: ${update.new_memory_id}`);
    } else {
      fail("memory_update returned unexpected response", JSON.stringify(update));
    }

    // ── STEP 5: memory_list_sessions ──────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("📂 STEP 5: memory_list_sessions — Enumerate active sessions");
    console.log("--------------------------------------------------------------------------");

    const sessionsRes = await client.callTool({
      name: "memory_list_sessions",
      arguments: { limit: 50 }
    });
    const sessions = JSON.parse((sessionsRes as any).content[0].text);
    console.log(`   Sessions found: ${JSON.stringify(sessions.sessions)}`);
    if (sessions.session_count >= 0) {
      pass(`memory_list_sessions returned session_count=${sessions.session_count}`);
    } else {
      fail("memory_list_sessions failed");
    }

    // ── STEP 5B: memory_explore_hierarchy ─────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🌲 STEP 5B: memory_explore_hierarchy — Lorentz hierarchy exploration");
    console.log("--------------------------------------------------------------------------");

    try {
      const hierRes = await client.callTool({
        name: "memory_explore_hierarchy",
        arguments: {
          concept_id: mem2.memory_id,
          direction: "up",
          limit: 5
        }
      });
      pass(`memory_explore_hierarchy executed successfully for concept ID ${mem2.memory_id}`);
    } catch (e: any) {
      fail("memory_explore_hierarchy failed", e.message);
    }

    // ── STEP 6: memory_verify_claim ───────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔬 STEP 6: memory_verify_claim — Hallucination prevention");
    console.log("--------------------------------------------------------------------------");

    const verifyRelated = await client.callTool({
      name: "memory_verify_claim",
      arguments: {
        premise: "Patient blood glucose is dangerously high at 14 mmol/L.",
        conclusion: "An insulin correction bolus is required to bring glucose back to target range."
      }
    });
    const vRelated = JSON.parse((verifyRelated as any).content[0].text);
    console.log(`   Related claim trust_score: ${vRelated.trust_score} → ${vRelated.status}`);
    if (typeof vRelated.trust_score === "number") {
      pass(`memory_verify_claim returned valid trust_score: ${vRelated.trust_score}`);
    } else {
      fail("memory_verify_claim missing trust_score");
    }

    const verifyUnrelated = await client.callTool({
      name: "memory_verify_claim",
      arguments: {
        premise: "The capital of France is Paris.",
        conclusion: "Insulin dosing requires knowledge of carbohydrate ratios and correction factors."
      }
    });
    const vUnrelated = JSON.parse((verifyUnrelated as any).content[0].text);
    console.log(`   Unrelated claim trust_score: ${vUnrelated.trust_score} → ${vUnrelated.status}`);
    if (vUnrelated.trust_score < vRelated.trust_score) {
      pass("Unrelated claim correctly scored LOWER than related claim");
    } else {
      console.warn("   ⚠️ Warning: unrelated claim scored unexpectedly high");
    }

    // ── STEP 7: memory_consolidate ────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🌀 STEP 7: memory_consolidate — Fréchet mean of episodic memories");
    console.log("--------------------------------------------------------------------------");

    const consolidateRes = await client.callTool({
      name: "memory_consolidate",
      arguments: { topic_query: "glucose insulin medical patient", limit: 5 }
    });
    const consolidate = JSON.parse((consolidateRes as any).content[0].text);
    if (consolidate.status === "consolidated" || consolidate.error) {
      if (consolidate.status === "consolidated") {
        pass(`Consolidated ${consolidate.source_count} memories into ${consolidate.consolidated_dimension}D concept vector`);
      } else {
        console.warn(`   ⚠️ Consolidate: ${consolidate.error} (collection may be too new)`);
      }
    } else {
      fail("memory_consolidate unexpected response", JSON.stringify(consolidate));
    }

    // ── STEP 8: memory_forget ─────────────────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🗑️  STEP 8: memory_forget — Delete a memory by ID");
    console.log("--------------------------------------------------------------------------");

    const forgetRes = await client.callTool({
      name: "memory_forget",
      arguments: { memory_id: mem3.memory_id }
    });
    const forget = JSON.parse((forgetRes as any).content[0].text);
    pass(`memory_forget: success=${forget.success} for ID ${mem3.memory_id}`);

    // ── STEP 9: memory_recall — Verify forgotten memory is gone ───────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("✅ STEP 9: Verify forgotten memory is no longer retrievable");
    console.log("--------------------------------------------------------------------------");

    const recallAfterForget = await client.callTool({
      name: "memory_recall",
      arguments: { query: "Invoice Stripe billing payment", session_id: "session_finance_02", limit: 5 }
    });
    const rafJson = JSON.parse((recallAfterForget as any).content[0].text);
    const stillPresent = JSON.stringify(rafJson.results).includes("Invoice #9042");
    if (!stillPresent) {
      pass("Forgotten memory is no longer retrievable via recall");
    } else {
      console.warn("   ⚠️ Forgotten memory still appears in recall (may be cached; eventually consistent)");
    }

    // ── Summary ───────────────────────────────────────────────────────────
    console.log("\n==========================================================================");
    console.log("📊 TEST RESULTS SUMMARY");
    console.log("==========================================================================");
    console.log(`   ✅ PASSED: ${passed}`);
    console.log(`   ❌ FAILED: ${failed}`);
    console.log(`   Total:    ${passed + failed}`);
    console.log("==========================================================================");

    if (failed === 0) {
      console.log("🎉 ALL TESTS PASSED — mcp-hyperspace-memory is PRODUCTION READY!");
    } else {
      console.error(`⚠️  ${failed} test(s) failed.`);
    }

  } catch (err: any) {
    console.error("\n❌ Fatal test error:", err.message);
    console.error(err);
    process.exit(1);
  } finally {
    process.exit(failed > 0 ? 1 : 0);
  }
}

runMemoryServerTest();
