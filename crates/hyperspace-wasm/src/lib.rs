use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use rexie::{ObjectStore, Rexie, TransactionMode};

// Hardcoded dimension for MVP.
type MyIndex = HnswIndex<1024, EuclideanMetric>;

const DB_NAME: &str = "hyperspace_db";
const STORE_NAME: &str = "storage"; // Object Store name

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
        // 1024 dims * 4 bytes (f32) = 4096 bytes per vector
        let storage = Arc::new(VectorStore::new(std::path::Path::new("mem"), 1024 * 4));

        // For MVP, simple RAM store setup.
        let config = Arc::new(GlobalConfig::default());

        let index = MyIndex::new(storage, QuantizationMode::None, config);

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

        let mut id_map = self.id_map.write();
        let mut rev_map = self.rev_map.write();

        if id_map.contains_key(&id) {
            return Err(JsValue::from_str("Duplicate ID not supported"));
        }

        let internal_id = self
            .index
            .insert(&vector, HashMap::new())
            .map_err(|e| JsValue::from_str(&e))?;

        id_map.insert(id, internal_id);
        rev_map.insert(internal_id, id);

        Ok(())
    }

    pub fn search(&self, vector: Vec<f64>, k: usize) -> Result<JsValue, JsValue> {
        if vector.len() != 1024 {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        let results = self
            .index
            .search(&vector, k, 100, &HashMap::new(), &[], None, None);

        let rev_map = self.rev_map.read();

        let mapped: Vec<serde_json::Value> = results
            .iter()
            .map(|(internal_id, dist)| {
                let user_id = rev_map.get(internal_id).copied().unwrap_or(*internal_id);
                serde_json::json!({
                    "id": user_id,
                    "distance": dist
                })
            })
            .collect();

        Ok(serde_wasm_bindgen::to_value(&mapped)?)
    }

    /// Persist current state to IndexedDB
    pub async fn save(&self) -> Result<(), JsValue> {
        let rexie = Rexie::builder(DB_NAME)
            .version(1)
            .add_object_store(ObjectStore::new(STORE_NAME))
            .build()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let transaction = rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let store_os = transaction
            .store(STORE_NAME)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 1. Export Storage (Bytes)
        let vector_store = self.index.get_storage();
        let store_bytes = vector_store.as_ref().export();
        let store_js = serde_wasm_bindgen::to_value(&store_bytes)?;
        store_os
            .put(&store_js, Some(&JsValue::from_str("vectors")))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 2. Export Index (Bytes)
        let index_bytes = self
            .index
            .save_to_bytes()
            .map_err(|e| JsValue::from_str(&e))?;
        let index_js = serde_wasm_bindgen::to_value(&index_bytes)?;
        store_os
            .put(&index_js, Some(&JsValue::from_str("index")))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 3. Export ID Maps
        let id_map = self.id_map.read();
        let map_js = serde_wasm_bindgen::to_value(&*id_map)?;
        store_os
            .put(&map_js, Some(&JsValue::from_str("id_map")))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        transaction
            .done()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        log("Saved to IndexedDB");
        Ok(())
    }

    /// Load state from IndexedDB
    pub async fn load(&mut self) -> Result<bool, JsValue> {
        let rexie = Rexie::builder(DB_NAME)
            .version(1)
            .add_object_store(ObjectStore::new(STORE_NAME))
            .build()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let transaction = rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let store_os = transaction
            .store(STORE_NAME)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Retrieve Vectors
        let vectors_js = store_os
            .get(&JsValue::from_str("vectors"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if vectors_js.is_undefined() {
            return Ok(false);
        }

        let vectors_bytes: Vec<u8> = serde_wasm_bindgen::from_value(vectors_js)?;

        // Retrieve Index
        let index_js = store_os
            .get(&JsValue::from_str("index"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let index_bytes: Vec<u8> = serde_wasm_bindgen::from_value(index_js)?;

        // Retrieve ID Map
        let map_js = store_os
            .get(&JsValue::from_str("id_map"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let id_map_data: HashMap<u32, u32> = serde_wasm_bindgen::from_value(map_js)?;

        // Reconstruct
        // 1. Restore Store
        // Element size: for QuantizationMode::None (default in new), it is 1024 * 4 = 4096 bytes if f32.

        // But HnswIndex insert stores f32 by default.
        // I should fix the element_size in `new()` to be correct.
        // 1024 * size_of::<f32>() = 4096.
        let element_size = 4096;
        let storage = Arc::new(VectorStore::from_bytes(
            std::path::Path::new("mem"),
            element_size,
            &vectors_bytes,
        ));

        let config = Arc::new(GlobalConfig::default());

        // 2. Restore Index
        let index = MyIndex::load_from_bytes(&index_bytes, storage, QuantizationMode::None, config)
            .map_err(|e| JsValue::from_str(&e))?;

        // Update self
        self.index = Arc::new(index);

        // Update Maps
        let mut id_map = self.id_map.write();
        let mut rev_map = self.rev_map.write();
        *id_map = id_map_data.clone();
        rev_map.clear();
        for (k, v) in id_map_data {
            rev_map.insert(v, k);
        }

        log("Loaded from IndexedDB");
        Ok(true)
    }
}
