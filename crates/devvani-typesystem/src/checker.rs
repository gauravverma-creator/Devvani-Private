use crate::{lakara::*, linga::*, symbol::*, type_env::TypeEnv, vacana::*, vibhakti::*};
use devvani_ast::node::KarakaParam;
use devvani_ast::ASTNode;
use std::collections::{HashMap, HashSet};
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
    PhalaVisamgati {
        expected: DevvaniType,
        found: DevvaniType,
    },
    NidanaAparichaya,
    PancakaAvishishtata,
    SamprāptiAyogyatā,
    DoshaAsangati {
        expected: DevvaniType,
        found: DevvaniType,
    },
    PhalaSandarbhaAbhava,
    /// D067 — श्वत्वभङ्ग (SvatvaBhanga): use after ownership transfer
    SvatvaBhanga { name: String },
    /// D068 — अधिकारद्वन्द्व (AdhikaraDvandva): conflicting simultaneous borrows
    AdhikaraDvandva { name: String },
    /// D069 — क्षयानन्तरउपयोग (KshayaAnantaraUpayoga): use after scope exit
    KshayaAnantaraUpayoga { name: String },
    /// D070 — विकारअधिकारद्वय (VikaraAdhikaraDvaya): two simultaneous mutable borrows
    VikaraAdhikaraDvaya { name: String },
    /// D071 — सामान्यअनिश्चितद्वन्द्व (SamanyaAnishchitaDvandva): conflicting generic-type inference
    SamanyaAnishchitaDvandva {
        name: String,
        param_name: String,
        found_type: DevvaniType,
        previous_type: DevvaniType,
    },
    /// D072 — सामान्यअनियता (SamanyaAniyata): uninferable generic parameter
    SamanyaAniyata {
        name: String,
        param_name: String,
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
            TypeCheckError::PhalaVisamgati { expected, found } => {
                write!(f, "Phala-visamgati: expected {:?}, found {:?}", expected, found)
            }
            TypeCheckError::NidanaAparichaya => {
                write!(f, "Nidana-aparichaya: Nidana target not Phalam type")
            }
            TypeCheckError::PancakaAvishishtata => {
                write!(f, "Pancaka-avishishtata: Nidana missing arogya or dosha arm")
            }
            TypeCheckError::SamprāptiAyogyatā => {
                write!(f, "Samprāpti-ayogyatā: Samprapti outside Phalam-returning function")
            }
            TypeCheckError::DoshaAsangati { expected, found } => {
                write!(f, "Dosha-asangati: expected {:?}, found {:?}", expected, found)
            }
            TypeCheckError::PhalaSandarbhaAbhava => {
                write!(f, "Phala-sandarbha-abhava: Arogya/Dosha without Phalam context")
            }
            TypeCheckError::SvatvaBhanga { name } => {
                write!(f, "Svatva-bhanga: ownership (Svatva) of '{}' has been moved away", name)
            }
            TypeCheckError::AdhikaraDvandva { name } => {
                write!(f, "Adhikara-dvandva: conflicting simultaneous borrows of '{}'", name)
            }
            TypeCheckError::KshayaAnantaraUpayoga { name } => {
                write!(f, "Kshaya-anantara-upayoga: use of '{}' after it went out of scope (Kshaya)", name)
            }
            TypeCheckError::VikaraAdhikaraDvaya { name } => {
                write!(f, "Vikara-adhikara-dvaya: two simultaneous mutable borrows of '{}'", name)
            }
            TypeCheckError::SamanyaAnishchitaDvandva { name, param_name, found_type, previous_type } => {
                write!(
                    f,
                    "Samanya-anishchita-dvandva: conflicting inference for generic param '{}' on '{}': found {:?}, but previously inferred {:?}",
                    param_name, name, found_type, previous_type
                )
            }
            TypeCheckError::SamanyaAniyata { name, param_name } => {
                write!(
                    f,
                    "Samanya-aniyata: generic param '{}' on '{}' cannot be inferred from call-site arguments",
                    param_name, name
                )
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
          ASTNode::PhalamType { .. } => {}
          ASTNode::ArogyaNode { value, .. } => f(value),
          ASTNode::DoshaNode { value, .. } => f(value),
ASTNode::NidanaNode { target, arogya_body, dosha_body, .. } => {
               f(target);
               arogya_body.iter().for_each(|n| f(n));
               dosha_body.iter().for_each(|n| f(n));
           }
           ASTNode::SandarbhaNode { target, .. } => f(target),
           ASTNode::SamprapatiNode { expr, .. } => f(expr),
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
    pub current_return_type: Option<DevvaniType>,
    pub current_generic_params: Vec<String>,
    pub nidana_context: Option<(DevvaniType, DevvaniType)>,
    /// Variables whose ownership has been moved
    moved_vars: HashSet<String>,
    /// Active borrows: variable name -> list of (is_mutable) flags
    active_borrows: HashMap<String, Vec<bool>>,
    /// Saved ownership state for scope nesting (DhatuDef, Kramashah, Nidana)
    moved_vars_stack: Vec<HashSet<String>>,
    active_borrows_stack: Vec<HashMap<String, Vec<bool>>>,
    /// Registry of DhatuDef parameter metadata keyed by function name
    function_params: HashMap<String, Vec<KarakaParam>>,
    /// Registry of DhatuDef return types keyed by function name
    function_return_types: HashMap<String, DevvaniType>,
    /// Registry of DhatuDef generic params keyed by function name
    function_generic_params: HashMap<String, Vec<String>>,
    /// Variables declared in scopes that have closed (accumulated across all scope pops)
    closed_scope_vars: HashSet<String>,
    /// Variables declared in the current scope (tracked per-scope via stack)
    current_scope_vars: HashSet<String>,
    /// Stack for saving current_scope_vars across nested scopes
    current_scope_vars_stack: Vec<HashSet<String>>,
}

    impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new("global"),
            errors: Vec::new(),
            current_lakara: None,
            current_return_type: None,
            current_generic_params: Vec::new(),
            nidana_context: None,
            moved_vars: HashSet::new(),
            active_borrows: HashMap::new(),
            moved_vars_stack: Vec::new(),
            active_borrows_stack: Vec::new(),
            function_params: HashMap::new(),
            function_return_types: HashMap::new(),
            function_generic_params: HashMap::new(),
            closed_scope_vars: HashSet::new(),
            current_scope_vars: HashSet::new(),
            current_scope_vars_stack: Vec::new(),
        }
    }

    /// Public accessor for the function parameters registry
    pub fn function_params(&self) -> &HashMap<String, Vec<KarakaParam>> {
        &self.function_params
    }

    /// Public mutable accessor for the function parameters registry
    pub fn function_params_mut(&mut self) -> &mut HashMap<String, Vec<KarakaParam>> {
        &mut self.function_params
    }

    /// Public accessor for the function return types registry
    pub fn function_return_types(&self) -> &HashMap<String, DevvaniType> {
        &self.function_return_types
    }

    /// Public accessor for the function generic params registry
    pub fn function_generic_params(&self) -> &HashMap<String, Vec<String>> {
        &self.function_generic_params
    }

    /// Resolve a Devvani type name to its DevvaniType representation.
    /// Checks generic params first, then environment, then built-in primitives.
    fn resolve_type_name(&self, type_name: &str) -> Option<DevvaniType> {
        if self.current_generic_params.contains(&type_name.to_string()) {
            return Some(DevvaniType::Samanya(type_name.to_string()));
        }
        if let Some(sym) = self.env.lookup(type_name) {
            return Some(sym.devvani_type.clone());
        }
        match type_name {
            "sankhya" | "purnaank" => Some(DevvaniType::Subject("Purnaank".to_string())),
            "dashaamsha" => Some(DevvaniType::Subject("Dashaamsha".to_string())),
            "vaak" => Some(DevvaniType::Vaak),
            _ => None,
        }
    }

    /// Returns true if a type is non-Copy (requires move semantics).
    /// Primitive numeric and boolean types are Copy; all others are not.
    fn is_non_copy_type(ty: &DevvaniType) -> bool {
        !matches!(
            ty,
            DevvaniType::Subject(s) if s == "Purnaank" || s == "Dashaamsha" || s == "Bool"
        )
    }

    /// Recursively collect all generic param names (Samanya) from a DevvaniType.
    fn collect_samanya_from_type(ty: &DevvaniType) -> Vec<String> {
        match ty {
            DevvaniType::Samanya(name) => vec![name.clone()],
            DevvaniType::Dravya(_, angas) => angas
                .iter()
                .flat_map(|(_, t)| Self::collect_samanya_from_type(t))
                .collect(),
            DevvaniType::Phalam(success, error) => Self::collect_samanya_from_type(success)
                .into_iter()
                .chain(Self::collect_samanya_from_type(error))
                .collect(),
            DevvaniType::Pankti(elem, _) => Self::collect_samanya_from_type(elem),
            DevvaniType::Avali(elem) => Self::collect_samanya_from_type(elem),
            DevvaniType::Sandarbha(inner, _) => Self::collect_samanya_from_type(inner),
            _ => Vec::new(),
        }
    }

    /// Substitute generic Samanya params in a DevvaniType using the inference map.
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
            DevvaniType::Sandarbha(inner, mut_) => DevvaniType::Sandarbha(
                Box::new(Self::substitute_samanya_in_type(*inner, inference)),
                mut_,
            ),
            other => other,
        }
    }

    /// Check an identifier (by name) for use-after-move. Returns the type
    /// if valid, or Unknown if the variable has been moved.
    fn check_identifier_use(&mut self, name: &str) -> DevvaniType {
        if self.moved_vars.contains(name) {
            self.errors.push(TypeCheckError::SvatvaBhanga {
                name: name.to_string(),
            });
            return DevvaniType::Unknown;
        }
        // Check closed_scope_vars BEFORE env.lookup to enforce block-level scoping
        // and emit D069 for variables that were declared in closed scopes
        if self.closed_scope_vars.contains(name) {
            self.errors.push(TypeCheckError::KshayaAnantaraUpayoga {
                name: name.to_string(),
            });
            return DevvaniType::Unknown;
        }
        if let Some(sym) = self.env.lookup(name) {
            sym.devvani_type.clone()
        } else {
            let role = infer_type_from_suffix(name);
            vibhakti_to_type(&role, name)
        }
    }

    /// Push ownership state onto the stack before entering a scope.
    fn push_ownership_state(&mut self) {
        self.moved_vars_stack.push(std::mem::take(&mut self.moved_vars));
        self.active_borrows_stack.push(std::mem::take(&mut self.active_borrows));
        self.current_scope_vars_stack.push(std::mem::take(&mut self.current_scope_vars));
    }

    /// Pop ownership state from the stack when exiting a scope.
    fn pop_ownership_state(&mut self) {
        self.moved_vars = self.moved_vars_stack.pop().unwrap_or_default();
        self.active_borrows = self.active_borrows_stack.pop().unwrap_or_default();
        // Add all variables declared in this scope to closed_scope_vars
        self.closed_scope_vars.extend(std::mem::take(&mut self.current_scope_vars));
        self.current_scope_vars = self.current_scope_vars_stack.pop().unwrap_or_default();
    }

    /// Reset ownership state for a fresh function body (DhatuDef).
    fn reset_ownership_state(&mut self) {
        self.moved_vars.clear();
        self.active_borrows.clear();
        self.closed_scope_vars.clear();
        self.current_scope_vars.clear();
    }

    pub fn check(&mut self, node: &ASTNode) -> DevvaniType {
        match node {
            ASTNode::KaryakramNode { shareera, .. } => {
                self.push_ownership_state();
                let mut last_type = DevvaniType::Unknown;
                for stmt in shareera {
                    last_type = self.check(stmt);
                }
                self.pop_ownership_state();
                last_type
            }
            ASTNode::Nama { base, .. } => {
                self.check_identifier_use(base)
            }
            ASTNode::PurnaankLiteral { .. } => DevvaniType::Subject("Purnaank".to_string()),
            ASTNode::DashaamshaLiteral { .. } => DevvaniType::Subject("Dashaamsha".to_string()),
            ASTNode::VaakLiteral { .. } => DevvaniType::Subject("Vaak".to_string()),

            ASTNode::AstiNode { naama, mulya } | ASTNode::BhavatiNode { naama, mulya } => {
                let ty = self.check(mulya);
                if let ASTNode::Nama { base, .. } = mulya.as_ref() {
                    if Self::is_non_copy_type(&ty) {
                        self.moved_vars.insert(base.clone());
                    }
                }
                let symbol = Symbol::new(naama, ty.clone(), &Vacana::Eka, &Linga::Pullinga, "var");
                self.env.define_symbol(naama, symbol);
                self.current_scope_vars.insert(naama.clone());
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
                self.current_scope_vars.insert(naama.clone());
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
                generic_params,
                params,
                body,
                return_type,
                lakara,
                ..
            } => {
                let l_str = format!("{:?}", lakara);
                let typesystem_lakara = lakara_from_str(&l_str).unwrap_or(Lakara::Lat);

                let old_lakara = self.current_lakara.clone();
                self.current_lakara = Some(typesystem_lakara.clone());

                let old_return_type = self.current_return_type.clone();
                let old_generic_params = self.current_generic_params.clone();
                self.current_generic_params = generic_params.clone();
                if let Some(rt) = return_type {
                    self.current_return_type = Some(self.check(rt));
                } else {
                    self.current_return_type = None;
                }

                self.reset_ownership_state();
                self.push_ownership_state();

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
                    let ty = if self.current_generic_params.contains(&param.type_name) {
                        DevvaniType::Samanya(param.type_name.clone())
                    } else {
                        DevvaniType::Parameter(param.name.clone())
                    };
                    let param_symbol =
                        Symbol::new(&param.name, ty, &Vacana::Eka, &Linga::Pullinga, "i64");
                    self.env.define_symbol(&param.name, param_symbol);
                    self.current_scope_vars.insert(param.name.clone());
                }
                self.function_params.insert(name.clone(), params.clone());

                for stmt in body {
                    self.check(stmt);
                }

                if let Some(rt) = &self.current_return_type {
                    self.function_return_types.insert(name.clone(), rt.clone());
                }
                self.function_generic_params
                    .insert(name.clone(), generic_params.clone());

                self.env = old_env;
                self.pop_ownership_state();
                self.current_lakara = old_lakara;
                self.current_return_type = old_return_type;
                self.current_generic_params = old_generic_params;

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

            ASTNode::DravyaDef { name, generic_params, angas, .. } => {
                let old_generic_params = self.current_generic_params.clone();
                self.current_generic_params = generic_params.clone();

                let mut resolved_angas: Vec<(String, DevvaniType)> = Vec::new();
                for anga in angas {
                    match self.resolve_type_name(&anga.type_name) {
                        Some(ty) => resolved_angas.push((anga.name.clone(), ty)),
                        None => {
                            self.errors.push(TypeCheckError::DravyaApariyata {
                                name: anga.type_name.clone(),
                            });
                            self.current_generic_params = old_generic_params;
                            return DevvaniType::Unknown;
                        }
                    }
                }

                self.current_generic_params = old_generic_params;
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

                let has_samanya = angas.iter().any(|(_, ty)| matches!(ty, DevvaniType::Samanya(_)));
                if has_samanya {
                    let mut inference: HashMap<String, DevvaniType> = HashMap::new();
                    let mut resolved_angas: Vec<(String, DevvaniType)> = Vec::new();

                    for (i, (anga_name, expected_ty)) in angas.iter().enumerate() {
                        let found_ty = self.check(&values[i]);

                        if let DevvaniType::Samanya(param_name) = expected_ty {
                            if let Some(previous_ty) = inference.get(param_name) {
                                if *previous_ty != found_ty {
                                    self.errors.push(TypeCheckError::SamanyaAnishchitaDvandva {
                                        name: dravya_name.clone(),
                                        param_name: param_name.clone(),
                                        found_type: found_ty,
                                        previous_type: previous_ty.clone(),
                                    });
                                    return DevvaniType::Unknown;
                                }
                            } else {
                                inference.insert(param_name.clone(), found_ty.clone());
                            }
                            resolved_angas.push((anga_name.clone(), found_ty.clone()));
                        } else {
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
                            resolved_angas.push((anga_name.clone(), expected_ty.clone()));
                        }
                    }

                    DevvaniType::Dravya(dravya_name.clone(), resolved_angas)
                } else {
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

                let mut all_args: Vec<&ASTNode> = Vec::new();
                if let Some(k) = karta.as_ref() {
                    all_args.push(k);
                }
                for a in karma.iter() {
                    all_args.push(a);
                }
                if let Some(k) = karana.as_ref() {
                    all_args.push(k);
                }
                if let Some(s) = sampradana.as_ref() {
                    all_args.push(s);
                }
                if let Some(a) = apadan.as_ref() {
                    all_args.push(a);
                }
                if let Some(a) = adhikarana.as_ref() {
                    all_args.push(a);
                }
                let arg_types: Vec<DevvaniType> = all_args.iter().map(|a| self.check(a)).collect();

                if let Some(params) = self.function_params.get(kriya) {
                    for (i, param) in params.iter().enumerate() {
                        if i >= arg_types.len() {
                            break;
                        }
                        if param.is_borrowed {
                            continue;
                        }
                        if Self::is_non_copy_type(&arg_types[i]) {
                            if let ASTNode::Nama { base, .. } = all_args[i] {
                                self.moved_vars.insert(base.clone());
                            }
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
                            if !karma.is_empty() {
                                self.errors.push(TypeCheckError::ApakarshanaAprayukta {
                                    found: t_karta.clone(),
                                });
                                return DevvaniType::Unknown;
                            }
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

                let is_generic = self
                    .function_generic_params
                    .get(kriya)
                    .map(|p| !p.is_empty())
                    .unwrap_or(false);

                if is_generic {
                    let generic_params_set: HashSet<String> = self
                        .function_generic_params
                        .get(kriya)
                        .map(|p| p.iter().cloned().collect())
                        .unwrap_or_default();
                    let mut inference: HashMap<String, DevvaniType> = HashMap::new();

                    if let Some(params) = self.function_params.get(kriya) {
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
                                        self.errors.push(TypeCheckError::SamanyaAnishchitaDvandva {
                                            name: kriya.clone(),
                                            param_name: param_type_name.clone(),
                                            found_type: arg_types[i].clone(),
                                            previous_type: previous_ty.clone(),
                                        });
                                        return DevvaniType::Unknown;
                                    }
                                } else {
                                    inference.insert(param_type_name.clone(), arg_types[i].clone());
                                }
                            }
                        }
                    }

                    if let Some(declared_return) = self.function_return_types.get(kriya) {
                        let needed_generic = Self::collect_samanya_from_type(declared_return);
                        let needed_set: HashSet<String> = needed_generic.iter().cloned().collect();
                        let inference_keys: HashSet<String> = inference.keys().cloned().collect();
                        let not_found: Vec<String> = needed_set
                            .difference(&inference_keys)
                            .cloned()
                            .collect();
                        if !not_found.is_empty() {
                            for missing in &not_found {
                                self.errors.push(TypeCheckError::SamanyaAniyata {
                                    name: kriya.clone(),
                                    param_name: missing.clone(),
                                });
                            }
                            return DevvaniType::Unknown;
                        }
                        return Self::substitute_samanya_in_type(declared_return.clone(), &inference);
                    }

                    DevvaniType::Subject(kriya.clone())
                } else {
                    DevvaniType::Subject(kriya.clone())
                }
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
                self.push_ownership_state();
                self.env = self.env.enter_scope(item_name);
                let item_symbol = Symbol::new(
                    item_name,
                    elem_ty,
                    &Vacana::Eka,
                    &Linga::Pullinga,
                    "i64",
                );
                self.env.define_symbol(item_name, item_symbol);
                self.current_scope_vars.insert(item_name.clone());
                for stmt in body {
                    self.check(stmt);
                }
                self.env = old_env;
                self.pop_ownership_state();
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

            ASTNode::PhalamType { success_type, error_type, .. } => {
                let success = self.resolve_type_name(success_type)
                    .unwrap_or_else(|| DevvaniType::Subject(success_type.clone()));
                let error = self.resolve_type_name(error_type)
                    .unwrap_or_else(|| DevvaniType::Subject(error_type.clone()));
                DevvaniType::Phalam(Box::new(success), Box::new(error))
            }

            ASTNode::ArogyaNode { value, .. } => {
                let value_type = self.check(value);
                let phalam_context = self.nidana_context.clone().or_else(|| {
                    match &self.current_return_type {
                        Some(DevvaniType::Phalam(success, error)) => {
                            Some(((**success).clone(), (**error).clone()))
                        }
                        _ => None,
                    }
                });

                if let Some((expected_success, _)) = phalam_context {
                    if !matches!(value_type, DevvaniType::Unknown) && value_type != expected_success {
                        self.errors.push(TypeCheckError::PhalaVisamgati {
                            expected: expected_success,
                            found: value_type.clone(),
                        });
                    }
                    value_type
                } else {
                    self.errors.push(TypeCheckError::PhalaSandarbhaAbhava);
                    DevvaniType::Unknown
                }
            }

            ASTNode::DoshaNode { value, .. } => {
                let value_type = self.check(value);
                let phalam_context = self.nidana_context.clone().or_else(|| {
                    match &self.current_return_type {
                        Some(DevvaniType::Phalam(success, error)) => {
                            Some(((**success).clone(), (**error).clone()))
                        }
                        _ => None,
                    }
                });

                if let Some((_, expected_error)) = phalam_context {
                    if !matches!(value_type, DevvaniType::Unknown) && value_type != expected_error {
                        self.errors.push(TypeCheckError::PhalaVisamgati {
                            expected: expected_error,
                            found: value_type.clone(),
                        });
                    }
                    value_type
                } else {
                    self.errors.push(TypeCheckError::PhalaSandarbhaAbhava);
                    DevvaniType::Unknown
                }
            }

            ASTNode::NidanaNode { target, arogya_bind, arogya_body, dosha_bind, dosha_body, .. } => {
                let target_type = self.check(target);
                let (success_ty, error_ty) = match target_type {
                    DevvaniType::Phalam(success, error) => (success, error),
                    _ => {
                        self.errors.push(TypeCheckError::NidanaAparichaya);
                        return DevvaniType::Unknown;
                    }
                };

                if arogya_body.is_empty() || dosha_body.is_empty() {
                    self.errors.push(TypeCheckError::PancakaAvishishtata);
                    return DevvaniType::Unknown;
                }

                let old_nidana = self.nidana_context.clone();
                self.nidana_context = Some((*success_ty.clone(), *error_ty.clone()));

                let old_env = self.env.clone();
                self.push_ownership_state();
                self.env = self.env.enter_scope("nidana_arogya");
                let arogya_symbol = Symbol::new(
                    arogya_bind,
                    *success_ty.clone(),
                    &Vacana::Eka,
                    &Linga::Pullinga,
                    "var",
                );
                self.env.define_symbol(arogya_bind, arogya_symbol);
                self.current_scope_vars.insert(arogya_bind.clone());
                for stmt in arogya_body {
                    self.check(stmt);
                }
                self.env = old_env;
                self.pop_ownership_state();

                let old_env = self.env.clone();
                self.push_ownership_state();
                self.env = self.env.enter_scope("nidana_dosha");
                let dosha_symbol = Symbol::new(
                    dosha_bind,
                    *error_ty.clone(),
                    &Vacana::Eka,
                    &Linga::Pullinga,
                    "var",
                );
                self.env.define_symbol(dosha_bind, dosha_symbol);
                self.current_scope_vars.insert(dosha_bind.clone());
                for stmt in dosha_body {
                    self.check(stmt);
                }
                self.env = old_env;
                self.pop_ownership_state();

                self.nidana_context = old_nidana;

                DevvaniType::Unknown
            }

ASTNode::SamprapatiNode { expr, .. } => {
                 let expr_type = self.check(expr);
                 match expr_type {
                     DevvaniType::Phalam(success_ty, error_ty) => {
                         match &self.current_return_type {
                             Some(DevvaniType::Phalam(_cur_success, cur_error)) => {
                                 let compatible = *error_ty == **cur_error
                                     || matches!(*error_ty, DevvaniType::Unknown)
                                     || matches!(**cur_error, DevvaniType::Unknown);
                                 if !compatible {
                                     self.errors.push(TypeCheckError::DoshaAsangati {
                                         expected: (**cur_error).clone(),
                                         found: (*error_ty).clone(),
                                     });
                                 }
                                 (*success_ty).clone()
                             }
                             _ => {
                                 self.errors.push(TypeCheckError::SamprāptiAyogyatā);
                                 DevvaniType::Unknown
                             }
                         }
                     }
                     _ => DevvaniType::Unknown,
                 }
             }

             ASTNode::SandarbhaNode { target, is_mutable, .. } => {
                 let target_type = self.check(target);
                 let borrow_is_mutable = *is_mutable;
                 let target_name = if let ASTNode::Nama { base, .. } = target.as_ref() {
                     Some(base.clone())
                 } else {
                     None
                 };
                 if let Some(ref name) = target_name {
                     if let Some(borrows) = self.active_borrows.get(name) {
                         let has_mutable = borrows.iter().any(|&b| b);
                         let has_immutable = borrows.iter().any(|&b| !b);
                         if borrow_is_mutable {
                             if has_mutable {
                                 self.errors.push(TypeCheckError::VikaraAdhikaraDvaya {
                                     name: name.clone(),
                                 });
                             } else if has_immutable {
                                 self.errors.push(TypeCheckError::AdhikaraDvandva {
                                     name: name.clone(),
                                 });
                             }
                         } else {
                             if has_mutable {
                                 self.errors.push(TypeCheckError::AdhikaraDvandva {
                                     name: name.clone(),
                                 });
                             }
                         }
                     }
                     self.active_borrows
                         .entry(name.clone())
                         .or_default()
                         .push(borrow_is_mutable);
                 }
                 DevvaniType::Sandarbha(Box::new(target_type), borrow_is_mutable)
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
             generic_params: vec![],
             lakara: Lakara::Lat,
             gana: Gana::Bhvadi,
             linga: Linga::Pullinga,
             vacana: Vacana::Eka,
             params: vec![],
             upasargas: vec![],
             return_karaka: None,
             return_type: None,
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
             generic_params: vec![],
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

    // Phalam (ErrorHandling) type system tests

    fn dhatu_def_with_return(
        name: &str,
        body: Vec<ASTNode>,
        return_type: Option<ASTNode>,
    ) -> ASTNode {
ASTNode::DhatuDef {
             name: name.to_string(),
             generic_params: vec![],
             lakara: Lakara::Lat,
             gana: Gana::Bhvadi,
             linga: Linga::Pullinga,
             vacana: Vacana::Eka,
             params: vec![],
             upasargas: vec![],
             return_karaka: None,
             return_type: return_type.map(Box::new),
             body,
             span: span(),
         }
     }

    #[test]
    fn test_phalam_type_resolution_basic() {
        let mut checker = TypeChecker::new();
        let phalam = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "vaak".to_string(),
            span: span(),
        };
        let ty = checker.check(&phalam);
        assert_eq!(
            ty,
            DevvaniType::Phalam(
                Box::new(DevvaniType::Subject("Purnaank".to_string())),
                Box::new(DevvaniType::Vaak)
            )
        );
    }

    #[test]
    fn test_phalam_type_resolution_nested() {
        let mut checker = TypeChecker::new();
        let inner = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "dashaamsha".to_string(),
            span: span(),
        };
        let outer = ASTNode::PhalamType {
            success_type: "custom".to_string(),
            error_type: "phalam_error".to_string(),
            span: span(),
        };
        let _ = checker.check(&inner);
        checker.check(&outer);
    }

    #[test]
    fn test_arogya_valid_inside_nidana() {
        let mut checker = TypeChecker::new();
        let target = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "Vaak".to_string(),
            span: span(),
        };
        let arogya = ASTNode::ArogyaNode {
            value: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            span: span(),
        };
        let dosha = ASTNode::DoshaNode {
            value: Box::new(ASTNode::VaakLiteral {
                value: "error".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let nidana = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "sukha".to_string(),
            arogya_body: vec![arogya],
            dosha_bind: "duhkha".to_string(),
            dosha_body: vec![dosha],
            span: span(),
        };
        checker.check(&nidana);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_arogya_mismatch_triggers_d061() {
        let mut checker = TypeChecker::new();
        let target = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "vaak".to_string(),
            span: span(),
        };
        let arogya = ASTNode::ArogyaNode {
            value: Box::new(ASTNode::VaakLiteral {
                value: "bad".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let nidana = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "sukha".to_string(),
            arogya_body: vec![arogya],
            dosha_bind: "duhkha".to_string(),
            dosha_body: vec![ASTNode::VaakLiteral {
                value: "error".to_string(),
                span: span(),
            }],
            span: span(),
        };
        checker.check(&nidana);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PhalaVisamgati { .. })),
            "expected PhalaVisamgati error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_arogya_no_enclosing_phalam_triggers_d066() {
        let mut checker = TypeChecker::new();
        let arogya = ASTNode::ArogyaNode {
            value: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            span: span(),
        };
        checker.check(&arogya);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PhalaSandarbhaAbhava)),
            "expected PhalaSandarbhaAbhava error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_dosha_no_enclosing_phalam_triggers_d066() {
        let mut checker = TypeChecker::new();
        let dosha = ASTNode::DoshaNode {
            value: Box::new(ASTNode::VaakLiteral {
                value: "err".to_string(),
                span: span(),
            }),
            span: span(),
        };
        checker.check(&dosha);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PhalaSandarbhaAbhava)),
            "expected PhalaSandarbhaAbhava error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_nidana_empty_arm_triggers_d063() {
        let mut checker = TypeChecker::new();
        let target = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "vaak".to_string(),
            span: span(),
        };
        let nidana = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "sukha".to_string(),
            arogya_body: vec![],
            dosha_bind: "duhkha".to_string(),
            dosha_body: vec![ASTNode::VaakLiteral {
                value: "err".to_string(),
                span: span(),
            }],
            span: span(),
        };
        checker.check(&nidana);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PancakaAvishishtata)),
            "expected PancakaAvishishtata error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_nidana_target_not_phalam_triggers_d062() {
        let mut checker = TypeChecker::new();
        let target = ASTNode::PurnaankLiteral { value: 5, span: span() };
        let nidana = ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind: "sukha".to_string(),
            arogya_body: vec![ASTNode::VaakLiteral {
                value: "ok".to_string(),
                span: span(),
            }],
            dosha_bind: "duhkha".to_string(),
            dosha_body: vec![ASTNode::VaakLiteral {
                value: "err".to_string(),
                span: span(),
            }],
            span: span(),
        };
        checker.check(&nidana);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::NidanaAparichaya)),
            "expected NidanaAparichaya error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_samprapti_inside_phalam_function_valid() {
        let mut checker = TypeChecker::new();
        let phalam = ASTNode::PhalamType {
            success_type: "sankhya".to_string(),
            error_type: "vaak".to_string(),
            span: span(),
        };
        checker.current_return_type = Some(checker.check(&phalam));
        let samprapti = ASTNode::SamprapatiNode {
            expr: Box::new(phalam),
            span: span(),
        };
        let ty = checker.check(&samprapti);
        assert_eq!(ty, DevvaniType::Subject("Purnaank".to_string()));
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_samprapti_inside_non_phalam_function_triggers_d064() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::SamprapatiNode {
            expr: Box::new(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "vaak".to_string(),
                span: span(),
            }),
            span: span(),
        }];
        let dhatu = dhatu_def_with_return(
            "bhojan",
            body,
            Some(ASTNode::VaakLiteral {
                value: "unknown".to_string(),
                span: span(),
            }),
        );
        let _ty = checker.check(&dhatu);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SamprāptiAyogyatā)),
            "expected SamprāptiAyogyatā error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_samprapti_incompatible_error_type_triggers_d065() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::SamprapatiNode {
            expr: Box::new(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "vaak".to_string(),
                span: span(),
            }),
            span: span(),
        }];
        let dhatu = dhatu_def_with_return(
            "bhojan",
            body,
            Some(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "dashaamsha".to_string(),
                span: span(),
            }),
        );
        let _ty = checker.check(&dhatu);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DoshaAsangati { .. })),
            "expected DoshaAsangati error, got: {:?}",
            checker.errors
        );
    }

    // ===== Ownership (Svatva/Adhikara/Kshaya) Tests =====

    #[test]
    fn test_move_use_after_move_d067() {
        // Declare src with Vaak type, then move it to dst, then use src again
        let body = vec![
            ASTNode::AstiNode {
                naama: "src".to_string(),
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "hello".to_string(),
                    span: span(),
                }),
            },
            ASTNode::AstiNode {
                naama: "dst".to_string(),
                mulya: Box::new(ASTNode::Nama {
                    base: "src".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            },
            ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::Nama {
                    base: "src".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            },
        ];
        let errors = check_dhatu(body);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SvatvaBhanga { .. })),
            "expected SvatvaBhanga D067 after moving 'src', got: {:?}",
            errors
        );
    }

    #[test]
    fn test_borrow_of_moved_var_d067() {
        // Declare src with Vaak type, move it to dst, then borrow src again
        let body = vec![
            ASTNode::AstiNode {
                naama: "src".to_string(),
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "hello".to_string(),
                    span: span(),
                }),
            },
            ASTNode::AstiNode {
                naama: "dst".to_string(),
                mulya: Box::new(ASTNode::Nama {
                    base: "src".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
            },
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "src".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: false,
                span: span(),
            },
        ];
        let errors = check_dhatu(body);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SvatvaBhanga { .. })),
            "expected SvatvaBhanga D067 when borrowing moved var 'src', got: {:?}",
            errors
        );
    }

    #[test]
    fn test_two_immutable_borrows_ok() {
        let body = vec![
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: false,
                span: span(),
            },
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: false,
                span: span(),
            },
        ];
        let errors = check_dhatu(body);
        let ownership_errors: Vec<_> = errors
            .iter()
            .filter(|e| matches!(e, TypeCheckError::AdhikaraDvandva { .. } | TypeCheckError::VikaraAdhikaraDvaya { .. } | TypeCheckError::SvatvaBhanga { .. }))
            .collect();
        assert!(
            ownership_errors.is_empty(),
            "expected no ownership errors for two immutable borrows, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_two_mutable_borrows_d070() {
        let body = vec![
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: true,
                span: span(),
            },
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: true,
                span: span(),
            },
        ];
        let errors = check_dhatu(body);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::VikaraAdhikaraDvaya { .. })),
            "expected VikaraAdhikaraDvaya D070 for two mutable borrows of 'x', got: {:?}",
            errors
        );
    }

    #[test]
    fn test_mutable_borrow_then_immutable_d068() {
        let body = vec![
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: true,
                span: span(),
            },
            ASTNode::SandarbhaNode {
                target: Box::new(ASTNode::Nama {
                    base: "x".to_string(),
                    vibhakti: devvani_ast::Vibhakti::Prathama,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    span: span(),
                }),
                is_mutable: false,
                span: span(),
            },
        ];
        let errors = check_dhatu(body);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AdhikaraDvandva { .. })),
            "expected AdhikaraDvandva D068 for mutable+immutable borrow conflict, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_borrowed_param_not_moved() {
        // Function with borrowed param 'x', called with 'src'. After call, src should not be moved.
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: true,
            is_mutable_borrow: false,
            span: span(),
            type_name: "sankhya".to_string(),
        }];
        let use_x = ASTNode::DhatuDef {
            name: "use_x".to_string(),
            generic_params: vec![],
            lakara: Lakara::Lat,
            gana: Gana::Bhvadi,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params,
            upasargas: vec![],
            return_karaka: None,
            return_type: None,
            body: vec![],
            span: span(),
        };
        let mut checker = TypeChecker::new();
        // Check the function definition first (registers params)
        let _ = checker.check(&use_x);
        // Now call use_x(src)
        let call = ASTNode::KriyaCall {
            karta: None,
            kriya: "use_x".to_string(),
            karma: vec![ASTNode::Nama {
                base: "src".to_string(),
                vibhakti: devvani_ast::Vibhakti::Dvitiya,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let _ty = checker.check(&call);
        let ownership_errors: Vec<_> = checker
            .errors
            .iter()
            .filter(|e| matches!(e, TypeCheckError::SvatvaBhanga { .. }))
            .collect();
        assert!(
            ownership_errors.is_empty(),
            "expected no SvatvaBhanga for borrowed param 'src', got: {:?}",
            checker.errors
        );
        assert!(
            !checker.moved_vars.contains("src"),
            "'src' should not be moved when passed to borrowed param"
        );
    }

    #[test]
    fn test_non_borrowed_param_moves_caller_var() {
        // Function with non-borrowed param 'x', called with 'src'. After call, src should be moved.
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            span: span(),
type_name: "sankhya".to_string(),
         }];
         let use_x = ASTNode::DhatuDef {
             name: "use_x".to_string(),
             generic_params: vec![],
             lakara: Lakara::Lat,
             gana: Gana::Bhvadi,
             linga: Linga::Pullinga,
             vacana: Vacana::Eka,
             params,
             upasargas: vec![],
             return_karaka: None,
             return_type: None,
             body: vec![],
             span: span(),
         };
         let mut checker = TypeChecker::new();
         // Check the function definition first (registers params)
         let _ = checker.check(&use_x);
         // Declare src with non-Copy type (Vaak)
        checker.check(&ASTNode::AstiNode {
            naama: "src".to_string(),
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
        });
        // Now call use_x(src) — src should be moved into the function
        let call = ASTNode::KriyaCall {
            karta: None,
            kriya: "use_x".to_string(),
            karma: vec![ASTNode::Nama {
                base: "src".to_string(),
                vibhakti: devvani_ast::Vibhakti::Dvitiya,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let _ty = checker.check(&call);
        assert!(
            checker.moved_vars.contains("src"),
            "'src' should be in moved_vars after being passed to non-borrowed param, got moved_vars: {:?}",
            checker.moved_vars
        );
    }

    // ===== D069 KshayaAnantaraUpayoga Tests =====

    #[test]
    fn test_use_after_scope_exit_d069() {
        // Declare a variable inside a KaryakramNode (block), then use it after the block ends
        // This should emit D069 KshayaAnantaraUpayoga
        let body = vec![
            ASTNode::KaryakramNode {
                shareera: vec![
                    ASTNode::AstiNode {
                        naama: "inner_var".to_string(),
                        mulya: Box::new(ASTNode::PurnaankLiteral {
                            value: 42,
                            span: span(),
                        }),
                    },
                ],
            },
            ASTNode::Nama {
                base: "inner_var".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            },
        ];
        let errors = check_dhatu(body);
        assert!(
            errors.iter().any(|e| matches!(e, TypeCheckError::KshayaAnantaraUpayoga { .. })),
            "expected KshayaAnantaraUpayoga D069 for use of 'inner_var' after scope exit, got: {:?}",
            errors
        );
    }

    // ===== Sāmānya (Generic) Part 2A Tests =====

    fn generic_dravya_def(name: &str, generic_params: Vec<&str>, angas: Vec<AngaField>) -> ASTNode {
        ASTNode::DravyaDef {
            name: name.to_string(),
            generic_params: generic_params.into_iter().map(|s| s.to_string()).collect(),
            angas,
            span: span(),
        }
    }

    fn generic_dhatu_def(
        name: &str,
        generic_params: Vec<&str>,
        params: Vec<KarakaParam>,
        return_type: Option<ASTNode>,
        body: Vec<ASTNode>,
    ) -> ASTNode {
        ASTNode::DhatuDef {
            name: name.to_string(),
            generic_params: generic_params.into_iter().map(|s| s.to_string()).collect(),
            lakara: Lakara::Lat,
            gana: Gana::Bhvadi,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params,
            upasargas: vec![],
            return_karaka: None,
            return_type: return_type.map(Box::new),
            body,
            span: span(),
        }
    }

    // (a) Generic Dravya with single param T

    #[test]
    fn test_generic_dravya_single_param_resolves_to_samanya() {
        let mut checker = TypeChecker::new();
        let def = generic_dravya_def(
            "Peti",
            vec!["T"],
            vec![anga_field("mulya", "T")],
        );
        let ty = checker.check(&def);
        assert!(matches!(ty, DevvaniType::Dravya(_, _)));
        if let DevvaniType::Dravya(name, angas) = &ty {
            assert_eq!(name, "Peti");
            assert_eq!(angas.len(), 1);
            assert_eq!(angas[0], ("mulya".to_string(), DevvaniType::Samanya("T".to_string())));
        }
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    // (b) Generic Dravya with two params T and U

    #[test]
    fn test_generic_dravya_two_params_resolve_correctly() {
        let mut checker = TypeChecker::new();
        let def = generic_dravya_def(
            "Joduha",
            vec!["T", "U"],
            vec![anga_field("pahila", "T"), anga_field("dusara", "U")],
        );
        let ty = checker.check(&def);
        assert!(matches!(ty, DevvaniType::Dravya(_, _)));
        if let DevvaniType::Dravya(name, angas) = &ty {
            assert_eq!(name, "Joduha");
            assert_eq!(angas.len(), 2);
            assert_eq!(angas[0], ("pahila".to_string(), DevvaniType::Samanya("T".to_string())));
            assert_eq!(angas[1], ("dusara".to_string(), DevvaniType::Samanya("U".to_string())));
        }
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    // (c) Generic Dhātu with param T and return type T

    #[test]
    fn test_generic_dhatu_param_and_return_resolve_to_samanya() {
        let mut checker = TypeChecker::new();
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            type_name: "T".to_string(),
            span: span(),
        }];
        let dhatu = generic_dhatu_def(
            "pratirupa",
            vec!["T"],
            params,
            Some(ASTNode::PhalamType {
                success_type: "T".to_string(),
                error_type: "Vaak".to_string(),
                span: span(),
            }),
            vec![],
        );
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    // (d) Arithmetic on Samanya-typed param produces PrakaaraAsangata

    #[test]
    fn test_generic_dhatu_yoga_on_samanya_param_produces_asangata() {
        let mut checker = TypeChecker::new();
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            type_name: "T".to_string(),
            span: span(),
        }];
        let body = vec![ASTNode::YogaNode {
            vama: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            dakshina: Box::new(ASTNode::Nama {
                base: "y".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        }];
        let dhatu = generic_dhatu_def(
            "samkala",
            vec!["T"],
            params,
            None,
            body,
        );
        let _ty = checker.check(&dhatu);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::PrakaaraAsangata(_))),
            "expected PrakaaraAsangata error for Yoga on Samanya param, got: {:?}",
            checker.errors
        );
    }

    // (e) Returning Samanya-typed param with matching Samanya return type

    #[test]
    fn test_generic_dhatu_return_samanya_param_is_valid() {
        let mut checker = TypeChecker::new();
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            type_name: "T".to_string(),
            span: span(),
        }];
        let body = vec![ASTNode::SamprapatiNode {
            expr: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            span: span(),
        }];
        let dhatu = generic_dhatu_def(
            "id",
            vec!["T"],
            params,
            Some(ASTNode::PhalamType {
                success_type: "T".to_string(),
                error_type: "Vaak".to_string(),
                span: span(),
            }),
            body,
        );
        let _ty = checker.check(&dhatu);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::SamprāptiAyogyatā)),
            "expected no return-type error, got: {:?}",
            checker.errors
        );
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    // (f) DevvaniType equality / compatibility for Samanya

    #[test]
    fn test_samanya_equality_same_name_is_compatible() {
        assert_eq!(DevvaniType::Samanya("T".to_string()), DevvaniType::Samanya("T".to_string()));
    }

    #[test]
    fn test_samanya_equality_different_name_is_not_compatible() {
        assert_ne!(DevvaniType::Samanya("T".to_string()), DevvaniType::Samanya("U".to_string()));
    }

    #[test]
    fn test_samanya_equality_with_concrete_type_is_not_compatible() {
        assert_ne!(DevvaniType::Samanya("T".to_string()), DevvaniType::Vaak);
        assert_ne!(DevvaniType::Samanya("T".to_string()), DevvaniType::Subject("Purnaank".to_string()));
    }

    // ===== Sāmānya (Generic) Part 2B — Type Inference Tests =====

    // (a) Generic Dravya Nirmāṇa — single generic param, single aṅga, concrete Vaak value provided
    #[test]
    fn test_generic_nirmana_single_param_infers_concrete_type() {
        let mut checker = TypeChecker::new();
        let def = generic_dravya_def(
            "Peti",
            vec!["T"],
            vec![anga_field("mulya", "T")],
        );
        let _ty = checker.check(&def);
        assert!(checker.errors.is_empty(), "definition should have no errors");

        let nirmana = ASTNode::NirmanaNode {
            dravya_name: "Peti".to_string(),
            values: vec![ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }],
            span: span(),
        };
        let result_ty = checker.check(&nirmana);
        assert!(
            checker.errors.is_empty(),
            "nirmana should infer T=Vaak, got: {:?}",
            checker.errors
        );
        match result_ty {
            DevvaniType::Dravya(name, angas) => {
                assert_eq!(name, "Peti");
                assert_eq!(angas.len(), 1);
                assert_eq!(angas[0], ("mulya".to_string(), DevvaniType::Subject("Vaak".to_string())));
            }
            _ => panic!("expected Dravya type, got {:?}", result_ty),
        }
    }

    // (b) Generic Dravya Nirmāṇa — same param T used at two aṅga positions, both Vaak → succeeds
    #[test]
    fn test_generic_nirmana_same_param_two_positions_matching_types_succeeds() {
        let mut checker = TypeChecker::new();
        let def = generic_dravya_def(
            "Yugala",
            vec!["T"],
            vec![anga_field("pahila", "T"), anga_field("dusara", "T")],
        );
        let _ty = checker.check(&def);
        assert!(checker.errors.is_empty());

        let nirmana = ASTNode::NirmanaNode {
            dravya_name: "Yugala".to_string(),
            values: vec![
                ASTNode::VaakLiteral {
                    value: "a".to_string(),
                    span: span(),
                },
                ASTNode::VaakLiteral {
                    value: "b".to_string(),
                    span: span(),
                },
            ],
            span: span(),
        };
        let result_ty = checker.check(&nirmana);
        assert!(
            checker.errors.is_empty(),
            "expected no inference errors, got: {:?}",
            checker.errors
        );
        match result_ty {
            DevvaniType::Dravya(name, angas) => {
                assert_eq!(name, "Yugala");
                assert_eq!(angas.len(), 2);
                assert_eq!(angas[0].1, DevvaniType::Subject("Vaak".to_string()));
                assert_eq!(angas[1].1, DevvaniType::Subject("Vaak".to_string()));
            }
            _ => panic!("expected Dravya type, got {:?}", result_ty),
        }
    }

    // (c) Generic Dravya Nirmāṇa — same param T at two positions with DIFFERENT values → D071
    #[test]
    fn test_generic_nirmana_same_param_two_positions_conflicting_types_produces_d071() {
        let mut checker = TypeChecker::new();
        let def = generic_dravya_def(
            "Yugala",
            vec!["T"],
            vec![anga_field("pahila", "T"), anga_field("dusara", "T")],
        );
        let _ty = checker.check(&def);
        assert!(checker.errors.is_empty());

        let nirmana = ASTNode::NirmanaNode {
            dravya_name: "Yugala".to_string(),
            values: vec![
                ASTNode::VaakLiteral {
                    value: "a".to_string(),
                    span: span(),
                },
                ASTNode::PurnaankLiteral {
                    value: 42,
                    span: span(),
                },
            ],
            span: span(),
        };
        let result_ty = checker.check(&nirmana);
        assert!(matches!(result_ty, DevvaniType::Unknown));
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::SamanyaAnishchitaDvandva { .. })),
            "expected SamanyaAnishchitaDvandva (D071), got: {:?}",
            checker.errors
        );
    }

    // (d) Generic Dhātu call — param T, return T, called with Vaak argument → result Vaak
    #[test]
    fn test_generic_dhatu_call_infers_return_type() {
        let mut checker = TypeChecker::new();
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            type_name: "T".to_string(),
            span: span(),
        }];
        let dhatu = generic_dhatu_def(
            "pratirupa",
            vec!["T"],
            params,
            Some(ASTNode::PhalamType {
                success_type: "T".to_string(),
                error_type: "Vaak".to_string(),
                span: span(),
            }),
            vec![],
        );
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty());

        let kriya_call = ASTNode::KriyaCall {
            karta: None,
            kriya: "pratirupa".to_string(),
            karma: vec![ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let result_ty = checker.check(&kriya_call);
        assert!(
            checker.errors.is_empty(),
            "expected no errors, got: {:?}",
            checker.errors
        );
        assert_eq!(
            result_ty,
            DevvaniType::Phalam(
                Box::new(DevvaniType::Subject("Vaak".to_string())),
                Box::new(DevvaniType::Subject("Vaak".to_string()))
            )
        );
    }

    // (e) Generic Dhātu call — D072: return type is Samanya("T") but no param uses T
    #[test]
    fn test_generic_dhatu_call_uninferable_return_type_produces_d072() {
        let mut checker = TypeChecker::new();
        // param "x" typed as concrete "vaak", not generic "T"
        let params = vec![KarakaParam {
            name: "x".to_string(),
            role: devvani_ast::KarakaRole::Karma,
            vibhakti: devvani_ast::Vibhakti::Dvitiya,
            is_borrowed: false,
            is_mutable_borrow: false,
            type_name: "vaak".to_string(),
            span: span(),
        }];
        let dhatu = generic_dhatu_def(
            "avaghataka",
            vec!["T"],
            params,
            Some(ASTNode::PhalamType {
                success_type: "T".to_string(),
                error_type: "Vaak".to_string(),
                span: span(),
            }),
            vec![],
        );
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty());

        let kriya_call = ASTNode::KriyaCall {
            karta: None,
            kriya: "avaghataka".to_string(),
            karma: vec![ASTNode::VaakLiteral {
                value: "x".to_string(),
                span: span(),
            }],
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span: span(),
        };
        let result_ty = checker.check(&kriya_call);
        assert!(matches!(result_ty, DevvaniType::Unknown));
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::SamanyaAniyata { .. })),
            "expected SamanyaAniyata (D072), got: {:?}",
            checker.errors
        );
    }

    // (f) Regression: non-generic Nirmāṇa and Dhātu calls unchanged

    #[test]
    fn test_non_generic_dravya_unchanged() {
        let mut checker = TypeChecker::new();
        let def = dravya_def(
            "manushya",
            vec![anga_field("naama", "vaak"), anga_field("sankhya", "sankhya")],
        );
        let ty = checker.check(&def);
        assert_eq!(ty, DevvaniType::Dravya(
            "manushya".to_string(),
            vec![
                ("naama".to_string(), DevvaniType::Vaak),
                ("sankhya".to_string(), DevvaniType::Subject("Purnaank".to_string())),
            ]
        ));
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_non_generic_dhatu_unchanged() {
        let mut checker = TypeChecker::new();
        let dhatu = dhatu_def("fetch", vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "data".to_string(),
                span: span(),
            }),
        }]);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }
}

