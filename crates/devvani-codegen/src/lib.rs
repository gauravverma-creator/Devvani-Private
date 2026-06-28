use devvani_ast::ASTNode;
use devvani_ast::KarakaRole;
use devvani_typesystem::{
    TypeChecker, Lakara, 
    lakara_to_scope, lakara_from_str,
    vaak::{MoveChecker, VaakOwnership},
};

// ── Error type ──────────────────────────────────────────────
#[derive(Debug)]
pub enum CodegenError {
    UnsupportedNode(String),
    TypeCheckFailed(String),
    IoError(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedNode(s) => write!(f, "Unsupported node: {}", s),
            CodegenError::TypeCheckFailed(s) => write!(f, "Type check failed: {}", s),
            CodegenError::IoError(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::error::Error for CodegenError {}

// ── Output target ────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodegenTarget {
    RustSource,   // emit .rs source
    Bytecode,     // emit simple bytecode (Vec<Instruction>)
}

// ── Simple bytecode instructions ─────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Bind { name: String, rust_type: String, mutable: bool },
    Call { subject: String, verb: String, args: Vec<String> },
    EnterScope { name: String, is_async: bool },
    ExitScope,
    Return { value: String },
    Comment(String),
    Output(String),
    Input(String),
    Yoga, Viyoga, Guna, Bhaga,
    Sama, Asama, Nyuuna, Adhika,
}

// ── Main codegen struct ───────────────────────────────────────
pub struct Codegen {
    pub target: CodegenTarget,
    pub type_checker: TypeChecker,
    pub move_checker: MoveChecker,
    pub instructions: Vec<Instruction>,
    pub rust_output: String,
    indent: usize,
}

impl Codegen {
    pub fn new(target: CodegenTarget) -> Self {
        Self {
            target,
            type_checker: TypeChecker::new(),
            move_checker: MoveChecker::new(),
            instructions: Vec::new(),
            rust_output: String::new(),
            indent: 0,
        }
    }

    pub fn generate(&mut self, node: &ASTNode) -> Result<(), CodegenError> {
        // 1. Run type checker first
        let errors = self.type_checker.check_program(node);
        if !errors.is_empty() {
            return Err(CodegenError::TypeCheckFailed(format!("{:?}", errors)));
        }

        // 2. Run move checker on VaakNode/VaakYogaNode
        let move_errors = self.check_moves(node);
        if !move_errors.is_empty() {
            return Err(CodegenError::TypeCheckFailed(move_errors.join("\n")));
        }

        // 3. Then call emit(node)
        self.emit(node)
    }

    fn check_moves(&mut self, node: &ASTNode) -> Vec<String> {
        let mut errors = Vec::new();
        self.walk_for_moves(node, &mut errors);
        errors
    }

