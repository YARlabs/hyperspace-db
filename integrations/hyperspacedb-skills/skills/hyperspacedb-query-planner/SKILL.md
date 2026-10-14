---
name: hyperspacedb-query-planner
description: >
  Query planning, intent disambiguation, and geometric anti-hallucination verification
  for HyperspaceDB. Use this skill whenever an agent needs to: pre-evaluate a raw user query,
  split multi-part questions into parallel vector searches, ask clarifying questions for ambiguous
  intents, or mathematically verify an LLM draft against source context chunks (Tribunal Verification).
  Trigger on: "query planner", "intent clarity", "split questions", "clarify intent",
  "verify generation", "anti-hallucination", "tribunal verification", "trust score".
---

# HyperspaceDB Query Planner & Tribunal Verification

In enterprise RAG and autonomous agent workflows, naive vector searches on raw user queries lead to poor recall, missed sub-questions, and ungrounded hallucinations.

HyperspaceDB solves this with a two-phase architecture:
1. **Pre-Retrieval Intent Planning**: `hyperspace_query_planner` evaluates clarity and splits compound questions into parallel queries before hitting the index.
2. **Post-Generation Tribunal Verification**: `verify_generation` mathematically verifies if the LLM's draft breaks the topological or geometric boundaries of the source context chunks.

---

## 1. Pre-Retrieval: Query Planner & Intent Disambiguator

Before querying HyperspaceDB, run the query planner to evaluate clarity and split compound questions.

### TypeScript / Node.js
```typescript
import { evaluateQueryLocally, clarifyAndSplitIntent } from 'hyperspace-memory-ts';

const plan = evaluateQueryLocally({
  raw_user_query: "Compare HIPAA vs GDPR retention limits and how do we encrypt records?",
  collections: ["compliance_policies", "security_docs"],
  max_sub_queries: 5
});

if (plan.status === "clarification_needed") {
  // Agent prompts the user with multiple-choice clarification
  console.log("Ask user:", plan.clarification?.question);
  console.log("Options:", plan.clarification?.options);
} else {
  // Execute parallel searches
  console.log("Parallel sub-queries:", plan.search_queries);
  // [
  //   "Compare HIPAA vs GDPR retention limits",
  //   "how do we encrypt records"
  // ]
}
```

### Python
```python
from hyperspace_memory import evaluate_query_locally, clarify_and_split_intent

plan = evaluate_query_locally(
    raw_user_query="How is performance?", # Vague pronoun
    collections=["system_benchmarks"]
)

if plan["status"] == "clarification_needed":
    print("Ask user:", plan["clarification"]["question"])
    print("Options:", plan["clarification"]["options"])
```

### MCP Tool Usage (`mcp-hyperspacedb` & `mcp-hyperspace-memory`)
Both MCP servers expose `hyperspace_query_planner`:
```json
{
  "name": "hyperspace_query_planner",
  "arguments": {
    "raw_user_query": "What are our travel budget limits and what is the reimbursement process?",
    "collections": ["hr_policies", "finance_docs"]
  }
}
```

---

## 2. Post-Generation: Tribunal Hallucination Verification

Once the LLM drafts an answer using retrieved context chunk IDs, mathematically verify that the answer respects the topological boundaries of the context chunks.

Geometric calculations are treated as an opaque server-side operation.

### Python SDK
```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051", api_key="sk_...")

result = client.verify_generation(
    llm_answer="The company reimburses up to $75/day for meals without receipts.",
    context_ids=["doc_hr_meals_12", "doc_finance_rules_04"],
    collection="hr_policies"
)

print(f"Trust Score: {result.trust_score:.2f}")
print(f"Is Hallucination: {result.is_hallucination}")
print(f"Details: {result.details}")

if result.is_hallucination:
    # Reject or trigger regenerative critique
    raise ValueError(f"Generation broke geometric context boundaries: {result.details}")
```

### TypeScript SDK
```typescript
import { HyperspaceClient } from 'hyperspace-sdk-ts';

const client = new HyperspaceClient('localhost:50051', 'sk_...');

const verification = await client.verifyGeneration(
  draftAnswer,
  ['context_chunk_1', 'context_chunk_2'],
  'policies_collection'
);

if (verification.is_hallucination) {
  console.warn(`Untrusted generation (${verification.trust_score}): ${verification.details}`);
} else {
  console.log(`Verified valid response (Trust: ${verification.trust_score})`);
}
```

### Rust SDK
```rust
use hyperspace_sdk::Client;

let mut client = Client::connect("http://localhost:50051".to_string(), Some("sk_...".to_string()), None).await?;

let res = client.verify_generation(
    &draft_answer,
    &["chunk_1".to_string(), "chunk_2".to_string()],
    "hr_policies",
).await?;

assert!(!res.is_hallucination);
println!("Trust Score: {}", res.trust_score);
```
