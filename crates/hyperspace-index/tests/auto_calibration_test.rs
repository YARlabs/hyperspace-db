//! Online auto-calibration of trained quantizers (PQ / OPQ / PQ4 / RaBitQ).

use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

const DIM: usize = 64;

fn gauss(seed: u64) -> f64 {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
    x ^= x >> 31;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    let u1 = ((x >> 11) as f64 / (1u64 << 53) as f64).max(1e-12);
    let mut y = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    y ^= y >> 29;
    let u2 = (y >> 11) as f64 / (1u64 << 53) as f64;
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Clustered, off-origin data: a data-independent default codebook fits it poorly.
fn point(i: u32) -> Vec<f64> {
    let c = u64::from(i % 25);
    (0..DIM)
        .map(|d| {
            let d = d as u64;
            2.0 + 3.0 * gauss(c * 1000 + d) + gauss(u64::from(i) * 7919 + d + 1_000_000)
        })
        .collect()
}

fn element_size(mode: QuantizationMode) -> usize {
    match mode {
        QuantizationMode::RaBitQ => hyperspace_core::rabitq::code_size(DIM),
        QuantizationMode::RaBitQ4 => hyperspace_core::rabitq::code_size_bits(DIM, 4),
        QuantizationMode::RaBitQ2 => hyperspace_core::rabitq::code_size_bits(DIM, 2),
        QuantizationMode::PQ4 => {
            hyperspace_core::pq4::code_size(hyperspace_core::pq4::default_num_subvectors(DIM))
        }
        _ => hyperspace_core::pq::default_num_subvectors(DIM),
    }
}

fn new_index(dir: &std::path::Path, mode: QuantizationMode) -> HnswIndex<EuclideanMetric> {
    let store = Arc::new(VectorStore::new(dir, element_size(mode)));
    HnswIndex::<EuclideanMetric>::new(store, mode, Arc::new(GlobalConfig::default()), DIM)
}

fn to_f32(v: &[f64]) -> Vec<f32> {
    v.iter().map(|&x| x as f32).collect()
}

/// Recall@10 inside a 50-candidate window (codes are re-ranked server-side).
fn recall(index: &HnswIndex<EuclideanMetric>, data: &[Vec<f64>]) -> f64 {
    let (k, queries) = (10usize, 30u32);
    let mut hit = 0usize;
    for qi in 0..queries {
        let q = point(data.len() as u32 + 7 * qi);
        let mut exact: Vec<(u32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let d = v.iter().zip(&q).map(|(a, b)| (a - b) * (a - b)).sum();
                (i as u32, d)
            })
            .collect();
        exact.sort_by(|a, b| a.1.total_cmp(&b.1));
        let res = index.search(
            &q,
            &HashMap::new(),
            &[],
            &SearchParams {
                top_k: 50,
                ef_search: 128,
                ..Default::default()
            },
        );
        hit += exact[..k]
            .iter()
            .filter(|(id, _)| res.iter().any(|(r, _)| r == id))
            .count();
    }
    hit as f64 / (queries as usize * k) as f64
}

enum Calibration {
    /// Default codebook forever.
    Never,
    /// Trained on the full data set before any insert (vacuum / rebuild path).
    Upfront,
    /// Trained online after the first `n` inserts, which are re-encoded in place.
    OnlineAfter(usize),
}

