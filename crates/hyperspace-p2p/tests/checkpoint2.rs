#![allow(clippy::pedantic)]
//! Checkpoint 2 Tests — `hyperspace-p2p`
//!
//! Tests cover:
//!   - NodeIdentity: generation, determinism, file persistence
//!   - MetaRouter: upsert, find_nearest (cosine), merkle root
//!   - ed25519 sign/verify via NodeIdentity

#[cfg(test)]
mod tests {
    use hyperspace_p2p::{
        identity::NodeIdentity,
        routing::{compute_merkle_root, MetaRouter, MetaRouterEntry},
    };
    use tempfile::NamedTempFile;

    // ─── NodeIdentity tests ────────────────────────────────────────────────

    #[test]
    fn test_node_identity_generate_fresh() {
        let tmp = NamedTempFile::new().unwrap();
        let identity = NodeIdentity::load_or_generate(tmp.path(), 42).unwrap();

        // PeerId must be set
        let peer_id_str = identity.peer_id.to_base58();
        assert!(!peer_id_str.is_empty(), "PeerId should not be empty");
        assert_eq!(identity.semantic_zone, 42);
    }

    #[test]
    fn test_node_identity_deterministic_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let id1 = NodeIdentity::from_mnemonic(mnemonic, 0).unwrap();
        let id2 = NodeIdentity::from_mnemonic(mnemonic, 0).unwrap();

