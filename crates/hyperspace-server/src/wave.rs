use hyperspace_core::{Collection, SearchParams};
use std::collections::HashMap;
use std::sync::Arc;

pub struct WaveInferenceParams {
    pub steps: usize,
    pub sigma: f64,
    pub damping: f64,
    pub top_k: usize,
    pub frequency: f64, // Added for PLL
}

impl Default for WaveInferenceParams {
    fn default() -> Self {
        Self {
            steps: 3,
            sigma: 1.0,
            damping: 0.8,
            top_k: 10,
            frequency: std::f64::consts::PI / 2.0, // Default 90 deg phase per unit distance
        }
    }
}

pub struct WaveInferenceEngine;

impl WaveInferenceEngine {
    pub async fn search_diffusive(
        collection: Arc<dyn Collection>,
        query: &[f64],
        params: WaveInferenceParams,
    ) -> Result<Vec<(u32, f64, HashMap<String, String>)>, String> {
        // 1. Initial Greedy Search (Seeds)
        let search_params = SearchParams {
            top_k: params.top_k * 5,
            ef_search: 100,
            ..Default::default()
        };

        let seeds = collection
            .search(query, &HashMap::new(), &[], &search_params)
            .await?;

        let mut energy_map: HashMap<u32, f64> = HashMap::new();
        for (id, score, _, _) in &seeds {
            // Initial energy proportional to inverse distance
            energy_map.insert(*id, 1.0 / (1.0 + score));
        }

        // 2. Diffusion Steps (Phase-Locked Loop)
        for _ in 0..params.steps {
            let mut next_energy = energy_map.clone();

            for (&id, &energy) in &energy_map {
                if energy.abs() < 0.01 {
                    continue;
                } // Skip dead waves

                // Get neighbors in Layer 0
                if let Ok(neighbors) = collection.graph_neighbors(id, 0, 10) {
                    if let Ok(distances) = collection.graph_neighbor_distances(id, &neighbors) {
                        for (neighbor_id, dist) in neighbors.into_iter().zip(distances) {
                            // Phase-Locked Loop (PLL) interference:
                            // The distance becomes a phase shift. Waves can destructively interfere.
                            let phase_shift = dist * params.frequency;
                            let resonance = energy
                                * (-dist.powi(2) / params.sigma.powi(2)).exp()
                                * phase_shift.cos()
                                * params.damping;

                            let e = next_energy.entry(neighbor_id).or_insert(0.0);
                            *e += resonance;
                        }
                    }
                }
            }
            energy_map = next_energy;
        }

        // 3. Collect Results
        let mut results: Vec<(u32, f64, HashMap<String, String>)> = energy_map
            .into_iter()
            .map(|(id, energy)| {
                let meta = collection.metadata_by_id(id);
                (id, energy, meta)
            })
            .collect();

        // Sort by energy (highest first)
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
    use tempfile::tempdir;
    use tokio::sync::broadcast;
    use super::{WaveInferenceParams, WaveInferenceEngine};

    #[tokio::test]
    async fn test_diffusive_resonance() {
        let dir = tempdir().unwrap();
        let (tx, _) = broadcast::channel(100);
        let (etx, _) = broadcast::channel(100);
        let mgr = CollectionManager::new(dir.path().to_path_buf(), tx, etx);

        // Create 2D Euclidean collection
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

        // Insert points in a chain: A -> B -> C
        // A (0,0), B (0.1, 0), C (0.2, 0)
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

        // Wait for indexing
        while col.queue_size() > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // Search near A (0,0)
        let params = WaveInferenceParams {
            steps: 2,
            sigma: 0.2,
            damping: 0.9,
            top_k: 3,
            frequency: std::f64::consts::PI / 2.0,
        };

        let results = WaveInferenceEngine::search_diffusive(col, &[0.0; 8], params)
            .await
            .unwrap();

        // Even if C is further from query than A, it should have picked up some resonance through B
        println!("Diffusive results: {:?}", results);
        assert!(results.len() >= 2);
        assert_eq!(results[0].2.get("name").unwrap(), "A");
    }
}
