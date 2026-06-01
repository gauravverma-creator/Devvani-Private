use devvani_ast::{Vibhakti, KarakaRole};

pub fn vibhakti_to_karaka(v: &Vibhakti) -> KarakaRole {
    match v {
        Vibhakti::Prathama => KarakaRole::Karta,
        Vibhakti::Dvitiya => KarakaRole::Karma,
        Vibhakti::Tritiya => KarakaRole::Karana,
        Vibhakti::Chaturthi => KarakaRole::Sampradana,
        Vibhakti::Panchami => KarakaRole::Apadana,
        Vibhakti::Shashthi => KarakaRole::Karta, // With possession flag in real system
        Vibhakti::Saptami => KarakaRole::Adhikarana,
    }
}

pub fn karaka_to_ast_role(k: &KarakaRole) -> &'static str {
    match k {
        KarakaRole::Karta => "Subject",
        KarakaRole::Karma => "Object",
        KarakaRole::Karana => "Instrument",
        KarakaRole::Sampradana => "ReturnTarget",
        KarakaRole::Apadana => "Source (Ablative)",
        KarakaRole::Apadan => "Source (Ablative)",
        KarakaRole::Adhikarana => "Scope",
    }
}
