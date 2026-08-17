use hyperspace_core::{
    CosineMetric, EuclideanMetric, GlobalConfig, LorentzMetric, PoincareMetric, QuantizationMode,
    SearchParams,
};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_search_dimension_mismatch_returns_empty_without_panic() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store.bin");
    let dim = 4;
    let store = Arc::new(VectorStore::new(&store_path, dim * 8 + 8));
    let config = Arc::new(GlobalConfig::default());

    let index =
        HnswIndex::<EuclideanMetric>::new(store.clone(), QuantizationMode::None, config, dim);

    // Insert 1 valid node
    let valid_vec = [1.0, 2.0, 3.0, 4.0];
    let id = index.insert_to_storage(&valid_vec).unwrap();
    index.index_node(id, HashMap::new()).unwrap();

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 50,
        ..Default::default()
    };
    let empty_filter = HashMap::new();

    // Query with smaller dimension (2 instead of 4)
    let small_query = [1.0, 2.0];
    let results = index.search(&small_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Dimension mismatch (too small) should return empty results"
    );

    // Query with larger dimension (6 instead of 4)
    let large_query = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let results = index.search(&large_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Dimension mismatch (too large) should return empty results"
    );
}

#[test]
fn test_search_lorentz_invalid_query_returns_empty_without_panic() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store_lorentz.bin");
    let dim = 4;
    let store = Arc::new(VectorStore::new(&store_path, dim * 8 + 8));
    let config = Arc::new(GlobalConfig::default());

    let index = HnswIndex::<LorentzMetric>::new(store.clone(), QuantizationMode::None, config, dim);

    // Valid Lorentz point on upper sheet: t = sqrt(1 + 1^2 + 1^2 + 1^2) = sqrt(4) = 2.0
    let valid_vec = [2.0, 1.0, 1.0, 1.0];
    let id = index.insert_to_storage(&valid_vec).unwrap();
    index.index_node(id, HashMap::new()).unwrap();

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 50,
        ..Default::default()
    };
    let empty_filter = HashMap::new();

    // 1. All zeros vector (violates t >= 1.0)
    let zero_query = [0.0, 0.0, 0.0, 0.0];
    let results = index.search(&zero_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "All-zero Lorentz query must return empty results instead of panicking"
    );

    // 2. Negative t coordinate (lower sheet)
    let lower_sheet_query = [-2.0, 1.0, 1.0, 1.0];
    let results = index.search(&lower_sheet_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Lower sheet Lorentz query must return empty results instead of panicking"
    );

    // 3. Spacelike vector (t^2 < spatial_norm_sq)
    let spacelike_query = [0.5, 1.0, 1.0, 1.0];
    let results = index.search(&spacelike_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Spacelike Lorentz query must return empty results instead of panicking"
    );

    // 4. Valid query should succeed and return the indexed point
    let results = index.search(&valid_vec, &empty_filter, &[], &search_params);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
}

#[test]
fn test_search_poincare_out_of_bounds_returns_empty_without_panic() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store_poincare.bin");
    let dim = 3;
    let store = Arc::new(VectorStore::new(&store_path, dim * 8 + 8));
    let config = Arc::new(GlobalConfig::default());

    let index =
        HnswIndex::<PoincareMetric>::new(store.clone(), QuantizationMode::None, config, dim);

    // Valid Poincare point strictly inside the unit ball (norm < 1.0)
    let valid_vec = [0.2, 0.2, 0.2];
    let id = index.insert_to_storage(&valid_vec).unwrap();
    index.index_node(id, HashMap::new()).unwrap();

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 50,
        ..Default::default()
    };
    let empty_filter = HashMap::new();

    // 1. Point outside the unit ball (norm >= 1.0)
    let oob_query = [1.0, 1.0, 1.0];
    let results = index.search(&oob_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Out-of-bounds Poincare query must return empty results instead of panicking"
    );

    // 2. Valid query finds the point
    let results = index.search(&valid_vec, &empty_filter, &[], &search_params);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
}

#[test]
fn test_search_nan_inf_returns_empty_without_panic() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("store_nan.bin");
    let dim = 3;
    let store = Arc::new(VectorStore::new(&store_path, dim * 8 + 8));
    let config = Arc::new(GlobalConfig::default());

    let index = HnswIndex::<CosineMetric>::new(store.clone(), QuantizationMode::None, config, dim);

    let valid_vec = [0.577, 0.577, 0.577];
    let id = index.insert_to_storage(&valid_vec).unwrap();
    index.index_node(id, HashMap::new()).unwrap();

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 50,
        ..Default::default()
    };
    let empty_filter = HashMap::new();

    let nan_query = [f64::NAN, 0.5, 0.5];
    let results = index.search(&nan_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "NaN query must return empty results instead of panicking"
    );

    let inf_query = [f64::INFINITY, 0.5, 0.5];
    let results = index.search(&inf_query, &empty_filter, &[], &search_params);
    assert!(
        results.is_empty(),
        "Infinity query must return empty results instead of panicking"
    );
}
