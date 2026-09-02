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
| `HYPERSPACE_API_KEY` | — | ✅ | Your HyperspaceDB SaaS API key |
| `MEMORY_COLLECTION` | `agent_memory` | — | Collection name to store memories in |
| `HYPERSPACE_HOST` | `the.yar.ink` | — | Override to use a self-hosted instance |

---

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
