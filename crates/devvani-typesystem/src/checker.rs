use devvani_ast::ASTNode;
use crate::{vibhakti::*, type_env::TypeEnv, lakara::*, vacana::*, linga::*, symbol::*};
use std::fmt;

#[derive(Debug, Clone)]
pub enum TypeCheckError {
    NaamaApraapta(String),
    PrakaaraVaisamya { expected: String, found: String },
    SatyaasatyaApekshita(String),
    PrakaaraAsangata(String),
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::NaamaApraapta(name) => write!(f, "Naama-apraapta: {}", name),
            TypeCheckError::PrakaaraVaisamya { expected, found } => {
                write!(f, "Prakaara-vaisamya: expected {}, found {}", expected, found)
            }
            TypeCheckError::SatyaasatyaApekshita(msg) => write!(f, "Satyaasatya-apekshita: {}", msg),
            TypeCheckError::PrakaaraAsangata(msg) => write!(f, "Prakaara-asangata: {}", msg),
        }
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

            ASTNode::YogaNode { vama, dakshina } |
            ASTNode::ViyogaNode { vama, dakshina } |
            ASTNode::GunaNode { vama, dakshina } |
            ASTNode::BhagaNode { vama, dakshina } => {
                let t_vama = self.check(vama);
                let t_dakshina = self.check(dakshina);
                
                let is_num = |t: &DevvaniType| matches!(t, DevvaniType::Subject(s) if s == "Purnaank" || s == "Dashaamsha");
                
                if !is_num(&t_vama) || !is_num(&t_dakshina) {
                    self.errors.push(TypeCheckError::PrakaaraAsangata("Arithmetic requires numeric types".to_string()));
                    return DevvaniType::Unknown;
                }
                
                if t_vama != t_dakshina {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya { 
                        expected: format!("{:?}", t_vama), 
                        found: format!("{:?}", t_dakshina) 
                    });
                }
                t_vama
            }

            ASTNode::SamaNode { vama, dakshina } |
            ASTNode::AsamaNode { vama, dakshina } |
            ASTNode::NyuunaNode { vama, dakshina } |
            ASTNode::AdhikaNode { vama, dakshina } => {
                let t_vama = self.check(vama);
                let t_dakshina = self.check(dakshina);
                if t_vama != t_dakshina {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya { 
                        expected: format!("{:?}", t_vama), 
                        found: format!("{:?}", t_dakshina) 
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

            ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
                let t_sthiti = self.check(sthiti);
                if !matches!(t_sthiti, DevvaniType::Subject(ref s) if s == "Bool") {
                    self.errors.push(TypeCheckError::SatyaasatyaApekshita("Yadi condition must be Bool".to_string()));
                }
                for stmt in tarhi { self.check(stmt); }
                if let Some(body) = anyatha {
                    for stmt in body { self.check(stmt); }
                }
                DevvaniType::Unknown
            }

            ASTNode::YavatNode { sthiti, shareera } => {
                let t_sthiti = self.check(sthiti);
                if !matches!(t_sthiti, DevvaniType::Subject(ref s) if s == "Bool") {
                    self.errors.push(TypeCheckError::SatyaasatyaApekshita("Yavat condition must be Bool".to_string()));
                }
                for stmt in shareera { self.check(stmt); }
                DevvaniType::Unknown
            }

            ASTNode::PunahNode { varam, shareera } => {
                let t_varam = self.check(varam);
                if !matches!(t_varam, DevvaniType::Subject(ref s) if s == "Purnaank") {
                    self.errors.push(TypeCheckError::PrakaaraVaisamya { 
                        expected: "Purnaank".to_string(), 
                        found: format!("{:?}", t_varam) 
                    });
                }
                for stmt in shareera { self.check(stmt); }
                DevvaniType::Unknown
            }

            ASTNode::DhatuDef { name, params, body, lakara, .. } => {
                let l_str = format!("{:?}", lakara);
                let typesystem_lakara = lakara_from_str(&l_str).unwrap_or(Lakara::Lat);
                
                let old_lakara = self.current_lakara.clone();
                self.current_lakara = Some(typesystem_lakara.clone());
                
                let scope = lakara_to_scope(&typesystem_lakara);
                let symbol = Symbol::new(name, DevvaniType::Scope(format!("{:?}", scope.kind)), &Vacana::Eka, &Linga::Pullinga, "fn");
                self.env.define_symbol(name, symbol);

                let old_env = self.env.clone();
                self.env = self.env.enter_scope(name);
                
                for param in params {
                    let ty = DevvaniType::Parameter(param.name.clone());
                    let param_symbol = Symbol::new(&param.name, ty, &Vacana::Eka, &Linga::Pullinga, "i64");
                    self.env.define_symbol(&param.name, param_symbol);
                }

                for stmt in body {
                    self.check(stmt);
                }

                self.env = old_env;
                self.current_lakara = old_lakara;

                match scope.return_wrapper {
                    ReturnWrapper::Future => DevvaniType::Subject(format!("Future<{}>", name)),
                    ReturnWrapper::Result => DevvaniType::Subject(format!("Result<{}>", name)),
                    _ => DevvaniType::Scope(name.clone()),
                }
            }

            ASTNode::KriyaCall { karta, kriya, karma, .. } => {
                if let Some(subject_node) = karta {
                    if let ASTNode::Nama { base, .. } = &**subject_node {
                        if self.env.lookup(base).is_none() {
                            self.errors.push(TypeCheckError::NaamaApraapta(base.clone()));
                        }
                    }
                }

                for arg in karma {
                    let arg_type = self.check(arg);
                    match arg_type {
                        DevvaniType::Parameter(_) => {}
                        _ => {
                            self.errors.push(TypeCheckError::PrakaaraVaisamya {
                                expected: "Parameter".to_string(),
                                found: format!("{:?}", arg_type),
                            });
                        }
                    }
                }

                DevvaniType::Subject(kriya.clone())
            }
            
            _ => DevvaniType::Unknown,
        }
    }

    pub fn check_program(&mut self, node: &ASTNode) -> Vec<TypeCheckError> {
        self.check(node);
        self.errors.clone()
    }
}
