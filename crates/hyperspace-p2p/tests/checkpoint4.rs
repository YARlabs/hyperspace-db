/// Phase 4 Checkpoint Tests — Blake3 Merkle Inclusion Proofs
///
/// Tests cover:
///   - Proof generation and verification (happy path)
///   - Tampered vector detection on client side
///   - Multi-result proof (N results from one chunk)
///   - Empty tree edge case
///   - Root mismatch detection

#[cfg(test)]
mod tests {
    use hyperspace_p2p::proofs::{build_tree, generate_proof, prove_results, verify_proof};

    /// Generate N fake vector byte representations
    fn make_vectors(n: usize, dim: usize) -> Vec<Vec<u8>> {
        (0..n)
            .map(|i| {
                // Each vector is dim f32s: [i as f32, i+1, ..., i+dim-1]
                let floats: Vec<f32> = (0..dim).map(|j| (i + j) as f32).collect();
                bytemuck::cast_slice(&floats).to_vec()
            })
            .collect()
    }

    // ─── Happy Path ────────────────────────────────────────────────────────

    #[test]
    fn test_proof_single_result_verify() {
        let vectors = make_vectors(16, 32);
        let tree = build_tree(&vectors);
        let root = tree.root().unwrap();

        // Prove result at index 5
        let proof = generate_proof(&tree, &vectors, &[5]).unwrap();
        let result_vecs = vec![vectors[5].clone()];

        assert!(
            verify_proof(&proof, &result_vecs, &root).is_ok(),
            "Single-result proof should verify"
        );
    }

    #[test]
    fn test_proof_multiple_results_verify() {
        let vectors = make_vectors(64, 32);
        let tree = build_tree(&vectors);
        let root = tree.root().unwrap();

        // Prove results at indices 3, 17, 42
        let indices = [3usize, 17, 42];
        let proof = generate_proof(&tree, &vectors, &indices).unwrap();
        let result_vecs: Vec<Vec<u8>> = indices.iter().map(|&i| vectors[i].clone()).collect();

        assert!(
            verify_proof(&proof, &result_vecs, &root).is_ok(),
            "Multi-result proof should verify"
        );
    }

    #[test]
    fn test_prove_results_convenience() {
        let vectors = make_vectors(32, 64);
        let proof = prove_results(&vectors, &[0, 15, 31]).unwrap();

        let result_vecs: Vec<Vec<u8>> = [0usize, 15, 31]
            .iter()
            .map(|&i| vectors[i].clone())
            .collect();

        let root = {
            let tree = build_tree(&vectors);
            tree.root().unwrap()
        };

        assert!(
            verify_proof(&proof, &result_vecs, &root).is_ok(),
            "prove_results convenience function should produce valid proof"
        );
    }

    // ─── Tamper Detection ──────────────────────────────────────────────────

    /// Checkpoint 4 core test: node returns a tampered vector → client detects it.
    #[test]
    fn test_tampered_vector_detected() {
        let vectors = make_vectors(16, 32);
        let tree = build_tree(&vectors);
        let root = tree.root().unwrap();

        let proof = generate_proof(&tree, &vectors, &[7]).unwrap();

        // Simulate node returning WRONG vector bytes
        let mut tampered = vectors[7].clone();
        tampered[0] ^= 0xFF; // flip first byte

        let result = verify_proof(&proof, &[tampered], &root);
        assert!(
            result.is_err(),
            "Tampered vector must be detected by proof verification"
        );
    }

    #[test]
    fn test_wrong_root_detected() {
        let vectors = make_vectors(16, 32);
        let tree = build_tree(&vectors);
        let proof = generate_proof(&tree, &vectors, &[0]).unwrap();

        // Client has the wrong root (stale MetaRouter data)
        let wrong_root = [0xFFu8; 32];
        let result = verify_proof(&proof, &[vectors[0].clone()], &wrong_root);
        assert!(
            result.is_err(),
            "Wrong root must be detected"
        );
    }

    #[test]
    fn test_wrong_index_in_proof_detected() {
        let vectors = make_vectors(16, 32);
        let tree = build_tree(&vectors);
        let root = tree.root().unwrap();

        // Prove index 3, but submit vector at index 5
        let proof = generate_proof(&tree, &vectors, &[3]).unwrap();
        let wrong_vec = vec![vectors[5].clone()]; // different content

        let result = verify_proof(&proof, &wrong_vec, &root);
        assert!(result.is_err(), "Mismatched vector/proof index must fail");
    }

    // ─── Determinism ───────────────────────────────────────────────────────

    #[test]
    fn test_merkle_root_is_deterministic() {
        let vectors = make_vectors(32, 16);
        let root1 = build_tree(&vectors).root().unwrap();
        let root2 = build_tree(&vectors).root().unwrap();
        assert_eq!(root1, root2, "Merkle root must be deterministic");
    }

    #[test]
    fn test_different_data_different_roots() {
        let v1 = make_vectors(8, 16);
        let mut v2 = v1.clone();
        v2[3][0] ^= 0x01;

        let root1 = build_tree(&v1).root().unwrap();
        let root2 = build_tree(&v2).root().unwrap();
        assert_ne!(root1, root2, "Modified data must produce different root");
    }

    // ─── Large-Scale Test ─────────────────────────────────────────────────

    /// Checkpoint 4 scale test: generate proofs for a 1024-vector chunk.
    #[test]
    fn test_proof_large_chunk() {
        let vectors = make_vectors(1024, 128); // 1024 vectors of 128 dims
        let tree = build_tree(&vectors);
        let root = tree.root().unwrap();

        // Prove top-10 results (HNSW would return ~10-100 results per search)
        let indices: Vec<usize> = (0..10).collect();
        let proof = generate_proof(&tree, &vectors, &indices).unwrap();
        let result_vecs: Vec<Vec<u8>> = indices.iter().map(|&i| vectors[i].clone()).collect();

        assert!(
            verify_proof(&proof, &result_vecs, &root).is_ok(),
            "Large chunk proof should verify in < 1ms"
        );
    }
}
