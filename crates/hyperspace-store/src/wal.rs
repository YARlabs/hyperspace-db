#![allow(clippy::cast_possible_truncation)]
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc32fast::Hasher;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;

const WAL_V3_MAGIC: u8 = 0xFF;

/// Durability mode for Write-Ahead Log.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WalSyncMode {
    /// Fsync every write (Durability: Max, Speed: Low)
    Strict,
    /// Flush to OS cache (Durability: Medium, Speed: Max)
    Async,
    /// Background Fsync (Durability: High, Speed: High)
    Batch,
}

/// Write-Ahead Log implementation for durability.
/// Appends operations to a log file with CRC32 checksums.
#[derive(Debug)]
pub struct Wal {
    file: BufWriter<File>,
    mode: WalSyncMode,
    path: std::path::PathBuf,
    current_size: u64,
    size_limit: u64,
    pending_entries: u64,
    /// Batch mode: track last fsync time for background sync
    last_fsync_time: std::time::Instant,
    /// Batch mode fsync interval in milliseconds
    batch_fsync_interval_ms: u64,
    /// Reusable scratch buffer for zero-alloc serialization
    scratch_buf: Vec<u8>,
}

/// Represents an operation stored in the WAL.
#[derive(Debug)]
pub enum WalEntry {
    Insert {
        id: u32,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
        logical_clock: u64,
    },
    Insert64 {
        id: u64,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
        logical_clock: u64,
    },
    Delete {
        id: u64,
        logical_clock: u64,
    },
    UpdatePayload {
        id: u64,
        metadata: HashMap<String, String>,
        logical_clock: u64,
    },
}

impl Wal {
    pub fn new(path: &std::path::Path, mode: WalSyncMode) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let metadata = file.metadata()?;
        let current_size = metadata.len();
        // Default 512 MB segment size (Quick Win #2: increased from 75 MB for better performance)
        // Can be overridden via set_size_limit() based on HS_WAL_SEGMENT_SIZE_MB env var
        // Batch mode fsync interval from env var (default 100ms)
        let batch_fsync_interval_ms = std::env::var("HYPERSPACE_WAL_BATCH_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(100);
        Ok(Self {
            file: BufWriter::with_capacity(128 * 1024, file),
            mode,
            path: path.to_owned(),
            current_size,
            size_limit: 512 * 1024 * 1024,
            pending_entries: 0,
            last_fsync_time: std::time::Instant::now(),
            batch_fsync_interval_ms,
            scratch_buf: Vec::with_capacity(4096),
        })
    }

    pub fn set_size_limit(&mut self, limit_bytes: u64) {
        self.size_limit = limit_bytes;
    }

    pub fn is_full(&self) -> bool {
        self.current_size >= self.size_limit
    }

    pub fn size(&self) -> u64 {
        self.current_size
    }

    pub fn pending_entries(&self) -> u64 {
        self.pending_entries
    }

    pub fn rotate(&mut self) -> io::Result<std::path::PathBuf> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;

        // Rename the old file to a frozen state based on current time
        let frozen_path = self.path.with_extension(format!(
            "frozen.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::rename(&self.path, &frozen_path)?;

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)?;
        self.file = BufWriter::with_capacity(128 * 1024, file);
        self.current_size = 0;
        self.pending_entries = 0;
        self.last_fsync_time = std::time::Instant::now(); // Reset fsync timer for new WAL
        Ok(frozen_path)
    }

