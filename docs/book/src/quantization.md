# 📉 Vector Quantization

HyperspaceDB supports multiple storage modes to balance **Precision vs Memory vs Speed**.
All modes operate transparently — no SDK changes required.

---

## Quantization Modes

| Mode | Bits/dim | Compression | Recall@10 | Best For |
|---|---|---|---|---|
| **None** | 64 (f64) | 1× | 100% | Research, exact recall |
| **ScalarI8** | 8 (i8) | 8× | ~98% | Production default |
| **SQ8 Anisotropic** | 8 (i8) | 8× | ~99%+ | Cosine / L2 (Sprint 6.2) |
| **Binary** | 1 (bit) | 64× | ~75–85% | Re-ranking, large datasets |
| **Lorentz SQ8** | 8 (i8) + scale | ~8× | ~95–98% | Hyperboloid (Lorentz) metric |
| **Zonal (MOND)** | mixed | 30–40%↓ RAM | ~99% | Hyperbolic (core + boundary) |

---

## 1. ScalarI8 (Default)

The default mode. Coordinates are mapped from `f64` to `i8 ∈ [-127, 127]` via:

```
q_i = round(x_i * 127)       // For Poincaré: x_i ∈ (-1, 1)
```

- **Compression**: 8× vs `f64`
- **Recall**: ~98% (@10 neighbors)
- **Distance**: Dequantized at query time (`a_i / 127.0`)

---

## 2. SQ8 Anisotropic (Sprint 6.2 / 7.1 — ScaNN-Inspired)

Standard isotropic quantization applies uniform rounding to all dimensions,
which **distorts the direction** (angle) of a vector. For Cosine/L2 metrics,
angular error causes more recall degradation than magnitude error.

**Anisotropic SQ8** penalizes orthogonal (directional) error far more than
parallel (magnitude) error during the quantization refinement step.

### Loss Function

$$L = \|e_\parallel\|^2 + t_w \cdot \|e_\perp\|^2$$

Where:
- $e_\parallel = (e \cdot \hat{x}) \hat{x}$ — projection of quantization error onto the original vector direction
- $e_\perp$ — component orthogonal to the original vector
- $t_w = 10$ (anisotropy weight) — penalizes directional error 10× more than magnitude error

### Coordinate Descent Refinement

After the initial isotropic quantization, each coordinate is refined by ±1 step
in i8-space and the one minimizing the anisotropic loss is selected:

```rust
for i in 0..N {
    // Try original, +1, -1
    for delta in [-1, 0, 1] {
        let candidate = (q[i] as i16 + delta).clamp(-127, 127) as i8;
        let loss = e_parallel_sq + t_weight * e_ortho_sq;
        if loss < best_loss { best = candidate; }
    }
    q[i] = best;
}
```

### Results

| Metric | Mode | Recall@10 Gain |
|---|---|---|
| Cosine | ScalarI8 → Anisotropic SQ8 | +5–8% |
| L2 | ScalarI8 → Anisotropic SQ8 | +3–5% |

### Implementation

The anisotropic refinement is in `QuantizedHyperVector::from_float()` in
`crates/hyperspace-core/src/vector.rs`.

---

## 3. Lorentz SQ8 (Dynamic-Range)

The Lorentz (hyperboloid) model has **unbounded coordinates**: the time component
`x[0] = cosh(r)` grows exponentially. A fixed `[-1, 1]` mapping would saturate immediately.

**Solution**: Per-vector dynamic-range scaling:

```
scale = max(|x_i|)
q_i   = round(x_i / scale * 127)   // i8
α     = scale                        // stored in alpha field (f32)
```

Dequantization: `x̃_i = (q_i / 127.0) * α`

See [Lorentz SQ8 deep-dive](lorentz_quantization.md) for full details.

---

## 4. Binary (1-bit)

Each coordinate is compressed to its sign bit. Distance uses **Hamming distance**.

- **Compression**: 64× vs `f64`
- **Recall**: ~75–85% (metric-dependent)
- **Use case**: First-pass re-ranking candidate retrieval over very large datasets
- **⚠️ Not supported for Lorentz**: sign destroys hierarchical depth information

---

## 5. Zonal Quantization — MOND (Sprint 6.3)

Inspired by Modified Newtonian Dynamics: near the center of hyperbolic space
the metric is smooth, but it explodes near the horizon.

```rust
pub enum ZonalVector {
    Core(Vec<i8>),       // ||x|| < 0.5: compress to i8 (~8x RAM saving)
    Boundary(Vec<f64>),  // ||x|| >= 0.5: keep full precision
}
```

**Enabled by a separate env var** (independent of `HS_QUANTIZATION_LEVEL`):

```bash
HS_ZONAL_QUANTIZATION=true   # Enable MOND zonal storage
```

When enabled, `zonal_storage: DashMap<NodeId, ZonalVector>` completely **replaces**
the standard `mmap`-based vector store. All read (`get_vector`) and write (`insert_to_storage`)
paths are routed through `zonal_storage`.

- **RAM reduction**: ~30–40% for datasets where most vectors are near the origin (`||x|| < 0.5`)
- **No precision loss** at the boundary (where the metric is most sensitive)
- **Compatible with all metrics**, not just Poincaré

---

## Configuration

Quantization mode is set via environment variable **before creating a collection**.
The mode is saved in `meta.json` alongside each collection and applied on reload.

```bash
# Default (ScalarI8 with Anisotropic refinement)
HS_QUANTIZATION_LEVEL=scalar

# Binary (1-bit Hamming)
HS_QUANTIZATION_LEVEL=binary

# Full f64 precision (debugging / research)
HS_QUANTIZATION_LEVEL=none
```

> **⚠️ Note**: The `--mode` CLI flag does **not** exist. Configuration is exclusively
> through `HS_QUANTIZATION_LEVEL` (env var or `.env` file). The mode is stored per-collection
> in `<data_dir>/<collection>/meta.json` at creation time.

> **Note**: The Lorentz SQ8 path is selected **automatically** when a collection's metric
> is `lorentz`, regardless of `HS_QUANTIZATION_LEVEL`. The `from_float_lorentz()` encoder
> is dispatched by the index layer (`hyperspace-index/src/lib.rs`).

---

## Choosing the Right Mode

```
Dataset characteristics
    │
    ├─ Full precision required (research)? ───────→ HS_QUANTIZATION_LEVEL=none
    │
    ├─ Lorentz/Hyperbolic metric? ────────────────→ Automatic (dynamic-range SQ8)
    │
    ├─ Memory-critical (>100M vectors)? ──────────→ HS_QUANTIZATION_LEVEL=binary
    │
    ├─ Cosine / L2, high recall needed? ──────────→ HS_QUANTIZATION_LEVEL=scalar (default)
    │                                                 → Anisotropic refinement applied
    └─ Hyperbolic, mixed density? ────────────────→ Zonal (MOND) via ZonalVector store
```
