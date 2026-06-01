use clap::{Parser as ClapParser, Subcommand};
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_compiler::{Compiler, diagnostics::DiagnosticEngine};
use std::fs;

#[derive(ClapParser)]
#[command(name = "devvani")]
#[command(about = "Devvani Compiler - Sanskrit-Powered AI Language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    /// Compile a Devvani file to Rust
    Compile {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
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
        Commands::Compile { file, output } => {
            let mut compiler = Compiler::new(file);
            if let Some(out) = output {
                compiler = compiler.with_output(out);
            }
            match compiler.compile() {
                Ok(rust_source) => {
                    if output.is_none() {
                        println!("{}", rust_source);
                    }
                }
                Err(report) => eprintln!("{}", report),
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

            let mut codegen = devvani_codegen::Codegen::new(devvani_codegen::CodegenTarget::RustSource);
            let errors = codegen.type_checker.check_program(&ast);
            
            println!("--- Symbol Table Check ---");
            let diagnostics: Vec<_> = errors.iter()
                .map(|e| DiagnosticEngine::from_type_error(e))
                .collect();
            
            println!("{}", DiagnosticEngine::report(&diagnostics));
        }
    }
}
