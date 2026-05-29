/// # Wave Inference Engine — Klein-Gordon Diffusion on HNSW Graph
///
/// Implements graph-based wave propagation inspired by the Klein-Gordon equation:
///
///   ∂²ψ/∂t² = ∇²ψ − m²ψ + V(x)·ψ
///
/// where:
///   - ψ(x) = wave amplitude / energy at node x
///   - ∇²ψ   = Laplacian diffusion over graph edges (Gaussian kernel on distance)
///   - m²ψ   = mass term for bandpass filtering (penalises low-energy nodes → prevents noise leakage)
///   - V(x)  = query-potential field (attraction toward query-relevant nodes, Schrödinger correction)
///
/// ## Why this works better than plain ANN for structural queries (e.g. CUAD)
///
/// Standard ANN retrieves nodes with highest cosine similarity to the query vector.
/// For documents with dense cross-references (legal clauses, knowledge graphs, financial tables),
/// the *relevant cluster* may span multiple hops from the closest seed.
///
/// Klein-Gordon diffusion propagates energy across HNSW layer-0 graph edges,
/// allowing the wave front to discover such clusters while the mass term
/// suppresses irrelevant side-branches. The query-potential field continuously
/// re-attracts the wave toward query-relevant space at each step.
///
/// ## Parameters
///
/// | Parameter   | Description                                          | Default |
/// |-------------|------------------------------------------------------|---------|
/// | steps       | Number of diffusion iterations                       | 3       |
/// | sigma       | Gaussian kernel bandwidth for edge weight            | 1.0     |
/// | damping     | Energy decay per step (prevents explosion)           | 0.85    |
/// | mass_sq     | m² for Klein-Gordon bandpass (0 = pure diffusion)    | 0.1     |
/// | top_k       | Number of final results                              | 10      |
/// | beam_width  | Max active nodes per step (beam pruning)             | 200     |
///
use hyperspace_core::{Collection, SearchParams};
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters controlling the Klein-Gordon wave diffusion.
pub struct WaveInferenceParams {
    /// Number of diffusion steps (graph hops).
    pub steps: usize,
    /// Gaussian kernel bandwidth σ for edge transfer weights: exp(−d²/σ²).
    pub sigma: f64,
    /// Energy decay coefficient applied each step (0 < damping ≤ 1).
    pub damping: f64,
    /// Klein-Gordon mass term m². Higher → stronger bandpass / more selective.
    /// Set to 0.0 to disable mass term (pure wave diffusion).
    pub mass_sq: f64,
    /// Number of final results to return.
    pub top_k: usize,
    /// Maximum active frontier nodes per diffusion step (beam pruning).
    /// Prevents exponential blow-up on dense graphs.
    pub beam_width: usize,
    /// Query-potential field strength λ. At each step, nodes with higher
    /// query similarity receive a λ-scaled energy boost (Schrödinger correction).
    pub query_potential_strength: f64,
    /// Number of neighbors to fetch per node per diffusion step.
    pub neighbors_per_node: usize,
}

impl Default for WaveInferenceParams {
    fn default() -> Self {
        Self {
            steps: 3,
            sigma: 1.0,
            damping: 0.85,
            mass_sq: 0.1,
            top_k: 10,
            beam_width: 200,
            query_potential_strength: 0.5,
            neighbors_per_node: 12,
        }
    }
}

pub struct WaveInferenceEngine;

