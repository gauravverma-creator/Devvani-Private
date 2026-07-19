use crate::error::ReversibleError;
use crate::types::{AncillaId, OpId, ReversibleOp};
use std::collections::HashMap;

/// AncillaStore manages the ancilla bit slots.
/// Each slot stores exactly one ReversibleOp.
/// After uncomputation, the slot is freed (not deleted — the record is marked consumed).
/// The next operation can then reuse the slot via the chain mechanism.
#[derive(Debug, Default)]
pub struct AncillaStore {
    slots: HashMap<AncillaId, ReversibleOp>,
    next_id: AncillaId,
    consumed_count: u64,
}

impl AncillaStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new ancilla slot and store the op. Returns the assigned AncillaId.
    pub fn store(&mut self, mut op: ReversibleOp) -> AncillaId {
        let id = self.next_id;
        self.next_id += 1;
        op.ancilla_id = id;
        self.slots.insert(id, op);
        id
    }

    /// Mark an ancilla slot as consumed (uncomputed). Does NOT delete the record.
    /// Returns error if already consumed.
    pub fn mark_consumed(&mut self, ancilla_id: AncillaId) -> Result<(), ReversibleError> {
        match self.slots.get_mut(&ancilla_id) {
            Some(op) if op.is_consumed => Err(ReversibleError::AncillaConsumed { ancilla_id }),
            Some(op) => {
                op.is_consumed = true;
                self.consumed_count += 1;
                Ok(())
            }
            None => Err(ReversibleError::UncomputeNotFound { op_id: ancilla_id }),
        }
    }

    /// Get a reference to the op stored in a slot
    pub fn get(&self, ancilla_id: AncillaId) -> Option<&ReversibleOp> {
        self.slots.get(&ancilla_id)
    }

    /// Get all ops (consumed and active) — used for audit/debug
    pub fn all_ops(&self) -> Vec<&ReversibleOp> {
        self.slots.values().collect()
    }

    /// Get only active (not yet consumed) ops
    pub fn active_ops(&self) -> Vec<&ReversibleOp> {
        self.slots.values().filter(|op| !op.is_consumed).collect()
    }

    /// Count of active (not consumed) slots
    pub fn active_count(&self) -> usize {
        self.slots.values().filter(|op| !op.is_consumed).count()
    }

    /// Count of consumed slots
    pub fn consumed_count(&self) -> u64 {
        self.consumed_count
    }

    /// Purge consumed slots to free memory. Returns count of slots freed.
    /// Only purges slots with no active dependents.
    pub fn purge_consumed(&mut self) -> usize {
        let consumed_ids: Vec<AncillaId> = self
            .slots
            .values()
            .filter(|op| op.is_consumed)
            .map(|op| op.ancilla_id)
            .collect();

        let active_dependencies: std::collections::HashSet<OpId> = self
            .slots
            .values()
            .filter(|op| !op.is_consumed)
            .flat_map(|op| op.dependencies.iter().copied())
            .collect();

        let purgeable: Vec<AncillaId> = consumed_ids
            .into_iter()
            .filter(|id| !active_dependencies.contains(id))
            .collect();

        let count = purgeable.len();
        for id in purgeable {
            self.slots.remove(&id);
        }
        count
    }

    pub fn total_allocated(&self) -> AncillaId {
        self.next_id
    }
}
