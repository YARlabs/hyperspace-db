use super::*;

#[test]
fn test_vector_validation() {
    let nan_vec = [f64::NAN, 0.0];
    let inf_vec = [f64::INFINITY, 0.0];
    let valid_vec = [0.1, 0.2];

    assert!(EuclideanMetric::validate(&nan_vec).is_err());
    assert!(EuclideanMetric::validate(&inf_vec).is_err());
    assert!(EuclideanMetric::validate(&valid_vec).is_ok());

    assert!(PoincareMetric::validate(&nan_vec).is_err());
    assert!(PoincareMetric::validate(&inf_vec).is_err());
    assert!(PoincareMetric::validate(&valid_vec).is_ok());
}

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
    let origin = HyperVector {
        coords: vec![1.0, 0.0, 0.0],
        alpha: 0.0, // unused for Lorentz
    };
    let q = QuantizedHyperVector::from_float_lorentz(&origin);

    // Spatial coordinates are all zero -> alpha is 0.0, time is reconstructed as sqrt(1 + 0) = 1.0
    assert!(f64::from(q.alpha).abs() < 1e-5);
    assert_eq!(q.coords[0], 127);
    assert_eq!(q.coords[1], 0);
    assert_eq!(q.coords[2], 0);
}

#[test]
fn test_lorentz_quantization_known_point() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Point at geodesic distance r=1.0: (cosh(1), sinh(1), 0)
    let r = 1.0_f64;
    let v = HyperVector {
        coords: vec![r.cosh(), r.sinh(), 0.0],
        alpha: 0.0,
    };
    let q = QuantizedHyperVector::from_float_lorentz(&v);

    // Spatial scale should be sinh(1) ~ 1.1752 (the largest absolute spatial coordinate)
    let expected_scale = r.sinh();
    assert!((f64::from(q.alpha) - expected_scale).abs() < 1e-3);

    // coords[0] is sentinel (127) since t is reconstructed on the hyperboloid as sqrt(1 + ||x||^2)
    assert_eq!(q.coords[0], 127);
    // coords[1] uses the full [-127, 127] dynamic range: round(sinh(1)/sinh(1) * 127) = 127
    assert_eq!(q.coords[1], 127);
    assert_eq!(q.coords[2], 0);
}

