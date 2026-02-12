use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, FilterExpr};
use hyperspace_core::vector::HyperVector;
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

const D: usize = 4;

fn arb_vector() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(-100.0..100.0, D)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_hnsw_insert_search_prop(
        vectors in proptest::collection::vec(arb_vector(), 20..50)
    ) {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("store.bin");
        let store = Arc::new(VectorStore::new(&store_path, std::mem::size_of::<HyperVector<D>>()));
        
        let config = Arc::new(GlobalConfig {
            ef_construction: 200.into(),
            ef_search: 200.into(),
            ..Default::default()
        });

        let index = HnswIndex::<D, EuclideanMetric>::new(
            store.clone(),
            QuantizationMode::None,
            config
        );

        // Insert
        for (i, vec_data) in vectors.iter().enumerate() {
            // Convert to fixed size array
            let coords: [f64; D] = vec_data.clone().try_into().expect("Vec len must be D");
            let hv = HyperVector::new_unchecked(coords);
            
            // Serialize struct to bytes (unsafe but needed for Zero-Copy simulation)
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    (&hv as *const HyperVector<D>) as *const u8,
                    std::mem::size_of::<HyperVector<D>>()
                )
            };
            
            let id = store.append(bytes).unwrap();
            assert_eq!(id, i as u32);
            
            // Verify storage
            let stored_bytes = store.get(id);
            let stored_hv = unsafe { &*(stored_bytes.as_ptr() as *const HyperVector<D>) };
            assert_eq!(stored_hv.coords, coords, "Vector storage mismatch at index {}", i);

            let meta = HashMap::new();
            index.index_node(id, meta).unwrap();
        }

        // Search for inserted vectors (Exact Recall Check)
        for (i, vec) in vectors.iter().enumerate() {
            let empty_filter = HashMap::new();
            // Use ef=200 to ensure we find it if it's there
            let results = index.search(vec, 1, 200, &empty_filter, &[], None, None);
            
            if let Some((_id, dist)) = results.first() {
                assert!(*dist < 1e-4, "Search for inserted vector {} failed. Dist: {}", i, dist);
            } else {
                 panic!("Inserted vector {} not found", i);
            }
        }
    }
}
