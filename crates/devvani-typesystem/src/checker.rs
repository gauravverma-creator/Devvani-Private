use crate::{lakara::*, linga::*, symbol::*, type_env::TypeEnv, vacana::*, vibhakti::*};
use devvani_ast::ASTNode;
use std::fmt;

#[derive(Debug, Clone)]
pub enum TypeCheckError {
    NaamaApraapta(String),
    PrakaaraVaisamya {
        expected: String,
        found: String,
    },
    SatyaasatyaApekshita(String),
    PrakaaraAsangata(String),
    AnavasthaDosha {
        dhatu_name: String,
    },
    PanktiAsangata {
        expected: DevvaniType,
        found: DevvaniType,
    },
    VinyasaAprayukta {
        found: DevvaniType,
    },
    VinyasaSimaLanghana {
        index: usize,
        len: usize,
    },
    KramashahAprayukta {
        found: DevvaniType,
    },
    AvaliAsangata {
        expected: DevvaniType,
        found: DevvaniType,
    },
    PrakshepaAprayukta {
        found: DevvaniType,
    },
    ApakarshanaAprayukta {
        found: DevvaniType,
    },
    SamavayaAprayukta {
        found: String,
    },
    DravyaApariyata {
        name: String,
    },
    AngaApraapya {
        dravya_name: String,
        anga_name: String,
    },
    NirmanaAsangati {
        dravya_name: String,
        expected_count: usize,
        found_count: usize,
        anga_name: String,
        position: usize,
        expected_type: DevvaniType,
        found_type: DevvaniType,
    },
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::NaamaApraapta(name) => write!(f, "Naama-apraapta: {}", name),
            TypeCheckError::PrakaaraVaisamya { expected, found } => {
                write!(
                    f,
                    "Prakaara-vaisamya: expected {}, found {}",
                    expected, found
                )
            }
            TypeCheckError::SatyaasatyaApekshita(msg) => {
                write!(f, "Satyaasatya-apekshita: {}", msg)
            }
            TypeCheckError::PrakaaraAsangata(msg) => write!(f, "Prakaara-asangata: {}", msg),
            TypeCheckError::AnavasthaDosha { dhatu_name } => {
                write!(
                    f,
                    "Anavastha-dosha: '{}' has no reachable base case",
                    dhatu_name
                )
            }
            TypeCheckError::PanktiAsangata { expected, found } => {
                write!(
                    f,
                    "Pankti-asangata: expected {:?}, found {:?}",
                    expected, found
                )
            }
            TypeCheckError::VinyasaAprayukta { found } => {
                write!(
                    f,
                    "Vinyasa-aprayukta: indexing applied to non-array type {:?}",
                    found
                )
            }
            TypeCheckError::VinyasaSimaLanghana { index, len } => {
                write!(
                    f,
                    "Vinyasa-sima-langhana: index {} out of bounds for array length {}",
                    index, len
                )
            }
            TypeCheckError::KramashahAprayukta { found } => {
                write!(
                    f,
                    "Kramashah-aprayukta: for-each requires a Pankti (array) as the iterable; found {:?}",
                    found
                )
            }
            TypeCheckError::AvaliAsangata { expected, found } => {
                write!(
                    f,
                    "Avali-asangata: expected {:?}, found {:?}",
                    expected, found
                )
            }
            TypeCheckError::PrakshepaAprayukta { found } => {
                write!(
                    f,
                    "Prakshepa-aprayukta: push operation requires Avali type as karta; found {:?}",
                    found
                )
            }
            TypeCheckError::ApakarshanaAprayukta { found } => {
                write!(
                    f,
                    "Apakarshana-aprayukta: pop operation requires Avali type as karta; found {:?}",
                    found
                )
            }
            TypeCheckError::SamavayaAprayukta { found } => {
                write!(
                    f,
                    "Samavaya-aprayukta: field access applied to non-struct type {}",
                    found
                )
            }
            TypeCheckError::DravyaApariyata { name } => {
                write!(
                    f,
                    "Dravya-apariyata: struct type '{}' not found",
                    name
                )
            }
            TypeCheckError::AngaApraapya { dravya_name, anga_name } => {
                write!(
                    f,
                    "Anga-apraapya: field '{}' not found on struct '{}'",
                    anga_name, dravya_name
                )
            }
            TypeCheckError::NirmanaAsangati { dravya_name, expected_count, found_count, anga_name, position, expected_type, found_type } => {
                if expected_count != found_count {
                    write!(
                        f,
                        "Nirmana-asangati: expected {} values for struct '{}', found {}",
                        expected_count, dravya_name, found_count
                    )
                } else {
                    write!(
                        f,
                        "Nirmana-asangati: at position {} (field '{}') on '{}', expected {:?}, found {:?}",
                        position, anga_name, dravya_name, expected_type, found_type
                    )
                }
            }
        }
    }
}

/// Recursively walk a node's children, invoking `f` on each direct child.
fn each_child(node: &ASTNode, f: &mut dyn FnMut(&ASTNode)) {
    match node {
        ASTNode::KaryakramNode { shareera } => shareera.iter().for_each(|n| f(n)),
        ASTNode::DhatuDef { body, .. } => body.iter().for_each(|n| f(n)),
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
                f(k);
            }
            karma.iter().for_each(|n| f(n));
            if let Some(k) = karana {
                f(k);
            }
            if let Some(k) = sampradana {
                f(k);
            }
            if let Some(k) = apadan {
                f(k);
            }
            if let Some(k) = adhikarana {
                f(k);
            }
        }
        ASTNode::AstiNode { mulya, .. } | ASTNode::BhavatiNode { mulya, .. } => f(mulya),
        ASTNode::YogaNode { vama, dakshina }
        | ASTNode::ViyogaNode { vama, dakshina }
        | ASTNode::GunaNode { vama, dakshina }
        | ASTNode::BhagaNode { vama, dakshina } => {
            f(vama);
            f(dakshina);
        }
        ASTNode::SamaNode { vama, dakshina }
        | ASTNode::AsamaNode { vama, dakshina }
        | ASTNode::NyuunaNode { vama, dakshina }
        | ASTNode::AdhikaNode { vama, dakshina } => {
            f(vama);
            f(dakshina);
        }
        ASTNode::VadatiNode { mulya } => f(mulya),
        ASTNode::YadiNode {
            sthiti,
            tarhi,
            anyatha,
        } => {
            f(sthiti);
            tarhi.iter().for_each(|n| f(n));
            if let Some(b) = anyatha {
                b.iter().for_each(|n| f(n));
            }
        }
        ASTNode::YavatNode { sthiti, shareera } => {
            f(sthiti);
            shareera.iter().for_each(|n| f(n));
        }
        ASTNode::PunahNode { varam, shareera } => {
            f(varam);
            shareera.iter().for_each(|n| f(n));
        }
        ASTNode::Dvandva { members, .. } => members.iter().for_each(|n| f(n)),
        ASTNode::VaakNode { mulya, .. } => f(mulya),
        ASTNode::VaakYogaNode { vama, dakshina, .. } => {
            f(vama);
            f(dakshina);
        }
        ASTNode::Samasa { parts, .. } => parts.iter().for_each(|n| f(n)),
        ASTNode::KritChain { steps, .. } => steps.iter().for_each(|n| f(n)),
        ASTNode::UpasargaApplied { node } => f(&node.target),
        ASTNode::TaddhitaChain { base, .. } => f(base),
        ASTNode::AvartanaNode { call, .. } => f(call),
        ASTNode::PanktiNode { elements, .. } => elements.iter().for_each(|n| f(n)),
         ASTNode::AvaliNode { elements, .. } => elements.iter().for_each(|n| f(n)),
