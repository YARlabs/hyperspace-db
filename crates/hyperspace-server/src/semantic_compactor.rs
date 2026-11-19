//! SemanticCompactor: Uses SuperKMeans to cluster and partition dataset vectors into Voronoi chunks.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use hyperspace_core::{GlobalConfig, Metric};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use serde::{Deserialize, Serialize};

use crate::meta_router::ChunkMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReport {
    pub initial_chunks: usize,
    pub compacted_chunks: usize,
    pub total_vectors: usize,
    pub dimension: usize,
    pub clustering_time_ms: u64,
    pub total_time_ms: u64,
}

pub struct SemanticCompactor;

impl SemanticCompactor {
    /// Repartitions a set of vectors into `target_partitions` Voronoi chunks using SuperKMeans.
    /// Each resulting chunk is written as an immutable `.hyp` directory containing:
    /// - VectorStore
    /// - `index.snap` (HNSW local snapshot)
    /// - Associated `ChunkMeta` with exact centroid computed by SuperKMeans.
    pub fn compact_vectors<M: Metric>(
        points: &[(u32, Vec<f64>, HashMap<String, String>)],
        target_partitions: usize,
        data_dir: &Path,
        mode: hyperspace_core::QuantizationMode,
        config: &Arc<GlobalConfig>,
        storage_f32: bool,
    ) -> Result<(CompactionReport, Vec<ChunkMeta>), String> {
        let start = Instant::now();
        if points.is_empty() {
            return Err("No vectors provided for semantic compaction".into());
        }
        let total_vectors = points.len();
        let dimension = points[0].1.len();
        let k = target_partitions.clamp(1, total_vectors);

        let cluster_start = Instant::now();
        let vectors: Vec<Vec<f64>> = points.iter().map(|(_, v, _)| v.clone()).collect();
        let (centroids, assignments) = hyperspace_clustering::cluster_vectors_f64(&vectors, k)
            .ok_or_else(|| "SuperKMeans clustering failed to produce clusters".to_string())?;
        let clustering_time_ms = cluster_start.elapsed().as_millis() as u64;

        // Group vectors by their assigned cluster index
        let mut partitions: Vec<Vec<usize>> = vec![Vec::new(); centroids.len()];
        for (vec_idx, &cluster_idx) in assignments.iter().enumerate() {
            if cluster_idx < partitions.len() {
                partitions[cluster_idx].push(vec_idx);
            }
        }

        let element_size =
            crate::chunk_searcher::storage_element_size::<M>(mode, dimension, storage_f32);

        let mut output_metas = Vec::new();

        for (c_idx, members) in partitions.into_iter().enumerate() {
            if members.is_empty() {
                continue;
            }
            let chunk_id = uuid::Uuid::new_v4().to_string();
            let chunk_name = format!("chunk_compact_{chunk_id}.hyp");
            let chunk_dir = data_dir.join(&chunk_name);

            std::fs::create_dir_all(&chunk_dir)
                .map_err(|e| format!("Failed to create compact chunk dir {chunk_name}: {e}"))?;

            let temp_store = Arc::new(VectorStore::new(&chunk_dir, element_size));
            let local_index = HnswIndex::<M>::new_with_storage_precision(
                temp_store.clone(),
                mode,
                config.clone(),
                storage_f32,
                dimension,
            );

            let mut insert_count = 0u32;
            let mut chunk_user_ids: Vec<u32> = Vec::with_capacity(members.len());
            for &member_idx in &members {
                let (user_id, vec, meta) = &points[member_idx];
                if let Ok(new_id) = local_index.insert_to_storage(vec) {
                    let _ = local_index.index_node(new_id, meta.clone());
                    let slot = new_id as usize;
                    if chunk_user_ids.len() <= slot {
                        chunk_user_ids.resize(slot + 1, u32::MAX);
                    }
                    chunk_user_ids[slot] = *user_id;
                    insert_count += 1;
                }
            }

            if insert_count > 0 {
                crate::chunk_searcher::ChunkManifest {
                    element_size: u32::try_from(element_size).unwrap_or(u32::MAX),
                    dimension: u32::try_from(dimension).unwrap_or(u32::MAX),
                    storage_f32,
                    ids: chunk_user_ids,
                }
                .write(&chunk_dir)
                .map_err(|e| format!("Failed to write chunk manifest for {chunk_name}: {e}"))?;
                // Freeze Layer-0 into cache-aligned continuous strided layout for zero-chasing search
                local_index.freeze_flat_layer0();
                local_index
                    .save_snapshot(&chunk_dir.join("index.snap"))
                    .map_err(|e| format!("Failed to save index snapshot for {chunk_name}: {e}"))?;

                output_metas.push(ChunkMeta {
                    chunk_id: chunk_name,
                    path: chunk_dir,
                    centroid: centroids[c_idx].clone(),
                    vector_count: insert_count,
                });
            }
        }

        let total_time_ms = start.elapsed().as_millis() as u64;
        let report = CompactionReport {
            initial_chunks: 0,
            compacted_chunks: output_metas.len(),
            total_vectors,
            dimension,
            clustering_time_ms,
            total_time_ms,
        };

        Ok((report, output_metas))
    }
}
