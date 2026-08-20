use devvani_ast::{ASTNode, KarakaParam, KarakaRole, NaamadheyaNode, VikaraEntry, VikaraKind};
use devvani_typesystem::{
    lakara_from_str, lakara_to_scope,
    vaak::{MoveChecker, VaakOwnership},
    DevvaniType, Lakara, TypeChecker,
};
use std::collections::{HashMap, HashSet};

// ── Identifier sanitization ──────────────────────────────────
fn sanitize_rust_ident(ident: &str) -> String {
    ident.replace('-', "_")
}

fn infer_return_type_from_body(body: &[ASTNode]) -> Option<String> {
    let last = body.last()?;
    match last {
        ASTNode::YogaNode { vama, dakshina }
        | ASTNode::ViyogaNode { vama, dakshina }
        | ASTNode::GunaNode { vama, dakshina }
        | ASTNode::BhagaNode { vama, dakshina } => {
            if matches!(vama.as_ref(), ASTNode::PurnaankLiteral { .. }) {
                Some("i64".to_string())
            } else if matches!(dakshina.as_ref(), ASTNode::PurnaankLiteral { .. }) {
                Some("i64".to_string())
            } else if matches!(vama.as_ref(), ASTNode::DashaamshaLiteral { .. }) {
                Some("f64".to_string())
            } else if matches!(dakshina.as_ref(), ASTNode::DashaamshaLiteral { .. }) {
                Some("f64".to_string())
            } else {
                None
            }
        }
        ASTNode::SamaNode { vama, dakshina }
        | ASTNode::AsamaNode { vama, dakshina }
        | ASTNode::NyuunaNode { vama, dakshina }
        | ASTNode::AdhikaNode { vama, dakshina } => {
            if matches!(vama.as_ref(), ASTNode::PurnaankLiteral { .. })
                || matches!(dakshina.as_ref(), ASTNode::PurnaankLiteral { .. })
            {
                Some("i64".to_string())
            } else if matches!(vama.as_ref(), ASTNode::DashaamshaLiteral { .. })
                || matches!(dakshina.as_ref(), ASTNode::DashaamshaLiteral { .. })
            {
                Some("f64".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

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
    collected_dhatu_instantiations:
        Vec<(String, String, HashMap<String, DevvaniType>, DevvaniType)>,
    current_dhatu_context: Option<(String, String)>,
    current_inference: Option<HashMap<String, DevvaniType>>,
    pending_vritti: Vec<String>,
    pending_tippani: Vec<(String, String)>,
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
            current_inference: None,
            pending_vritti: Vec::new(),
            pending_tippani: Vec::new(),
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
                for stmt in shareera {
                    if let ASTNode::BhashyaNode { text, .. } = stmt {
                        self.emit_doc_lines("//!", text);
                    } else {
                        break;
                    }
                }
                for stmt in shareera {
                    if let ASTNode::MrittikaNode {
                        package_name,
                        naamadheya,
                        vikaras,
                        ..
                    } = stmt
                    {
                        self.emit_mrittika_metadata(package_name, naamadheya, vikaras);
                    }
                }
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

                    self.rust_output.push_str(&format!(
                        "{}{}",
                        self.indent_str(),
                        if symbol.mutability.is_mutable {
                            format!("mut {}", sanitize_rust_ident(display_name))
                        } else {
                            sanitize_rust_ident(display_name).to_string()
                        },
                    ));
                } else {
                    self.rust_output.push_str(&format!(
                        "{}{}",
                        self.indent_str(),
                        sanitize_rust_ident(display_name)
                    ));
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
                self.rust_output.push_str(&format!(
                    "{}let {} = ",
                    self.indent_str(),
                    sanitize_rust_ident(naama)
                ));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind {
                    name: naama.clone(),
                    rust_type: "auto".into(),
                    mutable: false,
                });
            }
            ASTNode::BhavatiNode { naama, mulya } => {
                self.rust_output.push_str(&format!(
                    "{}let mut {} = ",
                    self.indent_str(),
                    sanitize_rust_ident(naama)
                ));
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
                self.instructions.push(Instruction::Bind {
                    name: naama.clone(),
                    rust_type: "auto".into(),
                    mutable: true,
                });
            }
            ASTNode::DharaNode {
                naamas,
                type_name,
                mulya,
                is_mutable,
                ..
            } => {
                let node_ptr = node as *const ASTNode;

                if naamas.len() == 1 {
                    let naama = &naamas[0];
                    let mut rust_ty_str = None;

                    if let Some(tn) = type_name {
                        let rust_ty = self.type_name_to_rust_type(tn);
                        rust_ty_str = Some(rust_ty);
                    } else if let Some(ty) = self.type_checker.node_type_map().get(&node_ptr) {
                        let resolved_ty = if let Some(ref inference) = self.current_inference {
                            Codegen::substitute_samanya_in_type(ty.clone(), inference)
                        } else {
                            ty.clone()
                        };
                        let rust_ty = self.type_name_to_rust_type_by_type(&resolved_ty);
                        if rust_ty != "auto" {
                            rust_ty_str = Some(rust_ty);
                        }
                    }

                    if let Some(rust_ty) = rust_ty_str {
                        if *is_mutable {
                            self.rust_output.push_str(&format!(
                                "{}let mut {}: {} = ",
                                self.indent_str(),
                                sanitize_rust_ident(naama),
                                rust_ty
                            ));
                        } else {
                            self.rust_output.push_str(&format!(
                                "{}let {}: {} = ",
                                self.indent_str(),
                                sanitize_rust_ident(naama),
                                rust_ty
                            ));
                        }
                        self.instructions.push(Instruction::Bind {
                            name: naama.clone(),
                            rust_type: rust_ty,
                            mutable: *is_mutable,
                        });
                    } else {
                        if *is_mutable {
                            self.rust_output.push_str(&format!(
                                "{}let mut {} = ",
                                self.indent_str(),
                                sanitize_rust_ident(naama)
                            ));
                        } else {
                            self.rust_output.push_str(&format!(
                                "{}let {} = ",
                                self.indent_str(),
                                sanitize_rust_ident(naama)
                            ));
                        }
                        self.instructions.push(Instruction::Bind {
                            name: naama.clone(),
                            rust_type: "auto".into(),
                            mutable: *is_mutable,
                        });
                    }
                } else {
                    let names_str = naamas
                        .iter()
                        .map(|n| sanitize_rust_ident(n))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if *is_mutable {
                        self.rust_output.push_str(&format!(
                            "{}let mut ({}) = ",
                            self.indent_str(),
                            names_str
                        ));
                    } else {
                        self.rust_output.push_str(&format!(
                            "{}let ({}) = ",
                            self.indent_str(),
                            names_str
                        ));
                    }
                    for naama in naamas {
                        self.instructions.push(Instruction::Bind {
                            name: naama.clone(),
                            rust_type: "auto".into(),
                            mutable: *is_mutable,
                        });
                    }
                }
                self.emit(mulya)?;
                self.rust_output.push_str(";\n");
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
                let sanitized = sanitize_rust_ident(naama);
                self.rust_output.push_str(&format!(
                    "{}let mut {} = String::new(); std::io::stdin().read_line(&mut {}).unwrap();\n",
                    self.indent_str(),
                    sanitized,
                    sanitized
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
            ASTNode::KriyaCall {
                karta,
                kriya,
                karma,
                karana,
                sampradana,
                apadan,
                adhikarana,
                ..
            } => {
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
                            self.rust_output
                                .push_str(&sanitize_rust_ident(display_name));
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
                            self.rust_output
                                .push_str(&sanitize_rust_ident(display_name));
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
                                kriya, karta, karma, karana, sampradana, apadan, adhikarana,
                            )
                            .unwrap_or_else(|| kriya.clone())
                        }
                    } else {
                        self.mangled_name_for_dhatu_call(
                            kriya, karta, karma, karana, sampradana, apadan, adhikarana,
                        )
                        .unwrap_or_else(|| kriya.clone())
                    };

                    self.rust_output.push_str(&self.indent_str());
                    self.rust_output.push_str(&sanitize_rust_ident(&emit_name));
                    self.rust_output.push_str("(");
                    for (i, arg) in karma.iter().enumerate() {
                        if i > 0 {
                            self.rust_output.push_str(", ");
                        }
                        if let ASTNode::SandarbhaNode { .. } = arg {
                            self.emit(arg)?;
                        } else if let Some(params) = self.type_checker.function_params().get(kriya)
                        {
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
                self.rust_output.push_str(&format!(
                    "{}for {} in ",
                    self.indent_str(),
                    sanitize_rust_ident(item_name)
                ));
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
                self.flush_doc_comments();
                let has_samanya = !generic_params.is_empty();

                if !has_samanya {
                    let ts_lakara =
                        lakara_from_str(&format!("{:?}", lakara)).unwrap_or(Lakara::Lat);
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
                        rust_params.push(format!(
                            "{}: {}",
                            sanitize_rust_ident(&param.name),
                            type_str
                        ));
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
                                let success_rust = self.type_name_to_rust_type(success_type);
                                let error_rust = self.type_name_to_rust_type(error_type);
                                return_type_str =
                                    format!(" -> Result<{}, {}>", success_rust, error_rust);
                            }
                            other => {
                                return_type_str =
                                    format!(" -> {}", self.generate_to_string(other)?);
                            }
                        }
                    }

                    if return_type_str.is_empty() {
                        if let Some(inferred_rt) =
                            self.type_checker.function_return_types().get(name)
                        {
                            match inferred_rt {
                                DevvaniType::Phalam(success, error) => {
                                    let success_rust = self.type_name_to_rust_type_by_type(success);
                                    let error_rust = self.type_name_to_rust_type_by_type(error);
                                    return_type_str =
                                        format!(" -> Result<{}, {}>", success_rust, error_rust);
                                }
                                other => {
                                    let rust_ty = self.type_name_to_rust_type_by_type(other);
                                    if rust_ty != "auto" {
                                        return_type_str = format!(" -> {}", rust_ty);
                                    }
                                }
                            }
                        }

                        if return_type_str.is_empty() {
                            if let Some(inferred) = infer_return_type_from_body(body) {
                                return_type_str = format!(" -> {}", inferred);
                            }
                        }
                    }

                    let async_kw = if scope.is_async { "async " } else { "" };
                    let line = format!(
                        "{}pub {}fn {}({}){return_type_str} {{\n",
                        self.indent_str(),
                        async_kw,
                        sanitize_rust_ident(name),
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
                    for (dhatu_name, mangled, inference, concrete_return) in instantiations {
                        if dhatu_name == *name {
                            if emitted_names.insert(mangled.clone()) {
                                let prev_context = self.current_dhatu_context.take();
                                self.current_dhatu_context = Some((name.clone(), mangled.clone()));
                                self.emit_monomorphized_dhatu(
                                    &mangled,
                                    params,
                                    &inference,
                                    Some(concrete_return),
                                    body,
                                );
                                self.current_dhatu_context = prev_context;
                            }
                        }
                    }
                }
            }
            ASTNode::DravyaDef {
                name,
                angas,
                generic_params,
                ..
            } => {
                self.flush_doc_comments();
                let has_samanya = !generic_params.is_empty();

                if !has_samanya {
                    self.rust_output
                        .push_str(&format!("{}#[derive(Debug, Clone)]\n", self.indent_str()));
                    if angas.is_empty() {
                        self.rust_output.push_str(&format!(
                            "{}struct {} {{}};\n",
                            self.indent_str(),
                            sanitize_rust_ident(name)
                        ));
                    } else {
                        self.rust_output.push_str(&format!(
                            "{}struct {} {{\n",
                            self.indent_str(),
                            sanitize_rust_ident(name)
                        ));

                        self.indent += 1;
                        for (i, anga) in angas.iter().enumerate() {
                            if i > 0 {
                                self.rust_output.push_str(",\n");
                            }
                            let rust_ty = self.type_name_to_rust_type(&anga.type_name);
                            self.rust_output.push_str(&format!(
                                "{}{}: {}",
                                self.indent_str(),
                                sanitize_rust_ident(&anga.name),
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
            ASTNode::SamavayaNode {
                target, anga_name, ..
            } => {
                self.emit(target)?;
                self.rust_output.push_str(".");
                self.rust_output.push_str(&sanitize_rust_ident(anga_name));
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

                let has_samanya = angas
                    .iter()
                    .any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                let emit_name = if has_samanya {
                    if let Some((generic_params, inference)) =
                        self.infer_generic_concrete_types(dravya_name, values)
                    {
                        self.mangled_generic_name(dravya_name, &generic_params, &inference)
                    } else {
                        dravya_name.clone()
                    }
                } else {
                    dravya_name.clone()
                };

                if angas.is_empty() {
                    self.rust_output.push_str(&format!(
                        "{}{} {{}}",
                        self.indent_str(),
                        sanitize_rust_ident(&emit_name)
                    ));
                } else {
                    self.rust_output.push_str(&format!(
                        "{}{} {{ ",
                        self.indent_str(),
                        sanitize_rust_ident(&emit_name)
                    ));
                    for (i, (field_name, _)) in angas.iter().enumerate() {
                        if i > 0 {
                            self.rust_output.push_str(", ");
                        }
                        self.rust_output
                            .push_str(&format!("{}: ", sanitize_rust_ident(field_name)));
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
                    sanitize_rust_ident(arogya_bind)
                ));
                self.indent += 1;
                self.emit_body(arogya_body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}},\n", self.indent_str()));
                self.rust_output.push_str(&format!(
                    "{}Err({}) => {{\n",
                    self.indent_str(),
                    sanitize_rust_ident(dosha_bind)
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
                target, is_mutable, ..
            } => {
                if *is_mutable {
                    self.rust_output.push_str("&mut ");
                } else {
                    self.rust_output.push_str("&");
                }
                self.emit(target)?;
            }
            ASTNode::SamyogaNode { body, .. } => {
                self.rust_output.push_str("std::thread::spawn(move || {\n");
                self.indent += 1;
                self.emit_body(body)?;
                self.indent -= 1;
                self.rust_output.push_str("})");
            }
            ASTNode::PraptiNode { handle, .. } => {
                self.emit(handle)?;
                self.rust_output.push_str(".join().unwrap()");
            }
            ASTNode::DutaBanaaNode { .. } => {
                self.rust_output.push_str("std::sync::mpsc::channel()");
            }
            ASTNode::DutaBhejNode {
                sender, message, ..
            } => {
                self.rust_output.push_str(&self.indent_str());
                self.emit(sender)?;
                self.rust_output.push_str(".send(");
                self.emit(message)?;
                self.rust_output.push_str(").unwrap();\n");
            }
            ASTNode::DutaGrahanNode { receiver, .. } => {
                self.emit(receiver)?;
                self.rust_output.push_str(".recv().unwrap()");
            }
            ASTNode::ManasNode { target, body, .. } => {
                let target_name = if let ASTNode::Nama { base, .. } = target.as_ref() {
                    base.clone()
                } else {
                    return Err(CodegenError::UnsupportedNode(
                        "ManasNode target must be a Nama identifier".to_string(),
                    ));
                };
                self.rust_output.push_str("{\n");
                self.indent += 1;
                self.rust_output.push_str(&format!(
                    "{}let mut {} = ",
                    self.indent_str(),
                    sanitize_rust_ident(&target_name)
                ));
                let saved_indent = self.indent;
                self.indent = 0;
                self.emit(target)?;
                self.indent = saved_indent;
                self.rust_output.push_str(".lock().unwrap();\n");
                self.emit_body(body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::ParinamaNode { mulyam, dhatus, .. } => {
                if dhatus.is_empty() {
                    return Err(CodegenError::UnsupportedNode(
                        "ParinamaNode with empty dhatus".to_string(),
                    ));
                }

                let is_fallible = dhatus.iter().any(|d| {
                    if let Some(return_ty) = self.type_checker.function_return_types().get(d) {
                        matches!(return_ty, DevvaniType::Phalam(_, _))
                    } else {
                        false
                    }
                });

                if dhatus.len() == 1 {
                    let dhatu_name = &dhatus[0];
                    let emit_name = self
                        .mangled_name_for_dhatu_call(
                            dhatu_name,
                            &None,
                            &[],
                            &None,
                            &None,
                            &None,
                            &None,
                        )
                        .unwrap_or_else(|| dhatu_name.clone());

                    self.rust_output.push_str(&sanitize_rust_ident(&emit_name));
                    self.rust_output.push_str("(");

                    if let Some(params) = self.type_checker.function_params().get(dhatu_name) {
                        if let Some(param) = params.get(0) {
                            if param.is_borrowed {
                                if param.is_mutable_borrow {
                                    self.rust_output.push_str("&mut ");
                                } else {
                                    self.rust_output.push_str("&");
                                }
                            }
                        }
                    }

                    self.emit(mulyam)?;
                    self.rust_output.push_str(")");
                } else if is_fallible {
                    let mut expr = String::new();

                    for (i, dhatu_name) in dhatus.iter().enumerate() {
                        let emit_name = self
                            .mangled_name_for_dhatu_call(
                                dhatu_name,
                                &None,
                                &[],
                                &None,
                                &None,
                                &None,
                                &None,
                            )
                            .unwrap_or_else(|| dhatu_name.clone());

                        let is_dhatu_fallible = if let Some(return_ty) =
                            self.type_checker.function_return_types().get(dhatu_name)
                        {
                            matches!(return_ty, DevvaniType::Phalam(_, _))
                        } else {
                            false
                        };

                        if i == 0 {
                            let mut borrow_prefix = String::new();
                            if let Some(params) =
                                self.type_checker.function_params().get(dhatu_name)
                            {
                                if let Some(param) = params.get(0) {
                                    if param.is_borrowed {
                                        if param.is_mutable_borrow {
                                            borrow_prefix = "&mut ".to_string();
                                        } else {
                                            borrow_prefix = "&".to_string();
                                        }
                                    }
                                }
                            }

                            let arg_str = self.expr_to_string(mulyam)?;
                            if is_dhatu_fallible {
                                expr = format!(
                                    "{}({}{})",
                                    sanitize_rust_ident(&emit_name),
                                    borrow_prefix,
                                    arg_str
                                );
                            } else {
                                expr = format!(
                                    "Ok({}({}{}))",
                                    sanitize_rust_ident(&emit_name),
                                    borrow_prefix,
                                    arg_str
                                );
                            }
                        } else {
                            let var_name = format!("v{}", i - 1);
                            if is_dhatu_fallible {
                                expr = format!(
                                    "{}.and_then(|{}| {}({}))",
                                    expr,
                                    var_name,
                                    sanitize_rust_ident(&emit_name),
                                    var_name
                                );
                            } else {
                                expr = format!(
                                    "{}.and_then(|{}| Ok({}({})))",
                                    expr,
                                    var_name,
                                    sanitize_rust_ident(&emit_name),
                                    var_name
                                );
                            }
                        }
                    }

                    self.rust_output.push_str(&expr);
                } else {
                    let mut expr = self.expr_to_string(mulyam)?;

                    for dhatu_name in dhatus.iter() {
                        let emit_name = self
                            .mangled_name_for_dhatu_call(
                                dhatu_name,
                                &None,
                                &[],
                                &None,
                                &None,
                                &None,
                                &None,
                            )
                            .unwrap_or_else(|| dhatu_name.clone());

                        let mut borrow_prefix = String::new();
                        if let Some(params) = self.type_checker.function_params().get(dhatu_name) {
                            if let Some(param) = params.get(0) {
                                if param.is_borrowed {
                                    if param.is_mutable_borrow {
                                        borrow_prefix = "&mut ".to_string();
                                    } else {
                                        borrow_prefix = "&".to_string();
                                    }
                                }
                            }
                        }

                        expr = format!(
                            "{}({}{})",
                            sanitize_rust_ident(&emit_name),
                            borrow_prefix,
                            expr
                        );
                    }

                    self.rust_output.push_str(&expr);
                }
            }
            ASTNode::ParikshaaNode {
                name,
                body,
                is_tarka,
                ..
            } => {
                if *is_tarka {
                    self.rust_output.push_str("#[test]\n#[should_panic]\n");
                } else {
                    self.rust_output.push_str("#[test]\n");
                }
                self.rust_output.push_str(&format!(
                    "{}fn {}() {{\n",
                    self.indent_str(),
                    sanitize_rust_ident(name)
                ));
                self.indent += 1;
                self.emit_body(body)?;
                self.indent -= 1;
                self.rust_output
                    .push_str(&format!("{}}}\n", self.indent_str()));
            }
            ASTNode::NigamanaNode { expr, .. } => {
                self.rust_output
                    .push_str(&format!("{}assert!(", self.indent_str()));
                self.emit(expr)?;
                self.rust_output.push_str(");\n");
            }
            ASTNode::SadrishyaNigamanaNode { left, right, .. } => {
                self.rust_output
                    .push_str(&format!("{}assert_eq!(", self.indent_str()));
                self.emit(left)?;
                self.rust_output.push_str(", ");
                self.emit(right)?;
                self.rust_output.push_str(");\n");
            }
            ASTNode::AsadrishyaNigamanaNode { left, right, .. } => {
                self.rust_output
                    .push_str(&format!("{}assert_ne!(", self.indent_str()));
                self.emit(left)?;
                self.rust_output.push_str(", ");
                self.emit(right)?;
                self.rust_output.push_str(");\n");
            }
            ASTNode::BhashyaNode { .. } => {}
            ASTNode::VrittiNode { text, .. } => {
                self.pending_vritti.push(text.clone());
            }
            ASTNode::TippaniNode {
                text, param_name, ..
            } => {
                self.pending_tippani
                    .push((param_name.clone(), text.clone()));
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

    fn emit_doc_lines(&mut self, prefix: &str, text: &str) {
        for line in text.split('\n') {
            self.rust_output
                .push_str(&format!("{}{} {}\n", self.indent_str(), prefix, line));
        }
    }

    fn emit_mrittika_metadata(
        &mut self,
        package_name: &str,
        naamadheya: &NaamadheyaNode,
        vikaras: &[VikaraEntry],
    ) {
        let mut block = String::new();
        block.push_str("# Devvani Package Metadata (मृत्तिका)\n");
        block.push_str(&format!("- Package: {}\n", package_name));
        block.push_str(&format!(
            "- Version (नामधेय): {}\n",
            naamadheya.version_string
        ));
        if !vikaras.is_empty() {
            block.push('\n');
            block.push_str("## Vikara History (विकार-इतिहास)\n");
            for vikara in vikaras {
                let tag = match vikara.kind {
                    VikaraKind::Sukshma => "SUKSHMA",
                    VikaraKind::Sthula => "STHULA",
                    VikaraKind::SatyaBheda => "SATYA-BHEDA",
                };
                block.push_str(&format!("- [{}] {}\n", tag, vikara.description));
            }
        }
        let block = block.trim_end_matches('\n');
        self.emit_doc_lines("//!", block);
    }

    fn flush_doc_comments(&mut self) {
        let vrittis: Vec<_> = self.pending_vritti.drain(..).collect();
        let tippanis: Vec<_> = self.pending_tippani.drain(..).collect();
        let has_vritti = !vrittis.is_empty();
        for vritti_text in vrittis {
            self.emit_doc_lines("///", &vritti_text);
        }
        if has_vritti || !tippanis.is_empty() {
            self.rust_output
                .push_str(&format!("{}///\n", self.indent_str()));
            if !tippanis.is_empty() {
                self.rust_output
                    .push_str(&format!("{}/// # Parameters\n", self.indent_str()));
                for (param_name, tippani_text) in tippanis {
                    self.rust_output.push_str(&format!(
                        "{}/// * {} - {}\n",
                        self.indent_str(),
                        param_name,
                        tippani_text
                    ));
                }
            }
        }
    }

    fn emit_body(&mut self, body: &[ASTNode]) -> Result<(), CodegenError> {
        let mut i = 0;
        while i < body.len() {
            match &body[i] {
                ASTNode::VrittiNode { text, .. } => {
                    self.pending_vritti.push(text.clone());
                    i += 1;
                }
                ASTNode::TippaniNode {
                    text, param_name, ..
                } => {
                    self.pending_tippani
                        .push((param_name.clone(), text.clone()));
                    i += 1;
                }
                ASTNode::BhashyaNode { .. } => {
                    i += 1;
                }
                ASTNode::MrittikaNode { .. } => {
                    i += 1;
                }
                _ => {
                    self.flush_doc_comments();
                    self.emit(&body[i])?;
                    if i < body.len() - 1 && !self.rust_output.ends_with(";\n") {
                        self.rust_output.push_str(";\n");
                    }
                    i += 1;
                }
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

    fn expr_to_string(&mut self, node: &ASTNode) -> Result<String, CodegenError> {
        let old_output = self.rust_output.clone();
        let old_indent = self.indent;
        self.rust_output = String::new();
        self.indent = 0;
        self.emit(node)?;
        let result = self.rust_output.clone();
        self.rust_output = old_output;
        self.indent = old_indent;
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

    fn mangled_generic_name(
        &self,
        base_name: &str,
        generic_params: &[String],
        inference: &HashMap<String, DevvaniType>,
    ) -> String {
        if generic_params.is_empty() {
            return base_name.to_string();
        }
        let suffix: Vec<String> = generic_params
            .iter()
            .filter_map(|param| {
                inference
                    .get(param)
                    .map(|ty| self.type_name_to_rust_type_by_type(ty))
            })
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
            DevvaniType::Avali(elem) => {
                DevvaniType::Avali(Box::new(Self::substitute_samanya_in_type(*elem, inference)))
            }
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

        let return_type =
            if let Some(declared_return) = self.type_checker.function_return_types().get(kriya) {
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

    fn emit_monomorphized_dravya(&mut self, mangled_name: &str, angas: &[(String, DevvaniType)]) {
        self.flush_doc_comments();
        self.rust_output
            .push_str(&format!("{}#[derive(Debug, Clone)]\n", self.indent_str()));
        if angas.is_empty() {
            self.rust_output.push_str(&format!(
                "{}struct {} {{}};\n",
                self.indent_str(),
                sanitize_rust_ident(mangled_name)
            ));
        } else {
            self.rust_output.push_str(&format!(
                "{}struct {} {{\n",
                self.indent_str(),
                sanitize_rust_ident(mangled_name)
            ));
            self.indent += 1;
            for (i, (field_name, field_ty)) in angas.iter().enumerate() {
                if i > 0 {
                    self.rust_output.push_str(",\n");
                }
                let rust_ty = self.type_name_to_rust_type_by_type(field_ty);
                self.rust_output.push_str(&format!(
                    "{}{}: {}",
                    self.indent_str(),
                    sanitize_rust_ident(field_name),
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
        let generic_params: Vec<String> = angas
            .iter()
            .filter_map(|(_, ty)| {
                if let DevvaniType::Samanya(p) = ty {
                    Some(p.clone())
                } else {
                    None
                }
            })
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
        return_type: Option<DevvaniType>,
        body: &[ASTNode],
    ) {
        self.flush_doc_comments();
        let mut rust_params = Vec::new();
        for param in params {
            let concrete_ty = if inference.contains_key(&param.type_name) {
                self.type_name_to_rust_type_by_type(inference.get(&param.type_name).unwrap())
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
            rust_params.push(format!(
                "{}: {}",
                sanitize_rust_ident(&param.name),
                type_str
            ));
        }

        let mut return_type_str = String::new();
        if let Some(rt) = return_type {
            match rt {
                DevvaniType::Phalam(success, error) => {
                    let success_rust = self.type_name_to_rust_type_by_type(&success);
                    let error_rust = self.type_name_to_rust_type_by_type(&error);
                    return_type_str = format!(" -> Result<{}, {}>", success_rust, error_rust);
                }
                other => {
                    let rust_ty = self.type_name_to_rust_type_by_type(&other);
                    if rust_ty != "auto" {
                        return_type_str = format!(" -> {}", rust_ty);
                    }
                }
            }
        }

        let line = format!(
            "{}pub fn {}({}){return_type_str} {{\n",
            self.indent_str(),
            sanitize_rust_ident(mangled_name),
            rust_params.join(", ")
        );
        self.rust_output.push_str(&line);

        self.indent += 1;
        self.current_inference = Some(inference.clone());
        self.emit_body(body).ok();
        self.current_inference = None;
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
            ASTNode::DhatuDef {
                body, return_type, ..
            } => {
                for stmt in body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
                if let Some(rt) = return_type {
                    self.walk_for_dhatu_instantiations(rt, set);
                }
            }
            ASTNode::YadiNode {
                sthiti,
                tarhi,
                anyatha,
            } => {
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
            ASTNode::NidanaNode {
                arogya_body,
                dosha_body,
                ..
            } => {
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
            ASTNode::NirmanaNode {
                dravya_name,
                values,
                ..
            } => {
                let angas = match self.type_checker.env.lookup(dravya_name) {
                    Some(sym) => match &sym.devvani_type {
                        DevvaniType::Dravya(_, angas) => angas.clone(),
                        _ => return,
                    },
                    None => return,
                };
                let has_samanya = angas
                    .iter()
                    .any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                if has_samanya {
                    let generic_params: Vec<String> = angas
                        .iter()
                        .filter_map(|(_, ty)| {
                            if let DevvaniType::Samanya(p) = ty {
                                Some(p.clone())
                            } else {
                                None
                            }
                        })
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
                    kriya, karta, karma, karana, sampradana, apadan, adhikarana,
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
            ASTNode::DharaNode { mulya, .. } => self.walk_for_dhatu_instantiations(mulya, set),
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
            ASTNode::SamyogaNode { body, .. } => {
                for stmt in body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::PraptiNode { handle, .. } => {
                self.walk_for_dhatu_instantiations(handle, set);
            }
            ASTNode::DutaBhejNode {
                sender, message, ..
            } => {
                self.walk_for_dhatu_instantiations(sender, set);
                self.walk_for_dhatu_instantiations(message, set);
            }
            ASTNode::DutaGrahanNode { receiver, .. } => {
                self.walk_for_dhatu_instantiations(receiver, set);
            }
            ASTNode::ManasNode { target, body, .. } => {
                self.walk_for_dhatu_instantiations(target, set);
                for stmt in body {
                    self.walk_for_dhatu_instantiations(stmt, set);
                }
            }
            ASTNode::ParinamaNode { mulyam, .. } => {
                self.walk_for_dhatu_instantiations(mulyam, set);
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

    fn walk_for_instantiations(
        &mut self,
        node: &ASTNode,
        set: &mut Vec<(String, String, Vec<(String, DevvaniType)>)>,
    ) {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::DhatuDef {
                body, return_type, ..
            } => {
                for stmt in body {
                    self.walk_for_instantiations(stmt, set);
                }
                if let Some(rt) = return_type {
                    self.walk_for_instantiations(rt, set);
                }
            }
            ASTNode::YadiNode {
                sthiti,
                tarhi,
                anyatha,
            } => {
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
            ASTNode::NidanaNode {
                arogya_body,
                dosha_body,
                ..
            } => {
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
            ASTNode::NirmanaNode {
                dravya_name,
                values,
                ..
            } => {
                let angas = match self.type_checker.env.lookup(dravya_name) {
                    Some(sym) => match &sym.devvani_type {
                        DevvaniType::Dravya(_, angas) => angas.clone(),
                        _ => return,
                    },
                    None => return,
                };
                let has_samanya = angas
                    .iter()
                    .any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                if has_samanya {
                    let generic_params: Vec<String> = angas
                        .iter()
                        .filter_map(|(_, ty)| {
                            if let DevvaniType::Samanya(p) = ty {
                                Some(p.clone())
                            } else {
                                None
                            }
                        })
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
                    let mangled =
                        self.mangled_generic_name(dravya_name, &generic_params, &inference);
                    let key = (dravya_name.clone(), mangled.clone(), resolved_angas.clone());
                    if !set.contains(&key) {
                        set.push((dravya_name.clone(), mangled, resolved_angas));
                    }
                }
            }
            ASTNode::VaakNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::AstiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::BhavatiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::DharaNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::VadatiNode { mulya, .. } => self.walk_for_instantiations(mulya, set),
            ASTNode::KriyaCall {
                karta,
                karma,
                karana,
                sampradana,
                apadan,
                adhikarana,
                ..
            } => {
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
            ASTNode::SamyogaNode { body, .. } => {
                for stmt in body {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::PraptiNode { handle, .. } => {
                self.walk_for_instantiations(handle, set);
            }
            ASTNode::DutaBhejNode {
                sender, message, ..
            } => {
                self.walk_for_instantiations(sender, set);
                self.walk_for_instantiations(message, set);
            }
            ASTNode::DutaGrahanNode { receiver, .. } => {
                self.walk_for_instantiations(receiver, set);
            }
            ASTNode::ManasNode { target, body, .. } => {
                self.walk_for_instantiations(target, set);
                for stmt in body {
                    self.walk_for_instantiations(stmt, set);
                }
            }
            ASTNode::ParinamaNode { mulyam, .. } => {
                self.walk_for_instantiations(mulyam, set);
            }
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
    use devvani_ast::{
        ASTNode, AngaField, Gana, KarakaParam, Lakara as AstLakara, Linga as AstLinga,
        NaamadheyaNode, Span, Vacana as AstVacana, Vibhakti, VikaraEntry, VikaraKind,
    };
    use devvani_compiler::Compiler;

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
        assert!(codegen.rust_source().contains("Ram"));
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
                    (
                        "sankhya".to_string(),
                        DevvaniType::Subject("Purnaank".to_string()),
                    ),
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
        assert_eq!(
            codegen.rust_source().trim(),
            "manushya { naama: \"raamah\", sankhya: 25 }"
        );
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
                    (
                        "sankhya1".to_string(),
                        DevvaniType::Subject("Purnaank".to_string()),
                    ),
                    (
                        "sankhya2".to_string(),
                        DevvaniType::Subject("Purnaank".to_string()),
                    ),
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
        codegen
            .type_checker
            .env
            .define("shunya", DevvaniType::Dravya("shunya".to_string(), vec![]));
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
        let angas = vec![AngaField {
            name: "inner".to_string(),
            type_name: "outer".to_string(),
            span: dummy_span(),
        }];
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
            is_exported: false,
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
                    (
                        "sankhya".to_string(),
                        DevvaniType::Subject("Purnaank".to_string()),
                    ),
                ],
            ),
        );
        codegen.type_checker.env.define(
            "roga",
            DevvaniType::Dravya(
                "roga".to_string(),
                vec![("naama".to_string(), DevvaniType::Vaak)],
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
            is_exported: false,
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
                    (
                        "sankhya".to_string(),
                        DevvaniType::Subject("Purnaank".to_string()),
                    ),
                ],
            ),
        );
        codegen.type_checker.env.define(
            "roga",
            DevvaniType::Dravya(
                "roga".to_string(),
                vec![("naama".to_string(), DevvaniType::Vaak)],
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
            is_exported: false,
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
            is_exported: false,
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
            is_exported: false,
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
        assert_eq!(
            count, 1,
            "expected exactly one Peti__String struct definition, got {}",
            count
        );
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
                    is_exported: false,
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
        assert!(
            output.contains("pub fn pratirupa__String(vastu: String) -> Result<String, String> {")
        );
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
                    is_exported: false,
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
        assert_eq!(
            count, 1,
            "expected exactly one pratirupa__String definition, got {}",
            count
        );
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
                    is_exported: false,
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
        assert!(
            output.contains("pub fn pratirupa__String(vastu: String) -> Result<String, String> {")
        );
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
                    is_exported: false,
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
        assert!(output
            .contains("pub fn yugala__String__i64(a: String, b: i64) -> Result<String, String> {"));
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
                    is_exported: false,
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
                    is_exported: false,
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

    // ===== Anumana (Type Inference) Codegen Tests =====

    #[test]
    fn test_dhara_inferred_integer_literal_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["x".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::PurnaankLiteral {
                    value: 5,
                    span: dummy_span(),
                }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("let x: i64 = 5;"),
            "expected explicit i64 type annotation, got:\n{}",
            output
        );
    }

    #[test]
    fn test_dhara_inferred_string_literal_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["s".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "hello".to_string(),
                    span: dummy_span(),
                }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("let s: String = \"hello\";"),
            "expected explicit String type annotation, got:\n{}",
            output
        );
    }

    #[test]
    fn test_dhara_chained_inference_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DharaNode {
                    naamas: vec!["x".to_string()],
                    type_name: None,
                    mulya: Box::new(ASTNode::PurnaankLiteral {
                        value: 5,
                        span: dummy_span(),
                    }),
                    is_mutable: false,
                    span: dummy_span(),
                },
                ASTNode::DharaNode {
                    naamas: vec!["y".to_string()],
                    type_name: None,
                    mulya: Box::new(ASTNode::Nama {
                        base: "x".to_string(),
                        vibhakti: devvani_ast::Vibhakti::Prathama,
                        linga: AstLinga::Pullinga,
                        vacana: AstVacana::Eka,
                        span: dummy_span(),
                    }),
                    is_mutable: false,
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("let x: i64 = 5;"),
            "expected x to have explicit i64 type, got:\n{}",
            output
        );
        assert!(
            output.contains("let y: i64 ="),
            "expected y to have explicit i64 type, got:\n{}",
            output
        );
    }

    #[test]
    fn test_dhatu_def_inferred_return_type_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DhatuDef {
                name: "get_num".to_string(),
                generic_params: vec![],
                lakara: devvani_ast::Lakara::Lat,
                gana: devvani_ast::Gana::Bhvadi,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                params: vec![],
                upasargas: vec![],
                return_karaka: None,
                return_type: None,
                body: vec![ASTNode::PurnaankLiteral {
                    value: 42,
                    span: dummy_span(),
                }],
                is_exported: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("pub fn get_num() -> i64 {"),
            "expected inferred i64 return type, got:\n{}",
            output
        );
    }

    #[test]
    fn test_dhatu_def_inferred_return_type_no_expression() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DhatuDef {
                name: "do_nothing".to_string(),
                generic_params: vec![],
                lakara: devvani_ast::Lakara::Lat,
                gana: devvani_ast::Gana::Bhvadi,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                params: vec![],
                upasargas: vec![],
                return_karaka: None,
                return_type: None,
                body: vec![],
                is_exported: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            !output.contains("->"),
            "expected no return type for unit function, got:\n{}",
            output
        );
        assert!(
            output.contains("pub fn do_nothing() {"),
            "expected function without return type, got:\n{}",
            output
        );
    }

    #[test]
    fn test_explicit_dhara_node_codegen_unchanged() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["x".to_string()],
                type_name: Some("sankhya".to_string()),
                mulya: Box::new(ASTNode::PurnaankLiteral {
                    value: 5,
                    span: dummy_span(),
                }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("let x: i64 = 5;"),
            "explicit-type DharaNode should still emit explicit type, got:\n{}",
            output
        );
    }

    #[test]
    fn test_explicit_dhatu_def_return_type_codegen_unchanged() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DhatuDef {
                name: "bhojan_dhatu".to_string(),
                generic_params: vec![],
                lakara: devvani_ast::Lakara::Lat,
                gana: devvani_ast::Gana::Bhvadi,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                params: vec![],
                upasargas: vec![],
                return_karaka: None,
                return_type: Some(Box::new(ASTNode::PhalamType {
                    success_type: "sankhya".to_string(),
                    error_type: "vaak".to_string(),
                    span: dummy_span(),
                })),
                body: vec![],
                is_exported: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("pub fn bhojan_dhatu() -> Result<i64, String> {"),
            "explicit-return-type DhatuDef should still emit explicit return type, got:\n{}",
            output
        );
    }

    #[test]
    fn test_generic_dhatu_inferred_dhara_inside_body() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "make_pair".to_string(),
                    generic_params: vec!["T".to_string()],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![devvani_ast::KarakaParam {
                        name: "x".to_string(),
                        type_name: "T".to_string(),
                        role: devvani_ast::KarakaRole::Karta,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        vibhakti: devvani_ast::Vibhakti::Prathama,
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![
                        ASTNode::DharaNode {
                            naamas: vec!["local".to_string()],
                            type_name: None,
                            mulya: Box::new(ASTNode::Nama {
                                base: "x".to_string(),
                                vibhakti: devvani_ast::Vibhakti::Prathama,
                                linga: AstLinga::Pullinga,
                                vacana: AstVacana::Eka,
                                span: dummy_span(),
                            }),
                            is_mutable: false,
                            span: dummy_span(),
                        },
                        ASTNode::Nama {
                            base: "local".to_string(),
                            vibhakti: devvani_ast::Vibhakti::Prathama,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        },
                    ],
                    is_exported: false,
                    span: dummy_span(),
                },
                ASTNode::KriyaCall {
                    karta: None,
                    kriya: "make_pair".to_string(),
                    karma: vec![ASTNode::PurnaankLiteral {
                        value: 10,
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
        assert!(
            output.contains("pub fn make_pair__i64(x: i64) -> i64 {"),
            "expected monomorphized generic function with inferred return type, got:\n{}",
            output
        );
        assert!(
            output.contains("let local: i64 ="),
            "expected inferred-type dhara inside generic body to have concrete type, got:\n{}",
            output
        );
    }

    // ===== Concurrency (Samyoga / Prapti / Duta / Manas) Codegen Tests =====

    #[test]
    fn test_samyoga_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::SamyogaNode {
            body: vec![ASTNode::PurnaankLiteral {
                value: 42,
                span: dummy_span(),
            }],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("std::thread::spawn(move || {"));
        assert!(output.contains("42"));
        assert!(output.contains("})"));
    }

    #[test]
    fn test_samyoga_as_dhara_initializer_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["h".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::SamyogaNode {
                    body: vec![ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("let h = std::thread::spawn(move || {"));
        assert!(output.contains("1"));
        assert!(output.contains("})"));
    }

    #[test]
    fn test_prapti_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::PraptiNode {
            handle: Box::new(ASTNode::Nama {
                base: "h".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "h.join().unwrap()");
    }

    #[test]
    fn test_duta_banaa_single_binding_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["channel".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::DutaBanaaNode { span: dummy_span() }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        assert_eq!(
            codegen.rust_source().trim(),
            "let channel = std::sync::mpsc::channel();"
        );
    }

    #[test]
    fn test_duta_banaa_tuple_destructuring_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DharaNode {
                naamas: vec!["bhejaka".to_string(), "grahaka".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::DutaBanaaNode { span: dummy_span() }),
                is_mutable: false,
                span: dummy_span(),
            }],
        };
        assert!(codegen.generate(&program).is_ok());
        assert_eq!(
            codegen.rust_source().trim(),
            "let (bhejaka, grahaka) = std::sync::mpsc::channel();"
        );
    }

    #[test]
    fn test_duta_bhej_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DutaBhejNode {
            sender: Box::new(ASTNode::Nama {
                base: "bhejaka".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            message: Box::new(ASTNode::VaakLiteral {
                value: "sandesha".to_string(),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(
            codegen.rust_source().trim(),
            "bhejaka.send(\"sandesha\").unwrap();"
        );
    }

    #[test]
    fn test_duta_grahan_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::DutaGrahanNode {
            receiver: Box::new(ASTNode::Nama {
                base: "grahaka".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "grahaka.recv().unwrap()");
    }

    #[test]
    fn test_manas_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::ManasNode {
            target: Box::new(ASTNode::Nama {
                base: "lock".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::PurnaankLiteral {
                    value: 7,
                    span: dummy_span(),
                }),
            }],
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("let mut lock = lock.lock().unwrap();"));
        assert!(output.contains("println!(\"{:?}\", 7);"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_concurrency_combined_codegen() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DharaNode {
                    naamas: vec!["bhejaka".to_string(), "grahaka".to_string()],
                    type_name: None,
                    mulya: Box::new(ASTNode::DutaBanaaNode { span: dummy_span() }),
                    is_mutable: false,
                    span: dummy_span(),
                },
                ASTNode::DharaNode {
                    naamas: vec!["h".to_string()],
                    type_name: None,
                    mulya: Box::new(ASTNode::SamyogaNode {
                        body: vec![
                            ASTNode::DutaBhejNode {
                                sender: Box::new(ASTNode::Nama {
                                    base: "bhejaka".to_string(),
                                    vibhakti: devvani_ast::Vibhakti::Prathama,
                                    linga: AstLinga::Pullinga,
                                    vacana: AstVacana::Eka,
                                    span: dummy_span(),
                                }),
                                message: Box::new(ASTNode::VaakLiteral {
                                    value: "hello".to_string(),
                                    span: dummy_span(),
                                }),
                                span: dummy_span(),
                            },
                            ASTNode::PurnaankLiteral {
                                value: 99,
                                span: dummy_span(),
                            },
                        ],
                        span: dummy_span(),
                    }),
                    is_mutable: false,
                    span: dummy_span(),
                },
                ASTNode::DharaNode {
                    naamas: vec!["result".to_string()],
                    type_name: None,
                    mulya: Box::new(ASTNode::PraptiNode {
                        handle: Box::new(ASTNode::Nama {
                            base: "h".to_string(),
                            vibhakti: devvani_ast::Vibhakti::Prathama,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }),
                    is_mutable: false,
                    span: dummy_span(),
                },
                ASTNode::DutaGrahanNode {
                    receiver: Box::new(ASTNode::Nama {
                        base: "grahaka".to_string(),
                        vibhakti: devvani_ast::Vibhakti::Prathama,
                        linga: AstLinga::Pullinga,
                        vacana: AstVacana::Eka,
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
            ],
        };
        assert!(codegen.generate(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("let (bhejaka, grahaka) = std::sync::mpsc::channel();"));
        assert!(output.contains("let h = std::thread::spawn(move || {"));
        assert!(output.contains("bhejaka.send(\"hello\").unwrap();"));
        assert!(output.contains("let result: i64 = h.join().unwrap();"));
        assert!(output.contains("grahaka.recv().unwrap()"));
    }

    // ===== Pariṇāma (Pipeline) Codegen Tests =====

    #[test]
    fn test_parinama_three_dhatu_nonfallible() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "inc".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![ASTNode::YogaNode {
                        vama: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        dakshina: Box::new(ASTNode::PurnaankLiteral {
                            value: 1,
                            span: dummy_span(),
                        }),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "double".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![ASTNode::YogaNode {
                        vama: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        dakshina: Box::new(ASTNode::PurnaankLiteral {
                            value: 2,
                            span: dummy_span(),
                        }),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "triple".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![ASTNode::YogaNode {
                        vama: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        dakshina: Box::new(ASTNode::PurnaankLiteral {
                            value: 3,
                            span: dummy_span(),
                        }),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let _ = codegen.type_checker.check_program(&program);

        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            dhatus: vec![
                "inc".to_string(),
                "double".to_string(),
                "triple".to_string(),
            ],
            span: dummy_span(),
        };
        assert!(codegen.emit(&parinama).is_ok());
        assert_eq!(codegen.rust_source().trim(), "triple(double(inc(5)))");
    }

    #[test]
    fn test_parinama_single_dhatu_nonfallible() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DhatuDef {
                name: "inc".to_string(),
                generic_params: vec![],
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
                    span: dummy_span(),
                    type_name: "sankhya".to_string(),
                }],
                upasargas: vec![],
                return_karaka: None,
                return_type: None,
                body: vec![ASTNode::YogaNode {
                    vama: Box::new(ASTNode::Nama {
                        base: "n".to_string(),
                        vibhakti: Vibhakti::Dvitiya,
                        linga: AstLinga::Pullinga,
                        vacana: AstVacana::Eka,
                        span: dummy_span(),
                    }),
                    dakshina: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                }],
                is_exported: false,
                span: dummy_span(),
            }],
        };
        let _ = codegen.type_checker.check_program(&program);

        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            dhatus: vec!["inc".to_string()],
            span: dummy_span(),
        };
        assert!(codegen.emit(&parinama).is_ok());
        assert_eq!(codegen.rust_source().trim(), "inc(5)");
    }

    #[test]
    fn test_parinama_all_fallible() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "fa".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "sankhya".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![ASTNode::ArogyaNode {
                        value: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "fb".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "sankhya".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![ASTNode::ArogyaNode {
                        value: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let _ = codegen.type_checker.check_program(&program);

        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            dhatus: vec!["fa".to_string(), "fb".to_string()],
            span: dummy_span(),
        };
        assert!(codegen.emit(&parinama).is_ok());
        assert_eq!(codegen.rust_source().trim(), "fa(5).and_then(|v0| fb(v0))");
    }

    #[test]
    fn test_parinama_mixed_fallible() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::DhatuDef {
                    name: "fa".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: Some(Box::new(ASTNode::PhalamType {
                        success_type: "sankhya".to_string(),
                        error_type: "vaak".to_string(),
                        span: dummy_span(),
                    })),
                    body: vec![ASTNode::ArogyaNode {
                        value: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "g".to_string(),
                    generic_params: vec![],
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
                        span: dummy_span(),
                        type_name: "sankhya".to_string(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![ASTNode::YogaNode {
                        vama: Box::new(ASTNode::Nama {
                            base: "n".to_string(),
                            vibhakti: Vibhakti::Dvitiya,
                            linga: AstLinga::Pullinga,
                            vacana: AstVacana::Eka,
                            span: dummy_span(),
                        }),
                        dakshina: Box::new(ASTNode::PurnaankLiteral {
                            value: 1,
                            span: dummy_span(),
                        }),
                    }],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let _ = codegen.type_checker.check_program(&program);

        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            dhatus: vec!["fa".to_string(), "g".to_string()],
            span: dummy_span(),
        };
        assert!(codegen.emit(&parinama).is_ok());
        assert_eq!(
            codegen.rust_source().trim(),
            "fa(5).and_then(|v0| Ok(g(v0)))"
        );
    }

    #[test]
    fn test_parinama_single_dhatu_fallible() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let program = ASTNode::KaryakramNode {
            shareera: vec![ASTNode::DhatuDef {
                name: "fa".to_string(),
                generic_params: vec![],
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
                    span: dummy_span(),
                    type_name: "sankhya".to_string(),
                }],
                upasargas: vec![],
                return_karaka: None,
                return_type: Some(Box::new(ASTNode::PhalamType {
                    success_type: "sankhya".to_string(),
                    error_type: "vaak".to_string(),
                    span: dummy_span(),
                })),
                body: vec![ASTNode::ArogyaNode {
                    value: Box::new(ASTNode::Nama {
                        base: "n".to_string(),
                        vibhakti: Vibhakti::Dvitiya,
                        linga: AstLinga::Pullinga,
                        vacana: AstVacana::Eka,
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }],
                is_exported: false,
                span: dummy_span(),
            }],
        };
        let _ = codegen.type_checker.check_program(&program);

        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            dhatus: vec!["fa".to_string()],
            span: dummy_span(),
        };
        assert!(codegen.emit(&parinama).is_ok());
        assert_eq!(codegen.rust_source().trim(), "fa(5)");
    }

    #[test]
    fn test_parikshaa_plain_emits_test_without_should_panic() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::ParikshaaNode {
            name: "my-test".to_string(),
            body: vec![ASTNode::NigamanaNode {
                expr: Box::new(ASTNode::SamaNode {
                    vama: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                    dakshina: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                }),
                span: dummy_span(),
            }],
            is_tarka: false,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("#[test]"),
            "expected #[test] in:\n{}",
            output
        );
        assert!(
            !output.contains("#[should_panic]"),
            "did not expect #[should_panic] in:\n{}",
            output
        );
        assert!(
            output.contains("fn my_test()"),
            "expected fn my_test in:\n{}",
            output
        );
    }

    #[test]
    fn test_parikshaa_tarka_emits_should_panic() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::ParikshaaNode {
            name: "tarka-test".to_string(),
            body: vec![ASTNode::NigamanaNode {
                expr: Box::new(ASTNode::SamaNode {
                    vama: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                    dakshina: Box::new(ASTNode::PurnaankLiteral {
                        value: 1,
                        span: dummy_span(),
                    }),
                }),
                span: dummy_span(),
            }],
            is_tarka: true,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("#[test]"),
            "expected #[test] in:\n{}",
            output
        );
        assert!(
            output.contains("#[should_panic]"),
            "expected #[should_panic] in:\n{}",
            output
        );
        assert!(
            output.contains("fn tarka_test()"),
            "expected fn tarka_test in:\n{}",
            output
        );
    }

    #[test]
    fn test_nigamana_emits_assert() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::NigamanaNode {
            expr: Box::new(ASTNode::SamaNode {
                vama: Box::new(ASTNode::PurnaankLiteral {
                    value: 1,
                    span: dummy_span(),
                }),
                dakshina: Box::new(ASTNode::PurnaankLiteral {
                    value: 1,
                    span: dummy_span(),
                }),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "assert!(1 == 1);");
    }

    #[test]
    fn test_sadrishya_nigamana_emits_assert_eq() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::SadrishyaNigamanaNode {
            left: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            right: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "assert_eq!(x, 5);");
    }

    #[test]
    fn test_asadrishya_nigamana_emits_assert_ne() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::AsadrishyaNigamanaNode {
            left: Box::new(ASTNode::Nama {
                base: "y".to_string(),
                vibhakti: Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            }),
            right: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        assert_eq!(codegen.rust_source().trim(), "assert_ne!(y, \"hello\");");
    }

    #[test]
    fn test_parikshaa_hyphenated_name_sanitized() {
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        let node = ASTNode::ParikshaaNode {
            name: "my-hyphenated-test".to_string(),
            body: vec![],
            is_tarka: false,
            span: dummy_span(),
        };
        assert!(codegen.emit(&node).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("fn my_hyphenated_test()"),
            "expected sanitized fn name in:\n{}",
            output
        );
    }

    // ===== Documentation (Vritti / Bhashya / Tippani) Codegen Tests =====

    #[test]
    fn test_vritti_only_emits_doc_comment_before_fn() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::VrittiNode {
                    text: "short doc".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "my_func".to_string(),
                    generic_params: vec![],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("/// short doc"),
            "expected vritti doc comment in:\n{}",
            output
        );
        assert!(output.contains("pub fn my_func()"));
        let vritti_pos = output.find("/// short doc").unwrap();
        let fn_pos = output.find("pub fn my_func").unwrap();
        assert!(
            vritti_pos < fn_pos,
            "vritti doc comment must appear before fn definition"
        );
    }

    #[test]
    fn test_vritti_with_two_tippani_emits_parameters_section() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::VrittiNode {
                    text: "adds two numbers".to_string(),
                    span: dummy_span(),
                },
                ASTNode::TippaniNode {
                    text: "the left operand".to_string(),
                    param_name: "x".to_string(),
                    span: dummy_span(),
                },
                ASTNode::TippaniNode {
                    text: "the right operand".to_string(),
                    param_name: "y".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "add".to_string(),
                    generic_params: vec![],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![
                        KarakaParam {
                            name: "x".to_string(),
                            role: KarakaRole::Karma,
                            vibhakti: Vibhakti::Dvitiya,
                            is_borrowed: false,
                            is_mutable_borrow: false,
                            type_name: "sankhya".to_string(),
                            span: dummy_span(),
                        },
                        KarakaParam {
                            name: "y".to_string(),
                            role: KarakaRole::Karma,
                            vibhakti: Vibhakti::Dvitiya,
                            is_borrowed: false,
                            is_mutable_borrow: false,
                            type_name: "sankhya".to_string(),
                            span: dummy_span(),
                        },
                    ],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("/// adds two numbers"));
        assert!(output.contains("/// # Parameters"));
        assert!(output.contains("/// * x - the left operand"));
        assert!(output.contains("/// * y - the right operand"));
        assert!(output.contains("pub fn add(x: i64, y: i64)"));
        let vritti_pos = output.find("/// adds two numbers").unwrap();
        let params_pos = output.find("/// # Parameters").unwrap();
        let fn_pos = output.find("pub fn add").unwrap();
        assert!(
            vritti_pos < params_pos,
            "vritti must appear before Parameters section"
        );
        assert!(
            params_pos < fn_pos,
            "Parameters section must appear before fn definition"
        );
    }

    #[test]
    fn test_tippani_only_no_vritti_emits_parameters_section() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::TippaniNode {
                    text: "the divisor".to_string(),
                    param_name: "d".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "divide".to_string(),
                    generic_params: vec![],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![KarakaParam {
                        name: "d".to_string(),
                        role: KarakaRole::Karma,
                        vibhakti: Vibhakti::Dvitiya,
                        is_borrowed: false,
                        is_mutable_borrow: false,
                        type_name: "sankhya".to_string(),
                        span: dummy_span(),
                    }],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("/// # Parameters"));
        assert!(output.contains("/// * d - the divisor"));
        assert!(output.contains("pub fn divide(d: i64)"));
        if let Some(params_pos) = output.find("/// # Parameters") {
            let preceding = &output[..params_pos];
            assert!(
                !preceding.ends_with("/// \n"),
                "no stray blank vritti line before Parameters section when there is no vritti"
            );
        }
    }

    #[test]
    fn test_two_bhashya_lines_emitted_at_file_top() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::BhashyaNode {
                    text: "module doc line 1".to_string(),
                    span: dummy_span(),
                },
                ASTNode::BhashyaNode {
                    text: "module doc line 2".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "foo".to_string(),
                    generic_params: vec![],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.starts_with("//! module doc line 1\n//! module doc line 2"));
        assert!(output.contains("pub fn foo()"));
        let bhashya1_pos = output.find("//! module doc line 1").unwrap();
        let bhashya2_pos = output.find("//! module doc line 2").unwrap();
        let fn_pos = output.find("pub fn foo").unwrap();
        assert!(
            bhashya1_pos < bhashya2_pos,
            "first bhashya must appear before second"
        );
        assert!(
            bhashya2_pos < fn_pos,
            "bhashya lines must appear before fn definition"
        );
    }

    #[test]
    fn test_dravya_def_with_vritti_emits_doc_comment() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::VrittiNode {
                    text: "struct docs".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DravyaDef {
                    name: "Person".to_string(),
                    generic_params: vec![],
                    angas: vec![AngaField {
                        name: "naama".to_string(),
                        type_name: "vaak".to_string(),
                        span: dummy_span(),
                    }],
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("/// struct docs"),
            "expected vritti doc comment on struct in:\n{}",
            output
        );
        assert!(output.contains("struct Person {"));
        let doc_pos = output.find("/// struct docs").unwrap();
        let struct_pos = output.find("struct Person").unwrap();
        assert!(
            doc_pos < struct_pos,
            "doc comment must appear before struct definition"
        );
    }

    #[test]
    fn test_multiline_vritti_emits_multiple_doc_lines() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::VrittiNode {
                    text: "line one\nline two\nline three".to_string(),
                    span: dummy_span(),
                },
                ASTNode::DhatuDef {
                    name: "my_func".to_string(),
                    generic_params: vec![],
                    lakara: devvani_ast::Lakara::Lat,
                    gana: devvani_ast::Gana::Bhvadi,
                    linga: AstLinga::Pullinga,
                    vacana: AstVacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    return_type: None,
                    body: vec![],
                    is_exported: false,
                    span: dummy_span(),
                },
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(output.contains("/// line one"));
        assert!(output.contains("/// line two"));
        assert!(output.contains("/// line three"));
        assert!(
            !output.contains("/// line one\nline two"),
            "each line must have its own /// prefix, not one /// with embedded newline"
        );
    }

    #[test]
    fn test_doc_e2e_generated_rust_compiles() {
        use std::process::Command;
        use tempfile::TempDir;

        let source = "bhashya \"Library for math operations\"।\n\
                      vritti \"Increments the input value\"।\n\
                      dhātu increment n karoti । n yoga 1 iti ।\n";

        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let src_path = tmp_dir.path().join("test_docs.dvn");
        std::fs::write(&src_path, source).expect("failed to write devvani source");

        let rust_code = Compiler::new(&src_path)
            .compile()
            .expect("compilation failed");

        assert!(rust_code.contains("//! Library for math operations"));
        assert!(rust_code.contains("/// Increments the input value"));
        assert!(rust_code.contains("pub fn increment"));

        let rust_path = tmp_dir.path().join("test_docs.rs");
        let out_path = tmp_dir.path().join("test_docs_out");
        let wrapped = format!("fn main() {{\n{}\n}}", rust_code);
        std::fs::write(&rust_path, wrapped).expect("failed to write temp rust file");

        let status = Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg("--crate-type")
            .arg("bin")
            .arg("--crate-name")
            .arg("test_docs_verify")
            .arg(&rust_path)
            .arg("-o")
            .arg(&out_path)
            .output()
            .expect("failed to run rustc");

        let stderr = String::from_utf8_lossy(&status.stderr);
        assert!(
            status.status.success(),
            "rustc failed for doc e2e:\n{}",
            stderr
        );
    }

    // ===== Versioning (Mrittika / Vikara) Codegen Tests =====

    fn mrittika_node(package_name: &str, version: &str, vikaras: Vec<VikaraEntry>) -> ASTNode {
        ASTNode::MrittikaNode {
            package_name: package_name.to_string(),
            naamadheya: NaamadheyaNode {
                version_string: version.to_string(),
                span: dummy_span(),
            },
            vikaras,
            span: dummy_span(),
        }
    }

    fn vikara_entry(kind: VikaraKind, desc: &str) -> VikaraEntry {
        VikaraEntry {
            kind,
            description: desc.to_string(),
            span: dummy_span(),
        }
    }

    fn dummy_dhatu(name: &str) -> ASTNode {
        ASTNode::DhatuDef {
            name: name.to_string(),
            generic_params: vec![],
            lakara: devvani_ast::Lakara::Lat,
            gana: devvani_ast::Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            return_type: None,
            body: vec![],
            is_exported: false,
            span: dummy_span(),
        }
    }

    #[test]
    fn test_mrittika_zero_vikaras_emits_no_history_section() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![mrittika_node("my_pkg", "1.2.0", vec![])],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("//! # Devvani Package Metadata (मृत्तिका)"),
            "expected metadata header in:\n{}",
            output
        );
        assert!(
            output.contains("//! - Package: my_pkg"),
            "expected package line in:\n{}",
            output
        );
        assert!(
            output.contains("//! - Version (नामधेय): 1.2.0"),
            "expected version line in:\n{}",
            output
        );
        assert!(
            !output.contains("## Vikara History"),
            "Vikara History section should be absent when there are zero vikaras:\n{}",
            output
        );
    }

    #[test]
    fn test_mrittika_only_sukshma_emits_correct_tags() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![mrittika_node(
                "my_pkg",
                "0.1.0",
                vec![
                    vikara_entry(VikaraKind::Sukshma, "fixed typo"),
                    vikara_entry(VikaraKind::Sukshma, "more fixes"),
                ],
            )],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("//! - [SUKSHMA] fixed typo"),
            "expected SUKSHMA tag for first entry in:\n{}",
            output
        );
        assert!(
            output.contains("//! - [SUKSHMA] more fixes"),
            "expected SUKSHMA tag for second entry in:\n{}",
            output
        );
        let pos1 = output.find("//! - [SUKSHMA] fixed typo").unwrap();
        let pos2 = output.find("//! - [SUKSHMA] more fixes").unwrap();
        assert!(pos1 < pos2, "vikara entries must preserve source order");
    }

    #[test]
    fn test_mrittika_mixed_order_preserves_source_order() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![mrittika_node(
                "my_pkg",
                "0.2.0",
                vec![
                    vikara_entry(VikaraKind::SatyaBheda, "breaking change"),
                    vikara_entry(VikaraKind::Sukshma, "internal fix"),
                    vikara_entry(VikaraKind::Sthula, "new feature"),
                ],
            )],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        let breaking_pos = output
            .find("//! - [SATYA-BHEDA] breaking change")
            .unwrap();
        let fix_pos = output.find("//! - [SUKSHMA] internal fix").unwrap();
        let feat_pos = output.find("//! - [STHULA] new feature").unwrap();
        assert!(
            breaking_pos < fix_pos && fix_pos < feat_pos,
            "vikara entries must appear in source order, not grouped by kind:\n{}",
            output
        );
        assert!(
            !output.contains("//! - [SUKSHMA] internal fix\n//! - [SUKSHMA] more"),
            "entries must not be regrouped by kind"
        );
    }

    #[test]
    fn test_no_mrittika_emits_no_metadata() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![dummy_dhatu("foo")],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            !output.contains("//! # Devvani Package Metadata"),
            "no metadata should be emitted when mrittika is absent:\n{}",
            output
        );
    }

    #[test]
    fn test_mrittika_after_bhashya_metadata_after_bhashya() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![
                ASTNode::BhashyaNode {
                    text: "Library for math operations".to_string(),
                    span: dummy_span(),
                },
                mrittika_node("my_pkg", "1.0.0", vec![]),
                dummy_dhatu("foo"),
            ],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        let bhashya_pos = output.find("//! Library for math operations").unwrap();
        let metadata_pos = output
            .find("//! # Devvani Package Metadata")
            .unwrap();
        let fn_pos = output.find("pub fn foo").unwrap();
        assert!(
            bhashya_pos < metadata_pos,
            "metadata block must appear immediately after Bhashya"
        );
        assert!(
            metadata_pos < fn_pos,
            "metadata block must appear before function definitions"
        );
    }

    #[test]
    fn test_mrittika_description_with_newline_split_into_doc_lines() {
        let program = ASTNode::KaryakramNode {
            shareera: vec![mrittika_node(
                "my_pkg",
                "1.0.0",
                vec![vikara_entry(VikaraKind::Sukshma, "line one\nline two")],
            )],
        };
        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        assert!(codegen.emit(&program).is_ok());
        let output = codegen.rust_source();
        assert!(
            output.contains("//! - [SUKSHMA] line one"),
            "first line of multi-line description must get //! prefix"
        );
        assert!(
            output.contains("//! line two"),
            "second line of multi-line description must get //! prefix"
        );
        assert!(
            !output.contains("//! - [SUKSHMA] line one\nline two"),
            "each line of a multi-line description must have its own //! prefix"
        );
    }

    #[test]
    fn test_mrittika_e2e_generated_rust_compiles() {
        use std::process::Command;
        use tempfile::TempDir;

        let source =
            "bhashya \"A versioned library\"।\n\
             mrittika \"versioned-lib\" {\n\
                 naamadheya \"0.2.0\"।\n\
                 satya-bheda \"removed deprecated API\"।\n\
                 sukshma-vikara \"fixed a bug\"।\n\
                 sthula-vikara \"added new feature\"।\n\
             }\n\
             dhātu increment n karoti । n yoga 1 iti ।\n";

        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let src_path = tmp_dir.path().join("test_mrittika_e2e.dvn");
        std::fs::write(&src_path, source).expect("failed to write devvani source");

        let rust_code = Compiler::new(&src_path)
            .compile()
            .expect("compilation failed");

        assert!(
            rust_code.contains("//! # Devvani Package Metadata (मृत्तिका)"),
            "expected metadata header in generated code:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains("//! - Package: versioned-lib"),
            "expected package name in:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains("//! - Version (नामधेय): 0.2.0"),
            "expected version in:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains("//! - [SATYA-BHEDA] removed deprecated API"),
            "expected satya-bheda entry in:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains("//! - [SUKSHMA] fixed a bug"),
            "expected sukshma entry in:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains("//! - [STHULA] added new feature"),
            "expected sthula entry in:\n{}",
            rust_code
        );
        // Verify source order is preserved (not grouped by kind)
        let sb = rust_code
            .find("//! - [SATYA-BHEDA] removed deprecated API")
            .unwrap();
        let sm = rust_code.find("//! - [SUKSHMA] fixed a bug").unwrap();
        let st = rust_code
            .find("//! - [STHULA] added new feature")
            .unwrap();
        assert!(
            sb < sm && sm < st,
            "vikara entries must preserve source order in:\n{}",
            rust_code
        );
        // Bhashya must appear before the metadata block
        let bhashya_pos = rust_code
            .find("//! A versioned library")
            .unwrap();
        let metadata_pos = rust_code
            .find("//! # Devvani Package Metadata")
            .unwrap();
        assert!(
            bhashya_pos < metadata_pos,
            "Bhashya must appear before metadata block"
        );

        // Verify the generated Rust compiles with rustc
        let rust_path = tmp_dir.path().join("test_mrittika_e2e.rs");
        let out_path = tmp_dir.path().join("test_mrittika_e2e_out");
        let wrapped = format!("fn main() {{\n{}\n}}", rust_code);
        std::fs::write(&rust_path, wrapped).expect("failed to write temp rust file");

        let status = Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg("--crate-type")
            .arg("bin")
            .arg("--crate-name")
            .arg("test_mrittika_e2e_verify")
            .arg(&rust_path)
            .arg("-o")
            .arg(&out_path)
            .output()
            .expect("failed to run rustc");

        let stderr = String::from_utf8_lossy(&status.stderr);
        assert!(
            status.status.success(),
            "rustc failed for mrittika e2e:\n{}",
            stderr
        );
    }
}
