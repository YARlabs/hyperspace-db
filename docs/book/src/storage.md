# Storage Format

HyperspaceDB uses a custom segmented file format designed for:
1.  **Fast Appends** (Zero seek time).
2.  **Mmap Compatibility** (OS manages caching).
3.  **Space Efficiency** (Quantization).

## Segmentation

Data is split into "Chunks" of fixed size ($2^{16} = 65,536$ vectors). This avoids allocating one giant file and allows easier lifecycle management.

*   `data/chunk_0.hyp`
*   ...

## LSM-Tree Segmentation

HyperspaceDB 3.0 adopts an **LSM-Tree** architecture. Data flows from hot memory to immutable on-disk segments:

1.  **MemTable (Hot)**: New vectors are indexed in an in-memory HNSW.
2.  **Immutable Chunks (Cold)**: When a WAL segment is rotated, the Flush Worker persists the MemTable into an immutable `.hyp` chunk. During this flush, the in-memory HNSW topology is re-written into a **Spatial Navigable Graph (Vamana / DiskANN format)** to minimize page faults when read via mmap from SSDs.
3.  **Local vs Cloud**: Chunks can live on local NVMe or be tiered to S3.

## S3 Cloud Tiering (Optional)

Using the `s3-tiering` feature, HyperspaceDB can offload cold chunks to an S3-compatible object store.

- **LRU Cache**: A byte-weighted cache (`HS_MAX_LOCAL_CACHE_GB`) manages how much data stays on local disk.
- **Lazy Load**: Search queries automatically trigger a download if a required chunk is only present in the cloud.
- **Backpressure**: Semaphore-limited concurrent downloads prevent IO/Network saturations.

## Directory Structure (Multi-Tenancy)

## File Layout

Each `.hyp` file is a flat array of fixed-size records. No headers, no metadata. Metadata is stored in the Index Snapshot or recovered from layout.

### Zonal Quantization (v3.0 LTS)

For hyperbolic collections, HyperspaceDB automatically applies **Zonal Quantization (MOND theory)** to vectors.
- Vectors near the origin ($||x|| < 0.5$) are tightly compressed as `i8` (`Core`).
- Vectors near the infinite boundary ($||x|| \to 1$) are preserved in pure `f64` (`Boundary`) to maintain strict exact precision required for hierarchical routing.

### Record Structure (`ScalarI8`)

When `QuantizationMode::ScalarI8` is active (and vector is within the `Core` zone):

| Byte Offset | Content | Type |
| :--- | :--- | :--- |
| `0..N` | Quantized Coordinates | `[i8; N]` |
| `N..N+4` | Pre-computed Alpha | `f32` |

Total size per vector (for N=8): $8 + 4 = 12$ bytes.
Without quantization (f64), it would be $8 \times 8 = 64$ bytes.
**Savings: ~81%**.

### Optional raw `f32` storage (v2.2.x)

For `QuantizationMode::None`, you can enable:

- `HS_STORAGE_FLOAT32=true`

In this mode, raw vectors are stored as `f32` in mmap and promoted to `f64` in distance kernels.  
This reduces raw-vector memory footprint by ~50% while preserving numerical behavior in hyperbolic math paths.

## Write-Ahead Log (WAL)

Path: `wal.log`

The WAL ensures durability.
Format:
*   `id` (u32)
*   `vector` ([f64; N])

It is only read during startup if the Index Snapshot is older than the last WAL entry.

## RAM Backend (WASM)

For WebAssembly deployments (`hyperspace-wasm`), the storage backend automatically switches to `RAMVectorStore`.

*   **Structure**: Uses `Vec<Arc<RwLock<Vec<u8>>>>` (Heap Memory) instead of memory-mapped files.
*   **Segmentation**: The same chunking logic (64k vectors) is preserved. This allows the core `HNSW` index to use the same addressing logic (`id >> 16`, `id & 0xFFFF`) regardless of the backend.
*   **Persistence**: Persistence is achieved by serializing the "used" portion of segments into a `Vec<u8>` blob and storing it in the browser's **IndexedDB**.
*   **Pre-allocation**: Creating a DB instance pre-allocates the first chunk (64k * VectorSize bytes) to avoid frequent allocation calls during inserts.
