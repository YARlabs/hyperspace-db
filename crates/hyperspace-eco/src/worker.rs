use crate::math::{self, CollectionEcoSchema};
use crate::storage::{EcoStorage, TelemetryRecord};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info};

/// Starts the carbon footprint monitoring background worker thread.
///
/// # Arguments
/// * `provider` - A thread-safe closure returning all active collections' configuration schemas.
/// * `db_path` - Path to the binary log file.
/// * `interval_secs` - Logging tick interval in seconds.
pub fn start_worker<F>(provider: F, db_path: PathBuf, interval_secs: u64)
where
    F: Fn() -> Vec<CollectionEcoSchema> + Send + Sync + 'static,
{
    std::thread::Builder::new()
        .name("hyperspace-eco-worker".to_string())
        .spawn(move || {
            let storage = match EcoStorage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    error!("🌿 [EcoMonitor] Failed to initialize storage: {:?}", e);
                    return;
                }
            };

            let mut last_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            info!(
                "🌿 [EcoMonitor] Dynamic background worker thread initialized. Interval: {}s",
                interval_secs
            );

            loop {
                std::thread::sleep(Duration::from_secs(interval_secs));

                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let dt = current_time.saturating_sub(last_time);
                if dt == 0 {
                    continue;
                }

                // Get dynamic active collection configurations
                let schemas = provider();

                let total_vectors: u64 = schemas.iter().map(|s| s.vector_count).sum();
                let total_baseline_bytes: u64 = schemas
                    .iter()
                    .map(CollectionEcoSchema::calculate_baseline_ram_bytes)
                    .sum();
                let total_actual_bytes: u64 = schemas
                    .iter()
                    .map(CollectionEcoSchema::calculate_actual_ram_bytes)
                    .sum();

                let baseline_gb = math::bytes_to_gb(total_baseline_bytes);
                let actual_gb = math::bytes_to_gb(total_actual_bytes);

                let baseline_wh = math::ram_gb_to_watt_hours(baseline_gb, dt as f64);
                let actual_wh = math::ram_gb_to_watt_hours(actual_gb, dt as f64);

                let co2_emitted = math::watt_hours_to_co2_kg(actual_wh);
                let co2_saved = math::watt_hours_to_co2_kg(baseline_wh.saturating_sub(actual_wh));

                let record = TelemetryRecord {
                    timestamp: current_time,
                    vector_count: total_vectors,
                    baseline_ram_bytes: total_baseline_bytes,
                    actual_ram_bytes: total_actual_bytes,
                    co2_emitted_kg: co2_emitted,
                    co2_saved_kg: co2_saved,
                };

                if let Err(e) = storage.append(&record) {
                    error!("🌿 [EcoMonitor] Failed to append telemetry record: {:?}", e);
                }

                last_time = current_time;
            }
        })
        .expect("🌿 [EcoMonitor] Failed to spawn background worker thread");
}

trait FloatSubExt {
    fn saturating_sub(self, other: Self) -> Self;
}

impl FloatSubExt for f64 {
    fn saturating_sub(self, other: Self) -> Self {
        let diff = self - other;
        if diff < 0.0 {
            0.0
        } else {
            diff
        }
    }
}
