use devvani_compiler::Compiler;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_pipeline_hello() {
    let _ = fs::write(
        "examples/hello_integration.dvn",
        "phalamulya asti 5 । phalamulya vadati ।",
    );
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
    let _ = fs::write("examples/hello_integration.dvn", "phalamulya asti 5 ।");
    let result = Compiler::new("examples/hello_integration.dvn").compile();
    if let Ok(output) = result {
        assert!(!output.trim().is_empty());
    }
}

#[test]
fn test_recursive_dhatu_codegen() {
    let source = "avartanah-dhatu n karoti । n yoga avartanah-dhatu n iti ।";
    let _ = fs::write("examples/recursive_integration.dvn", source);
    let result = Compiler::new("examples/recursive_integration.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    let call_count = code.matches("avartanah_dhatu(").count();
    assert!(
        call_count >= 2,
        "expected avartanah_dhatu( to appear >= 2 times, got {} in:\n{}",
        call_count,
        code
    );
}

#[test]
fn test_nonrecursive_dhatu_call_codegen() {
    let source = "prathamah-dhatu x karoti । x yoga 1 iti । dvitiyah-dhatu y karoti । y yoga prathamah-dhatu x iti ।";
    let _ = fs::write("examples/nonrecursive_integration.dvn", source);
    let result = Compiler::new("examples/nonrecursive_integration.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("prathamah_dhatu("),
        "expected prathamah_dhatu( in generated code:\n{}",
        code
    );
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
    use devvani_typesystem::krit::{
        krit_from_suffix, krit_to_derived_type, KritDerivedType, KritPratyaya,
    };
    let pratyaya = krit_from_suffix("kta");
    assert_eq!(pratyaya, KritPratyaya::Kta);
    assert_eq!(
        krit_to_derived_type(&pratyaya),
        KritDerivedType::PastPassive
    );
}

#[test]
fn test_krit_tum_infinitive() {
    use devvani_typesystem::krit::{
        krit_from_suffix, krit_to_derived_type, KritDerivedType, KritPratyaya,
    };
    let pratyaya = krit_from_suffix("tum");
    assert_eq!(pratyaya, KritPratyaya::Tum);
    assert_eq!(krit_to_derived_type(&pratyaya), KritDerivedType::Infinitive);
}

#[test]
fn test_krit_tavya_obligation() {
    use devvani_typesystem::krit::{
        krit_from_suffix, krit_to_derived_type, KritDerivedType, KritPratyaya,
    };
    let pratyaya = krit_from_suffix("tavya");
    assert_eq!(pratyaya, KritPratyaya::Tavya);
    assert_eq!(krit_to_derived_type(&pratyaya), KritDerivedType::Obligation);
}

#[test]
fn test_taddhita_tva_abstract_noun() {
    use devvani_typesystem::taddhita::{
        taddhita_from_suffix, taddhita_to_derived_type, TaddhitaDerivedType, TaddhitaPratyaya,
    };
    let pratyaya = taddhita_from_suffix("tva");
    assert_eq!(pratyaya, TaddhitaPratyaya::Tva);
    assert_eq!(
        taddhita_to_derived_type(&pratyaya),
        TaddhitaDerivedType::AbstractNoun
    );
}

#[test]
fn test_taddhita_tama_superlative() {
    use devvani_typesystem::taddhita::{
        taddhita_from_suffix, taddhita_to_derived_type, TaddhitaDerivedType, TaddhitaPratyaya,
    };
    let pratyaya = taddhita_from_suffix("tama");
    assert_eq!(pratyaya, TaddhitaPratyaya::Tama);
    assert_eq!(
        taddhita_to_derived_type(&pratyaya),
        TaddhitaDerivedType::Superlative
    );
}

#[test]
fn test_taddhita_iya_relational() {
    use devvani_typesystem::taddhita::{
        taddhita_from_suffix, taddhita_to_derived_type, TaddhitaDerivedType, TaddhitaPratyaya,
    };
    let pratyaya = taddhita_from_suffix("iya");
    assert_eq!(pratyaya, TaddhitaPratyaya::Iya);
    assert_eq!(
        taddhita_to_derived_type(&pratyaya),
        TaddhitaDerivedType::Relational
    );
}

#[test]
fn test_upasarga_export_private_conflict() {
    use devvani_ast::node::Span;
    use devvani_ast::node::{ASTNode, UpasargaDirective, UpasargaNode};
    use devvani_typesystem::upasarga::{UpasargaChecker, UpasargaError};
    let node = UpasargaNode {
        directives: vec![UpasargaDirective::Export, UpasargaDirective::Private],
        target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
        span: Span {
            line: 0,
            col: 0,
            len: 0,
        },
    };
    let result = UpasargaChecker.check(&node);
    assert!(
        matches!(result, Err(UpasargaError::UpasargaSangharsha { .. })),
        "Expected UpasargaSangharsha error for Export+Private conflict"
    );
}

#[test]
fn test_vibhakti_dvitiya_parameter_type() {
    use devvani_typesystem::vibhakti::{vibhakti_to_type, DevvaniType, VibhaktiRole};
    let result = vibhakti_to_type(&VibhaktiRole::Dvitiya, "ganana");
    assert_eq!(result, DevvaniType::Parameter("ganana".to_string()));
}

#[test]
fn test_nirmana_codegen_pipeline() {
    let source = "manushya dravya sankhya1 sankhya sankhya2 sankhya । manushya nirmāṇa 25 180 ।";
    let _ = fs::write("examples/nirmana_integration.dvn", source);
    let result = Compiler::new("examples/nirmana_integration.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("manushya { sankhya1: 25, sankhya2: 180 }"),
        "expected nirmana struct literal in generated code:\n{}",
        code
    );
}

#[test]
fn test_pipeline_type_inference() {
    let source = "dhara x = 5 । dhātu getnum karoti । x yoga 5 iti ।";
    let _ = fs::write("examples/inference_integration.dvn", source);
    let result = Compiler::new("examples/inference_integration.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    println!("GENERATED RUST:\n{}", code);
    assert!(
        code.contains("let x: i64 = 5;"),
        "expected inferred integer variable to have explicit i64 type, got:\n{}",
        code
    );
    assert!(
        code.contains("pub fn getnum() -> i64 {"),
        "expected inferred return type function to have explicit i64 return type, got:\n{}",
        code
    );
}

// ===== Pariṇāma (Pipeline) End-to-End Tests =====

#[test]
fn test_parinama_e2e_nonfallible() {
    let source =
        "inc-dhatu n karoti । n yoga 1 iti ।\n\
         double-dhatu n karoti । n yoga 2 iti ।\n\
         \n\
         dhara result = 5 pariṇāma [inc-dhatu, double-dhatu] ।\n";
    let _ = fs::write("examples/parinama_nonfallible.dvn", source);
    let result = Compiler::new("examples/parinama_nonfallible.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("double_dhatu(inc_dhatu(5))"),
        "expected nested parinama calls in generated code:\n{}",
        code
    );
}

#[test]
fn test_parinama_e2e_fallible() {
    let source =
        "fail-dhatu n phalam sankhya vaak karoti । arogya 5 । iti ।\n\
         \n\
         dhara result = 5 pariṇāma [fail-dhatu] ।\n";
    let _ = fs::write("examples/parinama_fallible.dvn", source);
    let result = Compiler::new("examples/parinama_fallible.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("fail_dhatu(5)"),
        "expected single fallible parinama call in generated code:\n{}",
        code
    );
}

#[test]
fn test_parinama_e2e_mixed_fallible() {
    let source =
        "fail-dhatu n phalam sankhya vaak karoti । arogya 5 । iti ।\n\
         double-dhatu n karoti । n yoga 2 iti ।\n\
         \n\
         dhara result = 5 pariṇāma [fail-dhatu, double-dhatu] ।\n";
    let _ = fs::write("examples/parinama_mixed.dvn", source);
    let result = Compiler::new("examples/parinama_mixed.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("fail_dhatu(5).and_then(|v0| Ok(double_dhatu(v0)))"),
        "expected mixed fallible parinama chain in generated code:\n{}",
        code
    );
}

fn assert_compiles(name: &str, generated_code: &str) {
    let tmp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
    let tmp_path = tmp_dir.path().join("parinama_verify.rs");
    let out_path = tmp_dir.path().join("parinama_verify_out");

    let wrapped = format!("fn main() {{\n{}\n}}", generated_code);
    fs::write(&tmp_path, wrapped)
        .expect("failed to write temp file");

    let status = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--crate-type")
        .arg("bin")
        .arg("--crate-name")
        .arg("parinama_verify")
        .arg(&tmp_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("failed to run rustc");

    let stderr = String::from_utf8_lossy(&status.stderr);
    assert!(
        status.status.success(),
        "rustc failed for {}:\n{}",
        name,
        stderr
    );
}

#[test]
fn test_parinama_e2e_nonfallible_compiles() {
    let source =
        "inc-dhatu n karoti । n yoga 1 iti ।\n\
         double-dhatu n karoti । n yoga 2 iti ।\n\
         \n\
         dhara result = 5 pariṇāma [inc-dhatu, double-dhatu] ।\n";
    let _ = fs::write("examples/parinama_nonfallible.dvn", source);
    let result = Compiler::new("examples/parinama_nonfallible.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("double_dhatu(inc_dhatu(5))"),
        "expected nested parinama calls in generated code:\n{}",
        code
    );
    assert_compiles("nonfallible parinama", &code);
}

#[test]
fn test_parinama_e2e_fallible_compiles() {
    let source =
        "fail-dhatu n phalam sankhya vaak karoti । arogya 5 । iti ।\n\
         \n\
         dhara result = 5 pariṇāma [fail-dhatu] ।\n";
    let _ = fs::write("examples/parinama_fallible.dvn", source);
    let result = Compiler::new("examples/parinama_fallible.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("fail_dhatu(5)"),
        "expected single fallible parinama call in generated code:\n{}",
        code
    );
    assert_compiles("fallible parinama", &code);
}

#[test]
fn test_parinama_e2e_mixed_fallible_compiles() {
    let source =
        "fail-dhatu n phalam sankhya vaak karoti । arogya 5 । iti ।\n\
         double-dhatu n karoti । n yoga 2 iti ।\n\
         \n\
         dhara result = 5 pariṇāma [fail-dhatu, double-dhatu] ।\n";
    let _ = fs::write("examples/parinama_mixed.dvn", source);
    let result = Compiler::new("examples/parinama_mixed.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        code.contains("fail_dhatu(5).and_then(|v0| Ok(double_dhatu(v0)))"),
        "expected mixed fallible parinama chain in generated code:\n{}",
        code
    );
    assert_compiles("mixed fallible parinama", &code);
}

