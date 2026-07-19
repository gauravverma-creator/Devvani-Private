use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Linga {
    Pullinga,  // masculine → immutable owned
    Strilinga, // feminine  → mutable
    Napumsaka, // neuter    → shared ref
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutabilityInfo {
    pub linga: Linga,
    pub is_mutable: bool,
    pub is_shared: bool,
    pub ownership: OwnershipKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipKind {
    Owned,
    MutableOwned,
    SharedRef,
}

pub fn linga_from_str(s: &str) -> Option<Linga> {
    match s.to_lowercase().as_str() {
        "pullinga" => Some(Linga::Pullinga),
        "strilinga" => Some(Linga::Strilinga),
        "napumsaka" | "napumsakalinga" => Some(Linga::Napumsaka),
        _ => None,
    }
}

pub fn linga_to_mutability(l: &Linga) -> MutabilityInfo {
    match l {
        Linga::Pullinga => MutabilityInfo {
            linga: Linga::Pullinga,
            is_mutable: false,
            is_shared: false,
            ownership: OwnershipKind::Owned,
        },
        Linga::Strilinga => MutabilityInfo {
            linga: Linga::Strilinga,
            is_mutable: true,
            is_shared: false,
            ownership: OwnershipKind::MutableOwned,
        },
        Linga::Napumsaka => MutabilityInfo {
            linga: Linga::Napumsaka,
            is_mutable: false,
            is_shared: true,
            ownership: OwnershipKind::SharedRef,
        },
    }
}

pub fn linga_to_rust_keyword(l: &Linga) -> &'static str {
    match l {
        Linga::Pullinga => "let",
        Linga::Strilinga => "let mut",
        Linga::Napumsaka => "let",
    }
}

impl fmt::Display for Linga {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for MutabilityInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MutabilityInfo(linga={}, mutable={}, shared={})",
            self.linga, self.is_mutable, self.is_shared
        )
    }
}

impl fmt::Display for OwnershipKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linga_from_str() {
        assert_eq!(linga_from_str("Strilinga"), Some(Linga::Strilinga));
        assert_eq!(linga_from_str("napumsakalinga"), Some(Linga::Napumsaka));
    }

    #[test]
    fn test_linga_to_mutability_stri() {
        let info = linga_to_mutability(&Linga::Strilinga);
        assert!(info.is_mutable);
        assert_eq!(info.ownership, OwnershipKind::MutableOwned);
    }

    #[test]
    fn test_linga_to_rust_keyword_pullinga() {
        assert_eq!(linga_to_rust_keyword(&Linga::Pullinga), "let");
    }

    #[test]
    fn test_linga_to_mutability_napumsaka() {
        let info = linga_to_mutability(&Linga::Napumsaka);
        assert!(info.is_shared);
        assert_eq!(info.ownership, OwnershipKind::SharedRef);
    }
}
