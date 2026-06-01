use devvani_ast::ASTNode;
use devvani_typesystem::{
    TypeChecker, Lakara, SamasaKind,
    lakara_to_scope, lakara_from_str, samasa_from_str, resolve_samasa,
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
}

// ── Main codegen struct ───────────────────────────────────────
pub struct Codegen {
    pub target: CodegenTarget,
    pub type_checker: TypeChecker,
    pub instructions: Vec<Instruction>,
    pub rust_output: String,
    indent: usize,
}

impl Codegen {
    pub fn new(target: CodegenTarget) -> Self {
        Self {
            target,
            type_checker: TypeChecker::new(),
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

        // 2. Then call emit(node)
        self.emit(node)
    }

    fn emit(&mut self, node: &ASTNode) -> Result<(), CodegenError> {
        match node {
            ASTNode::Program { statements, .. } => {
                for stmt in statements {
                    self.emit(stmt)?;
                }
            }
            ASTNode::Nama { base, .. } => {
                if let Some(symbol) = self.type_checker.env.lookup(base) {
                    self.instructions.push(Instruction::Bind {
                        name: symbol.name.clone(),
                        rust_type: symbol.rust_type_hint.clone(),
                        mutable: symbol.mutability.is_mutable,
                    });
                    
                    let line = format!("{}let {}: {};\n", 
                        self.indent_str(),
                        if symbol.mutability.is_mutable { format!("mut {}", symbol.name) } else { symbol.name.clone() },
                        symbol.rust_type_hint
                    );
                    self.rust_output.push_str(&line);
                }
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
                    } else if let ASTNode::IntLiteral { value, .. } = arg {
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
            ASTNode::Samasa { samasa_type, components, .. } => {
                let kind = samasa_from_str(&format!("{:?}", samasa_type)).unwrap_or(SamasaKind::Tatpurusha);
                let comps_refs: Vec<&str> = components.iter().map(|s| s.as_str()).collect();
                let resolved_node = resolve_samasa(&kind, &comps_refs);
                
                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str(&resolved_node.rust_repr);
                self.rust_output.push_str("\n");
                
                self.instructions.push(Instruction::Comment(format!("samasa: {}", resolved_node.resolved)));
            }
            ASTNode::IntLiteral { value, .. } => {
                self.rust_output.push_str(&value.to_string());
            }
            ASTNode::FloatLiteral { value, .. } => {
                self.rust_output.push_str(&value.to_string());
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
    use devvani_ast::{ASTNode, Vibhakti, Linga as AstLinga, Vacana as AstVacana, Lakara as AstLakara, SamasaType, Span};

    fn dummy_span() -> Span {
        Span { line: 0, col: 0, len: 0 }
    }

    #[test]
    fn test_empty_program() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::Program { statements: vec![], span: dummy_span() };
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
        let program = ASTNode::Program { statements: vec![node], span: dummy_span() };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.rust_source().contains("let Ramah"));
    }

    #[test]
    fn test_kriyacall_bytecode() {
        let mut codegen = Codegen::new(CodegenTarget::Bytecode);
        let nama = ASTNode::Nama {
            base: "Ramah".to_string(),
            vibhakti: Vibhakti::Prathama,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            span: dummy_span(),
        };
        let call = ASTNode::KriyaCall {
            karta: Some(Box::new(nama.clone())),
            kriya: "pathati".to_string(),
            karma: vec![],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        let program = ASTNode::Program { statements: vec![nama, call], span: dummy_span() };
        assert!(codegen.generate(&program).is_ok());
        
        let found = codegen.bytecode().iter().any(|ins| matches!(ins, Instruction::Call { verb, .. } if verb == "pathati"));
        assert!(found);
    }

    #[test]
    fn test_async_dhatu_def() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DhatuDef {
            name: "gacchati".to_string(),
            lakara: AstLakara::Lrt, // Async
            gana: devvani_ast::Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            body: vec![],
            span: dummy_span(),
        };
        let program = ASTNode::Program { statements: vec![node], span: dummy_span() };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.rust_source().contains("async fn gacchati"));
    }

    #[test]
    fn test_samasa_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::Samasa {
            samasa_type: SamasaType::Tatpurusha,
            parts: vec![],
            components: vec!["Rama".to_string(), "Putra".to_string()],
            resolved: "rama.putra".to_string(),
            span: dummy_span(),
        };
        let program = ASTNode::Program { statements: vec![node], span: dummy_span() };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.rust_source().contains("rama.putra"));
    }

    #[test]
    fn test_unknown_node_graceful() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::Comment { text: "hello".to_string(), span: dummy_span() };
        let program = ASTNode::Program { statements: vec![node], span: dummy_span() };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.bytecode().iter().any(|ins| matches!(ins, Instruction::Comment(_))));
    }
}
