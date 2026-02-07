use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Debug)]
pub struct Wal {
    file: BufWriter<File>,
}

use std::collections::HashMap;

#[derive(Debug)]
pub enum WalEntry {
    Insert {
        id: u32,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
    },
}

impl Wal {
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: BufWriter::new(file),
        })
    }

    pub fn append(
        &mut self,
        id: u32,
        vector: &[f64],
        metadata: &HashMap<String, String>,
    ) -> io::Result<()> {
        // OpCode 2 = Insert V2 (With Metadata)
        self.file.write_u8(2)?;
        self.file.write_u32::<LittleEndian>(id)?;

        // Vector
        self.file.write_u32::<LittleEndian>(vector.len() as u32)?;
        for &val in vector {
            self.file.write_f64::<LittleEndian>(val)?;
        }

        // Metadata
        self.file.write_u32::<LittleEndian>(metadata.len() as u32)?;
        for (k, v) in metadata {
            let k_bytes = k.as_bytes();
            self.file.write_u32::<LittleEndian>(k_bytes.len() as u32)?;
            self.file.write_all(k_bytes)?;

            let v_bytes = v.as_bytes();
            self.file.write_u32::<LittleEndian>(v_bytes.len() as u32)?;
            self.file.write_all(v_bytes)?;
        }

        self.file.flush()?;
        Ok(())
    }

    pub fn append_batch(
        &mut self,
        entries: &[(Vec<f64>, u32, HashMap<String, String>)],
    ) -> io::Result<()> {
        for (vector, id, metadata) in entries {
            // OpCode 2 = Insert V2 (With Metadata)
            self.file.write_u8(2)?;
            self.file.write_u32::<LittleEndian>(*id)?;

            // Vector
            self.file.write_u32::<LittleEndian>(vector.len() as u32)?;
            for &val in vector {
                self.file.write_f64::<LittleEndian>(val)?;
            }

            // Metadata
            self.file.write_u32::<LittleEndian>(metadata.len() as u32)?;
            for (k, v) in metadata {
                let k_bytes = k.as_bytes();
                self.file.write_u32::<LittleEndian>(k_bytes.len() as u32)?;
                self.file.write_all(k_bytes)?;

                let v_bytes = v.as_bytes();
                self.file.write_u32::<LittleEndian>(v_bytes.len() as u32)?;
                self.file.write_all(v_bytes)?;
            }
        }
        self.file.flush()?;
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
        let mut reader = BufReader::new(file);

        loop {
            let opcode = match reader.read_u8() {
                Ok(byte) => byte,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            match opcode {
                1 => {
                    // Legacy (No Metadata)
                    let id = reader.read_u32::<LittleEndian>()?;
                    let len = reader.read_u32::<LittleEndian>()?;
                    let mut vector = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        vector.push(reader.read_f64::<LittleEndian>()?);
                    }
                    callback(WalEntry::Insert {
                        id,
                        vector,
                        metadata: HashMap::new(),
                    });
                }
                2 => {
                    // V2 (With Metadata)
                    let id = reader.read_u32::<LittleEndian>()?;

                    // Vector
                    let vec_len = reader.read_u32::<LittleEndian>()?;
                    let mut vector = Vec::with_capacity(vec_len as usize);
                    for _ in 0..vec_len {
                        vector.push(reader.read_f64::<LittleEndian>()?);
                    }

                    // Metadata
                    let meta_len = reader.read_u32::<LittleEndian>()?;
                    let mut metadata = HashMap::with_capacity(meta_len as usize);
                    for _ in 0..meta_len {
                        // Key
                        let k_len = reader.read_u32::<LittleEndian>()? as usize;
                        let mut k_buf = vec![0u8; k_len];
                        reader.read_exact(&mut k_buf)?;
                        let key = String::from_utf8(k_buf)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        // Value
                        let v_len = reader.read_u32::<LittleEndian>()? as usize;
                        let mut v_buf = vec![0u8; v_len];
                        reader.read_exact(&mut v_buf)?;
                        let val = String::from_utf8(v_buf)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        metadata.insert(key, val);
                    }

                    callback(WalEntry::Insert {
                        id,
                        vector,
                        metadata,
                    });
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown OpCode")),
            }
        }
        Ok(())
    }
}