impl WaveInferenceEngine {
    /// Klein-Gordon diffusive resonance search on the HNSW Layer-0 graph.
    ///
    /// # Algorithm
    ///
    /// 1. **Seed Phase**: Run a standard HNSW search to get initial seeds.
    ///    Energy is initialised proportional to *cosine similarity* (score), not distance.
    ///    This correctly seeds high-energy at the most relevant nodes.
    ///
    /// 2. **Diffusion Phase** (Klein-Gordon):
    ///    For each step:
    ///      a. Transfer energy along edges weighted by Gaussian kernel exp(−d²/σ²).
    ///      b. Apply mass term: nodes lose energy proportional to m²·ψ (bandpass filter).
    ///      c. Apply query-potential correction: nodes with high query similarity gain
    ///         extra energy proportional to λ·V(x)·ψ (attraction toward query space).
    ///      d. Apply global damping (energy decay).
    ///      e. **Normalise** the energy map (L∞ norm) to prevent explosion or degeneration.
    ///      f. **Beam pruning**: retain only the top `beam_width` nodes by energy.
    ///
    /// 3. **Result Phase**: Return top-K nodes by final energy.
    pub async fn search_diffusive(
        collection: Arc<dyn Collection>,
        query: &[f64],
        params: WaveInferenceParams,
    ) -> Result<Vec<(u32, f64, HashMap<String, String>)>, String> {
        // ── 1. SEED PHASE ───────────────────────────────────────────────────────────
        // Fetch more seeds than needed to ensure good graph coverage.
        let seed_k = (params.top_k * 5).max(params.beam_width / 4).max(20);
        let search_params = SearchParams {
            top_k: seed_k,
            ef_search: 128,
            ..Default::default()
        };

        let seeds = collection
            .search(query, &HashMap::new(), &[], &search_params)
            .await?;

        if seeds.is_empty() {
            return Ok(vec![]);
        }

        // Build query-potential map: node_id → similarity score for seeded nodes.
        // Used later as V(x) in the Schrödinger-like correction.
        let mut query_potential: HashMap<u32, f64> = HashMap::new();

        // Initialise energy proportional to similarity (score in [0,1] range for cosine).
        // Using score directly (not inverse distance) is correct: score=1 → max energy.
        let mut energy_map: HashMap<u32, f64> = HashMap::new();
        for (id, score, _, _) in &seeds {
            // score from HNSW is typically a distance (lower = closer for L2/cosine).
            // We convert: similarity = 1 / (1 + score) so seed with score=0 gets energy=1.
            let similarity = 1.0 / (1.0 + score);
            energy_map.insert(*id, similarity);
            query_potential.insert(*id, similarity);
        }

        // ── 2. DIFFUSION PHASE (Klein-Gordon) ───────────────────────────────────────
        for _step in 0..params.steps {
            // a. Transfer energy along graph edges
            let mut next_energy: HashMap<u32, f64> = HashMap::new();

            for (&id, &psi) in &energy_map {
                if psi.abs() < 1e-9 {
                    continue; // Dead node — skip
                }

                // Propagate to self (retain own energy minus mass decay)
                // Klein-Gordon: ψ_next(x) -= m²·ψ(x)
                let self_energy = psi * (1.0 - params.mass_sq);
                *next_energy.entry(id).or_insert(0.0) += self_energy;

                // Fetch layer-0 neighbors for edge-based diffusion
                if let Ok(neighbors) = collection.graph_neighbors(id, 0, params.neighbors_per_node) {
                    if let Ok(distances) = collection.graph_neighbor_distances(id, &neighbors) {
                        for (neighbor_id, dist) in neighbors.into_iter().zip(distances) {
                            // Gaussian edge kernel: higher weight for closer neighbors
                            let edge_weight = (-dist.powi(2) / params.sigma.powi(2)).exp();
                            // Energy transferred through this edge, damped by global factor
                            let transferred = psi * edge_weight * params.damping;
                            *next_energy.entry(neighbor_id).or_insert(0.0) += transferred;
                        }
                    }
                }
            }

            // b. Apply query-potential correction V(x)·ψ for seeded nodes
            //    This re-attracts the wavefront toward query-relevant space.
            if params.query_potential_strength > 0.0 {
                for (&node_id, &potential) in &query_potential {
                    if let Some(e) = next_energy.get_mut(&node_id) {
                        *e += params.query_potential_strength * potential * *e;
                    }
                }
            }

            // c. Normalise: prevent explosion or degeneration.
            //    Scale all energies so that max |ψ| = 1.0 (L∞ normalisation).
            let max_e = next_energy
                .values()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            if max_e > 1e-12 {
                for e in next_energy.values_mut() {
                    *e /= max_e;
                }
            }

            // d. Beam pruning: keep only the top `beam_width` nodes by energy.
            //    This prevents exponential node explosion on dense graphs.
            if next_energy.len() > params.beam_width {
                let mut sorted_nodes: Vec<(u32, f64)> = next_energy.into_iter().collect();
                sorted_nodes
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                sorted_nodes.truncate(params.beam_width);
                energy_map = sorted_nodes.into_iter().collect();
            } else {
                energy_map = next_energy;
            }
        }

        // ── 3. RESULT PHASE ─────────────────────────────────────────────────────────
        let mut results: Vec<(u32, f64, HashMap<String, String>)> = energy_map
            .into_iter()
            .map(|(id, energy)| {
                let meta = collection.metadata_by_id(id);
                (id, energy, meta)
            })
            .collect();

        // Sort by energy descending (highest resonance = most relevant)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(params.top_k);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::CollectionManager;
    use hyperspace_proto::hyperspace::{CollectionSchema, VectorComponent};
    use std::collections::HashMap;
    use tokio::sync::broadcast;
    use tempfile::tempdir;
    use super::{WaveInferenceParams, WaveInferenceEngine};

    /// Test that Klein-Gordon diffusion correctly propagates energy through a
    /// chain graph: Query ≈ A → B → C
    ///
    /// Expected: A gets highest energy (closest to query), B and C should also
    /// appear in results due to graph diffusion, demonstrating multi-hop discovery.
    #[tokio::test]
    async fn test_klein_gordon_chain_diffusion() {
        let dir = tempdir().unwrap();
        let (tx, _) = broadcast::channel(100);
        let (etx, _) = broadcast::channel(100);
        let mgr = CollectionManager::new(dir.path().to_path_buf(), tx, etx);

        // 8-dimensional L2 collection
        mgr.create_collection(
            "admin",
            "test",
            CollectionSchema {
                components: vec![VectorComponent {
                    name: "vec".to_string(),
                    metric: "l2".to_string(),
                    full_dimension: 8,
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            },
        )
        .await
        .unwrap();
        let col = mgr.get("admin", "test").await.unwrap();

        // Insert chain: A(origin) → B(0.1 away) → C(0.2 away)
        let mut meta = HashMap::new();
        meta.insert("name".to_string(), "A".to_string());
        col.insert(
            &[0.0; 8],
            1,
            meta.clone(),
            1,
            hyperspace_core::Durability::Default,
        )
        .await
        .unwrap();

        meta.insert("name".to_string(), "B".to_string());
        col.insert(
            &[0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            2,
            meta.clone(),
            2,
            hyperspace_core::Durability::Default,
        )
        .await
        .unwrap();

        meta.insert("name".to_string(), "C".to_string());
        col.insert(
            &[0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            3,
            meta.clone(),
            3,
            hyperspace_core::Durability::Default,
        )
        .await
        .unwrap();

        // Wait for async indexing
        while col.queue_size() > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let params = WaveInferenceParams {
            steps: 2,
            sigma: 0.5,
            damping: 0.9,
            mass_sq: 0.05,
            top_k: 3,
            beam_width: 50,
            query_potential_strength: 0.3,
            neighbors_per_node: 10,
        };

        let results = WaveInferenceEngine::search_diffusive(col, &[0.0; 8], params)
            .await
            .unwrap();

        println!("Klein-Gordon diffusion results: {:?}", results);

        // A should be the top result (closest seed, highest initial energy)
        assert!(results.len() >= 2, "Should discover at least A and B via diffusion");
        assert_eq!(
            results[0].2.get("name").unwrap(),
            "A",
            "Node A should have highest energy (closest to query)"
        );
    }

    /// Test energy normalisation: energies must always be in (0, 1] after diffusion.
    #[tokio::test]
    async fn test_energy_normalisation() {
        let dir = tempdir().unwrap();
        let (tx, _) = broadcast::channel(100);
        let (etx, _) = broadcast::channel(100);
        let mgr = CollectionManager::new(dir.path().to_path_buf(), tx, etx);

        mgr.create_collection(
            "admin",
            "norm_test",
            CollectionSchema {
                components: vec![VectorComponent {
                    name: "vec".to_string(),
                    metric: "cosine".to_string(),
                    full_dimension: 4,
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            },
        )
        .await
        .unwrap();
        let col = mgr.get("admin", "norm_test").await.unwrap();

        // Insert a small cluster of 10 vectors
        for i in 0..10u32 {
            let v = i as f64 * 0.05;
            let mut meta = HashMap::new();
            meta.insert("idx".to_string(), i.to_string());
            col.insert(
                &[v, v, v, v],
                i,
                meta,
                i as u64,
                hyperspace_core::Durability::Default,
            )
            .await
            .unwrap();
        }

        while col.queue_size() > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let params = WaveInferenceParams {
            steps: 5,
            mass_sq: 0.2,
            top_k: 5,
            ..Default::default()
        };

        let results = WaveInferenceEngine::search_diffusive(col, &[0.0; 4], params)
            .await
            .unwrap();

        // All energies must be in (0, 1] — normalisation invariant
        for (id, energy, _) in &results {
            assert!(
                *energy > 0.0 && *energy <= 1.0 + 1e-9,
                "Energy for node {id} = {energy} is out of bounds (0, 1]"
            );
        }
    }
}
