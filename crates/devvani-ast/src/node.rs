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
    DharaNode {
        naama: String,
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
}
