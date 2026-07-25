use devvani_ast::ASTNode;
use devvani_ast::KarakaRole;
use devvani_typesystem::{
    lakara_from_str, lakara_to_scope,
    vaak::{MoveChecker, VaakOwnership},
    Lakara, TypeChecker, DevvaniType,
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
    RustSource, // emit .rs source
    Bytecode,   // emit simple bytecode (Vec<Instruction>)
}

// ── Simple bytecode instructions ─────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Bind {
        name: String,
        rust_type: String,
        mutable: bool,
    },
    Call {
        subject: String,
        verb: String,
        args: Vec<String>,
    },
    EnterScope {
        name: String,
        is_async: bool,
    },
    ExitScope,
    Return {
        value: String,
    },
    Comment(String),
    Output(String),
    Input(String),
    Yoga,
    Viyoga,
    Guna,
    Bhaga,
    Sama,
    Asama,
    Nyuuna,
    Adhika,
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
        let fatal_errors: Vec<_> = errors
            .iter()
            .filter(|e| !matches!(e, devvani_typesystem::TypeCheckError::AnavasthaDosha { .. }))
            .collect();
        if !fatal_errors.is_empty() {
            return Err(CodegenError::TypeCheckFailed(format!("{:?}", fatal_errors)));
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
                        errors.push(format!(
                            "Doṣa D030: '{}' — svāmitva-hāni (ownership moved, cannot use)",
                            naama
                        ));
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

    pub(crate) fn emit(&mut self, node: &ASTNode) -> Result<(), CodegenError> {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                self.emit_body(shareera)?;
            }
            ASTNode::Nama { base, .. } => {
                let display_name = if base.to_lowercase().ends_with("ah") {
                    &base[..base.len() - 2]
                } else {
                    base
                };

                if let Some(symbol) = self.type_checker.env.lookup(base) {
                    self.instructions.push(Instruction::Bind {
                        name: symbol.name.clone(),
                        rust_type: symbol.rust_type_hint.clone(),
                        mutable: symbol.mutability.is_mutable,
                    });

                    let line = format!(
                        "{}let {}: {};\n",
                        self.indent_str(),
                        if symbol.mutability.is_mutable {
                            format!("mut {}", display_name)
                        } else {
                            display_name.to_string()
                        },
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
            ASTNode::VaakNode {
                naama: _,
                mulya,
                karaka,
                is_mutable,
                span: _,
            } => {
                let kw = if *is_mutable { "let mut" } else { "let" };
                self.rust_output
                    .push_str(&format!("{} {} = ", self.indent_str(), kw));
                match karaka {
                    KarakaRole::Karana => self.rust_output.push_str("&"),
                    _ => {}
                }
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
            }
            ASTNode::VaakYogaNode {
                vama,
                dakshina,
                span: _,
            } => {
                self.emit(vama)?;
                self.rust_output.push_str(" + ");
                self.emit(dakshina)?;
                self.instructions.push(Instruction::Yoga);
            }
            ASTNode::AstiNode { naama, mulya } => {
                self.rust_output
                    .push_str(&format!("{}let {} = ", self.indent_str(), naama));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind {
                    name: naama.clone(),
                    rust_type: "auto".into(),
                    mutable: false,
                });
            }
            ASTNode::BhavatiNode { naama, mulya } => {
                self.rust_output
                    .push_str(&format!("{}let mut {} = ", self.indent_str(), naama));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind {
                    name: naama.clone(),
                    rust_type: "auto".into(),
                    mutable: true,
                });
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
                self.rust_output
                    .push_str(&format!("{}println!(\"{{:?}}\", ", self.indent_str()));
                self.emit(mulya)?;
                self.rust_output.push_str(");\n");
                self.instructions.push(Instruction::Output("stdout".into()));
            }
            ASTNode::PathatiNode { naama } => {
                self.rust_output.push_str(&format!(
                    "{}let mut {} = String::new(); std::io::stdin().read_line(&mut {}).unwrap();\n",
                    self.indent_str(),
                    naama,
                    naama
                ));
                self.instructions.push(Instruction::Input(naama.clone()));
            }
            ASTNode::YadiNode {
                sthiti,
                tarhi,
                anyatha,
            } => {
                self.rust_output
                    .push_str(&format!("{}if ", self.indent_str()));
                self.emit(sthiti)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                self.emit_body(&tarhi)?;
                self.indent -= 1;
                if let Some(else_body) = anyatha {
                    self.rust_output
                        .push_str(&format!("{}}} else {{\n", self.indent_str()));
                    self.indent += 1;
                    self.emit_body(else_body)?;
                    self.indent -= 1;
                }
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::YavatNode { sthiti, shareera } => {
                self.rust_output
                    .push_str(&format!("{}while ", self.indent_str()));
                self.emit(sthiti)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                self.emit_body(shareera)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::PunahNode { varam, shareera } => {
                self.rust_output
                    .push_str(&format!("{}for _ in 0..", self.indent_str()));
                self.emit(varam)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                self.emit_body(shareera)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::KriyaCall { karta, kriya, karma, .. } => {
                // Special-case handling for prakshepa-dhatu and apakarshana-dhatu
                if kriya == "prakshepa-dhatu" {
                    if let Some(karta_node) = karta {
                        // Emit just the variable name for karta (method receiver)
                        if let ASTNode::Nama { base, .. } = karta_node.as_ref() {
                            let display_name = if base.to_lowercase().ends_with("ah") {
                                &base[..base.len() - 2]
                            } else {
                                base
                            };
                            self.rust_output.push_str(display_name);
                        } else {
                            self.emit(karta_node)?;
                        }
                        self.rust_output.push_str(".push(");
                        if !karma.is_empty() {
                            self.emit(&karma[0])?;
                        }
                        self.rust_output.push_str(")");
                        self.rust_output.push_str(";\n");
                    }
                } else if kriya == "apakarshana-dhatu" {
                    if let Some(karta_node) = karta {
                        // Emit just the variable name for karta (method receiver)
                        if let ASTNode::Nama { base, .. } = karta_node.as_ref() {
                            let display_name = if base.to_lowercase().ends_with("ah") {
                                &base[..base.len() - 2]
                            } else {
                                base
                            };
                            self.rust_output.push_str(display_name);
                        } else {
                            self.emit(karta_node)?;
                        }
                        self.rust_output.push_str(".pop().unwrap()");
                    }
                } else {
                    self.instructions.push(Instruction::Call {
                        subject: String::new(),
                        verb: kriya.clone(),
                        args: Vec::new(),
                    });

                    self.rust_output.push_str(&self.indent_str());
                    self.rust_output.push_str(kriya);
                    self.rust_output.push_str("(");
                    for (i, arg) in karma.iter().enumerate() {
                        if i > 0 {
                            self.rust_output.push_str(", ");
                        }
                        self.emit(arg)?;
                    }
                    self.rust_output.push_str(");\n");
                }
            }
            ASTNode::AvartanaNode { call, .. } => self.emit(call.as_ref())?,
            ASTNode::PanktiNode { elements, .. } => {
                self.rust_output.push_str("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.rust_output.push_str(", ");
                    }
                    self.emit(elem)?;
                }
                self.rust_output.push_str("]");
            }
            ASTNode::AvaliNode { elements, .. } => {
                self.rust_output.push_str("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.rust_output.push_str(", ");
                    }
                    self.emit(elem)?;
                }
                self.rust_output.push_str("]");
            }
            ASTNode::VinyasaNode { target, index, .. } => {
                self.emit(target)?;
                self.rust_output.push_str("[");
                self.emit(index)?;
                self.rust_output.push_str("]");
            }
            ASTNode::KramashahNode {
                item_name,
                iterable,
                body,
                ..
            } => {
                self.rust_output
                    .push_str(&format!("{}for {} in ", self.indent_str(), item_name));
                self.emit(iterable)?;
                self.rust_output.push_str(".iter() {\n");
                self.indent += 1;
                self.emit_body(body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::DhatuDef {
                name,
                params,
                body,
                lakara,
                return_type,
                ..
            } => {
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

                let mut return_type_str = String::new();
                if let Some(rt) = return_type {
                    match rt.as_ref() {
                        ASTNode::PhalamType {
                            success_type,
                            error_type,
                            ..
                        } => {
                            let success_rust =
                                self.type_name_to_rust_type(success_type);
                            let error_rust =
                                self.type_name_to_rust_type(error_type);
                            return_type_str = format!(
                                " -> Result<{}, {}>",
                                success_rust, error_rust
                            );
                        }
                        other => {
                            return_type_str = format!(
                                " -> {}",
                                self.generate_to_string(other)?
                            );
                        }
                    }
                }

                let async_kw = if scope.is_async { "async " } else { "" };
                let line = format!(
                    "{}pub {}fn {}({}){return_type_str} {{\n",
                    self.indent_str(),
                    async_kw,
                    name,
                    rust_params.join(", ")
                );
                self.rust_output.push_str(&line);

                self.indent += 1;
                self.emit_body(body)?;
                self.indent -= 1;

                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str("}\n");

                self.instructions.push(Instruction::ExitScope);
            }
            ASTNode::DravyaDef { name, angas, .. } => {
                self.rust_output
                    .push_str(&format!("{}#[derive(Debug, Clone)]\n", self.indent_str()));
                if angas.is_empty() {
                    self.rust_output
                        .push_str(&format!("{}struct {} {{}}\n", self.indent_str(), name));
                } else {
                    self.rust_output
                        .push_str(&format!("{}struct {} {{\n", self.indent_str(), name));

                    self.indent += 1;
                    for (i, anga) in angas.iter().enumerate() {
                        if i > 0 {
                            self.rust_output.push_str(",\n");
                        }
                        let rust_ty = self.type_name_to_rust_type(&anga.type_name);
                        self.rust_output.push_str(&format!(
                            "{}{}: {}",
                            self.indent_str(),
                            anga.name,
                            rust_ty
                        ));
                    }
                    self.rust_output.push_str("\n");
                    self.indent -= 1;

                    self.rust_output
                        .push_str(&format!("{}}}\n", self.indent_str()));
                }
            }
            ASTNode::Samasa {
                components,
                resolved,
                ..
            } => {
                self.rust_output.push_str(&self.indent_str());
                self.rust_output.push_str(&resolved);
                self.rust_output.push_str("\n");

                self.instructions.push(Instruction::Comment(format!(
                    "samasa: {}",
                    components.join(".")
                )));
            }
            ASTNode::SamavayaNode { target, anga_name, .. } => {
                self.emit(target)?;
                self.rust_output.push_str(".");
                self.rust_output.push_str(anga_name);
            }
            ASTNode::NirmanaNode {
                dravya_name,
                values,
                ..
            } => {
                let sym = match self.type_checker.env.lookup(dravya_name) {
                    Some(s) => s,
                    None => {
                        return Err(CodegenError::UnsupportedNode(format!(
                            "Unknown dravya: {}",
                            dravya_name
                        )));
                    }
                };
                let angas = match &sym.devvani_type {
                    DevvaniType::Dravya(_name, angas) => angas.clone(),
                    _ => {
                        return Err(CodegenError::UnsupportedNode(format!(
                            "{} is not a dravya",
                            dravya_name
                        )));
                    }
                };

                if angas.is_empty() {
                    self.rust_output
                        .push_str(&format!("{}{} {{}}", self.indent_str(), dravya_name));
                } else {
                    self.rust_output
                        .push_str(&format!("{}{} {{ ", self.indent_str(), dravya_name));
                    for (i, (field_name, _)) in angas.iter().enumerate() {
                        if i > 0 {
                            self.rust_output.push_str(", ");
                        }
                        self.rust_output.push_str(&format!("{}: ", field_name));
                        self.emit(&values[i])?;
                    }
                    self.rust_output.push_str(" }");
                }
            }
            ASTNode::PhalamType {
                success_type,
                error_type,
                ..
            } => {
                let success_rust = self.type_name_to_rust_type(success_type);
                let error_rust = self.type_name_to_rust_type(error_type);
                self.rust_output
                    .push_str(&format!("Result<{}, {}>", success_rust, error_rust));
            }
            ASTNode::ArogyaNode { value, span: _ } => {
                self.rust_output.push_str("Ok(");
                self.emit(value)?;
                self.rust_output.push_str(")");
            }
            ASTNode::DoshaNode { value, span: _ } => {
                self.rust_output.push_str("Err(");
                self.emit(value)?;
                self.rust_output.push_str(")");
            }
            ASTNode::NidanaNode {
                target,
                arogya_bind,
                arogya_body,
                dosha_bind,
                dosha_body,
                span: _,
            } => {
                self.rust_output.push_str("match ");
                self.emit(target)?;
                self.rust_output.push_str(" {\n");
                self.indent += 1;
                self.rust_output.push_str(&format!(
                    "{}Ok({}) => {{\n",
                    self.indent_str(),
                    arogya_bind
                ));
                self.indent += 1;
                self.emit_body(arogya_body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}},\n", self.indent_str()));
                self.rust_output.push_str(&format!(
                    "{}Err({}) => {{\n",
                    self.indent_str(),
                    dosha_bind
                ));
                self.indent += 1;
                self.emit_body(dosha_body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::SamprapatiNode { expr, span: _ } => {
                self.emit(expr)?;
                self.rust_output.push_str("?");
            }
            _ => {
                let msg = format!("Unhandled node: {:?}", node);
                self.instructions.push(Instruction::Comment(msg.clone()));
                self.rust_output
                    .push_str(&format!("{}// {}\n", self.indent_str(), msg));
            }
        }
        Ok(())
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn emit_body(&mut self, body: &[ASTNode]) -> Result<(), CodegenError> {
        for (i, stmt) in body.iter().enumerate() {
            self.emit(stmt)?;
            if i < body.len() - 1 && !self.rust_output.ends_with(";\n") {
                self.rust_output.push_str(";\n");
            }
        }
        Ok(())
    }

    fn generate_to_string(&mut self, node: &ASTNode) -> Result<String, CodegenError> {
        let old_output = self.rust_output.clone();
        self.rust_output = String::new();
        self.emit(node)?;
        let result = self.rust_output.clone();
        self.rust_output = old_output;
        Ok(result)
    }

    fn type_name_to_rust_type(&self, type_name: &str) -> String {
        if let Some(ty) = self.type_checker.env.lookup_type(type_name) {
            match ty {
                DevvaniType::Vaak => "String".to_string(),
                DevvaniType::VaakBorrow => "&str".to_string(),
                DevvaniType::Subject(ref s) => match s.as_str() {
                    "Purnaank" => "i64".to_string(),
                    "Dashaamsha" => "f64".to_string(),
                    _ => s.clone(),
                },
                DevvaniType::Dravya(ref name, _) => name.clone(),
                DevvaniType::Pankti(ref elem, len) => {
                    let elem_ty = self.type_name_to_rust_type_by_type(elem);
                    format!("[{}; {}]", elem_ty, len)
                }
                DevvaniType::Avali(ref elem) => {
                    let elem_ty = self.type_name_to_rust_type_by_type(elem);
                    format!("Vec<{}>", elem_ty)
                }
                _ => type_name.to_string(),
            }
        } else {
            match type_name {
                "sankhya" | "purnaank" => "i64".to_string(),
                "dashaamsha" => "f64".to_string(),
                "vaak" => "String".to_string(),
                _ => type_name.to_string(),
            }
        }
    }

    fn type_name_to_rust_type_by_type(&self, ty: &DevvaniType) -> String {
        match ty {
            DevvaniType::Vaak => "String".to_string(),
            DevvaniType::VaakBorrow => "&str".to_string(),
            DevvaniType::Subject(ref s) => match s.as_str() {
                "Purnaank" => "i64".to_string(),
                "Dashaamsha" => "f64".to_string(),
                _ => s.clone(),
            },
            DevvaniType::Dravya(ref name, _) => name.clone(),
            DevvaniType::Pankti(ref elem, len) => {
                let elem_ty = self.type_name_to_rust_type_by_type(elem);
                format!("[{}; {}]", elem_ty, len)
            }
            DevvaniType::Avali(ref elem) => {
                let elem_ty = self.type_name_to_rust_type_by_type(elem);
                format!("Vec<{}>", elem_ty)
            }
            _ => "auto".to_string(),
        }
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
    use devvani_ast::{ASTNode, AngaField, Linga as AstLinga, Span, Vacana as AstVacana, Vibhakti};

    fn dummy_span() -> Span {
        Span {
            line: 0,
            col: 0,
            len: 0,
        }
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
        let program = ASTNode::KaryakramNode {
            shareera: vec![node],
        };
        assert!(codegen.generate(&program).is_ok());
        assert!(codegen.rust_source().contains("let Ram"));
    }

    #[test]
    fn test_pankti_literal_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let elements = vec![
            ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 2,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 3,
                span: dummy_span(),
            },
        ];
        let node = ASTNode::PanktiNode {
            elements,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_empty_pankti_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::PanktiNode {
            elements: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "[]");
    }

    #[test]
    fn test_nested_pankti_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let inner_elements = vec![
            ASTNode::PurnaankLiteral {
                value: 4,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            },
        ];
        let elements = vec![
            ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            },
            ASTNode::PanktiNode {
                elements: inner_elements,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 6,
                span: dummy_span(),
            },
        ];
        let node = ASTNode::PanktiNode {
            elements,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "[1, [4, 5], 6]");
    }

    #[test]
    fn test_vinyasa_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let index = ASTNode::PurnaankLiteral {
            value: 0,
            span: dummy_span(),
        };
        let target = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 2,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 3,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let node = ASTNode::VinyasaNode {
            target: Box::new(target),
            index: Box::new(index),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("[1, 2, 3][0]"));
    }

    #[test]
    fn test_vinyasa_with_expression_index_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let index = ASTNode::PurnaankLiteral {
            value: 1,
            span: dummy_span(),
        };
        let target = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 10,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 20,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 30,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let node = ASTNode::VinyasaNode {
            target: Box::new(target),
            index: Box::new(index),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("[10, 20, 30][1]"));
    }

    #[test]
    fn test_pankti_of_kriya_call_results_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let kriya_call = ASTNode::KriyaCall {
            karta: None,
            kriya: "add".to_string(),
            karma: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 2,
                    span: dummy_span(),
                },
            ],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        let elements = vec![kriya_call];
        let node = ASTNode::PanktiNode {
            elements,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("add(1, 2)"));
    }

    #[test]
    fn test_kramashah_basic_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let body_stmt = ASTNode::KriyaCall {
            karta: None,
            kriya: "println".to_string(),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        let iterable = ASTNode::PanktiNode {
            elements: vec![ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }],
            span: dummy_span(),
        };
        let node = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(iterable),
            body: vec![body_stmt],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("for x in [1].iter() {"));
        assert!(output.contains("println(1);"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_kramashah_empty_body_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let iterable = ASTNode::PanktiNode {
            elements: vec![ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }],
            span: dummy_span(),
        };
        let node = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(iterable),
            body: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("for x in [1].iter() {"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_kramashah_over_pankti_literal_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let body_stmt = ASTNode::KriyaCall {
            karta: None,
            kriya: "println".to_string(),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        let iterable = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 2,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 3,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let node = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(iterable),
            body: vec![body_stmt],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("[1, 2, 3].iter()"));
    }

    #[test]
    fn test_avali_literal_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let elements = vec![
            ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 2,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 3,
                span: dummy_span(),
            },
        ];
        let node = ASTNode::AvaliNode {
            elements,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "vec![1, 2, 3]");
    }

    #[test]
    fn test_avali_literal_empty_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::AvaliNode {
            elements: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "vec![]");
    }

    #[test]
    fn test_avali_nested_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let inner_elements = vec![
            ASTNode::PurnaankLiteral {
                value: 4,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            },
        ];
        let elements = vec![
            ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            },
            ASTNode::AvaliNode {
                elements: inner_elements,
                span: dummy_span(),
            },
            ASTNode::PurnaankLiteral {
                value: 6,
                span: dummy_span(),
            },
        ];
        let node = ASTNode::AvaliNode {
            elements,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "vec![1, vec![4, 5], 6]");
    }

    #[test]
    fn test_prakshepa_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "myavali".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: devvani_ast::Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: dummy_span(),
            })),
            kriya: String::from("prakshepa-dhatu"),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 10,
                span: dummy_span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("myavali.push(10)"));
    }

    #[test]
    fn test_apakarshana_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "myavali".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: devvani_ast::Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: dummy_span(),
            })),
            kriya: String::from("apakarshana-dhatu"),
            karma: vec![],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("myavali.pop().unwrap()"));
    }

    #[test]
    fn test_nirmana_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.env.define(
            "manushya",
            DevvaniType::Dravya(
                "manushya".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                    ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                ],
            ),
        );
        let node = ASTNode::NirmanaNode {
            dravya_name: "manushya".to_string(),
            values: vec![
                ASTNode::VaakLiteral {
                    value: "raamah".to_string(),
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 25,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "manushya { naama: \"raamah\", sankhya: 25 }");
    }

    #[test]
    fn test_nirmana_field_ordering() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.env.define(
            "manushya",
            DevvaniType::Dravya(
                "manushya".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                    ("sankhya1".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                    ("sankhya2".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                ],
            ),
        );
        let node = ASTNode::NirmanaNode {
            dravya_name: "manushya".to_string(),
            values: vec![
                ASTNode::VaakLiteral {
                    value: "raamah".to_string(),
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 25,
                    span: dummy_span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 180,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        let naama_pos = output.find("naama:").unwrap();
        let sankhya1_pos = output.find("sankhya1:").unwrap();
        let sankhya2_pos = output.find("sankhya2:").unwrap();
        assert!(naama_pos < sankhya1_pos);
        assert!(sankhya1_pos < sankhya2_pos);
    }

    #[test]
    fn test_nirmana_empty_dravya_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.env.define(
            "shunya",
            DevvaniType::Dravya("shunya".to_string(), vec![]),
        );
        let node = ASTNode::NirmanaNode {
            dravya_name: "shunya".to_string(),
            values: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "shunya {}");
    }

    #[test]
    fn test_nirmana_unknown_dravya_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::NirmanaNode {
            dravya_name: "unknown".to_string(),
            values: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_err());
    }

    #[test]
    fn test_dravya_def_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let angas = vec![
            AngaField {
                name: "naama".to_string(),
                type_name: "vaak".to_string(),
                span: dummy_span(),
            },
            AngaField {
                name: "sankhya".to_string(),
                type_name: "sankhya".to_string(),
                span: dummy_span(),
            },
        ];
        let node = ASTNode::DravyaDef {
            name: "manushya".to_string(),
            angas,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("#[derive(Debug, Clone)]"));
        assert!(output.contains("struct manushya {"));
        assert!(output.contains("naama: String,"));
        assert!(output.contains("sankhya: i64"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_dravya_def_empty_fields_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DravyaDef {
            name: "shunya".to_string(),
            angas: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source().trim();
        assert!(output.contains("struct shunya {}"));
    }

    #[test]
    fn test_samavaya_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let target = ASTNode::PurnaankLiteral {
            value: 1,
            span: dummy_span(),
        };
        let node = ASTNode::SamavayaNode {
            target: Box::new(target),
            anga_name: "naama".to_string(),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "1.naama");
    }

    #[test]
    fn test_samavaya_chained_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let inner = ASTNode::SamavayaNode {
            target: Box::new(ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }),
            anga_name: "a".to_string(),
            span: dummy_span(),
        };
        let outer = ASTNode::SamavayaNode {
            target: Box::new(inner),
            anga_name: "b".to_string(),
            span: dummy_span(),
        };
        assert!(codegen.emit(&outer).is_ok());
        assert_eq!(codegen.rust_source().trim(), "1.a.b");
    }

    #[test]
    fn test_dravya_with_dravya_typed_field_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let angas = vec![
            AngaField {
                name: "inner".to_string(),
                type_name: "outer".to_string(),
                span: dummy_span(),
            },
        ];
        let node = ASTNode::DravyaDef {
            name: "wrapper".to_string(),
            angas,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("inner: outer"));
        assert!(output.contains("struct wrapper {"));
    }

    #[test]
    fn test_dhatu_def_with_phalam_return_type_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DhatuDef {
            name: "bhojan_dhatu".to_string(),
            lakara: devvani_ast::Lakara::Lat,
            gana: devvani_ast::Gana::Bhvadi,
            linga: devvani_ast::Linga::Pullinga,
            vacana: devvani_ast::Vacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            return_type: Some(Box::new(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "vaak".to_string(),
                span: dummy_span(),
            })),
            body: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("-> Result<i64, String>"));
    }

    #[test]
    fn test_dhatu_def_phalam_wrapping_dravya_success_type() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.env.define(
            "manushya",
            DevvaniType::Dravya(
                "manushya".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                    ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                ],
            ),
        );
        codegen.type_checker.env.define(
            "roga",
            DevvaniType::Dravya(
                "roga".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                ],
            ),
        );
        let node = ASTNode::DhatuDef {
            name: "vyayama_dhatu".to_string(),
            lakara: devvani_ast::Lakara::Lat,
            gana: devvani_ast::Gana::Bhvadi,
            linga: devvani_ast::Linga::Pullinga,
            vacana: devvani_ast::Vacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            return_type: Some(Box::new(ASTNode::PhalamType {
                success_type: "manushya".to_string(),
                error_type: "roga".to_string(),
                span: dummy_span(),
            })),
            body: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("-> Result<manushya, roga>"));
    }

    #[test]
    fn test_arogya_node_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::ArogyaNode {
            value: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "Ok(42)");
    }

    #[test]
    fn test_dosha_node_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DoshaNode {
            value: Box::new(ASTNode::VaakLiteral {
                value: "error".to_string(),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "Err(\"error\")");
    }

    #[test]
    fn test_nidana_node_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let target = ASTNode::PurnaankLiteral {
            value: 1,
            span: dummy_span(),
        };
        let arogya_body = vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::PurnaankLiteral {
                value: 1,
                span: dummy_span(),
            }),
        }];
        let dosha_body = vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "fail".to_string(),
                span: dummy_span(),
            }),
        }];
        let node = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "s".to_string(),
            arogya_body,
            dosha_bind: "e".to_string(),
            dosha_body,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("match 1"));
        assert!(output.contains("Ok(s) => {"));
        assert!(output.contains("Err(e) => {"));
    }

    #[test]
    fn test_nidana_nested_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let target = ASTNode::PurnaankLiteral {
            value: 1,
            span: dummy_span(),
        };
        let inner_nidana = ASTNode::NidanaNode {
            target: Box::new(ASTNode::PurnaankLiteral {
                value: 2,
                span: dummy_span(),
            }),
            arogya_bind: "inner_s".to_string(),
            arogya_body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::PurnaankLiteral {
                    value: 3,
                    span: dummy_span(),
                }),
            }],
            dosha_bind: "inner_e".to_string(),
            dosha_body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "inner_fail".to_string(),
                    span: dummy_span(),
                }),
            }],
            span: dummy_span(),
        };
        let arogya_body = vec![inner_nidana];
        let dosha_body = vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "outer_fail".to_string(),
                span: dummy_span(),
            }),
        }];
        let node = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "s".to_string(),
            arogya_body,
            dosha_bind: "e".to_string(),
            dosha_body,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("Ok(s) => {"));
        assert!(output.contains("Ok(inner_s) => {"));
        assert!(output.contains("Err(inner_e) => {"));
    }

    #[test]
    fn test_samprapati_node_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::SamprapatiNode {
            expr: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "42?");
    }

    #[test]
    fn test_samprapati_on_kriya_call_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let kriya_call = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "myavali".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: devvani_ast::Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: dummy_span(),
            })),
            kriya: String::from("prakshepa-dhatu"),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 10,
                span: dummy_span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        let node = ASTNode::SamprapatiNode {
            expr: Box::new(kriya_call),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("myavali.push(10)"));
        assert!(output.contains("?"));
    }

    #[test]
    fn test_round_trip_phalam_arogya_dosha_samprapti() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.env.define(
            "manushya",
            DevvaniType::Dravya(
                "manushya".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                    ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                ],
            ),
        );
        codegen.type_checker.env.define(
            "roga",
            DevvaniType::Dravya(
                "roga".to_string(),
                vec![
                    ("naama".to_string(), DevvaniType::Vaak),
                ],
            ),
        );
        let dhatu = ASTNode::DhatuDef {
            name: "bhavana_dhatu".to_string(),
            lakara: devvani_ast::Lakara::Lat,
            gana: devvani_ast::Gana::Bhvadi,
            linga: devvani_ast::Linga::Pullinga,
            vacana: devvani_ast::Vacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            return_type: Some(Box::new(ASTNode::PhalamType {
                success_type: "manushya".to_string(),
                error_type: "roga".to_string(),
                span: dummy_span(),
            })),
            body: vec![
                ASTNode::ArogyaNode {
                    value: Box::new(ASTNode::NirmanaNode {
                        dravya_name: "manushya".to_string(),
                        values: vec![
                            ASTNode::VaakLiteral {
                                value: "rahul".to_string(),
                                span: dummy_span(),
                            },
                            ASTNode::PurnaankLiteral {
                                value: 30,
                                span: dummy_span(),
                            },
                        ],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                ASTNode::NidanaNode {
                    target: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                    arogya_bind: "p".to_string(),
                    arogya_body: vec![ASTNode::ArogyaNode {
                        value: Box::new(ASTNode::PurnaankLiteral {
                            value: 10,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    dosha_bind: "q".to_string(),
                    dosha_body: vec![ASTNode::SamprapatiNode {
                        expr: Box::new(ASTNode::PurnaankLiteral {
                            value: 5,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        assert!(codegen.emit(&dhatu).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("-> Result<manushya, roga>"));
        assert!(output.contains("manushya { naama: \"rahul\", sankhya: 30 "));
        assert!(output.contains("Ok("));
        assert!(output.contains("match 1"));
        assert!(output.contains("Ok(p) => {"));
        assert!(output.contains("Err(q) => {"));
        assert!(output.contains("5?"));
    }
}
