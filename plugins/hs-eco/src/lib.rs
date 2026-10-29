//! External HyperspaceDB Plugin: `hs-eco` (`eco`)
//!
//! Carbon Footprint Telemetry, Quantization Memory-Savings Calculator & Audited SHA-256 ESG Compliance Reporter.
//!
//! This crate compiles to a dynamic library (`cdylib`: `.dylib` / `.so` / `.dll`) and exports
//! its C-ABI plugin entrypoint via `hyperspace_pdk::hs_export_plugin!`.

#![allow(clippy::pedantic)]
#![allow(unknown_lints)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::needless_bool)]

pub mod esg_reporter;
pub mod math;
pub mod storage;
pub mod worker;

use async_trait::async_trait;
use hyperspace_pdk::{
    hs_export_plugin, EngineContext, HookStage, HsPlugin, InsertPoint, PluginManifest,
    ProcedureSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use esg_reporter::generate_esg_report;
pub use hyperspace_core::QuantizationMode;
pub use math::{CollectionEcoSchema, EcoEquivalents, EcoTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonMetrics {
    pub time_range: String,
    pub total_co2_emitted_kg: f64,
    pub total_co2_saved_kg: f64,
    pub power_saved_kwh: f64,
    pub current_vector_count: u64,
    pub current_baseline_ram_gb: f64,
    pub current_actual_ram_gb: f64,
    pub equivalents: math::EcoEquivalents,
    pub eco_tier: math::EcoTier,
    pub is_eco_certified: bool,
}

/// Start the background carbon footprint monitor.
pub fn start_eco_monitor<F>(provider: F, db_path: PathBuf, interval_secs: u64)
where
    F: Fn() -> Vec<CollectionEcoSchema> + Send + Sync + 'static,
{
    worker::start_worker(provider, db_path, interval_secs);
}

/// Retrieve the carbon savings and emissions metrics for the requested time range.
pub async fn get_carbon_metrics(
    db_path: PathBuf,
    time_range: &str,
) -> Result<CarbonMetrics, String> {
    let storage = storage::EcoStorage::new(db_path).map_err(|e| e.to_string())?;
    let records = storage.read_all().map_err(|e| e.to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let start_time = match time_range {
        "24h" | "1d" => now.saturating_sub(24 * 3600),
        "30d" | "1m" => now.saturating_sub(30 * 24 * 3600),
        "365d" | "1y" => now.saturating_sub(365 * 24 * 3600),
        _ => 0,
    };

    let mut total_co2_emitted = 0.0;
    let mut total_co2_saved = 0.0;

    for record in &records {
        if record.timestamp >= start_time {
            total_co2_emitted += record.co2_emitted_kg;
            total_co2_saved += record.co2_saved_kg;
        }
    }

    let last_rec = records.last();
    let (current_vectors, baseline_bytes, actual_bytes) = match last_rec {
        Some(r) => (r.vector_count, r.baseline_ram_bytes, r.actual_ram_bytes),
        None => (0, 0, 0),
    };

    let current_baseline_ram_gb = math::bytes_to_gb(baseline_bytes);
    let current_actual_ram_gb = math::bytes_to_gb(actual_bytes);

    let power_saved_kwh = if math::CO2_KG_PER_KWH > 0.0 {
        total_co2_saved / math::CO2_KG_PER_KWH
    } else {
        0.0
    };

    let equivalents = math::EcoEquivalents::calculate(total_co2_saved);

    let saved_bytes = baseline_bytes.saturating_sub(actual_bytes);
    let eco_tier = math::EcoTier::calculate(saved_bytes, baseline_bytes);
    let is_eco_certified = eco_tier.is_eco_certified();

    Ok(CarbonMetrics {
        time_range: time_range.to_string(),
        total_co2_emitted_kg: total_co2_emitted,
        total_co2_saved_kg: total_co2_saved,
        power_saved_kwh,
        current_vector_count: current_vectors,
        current_baseline_ram_gb,
        current_actual_ram_gb,
        equivalents,
        eco_tier,
        is_eco_certified,
    })
}

/// Official `hs-eco` (`eco`) external plugin for Carbon & ESG Telemetry.
pub struct HsEcoPlugin {
    default_db_path: PathBuf,
    observed_inserts: AtomicU64,
}

impl Default for HsEcoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HsEcoPlugin {
    pub fn new() -> Self {
        let data_dir = std::env::var("HS_DATA_DIR")
            .or_else(|_| std::env::var("HS_DATA_DIR_V4"))
            .unwrap_or_else(|_| "data_v4".to_string());
        Self {
            default_db_path: PathBuf::from(data_dir).join("eco_telemetry.bin"),
            observed_inserts: AtomicU64::new(0),
        }
    }

    fn resolve_db_path(&self, args: &Value) -> PathBuf {
        args.get("db_path")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.default_db_path.clone())
    }
}

#[async_trait]
impl HsPlugin for HsEcoPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "eco".to_string(),
            version: "4.2.0".to_string(),
            author: "HyperspaceDB Core Team".to_string(),
            description: "Carbon footprint telemetry, quantization RAM savings analyzer, and SHA-256 audited ESG compliance reporter.".to_string(),
            runtime: "native-rust-cdylib".to_string(),
            enabled: true,
            source: "external".to_string(),
            hooks: vec![HookStage::BeforeInsert],
            procedures: vec![
                ProcedureSchema {
                    name: "metrics".to_string(),
                    description: "Returns carbon emissions avoided (kg CO2e), energy saved (kWh), EcoTier badge, and environmental equivalents.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "range": { "type": "string", "default": "all", "description": "Time range: 24h, 30d, 365d, or all" },
                            "db_path": { "type": "string", "description": "Optional override path to eco_telemetry.bin" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "time_range": { "type": "string" },
                            "total_co2_emitted_kg": { "type": "number" },
                            "total_co2_saved_kg": { "type": "number" },
                            "power_saved_kwh": { "type": "number" },
                            "eco_tier": { "type": "string" },
                            "is_eco_certified": { "type": "boolean" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "esg_report".to_string(),
                    description: "Generates a cryptographically signed (SHA-256) Scope-3 ESG compliance report in JSON or CSV format.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "range": { "type": "string", "default": "all" },
                            "format": { "type": "string", "default": "json", "description": "json or csv" },
                            "db_path": { "type": "string" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "format": { "type": "string" },
                            "report": { "type": "string" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "collection_tier".to_string(),
                    description: "Evaluates the memory compression savings and EcoTier (Bronze/Silver/Gold/Platinum) for a collection.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "vector_count": { "type": "integer", "default": 100000 },
                            "full_dimension": { "type": "integer", "default": 1024 },
                            "mrl_dimension": { "type": "integer", "default": 1024 },
                            "quantization": { "type": "string", "default": "medium_plus" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "baseline_ram_bytes": { "type": "integer" },
                            "actual_ram_bytes": { "type": "integer" },
                            "saved_bytes": { "type": "integer" },
                            "eco_tier": { "type": "string" },
                            "is_eco_certified": { "type": "boolean" }
                        }
                    }),
                },
            ],
            custom_metrics: vec![],
        }
    }

    async fn before_insert(
        &self,
        _collection: &str,
        _metric: &str,
        _dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        self.observed_inserts
            .fetch_add(points.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        match method {
            "metrics" | "get_carbon_metrics" => {
                let range = args.get("range").and_then(|v| v.as_str()).unwrap_or("all");
                let db_path = self.resolve_db_path(&args);
                let metrics = get_carbon_metrics(db_path, range).await?;
                serde_json::to_value(metrics).map_err(|e| e.to_string())
            }
            "esg_report" | "generate_esg_report" => {
                let range = args.get("range").and_then(|v| v.as_str()).unwrap_or("all");
                let format = args
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");
                let db_path = self.resolve_db_path(&args);
                let raw = generate_esg_report(db_path, range, format).await?;
                if format.eq_ignore_ascii_case("csv") {
                    Ok(json!({
                        "format": "csv",
                        "report": raw
                    }))
                } else {
                    let parsed: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!(raw));
                    Ok(json!({
                        "format": "json",
                        "report": parsed,
                        "raw": raw
                    }))
                }
            }
            "collection_tier" => {
                let (vc, full_dim, mrl_dim, q_mode) = if let Ok(col) = ctx.collection() {
                    (
                        col.count() as u64,
                        col.dimension() as u32,
                        col.dimension() as u32,
                        col.quantization_mode(),
                    )
                } else {
                    let vc = args
                        .get("vector_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(100_000);
                    let fd = args
                        .get("full_dimension")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1024) as u32;
                    let md = args
                        .get("mrl_dimension")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(u64::from(fd)) as u32;
                    let q_str = args
                        .get("quantization")
                        .and_then(|v| v.as_str())
                        .unwrap_or("medium_plus");
                    let q = match q_str.to_ascii_lowercase().as_str() {
                        "none" => QuantizationMode::None,
                        "medium" | "scalar_i8" => QuantizationMode::ScalarI8,
                        "turbo" => QuantizationMode::Turbo,
                        "extreme" | "binary" => QuantizationMode::Binary,
                        _ => QuantizationMode::ScalarI4,
                    };
                    (vc, fd, md, q)
                };

                let schema = CollectionEcoSchema {
                    collection_name: ctx
                        .collection_name
                        .clone()
                        .unwrap_or_else(|| "simulated".to_string()),
                    vector_count: vc,
                    full_dimension: full_dim,
                    mrl_dimension: mrl_dim,
                    quantization: q_mode,
                };
                let baseline = schema.calculate_baseline_ram_bytes();
                let actual = schema.calculate_actual_ram_bytes();
                let saved = baseline.saturating_sub(actual);
                let tier = schema.calculate_eco_tier();
                Ok(json!({
                    "baseline_ram_bytes": baseline,
                    "actual_ram_bytes": actual,
                    "saved_bytes": saved,
                    "eco_tier": format!("{tier:?}"),
                    "is_eco_certified": tier.is_eco_certified(),
                    "observed_session_inserts": self.observed_inserts.load(Ordering::Relaxed)
                }))
            }
            other => Err(format!("Unknown procedure 'eco.{other}'")),
        }
    }
}

// Export C-ABI entrypoints (`hs_pdk_api_version` & `hs_plugin_create`) for dynamic loading from `./plugins/`
hs_export_plugin!(HsEcoPlugin::new);

#[cfg(test)]
mod tests;
