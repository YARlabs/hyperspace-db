/// GPU-accelerated batch Lorentz distance computation.
///
/// This module provides compute shader source (WGSL) and a Rust reference
/// implementation for batch computation of Lorentz (hyperboloid) distances
/// between quantized SQ8 vectors and full-precision query vectors.
///
/// # Architecture
///
/// The GPU path is designed for batch search operations where thousands of
/// quantized vectors need to be compared against a single query vector.
/// Each workgroup thread handles one stored vector, dequantizes it using
/// the per-vector scale factor, computes the Minkowski inner product, and
/// writes `acosh(-<a,b>_L)` to the output buffer.
///
/// # Integration
///
/// To use the GPU path, enable the `gpu` feature flag and ensure a
/// WebGPU/Vulkan-capable device is available. The shader can be loaded
/// with any WGSL-compatible runtime (wgpu, naga, etc.).
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                    GPU Compute Pipeline                      │
/// │                                                              │
/// │  Storage Buffer (read-only)          Uniform Buffer          │
/// │  ┌─────────────────────────┐        ┌──────────────────┐    │
/// │  │ [i8; N] coords per vec  │        │ [f32; N] query   │    │
/// │  │ f32 scale per vec       │        │ u32 num_vectors  │    │
/// │  │ ... (packed SQ8 data)   │        │ u32 dimension    │    │
/// │  └─────────────────────────┘        └──────────────────┘    │
/// │              │                               │               │
/// │              └──────────┬────────────────────┘               │
/// │                         ▼                                    │
/// │              ┌─────────────────────┐                        │
/// │              │   Compute Shader    │                        │
/// │              │   (256 threads/WG)  │                        │
/// │              │                     │                        │
/// │              │  1. Dequantize:     │                        │
/// │              │     x = q/127*scale │                        │
/// │              │                     │                        │
/// │              │  2. Minkowski dot:  │                        │
/// │              │     -x0*q0+Σxi*qi  │                        │
/// │              │                     │                        │
/// │              │  3. acosh(-inner)   │                        │
/// │              └─────────┬───────────┘                        │
/// │                        ▼                                    │
/// │              ┌─────────────────────┐                        │
/// │              │ Output Buffer       │                        │
/// │              │ [f32; num_vectors]  │                        │
/// │              │ (distances)         │                        │
/// │              └─────────────────────┘                        │
/// └──────────────────────────────────────────────────────────────┘
/// ```

/// WGSL compute shader source for batch Lorentz SQ8 distance computation.
///
/// Bind group layout:
/// - @group(0) @binding(0): Storage buffer with packed SQ8 vectors (read-only)
/// - @group(0) @binding(1): Uniform buffer with query vector and metadata
/// - @group(0) @binding(2): Storage buffer for output distances (read-write)
///
/// Each SQ8 vector is packed as: [i8; N] coords + f32 scale (alpha).
/// The shader dequantizes using dynamic-range scaling and computes the
/// Lorentz distance via Minkowski inner product + acosh.
pub const LORENTZ_DISTANCE_WGSL: &str = r#"
// Lorentz SQ8 Batch Distance Compute Shader
// Computes d(a,b) = acosh(-<a,b>_L) for quantized vectors vs float query.

