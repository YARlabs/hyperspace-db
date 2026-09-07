#[cfg(test)]
mod tests {
    use crate::hybrid::HybridQuantizedVector;
    use crate::vector::HyperVector;
    use crate::Metric;

    const LORENTZ_DIM: usize = 33;
    const EUCLIDEAN_DIM: usize = 768;

    #[test]
    fn test_hybrid_distance_exact() {
        let mut a = [0.0f64; 801];
        let mut b = [0.0f64; 801];

        // Lorentz part (t, x1, ...)
        a[0] = 1.0;
        a[1] = 0.0;
        b[0] = 1.0;
        b[1] = 0.0;

        // Euclidean part
        a[33] = 1.0;
        b[33] = 2.0;

        // HybridMetric::distance panics by design — use the low-level metric separately
        let d_lor = {
            let lor_a = &a[..LORENTZ_DIM];
            let lor_b = &b[..LORENTZ_DIM];
            crate::LorentzMetric::distance(lor_a, lor_b)
        };
        let d_euc = {
            let euc_a = &a[LORENTZ_DIM..];
            let euc_b = &b[LORENTZ_DIM..];
            crate::EuclideanMetric::distance(euc_a, euc_b)
        };
        let d = d_lor + d_euc;
        // d_lor = acosh(-(-1*1 + 0*0)) = acosh(1) = 0
        // d_euc = (1-2)^2 = 1
        // d_total = 0 + 1 = 1
        assert!((d - 1.0).abs() < 1e-5, "Expected ~1.0, got {d}");
    }

    #[test]
    fn test_asymmetric_quantization() {
        let mut v_coords = vec![0.0f64; 801];
        v_coords[0] = 2.0; // t
        v_coords[1] = 1.73205; // sqrt(3)
        v_coords[33] = 0.5; // Euclidean

        let v = HyperVector::new_unchecked(v_coords);
        let q = HybridQuantizedVector::from_float(&v, LORENTZ_DIM, EUCLIDEAN_DIM);

        // Check Lorentz precision (FP32)
        assert!((f64::from(q.lorentz[0]) - 2.0).abs() < 1e-6);
        assert!((f64::from(q.lorentz[1]) - 1.73205).abs() < 1e-4);

        // Check Euclidean quantization (INT8)
        // 0.5 * 127 = 63.5 -> 63 or 64
        assert!(q.euclidean[0] == 63 || q.euclidean[0] == 64);

        // Check distance reconstructed
        let d = q.lorentz_distance_to_float(&v);
        assert!(d < 0.01, "Expected near-zero self-distance, got {d}");
    }

    #[test]
    fn test_hybrid_mrl_simd() {
        let mut v_coords = vec![0.0f64; 801];
        v_coords[0] = 1.0;
        #[allow(clippy::cast_precision_loss)]
        for i in 33..801 {
            v_coords[i] = (i as f64) / 1000.0;
        }

        let v = HyperVector::new_unchecked(v_coords);
        let q = HybridQuantizedVector::from_float(&v, LORENTZ_DIM, EUCLIDEAN_DIM);

        // Test 96D MRL
        let d_96 = q.distance_mrl(&v, 96);
        // Test 768D (Full)
        let d_768 = q.distance_mrl(&v, 768);

        assert!(d_96 < d_768, "d_96={d_96} should be < d_768={d_768}");
        // Should be small because we compare q with its own source v
        assert!(d_768 < 0.1, "d_768={d_768} should be < 0.1");
    }

    #[test]
    fn test_low_bit_hybrid_quantization() {
        use crate::hybrid::HybridLowBitQuantizedVector;
        let mut v_coords = vec![0.0f64; 801];
        v_coords[0] = 2.0; // Lorentz
        v_coords[1] = 1.73205;
        v_coords[33] = 0.5; // Euclidean
        v_coords[34] = -0.25;

        let v = HyperVector::new_unchecked(v_coords);
        let q = HybridLowBitQuantizedVector::from_float(&v, LORENTZ_DIM, EUCLIDEAN_DIM);

        // Verify Lorentz parts are preserved as f32
        assert!((f64::from(q.lorentz[0]) - 2.0).abs() < 1e-6);
        assert!((f64::from(q.lorentz[1]) - 1.73205).abs() < 1e-4);

        // Verify Euclidean parts are packed in 4-bit block-wise quantization
        let packed_byte = q.euclidean_packed[0];
        let u1 = packed_byte >> 4;
        let u2 = packed_byte & 0x0F;
        assert_eq!(u1, 15);
        assert_eq!(u2, 4);

        // Distance to self should be near zero
        let d = q.lorentz_distance_to_float(&v);
        assert!(d < 0.01);
        let d_euc = q.euclidean_distance_sq_mrl(&v, EUCLIDEAN_DIM);
        assert!(d_euc < 0.01);
    }

    #[test]
    fn test_extreme_quantization_1bit() {
        use crate::hybrid::HybridExtremeQuantizedVector;
        let mut v_coords = vec![0.0f64; 801];
        v_coords[0] = 2.0; // Lorentz t
        v_coords[1] = 1.73205;
        v_coords[33] = 0.5; // Euclidean positive -> bit 1
        v_coords[34] = -0.25; // Euclidean negative -> bit 0

        let v = HyperVector::new_unchecked(v_coords);
        let q = HybridExtremeQuantizedVector::from_float(&v, LORENTZ_DIM, EUCLIDEAN_DIM);

        // Verify Lorentz parts are preserved as f32
        assert!((f64::from(q.lorentz[0]) - 2.0).abs() < 1e-6);
        assert!((f64::from(q.lorentz[1]) - 1.73205).abs() < 1e-4);

        // Verify Euclidean parts are packed in 1-bit per dim
        // Bit 0 is 1 (v[33] > 0), Bit 1 is 0 (v[34] <= 0)
        assert_eq!(q.euclidean_bits[0] & 0x01, 1);
        assert_eq!((q.euclidean_bits[0] >> 1) & 0x01, 0);

        // Serialization roundtrip
        let bytes = q.as_bytes();
        assert_eq!(bytes.len(), 33 * 4 + 4 + 768 / 8); // 132 + 4 + 96 = 232 bytes
        let q_deq = HybridExtremeQuantizedVector::from_bytes(&bytes);
        assert_eq!(q_deq.euclidean_bits, q.euclidean_bits);
        assert_eq!(q_deq.lorentz, q.lorentz);

        // Distance to self Lorentz part should be ~0
        let d_lor = q.lorentz_distance_to_float(&v);
        assert!(d_lor < 0.01);
    }
}
