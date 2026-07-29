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
    Naama(String),          // IAST unicode identifiers (Identifier)
    PurnaankLiteral(i64),   // IntLiteral
    DashaamshaLiteral(f64), // FloatLiteral
    VaakLiteral(String),    // StringLiteral

    // --- VIBHAKTI (Case markers as type annotations) ---
    Prathama,  // -h / -ḥ suffix  → Subject/Type
    Dvitiya,   // -m / -ṃ suffix  → Object/Param
    Tritiya,   // -ena suffix      → Instrument/Helper
    Chaturthi, // -āya suffix      → Dative/Return target
    Panchami,  // -āt suffix       → Ablative/Source
    Shashthi,  // -sya suffix      → Genitive/Parent
    Saptami,   // -e suffix        → Locative/Scope

    // --- VACANA (Number/Cardinality) ---
    Ekavachana,  // singular
    Dvivachana,  // dual
    Bahuvachana, // plural

    // --- LINGA (Gender → Mutability) ---
    Pullinga,       // masculine → mutable
    Strilinga,      // feminine  → immutable
    Napumsakalinga, // neuter    → const

    // --- LAKARA (Tense → Scope/Async markers) ---
    Lat,      // present tense    → normal fn
    Lit,      // perfect tense    → memoized fn
    Lut,      // periphrastic fut → scheduled fn
    Lrt,      // simple future    → async fn
    Let,      // subjunctive      → conditional fn
    Lot,      // imperative       → main/entry fn
    Lan,      // imperfect        → deprecated fn
    Vidhilin, // potential        → trait fn
    Asihlin,  // benedictive      → optional fn
    Lun,      // aorist           → unsafe fn

    // --- GANA (Verb class) ---
    Bhvadi,    // class 1
    Adadi,     // class 2
    Juhotyadi, // class 3
    Divadi,    // class 4
    Svadi,     // class 5
    Tudadi,    // class 6
    Rudhadi,   // class 7
    Tanadi,    // class 8
    Kryadi,    // class 9
    Curadi,    // class 10

    // --- UPASARGA (Prefixes → Compiler directives / module paths) ---
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
    Aa,
    Ni,
    Adhi,
    ApiUpasarga,
    Ati,
    Su,
    Ud,
    Abhi,
    Prati,
    Pari,
    Upa,

    // --- NIPATA & SANSKRIT KEYWORDS ---
    Ca,      // and
    Va,      // or
    Na,      // not
    Iti,     // end marker (इति)
    Eva,     // only/exactly
    Api,     // also/even
    Tu,      // but
    Yadi,    // if (यदि)
    Tarhi,   // then (तर्हि)
    Anyatha, // otherwise/else (अन्यथा)
    Kintu,   // however
    Punah,   // again/repeat (पुनः)
    Atha,    // now/begin
    Alam,    // enough/return

     // --- NEW SANSKRIT SYNTAX TOKENS ---
     Danda,    // । — statement terminator
     Asti,     // अस्ति — assignment (fixed)
     Bhavati,  // भवति — assignment (mutable)
     Vadati,   // वदति — output
     Pathati,  // पठति — input
     Yavat,    // यावत् — while condition
     Tavat,    // तावत् — while body start
     Kramasah, // क्रमशः — for-each iteration start
     Varam,    // वारम् — times
     Arambhah, // आरम्भः — program start
     Samaptih, // समाप्तिः — program end
     Yoga,     // योग — addition
     Viyoga,   // वियोग — subtraction
     Guna,     // गुण — multiplication
     Bhaga,    // भाग — division
     Sama,     // सम — equals
     AsamaH,   // असमः — not equals
     NyuunaH,  // न्यूनः — less than
     AdhikaH,  // अधिकः — greater than
     Avali,    // growable array literal keyword
     Dravya,   // struct definition keyword
     Anga,     // struct field keyword
     Nirmana,  // struct instantiation keyword
     Phalam,   // function return type keyword (Phalam-Samprapti pattern)
     Arogya,   // error-handling success constructor
     Dosha,    // error-handling failure constructor
Nidana,   // error-handling match/inspection keyword
      Samprapti, // error-propagation postfix (? operator)

      // --- OWNERSHIP (borrow/reference) ---
      Sandarbha, // reference declaration keyword
      Adhikara,  // immutable borrow marker
      Vikara,    // mutable borrow marker

      // --- SANDHI SPECIAL TOKENS ---
     Visarga,  // ḥ character standalone
     Anusvara, // ṃ character standalone

     // --- META ---
     NavaPankti, // Newline
     Aavakaasha, // Whitespace
     Tippani,    // Comment
     Samaapti,   // EOF
     Unknown(char),
      LBracket, // [
      RBracket, // ]
      Dot,      // .
}
