//! Sutra selection for the Vedic Batch Engine.
//!
//! In Pāṇini's Aṣṭādhyāyī, a Sutra is a compact rule that encodes a transformation.
//! In Devvani's reversible engine, a Sutra describes HOW a batch of operations
//! should be compressed/reduced while preserving reversibility.
//!
//! Primary sutra: Ūrdhva-Tiryagbhyām (ऊर्ध्वतिर्यग्भ्याम्)
//!   Literal: "vertically and crosswise"
//!   In our context: compress op history by combining vertical (sequential) chains
//!   and crosswise (parallel/independent) chains into a single reversible summary op.
//!
//! Fallback sutra: Nikhilam (निखिलम्)
//!   Literal: "all from 9"
//!   In our context: when Ūrdhva cannot compress (ops have cross-dependencies),
//!   apply Nikhilam — reduce only the independent suffix of the chain.
//!
//! SutraSelector auto-picks the best sutra for a given op batch.

use crate::types::{OpId, ReversibleOp};

/// Available Vedic computation sutras for batch compression
#[derive(Debug, Clone, PartialEq)]
pub enum Sutra {
    /// Ūrdhva-Tiryagbhyām — primary sutra, compresses sequential+parallel chains
    UrdhvaTiryak,
    /// Nikhilam — fallback sutra, reduces independent suffix only
    Nikhilam,
    /// Paravartya — used when ops form a division/inverse chain
    Paravartya,
    /// Anurupyena — proportionality-based compression for repeated ops
    Anurupyena,
}

impl Sutra {
    pub fn name(&self) -> &'static str {
        match self {
            Sutra::UrdhvaTiryak => "UrdhvaTiryagbhyam",
            Sutra::Nikhilam => "Nikhilam",
            Sutra::Paravartya => "Paravartya",
            Sutra::Anurupyena => "Anurupyena",
        }
    }

    pub fn sanskrit_name(&self) -> &'static str {
        match self {
            Sutra::UrdhvaTiryak => "ऊर्ध्वतिर्यग्भ्याम्",
            Sutra::Nikhilam => "निखिलम्",
            Sutra::Paravartya => "परावर्त्य",
            Sutra::Anurupyena => "अनुरूप्येण",
        }
    }
}

/// Result of sutra selection: which sutra to use and why
#[derive(Debug, Clone)]
pub struct SutraSelection {
    pub sutra: Sutra,
    pub reason: String,
    pub estimated_reduction: f64,  // 0.0 to 1.0, fraction of ops that can be compressed
}

/// SutraSelector analyzes a batch of ops and picks the best sutra.
pub struct SutraSelector;

impl SutraSelector {
    /// Analyze a batch of ReversibleOps and select the optimal sutra.
    /// Returns SutraSelection with the chosen sutra and estimated history reduction.
    pub fn select(ops: &[ReversibleOp]) -> SutraSelection {
        if ops.is_empty() {
            return SutraSelection {
                sutra: Sutra::UrdhvaTiryak,
                reason: "empty batch — default to UrdhvaTiryak".to_string(),
                estimated_reduction: 0.0,
            };
        }

        // Check for repeated op pattern (same forward_fn_name repeated) → Anurupyena
        let repeated = Self::has_repeated_pattern(ops);
        if repeated {
            return SutraSelection {
                sutra: Sutra::Anurupyena,
                reason: "repeated operation pattern detected — Anurupyena compression".to_string(),
                estimated_reduction: 0.75,
            };
        }

        // Check for inverse chain (each op's inverse_fn_name matches next op's forward_fn_name) → Paravartya
        let inverse_chain = Self::has_inverse_chain(ops);
        if inverse_chain {
            return SutraSelection {
                sutra: Sutra::Paravartya,
                reason: "inverse chain detected — Paravartya reduction".to_string(),
                estimated_reduction: 0.70,
            };
        }

        // Check cross-dependency density
        let dep_density = Self::dependency_density(ops);

        if dep_density < 0.3 {
            // Low dependency — UrdhvaTiryak can compress well
            SutraSelection {
                sutra: Sutra::UrdhvaTiryak,
                reason: format!(
                    "low dependency density ({:.2}) — UrdhvaTiryak full compression",
                    dep_density
                ),
                estimated_reduction: 0.75,
            }
        } else if dep_density < 0.7 {
            // Medium dependency — UrdhvaTiryak partial
            SutraSelection {
                sutra: Sutra::UrdhvaTiryak,
                reason: format!(
                    "medium dependency density ({:.2}) — UrdhvaTiryak partial compression",
                    dep_density
                ),
                estimated_reduction: 0.50,
            }
        } else {
            // High dependency — fall back to Nikhilam
            SutraSelection {
                sutra: Sutra::Nikhilam,
                reason: format!(
                    "high dependency density ({:.2}) — Nikhilam suffix reduction",
                    dep_density
                ),
                estimated_reduction: 0.30,
            }
        }
    }

    /// Fraction of ops that have at least one dependency (0.0 = none, 1.0 = all)
    fn dependency_density(ops: &[ReversibleOp]) -> f64 {
        if ops.is_empty() {
            return 0.0;
        }
        let with_deps = ops.iter().filter(|op| !op.dependencies.is_empty()).count();
        with_deps as f64 / ops.len() as f64
    }

    /// True if more than 50% of ops share the same forward_fn_name
    fn has_repeated_pattern(ops: &[ReversibleOp]) -> bool {
        if ops.len() < 3 {
            return false;
        }
        let mut counts = std::collections::HashMap::new();
        for op in ops {
            *counts.entry(&op.forward_fn_name).or_insert(0usize) += 1;
        }
        let max_count = counts.values().copied().max().unwrap_or(0);
        max_count as f64 / ops.len() as f64 > 0.5
    }

    /// True if any op's inverse_fn_name matches the next op's forward_fn_name
    fn has_inverse_chain(ops: &[ReversibleOp]) -> bool {
        if ops.len() < 2 {
            return false;
        }
        ops.windows(2)
            .any(|w| w[0].inverse_fn_name == w[1].forward_fn_name)
    }

    /// Given a SutraSelection, compute which OpIds to retain vs compress.
    /// Returns (retain_ids, compress_ids).
    /// compress_ids are ops that can be folded into a summary — their ancilla data
    /// is still preserved but they are evicted from the active window.
    pub fn partition(
        ops: &[ReversibleOp],
        selection: &SutraSelection,
    ) -> (Vec<OpId>, Vec<OpId>) {
        let compress_count =
            (ops.len() as f64 * selection.estimated_reduction).floor() as usize;
        let retain_count = ops.len() - compress_count;

        // Always retain the most recent ops (tail of the chain)
        let retain_ids: Vec<OpId> = ops[compress_count..].iter().map(|op| op.id).collect();
        let compress_ids: Vec<OpId> = ops[..compress_count].iter().map(|op| op.id).collect();

        let _ = retain_count; // suppress warning
        (retain_ids, compress_ids)
    }
}
