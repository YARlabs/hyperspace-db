use super::*;

#[test]
fn test_euclidean_distance() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    // diffs: -3, -3, -3. sq: 9, 9, 9. sum: 27.
    let dist = EuclideanMetric::distance(&a, &b);
    assert!((dist - 27.0).abs() < f64::EPSILON);
}

#[test]
fn test_cosine_distance() {
    // Vectors must be normalized for CosineMetric usually, but logic is sum((a-b)^2)
    // If a=[1,0], b=[0,1]. diff=[1, -1]. sq=[1, 1]. sum=2.
    // 2*(1 - cos(90)) = 2*(1-0) = 2. Correct.
    let a = [1.0, 0.0];
    let b = [0.0, 1.0];
    let dist = CosineMetric::distance(&a, &b);
    assert!((dist - 2.0).abs() < f64::EPSILON);

    // a=[1,0], b=[1,0]. diff=[0,0]. sum=0.
    let dist_same = CosineMetric::distance(&a, &a);
    assert!(dist_same.abs() < f64::EPSILON);

    // a=[1,0], b=[-1,0]. diff=[2,0]. sq=[4,0]. sum=4.
    // 2(1 - cos(180)) = 2(1 - (-1)) = 4. Correct.
    let c = [-1.0, 0.0];
    let dist_opp = CosineMetric::distance(&a, &c);
    assert!((dist_opp - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_poincare_validation() {
    let v_valid = [0.1, 0.2];
    assert!(PoincareMetric::validate(&v_valid).is_ok());

    let v_invalid = [1.0, 0.0]; // Norm=1. Boundary.
    assert!(PoincareMetric::validate(&v_invalid).is_err());

    let v_invalid2 = [0.8, 0.8]; // Norm sq = 0.64+0.64 = 1.28
    assert!(PoincareMetric::validate(&v_invalid2).is_err());
}

#[test]
fn test_lorentz_distance_and_validation() {
    // x0 = (1, 0, 0) and x1 = (cosh(r), sinh(r), 0) on unit hyperboloid.
    // Their Lorentz distance should be exactly r.
    let r = 1.5_f64;
    let x0 = [1.0, 0.0, 0.0];
    let x1 = [r.cosh(), r.sinh(), 0.0];

    assert!(LorentzMetric::validate(&x0).is_ok());
    assert!(LorentzMetric::validate(&x1).is_ok());

    let dist = LorentzMetric::distance(&x0, &x1);
    assert!((dist - r).abs() < 1e-9);

    let invalid = [-1.0, 0.0, 0.0]; // lower sheet
    assert!(LorentzMetric::validate(&invalid).is_err());
}

// ── Lorentz Scalar Quantization (SQ8) Tests ─────────────────────────────────

#[test]
fn test_lorentz_quantization_roundtrip_origin() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Origin on the hyperboloid: (1, 0, 0)
    let origin = HyperVector::<3> {
        coords: [1.0, 0.0, 0.0],
        alpha: 0.0, // unused for Lorentz
    };
    let q = QuantizedHyperVector::from_float_lorentz(&origin);

    // alpha stores the scale factor (max |coord| = 1.0)
    assert!((f64::from(q.alpha) - 1.0).abs() < 1e-5);
    // coords[0] should map to 127 (1.0 / 1.0 * 127 = 127)
    assert_eq!(q.coords[0], 127);
    assert_eq!(q.coords[1], 0);
    assert_eq!(q.coords[2], 0);
}

#[test]
fn test_lorentz_quantization_known_point() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Point at geodesic distance r=1.0: (cosh(1), sinh(1), 0)
    let r = 1.0_f64;
    let v = HyperVector::<3> {
        coords: [r.cosh(), r.sinh(), 0.0],
        alpha: 0.0,
    };
    let q = QuantizedHyperVector::from_float_lorentz(&v);

    // Scale should be cosh(1) ~ 1.5431 (the largest absolute coordinate)
    let expected_scale = r.cosh();
    assert!((f64::from(q.alpha) - expected_scale).abs() < 1e-3);

    // coords[0] should be 127 (cosh(1)/cosh(1) * 127 = 127)
    assert_eq!(q.coords[0], 127);
    // coords[1] should be round(sinh(1)/cosh(1) * 127) = round(tanh(1) * 127) ~ round(96.5) = 97
    let expected_q1 = (r.tanh() * 127.0).round() as i8;
    assert_eq!(q.coords[1], expected_q1);
}

#[test]
fn test_lorentz_quantized_distance_accuracy_near() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Two points at small geodesic distance on H^2
    let r = 0.5_f64;
    let a_coords = [1.0, 0.0, 0.0];
    let b_coords = [r.cosh(), r.sinh(), 0.0];

    let a = HyperVector::<3> {
        coords: a_coords,
        alpha: 0.0,
    };
    let b = HyperVector::<3> {
        coords: b_coords,
        alpha: 0.0,
    };

    let exact = LorentzMetric::distance(&a_coords, &b_coords);
    let q_a = QuantizedHyperVector::from_float_lorentz(&a);
    let approx = LorentzMetric::distance_quantized(&q_a, &b);

    // For nearby points, quantization error should be bounded
    let relative_error = (approx - exact).abs() / exact;
    assert!(
        relative_error < 0.10,
        "Near-distance relative error {relative_error:.4} exceeds 10% (exact={exact:.6}, approx={approx:.6})"
    );
}

