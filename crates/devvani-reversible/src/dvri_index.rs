// .dvri — Devvani Reversible Index format
// Stores a mapping: OpId -> (dvr_file_path, byte_offset, record_length)
// Used to locate specific ops in .dvr files without reading the whole file.
//
// File layout:
//   [4 bytes magic]      = 0x44 0x56 0x52 0x49  ("DVRI")
//   [8 bytes entry_count] = number of index entries (u64 LE)
//   [entries...]
//     each entry:
//       [8 bytes op_id]        u64 LE
//       [4 bytes path_len]     u32 LE
//       [path_len bytes]       UTF-8 file path
//       [8 bytes byte_offset]  u64 LE
//       [4 bytes record_len]   u32 LE

use crate::error::ReversibleError;
use crate::types::OpId;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

pub const DVRI_MAGIC: [u8; 4] = [0x44, 0x56, 0x52, 0x49];

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub op_id: OpId,
    pub dvr_path: PathBuf,
    pub byte_offset: u64,
    pub record_len: u32,
}

#[derive(Debug, Default)]
pub struct DvriIndex {
    entries: HashMap<OpId, IndexEntry>,
}

impl DvriIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.op_id, entry);
    }

    pub fn lookup(&self, op_id: OpId) -> Option<&IndexEntry> {
        self.entries.get(&op_id)
    }

    pub fn remove(&mut self, op_id: OpId) -> Option<IndexEntry> {
        self.entries.remove(&op_id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), ReversibleError> {
        writer
            .write_all(&DVRI_MAGIC)
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("write dvri magic failed: {}", e),
            })?;
        let count = self.entries.len() as u64;
        writer
            .write_all(&count.to_le_bytes())
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("write entry count failed: {}", e),
            })?;
        for entry in self.entries.values() {
            writer.write_all(&entry.op_id.to_le_bytes()).map_err(|e| {
                ReversibleError::PurgeFailed {
                    reason: format!("write op_id failed: {}", e),
                }
            })?;
            let path_str = entry.dvr_path.to_string_lossy();
            let path_bytes = path_str.as_bytes();
            let path_len = path_bytes.len() as u32;
            writer.write_all(&path_len.to_le_bytes()).map_err(|e| {
                ReversibleError::PurgeFailed {
                    reason: format!("write path_len failed: {}", e),
                }
            })?;
            writer
                .write_all(path_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("write path failed: {}", e),
                })?;
            writer
                .write_all(&entry.byte_offset.to_le_bytes())
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("write byte_offset failed: {}", e),
                })?;
            writer
                .write_all(&entry.record_len.to_le_bytes())
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("write record_len failed: {}", e),
                })?;
        }
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ReversibleError> {
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("read dvri magic failed: {}", e),
            })?;
        if magic != DVRI_MAGIC {
            return Err(ReversibleError::PurgeFailed {
                reason: "invalid .dvri magic bytes".to_string(),
            });
        }
        let mut count_bytes = [0u8; 8];
        reader
            .read_exact(&mut count_bytes)
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("read entry count failed: {}", e),
            })?;
        let count = u64::from_le_bytes(count_bytes) as usize;
        let mut index = DvriIndex::new();
        for _ in 0..count {
            let mut op_id_bytes = [0u8; 8];
            reader
                .read_exact(&mut op_id_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("read op_id failed: {}", e),
                })?;
            let op_id = u64::from_le_bytes(op_id_bytes);
            let mut path_len_bytes = [0u8; 4];
            reader
                .read_exact(&mut path_len_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("read path_len failed: {}", e),
                })?;
            let path_len = u32::from_le_bytes(path_len_bytes) as usize;
            let mut path_bytes = vec![0u8; path_len];
            reader
                .read_exact(&mut path_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("read path failed: {}", e),
                })?;
            let path_str =
                String::from_utf8(path_bytes).map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("invalid path utf8: {}", e),
                })?;
            let mut offset_bytes = [0u8; 8];
            reader
                .read_exact(&mut offset_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("read byte_offset failed: {}", e),
                })?;
            let byte_offset = u64::from_le_bytes(offset_bytes);
            let mut rec_len_bytes = [0u8; 4];
            reader
                .read_exact(&mut rec_len_bytes)
                .map_err(|e| ReversibleError::PurgeFailed {
                    reason: format!("read record_len failed: {}", e),
                })?;
            let record_len = u32::from_le_bytes(rec_len_bytes);
            index.insert(IndexEntry {
                op_id,
                dvr_path: PathBuf::from(path_str),
                byte_offset,
                record_len,
            });
        }
        Ok(index)
    }
}
