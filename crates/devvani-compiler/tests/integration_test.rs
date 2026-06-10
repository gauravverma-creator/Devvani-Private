use devvani_compiler::Compiler;
use std::fs;

#[test]
fn test_pipeline_hello() {
    let _ = fs::write("examples/hello_integration.dvn", "phalam asti 5 । phalam vadati ।");
    let result = Compiler::new("examples/hello_integration.dvn").compile();
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_pipeline_samasa() {
    let _ = fs::write("examples/samasa_integration.dvn", "x asti 10 ।");
    let result = Compiler::new("examples/samasa_integration.dvn").compile();
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_rust_output_not_empty() {
    let _ = fs::write("examples/hello_integration.dvn", "phalam asti 5 ।");
    let result = Compiler::new("examples/hello_integration.dvn").compile();
    if let Ok(output) = result {
        assert!(!output.trim().is_empty());
    }
}

#[test]
fn test_diagnostics_clean_report() {
    use devvani_compiler::diagnostics::DiagnosticEngine;
    let report = DiagnosticEngine::report(&[]);
    assert!(report.contains("Shuddham"));
}

// ===== PHASE 3 TYPE SYSTEM TESTS =====

#[test]
fn test_krit_kta_past_passive() {
    use devvani_typesystem::krit::{krit_from_suffix, krit_to_derived_type, KritPratyaya, KritDerivedType};
    let pratyaya = krit_from_suffix("kta");
    assert_eq!(pratyaya, KritPratyaya::Kta);
    assert_eq!(krit_to_derived_type(&pratyaya), KritDerivedType::PastPassive);
}

#[test]
fn test_krit_tum_infinitive() {
    use devvani_typesystem::krit::{krit_from_suffix, krit_to_derived_type, KritPratyaya, KritDerivedType};
    let pratyaya = krit_from_suffix("tum");
    assert_eq!(pratyaya, KritPratyaya::Tum);
    assert_eq!(krit_to_derived_type(&pratyaya), KritDerivedType::Infinitive);
}

#[test]
fn test_krit_tavya_obligation() {
    use devvani_typesystem::krit::{krit_from_suffix, krit_to_derived_type, KritPratyaya, KritDerivedType};
    let pratyaya = krit_from_suffix("tavya");
    assert_eq!(pratyaya, KritPratyaya::Tavya);
    assert_eq!(krit_to_derived_type(&pratyaya), KritDerivedType::Obligation);
}

#[test]
fn test_taddhita_tva_abstract_noun() {
    use devvani_typesystem::taddhita::{taddhita_from_suffix, taddhita_to_derived_type, TaddhitaPratyaya, TaddhitaDerivedType};
    let pratyaya = taddhita_from_suffix("tva");
    assert_eq!(pratyaya, TaddhitaPratyaya::Tva);
    assert_eq!(taddhita_to_derived_type(&pratyaya), TaddhitaDerivedType::AbstractNoun);
}

#[test]
fn test_taddhita_tama_superlative() {
    use devvani_typesystem::taddhita::{taddhita_from_suffix, taddhita_to_derived_type, TaddhitaPratyaya, TaddhitaDerivedType};
    let pratyaya = taddhita_from_suffix("tama");
    assert_eq!(pratyaya, TaddhitaPratyaya::Tama);
    assert_eq!(taddhita_to_derived_type(&pratyaya), TaddhitaDerivedType::Superlative);
}

#[test]
fn test_taddhita_iya_relational() {
    use devvani_typesystem::taddhita::{taddhita_from_suffix, taddhita_to_derived_type, TaddhitaPratyaya, TaddhitaDerivedType};
    let pratyaya = taddhita_from_suffix("iya");
    assert_eq!(pratyaya, TaddhitaPratyaya::Iya);
    assert_eq!(taddhita_to_derived_type(&pratyaya), TaddhitaDerivedType::Relational);
}

#[test]
fn test_upasarga_export_private_conflict() {
    use devvani_typesystem::upasarga::{UpasargaChecker, UpasargaError};
    use devvani_ast::node::{ASTNode, UpasargaDirective, UpasargaNode};
    use devvani_ast::node::Span;
    let node = UpasargaNode {
        directives: vec![UpasargaDirective::Export, UpasargaDirective::Private],
        target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
        span: Span { line: 0, col: 0, len: 0 },
    };
    let result = UpasargaChecker.check(&node);
    assert!(
        matches!(result, Err(UpasargaError::UpasargaSangharsha { .. })),
        "Expected UpasargaSangharsha error for Export+Private conflict"
    );
}

#[test]
fn test_vibhakti_dvitiya_parameter_type() {
    use devvani_typesystem::vibhakti::{VibhaktiRole, vibhakti_to_type, DevvaniType};
    let result = vibhakti_to_type(&VibhaktiRole::Dvitiya, "ganana");
    assert_eq!(result, DevvaniType::Parameter("ganana".to_string()));
}
