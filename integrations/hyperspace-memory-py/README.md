# hyperspace-memory (Python) 🚀

> **Drop-in replacement for Mem0, Zep, and MemGPT** powered by the **HyperspaceDB** native memory engine with 1-bit ADC quantization, MRL in-RAM cascades, and Lorentz hyperbolic spaces.

[![PyPI Version](https://img.shields.io/pypi/v/hyperspace-memory.svg)](https://pypi.org/project/hyperspace-memory/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ⚡ Key Advantages over Mem0 & Zep

- **100x Lower Search Latency**: Sub-millisecond (< 0.5 ms) retrieval via MRL in-RAM cascade indexing.
- **98% RAM & Storage Reduction**: Native `extreme` 1-bit ADC and `turbo` 4-bit Lloyd-Max quantization (compressed down to ~100B per vector).
- **Zero Extra LLM Calls**: Direct vector and graph traversal memory operations without forcing expensive LLM parsing for every insert.
- **100% Mem0 API Compatibility**: Replace `from mem0 import Memory` with `from hyperspace_memory import Memory` without altering your application code.

---

## 📦 Installation

```bash
pip install hyperspace-memory
```

---

## 🛠 Quick Start (Mem0 Drop-In Migration)

```python
from hyperspace_memory import Memory

# 1. Initialize Memory (connects to HyperspaceDB)
memory = Memory(config={
    "host": "the.yar.ink",
    "api_key": "YOUR_YARINK_API_KEY",
    "quantization": "medium_plus"  # "none" | "medium_plus" | "turbo" | "extreme"
})

# 2. Add Memory
res = memory.add(
    "User prefers Rust and dark mode UI over Python and light mode.",
    user_id="user_123",
    metadata={"category": "preferences"}
)
print("Added Memory:", res)

# 3. Search Memory
results = memory.search("What programming language does the user prefer?", user_id="user_123", limit=3)
for item in results:
    print(f"Memory: {item['memory']} (Score: {item['score']})")

# 4. Get All Memories for User
all_memories = memory.get_all(user_id="user_123")

# 5. Cognitive Consolidation (Fréchet Mean on Hyperboloid)
consolidation = memory.consolidate(
    "programming languages",
    summary_text="User strongly favors Rust and systems programming."
)
print("Consolidated concept:", consolidation["summary"])

# 6. Anti-Hallucination Claim Verification (Lorentz Geodesic Trust)
verification = memory.verify_claim(
    "User prefers Rust and dark mode UI over Python and light mode.",
    "User always requests Python notebooks with light themes."
)
print("Trust status:", verification["status"], "Score:", verification["trust_score"])

# 7. Delete & Reset
memory.delete(results[0]["id"])
```

---

## 🧠 Cognitive Operations

- `memory.consolidate(topic_query, ...)`: Synthesizes a cluster of related episodic memories into a single abstract semantic concept using the **Fréchet Mean** on the hyperboloid.
- `memory.verify_claim(premise, conclusion, threshold=0.30)`: Computes **Geodesic Trust Score** (Lorentz $\mathbb{H}^{32}$ + Cosine 96D) to verify that an LLM conclusion does not hallucinate.
- `memory.list_sessions(limit=100)`: Discovers and enumerates active conversational sessions.
- `memory.explore_hierarchy(concept_id, direction="down")`: Navigates conceptual taxonomy trees in Lorentz space.
- `memory.stats()`: Returns vector counts, cascade configuration, and collection status.

---

## 📄 License
MIT © YARlabs.

