use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_codegen::{DevvaniCodegen, TargetArch};
pub use devvani_codegen::TargetArch as Arch; // Re-export for convenience
use inkwell::context::Context;

#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    None,      // -O0
    Basic,     // -O1
    Full,      // -O2
    Extreme,   // -O3
}

pub struct DevvaniCompiler {
    source: String,
    output_path: String,
    target: TargetArch,
    emit_ir: bool,
}

pub struct CompileOutput {
    pub binary_path: String,
    pub ir_path: Option<String>,
    pub compile_time_ms: u64,
    pub binary_size_bytes: u64,
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
            output_path: "output.exe".to_string(),
            target: TargetArch::Native,
            emit_ir: false,
        }
    }

    pub fn compile(&self) -> Result<CompileOutput, DevvaniError> {
        let start = std::time::Instant::now();
        
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize(SandhiMode::Auto).map_err(|e| DevvaniError::LexerError(format!("{:?}", e)))?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| DevvaniError::ParserError(format!("{:?}", e)))?;

        let context = Context::create();
        let mut codegen = DevvaniCodegen::new(&context, "devvani_module");
        codegen.compile_program(&program).map_err(|e| DevvaniError::CodegenError(format!("{:?}", e)))?;

        if self.emit_ir {
            let ir = codegen.get_ir();
            std::fs::write("output.ll", ir).map_err(|e| DevvaniError::IOError(e.to_string()))?;
        }

        codegen.write_binary(&self.output_path, self.target).map_err(|e| DevvaniError::CodegenError(format!("{:?}", e)))?;

        let duration = start.elapsed();
        let binary_size = std::fs::metadata(&self.output_path).map(|m| m.len()).unwrap_or(0);

        Ok(CompileOutput {
            binary_path: self.output_path.clone(),
            ir_path: if self.emit_ir { Some("output.ll".to_string()) } else { None },
            compile_time_ms: duration.as_millis() as u64,
            binary_size_bytes: binary_size,
        })
    }

    pub fn check(&self) -> Result<(), DevvaniError> {
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize(SandhiMode::Auto).map_err(|e| DevvaniError::LexerError(format!("{:?}", e)))?;
        let mut parser = Parser::new(tokens);
        let _program = parser.parse().map_err(|e| DevvaniError::ParserError(format!("{:?}", e)))?;
        Ok(())
    }

    pub fn emit_ir(&self) -> Result<String, DevvaniError> {
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize(SandhiMode::Auto).map_err(|e| DevvaniError::LexerError(format!("{:?}", e)))?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| DevvaniError::ParserError(format!("{:?}", e)))?;

        let context = Context::create();
        let mut codegen = DevvaniCodegen::new(&context, "devvani_ir");
        codegen.compile_program(&program).map_err(|e| DevvaniError::CodegenError(format!("{:?}", e)))?;

        Ok(codegen.get_ir())
    }

    pub fn run_jit(&self) -> Result<(), DevvaniError> {
        let mut lexer = Lexer::new(&self.source);
        let tokens = lexer.tokenize(SandhiMode::Auto).map_err(|e| DevvaniError::LexerError(format!("{:?}", e)))?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| DevvaniError::ParserError(format!("{:?}", e)))?;

        let context = Context::create();
        let mut codegen = DevvaniCodegen::new(&context, "devvani_jit");
        codegen.compile_program(&program).map_err(|e| DevvaniError::CodegenError(format!("{:?}", e)))?;

        let ir_path = "temp_program.ll";
        let obj_path = "temp_program.o";
        let bin_path = "./temp_program_exec";
        
        std::fs::write(ir_path, codegen.get_ir()).map_err(|e| DevvaniError::IOError(e.to_string()))?;

        let llc_status = std::process::Command::new("llc-17")
            .args([ir_path, "-o", obj_path, "-filetype=obj"])
            .status()
            .map_err(|e| DevvaniError::IOError(e.to_string()))?;
        if !llc_status.success() {
            println!("IR causing failure:\n{}", codegen.get_ir());
            return Err(DevvaniError::CodegenError("llc-17 failed".to_string()));
        }

        let clang_status = std::process::Command::new("clang-17")
            .args([obj_path, "-o", bin_path, "-fPIC", "-no-pie"])
            .status()
            .map_err(|e| DevvaniError::IOError(e.to_string()))?;
        if !clang_status.success() {
            println!("IR causing failure:\n{}", codegen.get_ir());
            return Err(DevvaniError::CodegenError("clang-17 linking failed".to_string()));
        }

        Ok(())
    }
    
    pub fn with_emit_ir(mut self, emit: bool) -> Self {
        self.emit_ir = emit;
        self
    }

    pub fn with_output(mut self, path: &str) -> Self {
        self.output_path = path.to_string();
        self
    }
}
