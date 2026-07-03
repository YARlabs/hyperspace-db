/// PHASE 4: Blake3 Merkle Inclusion Proofs
///
/// Protects against result falsification: after a search, the node returns a
/// `MerkleProof` for each result vector. The client verifies the proof against
/// the chunk's known Merkle root (obtained from the MetaRouter).
///
/// # Algorithm
///   1. Each vector in a `.hyp` chunk is a leaf: `leaf = blake3(vector_bytes)`.
///   2. A `MerkleTree` is built from all leaves using `rs_merkle` + `blake3`.
///   3. For each result vector index, an `inclusion_proof` is generated.
///   4. The client recomputes `leaf = blake3(received_vector_bytes)` and
///      calls `proof.verify(root, &[leaf_index], &[leaf_hash], total_leaves)`.
///
/// # Why blake3?
///   Blake3 is 5–10× faster than SHA-256 on modern CPUs (SIMD-accelerated),
///   which makes proof generation negligible compared to HNSW search latency.

use rs_merkle::{Hasher, MerkleProof, MerkleTree};
use serde::{Deserialize, Serialize};

// ─── Blake3Hasher for rs_merkle ───────────────────────────────────────────────

#[derive(Clone)]
pub struct Blake3Hasher;

impl Hasher for Blake3Hasher {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        *blake3::hash(data).as_bytes()
    }
}

// ─── Public types ─────────────────────────────────────────────────────────────

/// Serialisable Merkle inclusion proof for a set of result vectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorInclusionProof {
    /// blake3 Merkle root of the chunk.
    pub root: [u8; 32],
    /// Total number of leaves (vectors) in the chunk.
    pub total_leaves: usize,
    /// Leaf indices of the result vectors.
    pub indices: Vec<usize>,
    /// blake3 hashes of the result vectors (leaf values).
    pub leaf_hashes: Vec<[u8; 32]>,
    /// Serialised rs_merkle proof bytes.
    pub proof_bytes: Vec<u8>,
}

/// Build a blake3-based Merkle tree from raw vector byte slices.
///
/// Each element of `vectors` is the serialised bytes of one vector in the chunk.
pub fn build_tree(vectors: &[Vec<u8>]) -> MerkleTree<Blake3Hasher> {
    let leaves: Vec<[u8; 32]> = vectors
        .iter()
        .map(|v| Blake3Hasher::hash(v))
        .collect();
    MerkleTree::<Blake3Hasher>::from_leaves(&leaves)
}

/// Generate an inclusion proof for the given `indices` within `tree`.
///
/// Returns `None` if the tree is empty.
pub fn generate_proof(
    tree: &MerkleTree<Blake3Hasher>,
    vectors: &[Vec<u8>],
    indices: &[usize],
) -> Option<VectorInclusionProof> {
    let root = tree.root()?;
    let total_leaves = tree.leaves_len();

    let proof: MerkleProof<Blake3Hasher> = tree.proof(indices);
    let proof_bytes = proof.to_bytes();

    let leaf_hashes: Vec<[u8; 32]> = indices
        .iter()
        .map(|&i| Blake3Hasher::hash(&vectors[i]))
        .collect();

    Some(VectorInclusionProof {
        root,
        total_leaves,
        indices: indices.to_vec(),
        leaf_hashes,
        proof_bytes,
    })
}

/// Verify a `VectorInclusionProof` on the client side.
///
/// The client re-hashes the received vector bytes and checks the proof against
/// the expected root obtained from the local MetaRouter cache.
///
/// Returns `Ok(())` on success, `Err` describing the failure.
pub fn verify_proof(
    proof: &VectorInclusionProof,
    received_vectors: &[Vec<u8>],
    expected_root: &[u8; 32],
) -> anyhow::Result<()> {
    if proof.indices.len() != received_vectors.len() {
        return Err(anyhow::anyhow!(
            "Proof index count ({}) != received vector count ({})",
            proof.indices.len(),
            received_vectors.len()
        ));
    }

    // Recompute leaf hashes from received vectors
    let recomputed: Vec<[u8; 32]> = received_vectors
        .iter()
        .map(|v| Blake3Hasher::hash(v))
        .collect();

    // Verify each recomputed hash matches what the proof claims
    for (i, (claimed, recomputed)) in proof.leaf_hashes.iter().zip(recomputed.iter()).enumerate() {
        if claimed != recomputed {
            return Err(anyhow::anyhow!(
                "Leaf hash mismatch at index {}: claimed {:?} != recomputed {:?}",
                proof.indices[i], claimed, recomputed
            ));
        }
    }

    // Reconstruct and verify proof
    let merkle_proof = MerkleProof::<Blake3Hasher>::from_bytes(&proof.proof_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to deserialise proof: {e}"))?;

    let valid = merkle_proof.verify(
        *expected_root,
        &proof.indices,
        &proof.leaf_hashes,
        proof.total_leaves,
    );

    if valid {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Merkle proof verification failed — root mismatch or invalid proof"
        ))
    }
}

/// Convenience function: build tree and generate proof in one step.
pub fn prove_results(
    all_vectors: &[Vec<u8>],
    result_indices: &[usize],
) -> Option<VectorInclusionProof> {
    let tree = build_tree(all_vectors);
    generate_proof(&tree, all_vectors, result_indices)
}