ASTNode::VinyasaNode { target, index, .. } => {
             f(target);
             f(index);
         }
         ASTNode::KramashahNode { iterable, body, .. } => {
             f(iterable);
             body.iter().for_each(|n| f(n));
         }
         ASTNode::SamavayaNode { target, .. } => f(target),
         ASTNode::DravyaDef { .. } => {}
         ASTNode::NirmanaNode { values, .. } => {
             values.iter().for_each(|n| f(n));
         }
         _ => {}
    }
}

/// Returns true if an `AvartanaNode` exists anywhere in the subtree rooted at `node`.
fn contains_avartana(node: &ASTNode) -> bool {
    if matches!(node, ASTNode::AvartanaNode { .. }) {
        return true;
    }
    let mut found = false;
    each_child(node, &mut |c| {
        if contains_avartana(c) {
            found = true;
        }
    });
    found
}

/// Returns true if a `YadiNode` exists anywhere in the subtree rooted at `node`.
fn subtree_contains_yadi(node: &ASTNode) -> bool {
    if matches!(node, ASTNode::YadiNode { .. }) {
        return true;
    }
    let mut found = false;
    each_child(node, &mut |c| {
        if subtree_contains_yadi(c) {
            found = true;
        }
    });
    found
}

/// Returns true if any reachable `YadiNode` has at least one branch
/// (`tarhi` or `anyatha`) that is fully free of `AvartanaNode`s.
fn subtree_has_free_branch_yadi(node: &ASTNode) -> bool {
    if let ASTNode::YadiNode { tarhi, anyatha, .. } = node {
        let tarhi_free = !tarhi.iter().any(contains_avartana);
        let anyatha_free = anyatha
            .as_ref()
            .map_or(false, |b| !b.iter().any(contains_avartana));
        if tarhi_free || anyatha_free {
            return true;
        }
    }
    let mut found = false;
    each_child(node, &mut |c| {
        if subtree_has_free_branch_yadi(c) {
            found = true;
        }
    });
    found
}

/// Determine whether a (recursive) DhatuDef body has a statically-reachable base case.
///
/// Rules:
///  * No `AvartanaNode` anywhere in the body  -> trivially `true` (not recursive).
///  * A `YadiNode` exists whose `tarhi` or `anyatha` branch is free of `AvartanaNode`
///    -> `true` (that branch terminates without recursing).
///  * No `YadiNode` at all, but at least one `AvartanaNode` exists -> `false`
///    (every path recurses; infinite regress risk).
///  * A `YadiNode` exists but none has a free branch -> `true` (conservative: don't flag).
fn has_reachable_base_case(body: &[ASTNode]) -> bool {
    if !body.iter().any(contains_avartana) {
        return true;
    }
    if body.iter().any(subtree_has_free_branch_yadi) {
        return true;
    }
    if !body.iter().any(subtree_contains_yadi) {
        return false;
    }
    true
}

fn resolve_type_name(env: &TypeEnv, type_name: &str) -> Option<DevvaniType> {
    if let Some(sym) = env.lookup(type_name) {
        return Some(sym.devvani_type.clone());
    }
    match type_name {
        "sankhya" | "purnaank" => Some(DevvaniType::Subject("Purnaank".to_string())),
        "dashaamsha" => Some(DevvaniType::Subject("Dashaamsha".to_string())),
        "vaak" => Some(DevvaniType::Vaak),
        _ => None,
    }
}

