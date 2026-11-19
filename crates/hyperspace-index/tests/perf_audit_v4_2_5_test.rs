//! Regression tests for the v4.2.5 performance & recall audit fixes.

use hyperspace_core::{
    DotProductMetric, EuclideanMetric, FilterExpr, GlobalConfig, PoincareMetric, QuantizationMode,
    SearchParams,
};
use hyperspace_index::{ordered_f64_key, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

fn params(top_k: usize) -> SearchParams {
    SearchParams {
        top_k,
        ef_search: 64,
        ..Default::default()
    }
}

#[test]
fn ordered_f64_key_preserves_numeric_order() {
    let values = [
        f64::NEG_INFINITY,
        -1e9,
        -2.5,
        -1.0,
        -1e-12,
        0.0,
        1e-12,
        0.5,
        2.5,
        1e9,
        f64::INFINITY,
    ];
    for w in values.windows(2) {
        assert!(
            ordered_f64_key(w[0]) < ordered_f64_key(w[1]),
            "{} vs {}",
            w[0],
            w[1]
        );
    }
    assert_eq!(ordered_f64_key(-0.0), ordered_f64_key(0.0));
}

#[test]
fn range_filter_matches_fractional_and_typed_values_without_scan() {
    let dir = tempdir().unwrap();
    let dim = 4;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 8 + 8));
    let index = HnswIndex::<EuclideanMetric>::new(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        dim,
    );

    let prices = [-3.5, 0.25, 1.5, 2.75, 10.0];
    for (i, price) in prices.iter().enumerate() {
        let v = [i as f64, 0.0, 0.0, 0.0];
        let id = index.insert_to_storage(&v).unwrap();
        let mut meta = HashMap::new();
        if i == 4 {
            // Typed numeric value (JSON payload) must be indexed under the plain key.
            meta.insert(
                "__hs_typed__price".to_string(),
                format!("{{\"v\":{price}}}"),
            );
        } else {
            meta.insert("price".to_string(), price.to_string());
        }
        index.index_node(id, meta).unwrap();
    }

    let range = |gte: Option<f64>, lte: Option<f64>| {
        let filters = [FilterExpr::Range {
            key: "price".into(),
            gte,
            lte,
        }];
        let mut ids: Vec<u32> = index
            .search(&[0.0; 4], &HashMap::new(), &filters, &params(10))
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        ids.sort_unstable();
        ids
    };

    assert_eq!(range(Some(0.2), Some(2.0)), vec![1, 2]);
    assert_eq!(range(Some(-4.0), Some(0.0)), vec![0]);
    assert_eq!(range(Some(2.75), None), vec![3, 4]);
    assert_eq!(range(None, Some(-3.5)), vec![0]);
    assert!(range(Some(100.0), None).is_empty());

    // Updating a value moves the id inside the numeric index.
    let mut patch = HashMap::new();
    patch.insert("price".to_string(), "50.5".to_string());
    index.update_payload_metadata(1, &patch).unwrap();
    assert_eq!(range(Some(0.2), Some(2.0)), vec![2]);
    assert_eq!(range(Some(50.0), Some(51.0)), vec![1]);

    // NOT(range) = all live nodes minus matches.
    let not = [FilterExpr::Not(Box::new(FilterExpr::Range {
        key: "price".into(),
        gte: Some(2.0),
        lte: None,
    }))];
    let mut ids: Vec<u32> = index
        .search(&[0.0; 4], &HashMap::new(), &not, &params(10))
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![0, 2]);
}

#[test]
fn dot_metric_ranks_negative_distances_correctly() {
    // `1 - dot` is negative for unnormalized vectors; candidate ordering must not clamp it.
    let dir = tempdir().unwrap();
    let dim = 4;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 8 + 8));
    let index = HnswIndex::<DotProductMetric>::new(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        dim,
    );
    for scale in [1.0, 3.0, 2.0, 5.0, 4.0] {
        let id = index.insert_to_storage(&[scale, 0.0, 0.0, 0.0]).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let res = index.search(&[1.0, 0.0, 0.0, 0.0], &HashMap::new(), &[], &params(3));
    let ids: Vec<u32> = res.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![3, 4, 1], "largest dot product first: {res:?}");
    assert!(res.windows(2).all(|w| w[0].1 <= w[1].1));
}

