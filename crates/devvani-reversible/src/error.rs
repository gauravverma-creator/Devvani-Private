use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReversibleError {
    #[error("Window overflow: cannot add operation, window is full (capacity: {capacity})")]
    WindowOverflow { capacity: usize },

    #[error("Uncompute failed: operation id {op_id} not found in ancilla store")]
    UncomputeNotFound { op_id: u64 },

    #[error("Dependency violation: operation {op_id} has {dep_count} dependents, cannot purge")]
    DependencyViolation { op_id: u64, dep_count: usize },

    #[error("Buffer write failed: RAM buffer is at capacity ({capacity} bytes)")]
    BufferFull { capacity: usize },

    #[error("Invalid window config: {reason}")]
    InvalidConfig { reason: String },

    #[error("Operation chain broken: ancilla bit {ancilla_id} was already consumed")]
    AncillaConsumed { ancilla_id: u64 },

    #[error("Purge failed: {reason}")]
    PurgeFailed { reason: String },
}