pub struct TypeChecker {
    pub env: TypeEnv,
    pub errors: Vec<TypeCheckError>,
    pub current_lakara: Option<Lakara>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new("global"),
            errors: Vec::new(),
            current_lakara: None,
        }
    }

    pub fn check(&mut self, node: &ASTNode) -> DevvaniType {
        match node {
            ASTNode::KaryakramNode { shareera, .. } => {
                let mut last_type = DevvaniType::Unknown;
                for stmt in shareera {
                    last_type = self.check(stmt);
                }
                last_type
            }
            ASTNode::Nama { base, .. } => {
                if let Some(sym) = self.env.lookup(base) {
                    sym.devvani_type.clone()
                } else {
                    let role = infer_type_from_suffix(base);
                    vibhakti_to_type(&role, base)
                }
            }
            ASTNode::PurnaankLiteral { .. } => DevvaniType::Subject("Purnaank".to_string()),
            ASTNode::DashaamshaLiteral { .. } => DevvaniType::Subject("Dashaamsha".to_string()),
            ASTNode::VaakLiteral { .. } => DevvaniType::Subject("Vaak".to_string()),

            ASTNode::AstiNode { naama, mulya } | ASTNode::BhavatiNode { naama, mulya } => {
                let ty = self.check(mulya);
                let symbol = Symbol::new(naama, ty.clone(), &Vacana::Eka, &Linga::Pullinga, "var");
                self.env.define_symbol(naama, symbol);
                ty
            }

            ASTNode::YogaNode { vama, dakshina }
            | ASTNode::ViyogaNode { vama, dakshina }
            | ASTNode::GunaNode { vama, dakshina }
            | ASTNode::BhagaNode { vama, dakshina } => {
                let t_vama = self.check(vama);
                let t_dakshina = self.check(dakshina);

                let is_num = |t: &DevvaniType| match t {
                    DevvaniType::Subject(s) => {
                        s == "Purnaank"
                            || s == "Dashaamsha"
                            || (s != "Bool"
                                && s != "Vaak"
                                && !s.contains("Future")
                                && !s.contains("Result"))
                    }
                    DevvaniType::Parameter(_) => true,
                    _ => false,
                };

                if !is_num(&t_vama) || !is_num(&t_dakshina) {
                    self.errors.push(TypeCheckError::PrakaaraAsangata(
                        "Arithmetic requires numeric types".to_string(),
                    ));
                    return DevvaniType::Unknown;
                }

                let types_compatible = |t1: &DevvaniType, t2: &DevvaniType| -> bool {
                    if t1 == t2 {
                        return true;
                    }
                    if matches!(t1, DevvaniType::Parameter(_))
                        || matches!(t2, DevvaniType::Parameter(_))
                    {
                        return true;
                    }
                    let is_generic = |t: &DevvaniType| match t {
                        DevvaniType::Subject(s) => {
                            s != "Purnaank"
                                && s != "Dashaamsha"
                                && s != "Bool"
                                && s != "Vaak"
                                && !s.contains("Future")
                                && !s.contains("Result")
                        }
                        _ => false,
                    };
                    if is_generic(t1) || is_generic(t2) {
                        return true;
                    }
                    false
                };

                if !types_compatible(&t_vama, &t_dakshina) {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya {
                        expected: format!("{:?}", t_vama),
                        found: format!("{:?}", t_dakshina),
                    });
                }
                t_vama
            }

            ASTNode::SamaNode { vama, dakshina }
            | ASTNode::AsamaNode { vama, dakshina }
            | ASTNode::NyuunaNode { vama, dakshina }
            | ASTNode::AdhikaNode { vama, dakshina } => {
                let t_vama = self.check(vama);
                let t_dakshina = self.check(dakshina);

                let types_compatible = |t1: &DevvaniType, t2: &DevvaniType| -> bool {
                    if t1 == t2 {
                        return true;
                    }
                    if matches!(t1, DevvaniType::Parameter(_))
                        || matches!(t2, DevvaniType::Parameter(_))
                    {
                        return true;
                    }
                    let is_generic = |t: &DevvaniType| match t {
                        DevvaniType::Subject(s) => {
                            s != "Purnaank"
                                && s != "Dashaamsha"
                                && s != "Bool"
                                && s != "Vaak"
                                && !s.contains("Future")
                                && !s.contains("Result")
                        }
                        _ => false,
                    };
                    if is_generic(t1) || is_generic(t2) {
                        return true;
                    }
                    false
                };

                if !types_compatible(&t_vama, &t_dakshina) {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya {
                        expected: format!("{:?}", t_vama),
                        found: format!("{:?}", t_dakshina),
                    });
                }
                DevvaniType::Subject("Bool".to_string())
            }

            ASTNode::VadatiNode { mulya } => {
                self.check(mulya);
                DevvaniType::Unknown
            }

            ASTNode::PathatiNode { naama } => {
                let ty = DevvaniType::Subject("Vaak".to_string());
                let symbol = Symbol::new(naama, ty.clone(), &Vacana::Eka, &Linga::Pullinga, "var");
                self.env.define_symbol(naama, symbol);
                ty
            }

            ASTNode::YadiNode {
                sthiti,
                tarhi,
                anyatha,
            } => {
                let t_sthiti = self.check(sthiti);
                if !matches!(t_sthiti, DevvaniType::Subject(ref s) if s == "Bool") {
                    self.errors.push(TypeCheckError::SatyaasatyaApekshita(
                        "Yadi condition must be Bool".to_string(),
                    ));
                }
                for stmt in tarhi {
                    self.check(stmt);
                }
                if let Some(body) = anyatha {
                    for stmt in body {
                        self.check(stmt);
                    }
                }
                DevvaniType::Unknown
            }

            ASTNode::YavatNode { sthiti, shareera } => {
                let t_sthiti = self.check(sthiti);
                if !matches!(t_sthiti, DevvaniType::Subject(ref s) if s == "Bool") {
                    self.errors.push(TypeCheckError::SatyaasatyaApekshita(
                        "Yavat condition must be Bool".to_string(),
                    ));
                }
                for stmt in shareera {
                    self.check(stmt);
                }
                DevvaniType::Unknown
            }

            ASTNode::PunahNode { varam, shareera } => {
                let t_varam = self.check(varam);
                if !matches!(t_varam, DevvaniType::Subject(ref s) if s == "Purnaank") {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya {
                        expected: "Purnaank".to_string(),
                        found: format!("{:?}", t_varam),
                    });
                }
                for stmt in shareera {
                    self.check(stmt);
                }
                DevvaniType::Unknown
            }

            ASTNode::DhatuDef {
                name,
                params,
                body,
                lakara,
                ..
            } => {
                let l_str = format!("{:?}", lakara);
                let typesystem_lakara = lakara_from_str(&l_str).unwrap_or(Lakara::Lat);

                let old_lakara = self.current_lakara.clone();
                self.current_lakara = Some(typesystem_lakara.clone());

                let scope = lakara_to_scope(&typesystem_lakara);
                let symbol = Symbol::new(
                    name,
                    DevvaniType::Scope(format!("{:?}", scope.kind)),
                    &Vacana::Eka,
                    &Linga::Pullinga,
                    "fn",
                );
                self.env.define_symbol(name, symbol);

                let old_env = self.env.clone();
                self.env = self.env.enter_scope(name);

                for param in params {
                    let ty = DevvaniType::Parameter(param.name.clone());
                    let param_symbol =
                        Symbol::new(&param.name, ty, &Vacana::Eka, &Linga::Pullinga, "i64");
                    self.env.define_symbol(&param.name, param_symbol);
                }

                for stmt in body {
                    self.check(stmt);
                }

                self.env = old_env;
                self.current_lakara = old_lakara;

                if !has_reachable_base_case(body) && body.iter().any(contains_avartana) {
                    self.errors.push(TypeCheckError::AnavasthaDosha {
                        dhatu_name: name.clone(),
                    });
                }

                match scope.return_wrapper {
                    ReturnWrapper::Future => DevvaniType::Subject(format!("Future<{}>", name)),
                    ReturnWrapper::Result => DevvaniType::Subject(format!("Result<{}>", name)),
                    _ => DevvaniType::Scope(name.clone()),
                }
            }

            ASTNode::DravyaDef { name, angas, .. } => {
                let mut resolved_angas: Vec<(String, DevvaniType)> = Vec::new();
                for anga in angas {
                    match resolve_type_name(&self.env, &anga.type_name) {
                        Some(ty) => resolved_angas.push((anga.name.clone(), ty)),
                        None => {
                            self.errors.push(TypeCheckError::DravyaApariyata {
                                name: anga.type_name.clone(),
                            });
                            return DevvaniType::Unknown;
                        }
                    }
                }
                let dravya_ty = DevvaniType::Dravya(name.clone(), resolved_angas);
                self.env.define(name, dravya_ty.clone());
                dravya_ty
            }

            ASTNode::NirmanaNode { dravya_name, values, .. } => {
                let sym = match self.env.lookup(dravya_name) {
                    Some(s) => s,
                    None => {
                        self.errors.push(TypeCheckError::DravyaApariyata {
                            name: dravya_name.clone(),
                        });
                        return DevvaniType::Unknown;
                    }
                };
                let (_dravya_type_name, angas): (String, Vec<(String, DevvaniType)>) = match &sym.devvani_type {
                    DevvaniType::Dravya(name, angas) => (name.clone(), angas.clone()),
                    _ => {
                        self.errors.push(TypeCheckError::DravyaApariyata {
                            name: dravya_name.clone(),
                        });
                        return DevvaniType::Unknown;
                    }
                };
                let expected_count = angas.len();
                let found_count = values.len();
                if expected_count != found_count {
                    self.errors.push(TypeCheckError::NirmanaAsangati {
                        dravya_name: dravya_name.clone(),
                        expected_count,
                        found_count,
                        anga_name: String::new(),
                        position: 0,
                        expected_type: DevvaniType::Unknown,
                        found_type: DevvaniType::Unknown,
                    });
                    return DevvaniType::Unknown;
                }
                for (i, (anga_name, expected_ty)) in angas.iter().enumerate() {
                    let found_ty = self.check(&values[i]);
                    if found_ty != *expected_ty {
                        self.errors.push(TypeCheckError::NirmanaAsangati {
                            dravya_name: dravya_name.clone(),
                            expected_count,
                            found_count,
                            anga_name: anga_name.clone(),
                            position: i,
                            expected_type: expected_ty.clone(),
                            found_type: found_ty.clone(),
                        });
                        return DevvaniType::Unknown;
                    }
                }
                DevvaniType::Dravya(dravya_name.clone(), angas)
            }

            ASTNode::KriyaCall {
                karta,
                kriya,
                karma,
                ..
            } => {
                if let Some(subject_node) = karta {
                    if let ASTNode::Nama { base, .. } = &**subject_node {
                        if self.env.lookup(base).is_none() {
                            self.errors
                                .push(TypeCheckError::NaamaApraapta(base.clone()));
                        }
                    }
                }

                for arg in karma {
                    let arg_type = self.check(arg);
                    match arg_type {
                        DevvaniType::Parameter(_)
                        | DevvaniType::Subject(_)
                        | DevvaniType::Vaak
                        | DevvaniType::VaakBorrow => {}
                        _ => {
                            self.errors.push(TypeCheckError::PrakaaraVaisamya {
                                expected: "Parameter/Subject".to_string(),
                                found: format!("{:?}", arg_type),
                            });
                        }
                    }
                }

                // Special-case handling for prakshepa-dhatu (push) and apakarshana-dhatu (pop)
                if kriya == "prakshepa-dhatu" {
                    let t_karta = self.check(karta.as_ref().unwrap());
                    match &t_karta {
                        DevvaniType::Avali(inner_ty) => {
                            // Validate karma has exactly 1 element
                            if karma.len() != 1 {
                                self.errors.push(TypeCheckError::PrakshepaAprayukta {
                                    found: t_karta.clone(),
                                });
                                return DevvaniType::Unknown;
                            }
                            // Validate karma element type matches inner_type (or inner is Unknown/permissive)
                            let karma_ty = self.check(&karma[0]);
                            if !matches!(inner_ty.as_ref(), DevvaniType::Unknown)
                                && &karma_ty != inner_ty.as_ref()
                                && !matches!(karma_ty, DevvaniType::Parameter(_))
                            {
                                self.errors.push(TypeCheckError::PrakshepaAprayukta {
                                    found: t_karta.clone(),
                                });
                                return DevvaniType::Unknown;
                            }
                            // Return type unchanged: Avali(inner_ty)
                            return DevvaniType::Avali(inner_ty.clone());
                        }
                        _ => {
                            self.errors.push(TypeCheckError::PrakshepaAprayukta {
                                found: t_karta,
                            });
                            return DevvaniType::Unknown;
                        }
                    }
                }

                if kriya == "apakarshana-dhatu" {
                    let t_karta = self.check(karta.as_ref().unwrap());
                    match &t_karta {
                        DevvaniType::Avali(inner_ty) => {
                            // Validate karma is empty
                            if !karma.is_empty() {
                                self.errors.push(TypeCheckError::ApakarshanaAprayukta {
                                    found: t_karta.clone(),
                                });
                                return DevvaniType::Unknown;
                            }
                            // Return type is inner element type
                            return inner_ty.as_ref().clone();
                        }
                        _ => {
                            self.errors.push(TypeCheckError::ApakarshanaAprayukta {
                                found: t_karta,
                            });
                            return DevvaniType::Unknown;
                        }
                    }
                }

                DevvaniType::Subject(kriya.clone())
            }

            ASTNode::AvartanaNode { call, .. } => self.check(call),

            ASTNode::VinyasaNode { target, index, .. } => {
                let t_target = self.check(target);
                let t_index = self.check(index);

                match t_target {
                    DevvaniType::Pankti(elem_ty, len) => {
                        let is_num = |t: &DevvaniType| match t {
                            DevvaniType::Subject(s) => {
                                s == "Purnaank"
                                    || s == "Dashaamsha"
                                    || (s != "Bool"
                                        && s != "Vaak"
                                        && !s.contains("Future")
                                        && !s.contains("Result"))
                            }
                            DevvaniType::Parameter(_) => true,
                            _ => false,
                        };

                        if !is_num(&t_index) {
                            self.errors.push(TypeCheckError::PrakaaraAsangata(
                                "Indexer must be numeric".to_string(),
                            ));
                            return elem_ty.as_ref().clone();
                        }

                        if let ASTNode::PurnaankLiteral { value, .. } = index.as_ref() {
                            if *value >= len as i64 {
                                self.errors.push(TypeCheckError::VinyasaSimaLanghana {
                                    index: *value as usize,
                                    len,
                                });
                            }
                        }

                        elem_ty.as_ref().clone()
                    }
                    _ => {
                        self.errors.push(TypeCheckError::VinyasaAprayukta {
                            found: t_target.clone(),
                        });
                        DevvaniType::Unknown
                    }
                }
            }

            ASTNode::SamavayaNode { target, anga_name, .. } => {
                let t_target = self.check(target);
                match t_target {
                    DevvaniType::Dravya(_dravya_name, angas) => {
                        for (name, ty) in angas {
                            if name == *anga_name {
                                return ty;
                            }
                        }
                        self.errors.push(TypeCheckError::AngaApraapya {
                            dravya_name: _dravya_name.clone(),
                            anga_name: anga_name.clone(),
                        });
                        DevvaniType::Unknown
                    }
                    _ => {
                        self.errors.push(TypeCheckError::SamavayaAprayukta {
                            found: format!("{:?}", t_target),
                        });
                        DevvaniType::Unknown
                    }
                }
            }

            ASTNode::KramashahNode { item_name, iterable, body, .. } => {
                let t_iterable = self.check(iterable);
                let elem_ty = match &t_iterable {
                    DevvaniType::Pankti(elem_ty, _len) => elem_ty.as_ref().clone(),
                    _ => {
                        self.errors.push(TypeCheckError::KramashahAprayukta {
                            found: t_iterable.clone(),
                        });
                        DevvaniType::Unknown
                    }
                };
                let old_env = self.env.clone();
                self.env = self.env.enter_scope(item_name);
                let item_symbol = Symbol::new(
                    item_name,
                    elem_ty,
                    &Vacana::Eka,
                    &Linga::Pullinga,
                    "i64",
                );
                self.env.define_symbol(item_name, item_symbol);
                for stmt in body {
                    self.check(stmt);
                }
                self.env = old_env;
                DevvaniType::Unknown
            }

            ASTNode::PanktiNode { elements, .. } => {
                if elements.is_empty() {
                    return DevvaniType::Pankti(Box::new(DevvaniType::Unknown), 0);
                }

                let mut element_types: Vec<DevvaniType> = Vec::new();
                for elem in elements {
                    let elem_ty = self.check(elem);
                    element_types.push(elem_ty);
                }

                let first_type = element_types[0].clone();
                for (_i, elem_ty) in element_types.iter().enumerate().skip(1) {
                    if elem_ty != &first_type {
                        self.errors.push(TypeCheckError::PanktiAsangata {
                            expected: first_type.clone(),
                            found: elem_ty.clone(),
                        });
                        return DevvaniType::Unknown;
                    }
                }

                DevvaniType::Pankti(Box::new(first_type), elements.len())
            }

            ASTNode::AvaliNode { elements, .. } => {
                if elements.is_empty() {
                    return DevvaniType::Avali(Box::new(DevvaniType::Unknown));
                }

                let mut element_types: Vec<DevvaniType> = Vec::new();
                for elem in elements {
                    let elem_ty = self.check(elem);
                    element_types.push(elem_ty);
                }

                let first_type = element_types[0].clone();
                for elem_ty in element_types.iter().skip(1) {
                    if elem_ty != &first_type {
                        self.errors.push(TypeCheckError::AvaliAsangata {
                            expected: first_type.clone(),
                            found: elem_ty.clone(),
                        });
                        return DevvaniType::Unknown;
                    }
                }

                DevvaniType::Avali(Box::new(first_type))
            }

            _ => DevvaniType::Unknown,
        }
    }

    pub fn check_program(&mut self, node: &ASTNode) -> Vec<TypeCheckError> {
        self.check(node);
        self.errors.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_ast::{ASTNode, AngaField, Gana, Lakara, Linga, Span, Vacana};

    fn span() -> Span {
        Span {
            line: 0,
            col: 0,
            len: 0,
        }
    }

    fn avartana(name: &str) -> ASTNode {
        ASTNode::AvartanaNode {
            call: Box::new(ASTNode::KriyaCall {
                karta: None,
                kriya: name.to_string(),
                karma: vec![],
                karana: None,
                sampradana: None,
                apadan: None,
                adhikarana: None,
                span: span(),
            }),
            span: span(),
        }
    }

    fn dhatu_def(name: &str, body: Vec<ASTNode>) -> ASTNode {
        ASTNode::DhatuDef {
            name: name.to_string(),
            lakara: Lakara::Lat,
            gana: Gana::Bhvadi,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            body,
            span: span(),
        }
    }

    fn yadi(tarhi: Vec<ASTNode>, anyatha: Option<Vec<ASTNode>>) -> ASTNode {
        ASTNode::YadiNode {
            sthiti: Box::new(ASTNode::Nama {
                base: "satyam".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            tarhi,
            anyatha,
        }
    }

    fn check_dhatu(body: Vec<ASTNode>) -> Vec<TypeCheckError> {
        let mut checker = TypeChecker::new();
        checker.check(&dhatu_def("recur", body));
        checker.errors
    }

    #[test]
    fn recursive_with_base_case_yields_no_anavastha() {
        // yadi ... tarhi (no recursion) ... anyatha (recursive) ... samaptih
        let body = vec![yadi(
            vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "base".to_string(),
                    span: span(),
                }),
            }],
            Some(vec![avartana("recur")]),
        )];
        let errors = check_dhatu(body);
        assert!(
            !errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
            "expected no AnavasthaDosha, got: {:?}",
            errors
        );
    }

    #[test]
    fn recursive_no_yadi_yields_anavastha() {
        // Recursive call directly, no conditional guard at all.
        let body = vec![avartana("recur")];
        let errors = check_dhatu(body);
        let dosha = errors
            .iter()
            .find(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. }));
        assert!(
            dosha.is_some(),
            "expected AnavasthaDosha, got: {:?}",
            errors
        );
        match dosha.unwrap() {
            TypeCheckError::AnavasthaDosha { dhatu_name } => {
                assert_eq!(dhatu_name, "recur");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn non_recursive_yields_no_anavastha() {
        // No AvartanaNode anywhere -> must not be flagged.
        let body = vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
        }];
        let errors = check_dhatu(body);
        assert!(
            !errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
            "non-recursive dhatu must not produce AnavasthaDosha, got: {:?}",
            errors
        );
    }

    #[test]
    fn recursive_yadi_both_branhes_recurse_is_conservative() {
        // Has a Yadi but both branches contain recursion -> conservative: no flag.
        let body = vec![yadi(vec![avartana("recur")], Some(vec![avartana("recur")]))];
        let errors = check_dhatu(body);
        assert!(
            !errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
            "conservative: both branches recurse but has guard, got: {:?}",
            errors
        );
    }

    // Pankti (fixed-size array) tests

    #[test]
    fn homogeneous_numeric_pankti_type_checks() {
        let mut checker = TypeChecker::new();
        let pankti = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 2,
                    span: span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 3,
                    span: span(),
                },
            ],
            span: span(),
        };
        let ty = checker.check(&pankti);
        assert!(matches!(ty, DevvaniType::Pankti(_, 3)));
        if let DevvaniType::Pankti(elem_ty, len) = &ty {
            assert_eq!(*len, 3);
            assert_eq!(**elem_ty, DevvaniType::Subject("Purnaank".to_string()));
        }
    }

    #[test]
    fn empty_pankti_type_checks_to_unknown() {
        let mut checker = TypeChecker::new();
        let pankti = ASTNode::PanktiNode {
            elements: vec![],
            span: span(),
        };
        let ty = checker.check(&pankti);
        assert_eq!(ty, DevvaniType::Pankti(Box::new(DevvaniType::Unknown), 0));
    }

    #[test]
    fn mixed_type_pankti_produces_pankti_asangata() {
        let mut checker = TypeChecker::new();
        let pankti = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: span(),
                },
                ASTNode::VaakLiteral {
                    value: "string".to_string(),
                    span: span(),
                },
            ],
            span: span(),
        };
        let _ty = checker.check(&pankti);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PanktiAsangata { .. })),
            "expected PanktiAsangata error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn nested_pankti_type_checks_correctly() {
        let mut checker = TypeChecker::new();
        let nested_pankti = ASTNode::PanktiNode {
            elements: vec![
                ASTNode::PanktiNode {
                    elements: vec![
                        ASTNode::PurnaankLiteral {
                            value: 1,
                            span: span(),
                        },
                        ASTNode::PurnaankLiteral {
                            value: 2,
                            span: span(),
                        },
                    ],
                    span: span(),
                },
                ASTNode::PanktiNode {
                    elements: vec![
                        ASTNode::PurnaankLiteral {
                            value: 3,
                            span: span(),
                        },
                        ASTNode::PurnaankLiteral {
                            value: 4,
                            span: span(),
                        },
                    ],
                    span: span(),
                },
            ],
            span: span(),
        };
        let ty = checker.check(&nested_pankti);
        assert!(matches!(ty, DevvaniType::Pankti(_, 2)));
        if let DevvaniType::Pankti(elem_ty, len) = &ty {
            assert_eq!(*len, 2);
            assert!(matches!(**elem_ty, DevvaniType::Pankti(_, 2)));
        }
    }

    #[test]
    fn valid_vinyasa_index_resolves_to_element_type() {
        let mut checker = TypeChecker::new();
        // First define an array variable
        let array_node = ASTNode::AstiNode {
            naama: "arr".to_string(),
            mulya: Box::new(ASTNode::PanktiNode {
                elements: vec![
                    ASTNode::PurnaankLiteral {
                        value: 10,
                        span: span(),
                    },
                    ASTNode::PurnaankLiteral {
                        value: 20,
                        span: span(),
                    },
                    ASTNode::PurnaankLiteral {
                        value: 30,
                        span: span(),
                    },
                ],
                span: span(),
            }),
        };
        checker.check(&array_node);

        // Now index it
        let vinyasa = ASTNode::VinyasaNode {
            target: Box::new(ASTNode::Nama {
                base: "arr".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            index: Box::new(ASTNode::PurnaankLiteral {
                value: 0,
                span: span(),
            }),
            span: span(),
        };
        let ty = checker.check(&vinyasa);
        assert_eq!(ty, DevvaniType::Subject("Purnaank".to_string()));
    }

    #[test]
    fn vinyasa_non_pankti_produces_aprayukta() {
        let mut checker = TypeChecker::new();
        let vinyasa = ASTNode::VinyasaNode {
            target: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            index: Box::new(ASTNode::PurnaankLiteral {
                value: 0,
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&vinyasa);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::VinyasaAprayukta { .. })),
            "expected VinyasaAprayukta error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn statically_out_of_bounds_index_produces_sima_langhana() {
        let mut checker = TypeChecker::new();
        // Define a 3-element array
        let array_node = ASTNode::AstiNode {
            naama: "x".to_string(),
            mulya: Box::new(ASTNode::PanktiNode {
                elements: vec![
                    ASTNode::PurnaankLiteral {
                        value: 1,
                        span: span(),
                    },
                    ASTNode::PurnaankLiteral {
                        value: 2,
                        span: span(),
                    },
                    ASTNode::PurnaankLiteral {
                        value: 3,
                        span: span(),
                    },
                ],
                span: span(),
            }),
        };
        checker.check(&array_node);

        // Index with out-of-bounds constant 5
        let vinyasa = ASTNode::VinyasaNode {
            target: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            index: Box::new(ASTNode::PurnaankLiteral {
                value: 5,
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&vinyasa);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::VinyasaSimaLanghana { .. })),
            "expected VinyasaSimaLanghana error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn variable_index_does_not_error() {
        let mut checker = TypeChecker::new();
        // Define array and index variable
        let array_node = ASTNode::AstiNode {
            naama: "arr".to_string(),
            mulya: Box::new(ASTNode::PanktiNode {
                elements: vec![
                    ASTNode::PurnaankLiteral {
                        value: 1,
                        span: span(),
                    },
                    ASTNode::PurnaankLiteral {
                        value: 2,
                        span: span(),
                    },
                ],
                span: span(),
            }),
        };
        checker.check(&array_node);

        let _idx_node = ASTNode::AstiNode {
            naama: "idx".to_string(),
            mulya: Box::new(ASTNode::PurnaankLiteral {
                value: 0,
                span: span(),
            }),
        };
        checker.check(&_idx_node);

        // Index with variable (not a compile-time constant)
        let vinyasa = ASTNode::VinyasaNode {
            target: Box::new(ASTNode::Nama {
                base: "arr".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            index: Box::new(ASTNode::Nama {
                base: "idx".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&vinyasa);
        // Should NOT have VinyasaSimaLanghana since index is a variable
        assert!(
            !checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::VinyasaSimaLanghana { .. })),
            "expected NO VinyasaSimaLanghana for variable index, got: {:?}",
            checker.errors
        );
    }

    // Kramashah (for-each loop) tests

    #[test]
    fn test_kramasah_over_pankti_ok() {
        let mut checker = TypeChecker::new();
        let kramasah = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(ASTNode::PanktiNode {
                elements: vec![
                    ASTNode::PurnaankLiteral { value: 1, span: span() },
                    ASTNode::PurnaankLiteral { value: 2, span: span() },
                    ASTNode::PurnaankLiteral { value: 3, span: span() },
                ],
                span: span(),
            }),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            }],
            span: span(),
        };
        checker.check(&kramasah);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::KramashahAprayukta { .. })),
            "expected no KramashahAprayukta error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_kramasah_item_type_matches_element() {
        let mut checker = TypeChecker::new();
        // After entering the loop, x should be typed as Purnaank
        let kramasah = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(ASTNode::PanktiNode {
                elements: vec![
                    ASTNode::PurnaankLiteral { value: 10, span: span() },
                    ASTNode::PurnaankLiteral { value: 20, span: span() },
                ],
                span: span(),
            }),
            body: vec![ASTNode::YogaNode {
                vama: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                dakshina: Box::new(ASTNode::PurnaankLiteral {
                    value: 5,
                    span: span(),
                }),
            }],
            span: span(),
        };
        checker.check(&kramasah);
        // Should NOT error because x is Purnaank and can participate in arithmetic
        assert!(
            !checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PrakaaraAsangata { .. })),
            "expected arithmetic to succeed with Purnaank item type, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_kramasah_over_non_pankti_errors() {
        let mut checker = TypeChecker::new();
        let kramasah = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(ASTNode::PurnaankLiteral { value: 42, span: span() }),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            }],
            span: span(),
        };
        let _ty = checker.check(&kramasah);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::KramashahAprayukta { .. })),
            "expected KramashahAprayukta error for non-Pankti iterable, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_kramasah_empty_pankti() {
        let mut checker = TypeChecker::new();
        let kramasah = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(ASTNode::PanktiNode {
                elements: vec![],
                span: span(),
            }),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            }],
            span: span(),
        };
        checker.check(&kramasah);
        // Should typecheck without crashing, no error for empty Pankti
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::KramashahAprayukta { .. })),
            "expected no error for empty Pankti, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_kramasah_scope_isolated() {
        let mut checker = TypeChecker::new();
        // First define a symbol to verify scope isolation
        checker.env.define("outer_var", DevvaniType::Subject("Purnaank".to_string()));
        assert!(checker.env.lookup("outer_var").is_some());

        let kramasah = ASTNode::KramashahNode {
            item_name: "x".to_string(),
            iterable: Box::new(ASTNode::PanktiNode {
                elements: vec![ASTNode::PurnaankLiteral { value: 1, span: span() }],
                span: span(),
            }),
            body: vec![],
            span: span(),
        };
        checker.check(&kramasah);

        // After check, x should NOT be in scope (scope was popped)
        assert!(
            checker.env.lookup("x").is_none(),
            "expected item 'x' to be out of scope after KramashahNode, but found it"
        );
        // outer_var should still be accessible (scope was properly restored)
        assert!(
            checker.env.lookup("outer_var").is_some(),
            "expected 'outer_var' to still be in scope after KramashahNode"
        );
    }

    #[test]
    fn test_kramasah_diagnostics_d053() {
        // Verify TypeCheckError::KramashahAprayukta exists and formats correctly
        let err = TypeCheckError::KramashahAprayukta {
            found: DevvaniType::Subject("Purnaank".to_string()),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Pankti"));
    }

    // Avali (growable array) tests

    #[test]
    fn test_avali_literal_homogeneous_type() {
        let mut checker = TypeChecker::new();
        let avali = ASTNode::AvaliNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 2,
                    span: span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 3,
                    span: span(),
                },
            ],
            span: span(),
        };
        let ty = checker.check(&avali);
        assert!(matches!(ty, DevvaniType::Avali(_)));
        if let DevvaniType::Avali(elem_ty) = &ty {
            assert_eq!(*elem_ty, Box::new(DevvaniType::Subject("Purnaank".to_string())));
        }
    }

    #[test]
    fn test_avali_literal_heterogeneous_error() {
        let mut checker = TypeChecker::new();
        let avali = ASTNode::AvaliNode {
            elements: vec![
                ASTNode::PurnaankLiteral {
                    value: 1,
                    span: span(),
                },
                ASTNode::VaakLiteral {
                    value: "text".to_string(),
                    span: span(),
                },
            ],
            span: span(),
        };
        let _ty = checker.check(&avali);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AvaliAsangata { .. })),
            "expected AvaliAsangata error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_avali_literal_empty_type() {
        let mut checker = TypeChecker::new();
        let avali = ASTNode::AvaliNode {
            elements: vec![],
            span: span(),
        };
        let ty = checker.check(&avali);
        assert_eq!(ty, DevvaniType::Avali(Box::new(DevvaniType::Unknown)));
    }

    #[test]
    fn test_prakshepa_valid() {
        let mut checker = TypeChecker::new();
        // Define avali variable
        let avali_node = ASTNode::AstiNode {
            naama: "arr".to_string(),
            mulya: Box::new(ASTNode::AvaliNode {
                elements: vec![
                    ASTNode::PurnaankLiteral {
                        value: 10,
                        span: span(),
                    },
                ],
                span: span(),
            }),
        };
        checker.check(&avali_node);

        // Push a matching Purnaank value
        let kriya = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "arr".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            })),
            kriya: "prakshepa-dhatu".to_string(),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 20,
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let ty = checker.check(&kriya);
        assert!(matches!(ty, DevvaniType::Avali(_)));
        if let DevvaniType::Avali(elem_ty) = &ty {
            assert_eq!(*elem_ty, Box::new(DevvaniType::Subject("Purnaank".to_string())));
        }
    }

    #[test]
    fn test_prakshepa_on_non_avali_error() {
        let mut checker = TypeChecker::new();
        // Define a Pankti (fixed array) instead of Avali
        let pankti_node = ASTNode::AstiNode {
            naama: "arr".to_string(),
            mulya: Box::new(ASTNode::PanktiNode {
                elements: vec![ASTNode::PurnaankLiteral {
                    value: 10,
                    span: span(),
                }],
                span: span(),
            }),
        };
        checker.check(&pankti_node);

        // Try prakshepa-dhatu on Pankti
        let kriya = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "arr".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            })),
            kriya: "prakshepa-dhatu".to_string(),
            karma: vec![ASTNode::PurnaankLiteral {
                value: 20,
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let _ty = checker.check(&kriya);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PrakshepaAprayukta { .. })),
            "expected PrakshepaAprayukta error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_apakarshana_valid() {
        let mut checker = TypeChecker::new();
        // Define avali variable
        let avali_node = ASTNode::AstiNode {
            naama: "arr".to_string(),
            mulya: Box::new(ASTNode::AvaliNode {
                elements: vec![
                    ASTNode::PurnaankLiteral {
                        value: 10,
                        span: span(),
                    },
                ],
                span: span(),
            }),
        };
        checker.check(&avali_node);

        // Pop from Avali
        let kriya = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "arr".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            })),
            kriya: "apakarshana-dhatu".to_string(),
            karma: vec![],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let ty = checker.check(&kriya);
        assert_eq!(ty, DevvaniType::Subject("Purnaank".to_string()));
    }

    #[test]
    fn test_apakarshana_on_non_avali_error() {
        let mut checker = TypeChecker::new();
        // Define a plain variable
        let var_node = ASTNode::AstiNode {
            naama: "x".to_string(),
            mulya: Box::new(ASTNode::PurnaankLiteral {
                value: 10,
                span: span(),
            }),
        };
        checker.check(&var_node);

        // Try apakarshana-dhatu on non-Avali
        let kriya = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            })),
            kriya: "apakarshana-dhatu".to_string(),
            karma: vec![],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let _ty = checker.check(&kriya);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ApakarshanaAprayukta { .. })),
            "expected ApakarshanaAprayukta error, got: {:?}",
            checker.errors
        );
    }

    fn dravya_def(name: &str, angas: Vec<AngaField>) -> ASTNode {
        ASTNode::DravyaDef {
            name: name.to_string(),
            angas,
            span: span(),
        }
    }

    fn anga_field(name: &str, type_name: &str) -> AngaField {
        AngaField {
            name: name.to_string(),
            type_name: type_name.to_string(),
            span: span(),
        }
    }

    // Dravya (struct) tests

    #[test]
    fn test_dravya_def_valid_fields() {
        let mut checker = TypeChecker::new();
        let def = dravya_def(
            "manushya",
            vec![anga_field("naama", "vaak"), anga_field("sankhya", "sankhya")],
        );
        let ty = checker.check(&def);
        assert!(matches!(ty, DevvaniType::Dravya(_, _)));
        if let DevvaniType::Dravya(name, angas) = &ty {
            assert_eq!(name, "manushya");
            assert_eq!(angas.len(), 2);
            assert_eq!(angas[0], ("naama".to_string(), DevvaniType::Vaak));
            assert_eq!(angas[1], ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())));
        }
    }

    #[test]
    fn test_dravya_def_empty_fields() {
        let mut checker = TypeChecker::new();
        let def = dravya_def("shunya", vec![]);
        let ty = checker.check(&def);
        assert!(matches!(ty, DevvaniType::Dravya(_, _)));
        if let DevvaniType::Dravya(name, angas) = &ty {
            assert_eq!(name, "shunya");
            assert!(angas.is_empty());
        }
    }

    #[test]
    fn test_dravya_def_unknown_field_type() {
        let mut checker = TypeChecker::new();
        let def = dravya_def("gadha", vec![anga_field("x", "agjadravya")]);
        let _ty = checker.check(&def);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DravyaApariyata { .. })),
            "expected DravyaApariyata error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_samavaya_valid_access() {
        let mut checker = TypeChecker::new();
        // Define the struct type first
        let def = dravya_def(
            "manushya",
            vec![anga_field("naama", "vaak"), anga_field("sankhya", "sankhya")],
        );
        checker.check(&def);

        // Create a variable whose type is the struct
        let obj = ASTNode::AstiNode {
            naama: "m".to_string(),
            mulya: Box::new(ASTNode::Nama {
                base: "manushya".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        };
        checker.check(&obj);

        // Access field
        let access = ASTNode::SamavayaNode {
            target: Box::new(ASTNode::Nama {
                base: "m".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            anga_name: "naama".to_string(),
            span: span(),
        };
        let ty = checker.check(&access);
        assert_eq!(ty, DevvaniType::Vaak);
    }

    #[test]
    fn test_samavaya_unknown_field() {
        let mut checker = TypeChecker::new();
        let def = dravya_def(
            "manushya",
            vec![anga_field("naama", "vaak")],
        );
        checker.check(&def);

        let obj = ASTNode::AstiNode {
            naama: "m".to_string(),
            mulya: Box::new(ASTNode::Nama {
                base: "manushya".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        };
        checker.check(&obj);

        let access = ASTNode::SamavayaNode {
            target: Box::new(ASTNode::Nama {
                base: "m".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            anga_name: "agaj".to_string(),
            span: span(),
        };
        let _ty = checker.check(&access);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AngaApraapya { .. })),
            "expected AngaApraapya error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_samavaya_on_non_dravya() {
        let mut checker = TypeChecker::new();
        let access = ASTNode::SamavayaNode {
            target: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            anga_name: "x".to_string(),
            span: span(),
        };
        let _ty = checker.check(&access);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SamavayaAprayukta { .. })),
            "expected SamavayaAprayukta error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_samavaya_chained_access() {
        let mut checker = TypeChecker::new();
        // Define two structs: pura (with field 'sthal') and khetra (with field 'pura')
        checker.check(&dravya_def(
            "pura",
            vec![anga_field("sthal", "vaak")],
        ));
        let def2 = dravya_def(
            "khetra",
            vec![anga_field("pura", "pura")],
        );
        checker.check(&def2);

        // Create a variable of type khetra
        let obj = ASTNode::AstiNode {
            naama: "k".to_string(),
            mulya: Box::new(ASTNode::Nama {
                base: "khetra".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        };
        checker.check(&obj);

        // First access: k.pura -> Dravya("pura", [(sthal, Vaak)])
        let access1 = ASTNode::SamavayaNode {
            target: Box::new(ASTNode::Nama {
                base: "k".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            anga_name: "pura".to_string(),
            span: span(),
        };
        let ty1 = checker.check(&access1);
        assert_eq!(ty1, DevvaniType::Dravya("pura".to_string(), vec![("sthal".to_string(), DevvaniType::Vaak)]));

        // Second access: (k.pura).sthal -> Vaak
        let access2 = ASTNode::SamavayaNode {
            target: Box::new(access1),
            anga_name: "sthal".to_string(),
            span: span(),
        };
        let ty2 = checker.check(&access2);
        assert_eq!(ty2, DevvaniType::Vaak);
    }

    // Nirmāṇa (struct instantiation) tests

    fn nirmana(dravya_name: &str, values: Vec<ASTNode>) -> ASTNode {
        ASTNode::NirmanaNode {
            dravya_name: dravya_name.to_string(),
            values,
            span: span(),
        }
    }

    #[test]
    fn test_nirmana_valid_instantiation() {
        let mut checker = TypeChecker::new();
        checker.check(&dravya_def(
            "manushya",
            vec![anga_field("sankhya", "sankhya"), anga_field("dashaamsha", "dashaamsha")],
        ));

        let instantiation = nirmana(
            "manushya",
            vec![
                ASTNode::PurnaankLiteral { value: 5, span: span() },
                ASTNode::DashaamshaLiteral { value: 3.0, span: span() },
            ],
        );
        let ty = checker.check(&instantiation);
        assert!(matches!(ty, DevvaniType::Dravya(_, _)));
        assert_eq!(
            ty,
            DevvaniType::Dravya(
                "manushya".to_string(),
                vec![
                    ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())),
                    ("dashaamsha".to_string(), DevvaniType::Subject("Dashaamsha".to_string())),
                ]
            )
        );
    }

    #[test]
    fn test_nirmana_value_count_mismatch() {
        let mut checker = TypeChecker::new();
        checker.check(&dravya_def(
            "manushya",
            vec![anga_field("sankhya", "sankhya"), anga_field("dashaamsha", "dashaamsha")],
        ));

        let instantiation = nirmana(
            "manushya",
            vec![ASTNode::PurnaankLiteral { value: 5, span: span() }],
        );
        let _ty = checker.check(&instantiation);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::NirmanaAsangati { .. })),
            "expected NirmanaAsangati error, got: {:?}",
            checker.errors
        );
        if let Some(TypeCheckError::NirmanaAsangati { expected_count, found_count, .. }) = checker.errors.iter().find(|e| matches!(e, TypeCheckError::NirmanaAsangati { .. })) {
            assert_eq!(*expected_count, 2);
            assert_eq!(*found_count, 1);
        }
    }

    #[test]
    fn test_nirmana_type_mismatch_at_position() {
        let mut checker = TypeChecker::new();
        checker.check(&dravya_def(
            "manushya",
            vec![anga_field("sankhya", "sankhya"), anga_field("dashaamsha", "dashaamsha")],
        ));

        let instantiation = nirmana(
            "manushya",
            vec![
                ASTNode::PurnaankLiteral { value: 5, span: span() },
                ASTNode::PurnaankLiteral { value: 3, span: span() },
            ],
        );
        let _ty = checker.check(&instantiation);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::NirmanaAsangati { .. })),
            "expected NirmanaAsangati error, got: {:?}",
            checker.errors
        );
        if let Some(TypeCheckError::NirmanaAsangati { anga_name, position, expected_type, found_type, .. }) = checker.errors.iter().find(|e| matches!(e, TypeCheckError::NirmanaAsangati { .. })) {
            assert_eq!(anga_name, "dashaamsha");
            assert_eq!(*position, 1);
            assert_eq!(*expected_type, DevvaniType::Subject("Dashaamsha".to_string()));
            assert_eq!(*found_type, DevvaniType::Subject("Purnaank".to_string()));
        }
    }

    #[test]
    fn test_nirmana_undefined_dravya_name() {
        let mut checker = TypeChecker::new();
        let instantiation = nirmana(
            "agadravya",
            vec![ASTNode::PurnaankLiteral { value: 1, span: span() }],
        );
        let _ty = checker.check(&instantiation);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DravyaApariyata { .. })),
            "expected DravyaApariyata error, got: {:?}",
            checker.errors
        );
        assert!(
            !checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::NirmanaAsangati { .. })),
            "expected no NirmanaAsangati error for undefined dravya, got: {:?}",
            checker.errors
        );
    }
}
