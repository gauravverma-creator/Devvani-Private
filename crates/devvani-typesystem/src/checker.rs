use devvani_ast::ASTNode;
use crate::{vibhakti::*, type_env::TypeEnv, lakara::*, vacana::*, linga::*, symbol::*};
use std::fmt;

#[derive(Debug, Clone)]
pub enum TypeCheckError {
    NaamaApraapta(String),
    PrakaaraVaisamya { expected: String, found: String },
    SatyaasatyaApekshita(String),
    PrakaaraAsangata(String),
    AnavasthaDosha { dhatu_name: String },
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
            TypeCheckError::AnavasthaDosha { dhatu_name } => {
                write!(f, "Anavastha-dosha: '{}' has no reachable base case", dhatu_name)
            }
        }
    }
}

/// Recursively walk a node's children, invoking `f` on each direct child.
fn each_child(node: &ASTNode, f: &mut dyn FnMut(&ASTNode)) {
    match node {
        ASTNode::KaryakramNode { shareera } => shareera.iter().for_each(|n| f(n)),
        ASTNode::DhatuDef { body, .. } => body.iter().for_each(|n| f(n)),
        ASTNode::KriyaCall { karta, karma, karana, sampradana, apadan, adhikarana, .. } => {
            if let Some(k) = karta { f(k); }
            karma.iter().for_each(|n| f(n));
            if let Some(k) = karana { f(k); }
            if let Some(k) = sampradana { f(k); }
            if let Some(k) = apadan { f(k); }
            if let Some(k) = adhikarana { f(k); }
        }
        ASTNode::AstiNode { mulya, .. } => f(mulya),
        ASTNode::BhavatiNode { mulya, .. } => f(mulya),
        ASTNode::YogaNode { vama, dakshina }
        | ASTNode::ViyogaNode { vama, dakshina }
        | ASTNode::GunaNode { vama, dakshina }
        | ASTNode::BhagaNode { vama, dakshina }
        | ASTNode::SamaNode { vama, dakshina }
        | ASTNode::AsamaNode { vama, dakshina }
        | ASTNode::NyuunaNode { vama, dakshina }
        | ASTNode::AdhikaNode { vama, dakshina } => { f(vama); f(dakshina); }
        ASTNode::VadatiNode { mulya } => f(mulya),
        ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
            f(sthiti);
            tarhi.iter().for_each(|n| f(n));
            if let Some(b) = anyatha { b.iter().for_each(|n| f(n)); }
        }
        ASTNode::YavatNode { sthiti, shareera } => { f(sthiti); shareera.iter().for_each(|n| f(n)); }
        ASTNode::PunahNode { varam, shareera } => { f(varam); shareera.iter().for_each(|n| f(n)); }
        ASTNode::Dvandva { members, .. } => members.iter().for_each(|n| f(n)),
        ASTNode::VaakNode { mulya, .. } => f(mulya),
        ASTNode::VaakYogaNode { vama, dakshina, .. } => { f(vama); f(dakshina); }
        ASTNode::Samasa { parts, .. } => parts.iter().for_each(|n| f(n)),
        ASTNode::KritChain { steps, .. } => steps.iter().for_each(|n| f(n)),
        ASTNode::UpasargaApplied { node } => f(&node.target),
        ASTNode::TaddhitaChain { base, .. } => f(base),
        ASTNode::AvartanaNode { call, .. } => f(call),
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
                
                let is_num = |t: &DevvaniType| match t {
                    DevvaniType::Subject(s) => s == "Purnaank" || s == "Dashaamsha" || (s != "Bool" && s != "Vaak" && !s.contains("Future") && !s.contains("Result")),
                    DevvaniType::Parameter(_) => true,
                    _ => false,
                };
                
                if !is_num(&t_vama) || !is_num(&t_dakshina) {
                    self.errors.push(TypeCheckError::PrakaaraAsangata("Arithmetic requires numeric types".to_string()));
                    return DevvaniType::Unknown;
                }
                
                let types_compatible = |t1: &DevvaniType, t2: &DevvaniType| -> bool {
                    if t1 == t2 {
                        return true;
                    }
                    if matches!(t1, DevvaniType::Parameter(_)) || matches!(t2, DevvaniType::Parameter(_)) {
                        return true;
                    }
                    let is_generic = |t: &DevvaniType| match t {
                        DevvaniType::Subject(s) => s != "Purnaank" && s != "Dashaamsha" && s != "Bool" && s != "Vaak" && !s.contains("Future") && !s.contains("Result"),
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
                
                let types_compatible = |t1: &DevvaniType, t2: &DevvaniType| -> bool {
                    if t1 == t2 {
                        return true;
                    }
                    if matches!(t1, DevvaniType::Parameter(_)) || matches!(t2, DevvaniType::Parameter(_)) {
                        return true;
                    }
                    let is_generic = |t: &DevvaniType| match t {
                        DevvaniType::Subject(s) => s != "Purnaank" && s != "Dashaamsha" && s != "Bool" && s != "Vaak" && !s.contains("Future") && !s.contains("Result"),
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

                if !has_reachable_base_case(body) && body.iter().any(contains_avartana) {
                    self.errors.push(TypeCheckError::AnavasthaDosha { dhatu_name: name.clone() });
                }

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
                        DevvaniType::Parameter(_) | DevvaniType::Subject(_) | DevvaniType::Vaak | DevvaniType::VaakBorrow => {}
                        _ => {
                            self.errors.push(TypeCheckError::PrakaaraVaisamya {
                                expected: "Parameter/Subject".to_string(),
                                found: format!("{:?}", arg_type),
                            });
                        }
                    }
                }

                DevvaniType::Subject(kriya.clone())
            }

            ASTNode::AvartanaNode { call, .. } => {
                self.check(call)
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
    use devvani_ast::{
        ASTNode, Gana, Lakara, Linga, Span, Vacana,
    };

    fn span() -> Span {
        Span { line: 0, col: 0, len: 0 }
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
        let body = vec![
            yadi(
                vec![ASTNode::VadatiNode {
                    mulya: Box::new(ASTNode::VaakLiteral {
                        value: "base".to_string(),
                        span: span(),
                    }),
                }],
                Some(vec![avartana("recur")]),
            ),
        ];
        let errors = check_dhatu(body);
        assert!(
            !errors.iter().any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
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
            !errors.iter().any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
            "non-recursive dhatu must not produce AnavasthaDosha, got: {:?}",
            errors
        );
    }

    #[test]
    fn recursive_yadi_both_branhes_recurse_is_conservative() {
        // Has a Yadi but both branches contain recursion -> conservative: no flag.
        let body = vec![yadi(
            vec![avartana("recur")],
            Some(vec![avartana("recur")]),
        )];
        let errors = check_dhatu(body);
        assert!(
            !errors.iter().any(|e| matches!(e, TypeCheckError::AnavasthaDosha { .. })),
            "conservative: both branches recurse but has guard, got: {:?}",
            errors
        );
    }
}
