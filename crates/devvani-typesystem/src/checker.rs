use crate::{lakara::*, linga::*, symbol::*, type_env::TypeEnv, vacana::*, vibhakti::*};
use devvani_ast::node::{KarakaParam, NaamadheyaNode, VikaraEntry, VikaraKind};
use devvani_ast::ASTNode;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem;

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
    /// D073 — अनुमानविफलता (AnumanaViphalata): type could not be inferred from expression
    AnumanaViphalata,
    /// D074 — अनुमानसंगतिभङ्ग (AnumanaSamgatiBhanga): conflicting inferred types across return paths
    AnumanaSamgatiBhanga,
    /// D075 — प्राप्त्यप्रयुक्त (PraptiAprayukta): prapti applied to non-thread-handle type
    PraptiAprayukta {
        found: DevvaniType,
    },
    /// D076 — दूतभेजअप्रयुक्त (DutaBhejAprayukta): bhej applied to non-sender type
    DutaBhejAprayukta {
        found: DevvaniType,
    },
    /// D077 — दूतग्रहणअप्रयुक्त (DutaGrahanAprayukta): grahan karo applied to non-receiver type
    DutaGrahanAprayukta {
        found: DevvaniType,
    },
    /// D078 — मनसअप्रयुक्त (ManasAprayukta): manas applied to non-mutex-guarded type
    ManasAprayukta {
        found: DevvaniType,
    },
    /// D079 — धाराविन्यासासंगति (DharaVinyasaAsangati): multi-name dhara binding arity mismatch
    DharaVinyasaAsangati {
        found: DevvaniType,
    },
    /// D080 — परिणामासंगति (ParinamaAsangati): type or arity mismatch in a Pariṇāma chain stage
    ParinamaAsangati {
        stage: usize,
        expected: DevvaniType,
        found: DevvaniType,
    },
    /// D081 — परिणामशून्यता (ParinamaShunya): empty dhatu chain `pariṇāma []` used where concrete type required
    ParinamaShunya,
    /// D082 — परिणामदोषवैषम्य (ParinamaDoshaVaisamya): incompatible error types from multiple fallible dhatus
    ParinamaDoshaVaisamya {
        error_a: DevvaniType,
        error_b: DevvaniType,
    },
    /// D086 — निगमनाभिन्नप्रकार (NigamanaAbhinnaPrakaar): nigamana's expression does not type-check to Boolean
    NigamanaNotBool {
        found: DevvaniType,
    },
    /// D087 — सादृश्यनिगमनवैषम्य (SadrishyaNigamanaVaisamya): sadrishya-nigamana / asadrishya-nigamana operands have mismatched types
    SadrishyaNigamanaMismatchedTypes {
        left: DevvaniType,
        right: DevvaniType,
    },
    /// D088 — सादृश्यनिगमनासमartha (SadrishyaNigamanaAsamartha): operand type does not support equality comparison
    SadrishyaNigamanaNotEqualityComparable {
        ty: DevvaniType,
    },
    /// D089 — परीक्षाशरीरावैषम्य (ParikshaaShariraVaisamya): parikshaa body does not type-check to unit/void
    ParikshaaBodyNotUnit,
    /// D090 — अवैधनामधेयरूप (InvalidNaamadheyaFormat): naamadheya string is not a valid MAJOR.MINOR.PATCH shape
    InvalidNaamadheyaFormat(String),
    /// D095 — अवैधपैकेजनाम (InvalidPackageName): package name in mrittika block is not a valid identifier
    InvalidPackageName,
    /// D096 — सत्यभेदमहत्तरबुंद (SatyaBhedaRequiresMajorBump): satya-bheda declared while MAJOR >= 1 without a major-version bump
    SatyaBhedaRequiresMajorBump,
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
            TypeCheckError::AnumanaViphalata => {
                write!(f, "Anumana-viphalata: type could not be inferred from the given expression")
            }
            TypeCheckError::AnumanaSamgatiBhanga => {
                write!(f, "Anumana-samgati-bhanga: conflicting inferred types across return paths")
            }
            TypeCheckError::PraptiAprayukta { found } => {
                write!(
                    f,
                    "Prapti-aprayukta: prapti requires a Samyoga (thread handle) type; found {:?}",
                    found
                )
            }
            TypeCheckError::DutaBhejAprayukta { found } => {
                write!(
                    f,
                    "Duta-bhej-aprayukta: bhej requires a DutaBhejaka (channel sender) type; found {:?}",
                    found
                )
            }
            TypeCheckError::DutaGrahanAprayukta { found } => {
                write!(
                    f,
                    "Duta-grahan-aprayukta: grahan karo requires a DutaGrahaka (channel receiver) type; found {:?}",
                    found
                )
            }
            TypeCheckError::ManasAprayukta { found } => {
                write!(
                    f,
                    "Manas-aprayukta: manas requires a Manas (mutex-guarded) type; found {:?}",
                    found
                )
            }
             TypeCheckError::DharaVinyasaAsangati { found } => {
                 write!(
                     f,
                     "Dhara-vinyasa-asangata: multi-name binding requires a Duta (sender, receiver) pair type; found {:?}",
                     found
                 )
             }
             TypeCheckError::ParinamaAsangati { stage, expected, found } => {
                 write!(
                     f,
                     "Parinama-asangata: stage {} type mismatch; expected {:?}, found {:?}",
                     stage, expected, found
                 )
             }
             TypeCheckError::ParinamaShunya => {
                 write!(f, "Parinama-shunya: empty dhatu chain")
             }
             TypeCheckError::ParinamaDoshaVaisamya { error_a, error_b } => {
                  write!(
                      f,
                      "Parinama-dosha-vaisamya: incompatible error types {:?} vs {:?}",
                      error_a, error_b
                  )
              }
              TypeCheckError::NigamanaNotBool { found } => {
                  write!(f, "Nigamana-not-bool: expression must be Bool, found {:?}", found)
              }
              TypeCheckError::SadrishyaNigamanaMismatchedTypes { left, right } => {
                  write!(f, "Sadrishya-nigamana-mismatched-types: left {:?} != right {:?}", left, right)
              }
              TypeCheckError::SadrishyaNigamanaNotEqualityComparable { ty } => {
                  write!(f, "Sadrishya-nigamana-not-equality-comparable: type {:?} does not support equality comparison", ty)
              }
               TypeCheckError::ParikshaaBodyNotUnit => {
                   write!(f, "Parikshaa-body-not-unit: parikshaa body must return unit/void")
               }
               TypeCheckError::InvalidNaamadheyaFormat(msg) => {
                   write!(f, "Invalid-naamadheya-format: {}", msg)
               }
               TypeCheckError::InvalidPackageName => {
                   write!(f, "Invalid-package-name: package name must be a non-empty ASCII identifier starting with a letter, containing only letters, digits, and hyphens, with no consecutive or trailing hyphens")
               }
               TypeCheckError::SatyaBhedaRequiresMajorBump => {
                   write!(f, "Satya-bheda-requires-major-bump: a breaking change (satya-bheda) at MAJOR >= 1 requires the naamadheya to reflect a major-version increment")
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
        ASTNode::AstiNode { mulya, .. } | ASTNode::BhavatiNode { mulya, .. } | ASTNode::DharaNode { mulya, .. } => f(mulya),
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
           ASTNode::SamyogaNode { body, .. } => body.iter().for_each(|n| f(n)),
           ASTNode::PraptiNode { handle, .. } => f(handle),
           ASTNode::DutaBanaaNode { .. } => {}
           ASTNode::DutaBhejNode { sender, message, .. } => {
               f(sender);
               f(message);
           }
           ASTNode::DutaGrahanNode { receiver, .. } => f(receiver),
            ASTNode::ManasNode { target, body, .. } => {
                f(target);
                body.iter().for_each(|n| f(n));
            }
            ASTNode::ParikshaaNode { body, .. } => {
                body.iter().for_each(|n| f(n));
            }
            ASTNode::NigamanaNode { expr, .. } => f(expr),
            ASTNode::SadrishyaNigamanaNode { left, right, .. } => {
                f(left);
                f(right);
            }
            ASTNode::AsadrishyaNigamanaNode { left, right, .. } => {
                f(left);
                f(right);
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
    /// Temporary map for recording node types during DhatuDef body checking
    /// (used for return-type inference across branches).
    node_type_map: HashMap<*const ASTNode, DevvaniType>,
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
            node_type_map: HashMap::new(),
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

    /// Public accessor for the node type inference map
    pub fn node_type_map(&self) -> &HashMap<*const ASTNode, DevvaniType> {
        &self.node_type_map
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

    fn types_compatible(t1: &DevvaniType, t2: &DevvaniType) -> bool {
        if t1 == t2 {
            return true;
        }
        if matches!(t1, DevvaniType::Parameter(_)) || matches!(t2, DevvaniType::Parameter(_)) {
            return true;
        }
        if matches!(t1, DevvaniType::Samanya(_)) || matches!(t2, DevvaniType::Samanya(_)) {
            return true;
        }
        false
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
            DevvaniType::Samyoga(inner) => Self::collect_samanya_from_type(inner),
            DevvaniType::DutaBhejaka(inner) => Self::collect_samanya_from_type(inner),
            DevvaniType::DutaGrahaka(inner) => Self::collect_samanya_from_type(inner),
            DevvaniType::Duta(sender, receiver) => Self::collect_samanya_from_type(sender)
                .into_iter()
                .chain(Self::collect_samanya_from_type(receiver))
                .collect(),
            DevvaniType::Manas(inner) => Self::collect_samanya_from_type(inner),
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
            DevvaniType::Samyoga(inner) => DevvaniType::Samyoga(Box::new(Self::substitute_samanya_in_type(*inner, inference))),
            DevvaniType::DutaBhejaka(inner) => DevvaniType::DutaBhejaka(Box::new(Self::substitute_samanya_in_type(*inner, inference))),
            DevvaniType::DutaGrahaka(inner) => DevvaniType::DutaGrahaka(Box::new(Self::substitute_samanya_in_type(*inner, inference))),
            DevvaniType::Duta(sender, receiver) => DevvaniType::Duta(
                Box::new(Self::substitute_samanya_in_type(*sender, inference)),
                Box::new(Self::substitute_samanya_in_type(*receiver, inference)),
            ),
            DevvaniType::Manas(inner) => DevvaniType::Manas(Box::new(Self::substitute_samanya_in_type(*inner, inference))),
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
        let ty = match node {
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

             ASTNode::DharaNode { naamas, type_name, mulya, .. } => {
                  let mulya_ty_raw = if let Some(t) = type_name {
                      let expected = match self.resolve_type_name(t) {
                          Some(ty) => ty,
                          None => {
                              self.errors.push(TypeCheckError::PrakaaraAsangata(
                                  format!("unknown type '{}'", t),
                              ));
                              DevvaniType::Unknown
                          }
                      };
                      let mulya_ty = self.check(mulya);
                      if mulya_ty != DevvaniType::Unknown && mulya_ty != expected {
                          self.errors.push(TypeCheckError::PanktiAsangata {
                              expected: expected.clone(),
                              found: mulya_ty,
                          });
                      }
                      expected
                  } else {
                      let mulya_ty = self.check(mulya);
                      if matches!(mulya_ty, DevvaniType::Unknown) {
                          self.errors.push(TypeCheckError::AnumanaViphalata);
                      }
                      mulya_ty
                  };

                  let bind_types: Vec<DevvaniType> = if naamas.len() > 1 {
                      if let DevvaniType::Duta(sender_ty, receiver_ty) = &mulya_ty_raw {
                          vec![sender_ty.as_ref().clone(), receiver_ty.as_ref().clone()]
                      } else {
                          self.errors.push(TypeCheckError::DharaVinyasaAsangati {
                              found: mulya_ty_raw.clone(),
                          });
                          vec![mulya_ty_raw.clone(); naamas.len()]
                      }
                  } else if matches!(mulya.as_ref(), ASTNode::DutaBanaaNode { .. }) {
                      if let DevvaniType::Duta(sender_ty, _receiver_ty) = &mulya_ty_raw {
                          vec![sender_ty.as_ref().clone()]
                      } else {
                          vec![mulya_ty_raw.clone()]
                      }
                  } else {
                      vec![mulya_ty_raw.clone()]
                  };

                  for (naama, ty) in naamas.iter().zip(bind_types.iter()) {
                      if let ASTNode::Nama { base, .. } = mulya.as_ref() {
                          if Self::is_non_copy_type(ty) {
                              self.moved_vars.insert(base.clone());
                          }
                      }
                      let symbol = Symbol::new(naama, ty.clone(), &Vacana::Eka, &Linga::Pullinga, "var");
                      self.env.define_symbol(naama, symbol);
                      self.current_scope_vars.insert(naama.clone());
                  }
                  bind_types.into_iter().next().unwrap_or(mulya_ty_raw)
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

            ASTNode::ParinamaNode { mulyam, dhatus, .. } => {
                if dhatus.is_empty() {
                    self.errors.push(TypeCheckError::ParinamaShunya);
                    return DevvaniType::Unknown;
                }
                let mut current_type = self.check(mulyam);
                let mut fallible_error: Option<DevvaniType> = None;
                for (stage_idx, dhatu_name) in dhatus.iter().enumerate() {
                    if let Some(params) = self.function_params.get(dhatu_name) {
                        if params.len() != 1 {
                            self.errors.push(TypeCheckError::ParinamaAsangati {
                                stage: stage_idx,
                                expected: DevvaniType::Unknown,
                                found: current_type.clone(),
                            });
                            current_type = DevvaniType::Unknown;
                            continue;
                        }
                        let param = &params[0];
                        let expected_input = self.resolve_type_name(&param.type_name)
                            .unwrap_or_else(|| DevvaniType::Parameter(param.name.clone()));
                        if current_type != DevvaniType::Unknown
                            && expected_input != DevvaniType::Unknown
                            && !Self::types_compatible(&current_type, &expected_input)
                        {
                            self.errors.push(TypeCheckError::ParinamaAsangati {
                                stage: stage_idx,
                                expected: expected_input.clone(),
                                found: current_type.clone(),
                            });
                        }
                        if let Some(declared_return) = self.function_return_types.get(dhatu_name) {
                            match declared_return {
                                DevvaniType::Phalam(success, error) => {
                                    if let Some(prev_error) = fallible_error {
                                        if prev_error != *error.as_ref() {
                                            self.errors.push(
                                                TypeCheckError::ParinamaDoshaVaisamya {
                                                    error_a: prev_error,
                                                    error_b: error.as_ref().clone(),
                                                },
                                            );
                                            return DevvaniType::Unknown;
                                        }
                                    }
                                    fallible_error = Some(error.as_ref().clone());
                                    current_type = success.as_ref().clone();
                                }
                                _ => {
                                    current_type = declared_return.clone();
                                }
                            }
                        } else {
                            current_type = expected_input;
                        }
                    } else {
                        self.errors.push(TypeCheckError::ParinamaAsangati {
                            stage: stage_idx,
                            expected: DevvaniType::Unknown,
                            found: current_type.clone(),
                        });
                    }
                }
                if let Some(error_ty) = fallible_error {
                    DevvaniType::Phalam(Box::new(current_type), Box::new(error_ty))
                } else {
                    current_type
                }
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
                        self.resolve_type_name(&param.type_name)
                            .unwrap_or_else(|| DevvaniType::Parameter(param.name.clone()))
                    };
                    let param_symbol =
                        Symbol::new(&param.name, ty, &Vacana::Eka, &Linga::Pullinga, "i64");
                    self.env.define_symbol(&param.name, param_symbol);
                    self.current_scope_vars.insert(param.name.clone());
                }
                self.function_params.insert(name.clone(), params.clone());

                let mut saved_map = mem::take(&mut self.node_type_map);

                for stmt in body {
                    self.check(stmt);
                }

                let body_map = mem::take(&mut self.node_type_map);
                for (k, v) in body_map {
                    saved_map.insert(k, v);
                }
                self.node_type_map = saved_map;

                let type_map = &self.node_type_map;

                if let Some(_rt) = return_type {
                    if let Some(resolved_rt) = &self.current_return_type {
                        self.function_return_types.insert(name.clone(), resolved_rt.clone());
                    }
                } else {
                    let return_types =
                        Self::collect_return_types_from_body(body, &type_map);
                    if return_types.is_empty() {
                        // No return-producing expression; leave function out of the registry
                    } else if return_types.len() == 1 {
                        self.function_return_types
                            .insert(name.clone(), return_types[0].clone());
                    } else {
                        let first = &return_types[0];
                        if return_types[1..].iter().all(|t| t == first) {
                            self.function_return_types
                                .insert(name.clone(), first.clone());
                        } else {
                            self.errors.push(TypeCheckError::AnumanaSamgatiBhanga);
                        }
                    }
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

             ASTNode::SamyogaNode { body, .. } => {
                 self.push_ownership_state();
                 let mut last_type = DevvaniType::Unknown;
                 for stmt in body {
                     last_type = self.check(stmt);
                 }
                 self.pop_ownership_state();
                 DevvaniType::Samyoga(Box::new(last_type))
             }

             ASTNode::PraptiNode { handle, .. } => {
                 let handle_type = self.check(handle);
                 match handle_type {
                     DevvaniType::Samyoga(inner) => *inner,
                     _ => {
                         self.errors.push(TypeCheckError::PraptiAprayukta {
                             found: handle_type,
                         });
                         DevvaniType::Unknown
                     }
                 }
             }

             ASTNode::DutaBanaaNode { .. } => {
                 let msg_ty = DevvaniType::Unknown;
                 DevvaniType::Duta(
                     Box::new(DevvaniType::DutaBhejaka(Box::new(msg_ty.clone()))),
                     Box::new(DevvaniType::DutaGrahaka(Box::new(msg_ty))),
                 )
             }

             ASTNode::DutaBhejNode { sender, message, .. } => {
                 let sender_type = self.check(sender);
                 match &sender_type {
                     DevvaniType::DutaBhejaka(_) => {}
                     _ => {
                         self.errors.push(TypeCheckError::DutaBhejAprayukta {
                             found: sender_type.clone(),
                         });
                     }
                 }
                 let _msg_type = self.check(message);
                 DevvaniType::Unknown
             }

             ASTNode::DutaGrahanNode { receiver, .. } => {
                 let receiver_type = self.check(receiver);
                 match receiver_type {
                     DevvaniType::DutaGrahaka(msg_ty) => *msg_ty,
                     _ => {
                         self.errors.push(TypeCheckError::DutaGrahanAprayukta {
                             found: receiver_type,
                         });
                         DevvaniType::Unknown
                     }
                 }
             }

             ASTNode::ManasNode { target, body, .. } => {
                 let target_type = self.check(target);
                 match target_type {
                     DevvaniType::Manas(_inner_ty) => {
                         self.push_ownership_state();
                         for stmt in body {
                             self.check(stmt);
                         }
                         self.pop_ownership_state();
                         DevvaniType::Unknown
                     }
                     _ => {
                         self.errors.push(TypeCheckError::ManasAprayukta {
                             found: target_type,
                         });
                         DevvaniType::Unknown
                     }
                  }
              }

              ASTNode::ParikshaaNode { name: _, body, is_tarka: _, span: _ } => {
                  self.push_ownership_state();
                  let mut last_type = DevvaniType::Unknown;
                  for stmt in body {
                      last_type = self.check(stmt);
                  }
                  self.pop_ownership_state();
                  if !matches!(last_type, DevvaniType::Unknown) {
                      self.errors.push(TypeCheckError::ParikshaaBodyNotUnit);
                  }
                  DevvaniType::Unknown
              }

              ASTNode::NigamanaNode { expr, .. } => {
                  let expr_type = self.check(expr);
                  if !matches!(expr_type, DevvaniType::Subject(ref s) if s == "Bool") {
                      self.errors.push(TypeCheckError::NigamanaNotBool {
                          found: expr_type,
                      });
                  }
                  DevvaniType::Subject("Bool".to_string())
              }

              ASTNode::SadrishyaNigamanaNode { left, right, .. }
              | ASTNode::AsadrishyaNigamanaNode { left, right, .. } => {
                  let left_type = self.check(left);
                  let right_type = self.check(right);

                  if matches!(left_type, DevvaniType::Unknown) || matches!(right_type, DevvaniType::Unknown) {
                      self.errors.push(TypeCheckError::SadrishyaNigamanaNotEqualityComparable {
                          ty: if matches!(left_type, DevvaniType::Unknown) { left_type } else { right_type },
                      });
                  } else if !Self::types_compatible(&left_type, &right_type) {
                      self.errors.push(TypeCheckError::SadrishyaNigamanaMismatchedTypes {
                          left: left_type,
                          right: right_type,
                      });
                  }

                   DevvaniType::Subject("Bool".to_string())
               }

               ASTNode::MrittikaNode { package_name, naamadheya, vikaras, .. } => {
                   self.check_mrittika(package_name, naamadheya, vikaras);
                   DevvaniType::Unknown
               }

                _ => DevvaniType::Unknown,
          };
          self.node_type_map.insert(node as *const ASTNode, ty.clone());
          ty
      }

    /// Collect inferred return types from all terminal expressions in a function body.
    fn collect_return_types_from_body(
        body: &[ASTNode],
        type_map: &HashMap<*const ASTNode, DevvaniType>,
    ) -> Vec<DevvaniType> {
        if body.is_empty() {
            return vec![];
        }
        let last = body.last().unwrap();
        let mut return_types = Self::collect_return_types_from_node(last, type_map);
        return_types.retain(|t| !matches!(t, DevvaniType::Unknown));
        return_types
    }

    fn collect_return_types_from_node(
        node: &ASTNode,
        type_map: &HashMap<*const ASTNode, DevvaniType>,
    ) -> Vec<DevvaniType> {
        match node {
            ASTNode::YadiNode { tarhi, anyatha, .. } => {
                let mut types = Vec::new();
                if let Some(last_tarhi) = tarhi.last() {
                    types.extend(Self::collect_return_types_from_node(last_tarhi, type_map));
                }
                if let Some(anyatha_body) = anyatha {
                    if let Some(last_anyatha) = anyatha_body.last() {
                        types.extend(Self::collect_return_types_from_node(last_anyatha, type_map));
                    }
                }
                types
            }
            ASTNode::YavatNode { shareera, .. } => {
                if let Some(last) = shareera.last() {
                    Self::collect_return_types_from_node(last, type_map)
                } else {
                    vec![]
                }
            }
            ASTNode::PunahNode { shareera, .. } => {
                if let Some(last) = shareera.last() {
                    Self::collect_return_types_from_node(last, type_map)
                } else {
                    vec![]
                }
            }
            ASTNode::KaryakramNode { shareera, .. } => {
                if let Some(last) = shareera.last() {
                    Self::collect_return_types_from_node(last, type_map)
                } else {
                    vec![]
                }
            }
            _ => type_map
                .get(&(node as *const ASTNode))
                .cloned()
                .into_iter()
                .collect(),
        }
    }

    /// Semantic validation for a `mrittika` (package manifest) block.
    ///
    /// CHECK 1 — D090: naamadheya must be a valid MAJOR.MINOR.PATCH string
    ///   (optionally with a pre-release suffix after a hyphen).
    /// CHECK 2 — D095: package_name must be a valid package identifier.
    /// CHECK 3 — D096: if satya-bheda entries are present and naamadheya is valid,
    ///   MAJOR must be 0 (pre-1.0) — otherwise a breaking change at MAJOR >= 1
    ///   requires a major-version bump.
    fn check_mrittika(
        &mut self,
        package_name: &str,
        naamadheya: &NaamadheyaNode,
        vikaras: &[VikaraEntry],
    ) {
        let naamadheya_valid = self.validate_naamadheya(naamadheya);
        self.validate_package_name(package_name);

        if naamadheya_valid {
            let has_satya_bheda = vikaras
                .iter()
                .any(|v| matches!(v.kind, VikaraKind::SatyaBheda));
            if has_satya_bheda {
                self.validate_satya_bheda_major_bump(&naamadheya.version_string);
            }
        }
    }

    fn validate_naamadheya(&mut self, naamadheya: &NaamadheyaNode) -> bool {
        let raw = naamadheya.version_string.clone();
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                "naamadheya must be a non-empty MAJOR.MINOR.PATCH string (e.g. \"1.0.0\")".to_string(),
            ));
            return false;
        }

        if trimmed != raw {
            self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                "naamadheya must not contain leading or trailing whitespace".to_string(),
            ));
            return false;
        }

        let pre_release = trimmed.split_once('-');
        let core = pre_release.map(|(c, _)| c).unwrap_or(trimmed);
        let parts: Vec<&str> = core.split('.').collect();

        if parts.len() != 3 {
            self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                format!("naamadheya must have exactly three dot-separated numeric components (MAJOR.MINOR.PATCH), found {} components in \"{}\"", parts.len(), trimmed),
            ));
            return false;
        }

        for part in &parts {
            if part.is_empty() {
                self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                    format!("naamadheya component must not be empty in \"{}\"", trimmed),
                ));
                return false;
            }
            if !part.chars().all(|c| c.is_ascii_digit()) {
                self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                    format!("naamadheya component \"{}\" must contain only ASCII digits in \"{}\"", part, trimmed),
                ));
                return false;
            }
            if part.len() > 1 && part.starts_with('0') {
                self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                    format!("naamadheya component \"{}\" must not have leading zeros in \"{}\"", part, trimmed),
                ));
                return false;
            }
        }

        if let Some((_, pre)) = pre_release {
            if pre.is_empty() || pre.trim().is_empty() {
                self.errors.push(TypeCheckError::InvalidNaamadheyaFormat(
                    format!("naamadheya pre-release suffix after hyphen must not be empty in \"{}\"", trimmed),
                ));
                return false;
            }
        }

        true
    }

    fn validate_package_name(&mut self, package_name: &str) {
        let trimmed = package_name.trim();

        if trimmed.is_empty() {
            self.errors.push(TypeCheckError::InvalidPackageName);
            return;
        }

        if trimmed != package_name {
            self.errors.push(TypeCheckError::InvalidPackageName);
            return;
        }

        let chars: Vec<char> = trimmed.chars().collect();
        if !chars[0].is_ascii_alphabetic() {
            self.errors.push(TypeCheckError::InvalidPackageName);
            return;
        }

        for (_i, c) in chars.iter().enumerate() {
            if !c.is_ascii_alphanumeric() && *c != '-' {
                self.errors.push(TypeCheckError::InvalidPackageName);
                return;
            }
        }

        if trimmed.ends_with('-') {
            self.errors.push(TypeCheckError::InvalidPackageName);
            return;
        }

        if trimmed.contains("--") {
            self.errors.push(TypeCheckError::InvalidPackageName);
            return;
        }
    }

    fn validate_satya_bheda_major_bump(&mut self, version_string: &str) {
        let trimmed = version_string.trim();
        let core = trimmed.split_once('-').map(|(c, _)| c).unwrap_or(trimmed);
        let parts: Vec<&str> = core.split('.').collect();

        if parts.is_empty() {
            return;
        }

        if let Ok(major) = parts[0].parse::<u64>() {
            if major >= 1 {
                self.errors.push(TypeCheckError::SatyaBhedaRequiresMajorBump);
            }
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

    // ===== Anumāṇa (Type Inference) Tests =====

    // (a) DharaNode with inferred integer literal → Purnaank

    #[test]
    fn test_dhara_inferred_integer_literal() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::DharaNode {
            naamas: vec!["x".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            is_mutable: false,
            span: span(),
        }];
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        if let Some(symbol) = checker.env.lookup("x") {
            assert_eq!(
                symbol.devvani_type,
                DevvaniType::Subject("Purnaank".to_string())
            );
        }
    }

    // (b) DharaNode with inferred string literal → Vaak

    #[test]
    fn test_dhara_inferred_string_literal() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::DharaNode {
            naamas: vec!["s".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
            is_mutable: false,
            span: span(),
        }];
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        if let Some(symbol) = checker.env.lookup("s") {
            assert_eq!(symbol.devvani_type, DevvaniType::Vaak);
        }
    }

    // (c) Chained inference: dhara x = 5, then dhara y = x

    #[test]
    fn test_dhara_chained_inference() {
        let mut checker = TypeChecker::new();
        let body = vec![
            ASTNode::DharaNode {
                naamas: vec!["x".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
                is_mutable: false,
                span: span(),
            },
            ASTNode::DharaNode {
                naamas: vec!["y".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::Nama {
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
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        if let Some(symbol) = checker.env.lookup("y") {
            assert_eq!(
                symbol.devvani_type,
                DevvaniType::Subject("Purnaank".to_string())
            );
        }
    }

    // (d) DharaNode explicit type — matches value

    #[test]
    fn test_dhara_explicit_type_matches() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::DharaNode {
            naamas: vec!["x".to_string()],
            type_name: Some("sankhya".to_string()),
            mulya: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            is_mutable: false,
            span: span(),
        }];
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        if let Some(symbol) = checker.env.lookup("x") {
            assert_eq!(
                symbol.devvani_type,
                DevvaniType::Subject("Purnaank".to_string())
            );
        }
    }

    // (e) DharaNode explicit type — mismatches value

    #[test]
    fn test_dhara_explicit_type_mismatch() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::DharaNode {
            naamas: vec!["x".to_string()],
            type_name: Some("vaak".to_string()),
            mulya: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            is_mutable: false,
            span: span(),
        }];
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PanktiAsangata { .. })),
            "expected PanktiAsangata for explicit type mismatch, got: {:?}",
            checker.errors
        );
    }

    // (f) Dhatu return inference — single consistent path

    #[test]
    fn test_dhatu_return_inference_single_path() {
        let mut checker = TypeChecker::new();
        let body = vec![
            ASTNode::DharaNode {
                naamas: vec!["x".to_string()],
                type_name: None,
                mulya: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
                is_mutable: false,
                span: span(),
            },
            ASTNode::PurnaankLiteral { value: 10, span: span() },
        ];
        let dhatu = dhatu_def("get_num", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        let inferred = checker.function_return_types().get("get_num");
        assert!(inferred.is_some(), "expected inferred return type for get_num");
        assert_eq!(
            inferred.unwrap(),
            &DevvaniType::Subject("Purnaank".to_string())
        );
    }

    // (g) Dhatu return inference — conflicting paths triggers D074

    #[test]
    fn test_dhatu_return_inference_conflicting_paths_d074() {
        let mut checker = TypeChecker::new();
        let body = vec![yadi(
            vec![ASTNode::PurnaankLiteral { value: 1, span: span() }],
            Some(vec![ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }]),
        )];
        let dhatu = dhatu_def("infer_me", body);
        let _ty = checker.check(&dhatu);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AnumanaSamgatiBhanga)),
            "expected AnumanaSamgatiBhanga (D074), got: {:?}",
            checker.errors
        );
    }

    // (h) Dhatu no return expression with omitted return type — no entry in registry

    #[test]
    fn test_dhatu_no_return_expression_no_return_type() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::VadatiNode {
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "data".to_string(),
                span: span(),
            }),
        }];
        let dhatu = dhatu_def("fetch", body);
        let _ty = checker.check(&dhatu);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        assert!(
            !checker.function_return_types().contains_key("fetch"),
            "fetch should not have an inferred return type when body has no return expression"
        );
    }

    // (i) D073 trigger: DharaNode whose mulya evaluates to an unknown type.
    //     VadatiNode returns Unknown (it is an output statement, not a value), so
    //     using it as an expression inside DharaNode exercises the defensive check.

    #[test]
    fn test_dhara_inference_unknown_triggers_d073() {
        let mut checker = TypeChecker::new();
        let body = vec![ASTNode::DharaNode {
            naamas: vec!["x".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "hi".to_string(),
                    span: span(),
                }),
            }),
            is_mutable: false,
            span: span(),
        }];
        let dhatu = dhatu_def("main", body);
        let _ty = checker.check(&dhatu);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::AnumanaViphalata)),
            "expected AnumanaViphalata (D073), got: {:?}",
            checker.errors
        );
    }

    // ===== Concurrency (Samyoga / Prapti / Duta / Manas) Tests =====

    #[test]
    fn test_samyoga_block_produces_thread_handle_type() {
        let mut checker = TypeChecker::new();
        let samyoga = ASTNode::SamyogaNode {
            body: vec![ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }],
            span: span(),
        };
        let ty = checker.check(&samyoga);
        assert!(matches!(ty, DevvaniType::Samyoga(_)));
        if let DevvaniType::Samyoga(inner) = &ty {
            assert_eq!(**inner, DevvaniType::Subject("Purnaank".to_string()));
        }
    }

    #[test]
    fn test_prapti_on_valid_handle_unwraps_inner_type() {
        let mut checker = TypeChecker::new();
        let handle = ASTNode::SamyogaNode {
            body: vec![ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }],
            span: span(),
        };
        let prapti = ASTNode::PraptiNode {
            handle: Box::new(handle),
            span: span(),
        };
        let ty = checker.check(&prapti);
        assert_eq!(ty, DevvaniType::Subject("Purnaank".to_string()));
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_prapti_on_non_handle_produces_d075() {
        let mut checker = TypeChecker::new();
        let prapti = ASTNode::PraptiNode {
            handle: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&prapti);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::PraptiAprayukta { .. })),
            "expected PraptiAprayukta D075, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_duta_banaa_binding_produces_sender_receiver_types() {
        let mut checker = TypeChecker::new();
        let binding = ASTNode::DharaNode {
            naamas: vec!["bhejaka".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::DutaBanaaNode { span: span() }),
            is_mutable: false,
            span: span(),
        };
        let _ty = checker.check(&binding);
        if let Some(sym) = checker.env.lookup("bhejaka") {
            assert!(
                matches!(sym.devvani_type, DevvaniType::DutaBhejaka(_)),
                "expected DutaBhejaka, got {:?}",
                sym.devvani_type
            );
        }
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_duta_bhej_on_valid_sender_succeeds() {
        let mut checker = TypeChecker::new();
        let channel = ASTNode::DutaBanaaNode { span: span() };
        let _pair_ty = checker.check(&channel);

        let sender_binding = ASTNode::DharaNode {
            naamas: vec!["bhejaka".to_string()],
            type_name: None,
            mulya: Box::new(channel),
            is_mutable: false,
            span: span(),
        };
        checker.check(&sender_binding);

        let bhej = ASTNode::DutaBhejNode {
            sender: Box::new(ASTNode::Nama {
                base: "bhejaka".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: span(),
            }),
            message: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let ty = checker.check(&bhej);
        assert_eq!(ty, DevvaniType::Unknown);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_duta_bhej_on_non_sender_produces_d076() {
        let mut checker = TypeChecker::new();
        let bhej = ASTNode::DutaBhejNode {
            sender: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            message: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&bhej);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DutaBhejAprayukta { .. })),
            "expected DutaBhejAprayukta D076, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_duta_grahan_on_valid_receiver_succeeds() {
        let mut checker = TypeChecker::new();
        let channel = ASTNode::DutaBanaaNode { span: span() };
        let _pair_ty = checker.check(&channel);

        let sender_binding = ASTNode::DharaNode {
            naamas: vec!["bhejaka".to_string()],
            type_name: None,
            mulya: Box::new(channel),
            is_mutable: false,
            span: span(),
        };
        checker.check(&sender_binding);

        checker.env.define(
            "grahaka",
            DevvaniType::DutaGrahaka(Box::new(DevvaniType::Unknown)),
        );

        let grahan = ASTNode::DutaGrahanNode {
            receiver: Box::new(ASTNode::Nama {
                base: "grahaka".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: span(),
            }),
            span: span(),
        };
        let ty = checker.check(&grahan);
        assert_eq!(ty, DevvaniType::Unknown);
        assert!(
            !checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DutaGrahanAprayukta { .. })),
            "expected no DutaGrahanAprayukta error for valid receiver, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_duta_grahan_on_non_receiver_produces_d077() {
        let mut checker = TypeChecker::new();
        let grahan = ASTNode::DutaGrahanNode {
            receiver: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&grahan);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DutaGrahanAprayukta { .. })),
            "expected DutaGrahanAprayukta D077, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_manas_on_valid_mutex_target_scopes_body() {
        let mut checker = TypeChecker::new();
        checker.env.define(
            "lock",
            DevvaniType::Manas(Box::new(DevvaniType::Vaak)),
        );

        let manas = ASTNode::ManasNode {
            target: Box::new(ASTNode::Nama {
                base: "lock".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: devvani_ast::Vacana::Eka,
                span: span(),
            }),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "inside manas".to_string(),
                    span: span(),
                }),
            }],
            span: span(),
        };
        let ty = checker.check(&manas);
        assert_eq!(ty, DevvaniType::Unknown);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_manas_on_non_mutex_target_produces_d078() {
        let mut checker = TypeChecker::new();
        let manas = ASTNode::ManasNode {
            target: Box::new(ASTNode::PurnaankLiteral {
                value: 42,
                span: span(),
            }),
            body: vec![],
            span: span(),
        };
        let _ty = checker.check(&manas);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ManasAprayukta { .. })),
            "expected ManasAprayukta D078, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_duta_banaa_tuple_destructuring_produces_correct_types() {
        let mut checker = TypeChecker::new();
        let binding = ASTNode::DharaNode {
            naamas: vec!["bhejaka".to_string(), "grahaka".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::DutaBanaaNode { span: span() }),
            is_mutable: false,
            span: span(),
        };
        let _ty = checker.check(&binding);
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
        if let Some(sym) = checker.env.lookup("bhejaka") {
            assert!(
                matches!(sym.devvani_type, DevvaniType::DutaBhejaka(_)),
                "expected bhejaka to be DutaBhejaka, got {:?}",
                sym.devvani_type
            );
        } else {
            panic!("bhejaka not found in symbol table");
        }
        if let Some(sym) = checker.env.lookup("grahaka") {
            assert!(
                matches!(sym.devvani_type, DevvaniType::DutaGrahaka(_)),
                "expected grahaka to be DutaGrahaka, got {:?}",
                sym.devvani_type
            );
        } else {
            panic!("grahaka not found in symbol table");
        }
    }

    fn parinama_dhatu(
        name: &str,
        params: Vec<(&str, &str)>,
        return_type: Option<ASTNode>,
    ) -> ASTNode {
        ASTNode::DhatuDef {
            name: name.to_string(),
            generic_params: vec![],
            lakara: Lakara::Lat,
            gana: Gana::Bhvadi,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params: params
                .into_iter()
                .map(|(n, t)| KarakaParam {
                    name: n.to_string(),
                    role: devvani_ast::KarakaRole::Karma,
                    vibhakti: devvani_ast::Vibhakti::Dvitiya,
                    is_borrowed: false,
                    is_mutable_borrow: false,
                    type_name: t.to_string(),
                    span: span(),
                })
                .collect(),
            upasargas: vec![],
            return_karaka: None,
            return_type: return_type.map(Box::new),
            body: vec![],
            span: span(),
        }
    }

    // Pariṇāma (Pipeline) type system tests

    #[test]
    fn test_parinama_three_dhatu_happy_path() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "d1",
            vec![("x", "purnaank")],
            Some(ASTNode::PurnaankLiteral { value: 0, span: span() }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "d2",
            vec![("x", "purnaank")],
            Some(ASTNode::DashaamshaLiteral { value: 0.0, span: span() }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "d3",
            vec![("x", "dashaamsha")],
            Some(ASTNode::VaakLiteral {
                value: "".to_string(),
                span: span(),
            }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["d1".to_string(), "d2".to_string(), "d3".to_string()],
            span: span(),
        };
        let ty = checker.check(&parinama);
        assert_eq!(ty, DevvaniType::Subject("Vaak".to_string()));
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_parinama_single_dhatu_happy_path() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "to_int",
            vec![("x", "purnaank")],
            Some(ASTNode::PurnaankLiteral { value: 0, span: span() }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["to_int".to_string()],
            span: span(),
        };
        let ty = checker.check(&parinama);
        assert_eq!(ty, DevvaniType::Subject("Purnaank".to_string()));
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_parinama_type_mismatch_between_stages_d080() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "d1",
            vec![("x", "purnaank")],
            Some(ASTNode::PurnaankLiteral { value: 0, span: span() }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "d2",
            vec![("x", "dashaamsha")],
            Some(ASTNode::DashaamshaLiteral { value: 0.0, span: span() }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["d1".to_string(), "d2".to_string()],
            span: span(),
        };
        let _ty = checker.check(&parinama);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParinamaAsangati { stage: 1, .. })),
            "expected ParinamaAsangati at stage 1, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_parinama_mulyam_type_mismatch_d080() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "to_int",
            vec![("x", "dashaamsha")],
            Some(ASTNode::DashaamshaLiteral { value: 0.0, span: span() }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["to_int".to_string()],
            span: span(),
        };
        let _ty = checker.check(&parinama);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParinamaAsangati { stage: 0, .. })),
            "expected ParinamaAsangati at stage 0, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_parinama_arity_error_d080() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "zero_arg",
            vec![],
            Some(ASTNode::PurnaankLiteral { value: 0, span: span() }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["zero_arg".to_string()],
            span: span(),
        };
        let _ty = checker.check(&parinama);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParinamaAsangati { .. })),
            "expected ParinamaAsangati arity error, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_parinama_empty_chain_d081() {
        let mut checker = TypeChecker::new();
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec![],
            span: span(),
        };
        let ty = checker.check(&parinama);
        assert_eq!(ty, DevvaniType::Unknown);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParinamaShunya)),
            "expected ParinamaShunya, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_parinama_fallible_propagation_mid_chain() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "fallible",
            vec![("x", "purnaank")],
            Some(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "vaak".to_string(),
                span: span(),
            }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "to_float",
            vec![("x", "purnaank")],
            Some(ASTNode::DashaamshaLiteral { value: 0.0, span: span() }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "to_vaak",
            vec![("x", "dashaamsha")],
            Some(ASTNode::VaakLiteral {
                value: "".to_string(),
                span: span(),
            }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["fallible".to_string(), "to_float".to_string(), "to_vaak".to_string()],
            span: span(),
        };
        let ty = checker.check(&parinama);
        assert_eq!(
            ty,
            DevvaniType::Phalam(
                Box::new(DevvaniType::Subject("Vaak".to_string())),
                Box::new(DevvaniType::Vaak)
            )
        );
        assert!(checker.errors.is_empty(), "expected no errors, got: {:?}", checker.errors);
    }

    #[test]
    fn test_parinama_fallible_conflict_d082() {
        let mut checker = TypeChecker::new();
        let _ = checker.check(&parinama_dhatu(
            "fallible_a",
            vec![("x", "purnaank")],
            Some(ASTNode::PhalamType {
                success_type: "sankhya".to_string(),
                error_type: "vaak".to_string(),
                span: span(),
            }),
        ));
        let _ = checker.check(&parinama_dhatu(
            "fallible_b",
            vec![("x", "purnaank")],
            Some(ASTNode::PhalamType {
                success_type: "dashaamsha".to_string(),
                error_type: "dashaamsha".to_string(),
                span: span(),
            }),
        ));
        let parinama = ASTNode::ParinamaNode {
            mulyam: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            dhatus: vec!["fallible_a".to_string(), "fallible_b".to_string()],
            span: span(),
        };
        let _ty = checker.check(&parinama);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParinamaDoshaVaisamya { .. })),
            "expected ParinamaDoshaVaisamya, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_dhara_multi_name_arity_mismatch_produces_d079() {
        let mut checker = TypeChecker::new();
        let binding = ASTNode::DharaNode {
            naamas: vec!["a".to_string(), "b".to_string()],
            type_name: None,
            mulya: Box::new(ASTNode::PurnaankLiteral { value: 42, span: span() }),
            is_mutable: false,
            span: span(),
        };
        let _ty = checker.check(&binding);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::DharaVinyasaAsangati { .. })),
            "expected DharaVinyasaAsangati D079 for non-Duta multi-name binding, got: {:?}",
            checker.errors
        );
    }

    // ===== Parīkṣā (Testing) TypeSystem Tests =====

    #[test]
    fn test_valid_parikshaa_block_unit_body_type_checks() {
        let mut checker = TypeChecker::new();
        let parikshaa = ASTNode::ParikshaaNode {
            name: "foo".to_string(),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "hello".to_string(),
                    span: span(),
                }),
            }],
            is_tarka: false,
            span: span(),
        };
        let errors = checker.check_program(&parikshaa);
        assert!(
            !errors.iter().any(|e| matches!(e, TypeCheckError::ParikshaaBodyNotUnit)),
            "expected no ParikshaaBodyNotUnit for unit body, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_valid_tarka_parikshaa_block_type_checks_and_preserves_flag() {
        let mut checker = TypeChecker::new();
        let parikshaa = ASTNode::ParikshaaNode {
            name: "tarka_test".to_string(),
            body: vec![ASTNode::VadatiNode {
                mulya: Box::new(ASTNode::VaakLiteral {
                    value: "tarka".to_string(),
                    span: span(),
                }),
            }],
            is_tarka: true,
            span: span(),
        };
        let ty = checker.check(&parikshaa);
        assert_eq!(ty, DevvaniType::Unknown);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::ParikshaaBodyNotUnit)),
            "expected no ParikshaaBodyNotUnit for tarka parikshaa, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_nigamana_bool_expr_passes() {
        let mut checker = TypeChecker::new();
        let nigamana = ASTNode::NigamanaNode {
            expr: Box::new(ASTNode::SamaNode {
                vama: Box::new(ASTNode::PurnaankLiteral { value: 1, span: span() }),
                dakshina: Box::new(ASTNode::PurnaankLiteral { value: 1, span: span() }),
            }),
            span: span(),
        };
        let ty = checker.check(&nigamana);
        assert_eq!(ty, DevvaniType::Subject("Bool".to_string()));
        assert!(
            checker.errors.is_empty(),
            "expected no errors for nigamana with Bool expr, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_nigamana_non_bool_expr_triggers_d086() {
        let mut checker = TypeChecker::new();
        let nigamana = ASTNode::NigamanaNode {
            expr: Box::new(ASTNode::PurnaankLiteral { value: 42, span: span() }),
            span: span(),
        };
        let _ty = checker.check(&nigamana);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::NigamanaNotBool { .. })),
            "expected NigamanaNotBool D086, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_sadrishya_nigamana_matching_types_passes() {
        let mut checker = TypeChecker::new();
        let assertion = ASTNode::SadrishyaNigamanaNode {
            left: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            right: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            span: span(),
        };
        let ty = checker.check(&assertion);
        assert_eq!(ty, DevvaniType::Subject("Bool".to_string()));
        assert!(
            checker.errors.is_empty(),
            "expected no errors for matching types, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_sadrishya_nigamana_mismatched_types_triggers_d087() {
        let mut checker = TypeChecker::new();
        let assertion = ASTNode::SadrishyaNigamanaNode {
            left: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            right: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&assertion);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SadrishyaNigamanaMismatchedTypes { .. })),
            "expected SadrishyaNigamanaMismatchedTypes D087, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_sadrishya_nigamana_unknown_type_triggers_d088() {
        let mut checker = TypeChecker::new();
        // First declare and move a non-Copy variable so that subsequent use yields Unknown
        checker.check(&ASTNode::AstiNode {
            naama: "x".to_string(),
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
        });
        checker.check(&ASTNode::AstiNode {
            naama: "y".to_string(),
            mulya: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        });
        let assertion = ASTNode::SadrishyaNigamanaNode {
            left: Box::new(ASTNode::Nama {
                base: "x".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            right: Box::new(ASTNode::PurnaankLiteral { value: 5, span: span() }),
            span: span(),
        };
        let _ty = checker.check(&assertion);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SadrishyaNigamanaNotEqualityComparable { .. })),
            "expected SadrishyaNigamanaNotEqualityComparable D088, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_asadrishya_nigamana_matching_types_passes() {
        let mut checker = TypeChecker::new();
        let assertion = ASTNode::AsadrishyaNigamanaNode {
            left: Box::new(ASTNode::VaakLiteral {
                value: "a".to_string(),
                span: span(),
            }),
            right: Box::new(ASTNode::VaakLiteral {
                value: "b".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let ty = checker.check(&assertion);
        assert_eq!(ty, DevvaniType::Subject("Bool".to_string()));
        assert!(
            checker.errors.is_empty(),
            "expected no errors for matching types, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_asadrishya_nigamana_mismatched_types_triggers_d087() {
        let mut checker = TypeChecker::new();
        let assertion = ASTNode::AsadrishyaNigamanaNode {
            left: Box::new(ASTNode::PurnaankLiteral { value: 1, span: span() }),
            right: Box::new(ASTNode::VaakLiteral {
                value: "x".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&assertion);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SadrishyaNigamanaMismatchedTypes { .. })),
            "expected SadrishyaNigamanaMismatchedTypes D087 for asadrishya, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_asadrishya_nigamana_unknown_type_triggers_d088() {
        let mut checker = TypeChecker::new();
        checker.check(&ASTNode::AstiNode {
            naama: "a".to_string(),
            mulya: Box::new(ASTNode::VaakLiteral {
                value: "hello".to_string(),
                span: span(),
            }),
        });
        checker.check(&ASTNode::AstiNode {
            naama: "b".to_string(),
            mulya: Box::new(ASTNode::Nama {
                base: "a".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
        });
        let assertion = ASTNode::AsadrishyaNigamanaNode {
            left: Box::new(ASTNode::Nama {
                base: "a".to_string(),
                vibhakti: devvani_ast::Vibhakti::Prathama,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                span: span(),
            }),
            right: Box::new(ASTNode::VaakLiteral {
                value: "y".to_string(),
                span: span(),
            }),
            span: span(),
        };
        let _ty = checker.check(&assertion);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::SadrishyaNigamanaNotEqualityComparable { .. })),
            "expected SadrishyaNigamanaNotEqualityComparable D088 for asadrishya, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_parikshaa_body_producing_value_triggers_d089() {
        let mut checker = TypeChecker::new();
        let parikshaa = ASTNode::ParikshaaNode {
            name: "bad_test".to_string(),
            body: vec![ASTNode::PurnaankLiteral { value: 42, span: span() }],
            is_tarka: false,
            span: span(),
        };
        let _ty = checker.check(&parikshaa);
        assert!(
            checker
                .errors
                .iter()
                .any(|e| matches!(e, TypeCheckError::ParikshaaBodyNotUnit)),
            "expected ParikshaaBodyNotUnit D089, got: {:?}",
            checker.errors
        );
    }

    // ===== Versioning (Mrittika / Vikara) Semantic Validation Tests =====

    fn mrittika_node(
        package_name: &str,
        version_string: &str,
        vikaras: Vec<VikaraEntry>,
    ) -> ASTNode {
        ASTNode::MrittikaNode {
            package_name: package_name.to_string(),
            naamadheya: NaamadheyaNode {
                version_string: version_string.to_string(),
                span: span(),
            },
            vikaras,
            span: span(),
        }
    }

    fn vikara_entry(kind: VikaraKind, description: &str) -> VikaraEntry {
        VikaraEntry {
            kind,
            description: description.to_string(),
            span: span(),
        }
    }

    // --- D090: InvalidNaamadheyaFormat ---

    #[test]
    fn test_naamadheya_valid_1_0_0() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected no D090 for valid \"1.0.0\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_valid_0_1_0() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "0.1.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.is_empty(),
            "expected no errors for valid \"0.1.0\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_valid_0_0_1() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "0.0.1", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_naamadheya_valid_10_20_30() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "10.20.30", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_naamadheya_valid_with_prerelease_alpha() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0.0-alpha", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_naamadheya_valid_with_prerelease_beta_dot_1() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "2.0.0-beta.1", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_naamadheya_invalid_too_few_components_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for \"1.0\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_too_many_components_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for \"1.0.0.0\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_leading_zero_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.00.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for leading zero, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_non_numeric_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "a.b.c", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for \"a.b.c\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_empty_prerelease_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0.0-", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for empty pre-release, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_empty_string_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for empty string, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_leading_whitespace_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", " 1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for leading whitespace, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_naamadheya_invalid_trailing_whitespace_d090() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-pkg", "1.0.0 ", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))),
            "expected D090 for trailing whitespace, got: {:?}",
            checker.errors
        );
    }

    // --- D095: InvalidPackageName ---

    #[test]
    fn test_package_name_valid_devvani_core() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("devvani-core", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected no D095 for \"devvani-core\", got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_valid_my_package() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my-package", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_package_name_valid_single_letter() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("a", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_package_name_valid_with_digits() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("package123", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_package_name_invalid_empty_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for empty name, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_invalid_whitespace_only_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("   ", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for whitespace-only name, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_invalid_starts_with_hyphen_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("-package", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for name starting with hyphen, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_invalid_trailing_hyphen_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("package-", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for trailing hyphen, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_invalid_consecutive_hyphens_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("my--package", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for consecutive hyphens, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_package_name_invalid_starts_with_digit_d095() {
        let mut checker = TypeChecker::new();
        let mrittika = mrittika_node("123package", "1.0.0", vec![]);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::InvalidPackageName)),
            "expected D095 for name starting with digit, got: {:?}",
            checker.errors
        );
    }

    // --- D096: SatyaBhedaRequiresMajorBump ---

    #[test]
    fn test_satya_bheda_major_zero_passes_silently() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![
            vikara_entry(VikaraKind::Sukshma, "internal fix"),
            vikara_entry(VikaraKind::SatyaBheda, "breaking API change"),
        ];
        let mrittika = mrittika_node("my-pkg", "0.5.0", vikaras);
        checker.check(&mrittika);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::SatyaBhedaRequiresMajorBump)),
            "D096 should not fire for MAJOR=0, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_satya_bheda_major_one_fires_d096() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![
            vikara_entry(VikaraKind::Sthula, "new feature"),
            vikara_entry(VikaraKind::SatyaBheda, "removed old API"),
        ];
        let mrittika = mrittika_node("my-pkg", "1.0.0", vikaras);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::SatyaBhedaRequiresMajorBump)),
            "expected D096 for MAJOR=1 with satya-bheda, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_satya_bheda_major_two_fires_d096() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![vikara_entry(VikaraKind::SatyaBheda, "breaking change")];
        let mrittika = mrittika_node("my-pkg", "2.3.1", vikaras);
        checker.check(&mrittika);
        assert!(
            checker.errors.iter().any(|e| matches!(e, TypeCheckError::SatyaBhedaRequiresMajorBump)),
            "expected D096 for MAJOR=2 with satya-bheda, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_satya_bheda_malformed_naamadheya_only_d090_no_d096() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![vikara_entry(VikaraKind::SatyaBheda, "breaking change")];
        let mrittika = mrittika_node("my-pkg", "not-a-version", vikaras);
        checker.check(&mrittika);
        let d090_count = checker.errors.iter().filter(|e| matches!(e, TypeCheckError::InvalidNaamadheyaFormat(_))).count();
        let d096_count = checker.errors.iter().filter(|e| matches!(e, TypeCheckError::SatyaBhedaRequiresMajorBump)).count();
        assert!(d090_count >= 1, "expected at least one D090, got: {:?}", checker.errors);
        assert_eq!(d096_count, 0, "D096 must not fire when naamadheya is malformed, got: {:?}", checker.errors);
    }

    #[test]
    fn test_no_satya_bheda_never_fires_d096() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![
            vikara_entry(VikaraKind::Sukshma, "patch"),
            vikara_entry(VikaraKind::Sthula, "minor"),
        ];
        let mrittika = mrittika_node("my-pkg", "1.0.0", vikaras);
        checker.check(&mrittika);
        assert!(
            !checker.errors.iter().any(|e| matches!(e, TypeCheckError::SatyaBhedaRequiresMajorBump)),
            "D096 must not fire without satya-bheda, got: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_valid_mrittika_block_zero_diagnostics() {
        let mut checker = TypeChecker::new();
        let vikaras = vec![
            vikara_entry(VikaraKind::Sukshma, "typo fix"),
            vikara_entry(VikaraKind::Sthula, "added new endpoint"),
            vikara_entry(VikaraKind::SatyaBheda, "removed deprecated endpoint"),
        ];
        let mrittika = mrittika_node("devvani-core", "0.5.0", vikaras);
        checker.check(&mrittika);
        assert!(
            checker.errors.is_empty(),
            "expected zero diagnostics for valid mrittika block, got: {:?}",
            checker.errors
        );
    }
}

