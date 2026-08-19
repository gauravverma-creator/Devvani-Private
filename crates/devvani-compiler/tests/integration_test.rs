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

fn assert_test_result(name: &str, generated_code: &str, expect_pass: bool) {
    let tmp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
    let tmp_path = tmp_dir.path().join("parikshaa_verify.rs");
    let out_path = tmp_dir.path().join("parikshaa_verify_out");

    fs::write(&tmp_path, generated_code)
        .expect("failed to write temp file");

    let compile_status = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--test")
        .arg(&tmp_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("failed to run rustc");

    let compile_stderr = String::from_utf8_lossy(&compile_status.stderr);
    assert!(
        compile_status.status.success(),
        "rustc --test failed for {}:\n{}",
        name,
        compile_stderr
    );

    let run_status = Command::new(&out_path)
        .output()
        .expect("failed to run test binary");

    let stdout = String::from_utf8_lossy(&run_status.stdout);
    let stderr = String::from_utf8_lossy(&run_status.stderr);

    if expect_pass {
        assert!(
            run_status.status.success(),
            "test FAILED for {} (expected PASS):\nstdout: {}\nstderr: {}",
            name, stdout, stderr
        );
    } else {
        assert!(
            !run_status.status.success(),
            "test PASSED for {} (expected FAIL):\nstdout: {}\nstderr: {}",
            name, stdout, stderr
        );
    }
}

#[test]
fn test_parikshaa_e2e_non_tarka_passes() {
    let source = "parikshaa my-test {\n    nigamana 1 sama 1 ।\n    \"done\" vadati ।\n}\n";
    let _ = fs::write("examples/parikshaa_pass.dvn", source);
    let result = Compiler::new("examples/parikshaa_pass.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(code.contains("#[test]"));
    assert!(!code.contains("#[should_panic]"));
    assert_test_result("non-tarka passing parikshaa", &code, true);
}

#[test]
fn test_parikshaa_e2e_non_tarka_fails() {
    let source = "parikshaa my-failing-test {\n    nigamana 1 sama 2 ।\n    \"done\" vadati ।\n}\n";
    let _ = fs::write("examples/parikshaa_fail.dvn", source);
    let result = Compiler::new("examples/parikshaa_fail.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(code.contains("#[test]"));
    assert!(!code.contains("#[should_panic]"));
    assert_test_result("non-tarka failing parikshaa", &code, false);
}

#[test]
fn test_parikshaa_e2e_tarka_passes() {
    let source = "tarka parikshaa failing-assert {\n    nigamana 1 sama 2 ।\n    \"done\" vadati ।\n}\n";
    let _ = fs::write("examples/parikshaa_tarka_pass.dvn", source);
    let result = Compiler::new("examples/parikshaa_tarka_pass.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(code.contains("#[should_panic]"));
    assert_test_result("tarka passing parikshaa", &code, true);
}

#[test]
fn test_parikshaa_e2e_tarka_fails() {
    let source = "tarka parikshaa passing-assert {\n    nigamana 1 sama 1 ।\n    \"done\" vadati ।\n}\n";
    let _ = fs::write("examples/parikshaa_tarka_fail.dvn", source);
    let result = Compiler::new("examples/parikshaa_tarka_fail.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(code.contains("#[should_panic]"));
    assert_test_result("tarka failing parikshaa", &code, false);
}

// ===== Versioning (Mrittika / Vikara) E2E Tests =====

fn mrittika_source() -> &'static str {
    "bhashya \"A versioned library\"।\n\
     mrittika \"versioned-lib\" {\n\
         naamadheya \"0.2.0\"।\n\
         satya-bheda \"removed deprecated API\"।\n\
         sukshma-vikara \"fixed a bug\"।\n\
         sthula-vikara \"added new feature\"।\n\
     }\n\
     dhātu increment n karoti । n yoga 1 iti ।\n"
}

#[test]
fn test_mrittika_e2e_metadata_block() {
    let _ = fs::write("examples/mrittika_e2e.dvn", mrittika_source());
    let result = Compiler::new("examples/mrittika_e2e.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();

    assert!(
        code.contains("//! # Devvani Package Metadata (मृत्तिका)"),
        "expected metadata header in:\n{}",
        code
    );
    assert!(
        code.contains("//! - Package: versioned-lib"),
        "expected package name in:\n{}",
        code
    );
    assert!(
        code.contains("//! - Version (नामधेय): 0.2.0"),
        "expected version string in:\n{}",
        code
    );
    assert!(
        code.contains("//! - [SATYA-BHEDA] removed deprecated API"),
        "expected satya-bheda entry in:\n{}",
        code
    );
    assert!(
        code.contains("//! - [SUKSHMA] fixed a bug"),
        "expected sukshma entry in:\n{}",
        code
    );
    assert!(
        code.contains("//! - [STHULA] added new feature"),
        "expected sthula entry in:\n{}",
        code
    );

    // Bhashya must appear before the metadata block
    let bhashya_pos = code.find("//! A versioned library").unwrap();
    let metadata_pos = code.find("//! # Devvani Package Metadata").unwrap();
    assert!(
        bhashya_pos < metadata_pos,
        "Bhashya must appear before metadata block"
    );

    // Vikara entries must preserve source order (satya-bheda, sukshma, sthula)
    let sb = code
        .find("//! - [SATYA-BHEDA] removed deprecated API")
        .unwrap();
    let sm = code.find("//! - [SUKSHMA] fixed a bug").unwrap();
    let st = code
        .find("//! - [STHULA] added new feature")
        .unwrap();
    assert!(
        sb < sm && sm < st,
        "vikara entries must preserve source order in:\n{}",
        code
    );
}

#[test]
fn test_mrittika_e2e_rustc_compiles_as_bin() {
    let _ = fs::write("examples/mrittika_rustc.dvn", mrittika_source());
    let result = Compiler::new("examples/mrittika_rustc.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert_compiles("mrittika_e2e_bin", &code);
}

#[test]
fn test_mrittika_e2e_rustc_compiles_as_lib() {
    let _ = fs::write("examples/mrittika_lib.dvn", mrittika_source());
    let result = Compiler::new("examples/mrittika_lib.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();

    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let rust_path = tmp_dir.path().join("mrittika_lib_verify.rs");
    fs::write(&rust_path, &code).expect("failed to write temp rust file");

    let status = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--crate-type")
        .arg("lib")
        .arg("--crate-name")
        .arg("mrittika_lib_verify")
        .arg(&rust_path)
        .output()
        .expect("failed to run rustc");

    let stderr = String::from_utf8_lossy(&status.stderr);
    assert!(
        status.status.success(),
        "rustc --crate-type lib failed for mrittika e2e:\n{}",
        stderr
    );
}

#[test]
fn test_mrittika_no_block_emits_no_metadata() {
    let _ = fs::write(
        "examples/no_mrittika.dvn",
        "dhātu myfunc n karoti । n yoga 1 iti ।\n",
    );
    let result = Compiler::new("examples/no_mrittika.dvn").compile();
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(
        !code.contains("//! # Devvani Package Metadata"),
        "no metadata should be emitted without a mrittika block:\n{}",
        code
    );
}
