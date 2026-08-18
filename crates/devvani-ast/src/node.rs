use serde::{Deserialize, Serialize};

pub use devvani_lexer::token::Span;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpasargaDirective {
    Export,
    Private,
    Inline,
    Override,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpasargaNode {
    pub directives: Vec<UpasargaDirective>,
    pub target: Box<ASTNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Vibhakti {
    Prathama,
    Dvitiya,
    Tritiya,
    Chaturthi,
    Panchami,
    Shashthi,
    Saptami,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Lakara {
    Lat,
    Lit,
    Lut,
    Lrt,
    Let,
    Lot,
    Lan,
    Vidhilin,
    Asihlin,
    Lun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SamasaType {
    Tatpurusha,
    Dvandva,
    Bahuvrihi,
    Avyayibhava,
    Karmadhaaraya,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KarakaRole {
    Karta,
    Karma,
    Karana,
    Sampradana,
    Apadana,
    Apadan,
    Adhikarana,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Linga {
    Pullinga,
    Strilinga,
    Napumsakalinga,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Vacana {
    Eka,
    Dvi,
    Bahu,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gana {
    Bhvadi,
    Adadi,
    Juhotyadi,
    Divadi,
    Svadi,
    Tudadi,
    Rudhadi,
    Tanadi,
    Kryadi,
    Curadi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Upasarga {
    Pra,
    Para,
    Apa,
    Sam,
    Anu,
    Ava,
    Nis,
    Nir,
    Dus,
    Dur,
    Vi,
    A,
    Aa,
    Ni,
    Adhi,
    Api,
    Ati,
    Su,
    Ud,
    Abhi,
    Prati,
    Pari,
    Upa,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KarakaParam {
    pub name: String,
    pub role: KarakaRole,
    pub vibhakti: Vibhakti,
    pub is_borrowed: bool,
    pub is_mutable_borrow: bool,
    pub type_name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngaField {
    pub name: String,
    pub type_name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ASTNode {
    KaryakramNode {
        shareera: Vec<ASTNode>,
    },
    DhatuDef {
        name: String,
        generic_params: Vec<String>,
        lakara: Lakara,
        gana: Gana,
        linga: Linga,
        vacana: Vacana,
        params: Vec<KarakaParam>,
        upasargas: Vec<Upasarga>,
        return_karaka: Option<KarakaRole>,
        return_type: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
        span: Span,
    },
    DravyaDef {
        name: String,
        generic_params: Vec<String>,
        angas: Vec<AngaField>,
        span: Span,
    },
     NirmanaNode {
         dravya_name: String,
         values: Vec<ASTNode>,
         span: Span,
     },
     PhalamType {
         success_type: String,
         error_type: String,
         span: Span,
     },
     ArogyaNode {
         value: Box<ASTNode>,
         span: Span,
     },
     DoshaNode {
         value: Box<ASTNode>,
         span: Span,
     },
NidanaNode {
          target: Box<ASTNode>,
          arogya_bind: String,
          arogya_body: Vec<ASTNode>,
          dosha_bind: String,
          dosha_body: Vec<ASTNode>,
          span: Span,
      },
      /// SandarbhaNode — a borrow/reference expression.
      /// `sandarbha adhikara x` (immutable) or `sandarbha vikara adhikara x` (mutable).
      SandarbhaNode {
          target: Box<ASTNode>,
          is_mutable: bool,
          span: Span,
      },
      SamprapatiNode {
         expr: Box<ASTNode>,
         span: Span,
     },
     KriyaCall {
        karta: Option<Box<ASTNode>>,
        kriya: String,
        karma: Vec<ASTNode>,
        karana: Option<Box<ASTNode>>,
        sampradana: Option<Box<ASTNode>>,
        apadan: Option<Box<ASTNode>>,
        adhikarana: Option<Box<ASTNode>>,
        span: Span,
    },
    Nama {
        base: String,
        vibhakti: Vibhakti,
        linga: Linga,
        vacana: Vacana,
        span: Span,
    },
    AstiNode {
        naama: String,
        mulya: Box<ASTNode>,
    },
    BhavatiNode {
        naama: String,
        mulya: Box<ASTNode>,
    },
    YogaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    ViyogaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    GunaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    BhagaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    SamaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    AsamaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    NyuunaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    AdhikaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
    },
    VadatiNode {
        mulya: Box<ASTNode>,
    },
    PathatiNode {
        naama: String,
    },
    YadiNode {
        sthiti: Box<ASTNode>,
        tarhi: Vec<ASTNode>,
        anyatha: Option<Vec<ASTNode>>,
    },
    YavatNode {
        sthiti: Box<ASTNode>,
        shareera: Vec<ASTNode>,
    },
    PunahNode {
        varam: Box<ASTNode>,
        shareera: Vec<ASTNode>,
    },
    Dvandva {
        members: Vec<ASTNode>,
        span: Span,
    },
    PurnaankLiteral {
        value: i64,
        span: Span,
    },
    DashaamshaLiteral {
        value: f64,
        span: Span,
    },
    VaakLiteral {
        value: String,
        span: Span,
    },
    /// DhāraNode — typed or inferred variable declaration using the dhara keyword.
    /// Syntax: dhara <name> [<type>] = <expr> ।
    /// Multi-name tuple destructuring: dhara [a, b] = <expr> ।
    DharaNode {
        naamas: Vec<String>,
        type_name: Option<String>,
        mulya: Box<ASTNode>,
        is_mutable: bool,
        span: Span,
    },
    /// VaakNode — owned string variable declaration with Kāraka ownership role.
    /// Kartā = owner, Karaṇa = immutable borrow, Apādāna = move.
    VaakNode {
        naama: String,
        mulya: Box<ASTNode>,
        karaka: KarakaRole,
        is_mutable: bool,
        span: Span,
    },
    /// VaakYogaNode — string concatenation (Yoga applied to strings)
    /// Mīmāṃsā Apūrva-vidhi: two strings produce a new owned string
    VaakYogaNode {
        vama: Box<ASTNode>,
        dakshina: Box<ASTNode>,
        span: Span,
    },
    Samasa {
        samasa_type: SamasaType,
        parts: Vec<ASTNode>,
        components: Vec<String>,
        resolved: String,
        span: Span,
    },
    KritChain {
        steps: Vec<ASTNode>,
        span: Span,
    },
    UpasargaApplied {
        node: Box<UpasargaNode>,
    },
    TaddhitaChain {
        base: Box<ASTNode>,
        suffixes: Vec<String>,
        span: Span,
    },
    /// AvartanaNode — marks a KriyaCall that is a direct self-recursive call
    /// (a DhatuDef calling its own name from within its own body).
    /// Wraps the underlying KriyaCall for recursion-specific analysis in a later phase.
    AvartanaNode {
        call: Box<ASTNode>, // always an ASTNode::KriyaCall
        span: Span,
    },
PanktiNode {
         elements: Vec<ASTNode>,
         span: Span,
     },
     AvaliNode {
         elements: Vec<ASTNode>,
         span: Span,
     },
      VinyasaNode {
         target: Box<ASTNode>,
         index: Box<ASTNode>,
         span: Span,
     },
     SamavayaNode {
         target: Box<ASTNode>,
         anga_name: String,
         span: Span,
     },
      KramashahNode {
         item_name: String,
         iterable: Box<ASTNode>,
         body: Vec<ASTNode>,
         span: Span,
     },
     /// SamyogaNode — spawn a concurrent thread/task block.
     /// Can appear as a bare statement or as the initializer expression of a `dhara` binding.
     /// Syntax: samyoga { <statements> }
     SamyogaNode {
         body: Vec<ASTNode>,
         span: Span,
     },
     /// PraptiNode — join/wait for a spawned thread's result expression.
     /// Syntax: prapti <handle>
     PraptiNode {
         handle: Box<ASTNode>,
         span: Span,
     },
     /// DutaBanaaNode — channel creation expression, produces a (sender, receiver) pair.
     /// Syntax: duta banaa
     DutaBanaaNode {
         span: Span,
     },
     /// DutaBhejNode — send a message on a channel sender.
     /// Syntax: <sender> bhej sandesha <message> ।
     DutaBhejNode {
         sender: Box<ASTNode>,
         message: Box<ASTNode>,
         span: Span,
     },
     /// DutaGrahanNode — receive a message on a channel receiver (blocking expression).
     /// Syntax: grahaka grahan karo
     DutaGrahanNode {
         receiver: Box<ASTNode>,
         span: Span,
     },
      /// ManasNode — mutex-guarded block (scoped lock, auto-unlock at end of block).
      /// Syntax: manas <mutex_var> { <statements> }
      ManasNode {
          target: Box<ASTNode>,
          body: Vec<ASTNode>,
          span: Span,
      },
       /// ParinamaNode — pipeline/postfix-transform expression.
       /// Syntax: <expr> pariṇāma [ <ident> (, <ident>)* ]
       /// Value flows left-to-right through each dhatu in `dhatus`.
       ParinamaNode {
           mulyam: Box<ASTNode>,
           dhatus: Vec<String>,
           span: Span,
       },
       /// ParikshaaNode — test declaration block.
       /// Syntax: [tarka] parikshaa <name> { <statements> }
       ParikshaaNode {
           name: String,
           body: Vec<ASTNode>,
           is_tarka: bool,
           span: Span,
       },
       /// NigamanaNode — assert-true statement.
       /// Syntax: nigamana <expr> ।
       NigamanaNode {
           expr: Box<ASTNode>,
           span: Span,
       },
       /// SadrishyaNigamanaNode — assert-equal statement.
       /// Syntax: sadrishya-nigamana <expr1> <expr2> ।
       SadrishyaNigamanaNode {
           left: Box<ASTNode>,
           right: Box<ASTNode>,
           span: Span,
       },
        /// AsadrishyaNigamanaNode — assert-not-equal statement.
        /// Syntax: asadrishya-nigamana <expr1> <expr2> ।
        AsadrishyaNigamanaNode {
            left: Box<ASTNode>,
            right: Box<ASTNode>,
            span: Span,
        },

         // --- DOCUMENTATION (ĀRṢA-VYĀKHYĀ) ---
         // VrittiNode, BhashyaNode, TippaniNode appear as standalone preceding
         // statements in the containing block (KaryakramNode.shareera) immediately
         // before the item they document.  The parser is responsible for ensuring
         // proper adjacency.  This avoids adding required fields to DhatuDef /
         // DravyaDef which would break downstream crates (codegen, typesystem, llvm).
         VrittiNode {
             text: String,
             span: Span,
         },
         BhashyaNode {
             text: String,
             span: Span,
         },
         TippaniNode {
             text: String,
             param_name: String,
             span: Span,
         },

         // --- VERSIONING (VIKARA) ---
         // MrittikaNode declares package identity and version info.
         // Appears as a top-level item in KaryakramNode.shareera.
         MrittikaNode {
             package_name: String,
             naamadheya: NaamadheyaNode,
             vikaras: Vec<VikaraEntry>,
             span: Span,
         },
     }

     // --- VERSIONING (VIKARA) SUPPORT TYPES ---

     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
     pub struct NaamadheyaNode {
         pub version_string: String,
         pub span: Span,
     }

     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
     pub enum VikaraKind {
         Sukshma,   // patch-level, internal-only change
         Sthula,    // minor-level, backward-compatible addition
         SatyaBheda // breaking change
     }

     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
     pub struct VikaraEntry {
         pub kind: VikaraKind,
         pub description: String,
         pub span: Span,
     }

     fn dummy_span() -> Span {
        Span { line: 1, col: 1, len: 1 }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_vritti_node_construction() {
            let node = ASTNode::VrittiNode {
                text: "short doc".to_string(),
                span: Span { line: 1, col: 1, len: 9 },
            };
            match node {
                ASTNode::VrittiNode { text, span } => {
                    assert_eq!(text, "short doc");
                    assert_eq!(span.line, 1);
                }
                _ => panic!("expected VrittiNode"),
            }
        }

        #[test]
        fn test_bhashya_node_construction() {
            let node = ASTNode::BhashyaNode {
                text: "module docs".to_string(),
                span: Span { line: 1, col: 1, len: 12 },
            };
            match node {
                ASTNode::BhashyaNode { text, span } => {
                    assert_eq!(text, "module docs");
                    assert_eq!(span.col, 1);
                }
                _ => panic!("expected BhashyaNode"),
            }
        }

        #[test]
        fn test_tippani_node_construction() {
            let node = ASTNode::TippaniNode {
                text: "note on x".to_string(),
                param_name: "x".to_string(),
                span: Span { line: 2, col: 5, len: 9 },
            };
            match node {
                ASTNode::TippaniNode { text, param_name, span } => {
                    assert_eq!(text, "note on x");
                    assert_eq!(param_name, "x");
                    assert_eq!(span.line, 2);
                }
                _ => panic!("expected TippaniNode"),
            }
        }
    }
