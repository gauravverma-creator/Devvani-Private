use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Vacana {
    Eka,  // singular
    Dvi,  // dual
    Bahu, // plural
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardinalityKind {
    Single,
    Pair,
    Collection,
}

pub fn vacana_from_str(s: &str) -> Option<Vacana> {
    match s.to_lowercase().as_str() {
        "eka" => Some(Vacana::Eka),
        "dvi" => Some(Vacana::Dvi),
        "bahu" => Some(Vacana::Bahu),
        _ => None,
    }
}

pub fn vacana_to_cardinality(v: &Vacana) -> CardinalityKind {
    match v {
        Vacana::Eka => CardinalityKind::Single,
        Vacana::Dvi => CardinalityKind::Pair,
        Vacana::Bahu => CardinalityKind::Collection,
    }
}

pub fn vacana_to_rust_type(v: &Vacana, inner: &str) -> String {
    match v {
        Vacana::Eka => inner.to_string(),
        Vacana::Dvi => format!("({}, {})", inner, inner),
        Vacana::Bahu => format!("Vec<{}>", inner),
    }
}

impl fmt::Display for Vacana {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for CardinalityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vacana_from_str() {
        assert_eq!(vacana_from_str("Bahu"), Some(Vacana::Bahu));
        assert_eq!(vacana_from_str("eka"), Some(Vacana::Eka));
    }

    #[test]
    fn test_vacana_to_rust_type_dvi() {
        assert_eq!(vacana_to_rust_type(&Vacana::Dvi, "i64"), "(i64, i64)");
    }

    #[test]
    fn test_vacana_to_rust_type_bahu() {
        assert_eq!(vacana_to_rust_type(&Vacana::Bahu, "String"), "Vec<String>");
    }
}
