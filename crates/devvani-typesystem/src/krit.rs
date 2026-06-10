use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum KritPratyaya {
    Kta,    // past passive participle - "done" e.g. krita
    Tavya,  // obligation/must - "to be done" e.g. kartavya
    Ana,    // present participle/agent - "doing" e.g. karana
    Tum,    // infinitive - "to do" e.g. kartum
    Ktva,   // absolutive - "having done" e.g. kritva
    Trich,  // agent noun - "one who does" e.g. karta
    Anta,   // present active participle e.g. kuranta
    Nya,    // obligation variant e.g. karanya
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KritType {
    pub pratyaya: KritPratyaya,
    pub base_dhatu: String,
    pub derived_type: KritDerivedType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KritDerivedType {
    PastPassive,      // Kta  - bool/state
    Obligation,       // Tavya, Nya - must-do marker
    PresentAgent,     // Ana, Anta - currently doing
    Infinitive,       // Tum - function reference
    Absolutive,       // Ktva - completed action
    AgentNoun,        // Trich - actor type
}

pub fn krit_from_suffix(suffix: &str) -> KritPratyaya {
    match suffix.to_lowercase().as_str() {
        "kta" | "ita" => KritPratyaya::Kta,
        "tavya" | "taniya" => KritPratyaya::Tavya,
        "ana" | "aniya" => KritPratyaya::Ana,
        "tum" => KritPratyaya::Tum,
        "ktva" | "tva" => KritPratyaya::Ktva,
        "trich" | "tr" => KritPratyaya::Trich,
        "anta" | "at" => KritPratyaya::Anta,
        "nya" | "ya" => KritPratyaya::Nya,
        _ => KritPratyaya::Unknown,
    }
}

pub fn krit_to_derived_type(k: &KritPratyaya) -> KritDerivedType {
    match k {
        KritPratyaya::Kta => KritDerivedType::PastPassive,
        KritPratyaya::Tavya | KritPratyaya::Nya => KritDerivedType::Obligation,
        KritPratyaya::Ana | KritPratyaya::Anta => KritDerivedType::PresentAgent,
        KritPratyaya::Tum => KritDerivedType::Infinitive,
        KritPratyaya::Ktva => KritDerivedType::Absolutive,
        KritPratyaya::Trich => KritDerivedType::AgentNoun,
        KritPratyaya::Unknown => KritDerivedType::PastPassive,
    }
}

impl fmt::Display for KritPratyaya {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for KritDerivedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_krit_from_suffix_kta() {
        assert_eq!(krit_from_suffix("kta"), KritPratyaya::Kta);
    }

    #[test]
    fn test_krit_from_suffix_tavya() {
        assert_eq!(krit_from_suffix("tavya"), KritPratyaya::Tavya);
    }

    #[test]
    fn test_krit_from_suffix_tum() {
        assert_eq!(krit_from_suffix("tum"), KritPratyaya::Tum);
    }

    #[test]
    fn test_krit_to_derived_infinitive() {
        assert_eq!(krit_to_derived_type(&KritPratyaya::Tum), KritDerivedType::Infinitive);
    }

    #[test]
    fn test_krit_to_derived_obligation() {
        assert_eq!(krit_to_derived_type(&KritPratyaya::Tavya), KritDerivedType::Obligation);
    }
}
