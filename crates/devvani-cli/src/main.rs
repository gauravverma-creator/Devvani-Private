use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use devvani_codegen::{Codegen, CodegenTarget};
use devvani_compiler::diagnostics::DiagnosticEngine;
use devvani_compiler::Compiler;
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_typesystem::checker::TypeChecker;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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
    Check { file: String },
    /// Install a Devvani module
    Install { package: String },
    /// List loaded modules and registry info
    Modules,
    /// Generate HTML documentation for a Devvani file
    Doc {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn sanitize_crate_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect()
}

fn write_cargo_project(dir: &Path, crate_name: &str, rust_code: &str) -> std::io::Result<()> {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            crate_name
        ),
    )?;
    fs::write(src_dir.join("lib.rs"), rust_code)?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
            println!(
                "āyātaḥ: installing package '{}' from https://registry.kosha.dev",
                package
            );
            let loader = devvani_module::ModuleLoader::new();
            let path = loader.cache_path(package);
            println!("siddham — package path resolved: {}", path.display());
        }
        Commands::Modules => {
            let pipeline = devvani_module::ModulePipeline::new();
            println!("koṣaḥ-sūcī — Devvani Module System v0.1.0");
            println!("Official Registry: https://registry.kosha.dev");
            println!("Local Cache: ~/.devvani/packages/");
            println!("Loaded modules: {}", pipeline.loaded_module_count());
        }
        Commands::Lex {
            file,
            json,
            sandhi_off,
        } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::IoError(e.to_string()),
                    );
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let sandhi_mode = if *sandhi_off {
                SandhiMode::Off
            } else {
                SandhiMode::Auto
            };
            match lexer.tokenize(sandhi_mode) {
                Ok(tokens) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&tokens).unwrap());
                    } else {
                        for token in tokens {
                            println!("{:?}", token);
                        }
                    }
                }
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::LexError(format!("{:?}", e)),
                    );
                    eprintln!("{}", diag.display());
                }
            }
        }
        Commands::Parse { file, json } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::IoError(e.to_string()),
                    );
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::LexError(format!("{:?}", e)),
                    );
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
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::ParseError(format!("{:?}", e)),
                    );
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
                let diagnostics: Vec<_> = errors
                    .iter()
                    .map(|e| DiagnosticEngine::from_type_error(e))
                    .collect();
                eprintln!("{}", DiagnosticEngine::report(&diagnostics));
                return;
            }

            // 4. Codegen (Rust source emission)
            let mut codegen = Codegen::new(CodegenTarget::RustSource);
            if let Err(e) = codegen.generate(&ast) {
                eprintln!("Codegen error: {:?}", e);
                return;
            }
            let rust_code = codegen.rust_source().to_string();

            let tmp_dir = match TempDir::new() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to create temp directory: {}", e);
                    std::process::exit(1);
                }
            };
            let tmp_rs_path = tmp_dir.path().join("generated.rs");
            if let Err(e) = fs::write(&tmp_rs_path, &rust_code) {
                eprintln!("Failed to write generated Rust source: {}", e);
                std::process::exit(1);
            }

            let binary_rs_path = tmp_dir.path().join("generated_binary.rs");
            let wrapped = format!("fn main() {{\n{}\n}}", rust_code);
            if let Err(e) = fs::write(&binary_rs_path, &wrapped) {
                eprintln!("Failed to write wrapped Rust source: {}", e);
                std::process::exit(1);
            }

            let default_out = Path::new(file).with_extension("");
            let out_base = output
                .as_deref()
                .unwrap_or_else(|| default_out.to_str().unwrap());

            match emit {
                EmitTarget::Ir => {
                    let out_path = format!("{}.rs", out_base);
                    fs::write(&out_path, rust_code).expect("Failed to write Rust source");
                    println!("✓ Rust source written to: {}", out_path);
                }
                EmitTarget::Obj => {
                    let out_path = format!("{}.o", out_base);
                    let output = std::process::Command::new("rustc")
                        .arg(&binary_rs_path)
                        .arg("--emit=obj")
                        .arg("-o")
                        .arg(&out_path)
                        .output()
                        .expect("Failed to run rustc");

                    if output.status.success() {
                        println!("✓ Object file written to: {}", out_path);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("rustc failed:\n{}", stderr);
                        std::process::exit(1);
                    }
                }
                EmitTarget::Binary => {
                    let bin_path = out_base;
                    let output = std::process::Command::new("rustc")
                        .arg(&binary_rs_path)
                        .arg("-o")
                        .arg(bin_path)
                        .output()
                        .expect("Failed to run rustc");

                    if output.status.success() {
                        println!("✓ Binary written to: {}", bin_path);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("rustc failed:\n{}", stderr);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Check { file } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::IoError(e.to_string()),
                    );
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::LexError(format!("{:?}", e)),
                    );
                    eprintln!("{}", diag.display());
                    return;
                }
            };
            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(a) => a,
                Err(e) => {
                    let diag = DiagnosticEngine::from_compiler_error(
                        &devvani_compiler::CompilerError::ParseError(format!("{:?}", e)),
                    );
                    eprintln!("{}", diag.display());
                    return;
                }
            };

            let mut checker = TypeChecker::new();
            let errors = checker.check_program(&ast);

            println!("--- Symbol Table Check ---");
            let diagnostics: Vec<_> = errors
                .iter()
                .map(|e| DiagnosticEngine::from_type_error(e))
                .collect();

            println!("{}", DiagnosticEngine::report(&diagnostics));
        }
        Commands::Doc { file, output } => {
            let compiler = Compiler::new(file);
            let rust_code = match compiler.compile() {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("Compilation error: {}", e);
                    std::process::exit(1);
                }
            };

            let crate_name = sanitize_crate_name(
                Path::new(file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("devvani_doc"),
            );

            let project_dir = if let Some(output_dir) = output {
                let path = PathBuf::from(output_dir);
                if let Err(e) = write_cargo_project(&path, &crate_name, &rust_code) {
                    eprintln!("Failed to write Cargo project: {}", e);
                    std::process::exit(1);
                }
                path
            } else {
                let tmp_dir = match TempDir::new() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Failed to create temp directory: {}", e);
                        std::process::exit(1);
                    }
                };
                let path = tmp_dir.path().to_path_buf();
                if let Err(e) = write_cargo_project(&path, &crate_name, &rust_code) {
                    eprintln!("Failed to write Cargo project: {}", e);
                    std::process::exit(1);
                }
                tmp_dir.keep()
            };

            let cargo_output = match std::process::Command::new("cargo")
                .arg("doc")
                .arg("--no-deps")
                .current_dir(&project_dir)
                .output()
            {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Failed to run cargo doc: {}", e);
                    std::process::exit(1);
                }
            };

            if !cargo_output.status.success() {
                let stderr = String::from_utf8_lossy(&cargo_output.stderr);
                eprintln!("cargo doc failed:\n{}", stderr);
                std::process::exit(1);
            }

            let doc_path = project_dir.join("target").join("doc").join(&crate_name).join("index.html");
            println!("{}", doc_path.display());
        }
    }
}
