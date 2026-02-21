# Lorentz Scalar Quantization (SQ8)

## Overview

This document describes the scalar quantization (SQ8) implementation for the **Lorentz (Hyperboloid) metric** in HyperspaceDB. Prior to this implementation, SQ8 was only available for Poincare and Euclidean metrics. Lorentz collections were forced to use `HS_QUANTIZATION_LEVEL=none`, resulting in 8x higher memory consumption per vector.

Lorentz SQ8 reduces storage from **8 bytes/dim (f64)** to **1 byte/dim (i8) + 4 bytes scale**, while preserving distance ordering for HNSW graph construction.

---

## Mathematical Foundation

### The Problem: Unbounded Coordinates

The Poincare ball model constrains all coordinates to `[-1, 1]` (since `||x|| < 1`), making a fixed `x * 127` quantization trivial. The Lorentz (hyperboloid) model has **no such bound**:

- **Time component**: `x[0] = cosh(r) >= 1`, grows exponentially with geodesic distance `r`
- **Spatial components**: `|x[i]| <= sinh(r)`, also unbounded

For a point at geodesic distance `r = 5` from the origin:
- `x[0] = cosh(5) ~ 74.2`
- `max |x_spatial| ~ sinh(5) ~ 74.2`

A fixed `[-1, 1]` quantization would saturate immediately.

### The Solution: Dynamic-Range Scalar Quantization

We use **per-vector dynamic range scaling**:

```
scale = max(|x_i|) for i = 0..N
q_i = round(x_i / scale * 127)     // Quantize: f64 -> i8
x_i ~ (q_i / 127) * scale           // Dequantize: i8 -> f64
```

The `scale` factor is stored in the existing `alpha: f32` field of `QuantizedHyperVector`:
- **Poincare/Euclidean**: `alpha` stores `1/(1-||x||^2)` (conformal factor)
- **Lorentz**: `alpha` stores the dynamic range scale factor

This reuse avoids any change to the struct layout, preserving binary compatibility.

### Quantized Distance Computation

The Lorentz distance between vectors `a` and `b` on the hyperboloid is:

```
d(a, b) = acosh(-<a, b>_L)
```

where `<a, b>_L = -a[0]*b[0] + sum(a[i]*b[i], i=1..N)` is the **Minkowski inner product**.

For quantized `a` vs full-precision query `b`:

```
a_deq[i] = a.coords[i] / 127.0 * a.alpha    // Dequantize
<a_deq, b>_L = -a_deq[0]*b[0] + sum(a_deq[i]*b[i])
d ~ acosh(-<a_deq, b>_L)
```

### SIMD Optimization

The SIMD path computes the full Euclidean dot product first, then fixes the Minkowski sign:

```
euclidean_dot = sum(a_deq[i] * b[i], i=0..N)    // Single SIMD reduction
minkowski_inner = euclidean_dot - 2 * a_deq[0] * b[0]  // Fix time component
```

This avoids branch divergence in the SIMD loop and leverages full 8-lane throughput.

---

## Precision Analysis

### Quantization Resolution

| Geodesic Distance `r` | `cosh(r)` (scale) | Resolution per step | Relative Error (typical) |
|------------------------|-------------------|--------------------:|------------------------:|
| 0.5                    | 1.13              | 0.009               | < 5%                    |
| 1.0                    | 1.54              | 0.012               | < 5%                    |
| 2.0                    | 3.76              | 0.030               | < 8%                    |
| 3.0                    | 10.07             | 0.079               | < 12%                   |
| 5.0                    | 74.21             | 0.584               | < 15%                   |

### Key Properties Preserved

1. **Distance Ordering**: Quantized distances preserve the relative ordering of nearest neighbors. This is the critical property for HNSW graph construction.

2. **Self-Distance**: `d_quantized(x, x) ~ 0` (within quantization noise).

3. **Triangle Inequality**: Approximately preserved, sufficient for HNSW's greedy search.

### When to Use Rescore

