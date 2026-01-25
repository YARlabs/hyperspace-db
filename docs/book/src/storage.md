# Storage Format

HyperspaceDB uses a custom segmented file format designed for:
1.  **Fast Appends** (Zero seek time).
2.  **Mmap Compatibility** (OS manages caching).
3.  **Space Efficiency** (Quantization).

## Segmentation

Data is split into "Chunks" of fixed size ($2^{16} = 65,536$ vectors). This avoids allocating one giant file and allows easier lifecycle management.

*   `data/chunk_0.hyp`
*   `data/chunk_1.hyp`
*   ...

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
