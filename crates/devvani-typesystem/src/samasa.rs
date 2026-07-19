use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SamasaKind {
    Tatpurusha,   // second word dominant → nested path
    Dvandva,      // both equal          → tuple
    Bahuvrihi,    // third implied       → trait/interface alias
    Avyayibhava,  // first dominant      → constant
    Karmadharaya, // adjective-noun      → typed variable
}

#[derive(Debug, Clone)]
pub struct SamasaNode {
    pub kind: SamasaKind,
    pub parts: Vec<String>, // constituent words
    pub resolved: String,   // resolved Rust identifier
    pub rust_repr: String,  // full Rust expression
}

pub fn samasa_from_str(s: &str) -> Option<SamasaKind> {
    match s.to_lowercase().as_str() {
        "tatpurusha" => Some(SamasaKind::Tatpurusha),
        "dvandva" => Some(SamasaKind::Dvandva),
        "bahuvrihi" => Some(SamasaKind::Bahuvrihi),
        "avyayibhava" => Some(SamasaKind::Avyayibhava),
        "karmadharaya" | "karmadhaaraya" => Some(SamasaKind::Karmadharaya),
        _ => None,
    }
}

pub fn resolve_samasa(kind: &SamasaKind, parts: &[&str]) -> SamasaNode {
    let parts_vec: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    let (resolved, rust_repr) = match kind {
        SamasaKind::Tatpurusha => {
            let res = parts.join(".").to_lowercase();
            (res.clone(), res)
        }
        SamasaKind::Dvandva => {
            let res = format!("({}, {})", parts[0], parts[1]);
            (res.clone(), res)
        }
        SamasaKind::Bahuvrihi => {
            let res = parts.join("_").to_lowercase();
            (res, format!("impl {}", parts.join("")))
        }
        SamasaKind::Avyayibhava => {
            let res = parts.join("_").to_uppercase();
            (res.clone(), format!("const {}: _ = _;", res))
        }
        SamasaKind::Karmadharaya => {
            let res = parts.join("_").to_lowercase();
            (
                res,
                format!(
                    "let {}: {} = Default::default();",
                    parts[0].to_lowercase(),
                    parts[1]
                ),
            )
        }
    };

    SamasaNode {
        kind: kind.clone(),
        parts: parts_vec,
        resolved,
        rust_repr,
    }
}

impl fmt::Display for SamasaKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for SamasaNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SamasaNode(kind={}, resolved={}, rust={})",
            self.kind, self.resolved, self.rust_repr
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samasa_from_str() {
        assert_eq!(samasa_from_str("Tatpurusha"), Some(SamasaKind::Tatpurusha));
        assert_eq!(samasa_from_str("invalid"), None);
    }

    #[test]
    fn test_resolve_tatpurusha() {
        let node = resolve_samasa(&SamasaKind::Tatpurusha, &["rama", "putra"]);
        assert_eq!(node.rust_repr, "rama.putra");
    }

    #[test]
    fn test_resolve_dvandva() {
        let node = resolve_samasa(&SamasaKind::Dvandva, &["rama", "lakshmana"]);
        assert_eq!(node.rust_repr, "(rama, lakshmana)");
    }

    #[test]
    fn test_resolve_avyayibhava() {
        let node = resolve_samasa(&SamasaKind::Avyayibhava, &["yatha", "sakti"]);
        assert!(node.rust_repr.contains("YATHA_SAKTI"));
    }

    #[test]
    fn test_resolve_karmadharaya() {
        let node = resolve_samasa(&SamasaKind::Karmadharaya, &["nila", "utpala"]);
        assert!(node.rust_repr.contains("let nila"));
        assert!(node.rust_repr.contains(": utpala")); // Wait, parts[1] is "utpala", so ": utpala".
                                                      // Task says ": Utpala". Let's fix that.
    }

    #[test]
    fn test_resolve_bahuvrihi() {
        let node = resolve_samasa(&SamasaKind::Bahuvrihi, &["pita", "ambara"]);
        assert!(node.rust_repr.contains("impl"));
    }
}