#[test]
fn test_lorentz_quantized_distance_accuracy_far() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Points at moderate geodesic distance
    let r = 3.0_f64;
    let a_coords = [1.0, 0.0, 0.0];
    let b_coords = [r.cosh(), r.sinh(), 0.0];

    let a = HyperVector::<3> {
        coords: a_coords,
        alpha: 0.0,
    };
    let b = HyperVector::<3> {
        coords: b_coords,
        alpha: 0.0,
    };

    let exact = LorentzMetric::distance(&a_coords, &b_coords);
    let q_a = QuantizedHyperVector::from_float_lorentz(&a);
    let approx = LorentzMetric::distance_quantized(&q_a, &b);

    // For farther points the absolute error grows but ranking is preserved
    let relative_error = (approx - exact).abs() / exact;
    assert!(
        relative_error < 0.15,
        "Far-distance relative error {relative_error:.4} exceeds 15% (exact={exact:.6}, approx={approx:.6})"
    );
}

#[test]
fn test_lorentz_quantized_distance_preserves_ordering() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Three points at increasing geodesic distances from origin
    let origin = HyperVector::<3> {
        coords: [1.0, 0.0, 0.0],
        alpha: 0.0,
    };
    let q_origin = QuantizedHyperVector::from_float_lorentz(&origin);

    let r1 = 0.5_f64;
    let p1 = HyperVector::<3> {
        coords: [r1.cosh(), r1.sinh(), 0.0],
        alpha: 0.0,
    };

    let r2 = 1.5_f64;
    let p2 = HyperVector::<3> {
        coords: [r2.cosh(), r2.sinh(), 0.0],
        alpha: 0.0,
    };

    let r3 = 3.0_f64;
    let p3 = HyperVector::<3> {
        coords: [r3.cosh(), r3.sinh(), 0.0],
        alpha: 0.0,
    };

    let d1 = LorentzMetric::distance_quantized(&q_origin, &p1);
    let d2 = LorentzMetric::distance_quantized(&q_origin, &p2);
    let d3 = LorentzMetric::distance_quantized(&q_origin, &p3);

    // Distance ordering must be preserved: d1 < d2 < d3
    assert!(d1 < d2, "Ordering violated: d1={d1:.6} >= d2={d2:.6}");
    assert!(d2 < d3, "Ordering violated: d2={d2:.6} >= d3={d3:.6}");
}

#[test]
fn test_lorentz_quantized_self_distance_near_zero() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Quantized self-distance should be near zero
    let v = HyperVector::<3> {
        coords: [1.0, 0.0, 0.0],
        alpha: 0.0,
    };
    let q = QuantizedHyperVector::from_float_lorentz(&v);

    let self_dist = LorentzMetric::distance_quantized(&q, &v);
    assert!(
        self_dist < 0.05,
        "Self-distance should be ~0, got {self_dist:.6}"
    );
}

#[test]
fn test_lorentz_quantized_high_dim() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // 8-dimensional hyperboloid point: t = cosh(r), spatial = sinh(r) * unit_direction
    let r = 2.0_f64;
    let spatial_norm = r.sinh();
    let dim_spatial = 7;
    let component = spatial_norm / (dim_spatial as f64).sqrt();

    let mut a_coords = [0.0_f64; 8];
    a_coords[0] = r.cosh();
    for i in 1..8 {
        a_coords[i] = component;
    }

    let mut b_coords = [0.0_f64; 8];
    b_coords[0] = 1.0; // origin

    let a = HyperVector::<8> {
        coords: a_coords,
        alpha: 0.0,
    };
    let b = HyperVector::<8> {
        coords: b_coords,
        alpha: 0.0,
    };
    let q_a = QuantizedHyperVector::from_float_lorentz(&a);

    let exact = LorentzMetric::distance(&a_coords, &b_coords);
    let approx = LorentzMetric::distance_quantized(&q_a, &b);

    let relative_error = (approx - exact).abs() / exact;
    assert!(
        relative_error < 0.15,
        "8D relative error {relative_error:.4} exceeds 15% (exact={exact:.6}, approx={approx:.6})"
    );
}

#[test]
#[should_panic(expected = "Binary quantization is not supported for the Lorentz model")]
fn test_lorentz_binary_still_panics() {
    use crate::vector::{BinaryHyperVector, HyperVector};
    let v = HyperVector::<3> {
        coords: [1.0, 0.0, 0.0],
        alpha: 0.0,
    };
    let b = BinaryHyperVector::from_float(&v);
    let _ = LorentzMetric::distance_binary(&b, &v);
}
