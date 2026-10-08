use crate::manager::CollectionMetadata;
use hyperspace_core::{
    Collection, CosineMetric, EuclideanMetric, HybridMetric, LorentzMetric, PoincareMetric,
};
use hyperspace_proto::hyperspace::CollectionSchema;
use hyperspace_store::VectorStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationManifest {
    pub collection_name: String,
    pub source_path: String,
    pub target_path: String,
    pub vector_count: usize,
    pub timestamp_epoch_secs: u64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub collection_name: String,
    pub vector_count: usize,
    pub duration_ms: u128,
}

/// HyperspaceDB Automatic Migration Engine (v3.x -> v4.0.0).
///
/// On server boot, inspects legacy drop folders (`data/legacy_v3/`, `data/legacy/`)
/// and unmigrated root directories, converts .hyp and wal.log files to v4 sharded Direct I/O layout,
/// and archives processed legacy files to `data/legacy_v3_migrated/`.
///
/// Guarantees:
/// 1. Strict idempotency: already migrated collections are NEVER re-migrated, protecting new writes.
/// 2. Lossless data preservation: utilizes WAL log when available for full f64 fidelity and metadata,
///    with robust fallback to dequantizing chunk_0.hyp.
/// 3. Verified persistence: ensures all shards flush WriteBuffers and save index.snap snapshots before completion.
pub struct MigrationEngine;

impl MigrationEngine {
    /// Discovers all legacy collections and migrates them to v4 format.
    /// By default migrates from `data_dir` into `data_dir/v4`.
    pub async fn run_startup_migration(data_dir: &Path) -> Result<Vec<MigrationReport>, String> {
        let v4_dir = data_dir.join("v4");
        Self::run_startup_migration_with_legacy(&v4_dir, data_dir).await
    }

