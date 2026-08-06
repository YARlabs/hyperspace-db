---
name: hyperspacedb-cognitive
description: >
  Cognitive AI tools for HyperspaceDB: Chain-of-Thought stability analysis, Koopman
  momentum prediction, trust scoring, and Lyapunov convergence for agent reasoning.
  Use this skill when working with AI agent memory, reasoning stability, hallucination
  detection, thought trajectory forecasting, or Koopman operator theory.
  Trigger on: "thought stability", "chain of thought", "CoT", "hallucination detection",
  "reasoning loop", "Lyapunov", "momentum", "trust score", "agent memory", "attractor".
---

# HyperspaceDB Cognitive AI Tools

HyperspaceDB provides **first-class cognitive primitives** for AI agents.
These tools are implemented as **client-side computations** in the SDK:
they fetch stored vectors via `getPoints()` and then apply mathematical analysis locally.
This means they work against any version of the HyperspaceDB server.

Supported geometry for cognitive tools:
- `lorentz`, `poincare` — full Lorentz/hyperbolic math
- `hybrid` — applies Lorentz math to first 33 dims, Euclidean to the rest
- `cosine`, `l2` — Euclidean approximations

---

## Core Concept: Thought Trajectories

A **thought trajectory** is a sequence of vector IDs representing the progression of
an agent's reasoning (e.g., each step of a Chain of Thought stored as a vector).

```
[id_1: "observe problem"] → [id_2: "form hypothesis"] → [id_3: "test hypothesis"] → ...
```

By storing CoT steps in HyperspaceDB, you can then apply mathematical analysis to
detect hallucination, measure convergence, and predict future reasoning direction.

---

## 1. Lyapunov Thought Stability Analysis

Determines whether a reasoning trajectory is **converging** (stable attractor) or
**diverging** (hallucination / reasoning loop).

> **Implementation**: `analyzeThoughtStability` fetches the vectors for the given IDs
> via `getPoints()`, then computes Lyapunov exponent client-side in the SDK.

```typescript
// trajectoryIds: ordered IDs of reasoning steps stored in the collection
const stability = await client.analyzeThoughtStability(
  trajectoryIds,      // number[] — ordered IDs of reasoning steps
  1.0,                // curvature parameter (1.0 for Lorentz/hybrid space)
  "reasoning_memory"
);
// returns: { lyapunov_exponent: number, is_stable: boolean, attractor_id?: number }
```

**Interpretation:**
| `lyapunov_exponent` | Meaning |
|---------------------|---------|
| `< 0` | Stable — converging on a logical conclusion |
| `≈ 0` | Neutral — bounded but not converging |
| `> 0` | Unstable — potential hallucination or infinite loop |

**Use case**: After each N steps of a reasoning chain, check stability. If unstable,
trigger self-correction or inject a grounding prompt.

---

## 2. Koopman Momentum Prediction

**Extrapolates future reasoning direction** using Koopman operator theory.
Given a trajectory, predicts where the agent's thought process will move next.

> **Implementation**: Fetches the last two trajectory vectors via `getPoints()`,
> calls `Metric::extrapolate_momentum()` client-side. For `hybrid` collections,
> Lorentz and Euclidean parts are extrapolated independently then recombined.

```typescript
const forecast = await client.predictMomentum(
  trajectoryIds,      // number[] — past reasoning steps (min 2 IDs needed)
  1.0,                // steps ahead to predict
  "reasoning_memory",
  1.0                 // curvature
);
// returns: number[] — predicted next vector in the collection's geometry
```

**Use case**: Pre-fetch relevant context *before* the agent needs it, based on where
its reasoning is heading — reducing latency in agentic loops.

---

## 3. Trust Score

Calculates a **composite stability and coherence score** (0.0 – 1.0) for a trajectory.
Combines Lyapunov exponent with geometric consistency.

> **Implementation**: Client-side computation on vectors fetched via `getPoints()`.

```typescript
const trust = await client.getTrustScore(
  trajectoryIds,      // number[]
  "reasoning_memory",
  1.0                 // curvature
);
// returns: number (0.0 – 1.0)
```

**Use case**: Gate critical decisions on trust score — only take action when
reasoning confidence exceeds a threshold (e.g., `trust > 0.75`).

---

## 4. Gromov Delta / Geometry Analysis

Analyzes a set of raw vectors to determine the optimal database geometry.
Ports the Gromov 4-point condition test.

```typescript
import { CognitiveMathExport } from 'hyperspace-sdk-ts';

const { delta, recommendation } = CognitiveMathExport.analyzeDeltaHyperbolicity(
  vectorSamples,  // number[][]
  100             // numSamples
);
// recommendation: "lorentz" | "poincare" | "cosine" | "l2"
```

---

## Pattern: Agentic Memory with Cognitive Feedback

```typescript
// 1. Store reasoning steps as vectors
const stepIds: number[] = [];
for (const step of chainOfThought) {
  const id = await client.insertText(step.text, { step: step.index }, "agent_cot");
  stepIds.push(id);
}

// 2. Check stability after every 5 steps
if (stepIds.length % 5 === 0) {
  const { is_stable, lyapunov_exponent } = await client.analyzeThoughtStability(
    stepIds.slice(-10), 1.0, "agent_cot"
  );
  if (!is_stable) {
    // Inject grounding: retrieve most stable past conclusion
    const trustScores = await client.getTrustScore(stepIds.slice(-10), "agent_cot");
    console.warn(`Reasoning diverging (λ=${lyapunov_exponent}). Trust: ${trustScores.score}`);
  }
}

// 3. Predict next topic cluster
const forecast = await client.predictMomentum(stepIds, 1.0, "agent_cot");
// Pre-fetch related documents for the predicted direction
const upcoming = await client.search(forecast.predicted_vector, 5, "knowledge_base");
```

---

## Math Utilities (Client-side)

```typescript
import { CognitiveMathExport as CognitiveMath, HyperbolicMath } from 'hyperspace-sdk-ts';

// Lorentz inner product
const inner = CognitiveMath.lorentzInner(v1, v2);

// Geodesic distance in Lorentz space
const dist = CognitiveMath.lorentzDist(v1, v2);

// Project to Lorentz hyperboloid
const projected = HyperbolicMath.toLorentz(euclideanVec);
```

---

## See Also

- [hyperspacedb-graph](../hyperspacedb-graph/SKILL.md) — graph traversal and hierarchy
- [hyperspacedb-mcp](../hyperspacedb-mcp/SKILL.md) — use cognitive tools via MCP in Claude/Cursor