#[test]
fn test_lorentz_quantized_distance_accuracy_near() {
    use crate::vector::{HyperVector, QuantizedHyperVector};

    // Two points at small geodesic distance on H^2
    let r = 0.5_f64;
    let a_coords = [1.0, 0.0, 0.0];
    let b_coords = [r.cosh(), r.sinh(), 0.0];

    let a = HyperVector {
        coords: a_coords.to_vec(),
        alpha: 0.0,
    };
    let b = HyperVector {
        coords: b_coords.to_vec(),
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

    let a = HyperVector {
        coords: a_coords.to_vec(),
        alpha: 0.0,
    };
    let b = HyperVector {
        coords: b_coords.to_vec(),
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
    let origin = HyperVector {
        coords: vec![1.0, 0.0, 0.0],
        alpha: 0.0,
    };
    let q_origin = QuantizedHyperVector::from_float_lorentz(&origin);

    let r1 = 0.5_f64;
    let p1 = HyperVector {
        coords: vec![r1.cosh(), r1.sinh(), 0.0],
        alpha: 0.0,
    };

    let r2 = 1.5_f64;
    let p2 = HyperVector {
        coords: vec![r2.cosh(), r2.sinh(), 0.0],
        alpha: 0.0,
    };

    let r3 = 3.0_f64;
    let p3 = HyperVector {
        coords: vec![r3.cosh(), r3.sinh(), 0.0],
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
    let v = HyperVector {
        coords: vec![1.0, 0.0, 0.0],
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

    let a = HyperVector {
        coords: a_coords.to_vec(),
        alpha: 0.0,
    };
    let b = HyperVector {
        coords: b_coords.to_vec(),
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
    let v = HyperVector {
        coords: vec![1.0, 0.0, 0.0],
        alpha: 0.0,
    };
    let b = BinaryHyperVector::from_float(&v);
    let _ = LorentzMetric::distance_binary(&b, &v);
}

// ── Lorentz validation tolerance for near-hyperboloid vectors ────────────────

#[test]
fn test_lorentz_validate_accepts_small_numerical_drift() {
    // Externally-produced embeddings (and quantization round-trips) can land
    // slightly off the ideal hyperboloid: -t^2 + |x|^2 == -1 only up to a small
    // numerical drift. `distance` already tolerates this via the acosh domain
    // clamp `(-inner_prod).max(1.0 + 1e-12)`, so a relaxed `validate` tolerance
    // must accept vectors the distance math handles fine.
    //
    // Test the tolerance directly (deterministic, independent of the
    // HS_LORENTZ_VALIDATE_TOL environment variable).
    let tol = 0.1_f64;
    let spatial = [0.30_f64, 0.20, 0.10];
    let spatial_sq: f64 = spatial.iter().map(|v| v * v).sum();
    for &drift in &[0.0_f64, 0.01, 0.05, 0.09] {
        // -t^2 + spatial_sq = -1 + drift  =>  t = sqrt(spatial_sq + 1 - drift)
        let t = (spatial_sq + 1.0 - drift).sqrt();
        let v = [t, spatial[0], spatial[1], spatial[2]];
        assert!(
            LorentzMetric::validate_with_tolerance(&v, tol).is_ok(),
            "drift {drift} within tolerance {tol} should validate"
        );
        // distance to itself must be finite (acosh domain is respected).
        let d = LorentzMetric::distance(&v, &v);
        assert!(
            d.is_finite(),
            "self-distance must be finite for drift {drift}"
        );
    }
}

#[test]
fn test_lorentz_validate_default_is_strict() {
    // With the default strict tolerance, a vector even slightly off the
    // hyperboloid is rejected — this preserves historical behaviour.
    let spatial = [0.30_f64, 0.20, 0.10];
    let spatial_sq: f64 = spatial.iter().map(|v| v * v).sum();
    let t = (spatial_sq + 1.0 - 0.05).sqrt(); // drift 0.05, well past 1e-6
    let v = [t, spatial[0], spatial[1], spatial[2]];
    assert!(
        LorentzMetric::validate_with_tolerance(&v, LORENTZ_VALIDATE_DEFAULT_TOL).is_err(),
        "drift 0.05 must be rejected at the strict default tolerance"
    );
    // The same vector passes once the tolerance is relaxed.
    assert!(LorentzMetric::validate_with_tolerance(&v, 0.1).is_ok());
}

#[test]
fn test_lorentz_validate_still_rejects_garbage() {
    // A guard against gross violations must remain regardless of tolerance:
    // a vector far off the hyperboloid, non-finite values, and the lower sheet
    // are all invalid.
    let far_off = [1.0_f64, 5.0, 5.0, 5.0]; // -1 + 74 == 73, way past tolerance
    assert!(LorentzMetric::validate_with_tolerance(&far_off, 0.1).is_err());

    let nan = [f64::NAN, 0.0, 0.0, 0.0];
    assert!(LorentzMetric::validate_with_tolerance(&nan, 0.1).is_err());

    let lower_sheet = [-2.0_f64, 0.0, 0.0, 0.0];
    assert!(LorentzMetric::validate_with_tolerance(&lower_sheet, 0.1).is_err());
}

#[test]
fn request_ef_search_defaults_and_clamps() {
    assert_eq!(
        crate::request_ef_search(None),
        0,
        "unset = collection default"
    );
    assert_eq!(
        crate::request_ef_search(Some(0)),
        0,
        "0 = collection default"
    );
    assert_eq!(crate::request_ef_search(Some(250)), 250);
    let max = crate::max_request_ef_search();
    assert!(max >= 1024);
    assert_eq!(crate::request_ef_search(Some(usize::MAX)), max, "DoS guard");
}

// Exact sentinel / zero comparisons are intended in these kernel tests.
#[allow(clippy::float_cmp)]
mod bit_plane_lut {
    use crate::vector::{BinaryHyperVector, BitPlaneLut, HyperVector, TernaryHyperVector};
    use proptest::prelude::*;

    fn vec_strategy() -> impl Strategy<Value = (Vec<f64>, Vec<f64>)> {
        // Odd dimensions exercise the partial nibble / byte tails.
        (1usize..300).prop_flat_map(|d| {
            (
                proptest::collection::vec(-2.0f64..2.0, d),
                proptest::collection::vec(-2.0f64..2.0, d),
            )
        })
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-4 * (1.0 + a.abs().max(b.abs()))
    }

    proptest! {
        #[test]
        fn binary_lut_matches_reference((v, q) in vec_strategy()) {
            let code = BinaryHyperVector::from_float(&HyperVector::new_unchecked(v)).as_bytes();
            let query = HyperVector::new_unchecked(q.clone());
            let lut = BitPlaneLut::new(&q);
            let reference = BinaryHyperVector::adc_distance_bytes(&code, &query);
            let fast = BinaryHyperVector::adc_distance_lut(&code, &lut);
            prop_assert!(close(reference, fast), "{reference} vs {fast}");
        }

        #[test]
        fn ternary_lut_matches_reference((v, q) in vec_strategy()) {
            let d = v.len();
            let code = TernaryHyperVector::from_float(&HyperVector::new_unchecked(v), 0).as_bytes();
            let query = HyperVector::new_unchecked(q.clone());
            let lut = BitPlaneLut::new(&q);
            let l2_ref = TernaryHyperVector::distance_bytes_l2_sq(&code, &query, d, 0);
            let l2 = TernaryHyperVector::distance_l2_sq_lut(&code, &lut, d);
            prop_assert!(close(l2_ref, l2), "l2 {l2_ref} vs {l2}");
            let cos_ref = TernaryHyperVector::distance_bytes_cosine(&code, &query, d, 0);
            let cos = TernaryHyperVector::distance_cosine_lut(&code, &lut, d);
            prop_assert!(close(cos_ref, cos), "cos {cos_ref} vs {cos}");
        }
    }

    #[test]
    fn masked_sum_is_exact_for_known_masks() {
        let q: Vec<f64> = (1..=12).map(f64::from).collect(); // 12 dims, odd nibble count
        let lut = BitPlaneLut::new(&q);
        assert!((lut.masked_sum(&[0xFF, 0x0F]) - 78.0).abs() < 1e-6);
        assert!((lut.masked_sum(&[0b0000_0101, 0]) - 4.0).abs() < 1e-6);
        assert!((lut.masked_sum(&[0, 0b1000]) - 12.0).abs() < 1e-6);
        assert_eq!(lut.masked_sum(&[]), 0.0);
        assert!((lut.q_sum - 78.0).abs() < 1e-9);
    }

    #[test]
    fn truncated_codes_do_not_panic() {
        let lut = BitPlaneLut::new(&[1.0; 64]);
        assert_eq!(BinaryHyperVector::adc_distance_lut(&[1, 2], &lut), f64::MAX);
        assert_eq!(
            TernaryHyperVector::distance_l2_sq_lut(&[0; 5], &lut, 64),
            f64::MAX
        );
        assert_eq!(
            TernaryHyperVector::distance_cosine_lut(&[0; 5], &lut, 64),
            2.0
        );
        // Code longer than the query: extra bits map to zero weight.
        let _ = lut.masked_sum(&[0xFF; 64]);
    }
}

/// Kernel micro-benchmark (`cargo test -p hyperspace-core --release -- --ignored --nocapture
/// bit_lut_kernel_speed`): per-bit reference vs nibble table, 1024 dims.
#[test]
#[ignore = "timing benchmark"]
fn bit_lut_kernel_speed() {
    use crate::vector::{BinaryHyperVector, BitPlaneLut, HyperVector, TernaryHyperVector};
    use std::hint::black_box;
    use std::time::Instant;
    let dim = 1024;
    let mk = |s: u64| -> Vec<f64> {
        (0..dim as u64)
            .map(|i| (((s * 7919 + i * 104_729) % 2001) as f64 / 1000.0) - 1.0)
            .collect()
    };
    let codes_b: Vec<Vec<u8>> = (0..256)
        .map(|s| BinaryHyperVector::from_float(&HyperVector::new_unchecked(mk(s))).as_bytes())
        .collect();
    let codes_t: Vec<Vec<u8>> = (0..256)
        .map(|s| TernaryHyperVector::from_float(&HyperVector::new_unchecked(mk(s)), 0).as_bytes())
        .collect();
    let q = HyperVector::new_unchecked(mk(999));
    let lut = BitPlaneLut::new(&q.coords);
    let iters = 400_000usize;
    let time = |f: &dyn Fn(usize) -> f64| {
        let t = Instant::now();
        let mut acc = 0.0;
        for i in 0..iters {
            acc += f(i);
        }
        black_box(acc);
        t.elapsed().as_nanos() as f64 / iters as f64
    };
    let b_ref = time(&|i| BinaryHyperVector::adc_distance_bytes(&codes_b[i & 255], &q));
    let b_lut = time(&|i| BinaryHyperVector::adc_distance_lut(&codes_b[i & 255], &lut));
    let t_ref = time(&|i| TernaryHyperVector::distance_bytes_l2_sq(&codes_t[i & 255], &q, dim, 0));
    let t_lut = time(&|i| TernaryHyperVector::distance_l2_sq_lut(&codes_t[i & 255], &lut, dim));
    let c_ref = time(&|i| TernaryHyperVector::distance_bytes_cosine(&codes_t[i & 255], &q, dim, 0));
    let c_lut = time(&|i| TernaryHyperVector::distance_cosine_lut(&codes_t[i & 255], &lut, dim));
    println!(
        "extreme ADC     : {b_ref:7.1} ns -> {b_lut:6.1} ns  (x{:.1})",
        b_ref / b_lut
    );
    println!(
        "ternary L2      : {t_ref:7.1} ns -> {t_lut:6.1} ns  (x{:.1})",
        t_ref / t_lut
    );
    println!(
        "ternary cosine  : {c_ref:7.1} ns -> {c_lut:6.1} ns  (x{:.1})",
        c_ref / c_lut
    );
}

#[allow(clippy::float_cmp)]
mod scalar_i4_f32_kernel {
    use crate::hybrid::ScalarI4Vector;
    use crate::vector::HyperVector;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn matches_reference(d in 1usize..200, seed in 0u64..1000) {
            let gen = |s: u64| -> Vec<f64> {
                (0..d as u64).map(|i| (((s * 7919 + i * 104_729) % 2001) as f64 / 1000.0) - 1.0).collect()
            };
            let v = gen(seed);
            let q = gen(seed + 1);
            let code = ScalarI4Vector::from_float(&HyperVector::new_unchecked(v), 0).as_bytes();
            let reference = ScalarI4Vector::distance_bytes_l2_sq_mrl(&code, &HyperVector::new_unchecked(q.clone()), d, 0);
            let qn: f64 = q.iter().map(|&x| f64::from(x as f32).powi(2)).sum();
            let (xe, xo) = ScalarI4Vector::deinterleave_query(&q, d);
            let fast = ScalarI4Vector::distance_l2_sq_deinterleaved(&code, &xe, &xo, qn, d);
            prop_assert!((reference - fast).abs() <= 1e-3 * (1.0 + reference), "{reference} vs {fast}");
        }
    }

    #[test]
    fn malformed_codes_are_rejected_without_panicking() {
        let (xe, xo) = ScalarI4Vector::deinterleave_query(&[0.5; 32], 32);
        assert_eq!(
            ScalarI4Vector::distance_l2_sq_deinterleaved(&[1, 2], &xe, &xo, 8.0, 32),
            f64::MAX
        );
        let mut code = 2u32.to_le_bytes().to_vec();
        code.extend_from_slice(&[0; 8]); // scales + packed bytes missing
        assert_eq!(
            ScalarI4Vector::distance_l2_sq_deinterleaved(&code, &xe, &xo, 8.0, 32),
            f64::MAX
        );
        // Query shorter than the code's blocks.
        assert_eq!(
            ScalarI4Vector::distance_l2_sq_deinterleaved(&code, &xe[..4], &xo, 8.0, 32),
            f64::MAX
        );
    }
}

#[test]
#[ignore = "timing benchmark"]
fn scalar_i4_kernel_speed() {
    use crate::hybrid::ScalarI4Vector;
    use crate::vector::HyperVector;
    use std::hint::black_box;
    use std::time::Instant;
    let dim = 1024;
    let mk = |s: u64| -> Vec<f64> {
        (0..dim as u64)
            .map(|i| (((s * 7919 + i * 104_729) % 2001) as f64 / 1000.0) - 1.0)
            .collect()
    };
    let codes: Vec<Vec<u8>> = (0..256)
        .map(|s| ScalarI4Vector::from_float(&HyperVector::new_unchecked(mk(s)), 0).as_bytes())
        .collect();
    let q = HyperVector::new_unchecked(mk(999));
    let qn: f64 = q.coords.iter().map(|&x| x * x).sum();
    let iters = 400_000usize;
    let time = |f: &dyn Fn(usize) -> f64| {
        let t = Instant::now();
        let mut acc = 0.0;
        for i in 0..iters {
            acc += f(i);
        }
        black_box(acc);
        t.elapsed().as_nanos() as f64 / iters as f64
    };
    let r = time(&|i| ScalarI4Vector::distance_bytes_l2_sq_mrl(&codes[i & 255], &q, dim, 0));
    let (xe, xo) = ScalarI4Vector::deinterleave_query(&q.coords, dim);
    let g =
        time(&|i| ScalarI4Vector::distance_l2_sq_deinterleaved(&codes[i & 255], &xe, &xo, qn, dim));
    println!(
        "medium_plus L2  : {r:7.1} ns -> {g:6.1} ns  (x{:.1})",
        r / g
    );
}

#[cfg(target_arch = "aarch64")]
#[test]
fn scalar_i4_neon_matches_scalar_block_sums() {
    use crate::hybrid::ScalarI4Vector;
    use crate::vector::HyperVector;
    for d in [16usize, 17, 100, 1024] {
        let v: Vec<f64> = (0..d)
            .map(|i| ((i * 37 % 101) as f64 / 50.0) - 1.0)
            .collect();
        let q: Vec<f64> = (0..d)
            .map(|i| ((i * 53 % 97) as f64 / 48.0) - 1.0)
            .collect();
        let code = ScalarI4Vector::from_float(&HyperVector::new_unchecked(v), 0).as_bytes();
        let (xe, xo) = ScalarI4Vector::deinterleave_query(&q, d);
        let qn: f64 = q.iter().map(|&x| f64::from(x as f32).powi(2)).sum();
        let fast = ScalarI4Vector::distance_l2_sq_deinterleaved(&code, &xe, &xo, qn, d);
        let reference =
            ScalarI4Vector::distance_bytes_l2_sq_mrl(&code, &HyperVector::new_unchecked(q), d, 0);
        assert!(
            (fast - reference).abs() <= 1e-3 * (1.0 + reference),
            "d={d}: {fast} vs {reference}"
        );

        // NEON and portable block sums agree.
        let blocks = d.div_ceil(16);
        let sc = u32::from_le_bytes(code[..4].try_into().unwrap()) as usize;
        let scales = &code[4..4 + sc * 4];
        let packed = &code[4 + sc * 4..];
        let (c1, s1) = crate::hybrid::i4_block_sums_scalar(scales, &packed[..blocks * 8], &xe, &xo);
        // SAFETY: slice lengths cover `blocks` by construction.
        let (c2, s2) =
            unsafe { crate::hybrid::i4_block_sums_neon(scales, packed, &xe, &xo, blocks) };
        assert!((c1 - c2).abs() <= 1e-3 * (1.0 + c1.abs()));
        assert!((s1 - s2).abs() <= 1e-3 * (1.0 + s1));
    }
}

mod sq8_int_kernels {
    use crate::simd::{i8_dot_and_self, i8_dot_and_self_scalar, sq8_scaled_l2_int, Sq8Query};
    use crate::vector::{HyperVector, QuantizedHyperVector};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn int_dot_paths_agree(v in proptest::collection::vec((-127i8..=127, -127i8..=127), 0..300)) {
            let a: Vec<i8> = v.iter().map(|p| p.0).collect();
            let b: Vec<i8> = v.iter().map(|p| p.1).collect();
            let r = i8_dot_and_self_scalar(&a, &b);
            prop_assert_eq!(i8_dot_and_self(&a, &b), r);
            #[cfg(target_arch = "aarch64")]
            {
                // SAFETY: NEON is mandatory on aarch64; dotprod is checked below.
                prop_assert_eq!(unsafe { crate::simd::i8_dot_and_self_neon(&a, &b) }, r);
                if std::arch::is_aarch64_feature_detected!("dotprod") {
                    prop_assert_eq!(unsafe { crate::simd::i8_dot_and_self_sdot(&a, &b) }, r);
                }
            }
        }

        #[test]
        fn scaled_sq8_distance_tracks_the_exact_distance(d in 8usize..600, seed in 0u64..500) {
            let gen = |s: u64| -> Vec<f64> {
                let v: Vec<f64> = (0..d as u64).map(|i| (((s * 7919 + i * 104_729) % 2001) as f64 / 1000.0) - 1.0).collect();
                let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
                v.into_iter().map(|x| x / n).collect()
            };
            let x = gen(seed);
            let q = gen(seed + 1);
            let code = QuantizedHyperVector::from_float_scaled(&HyperVector { coords: x.clone(), alpha: 1.0 });
            let exact: f64 = x.iter().zip(&q).map(|(a, b)| (a - b) * (a - b)).sum();
            let qf: Vec<f32> = q.iter().map(|&v| v as f32).collect();
            let approx = f64::from(sq8_scaled_l2_int(
                &code.coords.iter().map(|&c| c.cast_unsigned()).collect::<Vec<u8>>(),
                code.alpha,
                &Sq8Query::new(&qf),
            ));
            // 8-bit codes on both sides of unit vectors (distance range 0..4): the absolute
            // error stays within 0.5% of the range.
            prop_assert!((approx - exact).abs() <= 0.02, "{approx} vs {exact}");
        }
    }

    fn stored(v: &[f64]) -> Vec<u8> {
        let q = QuantizedHyperVector::from_float_scaled(&HyperVector {
            coords: v.to_vec(),
            alpha: 1.0,
        });
        let mut b: Vec<u8> = q.coords.iter().map(|&c| c.cast_unsigned()).collect();
        b.extend_from_slice(&q.alpha.to_le_bytes());
        b
    }

    #[test]
    fn scaled_pair_distance_matches_the_decoded_distance() {
        for d in [1usize, 15, 16, 17, 64, 333, 1024] {
            let a: Vec<f64> = (0..d)
                .map(|i| ((i * 37 % 101) as f64 - 50.0) / 70.0)
                .collect();
            let b: Vec<f64> = (0..d)
                .map(|i| ((i * 53 % 97) as f64 - 48.0) / 90.0)
                .collect();
            let (ba, bb) = (stored(&a), stored(&b));
            let dec = |bytes: &[u8]| -> Vec<f64> {
                let n = bytes.len() - 4;
                let alpha = f32::from_le_bytes(bytes[n..].try_into().unwrap());
                bytes[..n]
                    .iter()
                    .map(|&c| f64::from(c.cast_signed()) * f64::from(alpha / 127.0))
                    .collect()
            };
            let (da, db) = (dec(&ba), dec(&bb));
            let want: f64 = da.iter().zip(&db).map(|(x, y)| (x - y) * (x - y)).sum();
            let got = f64::from(crate::simd::sq8_scaled_l2_pair(&ba, &bb));
            assert!(
                (got - want).abs() <= 1e-3 * want.max(1.0),
                "d={d}: {got} vs {want}"
            );
            assert!(f64::from(crate::simd::sq8_scaled_l2_pair(&ba, &ba)) < 1e-4);
        }
    }

    #[test]
    fn scaled_int_kernel_matches_the_f32_query_kernel() {
        let d = 768;
        let x: Vec<f64> = (0..d)
            .map(|i| ((i * 31 % 89) as f64 - 44.0) / 400.0)
            .collect();
        let q: Vec<f32> = (0..d)
            .map(|i| ((i * 17 % 83) as f32 - 41.0) / 380.0)
            .collect();
        let bytes = stored(&x);
        let n = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[n..].try_into().unwrap());
        let f = crate::simd::sq8_scaled_l2_and_norm_sq(&bytes[..n], alpha / 127.0, &q).0;
        let i = sq8_scaled_l2_int(&bytes[..n], alpha, &Sq8Query::new(&q));
        assert!((f - i).abs() <= 0.01 * f.max(0.1), "{f} vs {i}");
    }

    #[test]
    fn zero_vector_scales_safely() {
        let b = stored(&[0.0; 40]);
        let d = sq8_scaled_l2_int(&b[..40], 1.0, &Sq8Query::new(&[0.5f32; 40]));
        assert!((d - 10.0).abs() < 0.2, "{d}");
        assert!(sq8_scaled_l2_int(&b[..40], 1.0, &Sq8Query::new(&[0.0f32; 40])) < 1e-6);
    }

    #[test]
    fn scaled_codes_use_the_full_int8_range() {
        let v: Vec<f64> = (0..1024).map(|i| ((i % 17) as f64 - 8.0) * 0.004).collect();
        let fixed = QuantizedHyperVector::from_float(
            &HyperVector {
                coords: v.clone(),
                alpha: 1.0,
            },
            false,
        );
        let scaled = QuantizedHyperVector::from_float_scaled(&HyperVector {
            coords: v,
            alpha: 1.0,
        });
        let span = |c: &[i8]| c.iter().map(|x| i32::from(*x).abs()).max().unwrap();
        assert!(
            span(&fixed.coords) <= 5,
            "legacy fixed scale wastes the range"
        );
        assert_eq!(span(&scaled.coords), 127);
    }
}
