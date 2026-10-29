# Hyperspace Plugin Development Kit (`hyperspace-pdk` / `hs-pdk`)

`hyperspace-pdk` is the official `pgrx`-inspired extension framework for **HyperspaceDB v4.2.0**. It turns HyperspaceDB into an extensible **Spatial & Graph Runtime**, allowing both **built-in core plugins** and **external dynamically loaded plugins (`cdylib`: `.dylib` / `.so` / `.dll` in `./plugins/`)** to add ingestion pipelines, AST/KAG graph extractors, cryptographic transformations, and custom non-Euclidean metrics without bloating the core engine.

---

## 1. Architecture: Built-in vs External Plugins

HyperspaceDB supports a two-tier plugin architecture:

1. **Built-in Core Plugins (`crates/hyperspace-pdk`)**:
   - **`hs-embed` (`embed`)**: Unified in-engine Markdown/text chunker, ONNX Runtime (`ort`), HuggingFace Hub auto-downloader, Remote API driver (`OpenAI`, `Cohere`, `Voyage`, `Mistral`, `Gemini`, `OpenRouter`, `YarInk`, `Generic`), deterministic manifold vectorizer, and KAG sequential/hierarchical chunk linker.
   - **`hs-shield` (`shield`)**: Server-side Zero-Knowledge Orthogonal Projection (`O(D log D)` keyed Fast Walsh-Hadamard Transform + SHA-256 Rademacher sign flipping + Fisher-Yates permutation) defending embeddings at rest/in-index against `Vec2Text` inversion attacks with zero accuracy loss. *(Complements client-side DePIN `ZK-Privacy` encryption in the SDKs, which protects vectors in transit and on untrusted DePIN nodes).*

2. **External Dynamic Plugins (`./plugins/` directory)**:
   - Compiled as standalone Rust `cdylib` crates (`.dylib` on macOS, `.so` on Linux, `.dll` on Windows) using the `hs_export_plugin!` macro (equivalent to `pgrx::pg_module_magic!()`).
   - Automatically discovered and loaded at server startup from `HYPERSPACE_PLUGINS_DIR` (defaults to `./plugins`).
   - **Reference External Plugin**: **`hs-git-ast` (`git_ast`)** located in [`plugins/hs-git-ast`](../../plugins/hs-git-ast) — Instant Codebase AST Topology & Zero-Cost KAG Builder for Rust, Python, TypeScript/JavaScript, and Go.

---

## 2. How to Create an External Plugin (Step-by-Step Developer Guide)

Let's walk through how `hs-git-ast` (or any custom plugin) is structured as an external plugin inside `./plugins/`.

### Step 1: Create the Plugin Crate in `./plugins/<your-plugin>/`

```text
plugins/
├── plugins.toml                # Catalog & default activation config
├── libhs_git_ast.dylib         # Compiled shared library (auto-discovered)
└── hs-git-ast/                 # Standalone external plugin crate
    ├── Cargo.toml
    ├── plugin.toml             # pgrx-style control manifest
    └── src/
        └── lib.rs
```

### Step 2: Configure `Cargo.toml` with `crate-type = ["cdylib"]`

```toml
[package]
name = "hs-my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
name = "hs_my_plugin"
crate-type = ["cdylib", "rlib"]

[dependencies]
hyperspace-pdk = { path = "../../crates/hyperspace-pdk" }
hyperspace-core = { path = "../../crates/hyperspace-core" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 3: Add `plugin.toml` Control File

```toml
name = "my_plugin"
version = "0.1.0"
api_version = 40200
library = "hs_my_plugin"
author = "Your Team"
description = "Custom HyperspaceDB extension"
default_enabled = true
```

### Step 4: Implement `HsPlugin` and Export with `hs_export_plugin!` in `src/lib.rs`

```rust
use hyperspace_pdk::prelude::*;

#[derive(Default)]
pub struct MyCustomPlugin;

