// .dvr — Devvani Reversible binary format
// File layout:
//   [4 bytes magic]     = 0x44 0x56 0x52 0x00  ("DVR\0")
//   [4 bytes version]   = 0x00 0x00 0x00 0x01
//   [8 bytes op_count]  = number of ReversibleOp records (little-endian u64)
//   [records...]        = each record is length-prefixed JSON (4-byte u32 LE length + UTF-8 bytes)

use crate::error::ReversibleError;
use crate::types::ReversibleOp;
use std::io::{Read, Write};

pub const DVR_MAGIC: [u8; 4] = [0x44, 0x56, 0x52, 0x00];
pub const DVR_VERSION: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

/// Serialize a list of ReversibleOps into .dvr binary format
pub fn write_dvr<W: Write>(writer: &mut W, ops: &[ReversibleOp]) -> Result<(), ReversibleError> {
    writer.write_all(&DVR_MAGIC).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("write magic failed: {}", e),
    })?;
    writer.write_all(&DVR_VERSION).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("write version failed: {}", e),
    })?;
    let op_count = ops.len() as u64;
    writer.write_all(&op_count.to_le_bytes()).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("write op_count failed: {}", e),
    })?;
    for op in ops {
        let json = serde_json::to_string(op).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("serialize op {} failed: {}", op.id, e),
        })?;
        let bytes = json.as_bytes();
        let len = bytes.len() as u32;
        writer.write_all(&len.to_le_bytes()).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("write record length failed: {}", e),
        })?;
        writer.write_all(bytes).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("write record data failed: {}", e),
        })?;
    }
    Ok(())
}

/// Deserialize a .dvr file back into a Vec<ReversibleOp>
pub fn read_dvr<R: Read>(reader: &mut R) -> Result<Vec<ReversibleOp>, ReversibleError> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("read magic failed: {}", e),
    })?;
    if magic != DVR_MAGIC {
        return Err(ReversibleError::PurgeFailed {
            reason: "invalid .dvr magic bytes".to_string(),
        });
    }
    let mut version = [0u8; 4];
    reader.read_exact(&mut version).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("read version failed: {}", e),
    })?;
    let mut count_bytes = [0u8; 8];
    reader.read_exact(&mut count_bytes).map_err(|e| ReversibleError::PurgeFailed {
        reason: format!("read op_count failed: {}", e),
    })?;
    let op_count = u64::from_le_bytes(count_bytes) as usize;
    let mut ops = Vec::with_capacity(op_count);
    for _ in 0..op_count {
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("read record length failed: {}", e),
        })?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("read record data failed: {}", e),
        })?;
        let op: ReversibleOp = serde_json::from_slice(&data).map_err(|e| ReversibleError::PurgeFailed {
            reason: format!("deserialize op failed: {}", e),
        })?;
        ops.push(op);
    }
    Ok(ops)
}
