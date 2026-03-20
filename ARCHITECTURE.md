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

    subgraph Embedding_Engine ["Embedding Engine"]
        S -->|InsertText| EE[Embedding Service]
        EE -->|Chunking| ONNX[ONNX Backend]
        EE -->|Pooling| API[Cloud API Backend]
    end
```

---

## 🌌 Cosmological Architecture (The Design Philosophy)

HyperspaceDB relies on principles from modern cosmology to execute vector search dynamically. The database maps semantic dispersion to real physical properties:

1. **Multi-Geometric Routing (The Hubble Tension):**
   Upper layers of the HNSW graph utilize the **Klein projective model**, routing queries via cheap Euclidean chord distances (SIMD-optimized), while the bottom exact-search layer converts metrics back to the **Lorentz hyperboloid** for maximum-precision ranking.
2. **Anisotropic Quantization (The Axis of Evil):**
   Semantic vectors are concentrated along principal components (not cleanly isotropic). Our engine applies a weighted Vector Quantization function ($L \approx ||x||^2 ||e_{||}||^2 + h(x) ||e_{\perp}||^2$) to penalize unaligned orthogonal semantic shifts.
3. **Zonal Quantization (The MOND Hypothesis):**
   At the core of the hyperbolic disk, nodes correspond to broad semantic clusters requiring fewer bits of precision (`i8`/`f16`). Nearing the Euclidean horizon ($||x|| \to 1$), exact relationships demand extreme mapping in pure `f64`.
4. **Density Pruning (The $S_8$ Void Tension):**
   Akin to comic voids and clustered galaxies, HyperspaceDB performs Density-based Graph Pruning. Outlying vectors inherit restricted edge mappings, reducing RAM.
5. **Memory Reconsolidation (AI Sleep Mode):**
   Continuous Riemannian SGD pulls vectors towards an attractor state (e.g. Flow Matching) directly via `TriggerReconsolidation`, restructuring the graph dynamically without full re-indexing.
6. **Cross-Feature Matching (Wasserstein-1):**
   Instead of $O(N^3)$ generic OT, we execute an ultra-fast $O(N)$ 1D L1-CDF algorithm to compare distributions along feature axes directly inside the metric dispatch.

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
    *   **Optimization**: We use Squared L2 distance to avoid expensive `sqrt` calls.
    
3.  **Wasserstein-1 (Cross-Feature Matching / 1D CDF)**
    *   **Formula**: $ d(u, v) = \sum |CDF_u(i) - CDF_v(i)| $
    *   **Optimization**: O(N) evaluation instead of O(N^3) Sinkhorn, used for structural distribution matching.

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

HyperspaceDB implements a hybrid replication model supporting both high-availability Cloud deployments and dynamic Edge swarms.

### 1. Leader-Follower Replication (WAL Anti-Entropy)
Used for stable cloud deployments. Maintains strict order.
1.  **Leader**: Accepts writes, appends to local WAL, and broadcasts replication stream.
2.  **Follower**: Connects to Leader via gRPC `Replicate()`, requests stream starting from its last persisted `logical_clock`.
3.  **Consistency**:
    -   **Logical Clocks**: Every WAL entry has a monotonic `logical_clock` ID.
    -   **Anti-Entropy**: Followers catch up by replaying missing entries from the Leader's stream.
    -   **Durability**: Followers persist their own WAL and snapshots entirely independently.

### 2. Edge-to-Edge Gossip Swarm
Used for robotics and Local-First AI where nodes are ephemeral and there is no central Leader.
1.  **UDP Heartbeats**: Nodes broadcast their presence and collection metadata summaries (`CollectionSummary`) every 5 seconds.
2.  **Network Discovery**: Nodes listen on `HS_GOSSIP_PORT` and construct a live registry of active peers. Stale peers are evicted after a 30-second TTL.
3.  **Merkle Delta Sync**: Upon discovery, nodes exchange 256-bucket XOR hashes. Only diverging partitions (buckets) are synchronized, making decentralized synchronization extremely bandwidth-efficient.

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
