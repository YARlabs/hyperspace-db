fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
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
    Ok(x.iter()
        .zip(y.iter())
        .map(|(xi, yi)| (num_left * xi + num_right * yi) / den)
        .collect())
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
    let v_norm = norm_sq(v).sqrt();
    if v_norm < 1e-15 {
        return Ok(x.to_vec());
    }
    let lambda_x = 2.0 / (1.0 - c * x2).max(1e-15);
    let scale = (c.sqrt() * lambda_x * v_norm / 2.0).tanh() / (c.sqrt() * v_norm);
    let step: Vec<f64> = v.iter().map(|vi| scale * vi).collect();
    mobius_add(x, &step, c)
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
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================

/// Calculates the spatial entropy (dispersion) of a `candidate` vector relative to its `neighbors`.
/// Used to track LLM hallucinations (Task 2.3.1).
/// Returns a value in [0, 1) where values approaching 1 imply high chaos (hallucination).
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
pub fn koopman_extrapolate(past: &[f64], current: &[f64], steps: f64, c: f64) -> Result<Vec<f64>, String> {
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
pub fn context_resonance(thought: &[f64], global_context: &[f64], resonance_factor: f64, c: f64) -> Result<Vec<f64>, String> {
    let pull_dir = log_map(thought, global_context, c)?;
    let factor = resonance_factor.clamp(0.0, 1.0);
    let applied_pull: Vec<f64> = pull_dir.iter().map(|v| v * factor).collect();
    exp_map(thought, &applied_pull, c)
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
        let neighbors = vec![
            vec![0.11, 0.1],
            vec![0.1, 0.12],
            vec![0.09, 0.09],
        ];
        let entropy = local_entropy(&candidate, &neighbors, 1.0).unwrap();
        // Since neighbors are close, entropy should be low (close to 0)
        assert!(entropy < 0.1);

        // Lyapunov Convergence
        let trajectory_converging = vec![
            vec![0.5, 0.5],
            vec![0.3, 0.3],
            vec![0.1, 0.1],
            vec![0.05, 0.05]
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
}
