use std::fmt;

/// Sanskrit Vibhakti (case) maps to compiler type roles
#[derive(Debug, Clone, PartialEq)]
pub enum VibhaktiRole {
    Prathama,   // Nominative   → Subject / Type Declaration
    Dvitiya,    // Accusative   → Function Parameter / Object
    Tritiya,    // Instrumental → Helper / Library
    Chaturthi,  // Dative       → Return Target / Receiver
    Panchami,   // Ablative     → Source / Origin
    Shashthi,   // Genitive     → Parent / Owner (struct field)
    Saptami,    // Locative     → Scope / Namespace / Module
}

#[derive(Debug, Clone, PartialEq)]
pub enum DevvaniType {
    Subject(String),       // Prathama
    Parameter(String),     // Dvitiya
    Instrument(String),    // Tritiya
    ReturnTarget(String),  // Chaturthi
    Source(String),        // Panchami
    Owner(String),         // Shashthi
    Scope(String),         // Saptami
    Unknown,
}

pub fn vibhakti_to_type(role: &VibhaktiRole, name: &str) -> DevvaniType {
    match role {
        VibhaktiRole::Prathama => DevvaniType::Subject(name.to_string()),
        VibhaktiRole::Dvitiya => DevvaniType::Parameter(name.to_string()),
        VibhaktiRole::Tritiya => DevvaniType::Instrument(name.to_string()),
        VibhaktiRole::Chaturthi => DevvaniType::ReturnTarget(name.to_string()),
        VibhaktiRole::Panchami => DevvaniType::Source(name.to_string()),
        VibhaktiRole::Shashthi => DevvaniType::Owner(name.to_string()),
        VibhaktiRole::Saptami => DevvaniType::Scope(name.to_string()),
    }
}

pub fn infer_type_from_suffix(word: &str) -> VibhaktiRole {
    let lower_word = word.to_lowercase();
    if lower_word.ends_with("ah") || lower_word.ends_with("ah") { // "aH" handled by lowercase
        VibhaktiRole::Prathama
    } else if lower_word.ends_with("am") {
        VibhaktiRole::Dvitiya
    } else if lower_word.ends_with("ena") {
        VibhaktiRole::Tritiya
    } else if lower_word.ends_with("aya") {
        VibhaktiRole::Chaturthi
    } else if lower_word.ends_with("at") {
        VibhaktiRole::Panchami
    } else if lower_word.ends_with("asya") {
        VibhaktiRole::Shashthi
    } else if lower_word.ends_with("e") {
        VibhaktiRole::Saptami
    } else {
        VibhaktiRole::Prathama
    }
}

impl fmt::Display for VibhaktiRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DevvaniType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DevvaniType::Subject(s) => write!(f, "Subject({})", s),
            DevvaniType::Parameter(s) => write!(f, "Parameter({})", s),
            DevvaniType::Instrument(s) => write!(f, "Instrument({})", s),
            DevvaniType::ReturnTarget(s) => write!(f, "ReturnTarget({})", s),
            DevvaniType::Source(s) => write!(f, "Source({})", s),
            DevvaniType::Owner(s) => write!(f, "Owner({})", s),
            DevvaniType::Scope(s) => write!(f, "Scope({})", s),
            DevvaniType::Unknown => write!(f, "Unknown"),
        }
    }
}
