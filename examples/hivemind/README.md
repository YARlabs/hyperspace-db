# HiveMind - Edge-Cloud Federation Demo

**Local-First AI** showcase application demonstrating HyperspaceDB's Edge-Cloud capabilities.

## Features

- 🧠 **Embedded Vector Database**: HyperspaceDB core runs directly in the desktop app
- 📄 **PDF Ingestion**: Extract text from PDFs and store locally
- 🔍 **Offline Search**: Full-text and semantic search without internet
- 💾 **Local Persistence**: Data stored in `~/.hivemind`
- 🔄 **Background Sync**: (Planned) Sync with cloud HyperspaceDB server

## Architecture

```
┌─────────────────────────────────────┐
│   Tauri Frontend (React/Vite)      │
│   - UI Dashboard                    │
│   - File Picker                     │
│   - Search Interface                │
└──────────────┬──────────────────────┘
               │ IPC Commands
┌──────────────▼──────────────────────┐
│   Tauri Backend (Rust)              │
│   ┌─────────────────────────────┐   │
│   │  HyperspaceDB (Embedded)    │   │
│   │  - HNSW Index               │   │
│   │  - MMap Storage             │   │
│   │  - Snapshot Persistence     │   │
│   └─────────────────────────────┘   │
│   - PDF Extraction (pdf-extract)    │
│   - Embedding (TODO: ONNX)          │
└─────────────────────────────────────┘
```

## Prerequisites

```bash
# Install Tauri CLI
cargo install tauri-cli

# Install Node dependencies
cd examples/hivemind
npm install
```

## Running

```bash
# Development mode
npm run tauri dev

# Build production app
npm run tauri build
```

## Usage

1. **Launch App**: Run `npm run tauri dev`
2. **Ingest PDF**: Click "📂 Ingest PDF" and select a PDF file
3. **View Stats**: Dashboard shows vector count and storage size
4. **Search**: (Coming soon) Search across ingested documents

## Implementation Status

- [x] Tauri app structure
- [x] Embedded HyperspaceDB
- [x] PDF text extraction
- [x] Local storage (~/.hivemind)
- [x] Basic UI dashboard
- [x] Actual embedding (via Server-Side Pipeline)
- [ ] Search UI
- [ ] Cloud sync
- [ ] Knowledge graph visualization

## Technical Details

- **Storage**: Memory-mapped files in `~/.hivemind/store/`
- **Index**: HNSW graph with 1024-dimensional vectors
- **Snapshot**: Periodic serialization to `~/.hivemind/index.snap`
- **Vector Size**: 4096 bytes (1024 dims × 4 bytes f32)

## Next Steps

See `TODO_ADOPTION.md` Task 3.1 for full roadmap.
