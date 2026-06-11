//! # devvani-reversible
//!
//! The Reversible Compute Engine for Devvani — CORE system.
//!
//! Architecture:
//!   - `types`              — Core types: ReversibleOp, AncillaId, OpId, UncomputeResult
//!   - `error`              — ReversibleError enum
//!   - `ancilla`            — AncillaStore: manages ancilla bit slots
//!   - `window`             — ComputeWindow: sliding window + WindowConfig
//!   - `ram_buffer`         — RamBuffer: double-buffer + ancilla + window combined
//!   - `operation_log`      — OperationLog: forward chain + uncompute chain
//!   - `dvr_format`         — .dvr binary format: serialization/deserialization
//!   - `dvri_index`         — .dvri index format: OpId → disk location mapping
//!   - `ssd_tier`           — SsdTier: write coalescing + disk persistence
//!   - `tiered_storage`     — TieredStorage: RAM + SSD unified
//!   - `engine`             — ReversibleEngine: unified public API
//!   - `sutra`              — SutraSelector: Vedic sutra auto-selection
//!   - `vedic_batch`        — VedicBatchEngine: 70-75% history reduction
//!   - `lakara_reversible`  — LakaaraReversible: reversibility Lakāra markers

pub mod ancilla;
pub mod dvr_format;
pub mod dvri_index;
pub mod engine;
pub mod error;
pub mod lakara_reversible;
pub mod operation_log;
pub mod ram_buffer;
pub mod ssd_tier;
pub mod sutra;
pub mod tiered_storage;
pub mod types;
pub mod vedic_batch;
pub mod window;

// Re-exports
pub use ancilla::AncillaStore;
pub use dvri_index::{DvriIndex, IndexEntry};
pub use engine::ReversibleEngine;
pub use error::ReversibleError;
pub use lakara_reversible::{LakaaraReversible, ReversibleDiagnostic};
pub use operation_log::OperationLog;
pub use ram_buffer::RamBuffer;
pub use ssd_tier::SsdTier;
pub use sutra::{Sutra, SutraSelection, SutraSelector};
pub use tiered_storage::TieredStorage;
pub use types::{
    AncillaId, EngineStats, OpId, ReversibleOp, SideEffectMarker, SideEffectType, UncomputeResult,
};
pub use vedic_batch::{VedicBatchEngine, VedicBatchStats};
pub use window::{ComputeWindow, WindowConfig};

#[cfg(test)]
mod tests;
