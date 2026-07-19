use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TaddhitaPratyaya {
    Tva,
    Ta,
    Iya,
    Maya,
    Vat,
    In,
    Tara,
    Tama,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaddhitaDerivedType {
    AbstractNoun,
    AbstractState,
    Relational,
    Compositional,
    Possessive,
    Comparative,
    Superlative,
}

pub fn taddhita_from_suffix(suffix: &str) -> TaddhitaPratyaya {
    match suffix.to_lowercase().as_str() {
        "tva" => TaddhitaPratyaya::Tva,
        "ta" | "taa" => TaddhitaPratyaya::Ta,
        "iya" => TaddhitaPratyaya::Iya,
        "maya" => TaddhitaPratyaya::Maya,
        "vat" | "vaan" => TaddhitaPratyaya::Vat,
        "in" => TaddhitaPratyaya::In,
        "tara" => TaddhitaPratyaya::Tara,
        "tama" => TaddhitaPratyaya::Tama,
        _ => TaddhitaPratyaya::Unknown,
    }
}

pub fn taddhita_to_derived_type(t: &TaddhitaPratyaya) -> TaddhitaDerivedType {
    match t {
        TaddhitaPratyaya::Tva => TaddhitaDerivedType::AbstractNoun,
        TaddhitaPratyaya::Ta => TaddhitaDerivedType::AbstractState,
        TaddhitaPratyaya::Iya => TaddhitaDerivedType::Relational,
        TaddhitaPratyaya::Maya => TaddhitaDerivedType::Compositional,
        TaddhitaPratyaya::Vat | TaddhitaPratyaya::In => TaddhitaDerivedType::Possessive,
        TaddhitaPratyaya::Tara => TaddhitaDerivedType::Comparative,
        TaddhitaPratyaya::Tama => TaddhitaDerivedType::Superlative,
        TaddhitaPratyaya::Unknown => TaddhitaDerivedType::AbstractNoun,
    }
}

impl fmt::Display for TaddhitaPratyaya {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for TaddhitaDerivedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taddhita_from_suffix_tva() {
        assert_eq!(taddhita_from_suffix("tva"), TaddhitaPratyaya::Tva);
    }

    #[test]
    fn test_taddhita_from_suffix_iya() {
        assert_eq!(taddhita_from_suffix("iya"), TaddhitaPratyaya::Iya);
    }

    #[test]
    fn test_taddhita_from_suffix_tara() {
        assert_eq!(taddhita_from_suffix("tara"), TaddhitaPratyaya::Tara);
    }

    #[test]
    fn test_taddhita_derived_abstract_noun() {
        assert_eq!(
            taddhita_to_derived_type(&TaddhitaPratyaya::Tva),
            TaddhitaDerivedType::AbstractNoun
        );
    }

    #[test]
    fn test_taddhita_derived_possessive() {
        assert_eq!(
            taddhita_to_derived_type(&TaddhitaPratyaya::Vat),
            TaddhitaDerivedType::Possessive
        );
    }

    #[test]
    fn test_taddhita_derived_superlative() {
        assert_eq!(
            taddhita_to_derived_type(&TaddhitaPratyaya::Tama),
            TaddhitaDerivedType::Superlative
        );
    }
}
