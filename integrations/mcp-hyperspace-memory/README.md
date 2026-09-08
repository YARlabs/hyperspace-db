# mcp-hyperspace-memory

> Zero-Code cognitive memory for AI agents — powered by HyperspaceDB SaaS.  
> **8 tools. One config block. No code required.**

## What is this?

`mcp-hyperspace-memory` is a [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server that gives your AI agent a production-grade, persistent memory backed by HyperspaceDB's hyperbolic vector engine.

It is intentionally scoped to **memory operations only** — no DDL, no collection management, no admin tools. If you need full vector DB access, use [`mcp-hyperspacedb`](../mcp-hyperspacedb) alongside it.

---

## Quick Start

### Claude Desktop / Cursor IDE / Windsurf / OpenWebUI

Add to your MCP config (`claude_desktop_config.json` or `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "memory": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspace-memory"],
      "env": {
        "HYPERSPACE_API_KEY": "<YOUR_API_KEY>",
        "MEMORY_COLLECTION": "agent_memory"
      }
    }
  }
}
```

That's it. Restart your agent and it will discover **8 memory tools** automatically.

Get your API key at **[yar.ink/dashboard](https://yar.ink/dashboard)**.

---

## Environment Variables

| Variable | Default | Required | Description |
|:---------|:--------|:--------:|:------------|
| `HYPERSPACE_API_KEY` | — | ✅ | Database API key (`sk_...` for SaaS or your local DB password) |
| `CDE_API_KEY` / `YAR_API_KEY` | — | — | Cloud API key for `v5_Embedding_801` (used when local DB has a custom password) |
| `MEMORY_COLLECTION` | `agent_memory` | — | Collection name to store memories in |
| `HYPERSPACE_HOST` | `the.yar.ink` | — | Set to `localhost:50051` to run with a self-hosted instance |

---

## 🔑 Authentication Architecture: How Keys Work

### Scenario 1: Managed Cloud SaaS (`the.yar.ink`)
```json
{
  "HYPERSPACE_API_KEY": "sk_YOUR_YAR_API_KEY",
  "MEMORY_COLLECTION": "agent_memory"
}
```
A single `sk_` key authenticates both your HyperspaceDB SaaS database storage and the `v5_Embedding_801` vectorizer.

### Scenario 2: Local DB (Default Security) + Cloud v5 Embeddings
```json
{
  "HYPERSPACE_HOST": "localhost:50051",
  "HYPERSPACE_API_KEY": "sk_YOUR_YAR_API_KEY",
  "MEMORY_COLLECTION": "agent_memory"
}
```
Because the host is `localhost` and the key starts with `sk_`, the client connects to your local DB with default credentials (`I_LOVE_HYPERSPACEDB`), while forwarding `sk_...` to `https://the.yar.ink/v1/embeddings` for high-dimensional vectorization. **Data stays 100% on your local disk!**

### Scenario 3: Local DB (Custom Password) + Cloud v5 Embeddings (Two Separate Keys)
If you configured a custom password in your local `.env` (`HYPERSPACE_API_KEY=my_local_secret`), provide **two distinct keys**:
```json
{
  "HYPERSPACE_HOST": "localhost:50051",
  "HYPERSPACE_API_KEY": "my_local_secret",
  "CDE_API_KEY": "sk_YOUR_YAR_API_KEY",
  "MEMORY_COLLECTION": "agent_memory"
}
```
* `HYPERSPACE_API_KEY`: Authenticates to the local gRPC server (`localhost:50051`).
* `CDE_API_KEY`: Authenticates to the cloud embedding API (`https://the.yar.ink/v1/embeddings`).

## Available Tools

| Tool | Description |
|:-----|:------------|
| `memory_remember` | Store a memory/fact/event with session_id and optional tags |
| `memory_recall` | Semantic search across stored memories, optionally scoped by session_id |
| `memory_forget` | Delete a memory by ID |
| `memory_update` | Replace an existing memory (delete + re-insert) |
| `memory_list_sessions` | List unique session_ids that have stored memories |
| `memory_explore_hierarchy` | Navigate Lorentz concept hierarchy (broader/narrower concepts) |
| `memory_consolidate` | Compress a cluster of related memories into one abstract Fréchet mean concept |
| `memory_verify_claim` | Verify logical consistency of a claim using hybrid Lorentz+Cosine geometry |

---

## Architecture

```
Agent (Claude / Cursor)
    ↓  MCP stdio
mcp-hyperspace-memory (this package)
    ↓  gRPC / hyperspace-sdk-ts
HyperspaceDB SaaS (the.yar.ink)
    └─ MEMORY_COLLECTION  (e.g. "agent_memory")
         ├─ Lorentz head  (33D, f32, hyperbolic hierarchy)
         └─ Euclidean head (768D, 1-bit ADC, 61.4× compressed)
```

### Key design decisions

- **`MEMORY_COLLECTION` in env**: The agent never needs to pass the collection name per-call. One env var scopes all memory to the right namespace.
- **Deterministic IDs**: Memory IDs are `hash(text + session_id)` — inserting the same memory twice is idempotent.
- **SaaS default**: `HYPERSPACE_HOST` defaults to `the.yar.ink`. Self-hosted instances work by setting the env var.
- **No admin tools**: No `create_collection`, `delete_collection`, or `rebuild_index`. Memory users never need these.

---

## Comparison with `mcp-hyperspacedb`

| Feature | `mcp-hyperspace-memory` | `mcp-hyperspacedb` |
|:--------|:-----------------------:|:------------------:|
| Target user | AI agent end-user | Developer / DevOps |
| Collection management | ❌ | ✅ |
| Cache control | ❌ | ✅ |
| Cognitive memory tools | ✅ (8 tools) | ❌ |
| Works local + SaaS | SaaS-only | ✅ both |
| Config complexity | 1 API key | host + API key |

---

## Self-Hosting

```json
{
  "env": {
    "HYPERSPACE_HOST": "localhost:50051",
    "HYPERSPACE_API_KEY": "I_LOVE_HYPERSPACEDB",
    "MEMORY_COLLECTION": "my_memory"
  }
}
```

---

## License

MIT — © YARlabs  
Docs: [yar.ink/memory](https://yar.ink/memory)
