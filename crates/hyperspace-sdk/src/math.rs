fn dot(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len();
    let mut sum = 0.0;
    let mut i = 0;

    #[cfg(feature = "nightly-simd")]
    {
        use std::simd::prelude::*;
        let mut sum_vec = f64x4::splat(0.0);
        while i + 4 <= n {
            let va = f64x4::from_slice(&a[i..i + 4]);
            let vb = f64x4::from_slice(&b[i..i + 4]);
            sum_vec += va * vb;
            i += 4;
        }
        sum += sum_vec.reduce_sum();
    }

    while i < n {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

fn norm_sq(v: &[f64]) -> f64 {
    dot(v, v)
}

fn project_to_ball(x: &[f64], c: f64) -> Vec<f64> {
    let n = norm_sq(x).sqrt();
    let max_n = (1.0 / c.sqrt()) - 1e-9;
    if n <= max_n || n <= 1e-15 {
        return x.to_vec();
    }
    let s = max_n / n;
    x.iter().map(|v| v * s).collect()
}

/// Computes Mobius addition in the Poincare ball model.
///
/// # Errors
///
/// Returns an error if:
/// - input vectors have different dimensions;
/// - curvature `c` is non-positive;
/// - denominator is numerically unstable (too close to zero).
pub fn mobius_add(x: &[f64], y: &[f64], c: f64) -> Result<Vec<f64>, String> {
    if x.len() != y.len() {
        return Err("Dimension mismatch".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let xy = dot(x, y);
    let x2 = norm_sq(x);
    let y2 = norm_sq(y);
    let num_left = 1.0 + 2.0 * c * xy + c * y2;
    let num_right = 1.0 - c * x2;
    let den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
    if den.abs() < 1e-15 {
        return Err("Mobius addition denominator too small".to_string());
    }

    let n = x.len();
    let mut result = vec![0.0; n];
    let mut i = 0;

    #[cfg(feature = "nightly-simd")]
    {
        use std::simd::prelude::*;
        let num_left_v = f64x4::splat(num_left);
        let num_right_v = f64x4::splat(num_right);
        let den_inv_v = f64x4::splat(1.0 / den);

        while i + 4 <= n {
            let vx = f64x4::from_slice(&x[i..i + 4]);
            let vy = f64x4::from_slice(&y[i..i + 4]);
            let res_v = (vx * num_left_v + vy * num_right_v) * den_inv_v;
            res_v.copy_to_slice(&mut result[i..i + 4]);
            i += 4;
        }
    }

    let den_inv = 1.0 / den;
    while i < n {
        result[i] = (num_left * x[i] + num_right * y[i]) * den_inv;
        i += 1;
    }

    Ok(result)
}

/// Maps a tangent vector `v` at point `x` to the manifold.
///
/// # Errors
///
/// Returns an error if:
/// - input vectors have different dimensions;
/// - curvature `c` is non-positive;
/// - internal Mobius addition fails.
pub fn exp_map(x: &[f64], v: &[f64], c: f64) -> Result<Vec<f64>, String> {
    if x.len() != v.len() {
        return Err("Dimension mismatch".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let x2 = norm_sq(x);
    let v_norm_sq = norm_sq(v);
    let v_norm = v_norm_sq.sqrt();
    if v_norm < 1e-15 {
        return Ok(x.to_vec());
    }
    let lambda_x = 2.0 / (1.0 - c * x2).max(1e-15);
    let scale = (c.sqrt() * lambda_x * v_norm / 2.0).tanh() / (c.sqrt() * v_norm);

    let xv = dot(x, v);
    let xy = scale * xv;
    let y2 = scale * scale * v_norm_sq;

    let num_left = 1.0 + 2.0 * c * xy + c * y2;
    let num_right = 1.0 - c * x2;
    let den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
    if den.abs() < 1e-15 {
        return Err("Mobius addition denominator too small".to_string());
    }

    let n = x.len();
    let mut result = vec![0.0; n];
    let mut i = 0;

    #[cfg(feature = "nightly-simd")]
    {
        use std::simd::prelude::*;
        let num_left_v = f64x4::splat(num_left);
        let num_right_v = f64x4::splat(num_right * scale);
        let den_inv_v = f64x4::splat(1.0 / den);

        while i + 4 <= n {
            let vx = f64x4::from_slice(&x[i..i + 4]);
            let vv = f64x4::from_slice(&v[i..i + 4]);
            let res_v = (vx * num_left_v + vv * num_right_v) * den_inv_v;
            res_v.copy_to_slice(&mut result[i..i + 4]);
            i += 4;
        }
    }

    let den_inv = 1.0 / den;
    while i < n {
        result[i] = (num_left * x[i] + num_right * scale * v[i]) * den_inv;
        i += 1;
    }

    Ok(result)
}

/// Maps a manifold point `y` back to the tangent space at `x`.
///
/// # Errors
///
/// Returns an error if:
/// - input vectors have different dimensions;
/// - curvature `c` is non-positive;
/// - internal Mobius addition fails.
pub fn log_map(x: &[f64], y: &[f64], c: f64) -> Result<Vec<f64>, String> {
    if x.len() != y.len() {
        return Err("Dimension mismatch".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let neg_x: Vec<f64> = x.iter().map(|xi| -xi).collect();
    let delta = mobius_add(&neg_x, y, c)?;
    let delta_norm = norm_sq(&delta).sqrt();
    if delta_norm < 1e-15 {
        return Ok(vec![0.0; x.len()]);
    }
    let x2 = norm_sq(x);
    let lambda_x = 2.0 / (1.0 - c * x2).max(1e-15);
    let arg = (c.sqrt() * delta_norm).min(1.0 - 1e-15);
    let factor = (2.0 / (lambda_x * c.sqrt())) * arg.atanh();
    Ok(delta.iter().map(|d| factor * d / delta_norm).collect())
}

/// Computes Riemannian gradient in the Poincare ball from Euclidean gradient.
///
/// # Errors
///
/// Returns an error if:
/// - point and gradient have different dimensions;
/// - curvature `c` is non-positive.
pub fn riemannian_gradient(x: &[f64], euclidean_grad: &[f64], c: f64) -> Result<Vec<f64>, String> {
    if x.len() != euclidean_grad.len() {
        return Err("Dimension mismatch".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let x2 = norm_sq(x);
    let lambda_x = 2.0 / (1.0 - c * x2).max(1e-15);
    let scale = 1.0 / (lambda_x * lambda_x);
    Ok(euclidean_grad.iter().map(|g| scale * g).collect())
}

fn gyro(u: &[f64], v: &[f64], w: &[f64], c: f64) -> Result<Vec<f64>, String> {
    let uv = mobius_add(u, v, c)?;
    let vw = mobius_add(v, w, c)?;
    let left = mobius_add(u, &vw, c)?;
    let neg_uv: Vec<f64> = uv.iter().map(|x| -x).collect();
    mobius_add(&neg_uv, &left, c)
}

/// Parallel-transports tangent vector `v` from point `x` to point `y`.
///
/// # Errors
///
/// Returns an error if:
/// - dimensions are inconsistent;
/// - curvature `c` is non-positive;
/// - internal Mobius operations fail.
pub fn parallel_transport(x: &[f64], y: &[f64], v: &[f64], c: f64) -> Result<Vec<f64>, String> {
    if x.len() != y.len() || x.len() != v.len() {
        return Err("Dimension mismatch".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let neg_x: Vec<f64> = x.iter().map(|xi| -xi).collect();
    let gyr = gyro(y, &neg_x, v, c)?;
    let lambda_x = 2.0 / (1.0 - c * norm_sq(x)).max(1e-15);
    let lambda_y = 2.0 / (1.0 - c * norm_sq(y)).max(1e-15);
    let scale = lambda_x / lambda_y;
    Ok(gyr.into_iter().map(|g| g * scale).collect())
}

/// Computes Fréchet mean on the Poincare ball for a set of points.
///
/// # Errors
///
/// Returns an error if:
/// - input set is empty;
/// - points have inconsistent dimensions;
/// - curvature `c` is non-positive;
/// - internal `log_map`/`exp_map` operations fail.
pub fn frechet_mean(
    points: &[Vec<f64>],
    c: f64,
    max_iter: usize,
    tol: f64,
) -> Result<Vec<f64>, String> {
    if points.is_empty() {
        return Err("Points set cannot be empty".to_string());
    }
    if c <= 0.0 {
        return Err("Curvature c must be > 0".to_string());
    }
    let dim = points[0].len();
    if points.iter().any(|p| p.len() != dim) {
        return Err("Dimension mismatch".to_string());
    }
    let n_points_u32 =
        u32::try_from(points.len()).map_err(|_| "Points set is too large".to_string())?;
    let inv = 1.0 / f64::from(n_points_u32);
    let mut mu = project_to_ball(&points[0], c);
    let iter_n = max_iter.max(1);
    for _ in 0..iter_n {
        let mut grad = vec![0.0; dim];
        for p in points {
            let lg = log_map(&mu, p, c)?;
            for (g, v) in grad.iter_mut().zip(lg.iter()) {
                *g += *v;
            }
        }
        for g in &mut grad {
            *g *= inv;
        }
        let g_norm = norm_sq(&grad).sqrt();
        if g_norm <= tol.max(1e-15) {
            break;
        }
        mu = exp_map(&mu, &grad, c)?;
        mu = project_to_ball(&mu, c);
    }
    Ok(mu)
}

// ==========================================
// Lorentz Model Math (Hyperboloid)
// ==========================================

/// Computes the Minkowski inner product (Lorentz product) between two vectors.
/// ⟨u, v⟩_L = -`u_0` * `v_0` + `Σ(u_i` * `v_i`)
#[must_use]
pub fn lorentz_product(u: &[f32], v: &[f32]) -> f32 {
    if u.is_empty() || v.is_empty() {
        return 0.0;
    }
    let mut dot = -u[0] * v[0];
    for (ui, vi) in u.iter().skip(1).zip(v.iter().skip(1)) {
        dot += ui * vi;
    }
    dot
}

/// Computes the Lorentz distance between two points on the hyperboloid.
#[must_use]
pub fn lorentz_dist(u: &[f32], v: &[f32]) -> f32 {
    let inner = -lorentz_product(u, v);
    // Clamp to 1.0 to avoid NaN in acosh due to floating point inaccuracies
    let arg = inner.max(1.0);
    arg.acosh()
}

/// Computes the parallel transport of a tangent vector `v` from point `x` to point `y`
/// in the Lorentz model.
/// # Errors
///
/// Returns a string error if denominator of parallel transport is zero or invalid due to numerical instability.
pub fn lorentz_parallel_transport(x: &[f32], y: &[f32], v: &[f32]) -> Result<Vec<f32>, String> {
    if x.len() != y.len() || x.len() != v.len() {
        return Err("Dimension mismatch".to_string());
    }
    let inner_xy = lorentz_product(x, y);
    let inner_yv = lorentz_product(y, v);
    let denom = 1.0 - inner_xy;

    // Prevent division by zero
    let factor = if denom.abs() < 1e-7 {
        0.0
    } else {
        inner_yv / denom
    };

    Ok(v.iter()
        .zip(x.iter())
        .zip(y.iter())
        .map(|((vi, xi), yi)| vi + factor * (xi + yi))
        .collect())
}

/// Converts a point from the Lorentz model (Hyperboloid) to the Poincaré Ball model.
#[must_use]
pub fn lorentz_to_poincare(x: &[f32]) -> Vec<f32> {
    if x.is_empty() {
        return vec![];
    }
    let denom = 1.0 + x[0];
    if denom.abs() < 1e-7 {
        return vec![0.0; x.len() - 1]; // Fallback
    }
    x.iter().skip(1).map(|xi| xi / denom).collect()
}

/// Converts a point from the Poincaré Ball model to the Lorentz model (Hyperboloid).
#[must_use]
pub fn poincare_to_lorentz(p: &[f32]) -> Vec<f32> {
    let p_sq: f32 = p.iter().map(|v| v * v).sum();
    let denom = 1.0 - p_sq;
    let denom = if denom.abs() < 1e-7 { 1e-7 } else { denom }; // Prevent division by zero

    let mut x = Vec::with_capacity(p.len() + 1);
    x.push((1.0 + p_sq) / denom);
    for pi in p {
        x.push(2.0 * pi / denom);
    }
    x
}

// ==========================================
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================

/// Calculates the spatial entropy (dispersion) of a `candidate` vector relative to its `neighbors`.
/// Used to track LLM hallucinations (Task 2.3.1).
/// Returns a value in [0, 1) where values approaching 1 imply high chaos (hallucination).
///
/// # Errors
/// Returns an error if internal hyperbolic mapping (`log_map`) fails due to curvature constraints.
#[allow(clippy::cast_precision_loss)]
pub fn local_entropy(candidate: &[f64], neighbors: &[Vec<f64>], c: f64) -> Result<f64, String> {
    if neighbors.is_empty() {
        return Ok(1.0); // Infinite entropy without neighbors
    }
    let mut total_deviation = 0.0;
    for neighbor in neighbors {
        let diff = log_map(candidate, neighbor, c)?;
        total_deviation += norm_sq(&diff).sqrt();
    }
    let mean_deviation = total_deviation / neighbors.len() as f64;
    // Logarithmic compression mapping deviation to [0, 1)
    Ok(1.0 - (-mean_deviation).exp())
}

/// Evaluates if a trajectory of vectors (e.g. Chain of Thought) converges to an attractor.
/// Calculates the average energy derivative (Lyapunov function derivative).
/// Negative values indicate convergence (stable), positive indicate divergence (chaos/hallucination).
///
/// # Errors
/// Returns an error if `trajectory` is too short, or if Fréchet mean or `log_map` calculations fail.
#[allow(clippy::cast_precision_loss)]
pub fn lyapunov_convergence(trajectory: &[Vec<f64>], c: f64) -> Result<f64, String> {
    if trajectory.len() < 3 {
        return Err("Need at least 3 points to evaluate convergence trend".to_string());
    }
    // The attractor is approximated by Fréchet mean
    let attractor = frechet_mean(trajectory, c, 32, 1e-6)?;

    let mut v_diff_sum = 0.0;
    for i in 0..trajectory.len() - 1 {
        let v_t0 = norm_sq(&log_map(&attractor, &trajectory[i], c)?).sqrt();
        let v_t1 = norm_sq(&log_map(&attractor, &trajectory[i + 1], c)?).sqrt();
        v_diff_sum += v_t1 - v_t0;
    }
    // Average rate of change of Lyapunov energy function V(x)
    Ok(v_diff_sum / (trajectory.len() - 1) as f64)
}

/// Extrapolates the trajectory in linear space (Koopman linearization) by tracking the
/// shift vector from `past` to `current` and projecting it forward.
///
/// # Errors
/// Returns an error if internal manifold mappings fail due to dimensions or non-positive curvature.
pub fn koopman_extrapolate(
    past: &[f64],
    current: &[f64],
    steps: f64,
    c: f64,
) -> Result<Vec<f64>, String> {
    // 1. Get velocity at past
    let velocity_at_past = log_map(past, current, c)?;
    // 2. Parallel transport to current
    let velocity_at_current = parallel_transport(past, current, &velocity_at_past, c)?;
    // 3. Extrapolate
    let future_velocity: Vec<f64> = velocity_at_current.iter().map(|v| v * steps).collect();
    // 4. Map back to manifold
    exp_map(current, &future_velocity, c)
}

/// Resonates a thought vector towards a global context vector (Phase-Locked Loop context synchronization).
/// Pulls the thought towards the context along the geodesic by `resonance_factor` [0, 1].
///
/// # Errors
/// Returns an error if the manifold mappings (`log_map`, `exp_map`) fail.
pub fn context_resonance(
    thought: &[f64],
    global_context: &[f64],
    resonance_factor: f64,
    c: f64,
) -> Result<Vec<f64>, String> {
    let pull_dir = log_map(thought, global_context, c)?;
    let factor = resonance_factor.clamp(0.0, 1.0);
    let applied_pull: Vec<f64> = pull_dir.iter().map(|v| v * factor).collect();
    exp_map(thought, &applied_pull, c)
}

pub fn poincare_to_lorentz_f64(p: &[f64]) -> Vec<f64> {
    let p_sq: f64 = p.iter().map(|v| v * v).sum();
    let denom = 1.0 - p_sq;
    let denom = if denom.abs() < 1e-7 { 1e-7 } else { denom };

    let mut x = Vec::with_capacity(p.len() + 1);
    x.push((1.0 + p_sq) / denom);
    for pi in p {
        x.push(2.0 * pi / denom);
    }
    x
}

pub fn lorentz_to_poincare_f64(x: &[f64]) -> Vec<f64> {
    if x.is_empty() {
        return Vec::new();
    }
    let denom = x[0] + 1.0;
    let denom = if denom.abs() < 1e-7 { 1e-7 } else { denom };
    x.iter().skip(1).map(|xi| xi / denom).collect()
}

pub fn generate_orthogonal_matrix(dimension: usize, seed_bytes: &[u8]) -> Vec<Vec<f64>> {
    use sha2::{Digest, Sha256};
    let mut matrix = vec![vec![0.0; dimension]; dimension];
    let mut current_hash = Sha256::digest(seed_bytes);
    let mut hash_offset = 0;

    for i in 0..dimension {
        for j in 0..dimension {
            if hash_offset >= 32 {
                current_hash = Sha256::digest(&current_hash);
                hash_offset = 0;
            }
            let bytes = [
                current_hash[hash_offset],
                current_hash[hash_offset + 1],
                current_hash[hash_offset + 2],
                current_hash[hash_offset + 3],
            ];
            let val_uint = u32::from_le_bytes(bytes);
            let val = (val_uint as f64 / 4_294_967_296.0) * 2.0 - 1.0;
            matrix[i][j] = val;
            hash_offset += 4;
        }
    }

    // Gram-Schmidt with Reorthogonalization
    for i in 0..dimension {
        let mut v = matrix[i].clone();
        for j in 0..i {
            let u = &matrix[j];
            let mut u_dot_v = dot(u, &v);
            for k in 0..dimension {
                v[k] -= u_dot_v * u[k];
            }
            u_dot_v = dot(u, &v);
            for k in 0..dimension {
                v[k] -= u_dot_v * u[k];
            }
        }
        let v_norm = norm_sq(&v).sqrt();
        if v_norm > 1e-15 {
            for k in 0..dimension {
                matrix[i][k] = v[k] / v_norm;
            }
        } else {
            for k in 0..dimension {
                matrix[i][k] = 0.0;
            }
            matrix[i][i] = 1.0;
        }
    }

    matrix
}

pub fn generate_lorentz_matrix(dimension: usize, seed_bytes: &[u8]) -> Vec<Vec<f64>> {
    use sha2::{Digest, Sha256};
    let d = dimension - 1;

    let mut r_seed = seed_bytes.to_vec();
    r_seed.extend_from_slice(b"spatial");
    let spatial_matrix = generate_orthogonal_matrix(d, &r_seed);

    let mut b_seed = seed_bytes.to_vec();
    b_seed.extend_from_slice(b"boost");
    let mut beta = vec![0.0; d];
    let mut current_hash = Sha256::digest(&b_seed);
    let mut hash_offset = 0;

    for i in 0..d {
        if hash_offset >= 32 {
            current_hash = Sha256::digest(&current_hash);
            hash_offset = 0;
        }
        let bytes = [
            current_hash[hash_offset],
            current_hash[hash_offset + 1],
            current_hash[hash_offset + 2],
            current_hash[hash_offset + 3],
        ];
        let val_uint = u32::from_le_bytes(bytes);
        let val = (val_uint as f64 / 4_294_967_296.0) * 2.0 - 1.0;
        beta[i] = val;
        hash_offset += 4;
    }

    let beta_norm = norm_sq(&beta).sqrt();
    if beta_norm > 1e-15 {
        for i in 0..d {
            beta[i] = (beta[i] / beta_norm) * 0.1;
        }
    } else {
        for i in 0..d {
            beta[i] = 0.0;
        }
        beta[0] = 0.1;
    }

    let beta_sq = dot(&beta, &beta);
    let gamma = 1.0 / (1.0 - beta_sq).sqrt();

    let mut lambda_b = vec![vec![0.0; dimension]; dimension];
    lambda_b[0][0] = gamma;
    for j in 1..dimension {
        lambda_b[0][j] = -gamma * beta[j - 1];
        lambda_b[j][0] = -gamma * beta[j - 1];
    }

    let factor = (gamma - 1.0) / beta_sq;
    for i in 1..dimension {
        for j in 1..dimension {
            let delta = if i == j { 1.0 } else { 0.0 };
            lambda_b[i][j] = delta + factor * beta[i - 1] * beta[j - 1];
        }
    }

    let mut lambda_r = vec![vec![0.0; dimension]; dimension];
    lambda_r[0][0] = 1.0;
    for i in 1..dimension {
        for j in 1..dimension {
            lambda_r[i][j] = spatial_matrix[i - 1][j - 1];
        }
    }

    let mut lambda = vec![vec![0.0; dimension]; dimension];
    for i in 0..dimension {
        for j in 0..dimension {
            let mut sum = 0.0;
            for k in 0..dimension {
                sum += lambda_b[i][k] * lambda_r[k][j];
            }
            lambda[i][j] = sum;
        }
    }

    lambda
}

pub fn project_vector(v: &[f64], matrix: &[Vec<f64>]) -> Vec<f64> {
    let dim = v.len();
    let mut projected = vec![0.0; dim];
    for j in 0..dim {
        let mut sum = 0.0;
        for i in 0..dim {
            sum += v[i] * matrix[i][j];
        }
        projected[j] = sum;
    }
    projected
}

pub fn inject_anisotropic_noise(v: &[f64], seed_bytes: &[u8], sigma: f64) -> Vec<f64> {
    if v.is_empty() || sigma <= 0.0 {
        return v.to_vec();
    }
    let dim = v.len();
    use sha2::{Digest, Sha256};
    let mut noise = vec![0.0; dim];
    let mut current_hash = Sha256::digest(seed_bytes);
    let mut hash_offset = 0;

    for i in 0..dim {
        if hash_offset >= 32 {
            current_hash = Sha256::digest(&current_hash);
            hash_offset = 0;
        }
        let bytes = [
            current_hash[hash_offset],
            current_hash[hash_offset + 1],
            current_hash[hash_offset + 2],
            current_hash[hash_offset + 3],
        ];
        let val_uint = u32::from_le_bytes(bytes);
        let val = (val_uint as f64 / 4_294_967_296.0) * 2.0 - 1.0;
        noise[i] = val;
        hash_offset += 4;
    }

    let v_norm = norm_sq(v).sqrt();
    if v_norm > 1e-15 {
        let proj_length = dot(&noise, v) / (v_norm * v_norm);
        for i in 0..dim {
            noise[i] -= proj_length * v[i];
        }
    }

    let noise_norm = norm_sq(&noise).sqrt();
    if noise_norm > 1e-15 {
        let scale = sigma * v_norm / noise_norm;
        v.iter()
            .zip(noise.iter())
            .map(|(vi, ni)| vi + ni * scale)
            .collect()
    } else {
        v.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobius_add_identity() {
        let x = vec![0.1, -0.2, 0.05];
        let zero = vec![0.0, 0.0, 0.0];
        let out = mobius_add(&x, &zero, 1.0).expect("mobius add");
        for (a, b) in out.iter().zip(x.iter()) {
            assert!((a - b).abs() < 1e-12);
        }
    }

    #[test]
    fn test_exp_log_roundtrip_small_step() {
        let x = vec![0.05, -0.03];
        let v = vec![0.001, 0.002];
        let y = exp_map(&x, &v, 1.0).expect("exp map");
        let v_back = log_map(&x, &y, 1.0).expect("log map");
        for (a, b) in v.iter().zip(v_back.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_riemannian_gradient_shape() {
        let x = vec![0.1, -0.2, 0.05];
        let g = vec![0.01, 0.02, -0.03];
        let rg = riemannian_gradient(&x, &g, 1.0).expect("riemannian grad");
        assert_eq!(rg.len(), g.len());
    }

    #[test]
    fn test_parallel_transport_shape() {
        let x = vec![0.05, -0.02];
        let y = vec![0.02, 0.03];
        let v = vec![0.001, -0.002];
        let out = parallel_transport(&x, &y, &v, 1.0).expect("parallel transport");
        assert_eq!(out.len(), v.len());
    }

    #[test]
    fn test_frechet_mean_shape() {
        let pts = vec![vec![0.05, 0.01], vec![0.06, 0.02], vec![0.04, 0.0]];
        let mu = frechet_mean(&pts, 1.0, 32, 1e-8).expect("frechet mean");
        assert_eq!(mu.len(), 2);
    }

    #[test]
    fn test_cognitive_math() {
        // Local Entropy
        let candidate = vec![0.1, 0.1];
        let neighbors = vec![vec![0.11, 0.1], vec![0.1, 0.12], vec![0.09, 0.09]];
        let entropy = local_entropy(&candidate, &neighbors, 1.0).unwrap();
        // Since neighbors are close, entropy should be low (close to 0)
        assert!(entropy < 0.1);

        // Lyapunov Convergence
        let trajectory_converging = vec![
            vec![0.5, 0.5],
            vec![0.3, 0.3],
            vec![0.1, 0.1],
            vec![0.05, 0.05],
        ];
        let lyapunov = lyapunov_convergence(&trajectory_converging, 1.0).unwrap();
        assert!(lyapunov < 0.0); // derivative < 0 implies convergence

        // Context Resonance
        let thought = vec![0.5, 0.0];
        let global_ctx = vec![0.0, 0.5];
        let pull = context_resonance(&thought, &global_ctx, 0.5, 1.0).unwrap();
        assert_eq!(pull.len(), 2);

        // Koopman Extrapolation
        let past = vec![0.1, 0.2];
        let current = vec![0.15, 0.25];
        let predicted = koopman_extrapolate(&past, &current, 1.0, 1.0).unwrap();
        assert_eq!(predicted.len(), 2);
    }

    #[test]
    fn test_lorentz_product_and_distance() {
        let u = vec![2.0f32, (3.0f32).sqrt()]; // ||u||^2 = -4 + 3 = -1
        let v = vec![2.0f32, (3.0f32).sqrt()];
        let dist = lorentz_dist(&u, &v);
        assert!(dist < 1e-4);

        let w = vec![3.0f32, (8.0f32).sqrt()]; // ||w||^2 = -9 + 8 = -1
        let product = lorentz_product(&u, &w);
        let expected = -2.0 * 3.0 + (3.0f32).sqrt() * (8.0f32).sqrt();
        assert!((product - expected).abs() < 1e-4);
        assert!(lorentz_dist(&u, &w) > 0.0);
    }

    #[test]
    fn test_lorentz_parallel_transport() {
        let x = vec![1.0f32, 0.0];
        let y = vec![2.0f32, (3.0f32).sqrt()];
        let v = vec![0.0f32, 1.0]; // Tangent to x: <v, x>_L = 0
        let transported = lorentz_parallel_transport(&x, &y, &v).unwrap();
        assert_eq!(transported.len(), 2);

        // <transported, y>_L should remain close to 0 as it's a tangent vector
        let dot_y = lorentz_product(&transported, &y);
        assert!(dot_y.abs() < 1e-4);
    }

    #[test]
    fn test_lorentz_poincare_conversions() {
        let l = vec![2.0f32, (3.0f32).sqrt()];
        let p = lorentz_to_poincare(&l);
        assert_eq!(p.len(), 1);
        let expected_p = (3.0f32).sqrt() / 3.0;
        assert!((p[0] - expected_p).abs() < 1e-4);

        let l_back = poincare_to_lorentz(&p);
        assert_eq!(l_back.len(), 2);
        for (v_orig, v_back) in l.iter().zip(l_back.iter()) {
            assert!((v_orig - v_back).abs() < 1e-4);
        }
    }

    #[test]
    fn test_zk_crypto_preservation() {
        let dim = 64;
        let seed = b"secret_seed_rust";
        let matrix = generate_orthogonal_matrix(dim, seed);

        for i in 0..dim {
            for j in 0..dim {
                let sum = dot(&matrix[i], &matrix[j]);
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((sum - expected).abs() < 1e-9);
            }
        }

        let mut u = vec![0.0; dim];
        let mut v = vec![0.0; dim];
        for i in 0..dim {
            u[i] = i as f64 * 0.1;
            v[i] = (dim - i) as f64 * 0.15;
        }

        let u_proj = project_vector(&u, &matrix);
        let v_proj = project_vector(&v, &matrix);

        let dist_orig = (u
            .iter()
            .zip(v.iter())
            .map(|(ui, vi)| (ui - vi) * (ui - vi))
            .sum::<f64>())
        .sqrt();
        let dist_proj = (u_proj
            .iter()
            .zip(v_proj.iter())
            .map(|(ui, vi)| (ui - vi) * (ui - vi))
            .sum::<f64>())
        .sqrt();

        assert!((dist_orig - dist_proj).abs() < 1e-9);

        let po_dim = 32;
        let po_seed = b"poincare_seed_rust";
        let po_matrix = generate_lorentz_matrix(po_dim + 1, po_seed);

        let mut p_u = vec![0.0; po_dim];
        let mut p_v = vec![0.0; po_dim];
        for i in 0..po_dim {
            p_u[i] = i as f64 * 0.005;
            p_v[i] = (po_dim - i) as f64 * 0.005;
        }

        let u_lorentz = poincare_to_lorentz_f64(&p_u);
        let v_lorentz = poincare_to_lorentz_f64(&p_v);

        let u_lorentz_proj = project_vector(&u_lorentz, &po_matrix);
        let v_lorentz_proj = project_vector(&v_lorentz, &po_matrix);

        let u_proj_po = lorentz_to_poincare_f64(&u_lorentz_proj);
        let v_proj_po = lorentz_to_poincare_f64(&v_lorentz_proj);

        let poincare_dist = |x: &[f64], y: &[f64]| -> f64 {
            let x_sq: f64 = x.iter().map(|vi| vi * vi).sum();
            let y_sq: f64 = y.iter().map(|vi| vi * vi).sum();
            let diff_sq: f64 = x
                .iter()
                .zip(y.iter())
                .map(|(xi, yi)| (xi - yi) * (xi - yi))
                .sum();
            let val = 1.0 + 2.0 * diff_sq / ((1.0 - x_sq) * (1.0 - y_sq));
            val.acosh()
        };

        let d_orig = poincare_dist(&p_u, &p_v);
        let d_proj = poincare_dist(&u_proj_po, &v_proj_po);

        assert!((d_orig - d_proj).abs() < 1e-9);
    }
}
