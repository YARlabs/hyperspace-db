import { planQuery, QueryPlannerResult } from "./query_planner.js";

async function runTests() {
  console.log("🧪 Testing HyperspaceDB Query Planner...\n");

  let passed = 0;
  let failed = 0;

  function assert(condition: boolean, msg: string) {
    if (condition) {
      console.log(`✅ PASS: ${msg}`);
      passed++;
    } else {
      console.error(`❌ FAIL: ${msg}`);
      failed++;
    }
  }

  // 1. Ambiguous Single Keyword
  console.log("--- 1. Testing Ambiguous Keyword ('performance') ---");
  const res1 = await planQuery({ raw_user_query: "performance" });
  assert(res1.status === "clarification_needed", "Status must be 'clarification_needed'");
  assert(res1.action === "ask_user_clarification", "Action must be 'ask_user_clarification'");
  assert(!res1.is_intent_clear, "is_intent_clear must be false");
  assert(res1.clarification !== null, "Clarification object must not be null");
  assert(res1.clarification!.options.length >= 3, "Must provide >= 3 multiple choice options");
  assert(res1.clarification!.question.length > 10, "Must provide a clear question");
  console.log("Clarification Question:", res1.clarification?.question);
  console.log("Options:", res1.clarification?.options);
  console.log();

  // 2. Dangling Pronoun without Context
  console.log("--- 2. Testing Dangling Pronoun ('how does it work?') ---");
  const res2 = await planQuery({ raw_user_query: "how does it work?" });
  assert(res2.status === "clarification_needed", "Status must be 'clarification_needed'");
  assert(!res2.is_intent_clear, "is_intent_clear must be false");
  assert(res2.clarification !== null, "Clarification must be present");
  console.log("Clarification Reason:", res2.clarification?.reason);
  console.log();

  // 3. Multi-Intent Conjoined Query with Subject Coreference Resolution
  console.log("--- 3. Testing Conjoined Multi-Question ---");
  const res3 = await planQuery({
    raw_user_query: "What is HyperspaceDB SuperKMeans and how does it prune distances?"
  });
  assert(res3.status === "ready_for_search", "Status must be 'ready_for_search'");
  assert(res3.action === "parallel_search", "Action must be 'parallel_search'");
  assert(res3.is_intent_clear, "is_intent_clear must be true");
  assert(res3.has_multiple_questions, "has_multiple_questions must be true");
  assert(res3.search_queries.length === 2, `Expected 2 sub-queries, got ${res3.search_queries.length}`);
  assert(
    res3.search_queries[1].toLowerCase().includes("superkmeans"),
    "Second sub-query must retain subject coreference 'SuperKMeans'"
  );
  console.log("Sub-queries:", res3.search_queries);
  console.log();

  // 4. Multiple Question Marks
  console.log("--- 4. Testing Multi-Question with Punctuation Separator ---");
  const res4 = await planQuery({
    raw_user_query: "What is vector throughput? How to configure 64-bit IDs in HyperspaceDB?"
  });
  assert(res4.status === "ready_for_search", "Status must be 'ready_for_search'");
  assert(res4.has_multiple_questions, "has_multiple_questions must be true");
  assert(res4.search_queries.length === 2, `Expected 2 sub-queries, got ${res4.search_queries.length}`);
  console.log("Sub-queries:", res4.search_queries);
  console.log();

  // 5. Multi-Dimensional Comparative Query
  console.log("--- 5. Testing Multi-Dimensional Comparative Query ---");
  const res5 = await planQuery({
    raw_user_query: "Compare Scalar-I8 and Binary-1bit in terms of search latency, RAM usage, and recall accuracy"
  });
  assert(res5.status === "ready_for_search", "Status must be 'ready_for_search'");
  assert(res5.has_multiple_questions, "has_multiple_questions must be true");
  assert(res5.search_queries.length === 3, `Expected 3 sub-queries, got ${res5.search_queries.length}`);
  console.log("Decomposed comparative queries:", res5.search_queries);
  console.log();

  // 6. Single Clear Query
  console.log("--- 6. Testing Clear Single-Intent Query ---");
  const res6 = await planQuery({
    raw_user_query: "How to configure Direct I/O in HyperspaceDB storage engine?"
  });
  assert(res6.status === "ready_for_search", "Status must be 'ready_for_search'");
  assert(res6.action === "direct_search", "Action must be 'direct_search'");
  assert(res6.is_intent_clear, "is_intent_clear must be true");
  assert(!res6.has_multiple_questions, "has_multiple_questions must be false");
  assert(res6.search_queries.length === 1, "Must have exactly 1 search query");
  assert(res6.clarification === null, "Clarification must be null");
  console.log("Direct Query:", res6.search_queries[0]);
  console.log();

  console.log(`\n========================================`);
  console.log(`RESULTS: ${passed} passed, ${failed} failed`);
  console.log(`========================================`);

  if (failed > 0) {
    process.exit(1);
  }
}

runTests().catch((err) => {
  console.error("Test execution failed:", err);
  process.exit(1);
});
