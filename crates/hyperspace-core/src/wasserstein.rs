pub struct WassersteinDistance;

impl WassersteinDistance {
    pub fn compute(a: &[f64], b: &[f64]) -> f64 {
        let mut sum_a = 0.0;
        let mut sum_b = 0.0;
        for &v in a {
            if v > 0.0 {
                sum_a += v;
            }
        }
        for &v in b {
            if v > 0.0 {
                sum_b += v;
            }
        }

        if sum_a <= 1e-9 || sum_b <= 1e-9 {
            return f64::MAX;
        }

        let mut cdf_a = 0.0;
        let mut cdf_b = 0.0;
        let mut dist = 0.0;

        for i in 0..a.len() {
            let val_a = if a[i] > 0.0 { a[i] / sum_a } else { 0.0 };
            let val_b = if b[i] > 0.0 { b[i] / sum_b } else { 0.0 };

            cdf_a += val_a;
            cdf_b += val_b;
            dist += (cdf_a - cdf_b).abs();
        }

        dist
    }
}