    fn walk_for_moves(&mut self, node: &ASTNode, errors: &mut Vec<String>) {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.walk_for_moves(stmt, errors);
                }
            }
            ASTNode::VaakNode { naama, karaka, .. } => {
                if *karaka == KarakaRole::Apadana {
                    if let Some(VaakOwnership::Moved) = self.move_checker.ownership_map.get(naama) {
                        errors.push(format!("Doṣa D030: '{}' — svāmitva-hāni (ownership moved, cannot use)", naama));
                    } else {
                        self.move_checker.do_move(naama).ok();
                    }
                } else {
                    let ownership = if *karaka == KarakaRole::Karta {
                        VaakOwnership::Karta
                    } else if *karaka == KarakaRole::Karana {
                        VaakOwnership::Karana
                    } else {
                        VaakOwnership::Karta
                    };
                    self.move_checker.register(naama.clone(), ownership);
                }
            }
            _ => {}
        }
    }

    fn emit(&mut self, node: &ASTNode) -> Result<(), CodegenError> {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.emit(stmt)?;
                }
            }
            ASTNode::Nama { base, .. } => {
                let display_name = if base.to_lowercase().ends_with("ah") {
                    &base[..base.len()-2]
                } else {
                    base
                };

                if let Some(symbol) = self.type_checker.env.lookup(base) {
                    self.instructions.push(Instruction::Bind {
                        name: symbol.name.clone(),
                        rust_type: symbol.rust_type_hint.clone(),
                        mutable: symbol.mutability.is_mutable,
                    });
                    
                    let line = format!("{}let {}: {};\n", 
                        self.indent_str(),
                        if symbol.mutability.is_mutable { format!("mut {}", display_name) } else { display_name.to_string() },
                        symbol.rust_type_hint
                    );
                    self.rust_output.push_str(&line);
                } else {
                    let line = format!("{}let {};\n", self.indent_str(), display_name);
                    self.rust_output.push_str(&line);
                }
            }
            ASTNode::PurnaankLiteral { value, .. } => {
                self.rust_output.push_str(&value.to_string());
            }
            ASTNode::DashaamshaLiteral { value, .. } => {
                self.rust_output.push_str(&value.to_string());
            }
            ASTNode::VaakLiteral { value, .. } => {
                self.rust_output.push_str(&format!("\"{}\"", value));
            }
            ASTNode::VaakNode { naama: _, mulya, karaka, is_mutable, span: _ } => {
                let kw = if *is_mutable { "let mut" } else { "let" };
                self.rust_output.push_str(&format!("{} {} = ", self.indent_str(), kw));
                match karaka {
                    KarakaRole::Karana => self.rust_output.push_str("&"),
                    _ => {}
                }
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
            }
            ASTNode::VaakYogaNode { vama, dakshina, span: _ } => {
                self.emit(vama)?;
                self.rust_output.push_str(" + ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Yoga);
            }
            ASTNode::AstiNode { naama, mulya } => {
                self.rust_output.push_str(&format!("{}let {} = ", self.indent_str(), naama));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind { name: naama.clone(), rust_type: "auto".into(), mutable: false });
            }
            ASTNode::BhavatiNode { naama, mulya } => {
                self.rust_output.push_str(&format!("{}let mut {} = ", self.indent_str(), naama));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind { name: naama.clone(), rust_type: "auto".into(), mutable: true });
            }
            ASTNode::YogaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" + ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Yoga);
            }
            ASTNode::ViyogaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" - ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Viyoga);
            }
            ASTNode::GunaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" * ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Guna);
            }
            ASTNode::BhagaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" / ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Bhaga);
            }
            ASTNode::SamaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" == ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Sama);
            }
            ASTNode::AsamaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" != ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Asama);
            }
            ASTNode::NyuunaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" < ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Nyuuna);
            }
            ASTNode::AdhikaNode { vama, dakshina } => {
                self.emit(vama)?;
                self.rust_output.push_str(" > ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Adhika);
            }
            ASTNode::VadatiNode { mulya } => {
                self.rust_output.push_str(&format!("{}println!(\"{{:?}}\", ", self.indent_str()));
                self.emit(mulya)?;
                self.rust_output.push_str(");\n");
                self.instructions.push(Instruction::Output("stdout".into()));
            }
            ASTNode::PathatiNode { naama } => {
                self.rust_output.push_str(&format!("{}let mut {} = String::new(); std::io::stdin().read_line(&mut {}).unwrap();\n", self.indent_str(), naama, naama));
                self.instructions.push(Instruction::Input(naama.clone()));
            }
            ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
                self.rust_output.push_str(&format!("{}if ", self.indent_str()));
                self.emit(sthiti)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                for stmt in tarhi { self.emit(stmt)?; }
                self.indent -= 1;
                if let Some(else_body) = anyatha {
                    self.rust_output.push_str(&format!("{}}} else {{\n", self.indent_str()));
                    self.indent += 1;
                    for stmt in else_body { self.emit(stmt)?; }
                    self.indent -= 1;
                }
                self.rust_output.push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::YavatNode { sthiti, shareera } => {
                self.rust_output.push_str(&format!("{}while ", self.indent_str()));
                self.emit(sthiti)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                for stmt in shareera { self.emit(stmt)?; }
                self.indent -= 1;
                self.rust_output.push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::PunahNode { varam, shareera } => {
                self.rust_output.push_str(&format!("{}for _ in 0..", self.indent_str()));
                self.emit(varam)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                for stmt in shareera { self.emit(stmt)?; }
                self.indent -= 1;
                self.rust_output.push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::KriyaCall { karta, kriya, karma, .. } => {
                let subject = if let Some(k) = karta {
                    match &**k {
                        ASTNode::Nama { base, .. } => base.clone(),
                        ASTNode::Samasa { resolved, .. } => resolved.clone(),
                        _ => "self".to_string(),
                    }
                } else {
                    "self".to_string()
                };

                let mut arg_names = Vec::new();
                for arg in karma {
                    if let ASTNode::Nama { base, .. } = arg {
                        arg_names.push(base.clone());
                    } else if let ASTNode::PurnaankLiteral { value, .. } = arg {
                        arg_names.push(value.to_string());
                    } else if let ASTNode::Samasa { resolved, .. } = arg {
                        arg_names.push(resolved.clone());
                    }
                }

                self.instructions.push(Instruction::Call {
                    subject: subject.clone(),
                    verb: kriya.clone(),
                    args: arg_names.clone(),
                });

                let line = format!("{}.{}({});\n", subject, kriya, arg_names.join(", "));
                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str(&line);
            }
            ASTNode::DhatuDef { name, params, body, lakara, .. } => {
                let ts_lakara = lakara_from_str(&format!("{:?}", lakara)).unwrap_or(Lakara::Lat);
                let scope = lakara_to_scope(&ts_lakara);
                
                self.instructions.push(Instruction::EnterScope {
                    name: name.clone(),
                    is_async: scope.is_async,
                });

                let mut rust_params = Vec::new();
                for param in params {
                    rust_params.push(format!("{}: i64", param.name));
                    self.instructions.push(Instruction::Bind {
                        name: param.name.clone(),
                        rust_type: "i64".to_string(),
                        mutable: false,
                    });
                }

                let async_kw = if scope.is_async { "async " } else { "" };
                let line = format!("{}pub {}fn {}({}) {{\n", self.indent_str(), async_kw, name, rust_params.join(", "));
                self.rust_output.push_str(&line);
                
                self.indent += 1;
                for stmt in body {
                    self.emit(stmt)?;
                }
                self.indent -= 1;
                
                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str("}\n");
                
                self.instructions.push(Instruction::ExitScope);
            }
            ASTNode::Samasa { components, resolved, .. } => {
                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str(&resolved);
                self.rust_output.push_str("\n");
                
                self.instructions.push(Instruction::Comment(format!("samasa: {}", components.join("."))));
            }
            _ => {
                let msg = format!("Unhandled node: {:?}", node);
                self.instructions.push(Instruction::Comment(msg.clone()));
                self.rust_output.push_str(&format!("{}// {}\n", self.indent_str(), msg));
            }
        }
        Ok(())
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    pub fn rust_source(&self) -> &str {
        &self.rust_output
    }

    pub fn bytecode(&self) -> &[Instruction] {
        &self.instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_ast::{ASTNode, Vibhakti, Linga as AstLinga, Vacana as AstVacana, Span};

    fn dummy_span() -> Span {
        Span { line: 0, col: 0, len: 0 }
    }

    #[test]
    fn test_empty_program() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::KaryakramNode { shareera: vec![] };
        assert!(codegen.generate(&node).is_ok());
    }

    #[test]
    fn test_nama_node() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::Nama {
            base: "Ramah".to_string(),
            vibhakti: Vibhakti::Prathama,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            span: dummy_span(),
        };
        let program = ASTNode::KaryakramNode { shareera: vec![node] };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.rust_source().contains("let Ram"));
    }
}
