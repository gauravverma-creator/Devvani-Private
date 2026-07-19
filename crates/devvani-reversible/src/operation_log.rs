use crate::types::{OpId, ReversibleOp, SideEffectMarker, UncomputeResult};
use std::collections::HashMap;

/// OperationLog maintains the full forward chain and uncompute chain.
/// It is separate from AncillaStore — AncillaStore manages memory slots,
/// OperationLog manages the logical sequence of operations.
///
/// Forward chain: ops in the order they were executed
/// Uncompute chain: ops in the reverse order they should be uncomputed
#[derive(Debug, Default)]
pub struct OperationLog {
    forward_chain: Vec<OpId>,
    uncompute_chain: Vec<OpId>, // reverse of forward, populated on demand
    op_index: HashMap<OpId, ReversibleOp>,
    side_effects: Vec<SideEffectMarker>,
    uncompute_results: Vec<UncomputeResult>,
}

impl OperationLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an op to the forward chain
    pub fn append(&mut self, op: ReversibleOp) {
        let id = op.id;
        self.forward_chain.push(id);
        self.op_index.insert(id, op);
        // Invalidate uncompute chain cache
        self.uncompute_chain.clear();
    }

    /// Record a side effect marker (not reversible, just tracked)
    pub fn record_side_effect(&mut self, marker: SideEffectMarker) {
        self.side_effects.push(marker);
    }

    /// Get the uncompute chain (reverse of forward chain, lazily built)
    pub fn uncompute_chain(&mut self) -> &[OpId] {
        if self.uncompute_chain.is_empty() && !self.forward_chain.is_empty() {
            self.uncompute_chain = self.forward_chain.iter().rev().copied().collect();
        }
        &self.uncompute_chain
    }

    /// Get a specific op by id
    pub fn get_op(&self, op_id: OpId) -> Option<&ReversibleOp> {
        self.op_index.get(&op_id)
    }

    /// Record the result of an uncompute attempt
    pub fn record_uncompute_result(&mut self, result: UncomputeResult) {
        self.uncompute_results.push(result);
    }

    pub fn forward_chain(&self) -> &[OpId] {
        &self.forward_chain
    }

    pub fn side_effects(&self) -> &[SideEffectMarker] {
        &self.side_effects
    }

    pub fn uncompute_results(&self) -> &[UncomputeResult] {
        &self.uncompute_results
    }

    pub fn total_ops(&self) -> usize {
        self.forward_chain.len()
    }

    pub fn total_side_effects(&self) -> usize {
        self.side_effects.len()
    }
}
