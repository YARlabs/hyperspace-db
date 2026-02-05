use wasm_bindgen::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

use hyperspace_core::{GlobalConfig, EuclideanMetric, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;

// Hardcoded dimension for MVP.
type MyIndex = HnswIndex<1024, EuclideanMetric>;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct HyperspaceDB {
    index: Arc<MyIndex>,
    // Mapping UserID -> InternalID
    id_map: RwLock<HashMap<u32, u32>>,
    // Reverse mapping InternalID -> UserID
    rev_map: RwLock<HashMap<u32, u32>>,
}

#[wasm_bindgen]
impl HyperspaceDB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<HyperspaceDB, JsValue> {
        console_error_panic_hook::set_once();
        
        // Use RAM implementation (via default-features = false in Cargo.toml deps)
        let storage = Arc::new(VectorStore::new(std::path::Path::new("mem"), 1024 * 8));
        let config = Arc::new(GlobalConfig::default());
        
        let index = MyIndex::new(
            storage,
            QuantizationMode::None,
            config
        );

        Ok(Self {
            index: Arc::new(index),
            id_map: RwLock::new(HashMap::new()),
            rev_map: RwLock::new(HashMap::new()),
        })
    }

    pub fn insert(&self, id: u32, vector: Vec<f64>) -> Result<(), JsValue> {
        if vector.len() != 1024 {
            return Err(JsValue::from_str("Dimension mismatch: expected 1024."));
        }
        
        // Lock both maps
        let mut id_map = self.id_map.write();
        let mut rev_map = self.rev_map.write();

        if id_map.contains_key(&id) {
             return Err(JsValue::from_str("Duplicate ID not supported in WASM MVP"));
        }

        // Insert to storage and index
        let internal_id = self.index.insert(&vector, HashMap::new())
             .map_err(|e| JsValue::from_str(&e))?;
             
        id_map.insert(id, internal_id);
        rev_map.insert(internal_id, id);
        
        Ok(())
    }

    pub fn search(&self, vector: Vec<f64>, k: usize) -> Result<JsValue, JsValue> {
         if vector.len() != 1024 {
            return Err(JsValue::from_str("Dimension mismatch"));
        }
        
        let results = self.index.search(
            &vector,
            k,
            100,
            &HashMap::new(),
            &[],
            None, None
        );
        
        let rev_map = self.rev_map.read();
        
        // Map to JS array of objects { id, distance }
        let mapped: Vec<serde_json::Value> = results.iter().map(|(internal_id, dist)| {
             let user_id = rev_map.get(internal_id).copied().unwrap_or(*internal_id);
             serde_json::json!({
                 "id": user_id,
                 "distance": dist
             })
        }).collect();
        
        Ok(serde_wasm_bindgen::to_value(&mapped)?)
    }
}
