use crate::{linga::*, vacana::*, vibhakti::DevvaniType};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub devvani_type: DevvaniType,
    pub cardinality: CardinalityKind,
    pub mutability: MutabilityInfo,
    pub rust_type_hint: String, // generated Rust type string
}

impl Symbol {
    pub fn new(
        name: &str,
        devvani_type: DevvaniType,
        vacana: &Vacana,
        linga: &Linga,
        inner_type: &str,
    ) -> Self {
        let cardinality = vacana_to_cardinality(vacana);
        let mutability = linga_to_mutability(linga);
        let rust_type_hint = vacana_to_rust_type(vacana, inner_type);

        // Handle shared reference in rust_type_hint if Napumsaka
        let final_rust_type = if mutability.is_shared {
            format!("&{}", rust_type_hint)
        } else {
            rust_type_hint
        };

        Self {
            name: name.to_string(),
            devvani_type,
            cardinality,
            mutability,
            rust_type_hint: final_rust_type,
        }
    }

    pub fn to_rust_binding(&self) -> String {
        let kw = linga_to_rust_keyword(&self.mutability.linga);
        format!("{} {}: {}", kw, self.name, self.rust_type_hint)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Symbol(name={}, type={}, rust={})",
            self.name, self.devvani_type, self.rust_type_hint
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_eka_pullinga() {
        let sym = Symbol::new(
            "ramah",
            DevvaniType::Subject("Ramah".to_string()),
            &Vacana::Eka,
            &Linga::Pullinga,
            "i64",
        );
        assert_eq!(sym.to_rust_binding(), "let ramah: i64");
    }

    #[test]
    fn test_symbol_bahu_strilinga() {
        let sym = Symbol::new(
            "sita",
            DevvaniType::Subject("Sita".to_string()),
            &Vacana::Bahu,
            &Linga::Strilinga,
            "String",
        );
        assert_eq!(sym.to_rust_binding(), "let mut sita: Vec<String>");
    }
}
