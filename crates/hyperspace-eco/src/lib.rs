#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_inception)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::format_push_string)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::unused_async)]

#[cfg(feature = "eco-monitor")]
pub mod esg_reporter;
#[cfg(feature = "eco-monitor")]
pub mod math;
#[cfg(feature = "eco-monitor")]
pub mod storage;
#[cfg(feature = "eco-monitor")]
pub mod worker;

#[cfg(feature = "eco-monitor")]
use std::path::PathBuf;
#[cfg(feature = "eco-monitor")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "eco-monitor")]
pub use esg_reporter::generate_esg_report;
#[cfg(feature = "eco-monitor")]
pub use hyperspace_core::QuantizationMode;
#[cfg(feature = "eco-monitor")]
pub use math::{CollectionEcoSchema, EcoEquivalents, EcoTier};

#[cfg(feature = "eco-monitor")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[cfg(feature = "eco-monitor")]
pub fn start_eco_monitor<F>(provider: F, db_path: PathBuf, interval_secs: u64)
where
    F: Fn() -> Vec<CollectionEcoSchema> + Send + Sync + 'static,
{
    worker::start_worker(provider, db_path, interval_secs);
}

/// Retrieve the carbon savings and emissions metrics for the requested time range.
#[cfg(feature = "eco-monitor")]
pub async fn get_carbon_metrics(
    db_path: PathBuf,
    time_range: &str,
) -> Result<CarbonMetrics, String> {
    let db_path_clone = db_path.clone();
    let time_range_str = time_range.to_string();

    tokio::task::spawn_blocking(move || {
        let storage = storage::EcoStorage::new(db_path_clone).map_err(|e| e.to_string())?;
        let records = storage.read_all().map_err(|e| e.to_string())?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let start_time = match time_range_str.as_str() {
            "24h" | "1d" => now.saturating_sub(24 * 3600),
            "30d" | "1m" => now.saturating_sub(30 * 24 * 3600),
            "365d" | "1y" => now.saturating_sub(365 * 24 * 3600),
            _ => 0, // "all" or fallback
        };

        let mut total_co2_emitted = 0.0;
        let mut total_co2_saved = 0.0;

        for record in &records {
            if record.timestamp >= start_time {
                total_co2_emitted += record.co2_emitted_kg;
                total_co2_saved += record.co2_saved_kg;
            }
        }

        // Get current values from the last record if available
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
            time_range: time_range_str,
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
    })
    .await
    .map_err(|e| e.to_string())?
}

// Stub implementation for when the feature is disabled (ensures clean compile-out)
#[cfg(not(feature = "eco-monitor"))]
pub fn start_eco_monitor<F>(_provider: F, _db_path: std::path::PathBuf, _interval_secs: u64)
where
    F: Fn() -> usize + Send + Sync + 'static,
{
    // No-op
}

#[cfg(not(feature = "eco-monitor"))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CarbonMetricsDisabled {
    pub error: String,
}

#[cfg(not(feature = "eco-monitor"))]
pub async fn get_carbon_metrics(
    _db_path: std::path::PathBuf,
    _time_range: &str,
) -> Result<CarbonMetricsDisabled, String> {
    Ok(CarbonMetricsDisabled {
        error: "EcoMonitor feature is disabled in this build".to_string(),
    })
}

#[cfg(not(feature = "eco-monitor"))]
pub async fn generate_esg_report(
    _db_path: std::path::PathBuf,
    _time_range: &str,
    _format: &str,
) -> Result<String, String> {
    Err("EcoMonitor feature is disabled in this build".to_string())
}

#[cfg(all(test, feature = "eco-monitor"))]
mod tests;
