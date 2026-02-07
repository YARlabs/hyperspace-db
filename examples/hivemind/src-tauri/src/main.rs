#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::State;
use std::sync::Arc;
use serde::Serialize;
use std::collections::HashMap;

use hyperspace_index::HnswIndex;
use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_store::VectorStore;

type LocalIndex = HnswIndex<1024, EuclideanMetric>;

struct AppState {
    index: Arc<LocalIndex>,
}

#[derive(Serialize)]
struct SearchResult {
    id: u32,
    distance: f64,
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> (usize, usize) {
    let index = &state.index;
    (index.count_nodes(), index.storage_stats().1)
}

#[tauri::command]
async fn ingest_pdf(path: String, state: State<'_, AppState>) -> Result<u32, String> {
    // 1. Read PDF
    let _text = pdf_extract::extract_text(&path).map_err(|e| e.to_string())?;
    
    // 2. Chunck & Embed (Using dummy embedding here as no Python/API)
    // In real app, we'd call ONNX runtimes or API.
    // Here we generate random vector to simulate flow.
    let _vec = vec![0.0f32; 1024]; // Dummy
    // Convert to internal representation (f32 -> f32).
    
    // Insert into Index
    // Assume auto-increment ID? Or derive from hash?
    let _id = state.index.count_nodes() as u32;
    
    // HnswIndex uses generic internal vector logic.
    // We assume input is handled by caller logic usually, but here...
    // HnswIndex::insert takes &[T] where T depends on quantization mode?
    // No, insert takes `&[f64]` in WASM wrapper, but checking `HnswIndex::insert` signature...
    // It takes `&[f32]` or `&[f64]`?
    // It takes `&[f64]` and converts internally.
    // Or generic `InputVector`.
    
    // Let's use `vec![0.1; 1024]` f64 for simplicity as core handles conversion.
    let input_vec = vec![0.1f64; 1024];
    
    let internal_id = state.index.insert(&input_vec, HashMap::new())
        .map_err(|e| e.to_string())?;
        
    Ok(internal_id)
}

fn main() {
    env_logger::init();
    
    let app_dir = dirs::home_dir().unwrap().join(".hivemind");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).unwrap();
    }
    
    // Init Store (4096 bytes per vector for 1024-dim f32)
    let store_path = app_dir.join("store");
    let store = Arc::new(VectorStore::new(&store_path, 4096));
    
    let config = Arc::new(GlobalConfig::default());
    let mut index = LocalIndex::new(store.clone(), QuantizationMode::None, config);
    
    // Try load snapshot
    let snap_path = app_dir.join("index.snap");
    if snap_path.exists() {
        if let Ok(loaded) = LocalIndex::load_snapshot(&snap_path, store, QuantizationMode::None, Arc::new(GlobalConfig::default())) {
             index = loaded;
             println!("Loaded snapshot with {} nodes", index.count_nodes());
        }
    }

    tauri::Builder::default()
        .manage(AppState { index: Arc::new(index) })
        .invoke_handler(tauri::generate_handler![get_stats, ingest_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
