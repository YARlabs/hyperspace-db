# hyperspace-memory (TypeScript) 🚀

> **Drop-in replacement for Mem0, Zep, and MemGPT** in TypeScript/JavaScript powered by the **HyperspaceDB** native memory engine with 1-bit ADC quantization, MRL in-RAM cascades, and Lorentz hyperbolic spaces.

[![NPM Version](https://img.shields.io/npm/v/hyperspace-memory.svg)](https://www.npmjs.com/package/hyperspace-memory)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ⚡ Key Advantages over Mem0 & Zep

- **100x Lower Search Latency**: Sub-millisecond (< 0.5 ms) retrieval via MRL in-RAM cascade indexing (vs 1200+ ms in LLM-based memory agents).
- **Zero-Knowledge Client-Side Encryption**: Vectors and chunk payloads are encrypted client-side using deterministic isometric projections + AES-256-GCM. Vec2Text inversion cosine collapses to $0.038$.
- **Autonomous Cognitive Decay & Superseding**: Native Ebbinghaus forgetting curve ($S(t) = S_0 \cdot e^{-\lambda \Delta t}$) and directed `SUPERSEDES` graph relations eliminate outdated facts without requiring slow, expensive LLM reflection calls.
- **98% RAM & Storage Reduction**: Native `extreme` 1-bit ADC and `turbo` 4-bit Lloyd-Max quantization.
- **100% Mem0 API Compatibility**: Replace `import { Memory } from "mem0ai"` with `import { Memory } from "hyperspace-memory"` without altering application logic.

---

## 📦 Installation

```bash
npm install hyperspace-memory
```

---

## 🛠 Quick Start (Mem0 Drop-In Migration)

```typescript
import { Memory } from "hyperspace-memory";

// 1. Initialize Memory with optional Zero-Knowledge Encryption & Ebbinghaus Decay
const memory = new Memory({
  host: "the.yar.ink",
  apiKey: "YOUR_YARINK_API_KEY",
  quantization: "medium_plus", // "none" | "medium_plus" | "turbo" | "extreme"
  encryptionKey: "user_confidential_master_key", // Zero-Knowledge client-side encryption
  decayLambda: 0.01,            // Ebbinghaus exponential forgetting rate
  autoSupersede: true           // Automatically links older contradicting concepts via SUPERSEDES edge
});


// 2. Add Memory
const res = await memory.add(
  "User prefers TypeScript and React over Angular.",
  { userId: "user_123", metadata: { category: "preferences" } }
);
console.log("Added Memory:", res);

// 3. Search Memory
const results = await memory.search("What UI framework does the user prefer?", {
  userId: "user_123",
  limit: 3
});
for (const item of results) {
  console.log(`Memory: ${item.memory} (Score: ${item.score})`);
}

// 4. Cognitive Consolidation (Fréchet Mean on Hyperboloid)
const consolidation = await memory.consolidate("UI frameworks", {
  summaryText: "User consistently prefers modern React/TypeScript stack across all projects."
});
console.log("Consolidated concept:", consolidation.summary);

// 5. Anti-Hallucination Claim Verification (Lorentz Geodesic Trust)
const verification = await memory.verifyClaim(
  "User prefers TypeScript and React over Angular.",
  "User would be enthusiastic about migrating the codebase to Angular."
);
console.log("Trust status:", verification.status, "Score:", verification.trustScore);

// 6. Delete & Reset
await memory.delete(results[0].id);
```

---

## 🧠 Cognitive Operations

- `memory.consolidate(topicQuery, options)`: Synthesizes a cluster of related episodic memories into a single abstract semantic concept using the **Fréchet Mean** in Lorentz hyperbolic space.
- `memory.verifyClaim(premise, conclusion, threshold?)`: Computes **Geodesic Trust Score** (Lorentz $\mathbb{H}^{32}$ + Cosine 96D) to verify that an LLM conclusion does not violate established facts or hallucinate.
- `memory.link(sourceId, targetId, relation?, weight?)`: Explicitly connects memory nodes with directed edges in the CSR graph.
- `memory.linkBatch(edges)`: Bulk-inserts directed topology edges.
- `memory.unlink(sourceId, targetId, relation?)`: Zero-lag deletion of explicit edges.
- `memory.induceEdges(rules)`: Declaratively constructs knowledge graph edges from existing metadata fields.
- `memory.listSessions(limit?)`: Discovers and enumerates active conversational sessions.
- `memory.exploreHierarchy(conceptId, direction?)`: Navigates conceptual subsumption and taxomomy trees.
- `memory.stats()`: Returns vector counts, cascade configuration, and collection status.

---

## 📄 License
MIT © YARlabs.

