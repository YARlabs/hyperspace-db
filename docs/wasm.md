# WebAssembly (WASM) & Local-First AI Support

HyperspaceDB includes a dedicated core for running directly in the browser via WebAssembly. This enables **Local-First AI** architectures where vector search happens on the user's device, ensuring zero latency and privacy.

## Feature Overview

| Feature | Server (Core) | WASM (Edge) |
|---------|---------------|-------------|
| **Storage** | Memory-Mapped Files (Disk) | RAM (In-Memory) |
| **Index** | HNSW (Persisted) | HNSW (Rebuilt/Loaded) |
| **Concurrency** | Read/Write Locking | Single-Threaded (mostly) |
| **Network** | gRPC / HTTP | None (Direct Call) |

## Installation

The WASM module is located in `crates/hyperspace-wasm`. To build it, you need `wasm-pack`.

### Prerequisites
```bash
cargo install wasm-pack
```

### Building the WASM Module
We provide a helper script to build the web-compatible WASM package:

```bash
./scripts/build_wasm.sh
```

This will generate the artifacts in `examples/wasm-demo/pkg`.

## Usage (JavaScript / TypeScript)

Once built, you can import `HyperspaceDB` into your web application (React, Vue, Svelte, or Vanilla JS).

```javascript
import init, { HyperspaceDB } from './pkg/hyperspace_wasm.js';

async function main() {
    // 1. Initialize WASM module
    await init();
    
    // 2. Create database instance
    const db = new HyperspaceDB();
    console.log("HyperspaceDB initialized in browser memory!");

    // 3. Insert Vectors
    // Note: Dimension is currently fixed to 1024 for MVP.
    const id = 1;
    const vector = new Float64Array(1024).fill(0.1); 
    
    try {
        db.insert(id, vector);
        console.log("Vector inserted!");
    } catch (e) {
        console.error("Insert failed:", e);
    }

    // 4. Search
    const query = new Float64Array(1024).fill(0.1);
    const k = 5; // Top 5 results
    
    const results = db.search(query, k);
    console.log("Search Results:", results);
    // Output: [{ id: 1, distance: 0.0 }]
}

main();
```

## Performance & Limitations

*   **Memory**: The database runs entirely in browser RAM. 1 Million vectors (1024-dim) would require ~4GB+ of RAM, which might crash a tab. We recommend limiting browser instances to <100k vectors.
*   **Persistence**: Currently, data is lost on page refresh. Future versions will support syncing with IndexedDB or OPFS.
*   **Sync**: Synchronization with the main HyperspaceDB server is planned for v2.0 (Edge-Cloud Federation).