    /// Migrates collections from legacy directory (`legacy_v3_dir`, e.g. `./data`)
    /// into target v4 storage (`v4_data_dir`, e.g. `./data_v4`).
    pub async fn run_startup_migration_with_legacy(
        v4_data_dir: &Path,
        legacy_v3_dir: &Path,
    ) -> Result<Vec<MigrationReport>, String> {
        let legacy_sub_dir = legacy_v3_dir.join("legacy_v3");
        let legacy_alt_dir = legacy_v3_dir.join("legacy");
        let v4_legacy_sub = v4_data_dir.join("legacy_v3");
        let migrated_archive_dir = legacy_v3_dir.join("legacy_v3_migrated");

        fs::create_dir_all(v4_data_dir).map_err(|e| e.to_string())?;

        let is_already_migrated = |col_name: &str| -> bool {
            let target = v4_data_dir.join(col_name);
            // 1. Explicit completion manifest in v4 directory
            if target.join(".migration_v4_complete.json").exists() {
                return true;
            }
            // 2. Collection exists with v4 shards and non-empty index snapshots
            if target.join("meta.json").exists() {
                let shard0_snap = target.join("shard_0").join("index.snap");
                if shard0_snap.exists() {
                    if let Ok(meta) = fs::metadata(&shard0_snap) {
                        if meta.len() > 56 {
                            // 56 bytes is empty snapshot header
                            return true;
                        }
                    }
                }
            }
            // 3. Collection is already archived in legacy_v3_migrated
            if migrated_archive_dir.join(col_name).exists() {
                return true;
            }
            false
        };

        let mut candidate_dirs: Vec<(String, PathBuf)> = Vec::new();

        // 1. Check legacy drop directories (legacy_v3, legacy)
        for drop_dir in &[&legacy_sub_dir, &legacy_alt_dir, &v4_legacy_sub] {
            if drop_dir.exists() {
                if let Ok(entries) = fs::read_dir(drop_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && Self::is_v3_collection_dir(&path) {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if !is_already_migrated(name) {
                                    candidate_dirs.push((name.to_string(), path));
                                } else {
                                    println!("⏭️ [MigrationEngine] Collection '{name}' already migrated to v4. Skipping to protect data.");
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Check root legacy_v3_dir for unmigrated v3 collections (e.g. data/)
        if legacy_v3_dir.exists() {
            if let Ok(entries) = fs::read_dir(legacy_v3_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default();
                        if dir_name != "v4"
                            && dir_name != "legacy_v3"
                            && dir_name != "legacy"
                            && dir_name != "legacy_v3_migrated"
                            && dir_name != "lost+found"
                            && dir_name != "playbooks"
                            && !dir_name.starts_with('.')
                            && Self::is_v3_collection_dir(&path)
                        {
                            if !is_already_migrated(dir_name) {
                                candidate_dirs.push((dir_name.to_string(), path));
                            } else {
                                println!("⏭️ [MigrationEngine] Collection '{dir_name}' already migrated to v4. Skipping to protect data.");
                            }
                        }
                    }
                }
            }
        }

        if candidate_dirs.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "🔄 [MigrationEngine] Found {} legacy v3.x collection(s) to migrate to v4.0.0 (from {} to {})",
            candidate_dirs.len(),
            legacy_v3_dir.display(),
            v4_data_dir.display()
        );

        let mut reports = Vec::new();

        for (col_name, source_path) in candidate_dirs {
            let start = Instant::now();
            println!(
                "🚀 [MigrationEngine] Migrating '{col_name}' from {} to v4 format in {}...",
                source_path.display(),
                v4_data_dir.display()
            );

            let target_col_dir = v4_data_dir.join(&col_name);
            let in_place = source_path == target_col_dir;
            let actual_source = if in_place {
                let staging_parent = v4_data_dir.join(".migrating_v3");
                let _ = fs::create_dir_all(&staging_parent);
                let staging_dir = staging_parent.join(&col_name);
                if staging_dir.exists() {
                    let _ = fs::remove_dir_all(&staging_dir);
                }
                if let Err(e) = fs::rename(&source_path, &staging_dir) {
                    eprintln!("❌ [MigrationEngine] Failed to stage legacy directory '{col_name}' for in-place migration: {e}");
                    continue;
                }
                staging_dir
            } else {
                // If target directory previously had a broken/partial attempt, clean it up before retrying
                if target_col_dir.exists() {
                    let _ = fs::remove_dir_all(&target_col_dir);
                }
                source_path.clone()
            };

            match Self::migrate_single_collection(&actual_source, &target_col_dir, &col_name).await
            {
                Ok(vector_count) => {
                    let duration = start.elapsed().as_millis();
                    println!(
                        "✅ [MigrationEngine] Successfully migrated '{col_name}' ({vector_count} vectors in {duration} ms)"
                    );

                    // Create manifest in target v4 directory
                    let manifest = MigrationManifest {
                        collection_name: col_name.clone(),
                        source_path: source_path.display().to_string(),
                        target_path: target_col_dir.display().to_string(),
                        vector_count,
                        timestamp_epoch_secs: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        status: "COMPLETED".to_string(),
                    };
                    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap_or_default();
                    let _ = fs::write(
                        target_col_dir.join(".migration_v4_complete.json"),
                        manifest_json,
                    );

                    // Archive source files to legacy_v3_migrated/{col_name}
                    let _ = fs::create_dir_all(&migrated_archive_dir);
                    let archive_target = migrated_archive_dir.join(&col_name);
                    Self::archive_source_directory(&actual_source, &archive_target);

                    reports.push(MigrationReport {
                        collection_name: col_name,
                        vector_count,
                        duration_ms: duration,
                    });
                }
                Err(e) => {
                    eprintln!("❌ [MigrationEngine] Failed to migrate '{col_name}': {e}. Cleaning up partial target directory.");
                    let _ = fs::remove_dir_all(&target_col_dir);
                    if in_place {
                        let _ = fs::rename(&actual_source, &source_path);
                    }
                }
            }
        }

        // Clean up temporary staging directory if empty
        let staging_parent = v4_data_dir.join(".migrating_v3");
        if staging_parent.exists() {
            let _ = fs::remove_dir(&staging_parent);
        }

        Ok(reports)
    }

    /// Checks if a directory contains v3.x collection markers (meta.json and/or chunk_*.hyp).
    pub fn is_v3_collection_dir(path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }
        if path.join(".legacy_migrated").exists()
            || path.join(".migration_v4_complete.json").exists()
        {
            return false;
        }
        // If it contains shard subdirectories (v4 architecture), it is NOT a v3 collection
        if path.join("shard_0").is_dir() {
            return false;
        }
        let meta_file = path.join("meta.json");
        let has_meta = meta_file.exists();

        // Check for chunk_*.hyp or payloads.hyp or wal.log
        let mut has_hyp_files = false;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".hyp") || name == "wal.log" {
                    has_hyp_files = true;
                    break;
                }
            }
        }

        has_meta || has_hyp_files
    }

    /// Robust archiving of legacy collection to prevent duplicate migrations.
    fn archive_source_directory(source: &Path, target: &Path) {
        if target.exists() {
            let _ = fs::remove_dir_all(target);
        }
        if let Err(e) = fs::rename(source, target) {
            eprintln!(
                "⚠️ [MigrationEngine] fs::rename failed ({e}), attempting fallback copy + remove..."
            );
            if Self::copy_dir_all(source, target).is_ok() {
                let _ = fs::remove_dir_all(source);
                println!(
                    "📦 [MigrationEngine] Archived legacy files to {}",
                    target.display()
                );
            } else {
                // If even copy failed, place a .legacy_migrated sentinel so it is never rescanned
                let _ = fs::write(source.join(".legacy_migrated"), "MIGRATED_TO_V4");
                eprintln!(
                    "⚠️ [MigrationEngine] Placed .legacy_migrated sentinel in {}",
                    source.display()
                );
            }
        } else {
            println!(
                "📦 [MigrationEngine] Archived legacy files to {}",
                target.display()
            );
        }
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                Self::copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    /// Migrates a single v3 collection directory to v4.
    async fn migrate_single_collection(
        source_dir: &Path,
        target_dir: &Path,
        col_name: &str,
    ) -> Result<usize, String> {
        fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

        // 1. Load or synthesize metadata
        let mut meta = if let Ok(m) = CollectionMetadata::load(source_dir) {
            m
        } else {
            CollectionMetadata {
                dimension: Some(128),
                metric: Some("euclidean".to_string()),
                quantization: "none".to_string(),
                sharded: true,
                schema: Some(CollectionSchema {
                    components: vec![hyperspace_proto::hyperspace::VectorComponent {
                        name: "default".to_string(),
                        metric: "euclidean".to_string(),
                        full_dimension: 128,
                        weight: 1.0,
                    }],
                    cascade_pipeline: vec![],
                }),
            }
        };

        // Target collection in v4 is always sharded
        meta.sharded = true;
        meta.save(target_dir).map_err(|e| e.to_string())?;

        let dim = meta.dimension() as usize;
        let metric_str = meta.metric_name().to_lowercase();
        let quant_mode = meta.quantization_mode();
        let schema = meta.get_schema();

        // 2. Initialize v4 ShardedCollection
        let (rep_tx, _) = tokio::sync::broadcast::channel(128);
        let node_id = "v4_migrator".to_string();

        macro_rules! migrate_with_metric {
            ($M:ty) => {{
                let v4_col = crate::sharded_engine::ShardedCollection::<$M>::new(
                    col_name.to_string(),
                    node_id,
                    target_dir.to_path_buf(),
                    quant_mode,
                    rep_tx,
                    dim,
                    schema,
                    None,
                )
                .await?;

                let mut migrated = 0;

                // Priority 1: Ingest via WAL log if present (Lossless f64 fidelity + full metadata + exact clocks)
                let wal_path = source_dir.join("wal.log");
                let mut wal_entries = Vec::new();
                if wal_path.exists() && fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0) > 0 {
                    let _ = hyperspace_store::wal::Wal::replay(&wal_path, |entry| {
                        wal_entries.push(entry);
                    });
                }

                if !wal_entries.is_empty() {
                    for entry in wal_entries {
                        match entry {
                            hyperspace_store::wal::WalEntry::Insert {
                                id,
                                mut vector,
                                metadata,
                                logical_clock,
                            } => {
                                // Sanity check vector length and finite values
                                if vector.len() < dim {
                                    vector.resize(dim, 0.0);
                                } else if vector.len() > dim {
                                    vector.truncate(dim);
                                }
                                for val in &mut vector {
                                    if !val.is_finite() {
                                        *val = 0.0;
                                    }
                                }

                                v4_col
                                    .insert(
                                        &vector,
                                        id,
                                        metadata,
                                        logical_clock,
                                        hyperspace_core::Durability::Default,
                                    )
                                    .await?;
                                migrated += 1;
                            }
                        }
                    }
                } else {
                    // Priority 2: Fallback to reading chunk_*.hyp and state.json
                    let element_size = Self::calculate_element_size(quant_mode, dim);
                    let legacy_store = VectorStore::new(source_dir, element_size);
                    let mut total_vectors = legacy_store.count();

                    let mut reverse_id_map: HashMap<u32, u32> = HashMap::new();
                    if let Ok(s) = fs::read_to_string(source_dir.join("state.json")) {
                        if let Ok(st) = serde_json::from_str::<serde_json::Value>(&s) {
                            if let Some(rev) = st.get("reverse_id_map").and_then(|m| m.as_object()) {
                                for (k, v) in rev {
                                    if let (Ok(int_id), Some(ext_id)) = (k.parse::<u32>(), v.as_u64()) {
                                        reverse_id_map.insert(int_id, ext_id as u32);
                                    }
                                }
                            }
                            if let Some(cnt) = st.get("id_map").and_then(|m| m.as_object()).map(|m| m.len()) {
                                total_vectors = total_vectors.max(cnt);
                            }
                        }
                    }

                    if total_vectors == 0 && element_size > 0 {
                        let chunk_0 = source_dir.join("chunk_0.hyp");
                        if chunk_0.exists() {
                            if let Ok(file_bytes) = fs::read(&chunk_0) {
                                if let Some(last_pos) = file_bytes.iter().rposition(|&b| b != 0) {
                                    total_vectors = (last_pos / element_size) + 1;
                                }
                            }
                        }
                    }

                    let legacy_payload_path = source_dir.join("payloads.hyp");
                    let payload_store = if legacy_payload_path.exists() {
                        hyperspace_store::PayloadStore::open(source_dir, 3).ok().map(Arc::new)
                    } else {
                        None
                    };

                    for i in 0..total_vectors {
                        let internal_id = i as u32;
                        let external_id = reverse_id_map.get(&internal_id).copied().unwrap_or(internal_id);
                        let raw_bytes = legacy_store.get(internal_id);

                        let coords = Self::decode_legacy_vector(raw_bytes, quant_mode, dim, &metric_str);

                        let mut meta_map = HashMap::new();
                        if let Some(ref p_store) = payload_store {
                            if let Ok(Some(bytes)) = p_store.fetch_blocking(internal_id) {
                                if let Ok(text) = String::from_utf8(bytes) {
                                    meta_map.insert("payload".to_string(), text);
                                }
                            }
                        }

                        v4_col
                            .insert(
                                &coords,
                                external_id,
                                meta_map,
                                (i as u64) + 1,
                                hyperspace_core::Durability::Default,
                            )
                            .await?;
                        migrated += 1;
                    }
                }

                // 3. Guarantee all asynchronous background indexing drains and persists snapshot to disk
                v4_col.flush_and_snapshot().await?;
                migrated
            }};
        }

        let migrated_count = match metric_str.as_str() {
            "poincare" => migrate_with_metric!(PoincareMetric),
            "cosine" => migrate_with_metric!(CosineMetric),
            "lorentz" => migrate_with_metric!(LorentzMetric),
            "hybrid" => migrate_with_metric!(HybridMetric),
            _ => migrate_with_metric!(EuclideanMetric),
        };

        Ok(migrated_count)
    }

    /// Calculates legacy element size in bytes matching v3 storage format.
    pub fn calculate_element_size(
        quant_mode: hyperspace_core::QuantizationMode,
        dim: usize,
    ) -> usize {
        match quant_mode {
            hyperspace_core::QuantizationMode::ScalarI8 => dim + 4,
            hyperspace_core::QuantizationMode::Binary => dim.div_ceil(8) + 4,
            hyperspace_core::QuantizationMode::AsymmetricHybrid801 => {
                if dim > 33 {
                    33 * 4 + (dim - 33) + 4
                } else {
                    dim * 4 + 4
                }
            }
            hyperspace_core::QuantizationMode::AsymmetricHybridLowBit => {
                if dim > 33 {
                    let euc_dim = dim - 33;
                    let num_blocks = euc_dim.div_ceil(16);
                    33 * 4 + 4 + num_blocks * 12
                } else {
                    dim * 4 + 4
                }
            }
            hyperspace_core::QuantizationMode::AsymmetricHybridExtreme => {
                if dim > 33 {
                    let euc_dim = dim - 33;
                    33 * 4 + 4 + euc_dim.div_ceil(8)
                } else {
                    dim * 4 + 4
                }
            }
            hyperspace_core::QuantizationMode::ScalarI4 => {
                let head_dim = if dim == 801 { 33 } else { 0 };
                let tail_dim = dim.saturating_sub(head_dim);
                head_dim * 4 + 4 + tail_dim.div_ceil(16) * 12
            }
            hyperspace_core::QuantizationMode::Turbo => {
                let head_dim = if dim == 801 { 33 } else { 0 };
                let tail_dim = dim.saturating_sub(head_dim);
                head_dim * 4 + 4 + tail_dim.div_ceil(2) + 4
            }
            hyperspace_core::QuantizationMode::ProductQuantization
            | hyperspace_core::QuantizationMode::OPQ => {
                hyperspace_core::pq::default_num_subvectors(dim)
            }
            hyperspace_core::QuantizationMode::None => dim * 8,
        }
    }

    /// Decodes raw quantized bytes into finite float coordinates.
    pub fn decode_legacy_vector(
        raw_bytes: &[u8],
        quant_mode: hyperspace_core::QuantizationMode,
        dim: usize,
        metric_str: &str,
    ) -> Vec<f64> {
        let mut coords = match quant_mode {
            hyperspace_core::QuantizationMode::AsymmetricHybridLowBit => {
                let lorentz_dim = 33.min(dim);
                let mut vec = Vec::with_capacity(dim);
                if raw_bytes.len() < lorentz_dim * 4 + 4 {
                    vec.resize(dim, 0.0);
                    return vec;
                }
                for i in 0..lorentz_dim {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(&raw_bytes[i * 4..(i + 1) * 4]);
                    vec.push(f32::from_le_bytes(buf) as f64);
                }
                let mut sc_bytes = [0u8; 4];
                sc_bytes.copy_from_slice(&raw_bytes[lorentz_dim * 4..lorentz_dim * 4 + 4]);
                let scales_count = u32::from_le_bytes(sc_bytes) as usize;
                let scales_offset = lorentz_dim * 4 + 4;
                let mut scales = Vec::with_capacity(scales_count);
                for i in 0..scales_count {
                    let offset = scales_offset + i * 4;
                    if offset + 4 <= raw_bytes.len() {
                        let mut buf = [0u8; 4];
                        buf.copy_from_slice(&raw_bytes[offset..offset + 4]);
                        scales.push(f32::from_le_bytes(buf));
                    } else {
                        scales.push(1.0);
                    }
                }
                let packed_offset = scales_offset + scales_count * 4;
                let euclidean_packed = if packed_offset <= raw_bytes.len() {
                    &raw_bytes[packed_offset..]
                } else {
                    &[]
                };
                for b in 0..scales_count {
                    let scale = scales.get(b).copied().unwrap_or(1.0) as f64;
                    for i in 0..16 {
                        let pair_idx = i / 2;
                        let is_second = i % 2 == 1;
                        let byte_offset = b * 8 + pair_idx;
                        let byte = euclidean_packed.get(byte_offset).copied().unwrap_or(0);
                        let u_val = if is_second { byte & 0x0F } else { byte >> 4 };
                        let val = (u_val as i8) - 8;
                        vec.push(val as f64 * scale);
                        if vec.len() >= dim {
                            break;
                        }
                    }
                    if vec.len() >= dim {
                        break;
                    }
                }
                vec.resize(dim, 0.0);
                vec
            }
            hyperspace_core::QuantizationMode::AsymmetricHybrid801 => {
                let lorentz_dim = 33.min(dim);
                let mut vec = Vec::with_capacity(dim);
                if raw_bytes.len() < lorentz_dim * 4 + 4 {
                    vec.resize(dim, 0.0);
                    return vec;
                }
                for i in 0..lorentz_dim {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(&raw_bytes[i * 4..(i + 1) * 4]);
                    vec.push(f32::from_le_bytes(buf) as f64);
                }
                let euc_bytes = &raw_bytes[lorentz_dim * 4 + 4..];
                for &b in euc_bytes {
                    let val = (b as i8) as f64 / 127.0;
                    vec.push(val);
                    if vec.len() >= dim {
                        break;
                    }
                }
                vec.resize(dim, 0.0);
                vec
            }
            hyperspace_core::QuantizationMode::ScalarI8 => {
                let mut vec = Vec::with_capacity(dim);
                let coords_len = if raw_bytes.len() >= 4 {
                    raw_bytes.len() - 4
                } else {
                    raw_bytes.len()
                };
                let alpha = if raw_bytes.len() >= coords_len + 4 {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(&raw_bytes[coords_len..coords_len + 4]);
                    f32::from_le_bytes(buf) as f64
                } else {
                    1.0
                };
                for i in 0..dim.min(coords_len) {
                    let val = (raw_bytes[i] as i8) as f64 / 127.0;
                    let c = if metric_str == "lorentz" {
                        val * alpha
                    } else {
                        val
                    };
                    vec.push(c);
                }
                vec.resize(dim, 0.0);
                vec
            }
            hyperspace_core::QuantizationMode::None => {
                let mut vec = Vec::with_capacity(dim);
                if raw_bytes.len() >= dim * 8 {
                    for d in 0..dim {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&raw_bytes[d * 8..(d + 1) * 8]);
                        let f = f64::from_le_bytes(b);
                        vec.push(if f.is_finite() { f } else { 0.0 });
                    }
                } else if raw_bytes.len() >= dim * 4 {
                    for d in 0..dim {
                        let mut b = [0u8; 4];
                        b.copy_from_slice(&raw_bytes[d * 4..(d + 1) * 4]);
                        let f = f32::from_le_bytes(b) as f64;
                        vec.push(if f.is_finite() { f } else { 0.0 });
                    }
                } else {
                    vec.resize(dim, 0.0);
                }
                vec
            }
            _ => {
                let mut vec = Vec::with_capacity(dim);
                if raw_bytes.len() >= dim * 8 {
                    for d in 0..dim {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&raw_bytes[d * 8..(d + 1) * 8]);
                        let f = f64::from_le_bytes(b);
                        vec.push(if f.is_finite() { f } else { 0.0 });
                    }
                } else {
                    vec.resize(dim, 0.0);
                }
                vec
            }
        };

        // Strict guarantee: all values must be finite numbers
        for val in &mut coords {
            if !val.is_finite() {
                *val = 0.0;
            }
        }

        coords
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_auto_migration_v3_to_v4() {
        let base_dir = tempdir().unwrap();
        let data_dir = base_dir.path();

        // 1. Create a simulated legacy collection in data/legacy_v3/legacy_items
        let legacy_col = data_dir.join("legacy_v3").join("legacy_items");
        fs::create_dir_all(&legacy_col).unwrap();

        // Create meta.json
        let meta = CollectionMetadata {
            dimension: Some(4),
            metric: Some("euclidean".to_string()),
            quantization: "none".to_string(),
            sharded: false,
            schema: Some(CollectionSchema {
                components: vec![hyperspace_proto::hyperspace::VectorComponent {
                    name: "default".to_string(),
                    metric: "euclidean".to_string(),
                    full_dimension: 4,
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            }),
        };
        meta.save(&legacy_col).unwrap();

        // Write some vectors into legacy VectorStore
        let element_size = 4 * 8; // 4 * f64 = 32 bytes
        let store = VectorStore::new(&legacy_col, element_size);
        for i in 0..5 {
            let vec: Vec<f64> = vec![i as f64, (i * 2) as f64, (i * 3) as f64, (i * 4) as f64];
            let mut bytes = Vec::new();
            for f in vec {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            store.append(&bytes).unwrap();
        }
        assert_eq!(store.count(), 5);

        // 2. Run startup migration!
        let reports = MigrationEngine::run_startup_migration(data_dir)
            .await
            .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].collection_name, "legacy_items");
        assert_eq!(reports[0].vector_count, 5);

        // 3. Verify that v4 directory now exists and legacy files are archived
        let v4_col_dir = data_dir.join("v4").join("legacy_items");
        assert!(v4_col_dir.exists());
        assert!(v4_col_dir.join("meta.json").exists());
        assert!(v4_col_dir.join(".migration_v4_complete.json").exists());

        // Verify archived to legacy_v3_migrated
        let archived = data_dir.join("legacy_v3_migrated").join("legacy_items");
        assert!(archived.exists());

        // 4. Running migration again should find 0 candidates (idempotent)
        let reports2 = MigrationEngine::run_startup_migration(data_dir)
            .await
            .unwrap();
        assert!(reports2.is_empty());
    }

    #[tokio::test]
    async fn test_migration_from_data_to_data_v4() {
        let base_dir = tempdir().unwrap();
        let legacy_data_dir = base_dir.path().join("data");
        let v4_data_dir = base_dir.path().join("data_v4");

        // 1. Simulate v3 folder "data/documents"
        let legacy_col = legacy_data_dir.join("documents");
        fs::create_dir_all(&legacy_col).unwrap();

        let meta = CollectionMetadata {
            dimension: Some(4),
            metric: Some("euclidean".to_string()),
            quantization: "none".to_string(),
            sharded: false,
            schema: Some(CollectionSchema {
                components: vec![hyperspace_proto::hyperspace::VectorComponent {
                    name: "default".to_string(),
                    metric: "euclidean".to_string(),
                    full_dimension: 4,
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            }),
        };
        meta.save(&legacy_col).unwrap();

        let element_size = 4 * 8;
        let store = VectorStore::new(&legacy_col, element_size);
        for i in 0..10 {
            let vec: Vec<f64> = vec![i as f64, 1.0, 2.0, 3.0];
            let mut bytes = Vec::new();
            for f in vec {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            store.append(&bytes).unwrap();
        }

        // 2. Run migration from `data` into `data_v4`
        let reports =
            MigrationEngine::run_startup_migration_with_legacy(&v4_data_dir, &legacy_data_dir)
                .await
                .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].collection_name, "documents");
        assert_eq!(reports[0].vector_count, 10);

        // 3. Verify destination in data_v4/documents
        let v4_col = v4_data_dir.join("documents");
        assert!(v4_col.exists());
        assert!(v4_col.join("meta.json").exists());
        assert!(v4_col.join(".migration_v4_complete.json").exists());

        // 4. Verify archive in data/legacy_v3_migrated/documents
        let archived = legacy_data_dir.join("legacy_v3_migrated").join("documents");
        assert!(archived.exists());

        // 5. Subsequent run should NOT re-migrate documents
        let reports2 =
            MigrationEngine::run_startup_migration_with_legacy(&v4_data_dir, &legacy_data_dir)
                .await
                .unwrap();
        assert!(reports2.is_empty());
    }

    #[tokio::test]
    async fn test_migration_with_wal_log_hybrid() {
        let base_dir = tempdir().unwrap();
        let legacy_data_dir = base_dir.path().join("data");
        let v4_data_dir = base_dir.path().join("data_v4");

        let legacy_col = legacy_data_dir.join("hybrid_col");
        fs::create_dir_all(&legacy_col).unwrap();

        let meta = CollectionMetadata {
            dimension: Some(801),
            metric: Some("hybrid".to_string()),
            quantization: "medium_plus".to_string(),
            sharded: false,
            schema: Some(CollectionSchema {
                components: vec![hyperspace_proto::hyperspace::VectorComponent {
                    name: "default".to_string(),
                    metric: "hybrid".to_string(),
                    full_dimension: 801,
                    weight: 1.0,
                }],
                cascade_pipeline: vec![],
            }),
        };
        meta.save(&legacy_col).unwrap();

        // Write into WAL log
        let wal_path = legacy_col.join("wal.log");
        let mut wal =
            hyperspace_store::wal::Wal::new(&wal_path, hyperspace_store::wal::WalSyncMode::Strict)
                .unwrap();
        let mut test_vec = vec![0.1f64; 801];
        // Ensure lorentz time coordinate is hyperbolic
        test_vec[0] = 5.0;
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "governance rule".to_string());

        wal.append(1001, &test_vec, &metadata, 1).unwrap();
        wal.sync().unwrap();

        let reports =
            MigrationEngine::run_startup_migration_with_legacy(&v4_data_dir, &legacy_data_dir)
                .await
                .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].collection_name, "hybrid_col");
        assert_eq!(reports[0].vector_count, 1);

        // Verify that v4 shards have snapshots
        let shard0_snap = v4_data_dir
            .join("hybrid_col")
            .join("shard_0")
            .join("index.snap");
        assert!(shard0_snap.exists());

        // Second run must be idempotent
        let reports2 =
            MigrationEngine::run_startup_migration_with_legacy(&v4_data_dir, &legacy_data_dir)
                .await
                .unwrap();
        assert!(reports2.is_empty());
    }

    #[tokio::test]
    async fn test_in_place_migration_same_volume() {
        let base_dir = tempdir().unwrap();
        // In container mounts (Docker/K8s), legacy data and v4 target are both in /app/data
        let shared_data_dir = base_dir.path().join("data");

        let col_dir = shared_data_dir.join("inplace_col");
        fs::create_dir_all(&col_dir).unwrap();

        let meta = CollectionMetadata {
            dimension: Some(128),
            metric: Some("cosine".to_string()),
            quantization: "none".to_string(),
            sharded: false,
            schema: None,
        };
        meta.save(&col_dir).unwrap();

        let wal_path = col_dir.join("wal.log");
        let mut wal =
            hyperspace_store::wal::Wal::new(&wal_path, hyperspace_store::wal::WalSyncMode::Strict)
                .unwrap();
        let test_vec = vec![0.5f64; 128];
        let mut metadata = HashMap::new();
        metadata.insert("tag".to_string(), "in_place_test".to_string());
        wal.append(1, &test_vec, &metadata, 1).unwrap();
        wal.sync().unwrap();

        // Run migration where v4_data_dir == legacy_v3_dir
        let reports =
            MigrationEngine::run_startup_migration_with_legacy(&shared_data_dir, &shared_data_dir)
                .await
                .unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].collection_name, "inplace_col");
        assert_eq!(reports[0].vector_count, 1);

        // Verify v4 structure exists in inplace_col
        assert!(col_dir.join("shard_0").is_dir());
        assert!(col_dir.join(".migration_v4_complete.json").exists());

        // Verify archive directory exists in shared_data_dir/legacy_v3_migrated/inplace_col
        let archived = shared_data_dir
            .join("legacy_v3_migrated")
            .join("inplace_col");
        assert!(archived.exists());

        // Subsequent run should detect v4 shard_0 and .migration_v4_complete.json and do nothing
        let reports2 =
            MigrationEngine::run_startup_migration_with_legacy(&shared_data_dir, &shared_data_dir)
                .await
                .unwrap();
        assert!(reports2.is_empty());
    }
}
