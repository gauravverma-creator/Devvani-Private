use clap::{Parser as ClapParser, Subcommand};
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_ast::visitor::{ASTVisitor, PrettyPrinter};
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
        #[arg(long)]
        symbols: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Lex { file, json, sandhi_off } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => { eprintln!("Error reading file: {}", e); return; }
            };
            let mut lexer = Lexer::new(&source);
            let sandhi_mode = if *sandhi_off { SandhiMode::Off } else { SandhiMode::Auto };
            match lexer.tokenize(sandhi_mode) {
                Ok(tokens) => {
                    if *json { println!("{}", serde_json::to_string_pretty(&tokens).unwrap()); }
                    else { for token in tokens { println!("{:?}", token); } }
                }
                Err(e) => eprintln!("Lex error: {}", e),
            }
        }
        Commands::Parse { file, json, symbols } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => { eprintln!("Error reading file: {}", e); return; }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize(SandhiMode::Auto) {
                Ok(t) => t,
                Err(e) => { eprintln!("Lex error: {}", e); return; }
            };
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&ast).unwrap());
                    } else {
                        let mut printer = PrettyPrinter::new();
                        let _ = printer.visit(&ast);
                    }
                    if *symbols {
                        println!("\nSymbol Table:");
                        println!("Note: Symbol table inspection enabled.");
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
    }
}
