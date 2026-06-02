use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_typesystem::checker::TypeChecker;
use devvani_llvm::codegen::IrEmitter;
use devvani_llvm::target::DevvaniTarget;
use devvani_compiler::diagnostics::DiagnosticEngine;
use inkwell::context::Context;
use inkwell::targets::FileType;
use std::fs;
use std::path::Path;

#[derive(ClapParser)]
#[command(name = "devvani")]
#[command(about = "Devvani Compiler - Sanskrit-Powered AI Language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
enum EmitTarget {
    Ir,
    Obj,
    Binary,
}

#[derive(Subcommand)]
enum Commands {
    /// Tokenize a Devvani file
    Lex {
        file: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        sandhi_off: bool,
    },
    /// Parse a Devvani file and show AST
    Parse {
        file: String,
        #[arg(long)]
        json: bool,
    },
    /// Compile a Devvani file (defaults to binary)
    Compile {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long, value_enum, default_value = "binary")]
        emit: EmitTarget,
    },
    /// Check a Devvani file (Type checking + Symbol Table)
    Check {
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lex { file, json, sandhi_off } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::IoError(e.to_string()));
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let sandhi_mode = if *sandhi_off { SandhiMode::Off } else { SandhiMode::Auto };
            match lexer.tokenize(sandhi_mode) {
                Ok(tokens) => {
                    if *json { println!("{}", serde_json::to_string_pretty(&tokens).unwrap()); }
                    else { for token in tokens { println!("{:?}", token); } }
                }
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::LexError(format!("{:?}", e)));
                    eprintln!("{}", diag.display());
                }
            }
        }
        Commands::Parse { file, json } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::IoError(e.to_string()));
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::LexError(format!("{:?}", e)));
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
                    } else {
                        println!("{:#?}", ast);
                    }
                }
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::ParseError(format!("{:?}", e)));
                    eprintln!("{}", diag.display());
                }
            }
        }
        Commands::Compile { file, output, emit } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return;
                }
            };

            // 1. Lex
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Lexer error: {:?}", e);
                    return;
                }
            };

            // 2. Parse
            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Parser error: {:?}", e);
                    return;
                }
            };

            // 3. Type check
            let mut checker = TypeChecker::new();
            let errors = checker.check_program(&ast);
            if !errors.is_empty() {
                let diagnostics: Vec<_> = errors.iter()
                    .map(|e| DiagnosticEngine::from_type_error(e))
                    .collect();
                eprintln!("{}", DiagnosticEngine::report(&diagnostics));
                return;
            }

            // 4. LLVM Codegen
            let context = Context::create();
            let module_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
            let mut emitter = IrEmitter::new(&context, module_name);
            let ir = match emitter.emit_ir(&ast) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Codegen error: {}", e);
                    return;
                }
            };

            let default_out = Path::new(file).with_extension("");
            let out_base = output.as_deref().unwrap_or_else(|| default_out.to_str().unwrap());

            match emit {
                EmitTarget::Ir => {
                    let out_path = format!("{}.ll", out_base);
                    fs::write(&out_path, ir).expect("Failed to write IR");
                    println!("✓ IR written to: {}", out_path);
                }
                EmitTarget::Obj => {
                    let out_path = format!("{}.o", out_base);
                    let target = DevvaniTarget::new_native().expect("Failed to init native target");
                    target.machine.write_to_file(&emitter.module, FileType::Object, Path::new(&out_path))
                        .expect("Failed to write object file");
                    println!("✓ Object file written to: {}", out_path);
                }
                EmitTarget::Binary => {
                    let obj_path = format!("{}.o", out_base);
                    let target = DevvaniTarget::new_native().expect("Failed to init native target");
                    target.machine.write_to_file(&emitter.module, FileType::Object, Path::new(&obj_path))
                        .expect("Failed to write temporary object file");
                    
                    let bin_path = out_base;
                    let status = std::process::Command::new("cc")
                        .arg(&obj_path)
                        .arg("-o")
                        .arg(bin_path)
                        .status()
                        .expect("Failed to run linker (cc)");

                    if status.success() {
                        println!("✓ Binary written to: {}", bin_path);
                        let _ = fs::remove_file(obj_path);
                    } else {
                        eprintln!("Linking failed");
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Check { file } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::IoError(e.to_string()));
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::LexError(format!("{:?}", e)));
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(a) => a,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(&devvani_compiler::CompilerError::ParseError(format!("{:?}", e)));
                    eprintln!("{}", diag.display());
                    return;
                }
            };

            let mut checker = TypeChecker::new();
            let errors = checker.check_program(&ast);
            
            println!("--- Symbol Table Check ---");
            let diagnostics: Vec<_> = errors.iter()
                .map(|e| DiagnosticEngine::from_type_error(e))
                .collect();
            
            println!("{}", DiagnosticEngine::report(&diagnostics));
        }
    }
}
