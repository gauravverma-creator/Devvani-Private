use crate::dvr_format::{read_dvr, write_dvr};
use crate::dvri_index::{DvriIndex, IndexEntry};
use crate::error::ReversibleError;
use crate::types::{OpId, ReversibleOp};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// SsdTier manages .dvr files and .dvri index on disk.
/// When RAM purges ops, they are written here.
/// Supports write coalescing: ops are batched and written together
/// rather than one file per op.
#[derive(Debug)]
pub struct SsdTier {
    base_dir: PathBuf,
    index: DvriIndex,
    current_dvr_path: PathBuf,
    coalesce_buffer: Vec<ReversibleOp>,
    coalesce_threshold: usize,
    total_written: u64,
}

impl SsdTier {
    /// Create a new SsdTier rooted at base_dir.
    /// coalesce_threshold: how many ops to buffer before flushing to disk (default 32).
    pub fn new(base_dir: impl AsRef<Path>, coalesce_threshold: usize) -> Result<Self, ReversibleError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("create SSD tier dir failed: {}", e),
        })?;
        let current_dvr_path = base_dir.join("segment_0000.dvr");
        Ok(Self {
            base_dir,
            index: DvriIndex::new(),
            current_dvr_path,
            coalesce_buffer: Vec::new(),
            coalesce_threshold,
            total_written: 0,
        })
    }

    pub fn with_default_threshold(base_dir: impl AsRef<Path>) -> Result<Self, ReversibleError> {
        Self::new(base_dir, 32)
    }

    /// Accept an op evicted from RAM. Buffers it for coalesced write.
    /// If buffer reaches coalesce_threshold, flushes to disk immediately.
    pub fn accept(&mut self, op: ReversibleOp) -> Result<(), ReversibleError> {
        self.coalesce_buffer.push(op);
        if self.coalesce_buffer.len() >= self.coalesce_threshold {
            self.flush()?;
        }
        Ok(())
    }

    /// Flush all buffered ops to the current .dvr segment file.
    /// Updates the .dvri index for each flushed op.
    pub fn flush(&mut self) -> Result<(), ReversibleError> {
        if self.coalesce_buffer.is_empty() {
            return Ok(());
        }

        let mut existing_ops: Vec<ReversibleOp> = if self.current_dvr_path.exists() {
            let f = File::open(&self.current_dvr_path).map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("open dvr for read failed: {}", e),
            })?;
            let mut reader = BufReader::new(f);
            read_dvr(&mut reader)?
        } else {
            vec![]
        };

        let header_size: u64 = 16;
        let mut running_offset = header_size;

        for op in &existing_ops {
            let json = serde_json::to_string(op).map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("serialize existing op failed: {}", e),
            })?;
            running_offset += 4 + json.len() as u64;
        }

        for op in &self.coalesce_buffer {
            let json = serde_json::to_string(op).map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("serialize new op failed: {}", e),
            })?;
            let record_len = json.len() as u32;
            self.index.insert(IndexEntry {
                op_id: op.id,
                dvr_path: self.current_dvr_path.clone(),
                byte_offset: running_offset,
                record_len,
            });
            running_offset += 4 + record_len as u64;
        }

        existing_ops.extend(self.coalesce_buffer.drain(..));
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.current_dvr_path)
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("open dvr for write failed: {}", e),
            })?;
        let mut writer = BufWriter::new(f);
        write_dvr(&mut writer, &existing_ops)?;
        self.total_written = self.index.len() as u64;

        self.save_index()?;
        Ok(())
    }

    /// Look up a specific op from disk by OpId using the index.
    pub fn fetch(&self, op_id: OpId) -> Result<Option<ReversibleOp>, ReversibleError> {
        let entry = match self.index.lookup(op_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        let f = File::open(&entry.dvr_path).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("open dvr for fetch failed: {}", e),
        })?;
        let mut reader = BufReader::new(f);
        reader.seek(SeekFrom::Start(entry.byte_offset)).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("seek to op failed: {}", e),
        })?;

        let mut len_bytes = [0u8; 4];
        std::io::Read::read_exact(&mut reader, &mut len_bytes).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("read record len failed: {}", e),
        })?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut data = vec![0u8; len];
        std::io::Read::read_exact(&mut reader, &mut data).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("read record data failed: {}", e),
        })?;
        let op: ReversibleOp = serde_json::from_slice(&data).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("deserialize fetched op failed: {}", e),
        })?;
        Ok(Some(op))
    }

    fn save_index(&self) -> Result<(), ReversibleError> {
        let index_path = self.base_dir.join("index.dvri");
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path)
            .map_err(|e| ReversibleError::PurgeFailed {
                reason: format!("open dvri for write failed: {}", e),
            })?;
        let mut writer = BufWriter::new(f);
        self.index.write(&mut writer)
    }

    pub fn index(&self) -> &DvriIndex {
        &self.index
    }

    pub fn total_written(&self) -> u64 {
        self.total_written
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn coalesce_buffer_len(&self) -> usize {
        self.coalesce_buffer.len()
    }
}
