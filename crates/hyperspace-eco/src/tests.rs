#[cfg(test)]
use crate::esg_reporter;
use crate::math::{CollectionEcoSchema, EcoTier};
use crate::storage::{EcoStorage, TelemetryRecord};
use hyperspace_core::QuantizationMode;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_dynamic_ram_calculations() {
    // Test Binary (extreme)
    let schema_bit1 = CollectionEcoSchema {
        collection_name: "test_bit1".to_string(),
        vector_count: 10_000,
        full_dimension: 1536,
        mrl_dimension: 256,
        quantization: QuantizationMode::Binary,
    };

    // Baseline: 10_000 * 1536 * 4 = 61,440,000 bytes
    assert_eq!(schema_bit1.calculate_baseline_ram_bytes(), 61_440_000);
    // Actual (Binary): 10_000 * (((256 + 7) / 8) + 4) = 10_000 * 36 = 360,000 bytes
    assert_eq!(schema_bit1.calculate_actual_ram_bytes(), 360_000);

    // Test ScalarI8 (medium)
    let schema_i8 = CollectionEcoSchema {
        collection_name: "test_i8".to_string(),
        vector_count: 10_000,
        full_dimension: 1536,
        mrl_dimension: 256,
        quantization: QuantizationMode::ScalarI8,
    };
    // Actual (ScalarI8): 10_000 * (256 + 4) = 2,600,000 bytes
    assert_eq!(schema_i8.calculate_actual_ram_bytes(), 2_600_000);

    // Test AsymmetricHybrid801
    let schema_hybrid = CollectionEcoSchema {
        collection_name: "test_hybrid".to_string(),
        vector_count: 10_000,
        full_dimension: 1536,
        mrl_dimension: 256,
        quantization: QuantizationMode::AsymmetricHybrid801,
    };
    // Actual (Hybrid): 10_000 * (33 * 4 + (256 - 33) + 4) = 10_000 * 359 = 3,590,000 bytes
    assert_eq!(schema_hybrid.calculate_actual_ram_bytes(), 3_590_000);

    // Test None (uncompressed)
    let schema_none = CollectionEcoSchema {
        collection_name: "test_none".to_string(),
        vector_count: 10_000,
        full_dimension: 1536,
        mrl_dimension: 256,
        quantization: QuantizationMode::None,
    };
    // Actual (None): 10_000 * (256 * 8 + 8) = 20,560,000 bytes
    assert_eq!(schema_none.calculate_actual_ram_bytes(), 20_560_000);
}

#[test]
fn test_eco_tier_badges() {
    // None tier (<= 10% saved)
    let tier_none = EcoTier::calculate(5, 100);
    assert_eq!(tier_none, EcoTier::None);
    assert!(!tier_none.is_eco_certified());

    // Bronze tier (> 10% and <= 30% saved)
    let tier_bronze = EcoTier::calculate(20, 100);
    assert_eq!(tier_bronze, EcoTier::Bronze);
    assert!(!tier_bronze.is_eco_certified());

    // Silver tier (> 30% and <= 50% saved)
    let tier_silver = EcoTier::calculate(40, 100);
    assert_eq!(tier_silver, EcoTier::Silver);
    assert!(!tier_silver.is_eco_certified());

    // Gold tier (> 50% and <= 75% saved)
    let tier_gold = EcoTier::calculate(60, 100);
    assert_eq!(tier_gold, EcoTier::Gold);
    assert!(tier_gold.is_eco_certified());

    // Platinum tier (> 75% saved)
    let tier_plat = EcoTier::calculate(85, 100);
    assert_eq!(tier_plat, EcoTier::Platinum);
    assert!(tier_plat.is_eco_certified());
}

#[tokio::test]
async fn test_esg_compliance_reporting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();
    let storage = EcoStorage::new(db_path.clone()).unwrap();

    // Write some mock records
    let rec = TelemetryRecord {
        timestamp: 1000,
        vector_count: 50_000,
        baseline_ram_bytes: 307_200_000, // 307.2 MB
        actual_ram_bytes: 25_600_000,    // 25.6 MB
        co2_emitted_kg: 0.05,
        co2_saved_kg: 0.45,
    };
    storage.append(&rec).unwrap();

    // Generate JSON report
    let json_report = esg_reporter::generate_esg_report(db_path.clone(), "all", "json")
        .await
        .unwrap();
    assert!(json_report.contains("\"scope_3_emissions_avoided_kg_co2e\": 0.45"));
    assert!(json_report.contains("\"energy_conserved_kwh\":"));
    assert!(json_report.contains("\"audit_signature\":"));

    // Generate CSV report
    let csv_report = esg_reporter::generate_esg_report(db_path, "all", "csv")
        .await
        .unwrap();
    assert!(csv_report.contains("Scope 3 Emissions Avoided (kg CO2e),0.450000"));
    assert!(csv_report.contains("Energy Conserved (kWh),"));
    assert!(csv_report.contains("Audit Signature,"));
}

#[test]
fn test_eco_storage_and_renaming() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();
    let storage = EcoStorage::new(db_path).unwrap();

    let rec = TelemetryRecord {
        timestamp: 500,
        vector_count: 10_000,
        baseline_ram_bytes: 61_440_000,
        actual_ram_bytes: 5_160_000,
        co2_emitted_kg: 0.001,
        co2_saved_kg: 0.009,
    };

    storage.append(&rec).unwrap();
    let retrieved = storage.get_last_record().unwrap().unwrap();
    assert_eq!(retrieved.baseline_ram_bytes, 61_440_000);
    assert_eq!(retrieved.actual_ram_bytes, 5_160_000);
    assert!((retrieved.co2_saved_kg - 0.009).abs() < f64::EPSILON);
}
