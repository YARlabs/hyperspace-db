# HyperspaceDB Skills for AI Agents

[![HyperspaceDB](https://img.shields.io/badge/HyperspaceDB-v3.1.2-cyan)](https://yar.ink/hyperspace)
[![Skills](https://img.shields.io/badge/Skills-5-green)](./skills/)
[![MCP](https://img.shields.io/badge/MCP-30%2B_tools-blue)](https://modelcontextprotocol.io)

**AI agent skills for HyperspaceDB** — a curated set of `SKILL.md` files and agent
configuration guides that teach Cursor, Claude Code, Antigravity, Windsurf, and custom
agents how to correctly and powerfully use HyperspaceDB.

---

## What Are Skills?

Skills are structured instruction files (`SKILL.md`) that AI coding assistants read to
understand how to work with a library or system. They follow the SKILL.md standard:

```yaml
---
name: skill-name
description: >
  When to trigger this skill. Used by the agent to decide whether to read it.
---

# Skill instructions...
```

When you add these skills to your project (e.g., `.agents/skills/`), your AI agent
automatically loads the relevant skill when it detects you're working with HyperspaceDB.

---

## Skills Included

| Skill | Description |
|-------|-------------|
| [`hyperspacedb-core`](./skills/hyperspacedb-core/SKILL.md) | Collections, insert, search, point operations |
| [`hyperspacedb-graph`](./skills/hyperspacedb-graph/SKILL.md) | Graph traversal, Lorentz hierarchy, concept parents |
| [`hyperspacedb-cognitive`](./skills/hyperspacedb-cognitive/SKILL.md) | CoT stability, Koopman momentum, trust score |
| [`hyperspacedb-depin`](./skills/hyperspacedb-depin/SKILL.md) | DePIN nodes, billing, storage economics |
| [`hyperspacedb-mcp`](./skills/hyperspacedb-mcp/SKILL.md) | MCP server setup and all 30+ tool reference |

## Agent Config Files

| File | For |
|------|-----|
| [`agents/cursor-rules.md`](./agents/cursor-rules.md) | Copy to `.cursorrules` in your project |
| [`agents/claude-code-instructions.md`](./agents/claude-code-instructions.md) | Copy to `CLAUDE.md` in your project root |
| [`agents/custom-agent-template.md`](./agents/custom-agent-template.md) | LangChain, LlamaIndex, AutoGen, n8n patterns |

---

## Installation

### Option 1: Copy Skills Directly (Any Agent)

```bash
# Copy all skills to your project's agents config dir
cp -r node_modules/hyperspacedb-skills/skills .agents/skills/
```

Or copy just the skills you need:
```bash
cp -r node_modules/hyperspacedb-skills/skills/hyperspacedb-core .agents/skills/
```

### Option 2: Reference from node_modules (Cursor/Windsurf)

In `.cursor/settings.json`:
```json
{
  "skillsDirectories": [
    "./node_modules/hyperspacedb-skills/skills"
  ]
}
```

### Option 3: Cursor Rules

Copy the content of [`agents/cursor-rules.md`](./agents/cursor-rules.md) into your `.cursorrules` file.

### Option 4: Claude Code

Copy [`agents/claude-code-instructions.md`](./agents/claude-code-instructions.md) content into your `CLAUDE.md`.

---

## MCP Server Quick Reference

For interactive use (Claude Desktop, Cursor Chat, etc.), use the MCP server:

```json
{
  "mcpServers": {
    "hyperspacedb": {
      "command": "npx",
      "args": ["-y", "mcp-hyperspacedb"],
      "env": {
        "HYPERSPACE_HOST": "your-node:50051",
        "HYPERSPACE_API_KEY": "your_key"
      }
    }
  }
}
```

Then ask your AI: *"Create a HyperspaceDB collection for my research papers."*

---

## Supported Agents & Hosts

| Agent/Host | Integration method |
|------------|-------------------|
| **Cursor** | `.cursorrules` + `.cursor/mcp.json` |
| **Claude Code** | `CLAUDE.md` + MCP config |
| **Claude Desktop** | MCP config only |
| **Windsurf** | `~/.windsurf/mcp.json` + skills directory |
| **Antigravity** | `.agents/skills/` directory |
| **LangChain agents** | `custom-agent-template.md` |
| **LlamaIndex agents** | `custom-agent-template.md` |
| **n8n workflows** | `n8n-nodes-hyperspacedb` package |
| **Custom agents** | System prompt injection from template |

---

## Version Compatibility

| `hyperspacedb-skills` | `hyperspace-sdk-ts` | HyperspaceDB server |
|-----------------------|--------------------|--------------------|
| `3.1.2` | `>=3.1.4` | `>=3.1.2` |

---

## License

MIT — © YAR Labs
