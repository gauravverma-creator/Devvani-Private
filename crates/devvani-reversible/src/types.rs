use serde::{Deserialize, Serialize};

/// Unique identifier for any reversible operation
pub type OpId = u64;

/// Unique identifier for an ancilla bit slot
pub type AncillaId = u64;

/// Represents a single reversible computation step.
/// forward_fn_name and inverse_fn_name are string identifiers (Devvani Dhatu names).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReversibleOp {
    pub id: OpId,
    pub forward_fn_name: String, // Devvani Dhatu name e.g. "yoga" (addition)
    pub inverse_fn_name: String, // Inverse Dhatu name e.g. "viyoga" (subtraction)
    pub input_snapshot: Vec<u8>, // Serialized input state before the operation
    pub output_snapshot: Vec<u8>, // Serialized output state after the operation
    pub ancilla_id: AncillaId,   // Which ancilla slot stores this op
    pub timestamp: u64,          // Monotonic counter, not wall clock
    pub is_consumed: bool,       // True after uncomputation has been performed
    pub dependencies: Vec<OpId>, // OpIds this operation depends on
}

/// Represents a side-effect marker — operations that write to I/O, filesystem, or
/// external state. These cannot be fully reversed but are tracked for audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SideEffectMarker {
    pub op_id: OpId,
    pub effect_type: SideEffectType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SideEffectType {
    IoWrite,
    FileSystem,
    NetworkCall,
    ExternalState,
}

/// Result of an uncomputation attempt
#[derive(Debug, Clone, PartialEq)]
pub enum UncomputeResult {
    /// Successfully uncomputed — input restored, ancilla cleared
    Success { op_id: OpId, ancilla_id: AncillaId },
    /// Operation was a side effect — cannot reverse, only logged
    SideEffectSkipped { op_id: OpId },
    /// Operation already consumed — idempotent skip
    AlreadyConsumed { op_id: OpId },
}

/// Statistics about the current engine state
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub total_ops_recorded: u64,
    pub total_ops_uncomputed: u64,
    pub total_purges: u64,
    pub current_window_size: usize,
    pub ram_bytes_used: usize,
    pub ram_bytes_capacity: usize,
}
