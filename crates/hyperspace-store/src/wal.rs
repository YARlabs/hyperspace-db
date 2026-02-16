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
}

impl Wal {
    pub fn new(path: &Path, mode: WalSyncMode) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: BufWriter::new(file),
            mode,
        })
    }

    fn serialize_entry(
        id: u32,
        vector: &[f64],
        metadata: &HashMap<String, String>,
        logical_clock: u64,
    ) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        // Internal Format: OpCode 3 (Insert V3 with clock)
        buf.write_u8(3)?;
        buf.write_u32::<LittleEndian>(id)?;
        buf.write_u64::<LittleEndian>(logical_clock)?;

        // Vector
        buf.write_u32::<LittleEndian>(vector.len() as u32)?;
        for &val in vector {
            buf.write_f64::<LittleEndian>(val)?;
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
        Ok(buf)
    }

    fn write_packet(&mut self, payload: &[u8]) -> io::Result<()> {
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

        // Flush to OS cache (always)
        self.file.flush()?;

        // Fsync to Disk (if Strict)
        if self.mode == WalSyncMode::Strict {
            self.file.get_ref().sync_all()?;
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
        let payload = Self::serialize_entry(id, vector, metadata, logical_clock)?;
        self.write_packet(&payload)
    }

    pub fn append_batch(
        &mut self,
        entries: &[(Vec<f64>, u32, HashMap<String, String>)],
        logical_clock: u64,
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            let payload = Self::serialize_entry(*id, vector, metadata, logical_clock)?;
            self.write_packet(&payload)?;
        }
        Ok(())
    }

    /// Force sync all changes to disk immediately.
    pub fn sync(&mut self) -> io::Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        Ok(())
    }

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

    fn parse_entry(cursor: &mut Cursor<Vec<u8>>) -> io::Result<WalEntry> {
        let opcode = cursor.read_u8()?;
        match opcode {
            3 => {
                let id = cursor.read_u32::<LittleEndian>()?;
                let logical_clock = cursor.read_u64::<LittleEndian>()?;
                let vec_len = cursor.read_u32::<LittleEndian>()?;
                let mut vector = Vec::with_capacity(vec_len as usize);
                for _ in 0..vec_len {
                    vector.push(cursor.read_f64::<LittleEndian>()?);
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
                let vec_len = cursor.read_u32::<LittleEndian>()?;
                let mut vector = Vec::with_capacity(vec_len as usize);
                for _ in 0..vec_len {
                    vector.push(cursor.read_f64::<LittleEndian>()?);
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
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown Legacy OpCode",
            )),
        }
    }
}