    fn serialize_entry_into(
        buf: &mut Vec<u8>,
        id: u32,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        buf.clear();
        let estimated_size = 1 + 4 + 8 + 4 + vector.len() * 8 + metadata.len() * 32;
        buf.reserve(estimated_size);
        // Internal Format: OpCode 3 (Insert V3 with clock)
        buf.write_u8(3)?;
        buf.write_u32::<LittleEndian>(id)?;
        buf.write_u64::<LittleEndian>(logical_clock)?;

        // Vector
        buf.write_u32::<LittleEndian>(vector.len() as u32)?;
        #[cfg(target_endian = "little")]
        {
            let bytes = unsafe {
                std::slice::from_raw_parts(vector.as_ptr().cast::<u8>(), vector.len() * 8)
            };
            buf.write_all(bytes)?;
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &val in vector {
                buf.write_f64::<LittleEndian>(val)?;
            }
        }

        // Metadata
        buf.write_u32::<LittleEndian>(metadata.len() as u32)?;
        for (k, v) in metadata {
            let k_bytes = k.as_bytes();
            buf.write_u32::<LittleEndian>(k_bytes.len() as u32)?;
            buf.write_all(k_bytes)?;

            let v_bytes = v.as_bytes();
            buf.write_u32::<LittleEndian>(v_bytes.len() as u32)?;
            buf.write_all(v_bytes)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn serialize_entry(
        id: u32,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        Self::serialize_entry_into(&mut buf, id, vector, metadata, logical_clock)?;
        Ok(buf)
    }

    fn write_scratch_packet(&mut self) -> io::Result<()> {
        let len = self.scratch_buf.len() as u32;
        let mut hasher = Hasher::new();
        hasher.update(&self.scratch_buf);
        let crc = hasher.finalize();

        // Header: [Magic: 1][Length: 4][CRC: 4]
        self.file.write_u8(WAL_V3_MAGIC)?;
        self.file.write_u32::<LittleEndian>(len)?;
        self.file.write_u32::<LittleEndian>(crc)?;

        // Payload
        self.file.write_all(&self.scratch_buf)?;

        // 9 bytes header + payload length
        self.current_size += 9 + self.scratch_buf.len() as u64;
        self.pending_entries += 1;

        Ok(())
    }

    #[allow(dead_code)]
    fn write_packet_internal(&mut self, payload: &[u8]) -> io::Result<()> {
        let len = payload.len() as u32;
        let mut hasher = Hasher::new();
        hasher.update(payload);
        let crc = hasher.finalize();

        // Header: [Magic: 1][Length: 4][CRC: 4]
        self.file.write_u8(WAL_V3_MAGIC)?;
        self.file.write_u32::<LittleEndian>(len)?;
        self.file.write_u32::<LittleEndian>(crc)?;

        // Payload
        self.file.write_all(payload)?;

        // 9 bytes header + payload length
        self.current_size += 9 + payload.len() as u64;
        self.pending_entries += 1;

        Ok(())
    }

    #[inline]
    fn apply_sync_policy(&mut self) -> io::Result<()> {
        match self.mode {
            WalSyncMode::Strict => {
                self.file.flush()?;
                self.file.get_ref().sync_all()?;
                self.last_fsync_time = std::time::Instant::now();
            }
            WalSyncMode::Batch => {
                if self.last_fsync_time.elapsed().as_millis()
                    >= u128::from(self.batch_fsync_interval_ms)
                {
                    self.file.flush()?;
                    self.file.get_ref().sync_all()?;
                    self.last_fsync_time = std::time::Instant::now();
                }
            }
            WalSyncMode::Async => {}
        }
        Ok(())
    }

    pub fn append(
        &mut self,
        id: u32,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        Self::serialize_entry_into(&mut self.scratch_buf, id, vector, metadata, logical_clock)?;
        self.write_scratch_packet()?;
        self.apply_sync_policy()
    }

    fn serialize_entry_u64_into(
        buf: &mut Vec<u8>,
        id: u64,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        buf.clear();
        let estimated_size = 1 + 8 + 8 + 4 + vector.len() * 8 + metadata.len() * 32;
        buf.reserve(estimated_size);
        // Internal Format: OpCode 4 (Insert V4 with u64 ID and clock)
        buf.write_u8(4)?;
        buf.write_u64::<LittleEndian>(id)?;
        buf.write_u64::<LittleEndian>(logical_clock)?;

        // Vector
        buf.write_u32::<LittleEndian>(vector.len() as u32)?;
        #[cfg(target_endian = "little")]
        {
            let bytes = unsafe {
                std::slice::from_raw_parts(vector.as_ptr().cast::<u8>(), vector.len() * 8)
            };
            buf.write_all(bytes)?;
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &val in vector {
                buf.write_f64::<LittleEndian>(val)?;
            }
        }

        // Metadata
        buf.write_u32::<LittleEndian>(metadata.len() as u32)?;
        for (k, v) in metadata {
            let k_bytes = k.as_bytes();
            buf.write_u32::<LittleEndian>(k_bytes.len() as u32)?;
            buf.write_all(k_bytes)?;

            let v_bytes = v.as_bytes();
            buf.write_u32::<LittleEndian>(v_bytes.len() as u32)?;
            buf.write_all(v_bytes)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn serialize_entry_u64(
        id: u64,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        Self::serialize_entry_u64_into(&mut buf, id, vector, metadata, logical_clock)?;
        Ok(buf)
    }

    fn serialize_delete_entry_into(
        buf: &mut Vec<u8>,
        id: u64,
        logical_clock: u64,
    ) -> io::Result<()> {
        buf.clear();
        buf.reserve(17);
        // Internal Format: OpCode 5 (Delete V4 with u64 ID and clock)
        buf.write_u8(5)?;
        buf.write_u64::<LittleEndian>(id)?;
        buf.write_u64::<LittleEndian>(logical_clock)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn serialize_delete_entry(id: u64, logical_clock: u64) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        Self::serialize_delete_entry_into(&mut buf, id, logical_clock)?;
        Ok(buf)
    }

    pub fn append_delete(&mut self, id: u64, logical_clock: u64) -> io::Result<()> {
        Self::serialize_delete_entry_into(&mut self.scratch_buf, id, logical_clock)?;
        self.write_scratch_packet()?;
        self.apply_sync_policy()
    }

    fn serialize_update_payload_entry_into(
        buf: &mut Vec<u8>,
        id: u64,
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        buf.clear();
        let estimated_size = 1 + 8 + 8 + 4 + metadata.len() * 32;
        buf.reserve(estimated_size);
        // Internal Format: OpCode 6 (UpdatePayload with u64 ID, clock, and metadata)
        buf.write_u8(6)?;
        buf.write_u64::<LittleEndian>(id)?;
        buf.write_u64::<LittleEndian>(logical_clock)?;

        // Metadata
        buf.write_u32::<LittleEndian>(metadata.len() as u32)?;
        for (k, v) in metadata {
            let k_bytes = k.as_bytes();
            buf.write_u32::<LittleEndian>(k_bytes.len() as u32)?;
            buf.write_all(k_bytes)?;

            let v_bytes = v.as_bytes();
            buf.write_u32::<LittleEndian>(v_bytes.len() as u32)?;
            buf.write_all(v_bytes)?;
        }
        Ok(())
    }

    pub fn append_update_payload(
        &mut self,
        id: u64,
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        Self::serialize_update_payload_entry_into(&mut self.scratch_buf, id, metadata, logical_clock)?;
        self.write_scratch_packet()?;
        self.apply_sync_policy()
    }

    pub fn append_batch(
        &mut self,
        entries: &[(Vec<f64>, u32, HashMap<String, String>)],
        logical_clock: u64,
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            Self::serialize_entry_into(
                &mut self.scratch_buf,
                *id,
                vector,
                metadata,
                logical_clock,
            )?;
            self.write_scratch_packet()?;
        }
        self.apply_sync_policy()
    }

    pub fn append_batch_borrowed(
        &mut self,
        entries: &[(&[f64], u32, &HashMap<String, String>)],
        logical_clock: u64,
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            Self::serialize_entry_into(
                &mut self.scratch_buf,
                *id,
                vector,
                metadata,
                logical_clock,
            )?;
            self.write_scratch_packet()?;
        }
        self.apply_sync_policy()
    }

    pub fn append_u64(
        &mut self,
        id: u64,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<()> {
        Self::serialize_entry_u64_into(&mut self.scratch_buf, id, vector, metadata, logical_clock)?;
        self.write_scratch_packet()?;
        self.apply_sync_policy()
    }

    pub fn append_batch_u64(
        &mut self,
        entries: &[(Vec<f64>, u64, HashMap<String, String>)],
        logical_clock: u64,
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            Self::serialize_entry_u64_into(
                &mut self.scratch_buf,
                *id,
                vector,
                metadata,
                logical_clock,
            )?;
            self.write_scratch_packet()?;
        }
        self.apply_sync_policy()
    }

    pub fn append_batch_u64_borrowed(
        &mut self,
        entries: &[(&[f64], u64, &HashMap<String, String>)],
        logical_clock: u64,
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            Self::serialize_entry_u64_into(
                &mut self.scratch_buf,
                *id,
                vector,
                metadata,
                logical_clock,
            )?;
            self.write_scratch_packet()?;
        }
        self.apply_sync_policy()
    }

    /// Force sync all changes to disk immediately.
    pub fn sync(&mut self) -> io::Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        self.last_fsync_time = std::time::Instant::now();
        Ok(())
    }
}

