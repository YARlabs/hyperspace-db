//! Shared constants and small utility types.

pub const PROPORTION_HORIZONTAL_DIM: f32 = 0.75;
pub const D_THRESHOLD_FOR_DCT_ROTATION: usize = 512;
pub const H_DIM_SIZE: usize = 64;
pub const MIN_PARTIAL_D: u32 = 16;

pub const DIMENSION_THRESHOLD_FOR_PRUNING: usize = 128;
pub const N_CLUSTERS_THRESHOLD_FOR_PRUNING: usize = 256;

#[cfg(target_os = "macos")]
pub const X_BATCH_SIZE: usize = 40960;
#[cfg(target_os = "macos")]
pub const Y_BATCH_SIZE: usize = 2048;
#[cfg(target_os = "macos")]
pub const MINI_BATCH_SIZE: usize = 256;

#[cfg(not(target_os = "macos"))]
pub const X_BATCH_SIZE: usize = 4096;
#[cfg(not(target_os = "macos"))]
pub const Y_BATCH_SIZE: usize = 1024;

pub const VECTOR_CHUNK_SIZE: usize = Y_BATCH_SIZE;

pub const RECALL_CONVERGENCE_PATIENCE: usize = 2;
pub const CENTROID_PERTURBATION_EPS: f32 = 1.0 / 1024.0;
pub const PRUNER_INITIAL_THRESHOLD: f32 = 1.5;
pub const HIERARCHICAL_PRUNER_INITIAL_THRESHOLD: f32 = 1.1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DistanceFunction {
    L2,
}

#[derive(Copy, Clone, Debug)]
pub struct KnnCandidate {
    pub index: u32,
    pub distance: f32,
}

impl KnnCandidate {
    pub const fn new(index: u32, distance: f32) -> Self {
        Self { index, distance }
    }
}