struct Params {
    num_vectors: u32,
    dimension: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<storage, read> quantized_data: array<i32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;

// acosh(x) = ln(x + sqrt(x^2 - 1)) for x >= 1
fn acosh_approx(x: f32) -> f32 {
    return log(x + sqrt(x * x - 1.0));
}

// Extract i8 from packed i32 (4 bytes per i32)
fn extract_i8(packed: i32, byte_idx: u32) -> f32 {
    let shift = byte_idx * 8u;
    let masked = (packed >> shift) & 0xFF;
    // Sign-extend from 8-bit
    let signed = select(masked, masked | i32(0xFFFFFF00), masked > 127);
    return f32(signed);
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vec_idx = gid.x;
    if (vec_idx >= params.num_vectors) {
        return;
    }

    let dim = params.dimension;
    // Each quantized vector: ceil(dim/4) i32s for coords + 1 i32 for scale (as f32 bits)
    let i32s_per_coords = (dim + 3u) / 4u;
    let stride = i32s_per_coords + 1u; // +1 for the scale factor
    let base = vec_idx * stride;

    // Read scale factor (stored as f32 bits in the last i32 slot)
    let scale_bits = quantized_data[base + i32s_per_coords];
    let scale = bitcast<f32>(scale_bits);
    let dequant_factor = scale / 127.0;

    // Compute Minkowski inner product: -a[0]*b[0] + sum(a[i]*b[i], i=1..dim)
    var minkowski_inner: f32 = 0.0;

    for (var i: u32 = 0u; i < dim; i = i + 1u) {
        let packed_idx = base + i / 4u;
        let byte_idx = i % 4u;
        let q_val = extract_i8(quantized_data[packed_idx], byte_idx);
        let a_val = q_val * dequant_factor;
        let b_val = query[i];

        if (i == 0u) {
            // Time-like component: negative sign (Minkowski signature)
            minkowski_inner = minkowski_inner - a_val * b_val;
        } else {
            // Space-like components: positive sign
            minkowski_inner = minkowski_inner + a_val * b_val;
        }
    }

    // d(a,b) = acosh(-<a,b>_L), clamped for numerical stability
    let arg = max(-minkowski_inner, 1.0 + 1e-7);
    distances[vec_idx] = acosh_approx(arg);
}
"#;

/// CPU reference implementation for batch Lorentz SQ8 distance.
///
/// Computes the distance from each quantized vector to the query vector.
/// This serves as a validation baseline for the GPU shader and as a
/// fallback when no GPU is available.
///
/// # Arguments
/// * `quantized_coords` - Slice of i8 coordinate arrays (one per stored vector)
/// * `scales` - Scale factor (alpha) for each quantized vector
/// * `query` - Full-precision query vector
/// * `dimension` - Vector dimensionality
///
/// # Returns
/// Vector of Lorentz distances, one per stored vector.
pub fn batch_lorentz_distance_cpu(
    quantized_coords: &[&[i8]],
    scales: &[f32],
    query: &[f64],
    dimension: usize,
) -> Vec<f64> {
    debug_assert_eq!(quantized_coords.len(), scales.len());

    quantized_coords
        .iter()
        .zip(scales.iter())
        .map(|(coords, &scale)| {
            let s = f64::from(scale);
            let inv_127 = s / 127.0;

            // Minkowski inner product
            let a0 = f64::from(coords[0]) * inv_127;
            let mut inner = -a0 * query[0];

            for i in 1..dimension {
                let a_val = f64::from(coords[i]) * inv_127;
                inner += a_val * query[i];
            }

            let arg = (-inner).max(1.0 + 1e-12);
            arg.acosh()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_lorentz_distance_cpu() {
        let r = 1.5_f64;
        let origin_coords: Vec<i8> = vec![127, 0, 0]; // quantized (1,0,0) with scale=1.0
        let origin_scale = 1.0_f32;

        let query = vec![r.cosh(), r.sinh(), 0.0]; // point at distance r

        let distances = batch_lorentz_distance_cpu(
            &[&origin_coords],
            &[origin_scale],
            &query,
            3,
        );

        let exact = r;
        let relative_error = (distances[0] - exact).abs() / exact;
        assert!(
            relative_error < 0.10,
            "Batch CPU distance error {relative_error:.4} (got {}, expected {exact})",
            distances[0]
        );
    }

    #[test]
    fn test_wgsl_shader_source_is_valid() {
        // Basic smoke test: shader source should contain expected entry points
        assert!(LORENTZ_DISTANCE_WGSL.contains("@compute"));
        assert!(LORENTZ_DISTANCE_WGSL.contains("fn main"));
        assert!(LORENTZ_DISTANCE_WGSL.contains("acosh_approx"));
        assert!(LORENTZ_DISTANCE_WGSL.contains("minkowski_inner"));
    }
}