impl Drop for Wal {
    fn drop(&mut self) {
        let _ = self.file.flush();
    }
}

impl Wal {
    pub fn replay<F>(path: &Path, mut callback: F) -> io::Result<()>
    where
        F: FnMut(WalEntry),
    {
        if !path.exists() {
            return Ok(());
        }

        let file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let mut reader = BufReader::new(file);
        let mut valid_pos = 0u64;

        loop {
            // Check for EOF or Magic Byte
            let magic = match reader.read_u8() {
                Ok(b) => b,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            if magic == WAL_V3_MAGIC {
                // --- V3 Format ---
                let Ok(len) = reader.read_u32::<LittleEndian>() else {
                    break; // Partial header
                };
                let Ok(stored_crc) = reader.read_u32::<LittleEndian>() else {
                    break; // Partial header
                };

                let mut payload = vec![0u8; len as usize];
                if reader.read_exact(&mut payload).is_err() {
                    break; // Partial payload
                }

                // Verify CRC
                let mut hasher = Hasher::new();
                hasher.update(&payload);
                if hasher.finalize() != stored_crc {
                    eprintln!("⚠️ WAL Corruption detected (CRC mismatch) at offset {valid_pos}. Truncating.");
                    break;
                }

                // Parse Payload
                let mut cursor = Cursor::new(payload);
                match Self::parse_entry(&mut cursor) {
                    Ok(entry) => callback(entry),
                    Err(e) => eprintln!("⚠️ Failed to parse WAL entry body: {e}"),
                }

                // Update valid position (Magic(1) + Len(4) + CRC(4) + Payload(len))
                valid_pos += 1 + 4 + 4 + u64::from(len);
            } else {
                // --- Legacy Format (V1/V2) ---
                // Magic byte is actually OpCode (1 or 2)
                let opcode = magic;

                // Parse based on opcode. Legacy format lacks length prefix, making recovery difficult on failure.

                if let Ok((entry, bytes_read)) = Self::parse_legacy_entry(opcode, &mut reader) {
                    callback(entry);
                    valid_pos += 1 + bytes_read as u64; // 1 for opcode
                } else {
                    eprintln!("⚠️ Legacy WAL Corruption or EOF at offset {valid_pos}. Truncating.");
                    break;
                }
            }
        }

        // Truncate if needed
        if valid_pos < file_len {
            eprintln!("🔥 Healing WAL: Truncating from {file_len} bytes to {valid_pos} bytes.");
            let file = OpenOptions::new().write(true).open(path)?;
            file.set_len(valid_pos)?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn parse_entry(cursor: &mut Cursor<Vec<u8>>) -> io::Result<WalEntry> {
        let opcode = cursor.read_u8()?;
        match opcode {
            4 => {
                let id = cursor.read_u64::<LittleEndian>()?;
                let logical_clock = cursor.read_u64::<LittleEndian>()?;
                let vec_len = cursor.read_u32::<LittleEndian>()? as usize;
                let mut vector = vec![0.0f64; vec_len];
                #[cfg(target_endian = "little")]
                {
                    let bytes = unsafe {
                        std::slice::from_raw_parts_mut(
                            vector.as_mut_ptr().cast::<u8>(),
                            vec_len * 8,
                        )
                    };
                    cursor.read_exact(bytes)?;
                }
                #[cfg(not(target_endian = "little"))]
                {
                    for val in &mut vector {
                        *val = cursor.read_f64::<LittleEndian>()?;
                    }
                }
                let meta_len = cursor.read_u32::<LittleEndian>()?;
                let mut metadata = HashMap::with_capacity(meta_len as usize);
                for _ in 0..meta_len {
                    let k_len = cursor.read_u32::<LittleEndian>()?;
                    let mut k_buf = vec![0u8; k_len as usize];
                    cursor.read_exact(&mut k_buf)?;
                    let key = String::from_utf8(k_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let v_len = cursor.read_u32::<LittleEndian>()?;
                    let mut v_buf = vec![0u8; v_len as usize];
                    cursor.read_exact(&mut v_buf)?;
                    let val = String::from_utf8(v_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    metadata.insert(key, val);
                }
                Ok(WalEntry::Insert64 {
                    id,
                    vector,
                    metadata,
                    logical_clock,
                })
            }
            3 => {
                let id = cursor.read_u32::<LittleEndian>()?;
                let logical_clock = cursor.read_u64::<LittleEndian>()?;
                let vec_len = cursor.read_u32::<LittleEndian>()? as usize;
                let mut vector = vec![0.0f64; vec_len];
                #[cfg(target_endian = "little")]
                {
                    let bytes = unsafe {
                        std::slice::from_raw_parts_mut(
                            vector.as_mut_ptr().cast::<u8>(),
                            vec_len * 8,
                        )
                    };
                    cursor.read_exact(bytes)?;
                }
                #[cfg(not(target_endian = "little"))]
                {
                    for val in &mut vector {
                        *val = cursor.read_f64::<LittleEndian>()?;
                    }
                }
                let meta_len = cursor.read_u32::<LittleEndian>()?;
                let mut metadata = HashMap::with_capacity(meta_len as usize);
                for _ in 0..meta_len {
                    let k_len = cursor.read_u32::<LittleEndian>()?;
                    let mut k_buf = vec![0u8; k_len as usize];
                    cursor.read_exact(&mut k_buf)?;
                    let key = String::from_utf8(k_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let v_len = cursor.read_u32::<LittleEndian>()?;
                    let mut v_buf = vec![0u8; v_len as usize];
                    cursor.read_exact(&mut v_buf)?;
                    let val = String::from_utf8(v_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    metadata.insert(key, val);
                }
                Ok(WalEntry::Insert {
                    id,
                    vector,
                    metadata,
                    logical_clock,
                })
            }
            2 => {
                let id = cursor.read_u32::<LittleEndian>()?;
                let vec_len = cursor.read_u32::<LittleEndian>()? as usize;
                let mut vector = vec![0.0f64; vec_len];
                #[cfg(target_endian = "little")]
                {
                    let bytes = unsafe {
                        std::slice::from_raw_parts_mut(
                            vector.as_mut_ptr().cast::<u8>(),
                            vec_len * 8,
                        )
                    };
                    cursor.read_exact(bytes)?;
                }
                #[cfg(not(target_endian = "little"))]
                {
                    for val in &mut vector {
                        *val = cursor.read_f64::<LittleEndian>()?;
                    }
                }
                let meta_len = cursor.read_u32::<LittleEndian>()?;
                let mut metadata = HashMap::with_capacity(meta_len as usize);
                for _ in 0..meta_len {
                    let k_len = cursor.read_u32::<LittleEndian>()?;
                    let mut k_buf = vec![0u8; k_len as usize];
                    cursor.read_exact(&mut k_buf)?;
                    let key = String::from_utf8(k_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let v_len = cursor.read_u32::<LittleEndian>()?;
                    let mut v_buf = vec![0u8; v_len as usize];
                    cursor.read_exact(&mut v_buf)?;
                    let val = String::from_utf8(v_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    metadata.insert(key, val);
                }
                // Legacy V2 inside V3 container: default clock 0
                Ok(WalEntry::Insert {
                    id,
                    vector,
                    metadata,
                    logical_clock: 0,
                })
            }
            5 => {
                let id = cursor.read_u64::<LittleEndian>()?;
                let logical_clock = cursor.read_u64::<LittleEndian>()?;
                Ok(WalEntry::Delete { id, logical_clock })
            }
            6 => {
                let id = cursor.read_u64::<LittleEndian>()?;
                let logical_clock = cursor.read_u64::<LittleEndian>()?;
                let meta_len = cursor.read_u32::<LittleEndian>()?;
                let mut metadata = HashMap::with_capacity(meta_len as usize);
                for _ in 0..meta_len {
                    let k_len = cursor.read_u32::<LittleEndian>()?;
                    let mut k_buf = vec![0u8; k_len as usize];
                    cursor.read_exact(&mut k_buf)?;
                    let key = String::from_utf8(k_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let v_len = cursor.read_u32::<LittleEndian>()?;
                    let mut v_buf = vec![0u8; v_len as usize];
                    cursor.read_exact(&mut v_buf)?;
                    let val = String::from_utf8(v_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    metadata.insert(key, val);
                }
                Ok(WalEntry::UpdatePayload {
                    id,
                    metadata,
                    logical_clock,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown OpCode in Payload",
            )),
        }
    }

    fn parse_legacy_entry<R: Read>(opcode: u8, reader: &mut R) -> io::Result<(WalEntry, usize)> {
        // Returns Entry and bytes read (excluding opcode)
        let mut bytes_read = 0;
        match opcode {
            1 => {
                let id = reader.read_u32::<LittleEndian>()?;
                bytes_read += 4;
                let len = reader.read_u32::<LittleEndian>()?;
                bytes_read += 4;
                let mut vector = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    vector.push(reader.read_f64::<LittleEndian>()?);
                    bytes_read += 8;
                }
                Ok((
                    WalEntry::Insert {
                        id,
                        vector,
                        metadata: HashMap::new(),
                        logical_clock: 0,
                    },
                    bytes_read,
                ))
            }
            2 => {
                // Standard V2
                let id = reader.read_u32::<LittleEndian>()?;
                bytes_read += 4;

                let vec_len = reader.read_u32::<LittleEndian>()?;
                bytes_read += 4;
                let mut vector = Vec::with_capacity(vec_len as usize);
                for _ in 0..vec_len {
                    vector.push(reader.read_f64::<LittleEndian>()?);
                    bytes_read += 8;
                }

                let meta_len = reader.read_u32::<LittleEndian>()?;
                bytes_read += 4;
                let mut metadata = HashMap::with_capacity(meta_len as usize);
                for _ in 0..meta_len {
                    let k_len = reader.read_u32::<LittleEndian>()?;
                    bytes_read += 4;
                    let mut k_buf = vec![0u8; k_len as usize];
                    reader.read_exact(&mut k_buf)?;
                    bytes_read += k_len as usize;
                    let key = String::from_utf8(k_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let v_len = reader.read_u32::<LittleEndian>()?;
                    bytes_read += 4;
                    let mut v_buf = vec![0u8; v_len as usize];
                    reader.read_exact(&mut v_buf)?;
                    bytes_read += v_len as usize;
                    let val = String::from_utf8(v_buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    metadata.insert(key, val);
                }
                Ok((
                    WalEntry::Insert {
                        id,
                        vector,
                        metadata,
                        logical_clock: 0,
                    },
                    bytes_read,
                ))
            }
            5 => {
                let id = reader.read_u64::<LittleEndian>()?;
                bytes_read += 8;
                let logical_clock = reader.read_u64::<LittleEndian>()?;
                bytes_read += 8;
                Ok((WalEntry::Delete { id, logical_clock }, bytes_read))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown Legacy OpCode",
            )),
        }
    }

    /// Quickly counts entries in a frozen WAL file without full deserialization.
    pub fn pending_entries_at_path(path: &Path) -> u64 {
        let mut count = 0;
        let _ = Self::replay(path, |_| count += 1);
        count
    }
}
