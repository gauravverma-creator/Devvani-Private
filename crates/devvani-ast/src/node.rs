use serde::{Deserialize, Serialize};

pub use devvani_lexer::token::Span;

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
    Lat, Lit, Lut, Lrt, Let, Lot, Lan, Vidhilin, Asihlin, Lun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SamasaType {
    Tatpurusha, Dvandva, Bahuvrihi, Avyayibhava, Karmadhaaraya,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div,
    Eq, Neq, NotEq,
    Lt, Gt, LtEq, GtEq,
    And, Or,
    Mod,
    Not,
}

pub type UnaryOp = BinaryOp;

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
    Bhvadi, Adadi, Juhotyadi, Divadi, Svadi,
    Tudadi, Rudhadi, Tanadi, Kryadi, Curadi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Upasarga {
    Pra, Para, Apa, Sam, Anu, Ava, Nis, Nir,
    Dus, Dur, Vi, A, Aa, Ni, Adhi, Api, Ati, Su,
    Ud, Abhi, Prati, Pari, Upa,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KarakaParam {
    pub name: String,
    pub role: KarakaRole,
    pub vibhakti: Vibhakti,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ASTNode {
    Program {
        statements: Vec<ASTNode>,
        span: Span,
    },
    DhatuDef {
        name: String,
        lakara: Lakara,
        gana: Gana,
        linga: Linga,
        vacana: Vacana,
        params: Vec<KarakaParam>,
        upasargas: Vec<Upasarga>,
        return_karaka: Option<KarakaRole>,
        body: Vec<ASTNode>,
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
    Conditional {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
        else_branch: Option<Vec<ASTNode>>,
        span: Span,
    },
    Loop {
        condition: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
        span: Span,
    },
    BinaryExpr {
        op: BinaryOp,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        span: Span,
    },
    UnaryExpr {
        op: BinaryOp,
        operand: Box<ASTNode>,
        span: Span,
    },
    Dvandva {
        members: Vec<ASTNode>,
        span: Span,
    },
    IntLiteral {
        value: i64,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        span: Span,
    },
    StringLiteral {
        value: String,
        span: Span,
    },
    BoolLiteral {
        value: bool,
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
    Return {
        value: Option<Box<ASTNode>>,
        span: Span,
    },
    Comment {
        text: String,
        span: Span,
    },
}
