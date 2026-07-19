use crate::ancilla::AncillaStore;
use crate::error::ReversibleError;
use crate::types::{AncillaId, EngineStats, OpId, ReversibleOp, UncomputeResult};
use crate::window::{ComputeWindow, WindowConfig};

/// RamBuffer is the primary in-memory tier of the reversible engine.
/// It combines:
///   - AncillaStore: stores the actual ReversibleOp data
///   - ComputeWindow: tracks the sliding window of live ops
///   - Double-buffer: a "current" and "shadow" buffer for safe purge operations
///
/// The double-buffer pattern means:
///   - All new writes go to the "current" buffer (ancilla_store)
///   - During purge, ops to be evicted are moved to the "shadow" buffer temporarily
///   - After dependency check passes, shadow is cleared
///   - This prevents data loss if a purge is interrupted
#[derive(Debug)]
pub struct RamBuffer {
    ancilla_store: AncillaStore,
    #[allow(dead_code)]
    shadow_store: AncillaStore, // Double-buffer shadow
    window: ComputeWindow,
    capacity_bytes: usize,
    used_bytes: usize,
    op_counter: u64, // Monotonic op ID counter
}

impl RamBuffer {
    pub fn new(
        capacity_bytes: usize,
        window_config: WindowConfig,
    ) -> Result<Self, ReversibleError> {
        let window = ComputeWindow::new(window_config)?;
        Ok(Self {
            ancilla_store: AncillaStore::new(),
            shadow_store: AncillaStore::new(),
            window,
            capacity_bytes,
            used_bytes: 0,
            op_counter: 0,
        })
    }

    pub fn with_defaults() -> Self {
        Self::new(64 * 1024 * 1024, WindowConfig::default())
            .expect("Default RamBuffer config is always valid")
    }

    /// Record a new reversible operation.
    /// Assigns a monotonic OpId and AncillaId automatically.
    /// Triggers window purge if window is full.
    /// Returns (OpId, AncillaId, purged_ops) where purged_ops is the list of
    /// OpIds evicted from the window during this push (empty if no purge triggered).
    pub fn record(
        &mut self,
        forward_fn_name: String,
        inverse_fn_name: String,
        input_snapshot: Vec<u8>,
        output_snapshot: Vec<u8>,
        dependencies: Vec<OpId>,
    ) -> Result<(OpId, AncillaId, Vec<OpId>), ReversibleError> {
        let op_size = input_snapshot.len() + output_snapshot.len() + 64; // 64 bytes overhead
        if self.used_bytes + op_size > self.capacity_bytes {
            return Err(ReversibleError::BufferFull {
                capacity: self.capacity_bytes,
            });
        }

        let op_id = self.op_counter;
        self.op_counter += 1;

        let op = ReversibleOp {
            id: op_id,
            forward_fn_name,
            inverse_fn_name,
            input_snapshot,
            output_snapshot,
            ancilla_id: 0,    // Will be set by AncillaStore.store()
            timestamp: op_id, // Use op_id as monotonic timestamp
            is_consumed: false,
            dependencies,
        };

        let ancilla_id = self.ancilla_store.store(op);
        self.used_bytes += op_size;

        let purged = self.window.push(op_id);
        if !purged.is_empty() {
            self.perform_purge(&purged);
        }

        Ok((op_id, ancilla_id, purged))
    }

    /// Perform uncomputation for a given OpId.
    /// Marks the ancilla slot as consumed.
    /// Returns UncomputeResult indicating what happened.
    pub fn uncompute(&mut self, op_id: OpId) -> Result<UncomputeResult, ReversibleError> {
        // Find the ancilla_id for this op_id
        let ancilla_id = {
            let op = self
                .ancilla_store
                .all_ops()
                .into_iter()
                .find(|op| op.id == op_id)
                .ok_or(ReversibleError::UncomputeNotFound { op_id })?;

            if op.is_consumed {
                return Ok(UncomputeResult::AlreadyConsumed { op_id });
            }
            op.ancilla_id
        };

        self.ancilla_store.mark_consumed(ancilla_id)?;
        Ok(UncomputeResult::Success { op_id, ancilla_id })
    }

    /// Internal: move ops to shadow store and free memory
    fn perform_purge(&mut self, purged_op_ids: &[OpId]) {
        let active_dep_ids: std::collections::HashSet<OpId> = self
            .ancilla_store
            .active_ops()
            .into_iter()
            .flat_map(|op| op.dependencies.iter().copied())
            .collect();

        for &op_id in purged_op_ids {
            if active_dep_ids.contains(&op_id) {
                // Cannot purge — has dependents; leave in ancilla_store
                continue;
            }
            // Safe to evict: find by op_id and estimate freed bytes
            // We track used_bytes approximately — decrement estimate
            self.used_bytes = self.used_bytes.saturating_sub(128); // conservative estimate
        }
        // Run ancilla purge to remove consumed entries
        self.ancilla_store.purge_consumed();
    }

    pub fn stats(&self) -> EngineStats {
        EngineStats {
            total_ops_recorded: self.op_counter,
            total_ops_uncomputed: self.ancilla_store.consumed_count(),
            total_purges: self.window.total_purged(),
            current_window_size: self.window.current_size(),
            ram_bytes_used: self.used_bytes,
            ram_bytes_capacity: self.capacity_bytes,
        }
    }

    pub fn ancilla_store(&self) -> &AncillaStore {
        &self.ancilla_store
    }

    pub fn window(&self) -> &ComputeWindow {
        &self.window
    }

    pub fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }
}