#[test]
fn poincare_near_neighbors_keep_distinct_distances() {
    // cosh(d) for d ~ 1e-4 is 1 + 5e-9: indistinguishable from 1.0 in f32 unless biased.
    let dir = tempdir().unwrap();
    let dim = 2;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 4 + 4));
    let index = HnswIndex::<PoincareMetric>::new_with_storage_precision(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        dim,
    );
    for x in [0.1003, 0.1001, 0.1002, 0.1004] {
        let id = index.insert_to_storage(&[x, 0.0]).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let res = index.search(&[0.1, 0.0], &HashMap::new(), &[], &params(4));
    let ids: Vec<u32> = res.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![1, 2, 0, 3], "{res:?}");
    assert!(res.windows(2).all(|w| w[0].1 < w[1].1), "{res:?}");
    assert!(res[0].1 > 0.0);
}

fn hybrid_point(seed: f64, dim: usize) -> Vec<f64> {
    let mut v = vec![0.0; dim];
    let mut norm = 0.0;
    for (i, x) in v.iter_mut().enumerate().take(33).skip(1) {
        *x = ((i as f64 + seed) * 0.37).sin() * 0.3;
        norm += *x * *x;
    }
    v[0] = (1.0 + norm).sqrt();
    for (i, x) in v.iter_mut().enumerate().skip(33) {
        *x = ((i as f64) * 0.13 + seed).cos();
    }
    v
}

#[test]
fn hybrid_squared_internal_distance_finalizes_to_metric_distance() {
    use hyperspace_core::{HybridMetric, Metric};
    let dir = tempdir().unwrap();
    let dim = 48;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 4 + 4));
    let index = HnswIndex::<HybridMetric>::new_with_storage_precision(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        dim,
    );
    let points: Vec<Vec<f64>> = (0..40)
        .map(|i| hybrid_point(f64::from(i) * 0.05, dim))
        .collect();
    for p in &points {
        let id = index.insert_to_storage(p).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let query = hybrid_point(0.42, dim);
    let res = index.search(&query, &HashMap::new(), &[], &params(10));
    assert_eq!(res.len(), 10);
    assert!(res.windows(2).all(|w| w[0].1 <= w[1].1));

    let mut exact: Vec<(u32, f64)> = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i as u32, HybridMetric::distance(p, &query)))
        .collect();
    exact.sort_by(|a, b| a.1.total_cmp(&b.1));
    assert_eq!(res[0].0, exact[0].0);
    for (id, d) in &res {
        let want = HybridMetric::distance(&points[*id as usize], &query);
        assert!(
            (d - want).abs() < 1e-3 * want.max(1.0),
            "id={id}: {d} vs {want}"
        );
    }
}

#[test]
fn repair_after_deletes_keeps_live_points_reachable() {
    let dir = tempdir().unwrap();
    let dim = 8;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 4 + 4));
    let index = HnswIndex::<EuclideanMetric>::new_with_storage_precision(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        dim,
    );
    let n = 600u32;
    let point = |i: u32| -> Vec<f64> {
        (0..dim)
            .map(|d| ((f64::from(i) * 0.731 + d as f64 * 1.37).sin()) * 10.0)
            .collect()
    };
    for i in 0..n {
        let id = index.insert_to_storage(&point(i)).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    // Delete two thirds of the graph, including likely hubs/entry point.
    for i in (0..n).filter(|i| i % 3 != 0) {
        index.delete(i);
    }
    assert!(index.repair_deleted() > 0);

    let mut found = 0;
    for i in (0..n).filter(|i| i % 3 == 0) {
        let res = index.search(&point(i), &HashMap::new(), &[], &params(1));
        if res.first().map(|r| r.0) == Some(i) {
            found += 1;
        }
        assert!(res.iter().all(|(id, _)| id % 3 == 0), "deleted id returned");
    }
    let live = (0..n).filter(|i| i % 3 == 0).count();
    assert!(
        found * 100 >= live * 98,
        "self-recall after repair: {found}/{live}"
    );
}

