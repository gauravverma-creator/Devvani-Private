use crate::error::ReversibleError;
use crate::ram_buffer::RamBuffer;
use crate::ssd_tier::SsdTier;
use crate::types::{AncillaId, EngineStats, OpId, ReversibleOp, UncomputeResult};
use crate::window::WindowConfig;
use std::path::Path;

/// TieredStorage combines RamBuffer (hot tier) and SsdTier (warm tier).
///
/// Write path:
///   record() → RamBuffer → if window purges → evicted ops go to SsdTier
///
/// Read path (for uncompute):
///   1. Check RamBuffer first (hot)
///   2. If not found, check SsdTier (warm) via index
///
/// The caller never needs to know which tier an op lives in.
#[derive(Debug)]
pub struct TieredStorage {
    ram: RamBuffer,
    ssd: SsdTier,
}

impl TieredStorage {
    pub fn new(
        ram_capacity_bytes: usize,
        window_config: WindowConfig,
        ssd_base_dir: impl AsRef<Path>,
        ssd_coalesce_threshold: usize,
    ) -> Result<Self, ReversibleError> {
        Ok(Self {
            ram: RamBuffer::new(ram_capacity_bytes, window_config)?,
            ssd: SsdTier::new(ssd_base_dir, ssd_coalesce_threshold)?,
        })
    }

    pub fn with_defaults(ssd_base_dir: impl AsRef<Path>) -> Result<Self, ReversibleError> {
        Self::new(
            64 * 1024 * 1024,
            WindowConfig::default(),
            ssd_base_dir,
            32,
        )
    }

    /// Record a new reversible operation.
    pub fn record(
        &mut self,
        forward_fn_name: String,
        inverse_fn_name: String,
        input_snapshot: Vec<u8>,
        output_snapshot: Vec<u8>,
        dependencies: Vec<OpId>,
    ) -> Result<(OpId, AncillaId), ReversibleError> {
        let (op_id, ancilla_id, purged_ids) = self.ram.record(
            forward_fn_name,
            inverse_fn_name,
            input_snapshot,
            output_snapshot,
            dependencies,
        )?;

        for purged_id in purged_ids {
            let op_opt: Option<ReversibleOp> = self
                .ram
                .ancilla_store()
                .all_ops()
                .into_iter()
                .find(|op| op.id == purged_id)
                .cloned();
            if let Some(op) = op_opt {
                if !op.is_consumed {
                    self.ssd.accept(op)?;
                }
            }
        }

        Ok((op_id, ancilla_id))
    }

    /// Uncompute an operation. Searches RAM first, then SSD.
    pub fn uncompute(&mut self, op_id: OpId) -> Result<UncomputeResult, ReversibleError> {
        let in_ram = self
            .ram
            .ancilla_store()
            .all_ops()
            .into_iter()
            .any(|op| op.id == op_id);

        if in_ram {
            return self.ram.uncompute(op_id);
        }

        match self.ssd.fetch(op_id)? {
            Some(op) if op.is_consumed => Ok(UncomputeResult::AlreadyConsumed { op_id }),
            Some(_) => Ok(UncomputeResult::Success {
                op_id,
                ancilla_id: op_id,
            }),
            None => Err(ReversibleError::UncomputeNotFound { op_id }),
        }
    }

    /// Flush all coalesced SSD writes to disk.
    pub fn flush_ssd(&mut self) -> Result<(), ReversibleError> {
        self.ssd.flush()
    }

    pub fn stats(&self) -> EngineStats {
        self.ram.stats()
    }

    pub fn ram(&self) -> &RamBuffer {
        &self.ram
    }

    pub fn ssd(&self) -> &SsdTier {
        &self.ssd
    }
}
