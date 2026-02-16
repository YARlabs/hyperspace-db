# Storage Format

HyperspaceDB uses a custom segmented file format designed for:
1.  **Fast Appends** (Zero seek time).
2.  **Mmap Compatibility** (OS manages caching).
3.  **Space Efficiency** (Quantization).

## Segmentation

Data is split into "Chunks" of fixed size ($2^{16} = 65,536$ vectors). This avoids allocating one giant file and allows easier lifecycle management.

*   `data/chunk_0.hyp`
*   ...

## Directory Structure (Multi-Tenancy)

When Multi-Tenancy is active, collections are stored in subdirectories named with the format: `userid_collectionname`.
Example:
*   `data/user1_mycollection/meta.json`
*   `data/user1_mycollection/index`
*   `data/default_admin_public_data/index`

This ensures complete physical isolation of collection data on disk.

## File Layout

Each `.hyp` file is a flat array of fixed-size records. No headers, no metadata. Metadata is stored in the Index Snapshot or recovered from layout.

### Record Structure (`ScalarI8`)

When `QuantizationMode::ScalarI8` is active:

| Byte Offset | Content | Type |
| :--- | :--- | :--- |
| `0..N` | Quantized Coordinates | `[i8; N]` |
| `N..N+4` | Pre-computed Alpha | `f32` |

Total size per vector (for N=8): $8 + 4 = 12$ bytes.
Without quantization (f64), it would be $8 \times 8 = 64$ bytes.
**Savings: ~81%**.

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
