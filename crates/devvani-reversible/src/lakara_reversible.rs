//! LakaaraReversible — the special Lakāra marker for reversible operations in Devvani.
//!
//! In Devvani's type system, Lakāra controls function scope and async markers.
//! LakaaraReversible extends this with a reversibility marker:
//!
//!   - PratyavartyaLit  — "reversible present" — function is reversible, runs forward
//!   - PratyavartyaLan  — "reversible past" — function result can be uncomputed
//!   - PratyavartyaLrt  — "reversible future" — function will record for later uncompute
//!   - AnapravartyaLot  — "irreversible imperative" — side effect, tracked but not reversible
//!
//! These integrate with the existing Lakāra system in devvani-typesystem.
//! This module defines the types; the actual type system integration is in devvani-typesystem.

use serde::{Deserialize, Serialize};

/// Reversibility annotation on a Devvani Dhātu (function)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LakaaraReversible {
    /// PratyavartyaLit — reversible present tense
    /// The Dhātu records its operation and can be uncomputed later.
    PratyavartyaLit {
        /// Name of the inverse Dhātu (must exist in scope)
        inverse_dhatu: String,
    },
    /// PratyavartyaLan — reversible past tense
    /// The Dhātu's result is already recorded; uncomputation restores prior state.
    PratyavartyaLan {
        inverse_dhatu: String,
        /// OpId of the recorded operation (set at runtime)
        recorded_op_id: Option<u64>,
    },
    /// PratyavartyaLrt — reversible future
    /// The Dhātu will execute and record; uncomputation is deferred.
    PratyavartyaLrt { inverse_dhatu: String },
    /// AnapravartyaLot — irreversible imperative (side effect)
    /// I/O, file writes, network calls. Tracked in side effect log only.
    AnapravartyaLot { effect_description: String },
}

impl LakaaraReversible {
    /// Returns true if this Lakāra represents a reversible operation
    pub fn is_reversible(&self) -> bool {
        !matches!(self, LakaaraReversible::AnapravartyaLot { .. })
    }

    /// Returns the inverse Dhātu name if applicable
    pub fn inverse_dhatu(&self) -> Option<&str> {
        match self {
            LakaaraReversible::PratyavartyaLit { inverse_dhatu }
            | LakaaraReversible::PratyavartyaLan { inverse_dhatu, .. }
            | LakaaraReversible::PratyavartyaLrt { inverse_dhatu } => Some(inverse_dhatu),
            LakaaraReversible::AnapravartyaLot { .. } => None,
        }
    }

    /// Returns the Lakāra name as a Sanskrit string
    pub fn sanskrit_name(&self) -> &'static str {
        match self {
            LakaaraReversible::PratyavartyaLit { .. } => "प्रत्यावर्त्यलिट्",
            LakaaraReversible::PratyavartyaLan { .. } => "प्रत्यावर्त्यलङ्",
            LakaaraReversible::PratyavartyaLrt { .. } => "प्रत्यावर्त्यलृट्",
            LakaaraReversible::AnapravartyaLot { .. } => "अनप्रावर्त्यलोट्",
        }
    }

    /// Returns the Lakāra name as an ASCII string (for diagnostics)
    pub fn ascii_name(&self) -> &'static str {
        match self {
            LakaaraReversible::PratyavartyaLit { .. } => "PratyavartyaLit",
            LakaaraReversible::PratyavartyaLan { .. } => "PratyavartyaLan",
            LakaaraReversible::PratyavartyaLrt { .. } => "PratyavartyaLrt",
            LakaaraReversible::AnapravartyaLot { .. } => "AnapravartyaLot",
        }
    }
}

/// Diagnostic codes for LakaaraReversible violations
/// These follow Devvani's Dxxx diagnostic code convention
#[derive(Debug, Clone, PartialEq)]
pub enum ReversibleDiagnostic {
    /// D020: Inverse Dhātu not found in scope
    InverseDhatuNotFound { dhatu_name: String },
    /// D021: PratyavartyaLan used without a recorded OpId
    MissingRecordedOpId { dhatu_name: String },
    /// D022: AnapravartyaLot in a reversible context (side effect in reversible function)
    SideEffectInReversibleContext { effect_description: String },
    /// D023: Uncomputation attempted on AnapravartyaLot
    UncomputeOnIrreversible { dhatu_name: String },
}

impl ReversibleDiagnostic {
    pub fn code(&self) -> &'static str {
        match self {
            ReversibleDiagnostic::InverseDhatuNotFound { .. } => "D020",
            ReversibleDiagnostic::MissingRecordedOpId { .. } => "D021",
            ReversibleDiagnostic::SideEffectInReversibleContext { .. } => "D022",
            ReversibleDiagnostic::UncomputeOnIrreversible { .. } => "D023",
        }
    }

    pub fn message(&self) -> String {
        match self {
            ReversibleDiagnostic::InverseDhatuNotFound { dhatu_name } => {
                format!(
                    "D020: inverse dhatu '{}' not found in current scope",
                    dhatu_name
                )
            }
            ReversibleDiagnostic::MissingRecordedOpId { dhatu_name } => {
                format!("D021: PratyavartyaLan on '{}' has no recorded OpId — was the operation executed?", dhatu_name)
            }
            ReversibleDiagnostic::SideEffectInReversibleContext { effect_description } => {
                format!("D022: side effect '{}' inside a reversible Dhātu — use AnapravartyaLot or move outside", effect_description)
            }
            ReversibleDiagnostic::UncomputeOnIrreversible { dhatu_name } => {
                format!(
                    "D023: cannot uncompute '{}' — marked AnapravartyaLot (irreversible)",
                    dhatu_name
                )
            }
        }
    }
}
