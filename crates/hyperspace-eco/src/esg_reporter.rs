use crate::math;
use crate::storage::EcoStorage;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Formally audited ESG Compliance Report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EsgReport {
    /// Timestamp of report generation (UNIX seconds)
    pub timestamp: u64,
    /// Requested time range
    pub time_range: String,
    /// Scope 3 Emissions Avoided (kg CO2e)
    pub scope_3_emissions_avoided_kg_co2e: f64,
    /// Energy Conserved (kWh)
    pub energy_conserved_kwh: f64,
    /// Storage Efficiency Ratio (Baseline RAM Bytes / Actual RAM Bytes)
    pub storage_efficiency_ratio: f64,
    /// Total carbon emitted by actual deployment in this range (kg CO2e)
    pub actual_co2_emitted_kg: f64,
    /// Total carbon that would be emitted by flat legacy deployment in this range (kg CO2e)
    pub baseline_co2_emitted_kg: f64,
    /// Timestamped cryptographic SHA-256 signature for compliance auditing
    pub audit_signature: String,
}

/// Generates an audited business-compliance report on saved power and carbon emissions.
///
/// # Arguments
/// * `db_path` - Path to the time-series log.
/// * `time_range` - Range filter: `"24h"`, `"30d"`, `"365d"`, or `"all"`.
/// * `format` - Output format: `"json"` or `"csv"`.
pub async fn generate_esg_report(
    db_path: PathBuf,
    time_range: &str,
    format: &str,
) -> Result<String, String> {
    tokio::task::spawn_blocking({
        let db_path = db_path.clone();
        let time_range = time_range.to_string();
        let format = format.to_lowercase();
        move || {
            let storage = EcoStorage::new(db_path).map_err(|e| e.to_string())?;
            let records = storage.read_all().map_err(|e| e.to_string())?;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let start_time = match time_range.as_str() {
                "24h" | "1d" => now.saturating_sub(24 * 3600),
                "30d" | "1m" => now.saturating_sub(30 * 24 * 3600),
                "365d" | "1y" => now.saturating_sub(365 * 24 * 3600),
                _ => 0,
            };

            let mut scope_3_emissions_avoided = 0.0;
            let mut actual_co2_emitted = 0.0;

            for record in &records {
                if record.timestamp >= start_time {
                    scope_3_emissions_avoided += record.co2_saved_kg;
                    actual_co2_emitted += record.co2_emitted_kg;
                }
            }

            let baseline_co2_emitted = actual_co2_emitted + scope_3_emissions_avoided;
            let energy_conserved_kwh = if math::CO2_KG_PER_KWH > 0.0 {
                scope_3_emissions_avoided / math::CO2_KG_PER_KWH
            } else {
                0.0
            };

            // Storage Efficiency Ratio based on the latest record
            let storage_efficiency_ratio = if let Some(last) = records.last() {
                if last.actual_ram_bytes > 0 {
                    last.baseline_ram_bytes as f64 / last.actual_ram_bytes as f64
                } else {
                    1.0
                }
            } else {
                1.0
            };

            // Cryptographic audit signature (SHA-256)
            let raw_data_to_sign = format!(
                "ts={};range={};avoided={:.6};conserved={:.6};ratio={:.6}",
                now,
                time_range,
                scope_3_emissions_avoided,
                energy_conserved_kwh,
                storage_efficiency_ratio
            );

            let mut hasher = Sha256::new();
            hasher.update(raw_data_to_sign.as_bytes());
            let mut audit_signature = String::new();
            for byte in hasher.finalize() {
                audit_signature.push_str(&format!("{:02x}", byte));
            }

            let report = EsgReport {
                timestamp: now,
                time_range,
                scope_3_emissions_avoided_kg_co2e: scope_3_emissions_avoided,
                energy_conserved_kwh,
                storage_efficiency_ratio,
                actual_co2_emitted_kg: actual_co2_emitted,
                baseline_co2_emitted_kg: baseline_co2_emitted,
                audit_signature,
            };

            if format == "csv" {
                let csv = format!(
                    "Metric,Value\n\
                     Report Timestamp,{}\n\
                     Time Range,{}\n\
                     Scope 3 Emissions Avoided (kg CO2e),{:.6}\n\
                     Energy Conserved (kWh),{:.6}\n\
                     Storage Efficiency Ratio (x),{:.6}\n\
                     Actual CO2 Emitted (kg CO2e),{:.6}\n\
                     Baseline CO2 Emitted (kg CO2e),{:.6}\n\
                     Audit Signature,{}\n",
                    report.timestamp,
                    report.time_range,
                    report.scope_3_emissions_avoided_kg_co2e,
                    report.energy_conserved_kwh,
                    report.storage_efficiency_ratio,
                    report.actual_co2_emitted_kg,
                    report.baseline_co2_emitted_kg,
                    report.audit_signature
                );
                Ok(csv)
            } else {
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
