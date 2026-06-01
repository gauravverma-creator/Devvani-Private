use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_codegen::{Codegen, CodegenTarget};

pub struct DevvaniCompiler {
    source: String,
    output_path: String,
    target: CodegenTarget,
}

pub struct CompileOutput {
    pub binary_path: String,
    pub rust_path: Option<String>,
    pub compile_time_ms: u64,
}

#[derive(Debug)]
pub enum DevvaniError {
    LexerError(String),
    ParserError(String),
    CodegenError(String),
    IOError(String),
}

impl DevvaniCompiler {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            output_path: "output.rs".to_string(),
            target: CodegenTarget::RustSource,
        }
    }

    pub fn compile(&self) -> Result<CompileOutput, DevvaniError> {
        let start = std::time::Instant::now();
        
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize(SandhiMode::Auto).map_err(|e| DevvaniError::LexerError(format!("{:?}", e)))?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| DevvaniError::ParserError(format!("{:?}", e)))?;

        let mut codegen = Codegen::new(self.target);
        codegen.generate(&program).map_err(|e| DevvaniError::CodegenError(format!("{:?}", e)))?;

        let source = codegen.rust_source();
        std::fs::write(&self.output_path, source).map_err(|e| DevvaniError::IOError(e.to_string()))?;

        let duration = start.elapsed();

        Ok(CompileOutput {
            binary_path: self.output_path.clone(),
            rust_path: Some(self.output_path.clone()),
            compile_time_ms: duration.as_millis() as u64,
        })
    }
}
