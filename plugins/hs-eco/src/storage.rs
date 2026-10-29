use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

pub const RECORD_SIZE: usize = 48;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TelemetryRecord {
    /// UNIX timestamp in seconds
    pub timestamp: u64,
    /// Total active vector count
    pub vector_count: u64,
    /// Memory footprint for baseline databases in bytes (flat f32 full dimension)
    pub baseline_ram_bytes: u64,
    /// Memory footprint for HyperspaceDB in bytes (MRL active dim and quantization)
    pub actual_ram_bytes: u64,
    /// CO2 emitted during the elapsed time since the last tick (in kg)
    pub co2_emitted_kg: f64,
    /// CO2 saved during the elapsed time since the last tick (in kg)
    pub co2_saved_kg: f64,
}

impl TelemetryRecord {
    pub fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut bytes = [0u8; RECORD_SIZE];
        bytes[0..8].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.vector_count.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.baseline_ram_bytes.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.actual_ram_bytes.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.co2_emitted_kg.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.co2_saved_kg.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: [u8; RECORD_SIZE]) -> Self {
        Self {
            timestamp: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            vector_count: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            baseline_ram_bytes: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            actual_ram_bytes: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            co2_emitted_kg: f64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            co2_saved_kg: f64::from_le_bytes(bytes[40..48].try_into().unwrap()),
        }
    }
}

pub struct EcoStorage {
    path: PathBuf,
}

impl EcoStorage {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }

    /// Append a telemetry record to the append-only binary file.
    pub fn append(&self, record: &TelemetryRecord) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(&record.to_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Read all records from the binary log using a memory-mapped file.
    pub fn read_all(&self) -> io::Result<Vec<TelemetryRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new().read(true).open(&self.path)?;
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            return Ok(Vec::new());
        }

        // Memory-map the file for fast reading
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let len = mmap.len();
        let num_records = len / RECORD_SIZE;

        let mut records = Vec::with_capacity(num_records);
        for i in 0..num_records {
            let start = i * RECORD_SIZE;
            let end = start + RECORD_SIZE;
            let mut record_bytes = [0u8; RECORD_SIZE];
            record_bytes.copy_from_slice(&mmap[start..end]);
            records.push(TelemetryRecord::from_bytes(record_bytes));
        }

        Ok(records)
    }

    /// Get the very last record in the log file, or None if empty.
    pub fn get_last_record(&self) -> io::Result<Option<TelemetryRecord>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let file = OpenOptions::new().read(true).open(&self.path)?;
        let metadata = file.metadata()?;
        let len = metadata.len();
        if len < RECORD_SIZE as u64 {
            return Ok(None);
        }

        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let start = (mmap.len() / RECORD_SIZE - 1) * RECORD_SIZE;
        let end = start + RECORD_SIZE;
        let mut record_bytes = [0u8; RECORD_SIZE];
        record_bytes.copy_from_slice(&mmap[start..end]);
        Ok(Some(TelemetryRecord::from_bytes(record_bytes)))
    }
}
