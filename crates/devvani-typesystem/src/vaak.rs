//! vaak.rs — Vaak (वाक्) String Type with Kāraka Ownership
//!
//! DESIGN AUTHORITY: Pāṇini's Aṣṭādhyāyī + Vaiśeṣika Dravya theory
//!
//! Vaiśeṣika: Śabda (sound/word) is a Guṇa (quality) of Ākāśa (space).
//! A Vaak (speech/string) is a Dravya (substance) that can be owned,
//! borrowed, or transferred — just like physical objects in Vaiśeṣika ontology.
//!
//! Kāraka Ownership Model for Vaak:
//!   Kartā (Prathamā)  → Owner: heap-allocated String, one owner at a time
//!   Karaṇa (Tṛtīyā)  → Immutable borrow: read-only &str reference
//!   Apādāna (Pañcamī) → Move: transfer ownership, original invalidated
//!
//! Mīmāṃsā Borrow Tiers (for future Sampradāna):
//!   Apūrva-vidhi   → New mutable borrow (first-time write access)
//!   Niyama-vidhi   → Restricted borrow (read-only)
//!   Pariṣaṅkhyā   → Exclusive borrow (only one mutable borrower)

use crate::vibhakti::DevvaniType;
use std::collections::HashMap;

/// VaakOwnership — tracks the ownership state of a Vaak string variable
#[derive(Debug, Clone, PartialEq)]
pub enum VaakOwnership {
    /// Kartā: this binding owns the string (heap-allocated)
    Karta,
    /// Karaṇa: this binding borrows the string immutably
    Karana,
    /// Apādāna: this binding has received ownership via move
    /// The original Kartā binding is now Moved (invalid)
    Apadana,
    /// Moved: this binding has transferred ownership away — now invalid
    Moved,
}

/// VaakSymbol — a string variable in the Devvani type system
#[derive(Debug, Clone)]
pub struct VaakSymbol {
    /// Variable name (IAST identifier)
    pub naama: String,
    /// Current ownership state
    pub ownership: VaakOwnership,
    /// Is mutable? (Pullinga=mutable, Strilinga=immutable, Napumsaka=const)
    pub is_mutable: bool,
    /// The DevvaniType for this symbol
    pub devvani_type: DevvaniType,
    /// LLVM/Rust type hint: "String" for Karta, "&str" for Karana
    pub rust_type_hint: String,
}

impl VaakSymbol {
    /// Create a new owned Vaak string (Kartā semantics)
    pub fn new_karta(naama: &str, is_mutable: bool) -> Self {
        Self {
            naama: naama.to_string(),
            ownership: VaakOwnership::Karta,
            is_mutable,
            devvani_type: DevvaniType::Vaak,
            rust_type_hint: "String".to_string(),
        }
    }

    /// Create an immutable borrow of a Vaak string (Karaṇa semantics)
    pub fn new_karana(naama: &str) -> Self {
        Self {
            naama: naama.to_string(),
            ownership: VaakOwnership::Karana,
            is_mutable: false,
            devvani_type: DevvaniType::VaakBorrow,
            rust_type_hint: "&str".to_string(),
        }
    }

    /// Move ownership from this symbol to another (Apādāna semantics).
    /// After calling this, self.ownership becomes Moved — it is invalid.
    /// Returns a new VaakSymbol with Apadana ownership (the receiver).
    pub fn move_to(&mut self, new_naama: &str) -> Result<VaakSymbol, VaakError> {
        match self.ownership {
            VaakOwnership::Moved => Err(VaakError::UseAfterMove {
                naama: self.naama.clone(),
            }),
            VaakOwnership::Karana => Err(VaakError::CannotMoveBorrow {
                naama: self.naama.clone(),
            }),
            VaakOwnership::Karta | VaakOwnership::Apadana => {
                self.ownership = VaakOwnership::Moved;
                Ok(VaakSymbol {
                    naama: new_naama.to_string(),
                    ownership: VaakOwnership::Apadana,
                    is_mutable: self.is_mutable,
                    devvani_type: DevvaniType::Vaak,
                    rust_type_hint: "String".to_string(),
                })
            }
        }
    }

    /// Check if this symbol can be read (not Moved)
    pub fn can_read(&self) -> Result<(), VaakError> {
        if self.ownership == VaakOwnership::Moved {
            Err(VaakError::UseAfterMove {
                naama: self.naama.clone(),
            })
        } else {
            Ok(())
        }
    }

    /// Check if this symbol can be written (must be Karta + mutable)
    pub fn can_write(&self) -> Result<(), VaakError> {
        match self.ownership {
            VaakOwnership::Moved => Err(VaakError::UseAfterMove {
                naama: self.naama.clone(),
            }),
            VaakOwnership::Karana => Err(VaakError::ImmutableBorrow {
                naama: self.naama.clone(),
            }),
            _ if !self.is_mutable => Err(VaakError::NotMutable {
                naama: self.naama.clone(),
            }),
            _ => Ok(()),
        }
    }

    /// Rust binding string for codegen reference
    pub fn to_rust_binding(&self) -> String {
        let kw = if self.is_mutable { "let mut" } else { "let" };
        format!("{} {}: {}", kw, self.naama, self.rust_type_hint)
    }
}

