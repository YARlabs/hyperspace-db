use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use hyperspace_core::{CosineMetric, EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use rexie::{ObjectStore, Rexie, TransactionMode};

enum IndexWrapper {
    L2Dim384(Arc<HnswIndex<384, EuclideanMetric>>),
    CosineDim384(Arc<HnswIndex<384, CosineMetric>>),
    L2Dim768(Arc<HnswIndex<768, EuclideanMetric>>),
    CosineDim768(Arc<HnswIndex<768, CosineMetric>>),
    L2Dim1024(Arc<HnswIndex<1024, EuclideanMetric>>),
    CosineDim1024(Arc<HnswIndex<1024, CosineMetric>>),
    L2Dim1536(Arc<HnswIndex<1536, EuclideanMetric>>),
    CosineDim1536(Arc<HnswIndex<1536, CosineMetric>>),
}

const DB_NAME: &str = "hyperspace_db";
const STORE_NAME: &str = "storage"; // Object Store name

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct HyperspaceDB {
    index: IndexWrapper,
    // Mapping UserID -> InternalID
    id_map: RwLock<HashMap<u32, u32>>,
    // Reverse mapping InternalID -> UserID
    rev_map: RwLock<HashMap<u32, u32>>,
    dimension: usize,
}

#[wasm_bindgen]
impl HyperspaceDB {
    /// Creates a new `HyperspaceDB` instance.
    ///
    /// # Errors
    /// Returns an error if initialization fails.
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize, metric: String) -> Result<HyperspaceDB, JsValue> {
        console_error_panic_hook::set_once();

        // Use RAM implementation
        // Element size depends on dimension (Scalar f32 = 4 bytes)
        let element_size = dimension * 4;
        let storage = Arc::new(VectorStore::new(std::path::Path::new("mem"), element_size));
        let config = Arc::new(GlobalConfig::default());
        let mode = QuantizationMode::None;
        let metric = metric.to_lowercase();

        let index = match (dimension, metric.as_str()) {
             (384, "l2" | "euclidean") => IndexWrapper::L2Dim384(Arc::new(HnswIndex::new(storage, mode, config))),
             (384, "cosine") => IndexWrapper::CosineDim384(Arc::new(HnswIndex::new(storage, mode, config))),
             (768, "l2" | "euclidean") => IndexWrapper::L2Dim768(Arc::new(HnswIndex::new(storage, mode, config))),
             (768, "cosine") => IndexWrapper::CosineDim768(Arc::new(HnswIndex::new(storage, mode, config))),
             (1024, "l2" | "euclidean") => IndexWrapper::L2Dim1024(Arc::new(HnswIndex::new(storage, mode, config))),
             (1024, "cosine") => IndexWrapper::CosineDim1024(Arc::new(HnswIndex::new(storage, mode, config))),
             (1536, "l2" | "euclidean") => IndexWrapper::L2Dim1536(Arc::new(HnswIndex::new(storage, mode, config))),
             (1536, "cosine") => IndexWrapper::CosineDim1536(Arc::new(HnswIndex::new(storage, mode, config))),

             _ => return Err(JsValue::from_str(&format!("Unsupported config: dim={dimension}, metric={metric}. Supported dims: 384, 768, 1024, 1536"))),
        };

        Ok(Self {
            index,
            id_map: RwLock::new(HashMap::new()),
            rev_map: RwLock::new(HashMap::new()),
            dimension,
        })
    }

    /// Inserts a vector.
    ///
    /// # Errors
    /// Returns error on dimension mismatch or duplicate ID.
    pub fn insert(&self, id: u32, vector: &[f64]) -> Result<(), JsValue> {
        if vector.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Dimension mismatch: expected {}.",
                self.dimension
            )));
        }

        let mut id_map = self.id_map.write();
        let mut rev_map = self.rev_map.write();

        if id_map.contains_key(&id) {
            return Err(JsValue::from_str("Duplicate ID not supported"));
        }

        macro_rules! insert_impl {
            ($idx:expr) => {
                $idx.insert(vector, HashMap::new())
                    .map_err(|e| JsValue::from_str(&e))?
            };
        }

        let internal_id = match &self.index {
            IndexWrapper::L2Dim384(idx) => insert_impl!(idx),
            IndexWrapper::CosineDim384(idx) => insert_impl!(idx),
            IndexWrapper::L2Dim768(idx) => insert_impl!(idx),
            IndexWrapper::CosineDim768(idx) => insert_impl!(idx),
            IndexWrapper::L2Dim1024(idx) => insert_impl!(idx),
            IndexWrapper::CosineDim1024(idx) => insert_impl!(idx),
            IndexWrapper::L2Dim1536(idx) => insert_impl!(idx),
            IndexWrapper::CosineDim1536(idx) => insert_impl!(idx),
        };

        id_map.insert(id, internal_id);
        rev_map.insert(internal_id, id);

        Ok(())
    }

    /// Searches for nearest neighbors.
    ///
    /// # Errors
    /// Returns error on dimension mismatch.
    pub fn search(&self, vector: &[f64], k: usize) -> Result<JsValue, JsValue> {
        if vector.len() != self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        macro_rules! search_impl {
            ($idx:expr) => {
                $idx.search(vector, k, 100, &HashMap::new(), &[], None, None)
            };
        }

        let results = match &self.index {
            IndexWrapper::L2Dim384(idx) => search_impl!(idx),
            IndexWrapper::CosineDim384(idx) => search_impl!(idx),
            IndexWrapper::L2Dim768(idx) => search_impl!(idx),
            IndexWrapper::CosineDim768(idx) => search_impl!(idx),
            IndexWrapper::L2Dim1024(idx) => search_impl!(idx),
            IndexWrapper::CosineDim1024(idx) => search_impl!(idx),
            IndexWrapper::L2Dim1536(idx) => search_impl!(idx),
            IndexWrapper::CosineDim1536(idx) => search_impl!(idx),
        };

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

    /// Persist current state to `IndexedDB`.
    ///
    /// # Errors
    /// Returns error if `IndexedDB` operations fail.
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
        let db_store = transaction
            .store(STORE_NAME)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 1. Export Storage (Bytes)
        let vector_store = match &self.index {
            IndexWrapper::L2Dim384(idx) => idx.get_storage(),
            IndexWrapper::CosineDim384(idx) => idx.get_storage(),
            IndexWrapper::L2Dim768(idx) => idx.get_storage(),
            IndexWrapper::CosineDim768(idx) => idx.get_storage(),
            IndexWrapper::L2Dim1024(idx) => idx.get_storage(),
            IndexWrapper::CosineDim1024(idx) => idx.get_storage(),
            IndexWrapper::L2Dim1536(idx) => idx.get_storage(),
            IndexWrapper::CosineDim1536(idx) => idx.get_storage(),
        };

        let store_bytes = vector_store.as_ref().export();
        let store_js = serde_wasm_bindgen::to_value(&store_bytes)?;
        db_store
            .put(&store_js, Some(&JsValue::from_str("vectors")))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 2. Export Index (Bytes)
        macro_rules! save_impl {
            ($idx:expr) => {
                $idx.save_to_bytes().map_err(|e| JsValue::from_str(&e))?
            };
        }

        let index_bytes = match &self.index {
            IndexWrapper::L2Dim384(idx) => save_impl!(idx),
            IndexWrapper::CosineDim384(idx) => save_impl!(idx),
            IndexWrapper::L2Dim768(idx) => save_impl!(idx),
            IndexWrapper::CosineDim768(idx) => save_impl!(idx),
            IndexWrapper::L2Dim1024(idx) => save_impl!(idx),
            IndexWrapper::CosineDim1024(idx) => save_impl!(idx),
            IndexWrapper::L2Dim1536(idx) => save_impl!(idx),
            IndexWrapper::CosineDim1536(idx) => save_impl!(idx),
        };
        let index_js = serde_wasm_bindgen::to_value(&index_bytes)?;
        db_store
            .put(&index_js, Some(&JsValue::from_str("index")))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // 3. Export ID Maps
        // Important: Serialize *before* awaiting to drop the lock!
        let map_js = {
            let id_map = self.id_map.read();
            serde_wasm_bindgen::to_value(&*id_map)?
        };

        db_store
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

    /// Load state from `IndexedDB`.
    ///
    /// # Errors
    /// Returns error if `IndexedDB` operations fail.
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
        let db_store = transaction
            .store(STORE_NAME)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Retrieve Vectors
        let vectors_js = db_store
            .get(&JsValue::from_str("vectors"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if vectors_js.is_undefined() {
            return Ok(false);
        }

        let vectors_bytes: Vec<u8> = serde_wasm_bindgen::from_value(vectors_js)?;

        // Retrieve Index
        let index_js = db_store
            .get(&JsValue::from_str("index"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let index_bytes: Vec<u8> = serde_wasm_bindgen::from_value(index_js)?;

        // Retrieve ID Map
        let map_js = db_store
            .get(&JsValue::from_str("id_map"))
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let id_map_data: HashMap<u32, u32> = serde_wasm_bindgen::from_value(map_js)?;

        // Reconstruct
        let element_size = self.dimension * 4;
        let storage = Arc::new(VectorStore::from_bytes(
            std::path::Path::new("mem"),
            element_size,
            &vectors_bytes,
        ));

        let config = Arc::new(GlobalConfig::default());
        let mode = QuantizationMode::None;

        // 2. Restore Index
        // We match on self.index to determine which type to load into
        let new_index_wrapper = match &self.index {
            IndexWrapper::L2Dim384(_) => IndexWrapper::L2Dim384(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::CosineDim384(_) => IndexWrapper::CosineDim384(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::L2Dim768(_) => IndexWrapper::L2Dim768(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::CosineDim768(_) => IndexWrapper::CosineDim768(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::L2Dim1024(_) => IndexWrapper::L2Dim1024(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::CosineDim1024(_) => IndexWrapper::CosineDim1024(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::L2Dim1536(_) => IndexWrapper::L2Dim1536(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
            IndexWrapper::CosineDim1536(_) => IndexWrapper::CosineDim1536(Arc::new(
                HnswIndex::load_from_bytes(&index_bytes, storage, mode, config)
                    .map_err(|e| JsValue::from_str(&e))?,
            )),
        };

        // Update self
        self.index = new_index_wrapper;

        // Update Maps
        let mut id_map = self.id_map.write();
        let mut rev_map = self.rev_map.write();

        id_map.clone_from(&id_map_data);

        rev_map.clear();
        for (k, v) in id_map_data {
            rev_map.insert(v, k);
        }

        log("Loaded from IndexedDB");
        Ok(true)
    }
}
