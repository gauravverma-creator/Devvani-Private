use devvani_compiler::Compiler;

#[test]
fn test_pipeline_hello() {
    let result = Compiler::new("../../examples/hello.dvn").compile();
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_pipeline_samasa() {
    let result = Compiler::new("../../examples/samasa.dvn").compile();
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_rust_output_not_empty() {
    let result = Compiler::new("../../examples/hello.dvn").compile();
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
