//! Mathematical calculations for carbon footprint and memory-saving efficiency.

use hyperspace_core::QuantizationMode;

/// Collection metadata representing the configurations for carbon footprint logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionEcoSchema {
    pub collection_name: String,
    pub vector_count: u64,
    pub full_dimension: u32,
    pub mrl_dimension: u32,
    pub quantization: QuantizationMode,
}

impl CollectionEcoSchema {
    /// Calculate RAM footprint in bytes if the collection was stored as uncompressed flat f32 vectors at full dimension
    #[must_use]
    pub fn calculate_baseline_ram_bytes(&self) -> u64 {
        self.vector_count * self.full_dimension as u64 * 4
    }

    /// Calculate RAM footprint in bytes incorporating chosen MRL dimension cutoff and quantization level
    #[must_use]
    pub fn calculate_actual_ram_bytes(&self) -> u64 {
        let dim = self.mrl_dimension as usize;
        let bytes_per_vector = match self.quantization {
            QuantizationMode::Binary => ((dim + 7) / 8 + 4) as u64,
            QuantizationMode::ScalarI8 => (dim + 4) as u64,
            QuantizationMode::AsymmetricHybrid801 => {
                if dim > 33 {
                    (33 * 4 + (dim - 33) + 4) as u64
                } else {
                    (dim * 4 + 4) as u64
                }
            }
            QuantizationMode::None => {
                let storage_f32_requested = std::env::var("HS_STORAGE_FLOAT32").is_ok_and(|v| {
                    matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on")
                });
                if storage_f32_requested {
                    (dim * 4 + 4) as u64
                } else {
                    (dim * 8 + 8) as u64
                }
            }
        };
        self.vector_count * bytes_per_vector
    }

    /// Determine the collection's individual EcoTier based on savings percentage
    #[must_use]
    pub fn calculate_eco_tier(&self) -> EcoTier {
        let baseline = self.calculate_baseline_ram_bytes();
        let actual = self.calculate_actual_ram_bytes();
        if baseline == 0 {
            return EcoTier::None;
        }
        let saved = baseline.saturating_sub(actual);
        EcoTier::calculate(saved, baseline)
    }
}

/// Gamified badges assigned based on memory saved compared to flat baseline
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EcoTier {
    /// Memory savings <= 10%
    None,
    /// Memory savings > 10% and <= 30%
    Bronze,
    /// Memory savings > 30% and <= 50%
    Silver,
    /// Memory savings > 50% and <= 75%
    Gold,
    /// Memory savings > 75%
    Platinum,
}

impl EcoTier {
    #[must_use]
    pub fn calculate(saved_bytes: u64, baseline_bytes: u64) -> Self {
        if baseline_bytes == 0 {
            return Self::None;
        }
        let ratio = saved_bytes as f64 / baseline_bytes as f64;
        if ratio > 0.75 {
            Self::Platinum
        } else if ratio > 0.50 {
            Self::Gold
        } else if ratio > 0.30 {
            Self::Silver
        } else if ratio > 0.10 {
            Self::Bronze
        } else {
            Self::None
        }
    }

    /// Awarded when Saved_Bytes > 50% of Baseline_RAM_Bytes (Gold or Platinum tier)
    #[must_use]
    pub fn is_eco_certified(&self) -> bool {
        matches!(self, Self::Gold | Self::Platinum)
    }
}

/// Power consumption per GB of RAM including PUE overhead in Watts.
pub const WATTS_PER_GB: f64 = 3.0;

/// CO2 emissions per kWh of electricity in kg.
pub const CO2_KG_PER_KWH: f64 = 0.4;

/// Tree absorption of CO2 per year in kg.
pub const CO2_KG_ABSORBED_BY_TREE_PER_YEAR: f64 = 22.0;

/// Average vehicle CO2 emissions per mile in kg.
pub const CO2_KG_PER_CAR_MILE: f64 = 0.402;

/// Average CO2 emissions per smartphone charge in kg.
pub const CO2_KG_PER_SMARTPHONE_CHARGE: f64 = 0.0083;

/// Convert bytes to binary Gigabytes (GiB).
#[must_use]
pub fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// Convert RAM GB to Watt-hours consumed over a specific duration in seconds.
/// Power (W) = GB * 3.0
/// Energy (Wh) = Power (W) * Time (hours)
#[must_use]
pub fn ram_gb_to_watt_hours(gb: f64, duration_secs: f64) -> f64 {
    let power_watts = gb * WATTS_PER_GB;
    let time_hours = duration_secs / 3600.0;
    power_watts * time_hours
}

/// Convert Watt-hours to kg of CO2.
/// Energy (kWh) = Wh / 1000
/// CO2 (kg) = kWh * 0.4
#[must_use]
pub fn watt_hours_to_co2_kg(wh: f64) -> f64 {
    let kwh = wh / 1000.0;
    kwh * CO2_KG_PER_KWH
}

/// Calculate environmental equivalents for a given amount of saved CO2.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct EcoEquivalents {
    pub tree_days: f64,
    pub car_miles: f64,
    pub smartphone_charges: f64,
}

impl EcoEquivalents {
    #[must_use]
    pub fn calculate(co2_saved_kg: f64) -> Self {
        let tree_days = if co2_saved_kg > 0.0 {
            co2_saved_kg / (CO2_KG_ABSORBED_BY_TREE_PER_YEAR / 365.0)
        } else {
            0.0
        };

        let car_miles = if co2_saved_kg > 0.0 {
            co2_saved_kg / CO2_KG_PER_CAR_MILE
        } else {
            0.0
        };

        let smartphone_charges = if co2_saved_kg > 0.0 {
            co2_saved_kg / CO2_KG_PER_SMARTPHONE_CHARGE
        } else {
            0.0
        };

        Self {
            tree_days,
            car_miles,
            smartphone_charges,
        }
    }
}
