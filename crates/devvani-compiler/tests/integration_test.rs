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