fn run(mode: QuantizationMode, calibration: &Calibration) -> f64 {
    let n = 1500u32;
    let data: Vec<Vec<f64>> = (0..n).map(point).collect();
    let dir = tempdir().unwrap();
    let mut index = new_index(dir.path(), mode);
    assert!(index.needs_calibration());

    if let Calibration::Upfront = calibration {
        let f32s: Vec<Vec<f32>> = data.iter().map(|v| to_f32(v)).collect();
        let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
        index.train_pq(&refs, 8);
        assert!(!index.needs_calibration());
    }

    for (i, v) in data.iter().enumerate() {
        if let Calibration::OnlineAfter(at) = calibration {
            if i == *at {
                let f32s: Vec<Vec<f32>> = data[..i].iter().map(|v| to_f32(v)).collect();
                let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
                let (re_encoded, missing) =
                    index.calibrate_online(&refs, 8, |id| data.get(id as usize).map(|v| to_f32(v)));
                assert_eq!(re_encoded, i, "every stored vector is re-encoded");
                assert_eq!(missing, 0);
                assert!(!index.needs_calibration());
                // A second call is a no-op.
                assert_eq!(index.calibrate_online(&refs, 8, |_| None), (0, 0));
            }
        }
        let id = index.insert_to_storage(v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    recall(&index, &data)
}

#[test]
fn online_calibration_matches_upfront_training() {
    for mode in [
        QuantizationMode::PQ4,
        QuantizationMode::ProductQuantization,
        QuantizationMode::RaBitQ,
        QuantizationMode::RaBitQ4,
        QuantizationMode::RaBitQ2,
    ] {
        let never = run(mode, &Calibration::Never);
        let upfront = run(mode, &Calibration::Upfront);
        let online = run(mode, &Calibration::OnlineAfter(300));
        assert!(
            online >= upfront - 0.05,
            "{mode:?}: online {online:.3} vs upfront {upfront:.3}"
        );
        assert!(
            online >= never - 0.02,
            "{mode:?}: online {online:.3} must not be worse than default codebook {never:.3}"
        );
    }
}

/// Extended RaBitQ (2 / 4 bits per dimension) ranks far better than the 1-bit code on the
/// same data, and 4 bits keep the true top-10 inside a 50-candidate rerank window.
#[test]
fn extended_rabitq_recall_grows_with_bits() {
    let one = run(QuantizationMode::RaBitQ, &Calibration::Upfront);
    let two = run(QuantizationMode::RaBitQ2, &Calibration::Upfront);
    let four = run(QuantizationMode::RaBitQ4, &Calibration::Upfront);
    assert!(two > one, "2-bit {two:.3} vs 1-bit {one:.3}");
    assert!(four >= two, "4-bit {four:.3} vs 2-bit {two:.3}");
    assert!(four >= 0.95, "4-bit top-10 within 50: {four:.3}");
}

#[test]
fn calibration_improves_pq4_over_default_codebook() {
    let never = run(QuantizationMode::PQ4, &Calibration::Never);
    let online = run(QuantizationMode::PQ4, &Calibration::OnlineAfter(300));
    assert!(
        online > never + 0.02,
        "PQ4 online calibration {online:.3} should beat the default codebook {never:.3}"
    );
}

#[test]
fn modes_without_training_never_need_calibration() {
    let dir = tempdir().unwrap();
    let store = Arc::new(VectorStore::new(dir.path(), DIM));
    let index = HnswIndex::<EuclideanMetric>::new(
        store,
        QuantizationMode::ScalarI8,
        Arc::new(GlobalConfig::default()),
        DIM,
    );
    assert!(!index.needs_calibration());
    assert!(index.is_calibrated());
    let sample = to_f32(&point(1));
    assert_eq!(index.calibrate_online(&[&sample], 4, |_| None), (0, 0));
}

#[test]
fn calibration_state_survives_snapshot_reload() {
    let mode = QuantizationMode::PQ4;
    let data: Vec<Vec<f64>> = (0..400).map(point).collect();
    let dir = tempdir().unwrap();
    let snap = dir.path().join("index.snap");
    {
        let index = new_index(dir.path(), mode);
        for v in &data {
            let id = index.insert_to_storage(v).unwrap();
            index.index_node(id, HashMap::new()).unwrap();
        }
        // An uncalibrated snapshot must reload as uncalibrated.
        index.save_snapshot(&snap).unwrap();
        let store = Arc::new(VectorStore::new(dir.path(), element_size(mode)));
        let reloaded = HnswIndex::<EuclideanMetric>::load_snapshot(
            &snap,
            store,
            mode,
            Arc::new(GlobalConfig::default()),
            DIM,
        )
        .unwrap();
        assert!(reloaded.needs_calibration());

        let f32s: Vec<Vec<f32>> = data.iter().map(|v| to_f32(v)).collect();
        let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
        index.calibrate_online(&refs, 8, |id| data.get(id as usize).map(|v| to_f32(v)));
        index.save_snapshot(&snap).unwrap();
    }
    let store = Arc::new(VectorStore::new(dir.path(), element_size(mode)));
    let reloaded = HnswIndex::<EuclideanMetric>::load_snapshot(
        &snap,
        store,
        mode,
        Arc::new(GlobalConfig::default()),
        DIM,
    )
    .unwrap();
    assert!(reloaded.is_calibrated());
    assert!(!reloaded.needs_calibration());
}

#[test]
fn concurrent_inserts_during_calibration_are_all_encoded_with_the_new_codebook() {
    let mode = QuantizationMode::PQ4;
    let n = 1500u32;
    let data: Arc<Vec<Vec<f64>>> = Arc::new((0..n).map(point).collect());
    let dir = tempdir().unwrap();
    let index = Arc::new(new_index(dir.path(), mode));
    for v in &data[..300] {
        let id = index.insert_to_storage(v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    std::thread::scope(|s| {
        let writer_index = Arc::clone(&index);
        let writer_data = Arc::clone(&data);
        s.spawn(move || {
            for v in &writer_data[300..] {
                let id = writer_index.insert_to_storage(v).unwrap();
                writer_index.index_node(id, HashMap::new()).unwrap();
            }
        });
        let f32s: Vec<Vec<f32>> = data[..300].iter().map(|v| to_f32(v)).collect();
        let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
        index.calibrate_online(&refs, 8, |id| data.get(id as usize).map(|v| to_f32(v)));
    });
    assert_eq!(index.count(), n as usize);
    // Same quality as a sequential online calibration: no vector kept a stale code.
    let r = recall(&index, &data);
    let sequential = run(mode, &Calibration::OnlineAfter(300));
    assert!(
        r >= sequential - 0.05,
        "concurrent {r:.3} vs sequential {sequential:.3}"
    );
}

#[test]
fn progressive_refit_tracks_growth_and_re_encodes_everything() {
    let mode = QuantizationMode::PQ4;
    let data: Vec<Vec<f64>> = (0..1200).map(point).collect();
    let dir = tempdir().unwrap();
    let index = new_index(dir.path(), mode);
    let raw = |id: u32| data.get(id as usize).map(|v| to_f32(v));
    for v in &data[..100] {
        let id = index.insert_to_storage(v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let f32s: Vec<Vec<f32>> = data.iter().map(|v| to_f32(v)).collect();
    let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
    assert_eq!(index.calibrate_online(&refs[..100], 8, raw), (100, 0));
    assert_eq!(index.calibrated_at(), 100);
    for v in &data[100..] {
        let id = index.insert_to_storage(v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    // calibrate_online is a no-op once calibrated; refit_online always re-fits.
    assert_eq!(index.calibrate_online(&refs, 8, raw), (0, 0));
    assert_eq!(index.refit_online(&refs, 8, raw), (1200, 0));
    assert_eq!(index.calibrated_at(), 1200);
    assert!(recall(&index, &data) >= 0.85);
}

/// A reloaded extended-RaBitQ index decodes its codes with the collection's bit width (the
/// centroid file does not record it) and returns the same results as before the reload.
#[test]
fn extended_rabitq_survives_snapshot_reload() {
    for mode in [QuantizationMode::RaBitQ4, QuantizationMode::RaBitQ2] {
        let data: Vec<Vec<f64>> = (0..600).map(point).collect();
        let dir = tempdir().unwrap();
        let snap = dir.path().join("index.snap");
        let params = SearchParams {
            top_k: 10,
            ef_search: 64,
            ..Default::default()
        };
        let queries: Vec<Vec<f64>> = (0..8).map(|i| point(5_000 + i)).collect();
        let before: Vec<Vec<u32>>;
        {
            let index = new_index(dir.path(), mode);
            let f32s: Vec<Vec<f32>> = data.iter().map(|v| to_f32(v)).collect();
            let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
            let mut index = index;
            index.train_pq(&refs, 8);
            for v in &data {
                let id = index.insert_to_storage(v).unwrap();
                index.index_node(id, HashMap::new()).unwrap();
            }
            before = queries
                .iter()
                .map(|q| {
                    index
                        .search(q, &HashMap::new(), &[], &params)
                        .into_iter()
                        .map(|r| r.0)
                        .collect()
                })
                .collect();
            index.save_snapshot(&snap).unwrap();
        }
        let store = Arc::new(VectorStore::new(dir.path(), element_size(mode)));
        let reloaded = HnswIndex::<EuclideanMetric>::load_snapshot(
            &snap,
            store,
            mode,
            Arc::new(GlobalConfig::default()),
            DIM,
        )
        .unwrap();
        assert_eq!(
            reloaded.rabitq.load().as_ref().map(|r| r.bits),
            mode.rabitq_bits(),
            "{mode:?} bit width after reload"
        );
        let after: Vec<Vec<u32>> = queries
            .iter()
            .map(|q| {
                reloaded
                    .search(q, &HashMap::new(), &[], &params)
                    .into_iter()
                    .map(|r| r.0)
                    .collect()
            })
            .collect();
        assert_eq!(before, after, "{mode:?}");
        // Decoded vectors stay close to the originals.
        let v = reloaded.get_vector(3).coords;
        let err: f64 = v
            .iter()
            .zip(&data[3])
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        let n: f64 = data[3].iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            err / n < 0.35,
            "{mode:?} reconstruction error {:.3}",
            err / n
        );
    }
}