        assert_eq!(
            id1.peer_id, id2.peer_id,
            "Same mnemonic must produce same PeerId"
        );
        assert_eq!(
            id1.public_key_bytes(),
            id2.public_key_bytes(),
            "Same mnemonic must produce same public key"
        );
    }

    #[test]
    fn test_node_identity_different_mnemonics_different_peers() {
        let m1 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let m2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let id1 = NodeIdentity::from_mnemonic(m1, 0).unwrap();
        let id2 = NodeIdentity::from_mnemonic(m2, 0).unwrap();

        assert_ne!(
            id1.peer_id, id2.peer_id,
            "Different mnemonics must produce different PeerIds"
        );
    }

    #[test]
    fn test_node_identity_persist_and_reload() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Generate and save
        let id1 = NodeIdentity::load_or_generate(&path, 7).unwrap();
        let peer1 = id1.peer_id;

        // Reload from same file
        let id2 = NodeIdentity::load_or_generate(&path, 7).unwrap();
        assert_eq!(peer1, id2.peer_id, "PeerId must survive disk round-trip");
    }

    #[test]
    fn test_node_identity_sign_and_verify() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let identity = NodeIdentity::from_mnemonic(mnemonic, 0).unwrap();

        let message = b"hyperspace-node-hello";
        let sig_bytes = identity.sign(message);

        // Verify using ed25519-dalek directly
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let vk = VerifyingKey::from_bytes(&identity.public_key_bytes()).unwrap();
        let sig = Signature::from_bytes(&sig_bytes);
        assert!(
            vk.verify(message, &sig).is_ok(),
            "Signature verification should succeed"
        );
    }

    #[test]
    fn test_node_identity_sign_tampered_fails() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let identity = NodeIdentity::from_mnemonic(mnemonic, 0).unwrap();

        let message = b"original message";
        let sig_bytes = identity.sign(message);
        let tampered = b"tampered message";

        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let vk = VerifyingKey::from_bytes(&identity.public_key_bytes()).unwrap();
        let sig = Signature::from_bytes(&sig_bytes);
        assert!(
            vk.verify(tampered, &sig).is_err(),
            "Signature should fail for tampered message"
        );
    }

    #[test]
    fn test_node_identity_libp2p_keypair() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let identity = NodeIdentity::from_mnemonic(mnemonic, 0).unwrap();
        let kp = identity.libp2p_keypair().unwrap();

        // Public key should be ed25519 and match
        assert!(
            matches!(kp.key_type(), libp2p::identity::KeyType::Ed25519),
            "Keypair should be Ed25519"
        );
    }

    // ─── MetaRouter tests ──────────────────────────────────────────────────

    fn make_entry(chunk_id: u64, centroid: Vec<f32>) -> MetaRouterEntry {
        MetaRouterEntry {
            chunk_id,
            semantic_centroid: centroid,
            merkle_root: [0u8; 32],
            holder_peers: vec!["peer-1".to_string()],
            updated_at: 0,
        }
    }

    #[test]
    fn test_metarouter_upsert_and_get() {
        let router = MetaRouter::new();
        router.upsert(make_entry(1, vec![1.0, 0.0, 0.0]));
        router.upsert(make_entry(2, vec![0.0, 1.0, 0.0]));

        assert_eq!(router.len(), 2);
        let e = router.get(1).unwrap();
        assert_eq!(e.chunk_id, 1);
    }

    #[test]
    fn test_metarouter_find_nearest_cosine() {
        let router = MetaRouter::new();
        // chunk 1 points in x-direction
        router.upsert(make_entry(1, vec![1.0, 0.0, 0.0]));
        // chunk 2 points in y-direction
        router.upsert(make_entry(2, vec![0.0, 1.0, 0.0]));
        // chunk 3 points in z-direction
        router.upsert(make_entry(3, vec![0.0, 0.0, 1.0]));

        // Query close to x-direction
        let results = router.find_nearest(&[0.9, 0.1, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].chunk_id, 1,
            "Nearest to x-query should be chunk 1"
        );
    }

    #[test]
    fn test_metarouter_upsert_overwrites() {
        let router = MetaRouter::new();
        router.upsert(make_entry(42, vec![1.0, 0.0]));
        let mut updated = make_entry(42, vec![0.0, 1.0]);
        updated.updated_at = 999;
        router.upsert(updated);

        assert_eq!(router.len(), 1, "Upsert should overwrite not append");
        let e = router.get(42).unwrap();
        assert_eq!(e.updated_at, 999, "Should reflect latest update");
    }

    // ─── Blake3 Merkle Root tests ──────────────────────────────────────────

    #[test]
    fn test_merkle_root_deterministic() {
        let leaves: Vec<Vec<u8>> = (0..16u64).map(|i| i.to_le_bytes().to_vec()).collect();

        let root1 = compute_merkle_root(&leaves);
        let root2 = compute_merkle_root(&leaves);
        assert_eq!(root1, root2, "Merkle root must be deterministic");
        assert_ne!(root1, [0u8; 32], "Merkle root must not be all zeros");
    }

    #[test]
    fn test_merkle_root_changes_on_tamper() {
        let leaves: Vec<Vec<u8>> = (0..8u64).map(|i| i.to_le_bytes().to_vec()).collect();
        let mut tampered = leaves.clone();
        tampered[3] = b"TAMPERED".to_vec();

        let root_original = compute_merkle_root(&leaves);
        let root_tampered = compute_merkle_root(&tampered);
        assert_ne!(
            root_original, root_tampered,
            "Tampered data must produce different Merkle root"
        );
    }

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaves = vec![b"single-vector".to_vec()];
        let root = compute_merkle_root(&leaves);
        assert_ne!(root, [0u8; 32]);
    }

    /// Simulate Checkpoint 2 scenario:
    /// 3 virtual nodes each with distinct identities and chunk routing
    #[test]
    fn test_three_node_simulation_routing() {
        let mnemonic1 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let mnemonic3 =
            "legal winner thank year wave sausage worth useful legal winner thank yellow";

        let id1 = NodeIdentity::from_mnemonic(mnemonic1, 0).unwrap();
        let id2 = NodeIdentity::from_mnemonic(mnemonic2, 1).unwrap();
        let id3 = NodeIdentity::from_mnemonic(mnemonic3, 2).unwrap();

        // Each node has a MetaRouter
        let router1 = MetaRouter::new();
        let router2 = MetaRouter::new();
        let router3 = MetaRouter::new();

        // Node 1 holds chunks in the x-direction
        router1.upsert(MetaRouterEntry {
            chunk_id: 100,
            semantic_centroid: vec![1.0, 0.0, 0.0],
            merkle_root: [1u8; 32],
            holder_peers: vec![id1.peer_id.to_base58()],
            updated_at: 1000,
        });

        // Node 2 holds chunks in y-direction
        router2.upsert(MetaRouterEntry {
            chunk_id: 200,
            semantic_centroid: vec![0.0, 1.0, 0.0],
            merkle_root: [2u8; 32],
            holder_peers: vec![id2.peer_id.to_base58()],
            updated_at: 1000,
        });

        // Node 3 holds chunks in z-direction
        router3.upsert(MetaRouterEntry {
            chunk_id: 300,
            semantic_centroid: vec![0.0, 0.0, 1.0],
            merkle_root: [3u8; 32],
            holder_peers: vec![id3.peer_id.to_base58()],
            updated_at: 1000,
        });

        // Simulate a DHT that aggregates all routing info (merge all into a "global view")
        let global_router = MetaRouter::new();
        for entry in router1.find_nearest(&[1.0, 0.0, 0.0], 10) {
            global_router.upsert(entry);
        }
        for entry in router2.find_nearest(&[0.0, 1.0, 0.0], 10) {
            global_router.upsert(entry);
        }
        for entry in router3.find_nearest(&[0.0, 0.0, 1.0], 10) {
            global_router.upsert(entry);
        }

        // Query in x-direction: should route to node1's chunk 100
        let result = global_router.find_nearest(&[1.0, 0.0, 0.0], 1);
        assert_eq!(result[0].chunk_id, 100, "x-query should route to chunk 100");

        // Query in y-direction: should route to node2's chunk 200
        let result = global_router.find_nearest(&[0.0, 1.0, 0.0], 1);
        assert_eq!(result[0].chunk_id, 200, "y-query should route to chunk 200");

        // Query in z-direction: should route to node3's chunk 300
        let result = global_router.find_nearest(&[0.0, 0.0, 1.0], 1);
        assert_eq!(result[0].chunk_id, 300, "z-query should route to chunk 300");

        // Verify all three nodes have distinct peer IDs (no collision)
        assert_ne!(id1.peer_id, id2.peer_id);
        assert_ne!(id2.peer_id, id3.peer_id);
        assert_ne!(id1.peer_id, id3.peer_id);
    }
}
