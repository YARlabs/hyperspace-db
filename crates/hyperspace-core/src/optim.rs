/// Entity responsible for performing Riemannian gradient descent steps
/// and background memory reconsolidation directly in the database.
pub struct MemoryReconsolidator {
    pub learning_rate: f64,
}

impl MemoryReconsolidator {
    pub fn new(lr: f64) -> Self {
        Self { learning_rate: lr }
    }

    /// In-place variant of `step_towards_poincare` that mutates `source` without any heap allocations.
    pub fn step_towards_poincare_inplace(&self, source: &mut [f64], target: &[f64]) {
        let mut norm_sq = 0.0;
        let len = source.len().min(target.len());

        // Euclidean direction as unscaled tangent proxy
        for i in 0..len {
            let step = target[i] - source[i];
            source[i] += self.learning_rate * step;
            norm_sq += source[i] * source[i];
        }

        // Project back to ensure it stays precisely within the Poincare manifold limits
        if norm_sq >= 1.0 {
            let scale = 0.999_999 / norm_sq.sqrt();
            for item in &mut source[..len] {
                *item *= scale;
            }
        }
    }

    /// Pulls the source vector slightly closer to the target vector using Riemannian SGD.
    /// This represents the "Flow Matching" micro-shift in AI sleep mode.
    pub fn step_towards_poincare(&self, source: &[f64], target: &[f64]) -> Vec<f64> {
        let mut new_pos = source.to_vec();
        self.step_towards_poincare_inplace(&mut new_pos, target);
        new_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poincare_dist(x: &[f64], y: &[f64]) -> f64 {
        let mut num = 0.0;
        let mut nx2 = 0.0;
        let mut ny2 = 0.0;
        for i in 0..x.len() {
            let diff = x[i] - y[i];
            num += diff * diff;
            nx2 += x[i] * x[i];
            ny2 += y[i] * y[i];
        }
        (1.0 + 2.0 * num / ((1.0 - nx2) * (1.0 - ny2))).acosh()
    }

    #[test]
    fn test_memory_reconsolidation_risk_oracle() {
        let recon = MemoryReconsolidator::new(0.01);
        let target = vec![0.5, 0.5]; // "Risk" feature vector

        let mut companies = Vec::new();
        for i in 0..400 {
            companies.push(vec![0.1 * (i as f64 / 400.0), -0.1 * (i as f64 / 400.0)]);
        }

        let initial_dists: Vec<f64> = companies
            .iter()
            .map(|c| poincare_dist(c, &target))
            .collect();

        let start = std::time::Instant::now();
        for comp in &mut companies {
            *comp = recon.step_towards_poincare(comp, &target);
        }
        let elapsed = start.elapsed();

        // Goal: successfully redistributes 400 vectors in < 100ms
        assert!(
            elapsed.as_millis() < 100,
            "Should be < 100ms, got {}ms",
            elapsed.as_millis()
        );

        // Ensure they actually moved closer
        let final_dists: Vec<f64> = companies
            .iter()
            .map(|c| poincare_dist(c, &target))
            .collect();

        for i in 0..400 {
            assert!(
                final_dists[i] < initial_dists[i],
                "Vector {i} did not move closer!"
            );
        }
    }
}
