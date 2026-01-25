use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read, BufWriter, BufReader};
use std::path::Path;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

#[derive(Debug)]
pub struct Wal {
    file: BufWriter<File>,
}

#[derive(Debug)]
pub enum WalEntry {
    Insert { id: u32, vector: Vec<f64> },
}

impl Wal {
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            file: BufWriter::new(file),
        })
    }

    pub fn append(&mut self, id: u32, vector: &[f64]) -> io::Result<()> {
        self.file.write_u8(1)?; // OpCode 1 = Insert
        self.file.write_u32::<LittleEndian>(id)?;
        self.file.write_u32::<LittleEndian>(vector.len() as u32)?;
        for &val in vector {
            self.file.write_f64::<LittleEndian>(val)?;
        }
        self.file.flush()?; 
        Ok(())
    }

    pub fn replay<F>(path: &Path, mut callback: F) -> io::Result<()> 
    where F: FnMut(WalEntry) 
    {
        if !path.exists() { return Ok(()); }
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
                    let id = reader.read_u32::<LittleEndian>()?;
                    let len = reader.read_u32::<LittleEndian>()?;
                    let mut vector = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        vector.push(reader.read_f64::<LittleEndian>()?);
                    }
                    callback(WalEntry::Insert { id, vector });
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown OpCode")),
            }
        }
        Ok(())
    }
}
