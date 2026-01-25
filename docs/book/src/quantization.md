
# 📉 Binary Quantization

HyperspaceDB supports multiple storage modes to balance **Performance vs Memory**.

## Modes

1. **ScalarI8** (Default): Vectors are compressed to 8-bit integers (`[-127, 127]`).
   - Compression: ~8x vs `f64`.
   - Recall: High (~98%).
2. **Binary** (1-bit): Vectors are compressed to single bits using sign threshold.
   - Compression: **64x** vs `f64`.
   - Performance: Blazing fast Hamming distance.
   - Recall: Moderate (Great for Re-ranking or large datasets).
3. **None**: Full precision `f64` storage.

## Usage

Start the server with your desired mode:

```bash
# Default (ScalarI8)
./hyperspace-server

# Ultra-Compact (Binary)
./hyperspace-server --mode binary

# Full Precision (Research)
./hyperspace-server --mode none
```

> **Note**: The mode is set at server startup and applies to the entire database instance. Mixing modes is not currently supported.
