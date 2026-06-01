
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // --- IDENTIFIERS & LITERALS ---
    Identifier(String),        // IAST unicode identifiers
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    
    // --- VIBHAKTI (Case markers as type annotations) ---
    Prathama,    // -h / -ḥ suffix  → Subject/Type
    Dvitiya,     // -m / -ṃ suffix  → Object/Param
    Tritiya,     // -ena suffix      → Instrument/Helper
    Chaturthi,   // -āya suffix      → Dative/Return target
    Panchami,    // -āt suffix       → Ablative/Source
    Shashthi,    // -sya suffix      → Genitive/Parent
    Saptami,     // -e suffix        → Locative/Scope
    
    // --- VACANA (Number/Cardinality) ---
    Ekavachana,   // singular
    Dvivachana,   // dual
    Bahuvachana,  // plural
    
    // --- LINGA (Gender → Mutability) ---
    Pullinga,        // masculine → mutable
    Strilinga,       // feminine  → immutable
    Napumsakalinga,  // neuter    → const
    
    // --- LAKARA (Tense → Scope/Async markers) ---
    Lat,        // present tense    → normal fn
    Lit,        // perfect tense    → memoized fn
    Lut,        // periphrastic fut → scheduled fn
    Lrt,        // simple future    → async fn
    Let,        // subjunctive      → conditional fn
    Lot,        // imperative       → main/entry fn
    Lan,        // imperfect        → deprecated fn
    Vidhilin,   // potential        → trait fn
    Asihlin,    // benedictive      → optional fn
    Lun,        // aorist           → unsafe fn
    
    // --- GANA (Verb class) ---
    Bhvadi,      // class 1
    Adadi,       // class 2
    Juhotyadi,   // class 3
    Divadi,      // class 4
    Svadi,       // class 5
    Tudadi,      // class 6
    Rudhadi,     // class 7
    Tanadi,      // class 8
    Kryadi,      // class 9
    Curadi,      // class 10
    
    // --- UPASARGA (Prefixes → Compiler directives / module paths) ---
    Pra,    Para,   Apa,   Sam,   Anu,
    Ava,    Nis,    Nir,   Dus,   Dur,
    Vi,     Aa,     Ni,    Adhi,  ApiUpasarga,
    Ati,    Su,     Ud,    Abhi,  Prati,
    Pari,   Upa,
    
    // --- NIPATA (Particles → Keywords/Operators) ---
    Ca,        // and (&&)
    Va,        // or (||)
    Na,        // not (!)
    Iti,       // end-of-statement / quote marker (like semicolon or closing quote)
    Eva,       // only/exactly (type assertion)
    Api,       // also/even (append)
    Tu,        // but (else)
    Yadi,      // if
    Tarhi,     // then
    Anyatha,   // otherwise (else)
    Kintu,     // however (break)
    Punah,     // again (continue)
    Atha,      // now/begin (block start)
    Alam,      // enough (return/stop)
    
    // --- SANDHI SPECIAL TOKENS ---
    Visarga,   // ḥ character standalone
    Anusvara,  // ṃ character standalone
    
    // --- PUNCTUATION & STRUCTURE ---
    Dot,           // .  (method call / samasa separator)
    DoubleColon,   // :: (type/namespace separator)
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    Comma,         // ,
    Semicolon,     // ;
    Equals,        // =
    Arrow,         // ->
    FatArrow,      // =>
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    EqualEqual,    // ==
    BangEqual,     // !=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    
    // --- META ---
    Newline,
    Whitespace,
    Comment,       // /* */ and //
    EOF,
    Unknown(char),
}
