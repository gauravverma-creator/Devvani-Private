pub mod diagnostics;

use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_codegen::{Codegen, CodegenTarget};
use diagnostics::{Diagnostic, DiagnosticEngine};
use std::fmt;

#[derive(Debug)]
pub enum CompilerError {
    IoError(String),
    LexError(String),
    ParseError(String),
    CodegenError(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::IoError(s) => write!(f, "IO Error: {}", s),
            CompilerError::LexError(s) => write!(f, "Lex Error: {}", s),
            CompilerError::ParseError(s) => write!(f, "Parse Error: {}", s),
            CompilerError::CodegenError(s) => write!(f, "Codegen Error: {}", s),
        }
    }
}

impl std::error::Error for CompilerError {}

pub struct Compiler {
    pub source_path: String,
    pub output_path: Option<String>,
    pub target: CodegenTarget,
}

impl Compiler {
    pub fn new(source_path: &str) -> Self {
        Compiler {
            source_path: source_path.to_string(),
            output_path: None,
            target: CodegenTarget::RustSource,
        }
    }

    pub fn with_output(mut self, path: &str) -> Self {
        self.output_path = Some(path.to_string());
        self
    }

    pub fn compile(&self) -> Result<String, String> {
        match self.compile_with_diagnostics() {
            Ok(s) => Ok(s),
            Err(diags) => Err(DiagnosticEngine::report(&diags)),
        }
    }

    pub fn compile_with_diagnostics(&self) -> Result<String, Vec<Diagnostic>> {
        let source = std::fs::read_to_string(&self.source_path)
            .map_err(|e| vec![DiagnosticEngine::from_compiler_error(&CompilerError::IoError(e.to_string()))])?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize(SandhiMode::Auto)
            .map_err(|e| vec![DiagnosticEngine::from_compiler_error(&CompilerError::LexError(format!("{:?}", e)))])?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| vec![DiagnosticEngine::from_compiler_error(&CompilerError::ParseError(format!("{:?}", e)))])?;

        // DEVVANI STDLIB HOOK: 
        // Before resolving user-defined functions, check prelude.
        // This gives every .dvn file access to all 70 Dhatus automatically.
        // use devvani_stdlib::prelude::devvani_prelude;
        // let prelude = devvani_prelude();

        let mut codegen = Codegen::new(self.target);
        if let Err(e) = codegen.generate(&ast) {
            match e {
                devvani_codegen::CodegenError::TypeCheckFailed(_) => {
                    // TypeCheckFailed contains the list of errors. 
                    // Let's re-run type checker to get the actual TypeCheckErrors.
                    // This is slightly inefficient but keeps the API clean for now.
                    let errors = codegen.type_checker.errors.iter()
                        .map(|te| DiagnosticEngine::from_type_error(te))
                        .collect();
                    return Err(errors);
                }
                _ => return Err(vec![DiagnosticEngine::from_codegen_error(&e)]),
            }
        }

        let output = codegen.rust_source().to_string();

        if let Some(ref out) = self.output_path {
            std::fs::write(out, &output)
                .map_err(|e| vec![DiagnosticEngine::from_compiler_error(&CompilerError::IoError(e.to_string()))])?;
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compile_nonexistent_file() {
        let compiler = Compiler::new("nonexistent.dvn");
        let result = compiler.compile();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("D007"));
    }

    #[test]
    fn test_compile_hello() {
        let _ = fs::create_dir_all("examples");
        let _ = fs::write("examples/hello_test.dvn", "rāmaḥ karoti phalam.");
        
        let compiler = Compiler::new("examples/hello_test.dvn");
        let result = compiler.compile();
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_ganana() {
        let _ = fs::write("examples/ganana_test.dvn", "ekaḥ. yogaḥ karoti dvitiīyaḥ.");
        let compiler = Compiler::new("examples/ganana_test.dvn");
        let result = compiler.compile();
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_output_writes_file() {
        let _ = fs::write("examples/output_test.dvn", "rāmaḥ.");
        let out_file = "examples/output_test.rs";
        if fs::metadata(out_file).is_ok() {
            let _ = fs::remove_file(out_file);
        }

        let compiler = Compiler::new("examples/output_test.dvn").with_output(out_file);
        let result = compiler.compile();
        assert!(result.is_ok());
        assert!(fs::metadata(out_file).is_ok());
        let _ = fs::remove_file(out_file);
    }
}