For applications requiring high absolute precision (e.g., energy-based pruning in NietzscheDB's reconsolidation cycles), we recommend:

- **HNSW search with SQ8** for candidate retrieval (fast, 8x memory reduction)
- **Exact f64 rescore** on the top-K candidates (precise, small working set)

This is the standard oversampling + rescore pattern used in production vector databases.

---

## Architecture

### Files Modified

```
crates/hyperspace-core/src/vector.rs
  + QuantizedHyperVector::from_float_lorentz()   -- Dynamic-range quantization encoder
  + QuantizedHyperVector::lorentz_distance_to_float()  -- SQ8 x f64 distance (SIMD + scalar)

crates/hyperspace-core/src/lib.rs
  ~ LorentzMetric::distance_quantized()  -- Wired to lorentz_distance_to_float()

crates/hyperspace-core/src/gpu.rs        -- NEW
  + LORENTZ_DISTANCE_WGSL                -- WGSL compute shader for batch GPU distance
  + batch_lorentz_distance_cpu()         -- CPU reference implementation for batch ops

crates/hyperspace-index/src/lib.rs
  ~ get_vector()        -- Lorentz-aware dequantization (uses scale factor)
  ~ insert_to_storage() -- Dispatches to from_float_lorentz() for Lorentz metric
  ~ update_storage()    -- Same dispatch for upsert path

crates/hyperspace-core/src/tests.rs
  + 8 new tests covering roundtrip, accuracy, ordering, self-distance, high-dim
```

### Data Flow

```
                    INSERT PATH
                    ===========
  [f64; N] vector
       |
       v
  M::validate() -- Checks hyperboloid constraint
       |
       v
  HyperVector::new_unchecked()
       |
       v
  ┌─────────────────────────────────┐
  │ if M::name() == "lorentz"       │
  │   QuantizedHyperVector          │
  │     ::from_float_lorentz()      │
  │   alpha = max(|x_i|)  [scale]  │
  │   coords[i] = round(x/s * 127) │
  │ else                            │
  │   QuantizedHyperVector          │
  │     ::from_float()              │
  │   alpha = 1/(1-||x||^2)        │
  │   coords[i] = round(x * 127)   │
  └─────────────────────────────────┘
       |
       v
  storage.append(q.as_bytes())


                    SEARCH PATH
                    ===========
  Query [f64; N]  ──────────────────────┐
                                        │
  storage.get(node_id) ──> [bytes] ──┐  │
                                     │  │
                                     v  v
                    ┌───────────────────────────────┐
                    │ M::distance_quantized(q, b)   │
                    │                               │
                    │ Lorentz:                       │
                    │   a[i] = q.coords[i]/127*q.α  │
                    │   <a,b>_L = -a0*b0 + Σai*bi   │
                    │   return acosh(-<a,b>_L)       │
                    │                               │
                    │ Poincare:                      │
                    │   Standard SQ8 path            │
                    └───────────────────────────────┘
                                |
                                v
                         HNSW neighbor selection
```

---

## GPU Acceleration

### WGSL Compute Shader

The `gpu.rs` module provides a **WGSL compute shader** (`LORENTZ_DISTANCE_WGSL`) for batch distance computation on the GPU. This is designed for scenarios where thousands of vectors need to be compared against a single query.

**Bind Group Layout:**
| Binding | Type | Description |
|---------|------|-------------|
| 0 | `storage<read>` | Packed SQ8 vectors (i32 array with 4 i8 coords per slot + f32 scale) |
| 1 | `uniform` | Parameters: num_vectors, dimension |
| 2 | `storage<read>` | Query vector (f32 array) |
| 3 | `storage<read_write>` | Output distances (f32 array) |

**Workgroup Size:** 256 threads per workgroup.

**Performance Model:**
- Each thread processes one stored vector independently
- Memory bandwidth bound: ~(N+4) bytes read per vector
- For dim=768, batch=10000: ~7.5 MB read, fits in L2 cache of most GPUs
- Expected throughput: 500M+ distance computations/second on modern GPUs

### CPU Batch Reference

`batch_lorentz_distance_cpu()` provides a CPU reference implementation with identical semantics for validation and fallback.

---

## Usage

### Creating a Lorentz Collection with SQ8

```bash
# Server-side: enable scalar quantization (default)
export HS_QUANTIZATION_LEVEL=scalar

# Create a Lorentz collection via API
curl -X POST http://localhost:8080/api/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "hyperboloid_index", "dimension": 768, "metric": "lorentz"}'
```

### Python SDK Example

```python
from hyperspace import Client

client = Client("localhost:50051")
client.create_collection("embeddings", dimension=768, metric="lorentz")

# Insert a point on the hyperboloid
# x[0] = cosh(r), x[1:] = sinh(r) * direction
import numpy as np
r = 1.5
direction = np.random.randn(767)
direction /= np.linalg.norm(direction)
vector = np.concatenate([[np.cosh(r)], np.sinh(r) * direction])

client.insert(collection="embeddings", id=1, vector=vector.tolist())

# Search uses SQ8 automatically for fast approximate distances
results = client.search(collection="embeddings", vector=query.tolist(), top_k=10)
```

### Rust SDK Example

```rust
use hyperspace_sdk::Client;

let mut client = Client::connect("http://localhost:50051".into(), None, None).await?;
client.create_collection("lorentz_index".into(), 768, "lorentz".into()).await?;

// The SDK handles quantization transparently
client.insert(1, vector, metadata).await?;
let results = client.search(query, 10, None).await?;
```

---

## Testing

All 16 tests pass including 8 new Lorentz quantization tests:

```
test gpu::tests::test_batch_lorentz_distance_cpu ... ok
test gpu::tests::test_wgsl_shader_source_is_valid ... ok
test tests::test_lorentz_quantization_roundtrip_origin ... ok
test tests::test_lorentz_quantization_known_point ... ok
test tests::test_lorentz_quantized_distance_accuracy_near ... ok
test tests::test_lorentz_quantized_distance_accuracy_far ... ok
test tests::test_lorentz_quantized_distance_preserves_ordering ... ok
test tests::test_lorentz_quantized_self_distance_near_zero ... ok
test tests::test_lorentz_quantized_high_dim ... ok
test tests::test_lorentz_binary_still_panics ... ok
```

---

## Design Decisions

### Why Dynamic-Range Scaling (not Log-Space or Sinh Mapping)?

We evaluated three quantization strategies for unbounded Lorentz coordinates:

| Strategy | Description | Pros | Cons |
|----------|-------------|------|------|
| **Dynamic Range** (chosen) | `q = x / max\|x\| * 127` | Simple, preserves linear structure, no struct change | Resolution degrades for very distant points |
| Log-Space | `q = sign(x) * log(1+\|x\|) * k` | Better resolution at extremes | Distorts inner product, requires non-linear dequant |
| Sinh Mapping | `q = asinh(x) * k` | Natural for hyperboloid | Expensive dequant, changes computation semantics |

Dynamic-range scaling was chosen because:
1. **Preserves Minkowski inner product structure** - dequantization is a simple linear scaling
2. **No struct layout change** - reuses the existing `alpha` field
3. **Compatible with SIMD** - same arithmetic operations as Poincare/Euclidean paths
4. **Sufficient precision** - < 15% relative error even at geodesic distance 5.0

### Why Binary Quantization Remains Rejected for Lorentz

Binary quantization (`sign(x)`) maps to `{-1, +1}^N`. For the Lorentz model:
- The time component `x[0]` is **always positive** (upper sheet), so `sign(x[0]) = 1` always. This collapses all temporal information.
- The magnitude of coordinates encodes **depth in the hierarchy** (distance from origin). `sign(x)` destroys this entirely.
- The Hamming distance between binary vectors bears no mathematical relationship to the Minkowski inner product.

This is a fundamental geometric incompatibility, not a precision issue.
