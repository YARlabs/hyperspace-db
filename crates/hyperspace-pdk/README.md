# Hyperspace Plugin Development Kit (`hyperspace-pdk` / `hs-pdk`)

`hyperspace-pdk` is the official `pgrx`-inspired extension framework for **HyperspaceDB v4.2.0**. It turns HyperspaceDB into an extensible **Spatial & Graph Runtime**, allowing first-party and community plugins to add ingestion pipelines, AST/KAG graph extractors, cryptographic transformations, and custom non-Euclidean metrics without bloating the core engine.

## Architectural Design

1. **Zero-Overhead Native Trait Interface (`HsPlugin`)**:
   - **Lifecycle Pipeline Hooks**: Intercept and mutate operations at `BeforeInsert`, `AfterInsert`, `BeforeSearch`, `AfterSearch`, and `OnLink`.
   - **Custom Engine Procedures (`call_procedure`)**: Expose RPC/REST procedures with full JSON Schema introspection (`PluginManifest`, `ProcedureSchema`).
   - **Direct `EngineContext` Access**: High-throughput access to `ShardedCollection`, Sidecar Payloads, and the CSR `GraphEngine` (`link_batch`).
2. **Adaptive Client SDKs (`client.ext.<plugin>.<method>`)**:
   - The server exposes `GET /v1/extensions/schema` (and gRPC `ListExtensions` / `CallExtension`).
   - Python, TypeScript, and Rust SDKs dynamically route calls to any registered plugin without requiring client regeneration.

---

## Official Built-in Flagship Plugins (v4.2.0)

### 1. `hs-embed` (`embed`) — Zero-Python In-Engine Embedder
Eliminates the "Python Tax" (external FastAPI/PyTorch/LangChain microservices).
- **Procedures**:
  - `ingest_text`: Markdown-aware semantic chunker (`chunk_markdown`) + deterministic multi-manifold vectorizer (`embed_for_manifold` supporting all 12 metrics including `poincare`, `lorentz`, and `hybrid`) + Sidecar Payload storage + automatic `NEXT_CHUNK` sequential graph edges in `GraphEngine`.
  - `chunk_text`: Standalone Markdown-aware text chunker with header breadcrumb propagation.
  - `embed_text`: Standalone manifold-aligned text vectorizer.
- **Hook (`BeforeInsert`)**: Automatically chunks and embeds points that arrive with an empty vector and `metadata["__hs_raw_text"]` (or `metadata["text"]` when `metadata["__hs_auto_embed"] == "true"`).

### 2. `hs-git-ast` (`git_ast`) — Instant Codebase Topology & Zero-Cost KAG
Builds a complete Knowledge Augmented Generation (KAG) graph from source code in milliseconds with zero LLM token cost.
- **Supported Languages**: Rust (`.rs`), Python (`.py`), TypeScript/JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`), and Go (`.go`).
- **Procedures**:
  - `ingest_files`: Parses an array of `{ path, content }` files into AST symbol nodes (`module`, `function`, `struct`, `class`, `trait`, `interface`, `impl`), embeds their signatures and bodies, stores source snippets in Sidecar Payloads, and links `CONTAINS`, `CALLS`, `IMPORTS`, and `IMPLEMENTS` edges in `GraphEngine`.
  - `ingest_directory`: Recursively scans a local repository directory and ingests all supported source files and topological edges in a single pass.
  - `parse_source`: Dry-run AST symbol and relation extractor returning JSON.

### 3. `hs-shield` (`shield`) — Zero-Knowledge Orthogonal Projection
Defends vector embeddings against `Vec2Text` inversion attacks while preserving **100% of exact inner products, cosine similarities, and Euclidean distances**.
- **Algorithm**: Keyed `O(D log D)` Fast Walsh-Hadamard Transform (FWHT) combined with SHA-256 Rademacher sign flipping (`±1`) and Fisher-Yates coordinate permutation.
- **Procedures**:
  - `configure`: Enables/disables `hs-shield` per collection or globally with a secret key.
  - `status`: Reports active shield status and key fingerprints.
  - `transform`: Applies forward (`rotate`) or inverse (`unrotate`) isometric orthogonal projection.
- **Hooks (`BeforeInsert` & `BeforeSearch`)**: Automatically rotates vectors on insert and query vectors on search for shielded collections.

---

## Usage Examples

### Python SDK
```python
from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051")

# 1. Zero-Python Markdown Ingestion (hs-embed)
client.ext.embed.ingest_text(
    collection="docs",
    text="# Contract 402\nSigned with Acme Corp...\n\n## SLA\n99.99% uptime.",
    max_chars=512,
    link_sequential=True,
)

# 2. Instant Codebase KAG Topology (hs-git-ast)
client.ext.git_ast.ingest_directory(
    collection="repo_kag",
    root_path="/path/to/repository",
)

# 3. Zero-Knowledge Vec2Text Defense (hs-shield)
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
    .ext("embed")
    .call("ingest_text", Some("docs"), &json!({
        "text": "# Architecture\nSpatial-vector runtime.",
        "link_sequential": true
    }))
    .await?;
```

