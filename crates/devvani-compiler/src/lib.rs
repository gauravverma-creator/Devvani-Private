use devvani_codegen::{Codegen, CodegenTarget};
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_reversible::VedicBatchEngine;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CompilerError {
    IoError(String),
    LexError(String),
    ParseError(String),
    CodegenError(String),
}

pub struct Compiler {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    pub reversible_engine: Option<VedicBatchEngine>,
}

impl Compiler {
    pub fn new<P: AsRef<Path>>(input: P) -> Self {
        Self {
            input_file: input.as_ref().to_path_buf(),
            output_file: None,
            reversible_engine: None,
        }
    }

    pub fn with_output<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.output_file = Some(output.as_ref().to_path_buf());
        self
    }

    pub fn compile(&self) -> Result<String, String> {
        let source = fs::read_to_string(&self.input_file).map_err(|e| format!("D007: {}", e))?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer
            .tokenize(SandhiMode::Auto)
            .map_err(|e| format!("D008: {:?}", e))?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("D009: {:?}", e))?;

        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen
            .generate(&ast)
            .map_err(|e| format!("D010: {:?}", e))?;

        let rust_code = codegen.rust_source().to_string();

        if let Some(out_path) = &self.output_file {
            fs::write(out_path, &rust_code).map_err(|e| format!("D006: {}", e))?;
        }

        Ok(rust_code)
    }

    /// Initialize the reversible compute engine for this compilation session.
    /// Call this before compiling if reversible tracking is needed.
    /// ssd_dir: path where .dvr/.dvri files will be written.
    pub fn enable_reversible_engine(&mut self, ssd_dir: impl AsRef<std::path::Path>) {
        use devvani_reversible::WindowConfig;
        match VedicBatchEngine::new(
            32 * 1024 * 1024, // 32MB RAM tier
            WindowConfig {
                max_ops: 512,
                purge_fraction: 0.80,
                dependency_check: true,
            },
            ssd_dir,
            16, // coalesce threshold
            64, // batch size
        ) {
            Ok(engine) => {
                self.reversible_engine = Some(engine);
            }
            Err(e) => {
                eprintln!(
                    "[devvani-compiler] warning: reversible engine init failed: {}",
                    e
                );
            }
        }
    }

    /// Returns true if the reversible engine is active for this session.
    pub fn has_reversible_engine(&self) -> bool {
        self.reversible_engine.is_some()
    }
}

pub mod diagnostics;

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
        let _ = fs::write("examples/hello_test.dvn", "phalamulya asti 5 ।");

        let compiler = Compiler::new("examples/hello_test.dvn");
        let result = compiler.compile();
        if let Err(ref e) = result {
            println!("Error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_ganana() {
        let _ = fs::write("examples/ganana_test.dvn", "eka asti 1 । 1 yoga 2 vadati ।");
        let compiler = Compiler::new("examples/ganana_test.dvn");
        let result = compiler.compile();
        if let Err(ref e) = result {
            println!("Error: {}", e);
        }
        assert!(result.is_ok());
    }
}
