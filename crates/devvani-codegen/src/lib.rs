use std::collections::{HashMap, HashSet};
use devvani_ast::ASTNode;
use devvani_ast::KarakaRole;
use devvani_ast::KarakaParam;
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
    collected_dravya_instantiations: Vec<(String, String, Vec<(String, DevvaniType)>)>,
    collected_dhatu_instantiations: Vec<(String, String, HashMap<String, DevvaniType>, DevvaniType)>,
    current_dhatu_context: Option<(String, String)>,
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
            collected_dravya_instantiations: Vec::new(),
            collected_dhatu_instantiations: Vec::new(),
            current_dhatu_context: None,
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

        // 3. Collect generic Dravya instantiations for monomorphization
        self.collect_dravya_instantiations(node);

        // 3b. Collect generic Dhātu instantiations for monomorphization
        self.collect_dhatu_instantiations(node);

        // 4. Then call emit(node)
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
            ASTNode::KriyaCall { karta, kriya, karma, karana, sampradana, apadan, adhikarana, .. } => {
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

                    let emit_name = if let Some((current_name, current_mangled)) =
                        self.current_dhatu_context.as_ref()
                    {
                        if kriya == current_name {
                            current_mangled.clone()
                        } else {
                            self.mangled_name_for_dhatu_call(
                                kriya,
                                karta,
                                karma,
                                karana,
                                sampradana,
                                apadan,
                                adhikarana,
                            )
                            .unwrap_or_else(|| kriya.clone())
                        }
                    } else {
                        self.mangled_name_for_dhatu_call(
                            kriya,
                            karta,
                            karma,
                            karana,
                            sampradana,
                            apadan,
                            adhikarana,
                        )
                        .unwrap_or_else(|| kriya.clone())
                    };

                    self.rust_output.push_str(&self.indent_str());
                    self.rust_output.push_str(&emit_name);
                    self.rust_output.push_str("(");
                     for (i, arg) in karma.iter().enumerate() {
                         if i > 0 {
                             self.rust_output.push_str(", ");
                         }
                         if let ASTNode::SandarbhaNode { .. } = arg {
                             self.emit(arg)?;
                         } else if let Some(params) = self.type_checker.function_params().get(kriya) {
                             if let Some(param) = params.get(i) {
                                 if param.is_borrowed {
                                     if param.is_mutable_borrow {
                                         self.rust_output.push_str("&mut ");
                                     } else {
                                         self.rust_output.push_str("&");
                                     }
                                 }
                             }
                             self.emit(arg)?;
                         } else {
                             self.emit(arg)?;
                         }
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
                generic_params,
                ..
            } => {
                let has_samanya = !generic_params.is_empty();

                if !has_samanya {
                    let ts_lakara = lakara_from_str(&format!("{:?}", lakara)).unwrap_or(Lakara::Lat);
                    let scope = lakara_to_scope(&ts_lakara);

                    self.instructions.push(Instruction::EnterScope {
                        name: name.clone(),
                        is_async: scope.is_async,
                    });

                    let mut rust_params = Vec::new();
                    for param in params {
                        let type_str = if param.is_borrowed {
                            if param.is_mutable_borrow {
                                "&mut i64".to_string()
                            } else {
                                "&i64".to_string()
                            }
                        } else {
                            "i64".to_string()
                        };
                        rust_params.push(format!("{}: {}", param.name, type_str));
                        self.instructions.push(Instruction::Bind {
                            name: param.name.clone(),
                            rust_type: type_str.clone(),
                            mutable: param.is_mutable_borrow,
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
                } else {
                    let instantiations: Vec<_> = self
                        .collected_dhatu_instantiations
                        .iter()
                        .filter(|(dhatu_name, ..)| dhatu_name == name)
                        .cloned()
                        .collect();
                    let mut emitted_names = HashSet::new();
                    for (dhatu_name, mangled, inference, _concrete_return) in instantiations {
                        if dhatu_name == *name {
                            if emitted_names.insert(mangled.clone()) {
                                let prev_context = self.current_dhatu_context.take();
                                self.current_dhatu_context = Some((name.clone(), mangled.clone()));
                                self.emit_monomorphized_dhatu(
                                    &mangled,
                                    params,
                                    &inference,
                                    return_type.as_deref(),
                                    body,
                                );
                                self.current_dhatu_context = prev_context;
                            }
                        }
                    }
                }
            }
            ASTNode::DravyaDef { name, angas, generic_params, .. } => {
                let has_samanya = !generic_params.is_empty();

                if !has_samanya {
                    self.rust_output
                        .push_str(&format!("{}#[derive(Debug, Clone)]\n", self.indent_str()));
                    if angas.is_empty() {
                        self.rust_output
                            .push_str(&format!("{}struct {} {{}};\n", self.indent_str(), name));
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
                            .push_str(&format!("{}}};\n", self.indent_str()));
                    }
                } else {
                    let instantiations: Vec<_> = self.collected_dravya_instantiations.clone();
                    let mut emitted_names = HashSet::new();
                    for (dravya_name, mangled, resolved_angas) in instantiations {
                        if dravya_name == *name {
                            if emitted_names.insert(mangled.clone()) {
                                self.emit_monomorphized_dravya(&mangled, &resolved_angas);
                            }
                        }
                    }
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

                let has_samanya = angas.iter().any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                let emit_name = if has_samanya {
                    if let Some((generic_params, inference)) = self.infer_generic_concrete_types(dravya_name, values) {
                        self.mangled_generic_name(dravya_name, &generic_params, &inference)
                    } else {
                        dravya_name.clone()
                    }
                } else {
                    dravya_name.clone()
                };

                if angas.is_empty() {
                    self.rust_output
                        .push_str(&format!("{}{} {{}}", self.indent_str(), emit_name));
                } else {
                    self.rust_output
                        .push_str(&format!("{}{} {{ ", self.indent_str(), emit_name));
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
            ASTNode::SandarbhaNode {
                target,
                is_mutable,
                ..
            } => {
                if *is_mutable {
                    self.rust_output.push_str("&mut ");
                } else {
                    self.rust_output.push_str("&");
                }
                self.emit(target)?;
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
                    "Vaak" => "String".to_string(),
                    "VaakBorrow" => "&str".to_string(),
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
                "Vaak" => "String".to_string(),
                "VaakBorrow" => "&str".to_string(),
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

    fn mangled_generic_name(&self, base_name: &str, generic_params: &[String], inference: &HashMap<String, DevvaniType>) -> String {
        if generic_params.is_empty() {
            return base_name.to_string();
        }
        let suffix: Vec<String> = generic_params
            .iter()
            .filter_map(|param| inference.get(param).map(|ty| self.type_name_to_rust_type_by_type(ty)))
            .collect();
        if suffix.is_empty() {
            base_name.to_string()
        } else {
            format!("{}__{}", base_name, suffix.join("__"))
        }
    }

    fn substitute_samanya_in_type(
        ty: DevvaniType,
        inference: &HashMap<String, DevvaniType>,
    ) -> DevvaniType {
        match ty {
            DevvaniType::Samanya(name) => inference
                .get(&name)
                .cloned()
                .unwrap_or(DevvaniType::Samanya(name)),
            DevvaniType::Dravya(name, angas) => DevvaniType::Dravya(
                name,
                angas
                    .into_iter()
                    .map(|(n, t)| (n, Self::substitute_samanya_in_type(t, inference)))
                    .collect(),
            ),
            DevvaniType::Phalam(success, error) => DevvaniType::Phalam(
                Box::new(Self::substitute_samanya_in_type(*success, inference)),
                Box::new(Self::substitute_samanya_in_type(*error, inference)),
            ),
            DevvaniType::Pankti(elem, len) => DevvaniType::Pankti(
                Box::new(Self::substitute_samanya_in_type(*elem, inference)),
                len,
            ),
            DevvaniType::Avali(elem) => DevvaniType::Avali(Box::new(Self::substitute_samanya_in_type(
                *elem, inference,
            ))),
            DevvaniType::Sandarbha(inner, mutability) => DevvaniType::Sandarbha(
                Box::new(Self::substitute_samanya_in_type(*inner, inference)),
                mutability,
            ),
            other => other,
        }
    }

    fn dhatu_call_mono_info(
        &mut self,
        kriya: &str,
        karta: &Option<Box<ASTNode>>,
        karma: &[ASTNode],
        karana: &Option<Box<ASTNode>>,
        sampradana: &Option<Box<ASTNode>>,
        apadan: &Option<Box<ASTNode>>,
        adhikarana: &Option<Box<ASTNode>>,
    ) -> Option<(String, HashMap<String, DevvaniType>, DevvaniType)> {
        let generic_params = match self.type_checker.function_generic_params().get(kriya) {
            Some(p) if !p.is_empty() => p.clone(),
            _ => return None,
        };

        let mut all_args = Vec::new();
        if let Some(k) = karta.as_ref() {
            all_args.push(k.as_ref());
        }
        for arg in karma.iter() {
            all_args.push(arg);
        }
        if let Some(k) = karana.as_ref() {
            all_args.push(k.as_ref());
        }
        if let Some(k) = sampradana.as_ref() {
            all_args.push(k.as_ref());
        }
        if let Some(k) = apadan.as_ref() {
            all_args.push(k.as_ref());
        }
        if let Some(k) = adhikarana.as_ref() {
            all_args.push(k.as_ref());
        }

        let arg_types: Vec<DevvaniType> = all_args
            .iter()
            .map(|a| self.type_checker.check(a))
            .collect();

        let generic_params_set: HashSet<String> = generic_params.iter().cloned().collect();
        let mut inference: HashMap<String, DevvaniType> = HashMap::new();

        if let Some(params) = self.type_checker.function_params().get(kriya) {
            for (i, param) in params.iter().enumerate() {
                if i >= arg_types.len() {
                    break;
                }
                if param.is_borrowed {
                    continue;
                }
                let param_type_name = &param.type_name;
                if generic_params_set.contains(param_type_name.as_str()) {
                    if let Some(previous_ty) = inference.get(param_type_name) {
                        if *previous_ty != arg_types[i] {
                            continue;
                        }
                    } else {
                        inference.insert(param_type_name.clone(), arg_types[i].clone());
                    }
                }
            }
        }

        let return_type = if let Some(declared_return) =
            self.type_checker.function_return_types().get(kriya)
        {
            Self::substitute_samanya_in_type(declared_return.clone(), &inference)
        } else {
            DevvaniType::Subject(kriya.to_string())
        };

        for (_, ty) in inference.iter() {
            if matches!(ty, DevvaniType::Samanya(_)) {
                return None;
            }
        }

        let mangled = self.mangled_generic_name(kriya, &generic_params, &inference);
        Some((mangled, inference, return_type))
    }

    fn mangled_name_for_dhatu_call(
        &mut self,
        kriya: &str,
        karta: &Option<Box<ASTNode>>,
        karma: &[ASTNode],
        karana: &Option<Box<ASTNode>>,
        sampradana: &Option<Box<ASTNode>>,
        apadan: &Option<Box<ASTNode>>,
        adhikarana: &Option<Box<ASTNode>>,
    ) -> Option<String> {
        self.dhatu_call_mono_info(kriya, karta, karma, karana, sampradana, apadan, adhikarana)
            .map(|(mangled, _, _)| mangled)
    }

    fn emit_monomorphized_dravya(
        &mut self,
        mangled_name: &str,
        angas: &[(String, DevvaniType)],
    ) {
        self.rust_output
            .push_str(&format!("{}#[derive(Debug, Clone)]\n", self.indent_str()));
        if angas.is_empty() {
            self.rust_output
                .push_str(&format!("{}struct {} {{}};\n", self.indent_str(), mangled_name));
        } else {
            self.rust_output
                .push_str(&format!("{}struct {} {{\n", self.indent_str(), mangled_name));
            self.indent += 1;
            for (i, (field_name, field_ty)) in angas.iter().enumerate() {
                if i > 0 {
                    self.rust_output.push_str(",\n");
                }
                let rust_ty = self.type_name_to_rust_type_by_type(field_ty);
                self.rust_output.push_str(&format!(
                    "{}{}: {}",
                    self.indent_str(),
                    field_name,
                    rust_ty
                ));
            }
            self.rust_output.push_str("\n");
            self.indent -= 1;
            self.rust_output
                .push_str(&format!("{}}};\n", self.indent_str()));
        }
    }

    fn infer_generic_concrete_types(
        &mut self,
        dravya_name: &str,
        values: &[ASTNode],
    ) -> Option<(Vec<String>, HashMap<String, DevvaniType>)> {
        let angas = match self.type_checker.env.lookup(dravya_name) {
            Some(sym) => match &sym.devvani_type {
                DevvaniType::Dravya(_, angas) => angas.clone(),
                _ => return None,
            },
            None => return None,
        };
        let generic_params: Vec<String> = angas.iter()
            .filter_map(|(_, ty)| if let DevvaniType::Samanya(p) = ty { Some(p.clone()) } else { None })
            .collect();
        if generic_params.is_empty() {
            return None;
        }
        let mut inference: HashMap<String, DevvaniType> = HashMap::new();
        for (i, (_anga_name, expected_ty)) in angas.iter().enumerate() {
            if let DevvaniType::Samanya(param_name) = expected_ty {
                if inference.contains_key(param_name) {
                    continue;
                }
                let found_ty = self.type_checker.check(&values[i]);
                if found_ty != DevvaniType::Unknown {
                    inference.insert(param_name.clone(), found_ty);
                }
            }
        }
        Some((generic_params, inference))
    }

    fn emit_monomorphized_dhatu(
        &mut self,
        mangled_name: &str,
        params: &[KarakaParam],
        inference: &HashMap<String, DevvaniType>,
        return_type: Option<&ASTNode>,
        body: &[ASTNode],
    ) {
        let mut rust_params = Vec::new();
        for param in params {
            let concrete_ty = if inference.contains_key(&param.type_name) {
                self.type_name_to_rust_type_by_type(
                    inference.get(&param.type_name).unwrap(),
                )
            } else {
                self.type_name_to_rust_type(&param.type_name)
            };
            let type_str = if param.is_borrowed {
                if param.is_mutable_borrow {
                    format!("&mut {}", concrete_ty)
                } else {
                    format!("&{}", concrete_ty)
                }
            } else {
                concrete_ty
            };
            rust_params.push(format!("{}: {}", param.name, type_str));
        }

        let mut return_type_str = String::new();
        if let Some(rt) = return_type {
            match rt {
                ASTNode::PhalamType {
                    success_type,
                    error_type,
                    ..
                } => {
                    let success_rust = if inference.contains_key(success_type) {
                        self.type_name_to_rust_type_by_type(
                            inference.get(success_type).unwrap(),
                        )
                    } else {
                        self.type_name_to_rust_type(success_type)
                    };
                    let error_rust = if inference.contains_key(error_type) {
                        self.type_name_to_rust_type_by_type(
                            inference.get(error_type).unwrap(),
                        )
                    } else {
                        self.type_name_to_rust_type(error_type)
                    };
                    return_type_str = format!(
                        " -> Result<{}, {}>",
                        success_rust, error_rust
                    );
                }
                ASTNode::Nama { base, .. } => {
                    let rust_ty = if inference.contains_key(base) {
                        self.type_name_to_rust_type_by_type(
                            inference.get(base).unwrap(),
                        )
                    } else {
                        self.type_name_to_rust_type(base)
                    };
                    return_type_str = format!(" -> {}", rust_ty);
                }
                other => {
                    return_type_str = format!(
                        " -> {}",
                        self.generate_to_string(other).unwrap_or_default()
                    );
                }
            }
        }

        let line = format!(
            "{}pub fn {}({}){return_type_str} {{\n",
            self.indent_str(),
            mangled_name,
            rust_params.join(", ")
        );
        self.rust_output.push_str(&line);

        self.indent += 1;
        self.emit_body(body).ok();
        self.indent -= 1;

        self.rust_output.push_str(&self.indent_str());
        self.rust_output.push_str("};\n");
    }

    fn collect_dhatu_instantiations(&mut self, node: &ASTNode) {
        self.collected_dhatu_instantiations.clear();
        let mut set = Vec::new();
        self.walk_for_dhatu_instantiations(node, &mut set);
        self.collected_dhatu_instantiations = set;
    }

    fn walk_for_dhatu_instantiations(
        &mut self,
        node: &ASTNode,
        set: &mut Vec<(String, String, HashMap<String, DevvaniType>, DevvaniType)>,
    ) {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::DhatuDef { body, return_type, .. } => {
                for stmt in body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
                if let Some(rt) = return_type {
                    self.walk_for_dhatu_instantiations(rt, set);
                }
            }
            ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
                self.walk_for_dhatu_instantiations(sthiti, set);
                for stmt in tarhi {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
                if let Some(else_body) = anyatha {
                    for stmt in else_body {
                        self.walk_for_dhatu_instantiations(stmt, set);
                    }
                }
            }
            ASTNode::YavatNode { sthiti, shareera } => {
                self.walk_for_dhatu_instantiations(sthiti, set);
                for stmt in shareera {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::PunahNode { varam, shareera } => {
                self.walk_for_dhatu_instantiations(varam, set);
                for stmt in shareera {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::KramashahNode { body, .. } => {
                for stmt in body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::NidanaNode { arogya_body, dosha_body, .. } => {
                for stmt in arogya_body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
                for stmt in dosha_body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::Dvandva { members, .. } => {
                for member in members {
                    self.walk_for_dhatu_instantiations(member, set);
                }
            }
            ASTNode::NirmanaNode { dravya_name, values, .. } => {
                let angas = match self.type_checker.env.lookup(dravya_name) {
                    Some(sym) => match &sym.devvani_type {
                        DevvaniType::Dravya(_, angas) => angas.clone(),
                        _ => return,
                    },
                    None => return,
                };
                let has_samanya = angas.iter().any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                if has_samanya {
                    let generic_params: Vec<String> = angas.iter()
                        .filter_map(|(_, ty)| if let DevvaniType::Samanya(p) = ty { Some(p.clone()) } else { None })
                        .collect();
                    let mut inference: HashMap<String, DevvaniType> = HashMap::new();
                    let mut resolved_angas: Vec<(String, DevvaniType)> = Vec::new();
                    for (i, (anga_name, expected_ty)) in angas.iter().enumerate() {
                        if let DevvaniType::Samanya(param_name) = expected_ty {
                            if let Some(prev) = inference.get(param_name) {
                                resolved_angas.push((anga_name.clone(), prev.clone()));
                            } else {
                                let found_ty = self.type_checker.check(&values[i]);
                                if found_ty != DevvaniType::Unknown {
                                    inference.insert(param_name.clone(), found_ty.clone());
                                }
                                resolved_angas.push((anga_name.clone(), found_ty));
                            }
                        } else {
                            resolved_angas.push((anga_name.clone(), expected_ty.clone()));
                        }
                    }
                }
            }
            ASTNode::KriyaCall {
                karta,
                karma,
                karana,
                sampradana,
                apadan,
                adhikarana,
                kriya,
                ..
            } => {
                if let Some((mangled, inference, return_type)) = self.dhatu_call_mono_info(
                    kriya,
                    karta,
                    karma,
                    karana,
                    sampradana,
                    apadan,
                    adhikarana,
                ) {
                    let key = (
                        kriya.clone(),
                        mangled.clone(),
                        inference.clone(),
                        return_type,
                    );
                    if !set.contains(&key) {
                        set.push(key);
                    }
                }

                if let Some(k) = karta {
                    self.walk_for_dhatu_instantiations(k, set);
                }
                for arg in karma {
                    self.walk_for_dhatu_instantiations(arg, set);
                }
                if let Some(k) = karana {
                    self.walk_for_dhatu_instantiations(k, set);
                }
                if let Some(k) = sampradana {
                    self.walk_for_dhatu_instantiations(k, set);
                }
                if let Some(k) = apadan {
                    self.walk_for_dhatu_instantiations(k, set);
                }
                if let Some(k) = adhikarana {
                    self.walk_for_dhatu_instantiations(k, set);
                }
            }
            ASTNode::AvartanaNode { call, .. } => {
                self.walk_for_dhatu_instantiations(call, set);
            }
            ASTNode::VaakNode { mulya, .. } => self.walk_for_dhatu_instantiations(mulya, set),
            ASTNode::AstiNode { mulya, .. } => self.walk_for_dhatu_instantiations(mulya, set),
            ASTNode::BhavatiNode { mulya, .. } => self.walk_for_dhatu_instantiations(mulya, set),
            ASTNode::VadatiNode { mulya, .. } => self.walk_for_dhatu_instantiations(mulya, set),
            ASTNode::VinyasaNode { target, index, .. } => {
                self.walk_for_dhatu_instantiations(target, set);
                self.walk_for_dhatu_instantiations(index, set);
            }
            ASTNode::SamavayaNode { target, .. } => {
                self.walk_for_dhatu_instantiations(target, set);
            }
            ASTNode::SandarbhaNode { target, .. } => {
                self.walk_for_dhatu_instantiations(target, set);
            }
            ASTNode::SamprapatiNode { expr, .. } => {
                self.walk_for_dhatu_instantiations(expr, set);
            }
            ASTNode::ArogyaNode { value, .. } => {
                self.walk_for_dhatu_instantiations(value, set);
            }
            ASTNode::DoshaNode { value, .. } => {
                self.walk_for_dhatu_instantiations(value, set);
            }
            _ => {}
        }
    }

    fn collect_dravya_instantiations(&mut self, node: &ASTNode) {
        self.collected_dravya_instantiations.clear();
        let mut set: Vec<(String, String, Vec<(String, DevvaniType)>)> = Vec::new();
        self.walk_for_instantiations(node, &mut set);
        self.collected_dravya_instantiations = set;
    }

    fn walk_for_instantiations(&mut self, node: &ASTNode, set: &mut Vec<(String, String, Vec<(String, DevvaniType)>)>) {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::DhatuDef { body, return_type, .. } => {
                for stmt in body {
                    self.walk_for_instantiations(stmt, set);
                }
                if let Some(rt) = return_type {
                    self.walk_for_instantiations(rt, set);
                }
            }
            ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
                self.walk_for_instantiations(sthiti, set);
                for stmt in tarhi {
                    self.walk_for_instantiations(stmt, set);
                }
                if let Some(else_body) = anyatha {
                    for stmt in else_body {
                        self.walk_for_instantiations(stmt, set);
                    }
                }
            }
            ASTNode::YavatNode { sthiti, shareera } => {
                self.walk_for_instantiations(sthiti, set);
                for stmt in shareera {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::PunahNode { varam, shareera } => {
                self.walk_for_instantiations(varam, set);
                for stmt in shareera {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::KramashahNode { body, .. } => {
                for stmt in body {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::NidanaNode { arogya_body, dosha_body, .. } => {
                for stmt in arogya_body {
                    self.walk_for_instantiations(stmt, set);
                }
                for stmt in dosha_body {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::Dvandva { members, .. } => {
                for member in members {
                    self.walk_for_instantiations(member, set);
                }
            }
            ASTNode::NirmanaNode { dravya_name, values, .. } => {
                let angas = match self.type_checker.env.lookup(dravya_name) {
                    Some(sym) => match &sym.devvani_type {
                        DevvaniType::Dravya(_, angas) => angas.clone(),
                        _ => return,
                    },
                    None => return,
                };
                let has_samanya = angas.iter().any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                if has_samanya {
                    let generic_params: Vec<String> = angas.iter()
                        .filter_map(|(_, ty)| if let DevvaniType::Samanya(p) = ty { Some(p.clone()) } else { None })
                        .collect();
                    let mut inference: HashMap<String, DevvaniType> = HashMap::new();
                    let mut resolved_angas: Vec<(String, DevvaniType)> = Vec::new();
        for (i, (anga_name, expected_ty)) in angas.iter().enumerate() {
                        if let DevvaniType::Samanya(param_name) = expected_ty {
                            if let Some(prev) = inference.get(param_name) {
                                resolved_angas.push((anga_name.clone(), prev.clone()));
                            } else {
                                let found_ty = self.type_checker.check(&values[i]);
                                if found_ty != DevvaniType::Unknown {
                                    inference.insert(param_name.clone(), found_ty.clone());
                                }
                                resolved_angas.push((anga_name.clone(), found_ty));
                            }
                        } else {
                            resolved_angas.push((anga_name.clone(), expected_ty.clone()));
                        }
                    }
                    let mangled = self.mangled_generic_name(dravya_name, &generic_params, &inference);
                    let key = (dravya_name.clone(), mangled.clone(), resolved_angas.clone());
                    if !set.contains(&key) {
                        set.push((dravya_name.clone(), mangled, resolved_angas));
                    }
                }
            }
            ASTNode::VaakNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::AstiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::BhavatiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::VadatiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::KriyaCall { karta, karma, karana, sampradana, apadan, adhikarana, .. } => {
                if let Some(k) = karta {
                    self.walk_for_instantiations(k, set);
                }
                for arg in karma {
                    self.walk_for_instantiations(arg, set);
                }
                if let Some(k) = karana {
                    self.walk_for_instantiations(k, set);
                }
                if let Some(k) = sampradana {
                    self.walk_for_instantiations(k, set);
                }
                if let Some(k) = apadan {
                    self.walk_for_instantiations(k, set);
                }
                if let Some(k) = adhikarana {
                    self.walk_for_instantiations(k, set);
                }
            }
            ASTNode::VinyasaNode { target, index, .. } => {
                self.walk_for_instantiations(target, set);
                self.walk_for_instantiations(index, set);
            }
            ASTNode::SamavayaNode { target, .. } => self.walk_for_instantiations(target, set),
            ASTNode::SandarbhaNode { target, .. } => self.walk_for_instantiations(target, set),
            ASTNode::SamprapatiNode { expr, .. } => self.walk_for_instantiations(expr, set),
            ASTNode::ArogyaNode { value, .. } => self.walk_for_instantiations(value, set),
            ASTNode::DoshaNode { value, .. } => self.walk_for_instantiations(value, set),
            _ => {}
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
use devvani_ast::{ASTNode, AngaField, Gana, KarakaParam, Linga as AstLinga, Lakara as AstLakara, Span, Vacana as AstVacana, Vibhakti};

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
            generic_params: vec![],
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
            generic_params: vec![],
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
            generic_params: vec![],
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
            generic_params: vec![],
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
            generic_params: vec![],
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
            generic_params: vec![],
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


    #[test]
    fn test_sandarbha_immutable_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let target = ASTNode::PurnaankLiteral {
            value: 42,
            span: dummy_span(),
        };
        let node = ASTNode::SandarbhaNode {
            target: Box::new(target),
            is_mutable: false,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert!(codegen.rust_source().contains("&42"));
        assert!(!codegen.rust_source().contains("&&"));
    }

    #[test]
    fn test_sandarbha_mutable_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let target = ASTNode::PurnaankLiteral {
            value: 42,
            span: dummy_span(),
        };
        let node = ASTNode::SandarbhaNode {
            target: Box::new(target),
            is_mutable: true,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert!(codegen.rust_source().contains("&mut 42"));
    }

    #[test]
    fn test_borrowed_immutable_param_signature() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let params = vec![KarakaParam {
            name: "var".to_string(),
            role: KarakaRole::Karma,
            vibhakti: Vibhakti::Dvitiya,
            is_borrowed: true,
            is_mutable_borrow: false,
            span: dummy_span(),
            type_name: "sankhya".to_string(),
        }];
        let dhatu = ASTNode::DhatuDef {
            name: "my_func".to_string(),
            generic_params: vec![],
            params,
            body: vec![],
            lakara: AstLakara::Lat,
            gana: Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            return_karaka: None,
            return_type: None,
            upasargas: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&dhatu).is_ok());
        assert!(codegen.rust_source().contains("var: &i64"));
    }

    #[test]
    fn test_borrowed_mutable_param_signature() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let params = vec![KarakaParam {
            name: "var".to_string(),
            role: KarakaRole::Karma,
            vibhakti: Vibhakti::Dvitiya,
            is_borrowed: true,
            is_mutable_borrow: true,
            span: dummy_span(),
            type_name: "sankhya".to_string(),
        }];
        let dhatu = ASTNode::DhatuDef {
            name: "my_func".to_string(),
            generic_params: vec![],
            params,
            body: vec![],
            lakara: AstLakara::Lat,
            gana: Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            return_karaka: None,
            return_type: None,
            upasargas: vec![],
            span: dummy_span(),
        };
        assert!(codegen.emit(&dhatu).is_ok());
        assert!(codegen.rust_source().contains("var: &mut i64"));
    }

    #[test]
    fn test_call_site_auto_wrap_immutable() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.function_params_mut().insert(
            "test_func".to_string(),
            vec![KarakaParam {
                name: "param".to_string(),
                role: KarakaRole::Karma,
                vibhakti: Vibhakti::Prathama,
                is_borrowed: true,
                is_mutable_borrow: false,
                span: dummy_span(),
                type_name: "sankhya".to_string(),
            }],
        );
        let args = vec![ASTNode::PurnaankLiteral {
            value: 10,
            span: dummy_span(),
        }];
        let node = ASTNode::KriyaCall {
            karta: None,
            kriya: "test_func".to_string(),
            karma: args,
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("test_func"));
        assert!(output.contains("&10"));
        assert!(!output.contains("&&"));
    }

    #[test]
    fn test_call_site_auto_wrap_mutable() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.function_params_mut().insert(
            "test_func_mut".to_string(),
            vec![KarakaParam {
                name: "param".to_string(),
                role: KarakaRole::Karma,
                vibhakti: Vibhakti::Prathama,
                is_borrowed: true,
                is_mutable_borrow: true,
                span: dummy_span(),
                type_name: "sankhya".to_string(),
            }],
        );
        let args = vec![ASTNode::PurnaankLiteral {
            value: 10,
            span: dummy_span(),
        }];
        let node = ASTNode::KriyaCall {
            karta: None,
            kriya: "test_func_mut".to_string(),
            karma: args,
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("test_func_mut"));
        assert!(output.contains("&mut 10"));
    }

    #[test]
    fn test_no_double_wrap_on_sandarbha_arg() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.function_params_mut().insert(
            "borrowed_func".to_string(),
            vec![KarakaParam {
                name: "param".to_string(),
                role: KarakaRole::Karma,
                vibhakti: Vibhakti::Dvitiya,
                is_borrowed: true,
                is_mutable_borrow: false,
                span: dummy_span(),
                type_name: "sankhya".to_string(),
            }],
        );
        let inner = ASTNode::PurnaankLiteral {
            value: 42,
            span: dummy_span(),
        };
        let sandarbha = ASTNode::SandarbhaNode {
            target: Box::new(inner),
            is_mutable: false,
            span: dummy_span(),
        };
        let args = vec![sandarbha];
        let node = ASTNode::KriyaCall {
            karta: None,
            kriya: "borrowed_func".to_string(),
            karma: args,
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("borrowed_func"));
        assert!(output.contains("&42"));
        assert!(!output.contains("&&"));
    }

    #[test]
    fn test_full_sandarbha_signature_roundtrip() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.type_checker.function_params_mut().insert(
            "entry".to_string(),
            vec![KarakaParam {
                name: "arg".to_string(),
                role: KarakaRole::Karma,
                vibhakti: Vibhakti::Dvitiya,
                is_borrowed: true,
                is_mutable_borrow: false,
                span: dummy_span(),
                type_name: "sankhya".to_string(),
            }],
        );
        let inner = ASTNode::PurnaankLiteral {
            value: 100,
            span: dummy_span(),
        };
        let borrowed_arg = ASTNode::SandarbhaNode {
            target: Box::new(inner),
            is_mutable: false,
            span: dummy_span(),
        };
        let node = ASTNode::KriyaCall {
            karta: None,
            kriya: "entry".to_string(),
            karma: vec![borrowed_arg],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("entry"));
        assert!(output.contains("&100"));
        assert!(!output.contains("&&"));
    }

    #[test]
    fn test_generic_dravya_single_param_single_instantiation() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DravyaDef {
                    name: "Peti".to_string(),
                    generic_params: vec!["T".to_string()],
                    angas: vec![AngaField {
                        name: "mulya".to_string(),
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::VaakLiteral {
                        value: "hello".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("struct Peti__String {"));
        assert!(output.contains("mulya: String"));
        assert!(output.contains("Peti__String { mulya: \"hello\" }"));
    }

    #[test]
    fn test_generic_dravya_same_param_same_type_deduplicates() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DravyaDef {
                    name: "Peti".to_string(),
                    generic_params: vec!["T".to_string()],
                    angas: vec![AngaField {
                        name: "mulya".to_string(),
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::VaakLiteral {
                        value: "first".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::VaakLiteral {
                        value: "second".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        let count = output.matches("struct Peti__String {").count();
        assert_eq!(count, 1, "expected exactly one Peti__String struct definition, got {}", count);
        assert!(output.contains("Peti__String { mulya: \"first\" }"));
        assert!(output.contains("Peti__String { mulya: \"second\" }"));
    }

    #[test]
    fn test_generic_dravya_same_param_two_different_types() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DravyaDef {
                    name: "Peti".to_string(),
                    generic_params: vec!["T".to_string()],
                    angas: vec![AngaField {
                        name: "mulya".to_string(),
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::VaakLiteral {
                        value: "vaak".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::PurnaankLiteral {
                        value: 42,
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("struct Peti__String {"));
        assert!(output.contains("struct Peti__i64 {"));
        assert!(output.contains("Peti__String { mulya: \"vaak\" }"));
        assert!(output.contains("Peti__i64 { mulya: 42 }"));
    }

    #[test]
    fn test_generic_dravya_two_params_single_instantiation() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DravyaDef {
                    name: "Yugala".to_string(),
                    generic_params: vec!["T".to_string(), "U".to_string()],
                    angas: vec![
                        AngaField {
                            name: "a".to_string(),
                            type_name: "T".to_string(),
                            span: dummy_span(),
                        },
                        AngaField {
                            name: "b".to_string(),
                            type_name: "U".to_string(),
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Yugala".to_string(),
                    values: vec![
                        ASTNode::VaakLiteral {
                            value: "hello".to_string(),
                            span: dummy_span(),
                        },
                        ASTNode::PurnaankLiteral {
                            value: 7,
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("struct Yugala__String__i64 {"));
        assert!(output.contains("a: String"));
        assert!(output.contains("b: i64"));
        assert!(output.contains("Yugala__String__i64 { a: \"hello\", b: 7 }"));
    }

    // ── Part 3B: Generic Dhātu (function) monomorphization tests ──

    #[test]
    fn test_generic_dhatu_single_param_single_instantiation() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "pratirupa".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![KarakaParam {
                        name: "vastu".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "pratirupa".to_string(),
                    karma: vec![ASTNode::VaakLiteral {
                        value: "vaak".to_string(),
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("pub fn pratirupa__String(vastu: String) -> Result<String, String> {"));
        assert!(output.contains("pratirupa__String(\"vaak\")"));
    }

    #[test]
    fn test_generic_dhatu_same_param_same_type_deduplicates() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "pratirupa".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![KarakaParam {
                        name: "vastu".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "pratirupa".to_string(),
                    karma: vec![ASTNode::VaakLiteral {
                        value: "first".to_string(),
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "pratirupa".to_string(),
                    karma: vec![ASTNode::VaakLiteral {
                        value: "second".to_string(),
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        let count = output.matches("pub fn pratirupa__String").count();
        assert_eq!(count, 1, "expected exactly one pratirupa__String definition, got {}", count);
        assert!(output.contains("pratirupa__String(\"first\")"));
        assert!(output.contains("pratirupa__String(\"second\")"));
    }

    #[test]
    fn test_generic_dhatu_same_param_two_different_types() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "pratirupa".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![KarakaParam {
                        name: "vastu".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "pratirupa".to_string(),
                    karma: vec![ASTNode::VaakLiteral {
                        value: "vaak".to_string(),
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "pratirupa".to_string(),
                    karma: vec![ASTNode::PurnaankLiteral {
                        value: 42,
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("pub fn pratirupa__String(vastu: String) -> Result<String, String> {"));
        assert!(output.contains("pub fn pratirupa__i64(vastu: i64) -> Result<i64, String> {"));
        assert!(output.contains("pratirupa__String(\"vaak\")"));
        assert!(output.contains("pratirupa__i64(42)"));
    }

    #[test]
    fn test_generic_dhatu_two_params_single_instantiation() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "yugala".to_string(),
                    generic_params: vec!["T".to_string(), "U".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![
                        KarakaParam {
                            name: "a".to_string(),
                            role: KarakaRole::Karma,
                            vibhakti: Vibhakti::Dvitiya,
                            is_borrowed: false,
                            is_mutable_borrow: false,
                            type_name: "T".to_string(),
                            span: dummy_span(),
                        },
                        KarakaParam {
                            name: "b".to_string(),
                            role: KarakaRole::Karma,
                            vibhakti: Vibhakti::Dvitiya,
                            is_borrowed: false,
                            is_mutable_borrow: false,
                            type_name: "U".to_string(),
                            span: dummy_span(),
                        },
                    ],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "yugala".to_string(),
                    karma: vec![
                        ASTNode::VaakLiteral {
                            value: "hello".to_string(),
                            span: dummy_span(),
                        },
                        ASTNode::PurnaankLiteral {
                            value: 7,
                            span: dummy_span(),
                        },
                    ],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("pub fn yugala__String__i64(a: String, b: i64) -> Result<String, String> {"));
        assert!(output.contains("yugala__String__i64(\"hello\", 7)"));
    }

    #[test]
    fn test_generic_dhatu_recursive_self_call() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "factorial".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![KarakaParam {
                        name: "n".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![ASTNode::AvartanaNode {
                        call: Box::new(ASTNode::KriyaCall {
                            karta: None,
                            kriya: "factorial".to_string(),
                            karma: vec![ASTNode::PurnaankLiteral {
                                value: 1,
                                span: dummy_span(),
                            }],
                            karana: None,
                            sampradana: None,
                            apadan: None,
                            adhikarana: None,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "factorial".to_string(),
                    karma: vec![ASTNode::PurnaankLiteral {
                        value: 5,
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        let result = codegen.generate(&program);
        assert!(result.is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("pub fn factorial__i64(n: i64) -> Result<i64, String> {"));
        assert!(output.contains("factorial__i64(1)"));
        assert!(output.contains("factorial__i64(5)"));
    }

    #[test]
    fn test_generic_dravya_and_dhatu_together() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DravyaDef {
                    name: "Peti".to_string(),
                    generic_params: vec!["T".to_string()],
                    angas: vec![AngaField {
                        name: "mulya".to_string(),
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "echo".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: devvani_ast::Linga::Pullinga,
                    vacana: devvani_ast::Vacana::Eka,
                    params: vec![KarakaParam {
                        name: "x".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "T".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "T".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![],
                    span: dummy_span(),
                },
                ASTNode::NirmanaNode {
                    dravya_name: "Peti".to_string(),
                    values: vec![ASTNode::VaakLiteral {
                        value: "hello".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "echo".to_string(),
                    karma: vec![ASTNode::PurnaankLiteral {
                        value: 42,
                        span: dummy_span(),
                    }],
                    karana: None,
                    sampradana: None,
                    apadan: None,
                    adhikarana: None,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("struct Peti__String {"));
        assert!(output.contains("mulya: String"));
        assert!(output.contains("pub fn echo__i64(x: i64) -> Result<i64, String> {"));
        assert!(output.contains("echo__i64(42)"));
    }
}