impl MyCustomPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HsPlugin for MyCustomPlugin {
    fn manifest(&self) -> PluginManifest {
        hs_plugin_manifest!(
            name: "my_plugin",
            version: "0.1.0",
            author: "Your Team",
            description: "Custom vector enrichment and KAG linker",
            hooks: [HookStage::BeforeInsert],
            procedures: [
                ProcedureSchema {
                    name: "hello".to_string(),
                    description: "Sample custom procedure".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": { "name": { "type": "string" } }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": { "greeting": { "type": "string" } }
                    }),
                }
            ]
        )
    }

    async fn before_insert(
        &self,
        _collection: &str,
        _metric: &str,
        _dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        for pt in points.iter_mut() {
            pt.metadata.insert("processed_by".to_string(), "my_plugin".to_string());
        }
        Ok(())
    }

    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        _ctx: &EngineContext,
    ) -> Result<Value, String> {
        match method {
            "hello" => {
                let who = args.get("name").and_then(|v| v.as_str()).unwrap_or("Hyperspace");
                Ok(json!({ "greeting": format!("Hello, {who}!") }))
            }
            other => Err(format!("Unknown procedure 'my_plugin.{other}'")),
        }
    }
}

// Exports `hs_pdk_api_version` and `hs_plugin_create` C-ABI symbols for dynamic loading:
hs_export_plugin!(MyCustomPlugin::new);
```

### Step 5: Build and Place in `./plugins/`

```bash
cargo build -p hs-my-plugin --release
cp target/release/libhs_my_plugin.dylib ./plugins/   # macOS (.so on Linux, .dll on Windows)
```
*(Note: If your plugin directory inside `./plugins/<name>/` contains `plugin.toml`, `PluginRegistry` also automatically locates its compiled library in `target/release` or `target/debug` during development!)*

---

## 3. Activating, Deactivating, and Hot-Reloading Plugins

### Option A: Via `.env` at Server Startup

```env
# Directory containing external plugins (.dylib / .so / .dll)
HYPERSPACE_PLUGINS_DIR=./plugins

# Comma-separated list of enabled plugins (or * for all)
HYPERSPACE_ENABLED_PLUGINS=embed,shield,git_ast

# Explicitly disable specific plugins at boot
HYPERSPACE_DISABLED_PLUGINS=
```

### Option B: At Runtime via HTTP REST API (Zero Downtime)

1. **Inspect all plugins, their schemas, `source` (`builtin` vs `external:...`), and `enabled` status:**
   ```bash
   curl http://localhost:8080/v1/extensions/schema
   ```
2. **Disable a plugin on the fly (skips its hooks and blocks procedure calls):**
   ```bash
   curl -X POST http://localhost:8080/v1/extensions/git_ast/disable \
     -H "Authorization: Bearer I_LOVE_HYPERSPACEDB"
   ```
3. **Enable a plugin on the fly:**
   ```bash
   curl -X POST http://localhost:8080/v1/extensions/git_ast/enable \
     -H "Authorization: Bearer I_LOVE_HYPERSPACEDB"
   ```
4. **Hot-reload newly added `.dylib` / `.so` plugins from `./plugins/` without restarting the server:**
   ```bash
   curl -X POST http://localhost:8080/v1/extensions/reload \
     -H "Authorization: Bearer I_LOVE_HYPERSPACEDB"
   ```

---

## 4. SDK Usage Examples (`client.ext.<plugin>.<method>`)

### Python SDK
```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051")

# 1. Built-in hs-embed: Zero-Python Markdown Ingestion
client.ext.embed.ingest_text(
    collection="docs",
    text="# Contract 402\nSigned with Acme Corp...\n\n## SLA\n99.99% uptime.",
    max_chars=512,
    link_sequential=True,
)

# 2. External hs-git-ast (loaded from ./plugins/): Instant Codebase KAG Topology
client.ext.git_ast.ingest_directory(
    collection="repo_kag",
    root_path="/path/to/repository",
)

# 3. Built-in hs-shield: Server-Side Zero-Knowledge Vec2Text Defense
client.ext.shield.configure(
    collection="confidential_docs",
    secret_key="kms-derived-tenant-secret",
)
```

### TypeScript SDK
```typescript
import { HyperspaceClient } from 'hyperspace-sdk-ts';

const client = new HyperspaceClient('localhost:50051');

await client.ext.git_ast.ingest_files({
  collection: 'code_kag',
  files: [{ path: 'src/main.rs', content: 'pub fn main() { helper(); }\npub fn helper() {}' }]
});
```

### Rust SDK
```rust
use serde_json::json;

let res = client
    .ext("git_ast")
    .call("parse_source", None, &json!({
        "files": [{ "path": "src/lib.rs", "content": "pub fn init() {}" }]
    }))
    .await?;
```
