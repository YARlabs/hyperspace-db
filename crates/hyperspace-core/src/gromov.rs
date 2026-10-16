#![allow(clippy::vec_init_then_push)]
#![allow(clippy::similar_names)]
use rand::Rng;

/// Computes Gromov's delta-hyperbolicity of a dataset.
/// A metric space is delta-hyperbolic if for any 4 points x,y,u,v:
/// d(x,y) + d(u,v) <= max(d(x,u)+d(y,v), d(x,v)+d(y,u)) + 2*delta
/// So we find the max over a random subset of 4-tuples.
/// We sample randomly to avoid O(N^4) complexity.
#[must_use]
pub fn analyze_delta_hyperbolicity(vectors: &[Vec<f64>], num_samples: usize) -> (f64, String) {
    if vectors.len() < 4 {
        return (0.0, "euclidean".to_string());
    }

    let mut max_delta = 0.0;
    let mut rng = rand::thread_rng();

    for _ in 0..num_samples {
        let i = rng.gen_range(0..vectors.len());
        let j = rng.gen_range(0..vectors.len());
        let k = rng.gen_range(0..vectors.len());
        let l = rng.gen_range(0..vectors.len());

        if i == j || i == k || i == l || j == k || j == l || k == l {
            continue;
        }

        let d_ij = l2_dist(&vectors[i], &vectors[j]);
        let d_kl = l2_dist(&vectors[k], &vectors[l]);

        let d_ik = l2_dist(&vectors[i], &vectors[k]);
        let d_jl = l2_dist(&vectors[j], &vectors[l]);

        let d_il = l2_dist(&vectors[i], &vectors[l]);
        let d_jk = l2_dist(&vectors[j], &vectors[k]);

        let s1 = d_ij + d_kl;
        let s2 = d_ik + d_jl;
        let s3 = d_il + d_jk;

        let (first, second) = if s1 >= s2 {
            if s1 >= s3 {
                (s1, s2.max(s3))
            } else {
                (s3, s1)
            }
        } else if s2 >= s3 {
            (s2, s1.max(s3))
        } else {
            (s3, s2)
        };

        let delta = (first - second) * 0.5;
        if delta > max_delta {
            max_delta = delta;
        }
    }

    let mut is_normalized = true;
    for v in vectors.iter().take(20) {
        let n = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if (n - 1.0).abs() > 1e-2 {
            is_normalized = false;
            break;
        }
    }

    // In Gromov 4-point condition, if the delta is very small compared to the space
    // it exhibits tree-like geometry. A low delta relative to average distances
    // indicates hyperbolic structure.
    let recommendation = if max_delta < 0.15 {
        "lorentz".to_string() // Extremely hyperbolic (better scaling)
    } else if max_delta < 0.30 {
        "poincare".to_string() // Mildly hyperbolic
    } else if is_normalized {
        "cosine".to_string() // Dense vectors on a hypersphere
    } else {
        "l2".to_string() // General Euclidean
    };

    (max_delta, recommendation)
}

fn l2_dist(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let (chunks_a, rem_a) = a[..n].as_chunks::<4>();
    let (chunks_b, rem_b) = b[..n].as_chunks::<4>();
    let mut acc = [0.0; 4];
    for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
        for l in 0..4 {
            let diff = ca[l] - cb[l];
            acc[l] += diff * diff;
        }
    }
    let mut sum = acc[0] + acc[1] + acc[2] + acc[3];
    for (&x, &y) in rem_a.iter().zip(rem_b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_hyperbolicity() {
        let mut vecs = vec![];
        vecs.push(vec![0.0, 0.0]); // root
        vecs.push(vec![1.0, 0.0]); // child
        vecs.push(vec![-1.0, 0.0]); // child
        vecs.push(vec![2.0, 0.0]); // grandchild
        vecs.push(vec![-2.0, 0.0]); // grandchild

        let (delta, rec) = analyze_delta_hyperbolicity(&vecs, 10);
        assert!(
            delta < 0.2,
            "Delta should be near zero for a line/tree. Got: {delta}"
        );
        assert_eq!(rec, "lorentz");
    }
}
