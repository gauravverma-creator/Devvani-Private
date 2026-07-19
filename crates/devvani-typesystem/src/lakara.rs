use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Lakara {
    Lat,      // sync
    Lit,      // pure
    Lut,      // lazy
    Lrt,      // async
    Let,      // optional
    Lot,      // void/imperative
    Lan,      // mutable/stateful
    Vidhilin, // conditional/Result
    Asihlin,  // try/fallback
    Lun,      // transactional
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Sync,
    Pure,
    Lazy,
    Async,
    Optional,
    Void,
    Mutable,
    Conditional,
    Fallback,
    Transactional,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionScope {
    pub lakara: Lakara,
    pub kind: ScopeKind,
    pub is_async: bool,
    pub return_wrapper: ReturnWrapper,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnWrapper {
    None,     // Lot  → ()
    Direct,   // Lat, Lit, Lan, Lun → T
    Future,   // Lrt  → Future<T>
    Option,   // Let  → Option<T>
    Result,   // Vidhilin → Result<T,E>
    Lazy,     // Lut  → impl Fn() -> T
    Fallback, // Asihlin → T with fallback
}

pub fn lakara_from_str(s: &str) -> Option<Lakara> {
    match s.to_lowercase().as_str() {
        "lat" => Some(Lakara::Lat),
        "lit" => Some(Lakara::Lit),
        "lut" => Some(Lakara::Lut),
        "lrt" => Some(Lakara::Lrt),
        "let" => Some(Lakara::Let),
        "lot" => Some(Lakara::Lot),
        "lan" => Some(Lakara::Lan),
        "vidhilin" => Some(Lakara::Vidhilin),
        "asihlin" => Some(Lakara::Asihlin),
        "lun" => Some(Lakara::Lun),
        _ => None,
    }
}

pub fn lakara_to_scope(lakara: &Lakara) -> FunctionScope {
    match lakara {
        Lakara::Lat => FunctionScope {
            lakara: Lakara::Lat,
            kind: ScopeKind::Sync,
            is_async: false,
            return_wrapper: ReturnWrapper::Direct,
        },
        Lakara::Lit => FunctionScope {
            lakara: Lakara::Lit,
            kind: ScopeKind::Pure,
            is_async: false,
            return_wrapper: ReturnWrapper::Direct,
        },
        Lakara::Lut => FunctionScope {
            lakara: Lakara::Lut,
            kind: ScopeKind::Lazy,
            is_async: false,
            return_wrapper: ReturnWrapper::Lazy,
        },
        Lakara::Lrt => FunctionScope {
            lakara: Lakara::Lrt,
            kind: ScopeKind::Async,
            is_async: true,
            return_wrapper: ReturnWrapper::Future,
        },
        Lakara::Let => FunctionScope {
            lakara: Lakara::Let,
            kind: ScopeKind::Optional,
            is_async: false,
            return_wrapper: ReturnWrapper::Option,
        },
        Lakara::Lot => FunctionScope {
            lakara: Lakara::Lot,
            kind: ScopeKind::Void,
            is_async: false,
            return_wrapper: ReturnWrapper::None,
        },
        Lakara::Lan => FunctionScope {
            lakara: Lakara::Lan,
            kind: ScopeKind::Mutable,
            is_async: false,
            return_wrapper: ReturnWrapper::Direct,
        },
        Lakara::Vidhilin => FunctionScope {
            lakara: Lakara::Vidhilin,
            kind: ScopeKind::Conditional,
            is_async: false,
            return_wrapper: ReturnWrapper::Result,
        },
        Lakara::Asihlin => FunctionScope {
            lakara: Lakara::Asihlin,
            kind: ScopeKind::Fallback,
            is_async: false,
            return_wrapper: ReturnWrapper::Fallback,
        },
        Lakara::Lun => FunctionScope {
            lakara: Lakara::Lun,
            kind: ScopeKind::Transactional,
            is_async: false,
            return_wrapper: ReturnWrapper::Direct,
        },
    }
}

pub fn is_async(lakara: &Lakara) -> bool {
    matches!(lakara, Lakara::Lrt)
}

pub fn is_pure(lakara: &Lakara) -> bool {
    matches!(lakara, Lakara::Lit)
}

pub fn is_mutable(lakara: &Lakara) -> bool {
    matches!(lakara, Lakara::Lan | Lakara::Lun)
}

impl fmt::Display for Lakara {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for ScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for FunctionScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FunctionScope(lakara={}, kind={}, async={})",
            self.lakara, self.kind, self.is_async
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lakara_from_str() {
        assert_eq!(lakara_from_str("Lat"), Some(Lakara::Lat));
        assert_eq!(lakara_from_str("lat"), Some(Lakara::Lat));
        assert_eq!(lakara_from_str("invalid"), None);
    }

    #[test]
    fn test_async_lrt() {
        assert_eq!(lakara_from_str("Lrt"), Some(Lakara::Lrt));
        assert!(is_async(&Lakara::Lrt));
    }

    #[test]
    fn test_pure_lit() {
        let scope = lakara_to_scope(&Lakara::Lit);
        assert_eq!(scope.kind, ScopeKind::Pure);
        assert!(is_pure(&Lakara::Lit));
    }

    #[test]
    fn test_vidhilin_result() {
        let scope = lakara_to_scope(&Lakara::Vidhilin);
        assert_eq!(scope.return_wrapper, ReturnWrapper::Result);
    }

    #[test]
    fn test_round_trip_and_all_lakaras() {
        let lakaras = vec![
            Lakara::Lat,
            Lakara::Lit,
            Lakara::Lut,
            Lakara::Lrt,
            Lakara::Let,
            Lakara::Lot,
            Lakara::Lan,
            Lakara::Vidhilin,
            Lakara::Asihlin,
            Lakara::Lun,
        ];
        for l in lakaras {
            let s = format!("{}", l);
            assert_eq!(lakara_from_str(&s), Some(l.clone()));
        }
    }
}
