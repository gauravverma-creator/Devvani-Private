use crate::error::ReversibleError;
use crate::operation_log::OperationLog;
use crate::tiered_storage::TieredStorage;
use crate::types::{AncillaId, EngineStats, OpId, ReversibleOp, SideEffectMarker, UncomputeResult};
use crate::window::WindowConfig;
use std::path::Path;

/// ReversibleEngine is the unified public API for the entire reversible compute system.
///
/// It combines:
///   - TieredStorage (RAM + SSD)
///   - OperationLog (forward chain + uncompute chain)
///
/// Users of this crate interact only with ReversibleEngine.
/// All internal tier management is transparent.
#[derive(Debug)]
pub struct ReversibleEngine {
    storage: TieredStorage,
    log: OperationLog,
}

impl ReversibleEngine {
    pub fn new(
        ram_capacity_bytes: usize,
        window_config: WindowConfig,
        ssd_base_dir: impl AsRef<Path>,
        ssd_coalesce_threshold: usize,
    ) -> Result<Self, ReversibleError> {
        Ok(Self {
            storage: TieredStorage::new(
                ram_capacity_bytes,
                window_config,
                ssd_base_dir,
                ssd_coalesce_threshold,
            )?,
            log: OperationLog::new(),
        })
    }

    pub fn with_defaults(ssd_base_dir: impl AsRef<Path>) -> Result<Self, ReversibleError> {
        Ok(Self {
            storage: TieredStorage::with_defaults(ssd_base_dir)?,
            log: OperationLog::new(),
        })
    }

    /// Record a new reversible operation.
    pub fn record(
        &mut self,
        forward_fn_name: impl Into<String>,
        inverse_fn_name: impl Into<String>,
        input_snapshot: Vec<u8>,
        output_snapshot: Vec<u8>,
        dependencies: Vec<OpId>,
    ) -> Result<(OpId, AncillaId), ReversibleError> {
        let fwd = forward_fn_name.into();
        let inv = inverse_fn_name.into();

        let (op_id, ancilla_id) = self.storage.record(
            fwd.clone(),
            inv.clone(),
            input_snapshot.clone(),
            output_snapshot.clone(),
            dependencies.clone(),
        )?;

        let op = ReversibleOp {
            id: op_id,
            forward_fn_name: fwd,
            inverse_fn_name: inv,
            input_snapshot,
            output_snapshot,
            ancilla_id,
            timestamp: op_id,
            is_consumed: false,
            dependencies,
        };
        self.log.append(op);

        Ok((op_id, ancilla_id))
    }

    /// Perform uncomputation for a specific op.
    pub fn uncompute(&mut self, op_id: OpId) -> Result<UncomputeResult, ReversibleError> {
        let result = self.storage.uncompute(op_id)?;
        self.log.record_uncompute_result(result.clone());
        Ok(result)
    }

    /// Uncompute all ops in the log in reverse order (full uncomputation).
    pub fn uncompute_all(&mut self) -> Result<Vec<UncomputeResult>, ReversibleError> {
        let chain: Vec<OpId> = self.log.uncompute_chain().to_vec();
        let mut results = Vec::new();
        for op_id in chain {
            let result = self.storage.uncompute(op_id)?;
            self.log.record_uncompute_result(result.clone());
            results.push(result);
        }
        Ok(results)
    }

    /// Record a side effect (non-reversible, tracked for audit).
    pub fn record_side_effect(&mut self, marker: SideEffectMarker) {
        self.log.record_side_effect(marker);
    }

    /// Flush buffered SSD writes to disk.
    pub fn flush(&mut self) -> Result<(), ReversibleError> {
        self.storage.flush_ssd()
    }

    pub fn stats(&self) -> EngineStats {
        self.storage.stats()
    }

    pub fn log(&self) -> &OperationLog {
        &self.log
    }

    pub fn storage(&self) -> &TieredStorage {
        &self.storage
    }
}