/// VaakError — ownership violation errors for Vaak strings
/// Named after Sanskrit grammar error tradition (Doṣa = defect)
#[derive(Debug, Clone, PartialEq)]
pub enum VaakError {
    /// Used a string variable after its ownership was moved (Apādāna completed)
    UseAfterMove { naama: String },
    /// Tried to move an immutable borrow (Karaṇa cannot be moved)
    CannotMoveBorrow { naama: String },
    /// Tried to write to an immutable binding (Strilinga/Napumsaka)
    ImmutableBorrow { naama: String },
    /// Tried to write to a non-mutable variable
    NotMutable { naama: String },
}

impl std::fmt::Display for VaakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaakError::UseAfterMove { naama } => write!(
                f,
                "Doṣa D030: '{}' — svāmitva-hāni (ownership moved, cannot use)",
                naama
            ),
            VaakError::CannotMoveBorrow { naama } => write!(
                f,
                "Doṣa D031: '{}' — karaṇa-apādāna-doṣa (cannot move an immutable borrow)",
                naama
            ),
            VaakError::ImmutableBorrow { naama } => write!(
                f,
                "Doṣa D032: '{}' — karaṇa-lekha-doṣa (cannot write to immutable borrow)",
                naama
            ),
            VaakError::NotMutable { naama } => write!(
                f,
                "Doṣa D033: '{}' — sthira-lekha-doṣa (variable is not mutable)",
                naama
            ),
        }
    }
}

impl std::error::Error for VaakError {}

pub struct MoveChecker {
    pub ownership_map: HashMap<String, VaakOwnership>,
}

impl MoveChecker {
    pub fn new() -> Self {
        Self {
            ownership_map: HashMap::new(),
        }
    }

    pub fn check_use(&self, naama: &str) -> Result<(), VaakError> {
        if let Some(ownership) = self.ownership_map.get(naama) {
            if *ownership == VaakOwnership::Moved {
                return Err(VaakError::UseAfterMove {
                    naama: naama.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn do_move(&mut self, naama: &str) -> Result<(), VaakError> {
        match self.ownership_map.get(naama) {
            Some(VaakOwnership::Moved) => Err(VaakError::UseAfterMove {
                naama: naama.to_string(),
            }),
            Some(VaakOwnership::Karana) => Err(VaakError::CannotMoveBorrow {
                naama: naama.to_string(),
            }),
            Some(VaakOwnership::Karta | VaakOwnership::Apadana) => {
                self.ownership_map
                    .insert(naama.to_string(), VaakOwnership::Moved);
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn register(&mut self, naama: String, ownership: VaakOwnership) {
        self.ownership_map.insert(naama, ownership);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karta_creation() {
        let sym = VaakSymbol::new_karta("vāk", true);
        assert_eq!(sym.ownership, VaakOwnership::Karta);
        assert_eq!(sym.rust_type_hint, "String");
        assert!(sym.can_read().is_ok());
        assert!(sym.can_write().is_ok());
    }

    #[test]
    fn test_karana_cannot_write() {
        let sym = VaakSymbol::new_karana("vāk_ref");
        assert!(sym.can_read().is_ok());
        assert!(sym.can_write().is_err());
    }

    #[test]
    fn test_move_invalidates_original() {
        let mut owner = VaakSymbol::new_karta("mūla", true);
        let _new_owner = owner.move_to("navam").unwrap();
        assert_eq!(owner.ownership, VaakOwnership::Moved);
        assert!(owner.can_read().is_err());
    }

    #[test]
    fn test_use_after_move_error() {
        let mut owner = VaakSymbol::new_karta("mūla", true);
        let _ = owner.move_to("navam").unwrap();
        let err = owner.can_read().unwrap_err();
        assert!(matches!(err, VaakError::UseAfterMove { .. }));
    }

    #[test]
    fn test_cannot_move_borrow() {
        let mut borrow = VaakSymbol::new_karana("rin");
        let err = borrow.move_to("navam").unwrap_err();
        assert!(matches!(err, VaakError::CannotMoveBorrow { .. }));
    }

    #[test]
    fn test_immutable_karta_cannot_write() {
        let sym = VaakSymbol::new_karta("sthira", false);
        assert!(sym.can_write().is_err());
        assert!(matches!(
            sym.can_write().unwrap_err(),
            VaakError::NotMutable { .. }
        ));
    }

    #[test]
    fn test_karana_type() {
        let sym = VaakSymbol::new_karana("rin");
        assert_eq!(sym.devvani_type, DevvaniType::VaakBorrow);
        assert_eq!(sym.rust_type_hint, "&str");
    }

    #[test]
    fn test_karta_register() {
        let mut checker = MoveChecker::new();
        checker.register("vāk".to_string(), VaakOwnership::Karta);
        assert!(checker.check_use("vāk").is_ok());
    }

    #[test]
    fn test_move_transfers() {
        let mut checker = MoveChecker::new();
        checker.register("mūla".to_string(), VaakOwnership::Karta);
        assert!(checker.do_move("mūla").is_ok());
        assert!(checker.check_use("mūla").is_err());
    }

    #[test]
    fn test_double_move_fails() {
        let mut checker = MoveChecker::new();
        checker.register("mūla".to_string(), VaakOwnership::Karta);
        let _ = checker.do_move("mūla").unwrap();
        assert!(checker.do_move("mūla").is_err());
    }

    #[test]
    fn test_karana_no_move() {
        let mut checker = MoveChecker::new();
        checker.register("rin".to_string(), VaakOwnership::Karana);
        assert!(checker.check_use("rin").is_ok());
        assert!(checker.do_move("rin").is_err());
    }
}
