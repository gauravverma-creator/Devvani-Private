use devvani_ast::ASTNode;
use crate::{vibhakti::*, type_env::TypeEnv, lakara::*, vacana::*, linga::*, symbol::*};
use std::fmt;

#[derive(Debug, Clone)]
pub enum TypeCheckError {
    UndefinedName(String),
    TypeMismatch { expected: String, found: String },
    InvalidVibhaktiUsage(String),
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::UndefinedName(name) => write!(f, "Undefined name: {}", name),
            TypeCheckError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            TypeCheckError::InvalidVibhaktiUsage(msg) => write!(f, "Invalid vibhakti usage: {}", msg),
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

    pub fn current_scope_kind(&self) -> ScopeKind {
        match &self.current_lakara {
            Some(l) => lakara_to_scope(l).kind,
            None => ScopeKind::Sync,
        }
    }

    pub fn check(&mut self, node: &ASTNode) -> DevvaniType {
        match node {
            ASTNode::Program { statements, .. } => {
                let mut last_type = DevvaniType::Unknown;
                for stmt in statements {
                    last_type = self.check(stmt);
                }
                last_type
            }
            ASTNode::Nama { base, vacana, linga, .. } => {
                let role = infer_type_from_suffix(base);
                let ty = vibhakti_to_type(&role, base);
                
                let vacana_str = format!("{:?}", vacana);
                let ts_vacana = vacana_from_str(&vacana_str).unwrap_or(Vacana::Eka);
                
                let linga_str = format!("{:?}", linga);
                let ts_linga = linga_from_str(&linga_str).unwrap_or(Linga::Pullinga);

                let symbol = Symbol::new(base, ty.clone(), &ts_vacana, &ts_linga, "i64");
                self.env.define_symbol(base, symbol);
                ty
            }
            ASTNode::KriyaCall { karta, kriya, karma, .. } => {
                if let Some(subject_node) = karta {
                    if let ASTNode::Nama { base, .. } = &**subject_node {
                        if self.env.lookup(base).is_none() {
                            self.errors.push(TypeCheckError::UndefinedName(base.clone()));
                        }
                    }
                }

                for arg in karma {
                    let arg_type = self.check(arg);
                    match arg_type {
                        DevvaniType::Parameter(_) => {}
                        _ => {
                            self.errors.push(TypeCheckError::TypeMismatch {
                                expected: "Parameter".to_string(),
                                found: format!("{}", arg_type),
                            });
                        }
                    }
                }

                DevvaniType::Subject(kriya.clone())
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
    use devvani_ast::Span;
    use devvani_ast::Vibhakti;
    use devvani_ast::Linga as AstLinga;
    use devvani_ast::Vacana as AstVacana;
    use devvani_ast::Lakara as AstLakara;
    use devvani_ast::Gana;

    fn dummy_span() -> Span {
        Span { line: 0, col: 0, len: 0 }
    }

    #[test]
    fn test_nama_inference() {
        let mut checker = TypeChecker::new();
        let node = ASTNode::Nama {
            base: "Ramah".to_string(),
            vibhakti: Vibhakti::Prathama,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            span: dummy_span(),
        };
        let ty = checker.check(&node);
        assert_eq!(ty, DevvaniType::Subject("Ramah".to_string()));
    }

    #[test]
    fn test_lookup_after_define() {
        let mut checker = TypeChecker::new();
        let ty = DevvaniType::Subject("Ramah".to_string());
        checker.env.define("Ramah", ty.clone());
        assert_eq!(checker.env.lookup_type("Ramah"), Some(&ty));
    }

    #[test]
    fn test_enter_scope_parent_lookup() {
        let mut global_env = TypeEnv::new("global");
        let ty = DevvaniType::Subject("Ramah".to_string());
        global_env.define("Ramah", ty.clone());
        let local_env = global_env.enter_scope("local");
        assert_eq!(local_env.lookup_type("Ramah"), Some(&ty));
    }

    #[test]
    fn test_kriyacall_undefined_subject() {
        let mut checker = TypeChecker::new();
        let node = ASTNode::KriyaCall {
            karta: Some(Box::new(ASTNode::Nama {
                base: "Unknown".to_string(),
                vibhakti: Vibhakti::Prathama,
                linga: AstLinga::Pullinga,
                vacana: AstVacana::Eka,
                span: dummy_span(),
            })),
            kriya: "pathati".to_string(),
            karma: vec![],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: dummy_span(),
        };
        checker.check(&node);
        assert!(!checker.errors.is_empty());
        match &checker.errors[0] {
            TypeCheckError::UndefinedName(name) => assert_eq!(name, "Unknown"),
            _ => panic!("Expected UndefinedName error"),
        }
    }

    #[test]
    fn test_dhatu_def_lakara_async() {
        let mut checker = TypeChecker::new();
        let node = ASTNode::DhatuDef {
            name: "gacchati".to_string(),
            lakara: AstLakara::Lrt,
            gana: Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            body: vec![],
            span: dummy_span(),
        };
        let ty = checker.check(&node);
        assert_eq!(ty, DevvaniType::Subject("Future<gacchati>".to_string()));
        let sym = checker.env.lookup("gacchati").unwrap();
        assert_eq!(sym.devvani_type, DevvaniType::Scope("Async".to_string()));
    }

    #[test]
    fn test_dhatu_def_lakara_vidhilin() {
        let mut checker = TypeChecker::new();
        let node = ASTNode::DhatuDef {
            name: "pateh".to_string(),
            lakara: AstLakara::Vidhilin,
            gana: Gana::Bhvadi,
            linga: AstLinga::Pullinga,
            vacana: AstVacana::Eka,
            params: vec![],
            upasargas: vec![],
            return_karaka: None,
            body: vec![],
            span: dummy_span(),
        };
        let ty = checker.check(&node);
        assert_eq!(ty, DevvaniType::Subject("Result<pateh>".to_string()));
    }
}
