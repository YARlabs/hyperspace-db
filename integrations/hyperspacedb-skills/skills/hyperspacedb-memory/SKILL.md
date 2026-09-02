---
name: hyperspacedb-memory
description: >
  Zero-overhead cognitive long-term and episodic memory for AI agents (Mem0 drop-in
  and mcp-hyperspace-memory). Use this skill when implementing agent memory, episodic
  fact storage, semantic recall with session isolation, memory update/delete, Fréchet
  mean consolidation, or geodesic hallucination defense.
  Trigger on: "agent memory", "Mem0", "episodic memory", "remember", "recall",
  "forget", "mcp-hyperspace-memory", "hyperspace-memory", "long term memory".
---

# Hyperspace AI Agent Memory (Mem0 Alternative)

Hyperspace Memory is a high-performance cognitive memory layer designed for LLMs, autonomous agents, and spatial multi-agent swarms.

Compared to legacy memory tools (Mem0, Zep):
- **100x lower latency** (< 2ms vs 2,500ms)
- **Zero LLM tokens on write** ($0.000000 per insert vs $0.01 per insert)
- **61.4x RAM compression** via extreme 1-bit ADC quantization
- **Lorentz $\mathbb{H}^{32}$ hierarchy representation** + Geodesic Hallucination Guard

---

## 1. Tier 1: Zero-Code MCP Server (`mcp-hyperspace-memory`)

Connect to Claude Desktop, Cursor, Windsurf, or Antigravity with a single JSON config:

```json
{
  "mcpServers": {
    "hyperspace-memory": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspace-memory"],
      "env": {
        "HYPERSPACE_HOST": "the.yar.ink",
        "HYPERSPACE_API_KEY": "YOUR_YARINK_API_KEY"
      }
    }
  }
}
```

### The 8 Cognitive Tools:
1. `memory_remember`: Store episodic facts with auto-vectorization (`text`, `session_id`, `tags`, `importance`).
2. `memory_recall`: Semantic search with session isolation (`query`, `session_id`, `top_k`).
3. `memory_update`: Modify an existing memory by ID (`memory_id`, `new_text`).
4. `memory_forget`: Delete a memory item by ID (`memory_id`).
5. `memory_list_sessions`: Enumerate all active conversation sessions.
6. `memory_explore_hierarchy`: Traverse concept taxonomy in Lorentz space.
7. `memory_consolidate`: Compute Fréchet mean of episodic memories into an abstract concept.
8. `memory_verify_claim`: Calculate Geodesic Trust Score against stored premises to block hallucinations.

---

## 2. Tier 2: Drop-in Mem0 Replacement SDKs

### Python (`hyperspace-memory`)
```bash
pip install hyperspace-memory
```

```python
from hyperspace_memory import Memory

# 1-Line Drop-in Replacement for Mem0
memory = Memory(config={
    "host": "the.yar.ink",
    "api_key": "YOUR_YARINK_API_KEY",
    "quantization": "extreme"  # 1-bit ADC + Lorentz f32
})

# Add memory (<2ms, 0 tokens)
res = memory.add(
    "Patient blood glucose is 14.2 mmol/L with rapid rising trend.",
    user_id="user_alice",
    agent_id="agent_medical",
    metadata={"category": "glucose"}
)
mem_id = res["results"][0]["id"]

# Semantic search with user isolation
memories = memory.search("glucose trend", user_id="user_alice", limit=5)

# Update & Delete
memory.update(mem_id, "Patient blood glucose normalized to 6.5 mmol/L.")
memory.delete(mem_id)
```

### TypeScript (`hyperspace-memory`)
```bash
npm install hyperspace-memory
```

```typescript
import { Memory } from "hyperspace-memory";

const memory = new Memory({
  host: "the.yar.ink",
  apiKey: "YOUR_YARINK_API_KEY",
  quantization: "extreme"
});

const res = await memory.add(
  "Deployment target: production-us-east-1 on Kubernetes v1.30.",
  { userId: "devops_bob", agentId: "infra_agent" }
);

const results = await memory.search("Kubernetes version?", { userId: "devops_bob" });
console.log(results[0].memory);
```
