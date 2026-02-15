# HyperspaceDB Architecture Guide

HyperspaceDB is a specialized vector database designed for high-performance hyperbolic embedding search. This document details its internal architecture, storage format, and indexing strategies.

---

## 🏗 System Overview

The system follows a strict **Command-Query Separation (CQS)** pattern, tailored for write-heavy ingestion and latency-sensitive search.

```mermaid
graph TD
    Client[Client (gRPC)] -->|Insert| S[Server Service]
    Client -->|Search| S
    
    subgraph Persistence Layer
        S -->|1. Append| WAL[Write-Ahead Log]
        S -->|2. Append| VS[Vector Store]
    end
    
    subgraph Indexing Layer
        S -->|3. Send ID| Q[Async Queue (Channel)]
        Q -->|Pop| W[Indexer Worker]
        W -->|Update| HNSW[HNSW Graph (RAM)]
    end
    
    subgraph Background Tasks
        Snap[Snapshotter] -->|Serialize| Disk[Index Snapshot (.snap)]
    end
```

---

## 💾 Storage Layer (hyperspace-store)

### 1. Vector Storage (`data/`)
Vectors are stored in a segmented, append-only format using **Memory-Mapped Files (mmap)**.

*   **Segments**: Data is split into chunks of 65,536 vectors (`2^16`).
*   **Files**: `chunk_0.hyp`, `chunk_1.hyp`, etc.
*   **Quantization**: Vectors are optionally quantized (e.g., `ScalarI8`), reducing size from 64-bit float to 8-bit integer per dimension (8x compression).

### 2. Write-Ahead Log (`wal.log`)
Writes are durable. Every insert is appended to `wal.log` before being acknowledged. Upon restart, the WAL helps recover data that wasn't yet persisted in the Index Snapshot.

---

## 🕸 Indexing Layer (hyperspace-index)

### Hyperbolic HNSW
We implement a modified **Hierarchical Navigable Small World** graph optimized for the Poincaré Ball model.

*   **Distance Metric**: Poincaré distance formula:
    $$ d(u, v) = \text{acosh}\left(1 + 2 \frac{||u-v||^2}{(1-||u||^2)(1-||v||^2)}\right) $$
*   **Optimization**: We compare $||u-v||^2$ and cached normalization factors $\alpha = 1/(1-||u||^2)$ to avoid expensive `acosh` calls during graph traversal.
*   **Locking**: The graph uses fine-grained `RwLock` per node layer, allowing concurrent searches and updates.

### Dynamic Configuration
Parameters `ef_search` (search depth) and `ef_construction` (build quality) are stored in `AtomicUsize` global config, allowing runtime tuning without restarts.

---

## ⚡️ Performance Traits

1.  **Async Indexing**: Client receives `OK` as soon as data hits the WAL. Indexing happens in the background.
2.  **Zero-Copy Read**: Search uses `mmap` to read quantized vectors directly from OS cache without heap allocation.
3.  **SIMD Acceleration**: Distance calculations use `std::simd` (Portable SIMD) for 4-8x speedup on supported CPUs (AVX2, Neon).

---

## 🔄 Lifecycle

1.  **Startup**: 
    - Load `index.snap` (Rkyv zero-copy deserialization).
    - Replay `wal.log` for any missing vectors.
2.  **Runtime**:
    - Serve read/write requests.
    - Background worker consumes indexing queue.
    - Snapshotter periodically saves graph state.
3.  **Shutdown**:
    - Stop accepting writes.
    - Drain indexing queue.
    - Save final snapshot.
    - Close file handles.

## Memory Management & Stability

### Cold Storage Architecture
HyperspaceDB implements a "Cold Storage" mechanism to handle large numbers of collections efficiently:
1.  **Lazy Loading**: Collections are not loaded into RAM at startup. Instead, only metadata is scanned. The actual collection (vector index, storage) is instantiated from disk only upon the first `get()` request.
2.  **Idle Eviction (Reaper)**: A background task runs every 60 seconds to scan for idle collections. Any collection not accessed for a configurable period (default: 1 hour) is automatically unloaded from memory to free up RAM.
3.  **Graceful Shutdown**: When a collection is evicted or deleted, its `Drop` implementation ensures that all associated background tasks (indexing, snapshotting) are immediately aborted, preventing resource leaks and panicked threads.

This architecture allows HyperspaceDB to support thousands of collections while keeping the active memory footprint low, scaling based on actual usage rather than total data.