#[test]
fn acorn_and_patience_respect_filters() {
    let dir = tempdir().unwrap();
    let dim = 8;
    let store = Arc::new(VectorStore::new(dir.path(), dim * 4 + 4));
    let index = HnswIndex::<EuclideanMetric>::new_with_storage_precision(
        store,
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        dim,
    );
    // Large enough to bypass the filtered brute-force path (> 2000 nodes).
    let n = 4000u32;
    let point = |i: u32| -> Vec<f64> {
        (0..dim)
            .map(|d| ((f64::from(i) * 0.311 + d as f64 * 0.77).cos()) * 5.0)
            .collect()
    };
    for i in 0..n {
        let id = index.insert_to_storage(&point(i)).unwrap();
        let mut meta = HashMap::new();
        meta.insert("tenant".to_string(), (i % 10).to_string());
        index.index_node(id, meta).unwrap();
    }
    let mut filter = HashMap::new();
    filter.insert("tenant".to_string(), "3".to_string());
    let q = point(1233);
    let res = index.search(&q, &filter, &[], &params(10));
    assert_eq!(res.len(), 10);
    assert!(res.iter().all(|(id, _)| id % 10 == 3));
    assert_eq!(res[0].0, 1233);
}

fn code_mode_recall(mode: QuantizationMode) -> f64 {
    let dim = 64;
    let n = 1500u32;
    let element = match mode {
        QuantizationMode::RaBitQ => hyperspace_core::rabitq::code_size(dim),
        _ => hyperspace_core::pq4::code_size(hyperspace_core::pq4::default_num_subvectors(dim)),
    };
    let dir = tempdir().unwrap();
    let store = Arc::new(VectorStore::new(dir.path(), element));
    let mut index =
        HnswIndex::<EuclideanMetric>::new(store, mode, Arc::new(GlobalConfig::default()), dim);
    // Clustered pseudo-Gaussian data (embedding-like): 25 centers + isotropic noise.
    let gauss = |seed: u64| -> f64 {
        let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
        x ^= x >> 31;
        x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x ^= x >> 27;
        let u1 = ((x >> 11) as f64 / (1u64 << 53) as f64).max(1e-12);
        let mut y = x.wrapping_mul(0x94D0_49BB_1331_11EB);
        y ^= y >> 29;
        let u2 = (y >> 11) as f64 / (1u64 << 53) as f64;
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    };
    let point = |i: u32| -> Vec<f64> {
        let c = u64::from(i % 25);
        (0..dim)
            .map(|d| {
                let d = d as u64;
                2.0 + 3.0 * gauss(c * 1000 + d) + gauss(u64::from(i) * 7919 + d + 1_000_000)
            })
            .collect()
    };
    let data: Vec<Vec<f64>> = (0..n).map(point).collect();
    let f32s: Vec<Vec<f32>> = data
        .iter()
        .map(|v| v.iter().map(|&x| x as f32).collect())
        .collect();
    let refs: Vec<&[f32]> = f32s.iter().map(Vec::as_slice).collect();
    index.train_pq(&refs, 8);
    for v in &data {
        let id = index.insert_to_storage(v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let k = 10;
    let mut hit = 0usize;
    let queries = 30;
    for qi in 0..queries {
        let q = point(n + 7 * qi);
        let mut exact: Vec<(u32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    i as u32,
                    v.iter().zip(&q).map(|(a, b)| (a - b) * (a - b)).sum(),
                )
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
        // Recall@10 within a 50-candidate re-rank window (codes are re-ranked server-side).
        hit += exact[..k]
            .iter()
            .filter(|(id, _)| res.iter().any(|(r, _)| r == id))
            .count();
    }
    hit as f64 / (queries as usize * k) as f64
}

#[test]
fn rabitq_mode_end_to_end_recall() {
    let r = code_mode_recall(QuantizationMode::RaBitQ);
    assert!(r >= 0.85, "RaBitQ recall@10 (50-window): {r}");
}

#[test]
fn pq4_mode_end_to_end_recall() {
    let r = code_mode_recall(QuantizationMode::PQ4);
    assert!(r >= 0.85, "PQ4 recall@10 (50-window): {r}");
}
