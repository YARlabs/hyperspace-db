# HyperspaceDB Architecture Guide

HyperspaceDB is a specialized vector database designed for high-performance hyperbolic embedding search. This document details its internal architecture, storage format, and indexing strategies.

---

## 🏗 System Overview

The system follows a strict **Command-Query Separation (CQS)** pattern, tailored for write-heavy ingestion and latency-sensitive search.

```mermaid
graph TD
    Client[Client (gRPC)] -->|Insert| S[Server Service]
    Client -->|Search| S
    
    subgraph LSM_Tree ["LSM-Tree Storage Engine"]
        S -->|1. Append| WAL[Active WAL]
        WAL -.->|Rotation| Chunk[Immutable Chunk]
        S -->|2. Search| MemT[MemTable (HNSW)]
        MemT -.->|Flush| Chunk
        Chunk -->|Tiering| S3[(S3 / Cloud)]
        Chunk -->|Search| Router{Meta-Router}
        Router -->|Route| Chunk
    end
    
    subgraph Read_Path ["Scatter-Gather Search"]
        S --> Searcher[Chunk Searcher]
        Searcher -->|Parallel mmap| Chunk
        S --> Merge[Result Merger]
    end
```

---

## 💾 Storage Layer (LSM-Tree Architecture)

HyperspaceDB 3.0 uses an **LSM-Tree** inspired architecture for vector search, optimized for high throughput and cloud tiering.

### 1. MemTable & WAL
New vectors are first appended to the **Write-Ahead Log (WAL)** and simultaneously indexed in an in-memory **HNSW MemTable**.
- **Rotation**: Once the WAL reaches `HS_WAL_SEGMENT_SIZE_MB`, it is rotated (frozen).
- **Flushing**: A background **Flush Worker** converts the frozen segment into a highly optimized, immutable **HNSW Chunk** (`.hyp`).
- **RAM Reclamation**: After the flush completes, the old MemTable is atomically swapped for a fresh one, freeing up significant memory.

### 2. Immutable Chunks (SSTables)
Data is stored in segmented `.hyp` files, each containing a subset of the collection.
- **Quantization**: Vectors are optionally quantized (e.g., `ScalarI8`), reducing size by 8x or more.
- **MMap**: Hot chunks are searched directly via memory-mapping, leveraging OS page cache.

### 3. S3 Cloud Tiering (hyperspace-tiering)
Cold chunks can be transparently offloaded to **S3-compatible storage**.
- **Local Cache**: A byte-weighted **LRU cache** manages local disk usage (`HS_MAX_LOCAL_CACHE_GB`).
- **Dynamic Fetch**: Chunks not present locally are automatically downloaded on-demand during search.

---

## 🕸 Indexing Layer (hyperspace-index)

### Metric Abstraction & HNSW
We use a Generic Metric system (`Metric<N>`) to support multiple geometries efficiently, dispatched at compile-time via Const Generics.

#### Cognitive Math & Tribunal Router
HyperspaceDB SDK includes a **Cognitive Math** engine built upon the HNSW graph that performs Phase-Locked Loop context tracking, Koopman extrapolations, and calculates LLM hallucination chaos via `local_entropy`. The **Heterogeneous Tribunal Framework** uses the Graph Traversal API to assign a "Geometric Trust Score" to any LLM claim by validating logical paths between ideas.

1.  **Hyperbolic Space (Poincaré Ball)**
    *   **Formula**: $ d(u, v) = \text{acosh}\left(1 + 2 \frac{||u-v||^2}{(1-||u||^2)(1-||v||^2)}\right) $
    *   **Optimization**: We utilize pre-computed normalization factors $\alpha$ and avoid `acosh` during graph traversal.
    *   **Constraint**: Vectors strictly inside unit ball ($||u|| < 1$).

2.  **Euclidean Space (Squared L2)**
    *   **Formula**: $ d(u, v) = \sum (u_i - v_i)^2 $
    *   **Optimization**: We use Squared L2 distance to avoid expensive `sqrt` calls (monotonicity is preserved for HNSW).
    *   **Compatibility**: Optimized for OpenAI, Cohere, and other standard embeddings.

*   **Locking**: The graph uses fine-grained `RwLock` per node layer, allowing concurrent searches and updates.

### Dynamic Configuration
Parameters `ef_search` (search depth) and `ef_construction` (build quality) are stored in `AtomicUsize` global config, allowing runtime tuning without restarts.

---

## ⚡️ Performance Traits

1.  **Async Indexing**: Client receives `OK` as soon as data hits the WAL. Indexing happens in the background.
2.  **Zero-Copy Read**: Search uses `mmap` to read quantized vectors directly from OS cache without heap allocation.
3.  **SIMD Acceleration**: Distance calculations use `std::simd` (Portable SIMD) for 4-8x speedup on supported CPUs (AVX2, Neon).

3.  **SIMD Acceleration**: Distance calculations use `std::simd` (Portable SIMD) for 4-8x speedup on supported CPUs (AVX2, Neon).

---

## 🏙 Multi-Tenancy (Since v2.0)

HyperspaceDB supports SaaS-style multi-tenancy natively.

-   **Namespace Isolation**: Collections are logical entities namespaced by `user_id`.
    -   Format: `{user_id}_{collection_name}`
    -   Example: `cust_123_vectors`, `cust_456_vectors`.
-   **Security**: API Requests require `x-hyperspace-user-id` header (injected by authenticating proxy or middleware).
-   **Resource Accounting**: Disk usage and vector counts are tracked per-user for billing.

## 🔁 Replication & Consistency (Since v2.0)

HyperspaceDB uses a **Leader-Follower** replication model with **Async Anti-Entropy**.

1.  **Leader**: Accepts writes, appends to local WAL, and broadcasts replication stream.
2.  **Follower**: Connects to Leader, requests stream starting from its last persisted `logical_clock`.
3.  **Consistency**:
    -   **Logical Clocks**: Every WAL entry has a monotonic `logical_clock` ID.
    -   **Anti-Entropy**: Followers catch up by replaying missing entries from the Leader's stream.
    -   **Durability**: Followers persist their own WAL and snapshots entirely independently.

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
