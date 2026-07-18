## STEP 1 — Git state

### git log -1 --stat
```
commit a477dd8d090d88a97897502c9b7191824ee9b35e
Author: gauravverma-creator <gauravvermaofficial0@gmail.com>
Date:   Sun Jun 28 02:58:10 2026 +0000

    feat: Strings complete — VaakNode/VaakYogaNode AST, MoveChecker, 4 C-ABI Dhatus (yoga/parimana/khanda/mukta), LLVM i8* mapping, auto-free Adhikarana, 75 tests passing

 crates/devvani-ast/src/node.rs            |  16 ++
 crates/devvani-codegen/src/lib.rs         |  63 ++++++-
 crates/devvani-lexer/src/error.rs         |   3 +
 crates/devvani-lexer/src/lexer.rs         |  36 +++-
 crates/devvani-llvm/src/codegen.rs        |  95 +++++++++-
 crates/devvani-llvm/src/type_map.rs       |   21 ++-
 crates/devvani-stdlib/src/lib.rs          |   8 +
 crates/devvani-stdlib/src/string.rs       | 158 +++++++++++++++++
 crates/devvani-typesystem/src/lib.rs      |   2 +
 crates/devvani-typesystem/src/vaak.rs     | 279 ++++++++++++++++++++++++++++++
 crates/devvani-typesystem/src/vibhakti.rs |   6 +
 tests/strings/vaak_test.dvn               |   1 +
 tests/strings/vaak_test.ll                |  17 ++
 13 files changed, 693 insertions(+), 12 deletions(-)
```

### git status
```
On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   crates/devvani-compiler/examples/hello_integration.dvn

no changes added to commit (use "git add" and/or "git commit -a")
```

### git branch --show-current
```
main
```

## STEP 2 — Directory structure

### .rs files grouped by crate folder

**crates/devvani-ast/src/**
- crates/devvani-ast/src/lib.rs
- crates/devvani-ast/src/node.rs
- crates/devvani-ast/src/visitor.rs

**crates/devvani-cli/src/**
- crates/devvani-cli/src/main.rs

**crates/devvani-codegen/src/**
- crates/devvani-codegen/src/lib.rs

**crates/devvani-compiler/src/**
- crates/devvani-compiler/src/diagnostics.rs
- crates/devvani-compiler/src/lib.rs

**crates/devvani-compiler/tests/**
- crates/devvani-compiler/tests/integration_test.rs

**crates/devvani-lexer/src/**
- crates/devvani-lexer/src/error.rs
- crates/devvani-lexer/src/lexer.rs
- crates/devvani-lexer/src/lib.rs
- crates/devvani-lexer/src/sandhi.rs
- crates/devvani-lexer/src/token.rs
- crates/devvani-lexer/src/unicode_map.rs

**crates/devvani-llvm/src/**
- crates/devvani-llvm/src/codegen.rs
- crates/devvani-llvm/src/error.rs
- crates/devvani-llvm/src/lib.rs
- crates/devvani-llvm/src/target.rs
- crates/devvani-llvm/src/type_map.rs

**crates/devvani-llvm/tests/**
- crates/devvani-llvm/tests/pipeline_test.rs

**crates/devvani-module/src/**
- crates/devvani-module/src/error.rs
- crates/devvani-module/src/lib.rs
- crates/devvani-module/src/loader.rs
- crates/devvani-module/src/manifest.rs
- crates/devvani-module/src/pipeline.rs
- crates/devvani-module/src/registry.rs
- crates/devvani-module/src/resolver.rs
- crates/devvani-module/src/visibility.rs

**crates/devvani-number/src/**
- crates/devvani-number/src/arithmetic.rs
- crates/devvani-number/src/binary.rs
- crates/devvani-number/src/display.rs
- crates/devvani-number/src/lib.rs
- crates/devvani-number/src/platform.rs

**crates/devvani-parser/src/**
- crates/devvani-parser/src/error.rs
- crates/devvani-parser/src/karaka_map.rs
- crates/devvani-parser/src/lib.rs
- crates/devvani-parser/src/parser.rs
- crates/devvani-parser/src/symbol_table.rs

**crates/devvani-reversible/src/**
- crates/devvani-reversible/src/ancilla.rs
- crates/devvani-reversible/src/dvr_format.rs
- crates/devvani-reversible/src/dvri_index.rs
- crates/devvani-reversible/src/engine.rs
- crates/devvani-reversible/src/error.rs
- crates/devvani-reversible/src/lakara_reversible.rs
- crates/devvani-reversible/src/lib.rs
- crates/devvani-reversible/src/operation_log.rs
- crates/devvani-reversible/src/ram_buffer.rs
- crates/devvani-reversible/src/ssd_tier.rs
- crates/devvani-reversible/src/sutra.rs
- crates/devvani-reversible/src/tiered_storage.rs
- crates/devvani-reversible/src/types.rs
- crates/devvani-reversible/src/vedic_batch.rs
- crates/devvani-reversible/src/window.rs

**crates/devvani-reversible/src/tests/**
- crates/devvani-reversible/src/tests/test_ancilla.rs
- crates/devvani-reversible/src/tests/test_dvr_format.rs
- crates/devvani-reversible/src/tests/test_engine.rs
- crates/devvani-reversible/src/tests/test_lakara_reversible.rs
- crates/devvani-reversible/src/tests/test_operation_log.rs
- crates/devvani-reversible/src/tests/test_ram_buffer.rs
- crates/devvani-reversible/src/tests/test_ssd_tier.rs
- crates/devvani-reversible/src/tests/test_sutra.rs
- crates/devvani-reversible/src/tests/test_vedic_batch.rs
- crates/devvani-reversible/src/tests/test_window.rs

**crates/devvani-stdlib/src/**
- crates/devvani-stdlib/src/lib.rs
- crates/devvani-stdlib/src/prelude.rs
- crates/devvani-stdlib/src/registry.rs
- crates/devvani-stdlib/src/string.rs
- crates/devvani-stdlib/src/dhatu/advanced.rs
- crates/devvani-stdlib/src/dhatu/collections.rs
- crates/devvani-stdlib/src/dhatu/introspect.rs
- crates/devvani-stdlib/src/dhatu/io.rs
- crates/devvani-stdlib/src/dhatu/iteration.rs
- crates/devvani-stdlib/src/dhatu/itertools.rs
- crates/devvani-stdlib/src/dhatu/math.rs
- crates/devvani-stdlib/src/dhatu/object.rs
- crates/devvani-stdlib/src/dhatu/types.rs

**crates/devvani-typesystem/src/**
- crates/devvani-typesystem/src/checker.rs
- crates/devvani-typesystem/src/krit.rs
- crates/devvani-typesystem/src/lakara.rs
- crates/devvani-typesystem/src/lib.rs
- crates/devvani-typesystem/src/linga.rs
- crates/devvani-typesystem/src/samasa.rs
- crates/devvani-typesystem/src/symbol.rs
- crates/devvani-typesystem/src/taddhita.rs
- crates/devvani-typesystem/src/type_env.rs
- crates/devvani-typesystem/src/upasarga.rs
- crates/devvani-typesystem/src/vaak.rs
- crates/devvani-typesystem/src/vacana.rs

## STEP 3 — Full file contents

### crates/devvani-ast/src/node.rs
```rust
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
    Lat, Lit, Lut, Lrt, Let, Lot, Lan, Vidhilin, Asihlin, Lun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SamasaType {
    Tatpurusha, Dvandva, Bahuvrihi, Avyayibhava, Karmadhaaraya,
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
    KaryakramNode {
        shareera: Vec<ASTNode>,
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
}
```

### crates/devvani-lexer/src/token.rs
```rust
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
    Naama(String),            // IAST unicode identifiers (Identifier)
    PurnaankLiteral(i64),     // IntLiteral
    DashaamshaLiteral(f64),   // FloatLiteral
    VaakLiteral(String),      // StringLiteral
    
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

    // --- NIPATA & SANSKRIT KEYWORDS ---
    Ca,        // and
    Va,        // or
    Na,        // not
    Iti,       // end marker (इति)
    Eva,       // only/exactly
    Api,       // also/even
    Tu,        // but
    Yadi,      // if (यदि)
    Tarhi,     // then (तर्हि)
    Anyatha,   // otherwise/else (अन्यथा)
    Kintu,     // however
    Punah,     // again/repeat (पुनः)
    Atha,      // now/begin
    Alam,      // enough/return

    // --- NEW SANSKRIT SYNTAX TOKENS ---
    Danda,          // । — statement terminator
    Asti,           // अस्ति — assignment (fixed)
    Bhavati,        // भवति — assignment (mutable)
    Vadati,         // वदति — output
    Pathati,        // पठति — input
    Yavat,          // यावत् — while condition
    Tavat,          // तावत् — while body start
    Varam,          // वारम् — times
    Arambhah,       // आरम्भः — program start
    Samaptih,       // समाप्तिः — program end
    Yoga,           // योग — addition
    Viyoga,         // वियोग — subtraction
    Guna,           // गुण — multiplication
    Bhaga,          // भाग — division
    Sama,           // सम — equals
    AsamaH,         // असमः — not equals
    NyuunaH,        // न्यूनः — less than
    AdhikaH,        // अधिकः — greater than

    // --- SANDHI SPECIAL TOKENS ---
    Visarga,   // ḥ character standalone
    Anusvara,  // ṃ character standalone

    // --- META ---
    NavaPankti,    // Newline
    Aavakaasha,    // Whitespace
    Tippani,       // Comment
    Samaapti,      // EOF
    Unknown(char),
}
```

### crates/devvani-parser/src/parser.rs
```rust
use devvani_lexer::{Token, TokenKind, Span};
use devvani_ast::*;
use crate::error::ParseError;
use crate::symbol_table::{SymbolTable, Symbol, SymbolKind};
use crate::karaka_map::vibhakti_to_karaka;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub symbols: SymbolTable,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            symbols: SymbolTable::new(),
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParseError> {
        let mut shareera = Vec::new();
        while !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        Ok(ASTNode::KaryakramNode {
            shareera,
        })
    }

    fn parse_vakya(&mut self) -> Result<ASTNode, ParseError> {
        while self.check(&TokenKind::Danda) {
            self.advance();
        }
        if self.is_at_end() {
            return Err(ParseError::Generic("Unexpected EOF".to_string()));
        }

        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Arambhah => self.parse_karyakram(),
            TokenKind::Yadi => self.parse_yadi(),
            TokenKind::Yavat => self.parse_yavat(),
            TokenKind::Naama(ref name) => {
                if name.ends_with("-dhatu") {
                    self.parse_dhatu_def()
                } else if self.check_ahead(1, &TokenKind::Asti) {
                    self.parse_asti()
                } else if self.check_ahead(1, &TokenKind::Bhavati) {
                    self.parse_bhavati()
                } else if self.check_ahead(1, &TokenKind::Pathati) {
                    self.parse_pathati()
                } else {
                    let expr = self.parse_arithmetic()?;
                    if self.check(&TokenKind::Vadati) {
                        self.advance();
                        self.expect(TokenKind::Danda)?;
                        Ok(ASTNode::VadatiNode { mulya: Box::new(expr) })
                    } else {
                        if self.check(&TokenKind::Danda) { self.advance(); }
                        Ok(expr)
                    }
                }
            }
            _ => {
                let expr = self.parse_arithmetic()?;
                if self.check(&TokenKind::Vadati) {
                    self.advance();
                    self.expect(TokenKind::Danda)?;
                    Ok(ASTNode::VadatiNode { mulya: Box::new(expr) })
                } else {
                    if self.check(&TokenKind::Danda) { self.advance(); }
                    Ok(expr)
                }
            }
        }
    }

    fn parse_karyakram(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Arambhah)?;
        let mut shareera = Vec::new();
        while !self.check(&TokenKind::Samaptih) && !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        self.expect(TokenKind::Samaptih)?;
        Ok(ASTNode::KaryakramNode { shareera })
    }

    fn parse_dhatu_def(&mut self) -> Result<ASTNode, ParseError> {
        let name_tok = self.expect_identifier()?;
        let name = if let TokenKind::Naama(n) = name_tok.kind { n } else { unreachable!() };
        
        let mut params = Vec::new();
        while !self.is_karoti() && !self.check(&TokenKind::Danda) && !self.is_at_end() {
            let p_tok = self.expect_identifier()?;
            let p_name = if let TokenKind::Naama(n) = p_tok.kind { n } else { unreachable!() };
            let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
            params.push(KarakaParam {
                name: p_name,
                role: vibhakti_to_karaka(&vibhakti),
                vibhakti,
                span: p_tok.span,
            });
        }
        
        if self.is_karoti() { self.advance(); }
        self.expect(TokenKind::Danda)?;
        
        self.symbols.push_scope();
        for p in &params {
            let _ = self.symbols.define(&p.name, Symbol {
                name: p.name.clone(),
                kind: SymbolKind::Param { role: p.role.clone() },
                karaka: p.role.clone(),
                vibhakti: p.vibhakti.clone(),
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                defined_at: p.span,
            });
        }
        
        let mut body = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            body.push(self.parse_vakya()?);
        }
        self.symbols.pop_scope();
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }

        Ok(ASTNode::DhatuDef {
            name,
            gana: Gana::Bhvadi,
            lakara: Lakara::Lat,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params,
            return_karaka: None,
            body,
            upasargas: vec![],
            span: name_tok.span,
        })
    }

    fn is_karoti(&self) -> bool {
        if let TokenKind::Naama(n) = &self.peek().kind {
            n == "karoti"
        } else {
            false
        }
    }

    fn parse_asti(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Asti)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::AstiNode { naama, mulya: Box::new(mulya) })
    }

    fn parse_bhavati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Bhavati)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::BhavatiNode { naama, mulya: Box::new(mulya) })
    }

    fn parse_pathati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Pathati)?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::PathatiNode { naama })
    }

    fn parse_yadi(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Yadi)?;
        let sthiti = Box::new(self.parse_arithmetic()?);
        self.expect(TokenKind::Tarhi)?;
        
        let mut tarhi = Vec::new();
        while !self.check(&TokenKind::Anyatha) && !self.check(&TokenKind::Iti) && !self.is_at_end() {
            tarhi.push(self.parse_vakya()?);
        }
        
        let mut anyatha = None;
        if self.check(&TokenKind::Anyatha) {
            self.advance();
            let mut body = Vec::new();
            while !self.check(&TokenKind::Iti) && !self.is_at_end() {
                body.push(self.parse_vakya()?);
            }
            anyatha = Some(body);
        }
        
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }
        
        Ok(ASTNode::YadiNode { sthiti, tarhi, anyatha })
    }

    fn parse_yavat(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Yavat)?;
        let sthiti = Box::new(self.parse_arithmetic()?);
        self.expect(TokenKind::Tavat)?;
        
        let mut shareera = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }
        
        Ok(ASTNode::YavatNode { sthiti, shareera })
    }

    fn parse_arithmetic(&mut self) -> Result<ASTNode, ParseError> {
        let mut left = self.parse_primary()?;
        
        while let Some(tok) = self.match_any(&[TokenKind::Yoga, TokenKind::Viyoga, TokenKind::Guna, TokenKind::Bhaga]) {
            let right = self.parse_primary()?;
            left = match tok.kind {
                TokenKind::Yoga => ASTNode::YogaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Viyoga => ASTNode::ViyogaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Guna => ASTNode::GunaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Bhaga => ASTNode::BhagaNode { vama: Box::new(left), dakshina: Box::new(right) },
                _ => unreachable!(),
            };
        }
        
        if let Some(tok) = self.match_any(&[TokenKind::Sama, TokenKind::AsamaH, TokenKind::NyuunaH, TokenKind::AdhikaH]) {
            let right = self.parse_arithmetic()?;
            left = match tok.kind {
                TokenKind::Sama => ASTNode::SamaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::AsamaH => ASTNode::AsamaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::NyuunaH => ASTNode::NyuunaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::AdhikaH => ASTNode::AdhikaNode { vama: Box::new(left), dakshina: Box::new(right) },
                _ => unreachable!(),
            };
        }

        if self.check(&TokenKind::Varam) {
            self.advance();
            let mut shareera = Vec::new();
            while !self.check(&TokenKind::Iti) && !self.is_at_end() {
                shareera.push(self.parse_vakya()?);
            }
            self.expect(TokenKind::Iti)?;
            if self.check(&TokenKind::Danda) { self.advance(); }
            left = ASTNode::PunahNode { varam: Box::new(left), shareera };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::PurnaankLiteral(value) => Ok(ASTNode::PurnaankLiteral { value, span: tok.span }),
            TokenKind::DashaamshaLiteral(value) => Ok(ASTNode::DashaamshaLiteral { value, span: tok.span }),
            TokenKind::VaakLiteral(value) => Ok(ASTNode::VaakLiteral { value, span: tok.span }),
            TokenKind::Naama(name) => {
                let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
                Ok(ASTNode::Nama {
                    base: name,
                    vibhakti,
                    vacana: Vacana::Eka,
                    linga: Linga::Pullinga,
                    span: tok.span,
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: tok.kind,
                span: tok.span,
            }),
        }
    }

    fn match_vibhakti(&mut self) -> Option<Vibhakti> {
        let tok = self.peek();
        let v = match tok.kind {
            TokenKind::Visarga => Some(Vibhakti::Prathama),
            TokenKind::Anusvara => Some(Vibhakti::Dvitiya),
            _ => None,
        };
        if v.is_some() { self.advance(); }
        v
    }

    fn expect_identifier(&mut self) -> Result<Token, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Naama(_) => Ok(tok),
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: tok.kind,
                span: tok.span,
            }),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let tok = self.advance();
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: tok.kind,
                span: tok.span,
            })
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() { return false; }
        &self.peek().kind == kind
    }

    fn check_ahead(&self, n: usize, kind: &TokenKind) -> bool {
        if self.pos + n >= self.tokens.len() { return false; }
        &self.tokens[self.pos + n].kind == kind
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        for kind in kinds {
            if self.check(kind) {
                return Some(self.advance());
            }
        }
        None
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.pos += 1; }
        self.tokens[self.pos - 1].clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Samaapti
    }
}
```

### crates/devvani-typesystem/src/vibhakti.rs
```rust
use std::fmt;

/// Sanskrit Vibhakti (case) maps to compiler type roles
#[derive(Debug, Clone, PartialEq)]
pub enum VibhaktiRole {
    Prathama,   // Nominative   → Subject / Type Declaration
    Dvitiya,    // Accusative   → Function Parameter / Object
    Tritiya,    // Instrumental → Helper / Library
    Chaturthi,  // Dative       → Return Target / Receiver
    Panchami,   // Ablative     → Source / Origin
    Shashthi,   // Genitive     → Parent / Owner (struct field)
    Saptami,    // Locative     → Scope / Namespace / Module
}

#[derive(Debug, Clone, PartialEq)]
pub enum DevvaniType {
    Subject(String),       // Prathama
    Parameter(String),     // Dvitiya
    Instrument(String),    // Tritiya
    ReturnTarget(String),  // Chaturthi
    Source(String),        // Panchami
    Owner(String),         // Shashthi
    Scope(String),         // Saptami
    Unknown,
    /// Vaak — owned String type (Kartā semantics)
    Vaak,
    /// VaakBorrow — immutable string borrow (Karaṇa semantics)  
    VaakBorrow,
}

pub fn vibhakti_to_type(role: &VibhaktiRole, name: &str) -> DevvaniType {
    match role {
        VibhaktiRole::Prathama => DevvaniType::Subject(name.to_string()),
        VibhaktiRole::Dvitiya => DevvaniType::Parameter(name.to_string()),
        VibhaktiRole::Tritiya => DevvaniType::Instrument(name.to_string()),
        VibhaktiRole::Chaturthi => DevvaniType::ReturnTarget(name.to_string()),
        VibhaktiRole::Panchami => DevvaniType::Source(name.to_string()),
        VibhaktiRole::Shashthi => DevvaniType::Owner(name.to_string()),
        VibhaktiRole::Saptami => DevvaniType::Scope(name.to_string()),
    }
}

pub fn infer_type_from_suffix(word: &str) -> VibhaktiRole {
    let lower_word = word.to_lowercase();
    if lower_word.ends_with("ah") || lower_word.ends_with("ah") { // "aH" handled by lowercase
        VibhaktiRole::Prathama
    } else if lower_word.ends_with("am") {
        VibhaktiRole::Dvitiya
    } else if lower_word.ends_with("ena") {
        VibhaktiRole::Tritiya
    } else if lower_word.ends_with("aya") {
        VibhaktiRole::Chaturthi
    } else if lower_word.ends_with("at") {
        VibhaktiRole::Panchami
    } else if lower_word.ends_with("asya") {
        VibhaktiRole::Shashthi
    } else if lower_word.ends_with("e") {
        VibhaktiRole::Saptami
    } else if lower_word == "purnaankliteral" || lower_word == "dashaamshaliteral" || lower_word == "vaakliteral" {
        VibhaktiRole::Prathama
    } else {
        VibhaktiRole::Prathama
    }
}

impl fmt::Display for VibhaktiRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VibhaktiRole::Prathama => write!(f, "Prathama"),
            VibhaktiRole::Dvitiya => write!(f, "Dvitiya"),
            VibhaktiRole::Tritiya => write!(f, "Tritiya"),
            VibhaktiRole::Chaturthi => write!(f, "Chaturthi"),
            VibhaktiRole::Panchami => write!(f, "Panchami"),
            VibhaktiRole::Shashthi => write!(f, "Shashthi"),
            VibhaktiRole::Saptami => write!(f, "Saptami"),
        }
    }
}

impl fmt::Display for DevvaniType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DevvaniType::Subject(s) => write!(f, "Subject({})", s),
            DevvaniType::Parameter(s) => write!(f, "Parameter({})", s),
            DevvaniType::Instrument(s) => write!(f, "Instrument({})", s),
            DevvaniType::ReturnTarget(s) => write!(f, "ReturnTarget({})", s),
            DevvaniType::Source(s) => write!(f, "Source({})", s),
            DevvaniType::Owner(s) => write!(f, "Owner({})", s),
            DevvaniType::Scope(s) => write!(f, "Scope({})", s),
            DevvaniType::Unknown => write!(f, "Unknown"),
            DevvaniType::Vaak => write!(f, "Vaak"),
            DevvaniType::VaakBorrow => write!(f, "VaakBorrow"),
        }
    }
}
```

### crates/devvani-typesystem/src/symbol.rs
```rust
use std::fmt;
use crate::{vibhakti::DevvaniType, vacana::*, linga::*};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub devvani_type: DevvaniType,
    pub cardinality: CardinalityKind,
    pub mutability: MutabilityInfo,
    pub rust_type_hint: String,  // generated Rust type string
}

impl Symbol {
    pub fn new(
        name: &str,
        devvani_type: DevvaniType,
        vacana: &Vacana,
        linga: &Linga,
        inner_type: &str,
    ) -> Self {
        let cardinality = vacana_to_cardinality(vacana);
        let mutability = linga_to_mutability(linga);
        let rust_type_hint = vacana_to_rust_type(vacana, inner_type);
        
        // Handle shared reference in rust_type_hint if Napumsaka
        let final_rust_type = if mutability.is_shared {
            format!("&{}", rust_type_hint)
        } else {
            rust_type_hint
        };

        Self {
            name: name.to_string(),
            devvani_type,
            cardinality,
            mutability,
            rust_type_hint: final_rust_type,
        }
    }
    
    pub fn to_rust_binding(&self) -> String {
        let kw = linga_to_rust_keyword(&self.mutability.linga);
        format!("{} {}: {}", kw, self.name, self.rust_type_hint)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol(name={}, type={}, rust={})", self.name, self.devvani_type, self.rust_type_hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_eka_pullinga() {
        let sym = Symbol::new("ramah", DevvaniType::Subject("Ramah".to_string()), &Vacana::Eka, &Linga::Pullinga, "i64");
        assert_eq!(sym.to_rust_binding(), "let ramah: i64");
    }

    #[test]
    fn test_symbol_bahu_strilinga() {
        let sym = Symbol::new("sita", DevvaniType::Subject("Sita".to_string()), &Vacana::Bahu, &Linga::Strilinga, "String");
        assert_eq!(sym.to_rust_binding(), "let mut sita: Vec<String>");
    }
}
```

### crates/devvani-compiler/src/lib.rs
```rust
use devvani_lexer::{Lexer, SandhiMode};
use devvani_parser::Parser;
use devvani_codegen::{Codegen, CodegenTarget};
use devvani_reversible::VedicBatchEngine;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CompilerError {
    IoError(String),
    LexError(String),
    ParseError(String),
    CodegenError(String),
}

pub struct Compiler {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    pub reversible_engine: Option<VedicBatchEngine>,
}

impl Compiler {
    pub fn new<P: AsRef<Path>>(input: P) -> Self {
        Self {
            input_file: input.as_ref().to_path_buf(),
            output_file: None,
            reversible_engine: None,
        }
    }

    pub fn with_output<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.output_file = Some(output.as_ref().to_path_buf());
        self
    }

    pub fn compile(&self) -> Result<String, String> {
        let source = fs::read_to_string(&self.input_file)
            .map_err(|e| format!("D007: {}", e))?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize(SandhiMode::Auto)
            .map_err(|e| format!("D008: {:?}", e))?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| format!("D009: {:?}", e))?;

        let mut codegen = Codegen::new(CodegenTarget::RustSource);
        codegen.generate(&ast)
            .map_err(|e| format!("D010: {:?}", e))?;

        let rust_code = codegen.rust_source().to_string();

        if let Some(out_path) = &self.output_file {
            fs::write(out_path, &rust_code)
                .map_err(|e| format!("D006: {}", e))?;
        }

        Ok(rust_code)
    }

    /// Initialize the reversible compute engine for this compilation session.
    /// Call this before compiling if reversible tracking is needed.
    /// ssd_dir: path where .dvr/.dvri files will be written.
    pub fn enable_reversible_engine(&mut self, ssd_dir: impl AsRef<std::path::Path>) {
        use devvani_reversible::WindowConfig;
        match VedicBatchEngine::new(
            32 * 1024 * 1024, // 32MB RAM tier
            WindowConfig {
                max_ops: 512,
                purge_fraction: 0.80,
                dependency_check: true,
            },
            ssd_dir,
            16,  // coalesce threshold
            64,  // batch size
        ) {
            Ok(engine) => {
                self.reversible_engine = Some(engine);
            }
            Err(e) => {
                eprintln!("[devvani-compiler] warning: reversible engine init failed: {}", e);
            }
        }
    }

    /// Returns true if the reversible engine is active for this session.
    pub fn has_reversible_engine(&self) -> bool {
        self.reversible_engine.is_some()
    }
}

pub mod diagnostics;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compile_nonexistent_file() {
        let compiler = Compiler::new("nonexistent.dvn");
        let result = compiler.compile();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("D007"));
    }

    #[test]
    fn test_compile_hello() {
        let _ = fs::create_dir_all("examples");
        let _ = fs::write("examples/hello_test.dvn", "phalam asti 5 ।");
        
        let compiler = Compiler::new("examples/hello_test.dvn");
        let result = compiler.compile();
        if let Err(ref e) = result { println!("Error: {}", e); }
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_ganana() {
        let _ = fs::write("examples/ganana_test.dvn", "eka asti 1 । 1 yoga 2 vadati ।");
        let compiler = Compiler::new("examples/ganana_test.dvn");
        let result = compiler.compile();
        if let Err(ref e) = result { println!("Error: {}", e); }
        assert!(result.is_ok());
    }
}
```

### crates/devvani-typesystem/src/vaak.rs
```rust
//! vaak.rs — Vaak (वाक्) String Type with Kāraka Ownership
//!
//! DESIGN AUTHORITY: Pāṇini's Aṣṭādhyāyī + Vaiśeṣika Dravya theory
//!
//! Vaiśeṣika: Śabda (sound/word) is a Guṇa (quality) of Ākāśa (space).
//! A Vaak (speech/string) is a Dravya (substance) that can be owned,
//! borrowed, or transferred — just like physical objects in Vaiśeṣika ontology.
//!
//! Kāraka Ownership Model for Vaak:
//!   Kartā (Prathamā)  → Owner: heap-allocated String, one owner at a time
//!   Karaṇa (Tṛtīyā)  → Immutable borrow: read-only &str reference
//!   Apādāna (Pañcamī) → Move: transfer ownership, original invalidated
//!
//! Mīmāṃsā Borrow Tiers (for future Sampradāna):
//!   Apūrva-vidhi   → New mutable borrow (first-time write access)
//!   Niyama-vidhi   → Restricted borrow (read-only)
//!   Pariṣaṅkhyā   → Exclusive borrow (only one mutable borrower)

use crate::vibhakti::DevvaniType;
use std::collections::HashMap;

/// VaakOwnership — tracks the ownership state of a Vaak string variable
#[derive(Debug, Clone, PartialEq)]
pub enum VaakOwnership {
    /// Kartā: this binding owns the string (heap-allocated)
    Karta,
    /// Karaṇa: this binding borrows the string immutably
    Karana,
    /// Apādāna: this binding has received ownership via move
    /// The original Kartā binding is now Moved (invalid)
    Apadana,
    /// Moved: this binding has transferred ownership away — now invalid
    Moved,
}

/// VaakSymbol — a string variable in the Devvani type system
#[derive(Debug, Clone)]
pub struct VaakSymbol {
    /// Variable name (IAST identifier)
    pub naama: String,
    /// Current ownership state
    pub ownership: VaakOwnership,
    /// Is mutable? (Pullinga=mutable, Strilinga=immutable, Napumsaka=const)
    pub is_mutable: bool,
    /// The DevvaniType for this symbol
    pub devvani_type: DevvaniType,
    /// LLVM/Rust type hint: "String" for Karta, "&str" for Karana
    pub rust_type_hint: String,
}

impl VaakSymbol {
    /// Create a new owned Vaak string (Kartā semantics)
    pub fn new_karta(naama: &str, is_mutable: bool) -> Self {
        Self {
            naama: naama.to_string(),
            ownership: VaakOwnership::Karta,
            is_mutable,
            devvani_type: DevvaniType::Vaak,
            rust_type_hint: "String".to_string(),
        }
    }

    /// Create an immutable borrow of a Vaak string (Karaṇa semantics)
    pub fn new_karana(naama: &str) -> Self {
        Self {
            naama: naama.to_string(),
            ownership: VaakOwnership::Karana,
            is_mutable: false,
            devvani_type: DevvaniType::VaakBorrow,
            rust_type_hint: "&str".to_string(),
        }
    }

    /// Move ownership from this symbol to another (Apādāna semantics).
    /// After calling this, self.ownership becomes Moved — it is invalid.
    /// Returns a new VaakSymbol with Apadana ownership (the receiver).
    pub fn move_to(&mut self, new_naama: &str) -> Result<VaakSymbol, VaakError> {
        match self.ownership {
            VaakOwnership::Moved => Err(VaakError::UseAfterMove {
                naama: self.naama.clone(),
            }),
            VaakOwnership::Karana => Err(VaakError::CannotMoveBorrow {
                naama: self.naama.clone(),
            }),
            VaakOwnership::Karta | VaakOwnership::Apadana => {
                self.ownership = VaakOwnership::Moved;
                Ok(VaakSymbol {
                    naama: new_naama.to_string(),
                    ownership: VaakOwnership::Apadana,
                    is_mutable: self.is_mutable,
                    devvani_type: DevvaniType::Vaak,
                    rust_type_hint: "String".to_string(),
                })
            }
        }
    }

    /// Check if this symbol can be read (not Moved)
    pub fn can_read(&self) -> Result<(), VaakError> {
        if self.ownership == VaakOwnership::Moved {
            Err(VaakError::UseAfterMove { naama: self.naama.clone() })
        } else {
            Ok(())
        }
    }

    /// Check if this symbol can be written (must be Karta + mutable)
    pub fn can_write(&self) -> Result<(), VaakError> {
        match self.ownership {
            VaakOwnership::Moved => Err(VaakError::UseAfterMove { naama: self.naama.clone() }),
            VaakOwnership::Karana => Err(VaakError::ImmutableBorrow { naama: self.naama.clone() }),
            _ if !self.is_mutable => Err(VaakError::NotMutable { naama: self.naama.clone() }),
            _ => Ok(()),
        }
    }

    /// Rust binding string for codegen reference
    pub fn to_rust_binding(&self) -> String {
        let kw = if self.is_mutable { "let mut" } else { "let" };
        format!("{} {}: {}", kw, self.naama, self.rust_type_hint)
    }
}

/// VaakError — ownership violation errors for Vaak strings
/// Named after Sanskrit grammar error tradition (Doṣa = defect)
#[derive(Debug, Clone, PartialEq)]
pub enum VaakError {
    /// Used a string variable after its ownership was moved (Apādāna completed)
    UseAfterMove { naama: String },
    /// Tried to move an immutable borrow (Karaṇa cannot be moved)
    CannotMoveBorrow { naama: String },
    /// Tried to write to an immutable binding (Strilinga/Napumsaka)
    ImmutableBorrow { naama: String },
    /// Tried to write to a non-mutable variable
    NotMutable { naama: String },
}

impl std::fmt::Display for VaakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaakError::UseAfterMove { naama } =>
                write!(f, "Doṣa D030: '{}' — svāmitva-hāni (ownership moved, cannot use)", naama),
            VaakError::CannotMoveBorrow { naama } =>
                write!(f, "Doṣa D031: '{}' — karaṇa-apādāna-doṣa (cannot move an immutable borrow)", naama),
            VaakError::ImmutableBorrow { naama } =>
                write!(f, "Doṣa D032: '{}' — karaṇa-lekha-doṣa (cannot write to immutable borrow)", naama),
            VaakError::NotMutable { naama } =>
                write!(f, "Doṣa D033: '{}' — sthira-lekha-doṣa (variable is not mutable)", naama),
        }
    }
}

impl std::error::Error for VaakError {}

pub struct MoveChecker {
    pub ownership_map: HashMap<String, VaakOwnership>,
}

impl MoveChecker {
    pub fn new() -> Self {
        Self {
            ownership_map: HashMap::new(),
        }
    }

    pub fn check_use(&self, naama: &str) -> Result<(), VaakError> {
        if let Some(ownership) = self.ownership_map.get(naama) {
            if *ownership == VaakOwnership::Moved {
                return Err(VaakError::UseAfterMove { naama: naama.to_string() });
            }
        }
        Ok(())
    }

    pub fn do_move(&mut self, naama: &str) -> Result<(), VaakError> {
        match self.ownership_map.get(naama) {
            Some(VaakOwnership::Moved) => Err(VaakError::UseAfterMove { naama: naama.to_string() }),
            Some(VaakOwnership::Karana) => Err(VaakError::CannotMoveBorrow { naama: naama.to_string() }),
            Some(VaakOwnership::Karta | VaakOwnership::Apadana) => {
                self.ownership_map.insert(naama.to_string(), VaakOwnership::Moved);
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn register(&mut self, naama: String, ownership: VaakOwnership) {
        self.ownership_map.insert(naama, ownership);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karta_creation() {
        let sym = VaakSymbol::new_karta("vāk", true);
        assert_eq!(sym.ownership, VaakOwnership::Karta);
        assert_eq!(sym.rust_type_hint, "String");
        assert!(sym.can_read().is_ok());
        assert!(sym.can_write().is_ok());
    }

    #[test]
    fn test_karana_cannot_write() {
        let sym = VaakSymbol::new_karana("vāk_ref");
        assert!(sym.can_read().is_ok());
        assert!(sym.can_write().is_err());
    }

    #[test]
    fn test_move_invalidates_original() {
        let mut owner = VaakSymbol::new_karta("mūla", true);
        let _new_owner = owner.move_to("navam").unwrap();
        assert_eq!(owner.ownership, VaakOwnership::Moved);
        assert!(owner.can_read().is_err());
    }

    #[test]
    fn test_use_after_move_error() {
        let mut owner = VaakSymbol::new_karta("mūla", true);
        let _ = owner.move_to("navam").unwrap();
        let err = owner.can_read().unwrap_err();
        assert!(matches!(err, VaakError::UseAfterMove { .. }));
    }

    #[test]
    fn test_cannot_move_borrow() {
        let mut borrow = VaakSymbol::new_karana("rin");
        let err = borrow.move_to("navam").unwrap_err();
        assert!(matches!(err, VaakError::CannotMoveBorrow { .. }));
    }

    #[test]
    fn test_immutable_karta_cannot_write() {
        let sym = VaakSymbol::new_karta("sthira", false);
        assert!(sym.can_write().is_err());
        assert!(matches!(sym.can_write().unwrap_err(), VaakError::NotMutable { .. }));
    }

    #[test]
    fn test_karana_type() {
        let sym = VaakSymbol::new_karana("rin");
        assert_eq!(sym.devvani_type, DevvaniType::VaakBorrow);
        assert_eq!(sym.rust_type_hint, "&str");
    }

#[test]
    fn test_karta_register() {
        let mut checker = MoveChecker::new();
        checker.register("vāk".to_string(), VaakOwnership::Karta);
        assert!(checker.check_use("vāk").is_ok());
    }

    #[test]
    fn test_move_transfers() {
        let mut checker = MoveChecker::new();
        checker.register("mūla".to_string(), VaakOwnership::Karta);
        assert!(checker.do_move("mūla").is_ok());
        assert!(checker.check_use("mūla").is_err());
    }

    #[test]
    fn test_double_move_fails() {
        let mut checker = MoveChecker::new();
        checker.register("mūla".to_string(), VaakOwnership::Karta);
        let _ = checker.do_move("mūla").unwrap();
        assert!(checker.do_move("mūla").is_err());
    }

    #[test]
    fn test_karana_no_move() {
        let mut checker = MoveChecker::new();
        checker.register("rin".to_string(), VaakOwnership::Karana);
        assert!(checker.check_use("rin").is_ok());
        assert!(checker.do_move("rin").is_err());
    }
}
```

## STEP 4 — Keyword search

### grep -rn "DhatuDef" crates/
```
crates/devvani-ast/src/node.rs:93:    DhatuDef {
crates/devvani-ast/src/visitor.rs:26:            ASTNode::DhatuDef { .. } => self.visit_dhatu_def(node),
crates/devvani-ast/src/visitor.rs:73:        if let ASTNode::DhatuDef { name, lakara, .. } = node {
crates/devvani-ast/src/visitor.rs:75:            println!("DhatuDef [{}] lakara={:?}", name, lakara);
crates/devvani-codegen/src/lib.rs:317:            ASTNode::DhatuDef { name, params, body, lakara, .. } => {
crates/devvani-llvm/src/codegen.rs:95:            ASTNode::DhatuDef { name, params, body, .. } => {
crates/devvani-llvm/tests/pipeline_test.rs:16:            ASTNode::DhatuDef {
crates/devvani-llvm/tests/pipeline_test.rs:45:            ASTNode::DhatuDef {
crates/devvani-parser/src/parser.rs:131:        Ok(ASTNode::DhatuDef {
crates/devvani-typesystem/src/checker.rs:152:            ASTNode::DhatuDef { name, params, body, lakara, .. } => {
```

### grep -rn "fn parse_dhatu" crates/
```
crates/devvani-parser/src/parser.rs:90:    fn parse_dhatu_def(&mut self) -> Result<ASTNode, ParseError> {
```

### grep -rn "recursion" crates/ -i
```
crates/devvani-module/src/resolver.rs:37:        let mut recursion_stack = HashSet::new();
crates/devvani-module/src/resolver.rs:42:                if let Some(cycle) = self.dfs(node, &mut visited, &mut recursion_stack, &mut path) {
crates/devvani-module/src/resolver.rs:54:        recursion_stack: &mut HashSet<String>,
crates/devvani-module/src/resolver.rs:58:                recursion_stack.insert(node.to_string());
crates/devvani-module/src/resolver.rs:64:                    if let Some(cycle) = self.dfs(dep, visited, recursion_stack, path) {
crates/devvani-module/src/resolver.rs:67:                } else if recursion_stack.contains(dep) {
crates/devvani-module/src/resolver.rs:77:        recursion_stack.remove(node);
```

### grep -rn "Avartana" crates/ -i
```
(no results)
```

### grep -rn "call" crates/ -i --include=*.rs -l
```
crates/devvani-ast/src/node.rs
crates/devvani-ast/src/visitor.rs
crates/devvani-codegen/src/lib.rs
crates/devvani-compiler/src/lib.rs
crates/devvani-llvm/src/codegen.rs
crates/devvani-module/src/pipeline.rs
crates/devvani-module/src/resolver.rs
crates/devvani-reversible/src/lakara_reversible.rs
crates/devvani-reversible/src/ram_buffer.rs
crates/devvani-reversible/src/sutra.rs
crates/devvani-reversible/src/tests/test_lakara_reversible.rs
crates/devvani-reversible/src/tests/test_operation_log.rs
crates/devvani-reversible/src/tiered_storage.rs
crates/devvani-reversible/src/types.rs
crates/devvani-reversible/src/vedic_batch.rs
crates/devvani-stdlib/src/dhatu/advanced.rs
crates/devvani-stdlib/src/dhatu/collections.rs
crates/devvani-stdlib/src/dhatu/introspect.rs
crates/devvani-stdlib/src/dhatu/io.rs
crates/devvani-stdlib/src/dhatu/iteration.rs
crates/devvani-stdlib/src/dhatu/itertools.rs
crates/devvani-stdlib/src/dhatu/math.rs
crates/devvani-stdlib/src/dhatu/object.rs
crates/devvani-stdlib/src/dhatu/types.rs
crates/devvani-stdlib/src/lib.rs
crates/devvani-stdlib/src/prelude.rs
crates/devvani-stdlib/src/registry.rs
crates/devvani-stdlib/src/string.rs
crates/devvani-typesystem/src/checker.rs
crates/devvani-typesystem/src/vaak.rs
```

### grep -rn "enum.*Error" crates/ --include=*.rs
```
crates/devvani-codegen/src/lib.rs:11:pub enum CodegenError {
crates/devvani-compiler/src/lib.rs:9:pub enum CompilerError {
crates/devvani-lexer/src/error.rs:5:pub enum LexError {
crates/devvani-llvm/src/error.rs:4:pub enum DevvaniLLVMError {
crates/devvani-module/src/error.rs:2:pub enum ModuleError {
crates/devvani-number/src/lib.rs:22:pub enum NumberError {
crates/devvani-parser/src/error.rs:6:pub enum ParseError {
crates/devvani-reversible/src/error.rs:4:pub enum ReversibleError {
crates/devvani-stdlib/src/lib.rs:19:pub enum StdlibError {
crates/devvani-typesystem/src/checker.rs:6:pub enum TypeCheckError {
crates/devvani-typesystem/src/upasarga.rs:6:pub enum UpasargaError {
crates/devvani-typesystem/src/vaak.rs:127:pub enum VaakError {
```

### grep -rn "D0" crates/ --include=*.rs
```
crates/devvani-codegen/src/lib.rs:106:                        errors.push(format!("Doṣa D030: \'{}\' — svāmitva-hāni (ownership moved, cannot use)", naama));
crates/devvani-compiler/src/diagnostics.rs:17:    pub code: String,          // e.g. "D001", "S002"
crates/devvani-compiler/src/diagnostics.rs:60:                code: "D001".to_string(),
crates/devvani-compiler/src/diagnostics.rs:71:                code: "D002".to_string(),
crates/devvani-compiler/src/diagnostics.rs:82:                code: "D003".to_string(),
crates/devvani-compiler/src/diagnostics.rs:91:                code: "D011".to_string(),
crates/devvani-compiler/src/diagnostics.rs:105:                code: "D004".to_string(),
crates/devvani-compiler/src/diagnostics.rs:115:                code: "D005".to_string(),
crates/devvani-compiler/src/diagnostics.rs:124:                code: "D006".to_string(),
crates/devvani-compiler/src/diagnostics.rs:138:                code: "D007".to_string(),
crates/devvani-compiler/src/diagnostics.rs:149:                code: "D008".to_string(),
crates/devvani-compiler/src/diagnostics.rs:159:                code: "D009".to_string(),
crates/devvani-compiler/src/diagnostics.rs:169:                code: "D010".to_string(),
crates/devvani-compiler/src/diagnostics.rs:201:        assert_eq!(diag.code, "D001");
crates/devvani-compiler/src/diagnostics.rs:209:        assert_eq!(diag.code, "D009");
crates/devvani-compiler/src/diagnostics.rs:227:        assert!(report.contains("D001"));
crates/devvani-compiler/src/diagnostics.rs:228:        assert!(report.contains("D009"));
crates/devvani-compiler/src/lib.rs:38:            .map_err(|e| format!("D007: {}", e))?;
crates/devvani-compiler/src/lib.rs:42:            .map_err(|e| format!("D008: {:?}", e))?;
crates/devvani-compiler/src/lib.rs:46:            .map_err(|e| format!("D009: {:?}", e))?;
crates/devvani-compiler/src/lib.rs:50:            .map_err(|e| format!("D010: {:?}", e))?;
crates/devvani-compiler/src/lib.rs:56:                .map_err(|e| format!("D006: {}", e))?;
crates/devvani-compiler/src/lib.rs:105:        assert!(result.unwrap_err().contains("D007"));
crates/devvani-reversible/src/lakara_reversible.rs:85:    /// D020: Inverse Dhātu not found in scope
crates/devvani-reversible/src/lakara_reversible.rs:87:    /// D021: PratyavartyaLan used without a recorded OpId
crates/devvani-reversible/src/lakara_reversible.rs:89:    /// D022: AnapravartyaLot in a reversible context (side effect in reversible function)
crates/devvani-reversible/src/lakara_reversible.rs:91:    /// D023: Uncomputation attempted on AnapravartyaLot
crates/devvani-reversible/src/lakara_reversible.rs:98:            ReversibleDiagnostic::InverseDhatuNotFound { .. } => "D020",
crates/devvani-reversible/src/lakara_reversible.rs:99:            ReversibleDiagnostic::MissingRecordedOpId { .. } => "D021",
crates/devvani-reversible/src/lakara_reversible.rs:100:            ReversibleDiagnostic::SideEffectInReversibleContext { .. } => "D022",
crates/devvani-reversible/src/lakara_reversible.rs:101:            ReversibleDiagnostic::UncomputeOnIrreversible { .. } => "D023",
crates/devvani-reversible/src/lakara_reversible.rs:108:                format!("D020: inverse dhatu \'{}\' not found in current scope", dhatu_name)
crates/devvani-reversible/src/lakara_reversible.rs:111:                format!("D021: PratyavartyaLan on \'{}\' has no recorded OpId — was the operation executed?", dhatu_name)
crates/devvani-reversible/src/lakara_reversible.rs:114:                format!("D022: side effect \'{}\' inside a reversible Dhātu — use AnapravartyaLot or move outside", effect_description)
crates/devvani-reversible/src/lakara_reversible.rs:117:                format!("D023: cannot uncompute \'{}\' — marked AnapravartyaLot (irreversible)", dhatu_name)
crates/devvani-reversible/src/tests/test_lakara_reversible.rs:38:    assert_eq!(d.code(), "D020");
crates/devvani-reversible/src/tests/test_lakara_reversible.rs:39:    assert!(d.message().contains("D020"));
crates/devvani-reversible/src/tests/test_lakara_reversible.rs:44:    assert_eq!(d2.code(), "D023");
crates/devvani-reversible/src/tests/test_lakara_reversible.rs:45:    assert!(d2.message().contains("D023"));
crates/devvani-typesystem/src/vaak.rs:142:                write!(f, "Doṣa D030: \'{}\' — svāmitva-hāni (ownership moved, cannot use)", naama),
crates/devvani-typesystem/src/vaak.rs:144:                write!(f, "Doṣa D031: \'{}\' — karaṇa-apādāna-doṣa (cannot move an immutable borrow)", naama),
crates/devvani-typesystem/src/vaak.rs:146:                write!(f, "Doṣa D032: \'{}\' — karaṇa-lekha-doṣa (cannot write to immutable borrow)", naama),
crates/devvani-typesystem/src/vaak.rs:148:                write!(f, "Doṣa D033: \'{}\' — sthira-lekha-doṣa (variable is not mutable)", naama),
```

## STEP 5 — Error code inventory

### Error code definition file (crates/devvani-compiler/src/diagnostics.rs)

```rust
use devvani_typesystem::TypeCheckError;
use devvani_codegen::CodegenError;
use crate::CompilerError;

// Severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Dosha,    // Error   (दोष)
    Sanka,    // Warning (शंका)
    Suchana,  // Info    (सूचना)
}

// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,          // e.g. "D001", "S002"
    pub sanskrit_title: String,// e.g. "अपरिचित नाम"
    pub roman_title: String,   // e.g. "Aparicita Nama"
    pub message: String,       // full explanation
    pub sutra_ref: Option<String>, // e.g. "sutra 1.4.54"
    pub hint: Option<String>,  // suggested fix
}

impl Diagnostic {
    pub fn display(&self) -> String {
        let severity_str = match self.severity {
            Severity::Dosha => "दोष Dosha",
            Severity::Sanka => "शंका Sanka",
            Severity::Suchana => "सूचना Suchana",
        };

        let mut output = format!(
            "── {} {} | {} ──────────────────\n",
            severity_str, self.code, self.roman_title
        );
        output.push_str(&format!(" {}: {}\n", self.sanskrit_title, self.message));
        
        if let Some(sutra) = &self.sutra_ref {
            output.push_str(&format!(" Sutra: {}\n", sutra));
        }
        
        if let Some(hint) = &self.hint {
            output.push_str(&format!(" Hint: {}\n", hint));
        }
        
        output.push_str("────────────────────────────────────────────────");
        output
    }
}

// Diagnostic registry — maps error kinds to Diagnostics
pub struct DiagnosticEngine;

impl DiagnosticEngine {
    pub fn from_type_error(err: &TypeCheckError) -> Diagnostic {
        match err {
            TypeCheckError::NaamaApraapta(name) => Diagnostic {
                severity: Severity::Dosha,
                code: "D001".to_string(),
                sanskrit_title: "अपरिचित नाम".to_string(),
                roman_title: "Aparicita Nama".to_string(),
                message: format!(
                    "'{}' Prathama Vibhakti mein Kartā ke roop mein \
                     nahi mila. Pehle ise define karo.", name),
                sutra_ref: Some("1.4.54 (Kartā — svatantraḥ kartā)".to_string()),
                hint: Some(format!("'{}' ko pehle declare karo: rāmaḥ", name)),
            },
            TypeCheckError::PrakaaraVaisamya { expected, found } => Diagnostic {
                severity: Severity::Dosha,
                code: "D002".to_string(),
                sanskrit_title: "विभक्ति-भेद".to_string(),
                roman_title: "Vibhakti Bheda".to_string(),
                message: format!(
                    "Pratyāśit (expected): {} — Prāpta (found): {}. \
                     Vibhakti mismatch.", expected, found),
                sutra_ref: Some("1.1.2".to_string()),
                hint: Some("Sahi Vibhakti pratyaya lagao.".to_string()),
            },
            TypeCheckError::SatyaasatyaApekshita(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D003".to_string(),
                sanskrit_title: "सत्यासत्य-अपेक्षित".to_string(),
                roman_title: "Satyaasatya Apeksita".to_string(),
                message: format!("Satyasatya (Bool) अपेक्षित है: {}", msg),
                sutra_ref: Some("1.1.3".to_string()),
                hint: Some("Yadi/Yavat ki sthiti Satyasatya honi chahiye.".to_string()),
            },
            TypeCheckError::PrakaaraAsangata(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D011".to_string(),
                sanskrit_title: "असंगत प्रकार".to_string(),
                roman_title: "Prakara Asangata".to_string(),
                message: format!("Yaha prakara asangata hai: {}", msg),
                sutra_ref: None,
                hint: None,
            },
        }
    }

    pub fn from_codegen_error(err: &CodegenError) -> Diagnostic {
        match err {
            CodegenError::UnsupportedNode(n) => Diagnostic {
                severity: Severity::Dosha,
                code: "D004".to_string(),
                sanskrit_title: "असमर्थित पद".to_string(),
                roman_title: "Asamarthita Pada".to_string(),
                message: format!("'{}' — yeh pada abhi codegen mein \
                                  samarthit nahi.", n),
                sutra_ref: None,
                hint: Some("Devvani ke supported constructs dekho.".to_string()),
            },
            CodegenError::TypeCheckFailed(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D005".to_string(),
                sanskrit_title: "प्रकार-परीक्षा विफल".to_string(),
                roman_title: "Prakar Pariksha Vifal".to_string(),
                message: format!("Type check fail: {}", msg),
                sutra_ref: None,
                hint: None,
            },
            CodegenError::IoError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D006".to_string(),
                sanskrit_title: "संचिका-दोष".to_string(),
                roman_title: "Sanchika Dosha".to_string(),
                message: format!("File operation fail: {}", msg),
                sutra_ref: None,
                hint: Some("File path aur permissions check karo.".to_string()),
            },
        }
    }

    pub fn from_compiler_error(err: &CompilerError) -> Diagnostic {
        match err {
            CompilerError::IoError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D007".to_string(),
                sanskrit_title: "संचिका-दोष".to_string(),
                roman_title: "Sanchika Dosha".to_string(),
                message: format!("'{}' file nahi mili ya padhi nahi ja \
                                  sakti.", msg),
                sutra_ref: None,
                hint: Some("Sahi file path do: devvani compile \
                              <file.dvn>".to_string()),
            },
            CompilerError::LexError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D008".to_string(),
                sanskrit_title: "वर्ण-विश्लेषण-दोष".to_string(),
                roman_title: "Varna Vishleshan Dosha".to_string(),
                message: format!("Shabda pahchana mein samasya: {}", msg),
                sutra_ref: Some("1.1.1".to_string()),
                hint: Some("IAST Unicode sahi hai? \
                              Matra aur anusvara check karo.".to_string()),
            },
            CompilerError::ParseError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D009".to_string(),
                sanskrit_title: "वाक्य-संरचना-दोष".to_string(),
                roman_title: "Vakya Sanrachna Dosha".to_string(),
                message: format!("SOV krama galat hai: {}", msg),
                sutra_ref: Some("2.1.1".to_string()),
                hint: Some("Devvani SOV order follow karo: \
                              Kartā Karma Kriyā.".to_string()),
            },
            CompilerError::CodegenError(msg) => Diagnostic {
                severity: Severity::Dosha,
                code: "D010".to_string(),
                sanskrit_title: "कोड-निर्माण-दोष".to_string(),
                roman_title: "Code Nirman Dosha".to_string(),
                message: format!("Rust code generation mein samasya: \
                                  {}", msg),
                sutra_ref: None,
                hint: None,
            },
        }
    }

    pub fn report(diagnostics: &[Diagnostic]) -> String {
        if diagnostics.is_empty() {
            return "✓ Shuddham — कोई दोष नही | No errors found.\n"
                .to_string();
        }
        diagnostics.iter()
            .map(|d| d.display())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_typesystem::TypeCheckError;

    #[test]
    fn test_from_type_error_naama_apraapta() {
        let err = TypeCheckError::NaamaApraapta("ramah".to_string());
        let diag = DiagnosticEngine::from_type_error(&err);
        assert_eq!(diag.code, "D001");
        assert!(diag.display().contains("Aparicita Nama"));
    }

    #[test]
    fn test_from_compiler_error_parse() {
        let err = CompilerError::ParseError("test".to_string());
        let diag = DiagnosticEngine::from_compiler_error(&err);
        assert_eq!(diag.code, "D009");
        assert!(diag.display().contains("SOV"));
    }

    #[test]
    fn test_report_empty() {
        let report = DiagnosticEngine::report(&[]);
        assert!(report.contains("Shuddham"));
    }

    #[test]
    fn test_report_with_diagnostics() {
        let err1 = TypeCheckError::NaamaApraapta("ramah".to_string());
        let diag1 = DiagnosticEngine::from_type_error(&err1);
        let err2 = CompilerError::ParseError("test".to_string());
        let diag2 = DiagnosticEngine::from_compiler_error(&err2);
        
        let report = DiagnosticEngine::report(&[diag1, diag2]);
        assert!(report.contains("D001"));
        assert!(report.contains("D009"));
    }
}
```

## STEP 6 — Test file locations

```
./crates/devvani-codegen/src/lib.rs
./crates/devvani-compiler/src/diagnostics.rs
./crates/devvani-compiler/src/lib.rs
./crates/devvani-compiler/tests/integration_test.rs
./crates/devvani-lexer/src/lexer.rs
./crates/devvani-llvm/src/lib.rs
./crates/devvani-llvm/tests/pipeline_test.rs
./crates/devvani-module/src/loader.rs
./crates/devvani-module/src/manifest.rs
./crates/devvani-module/src/pipeline.rs
./crates/devvani-module/src/resolver.rs
./crates/devvani-module/src/visibility.rs
./crates/devvani-reversible/src/tests/test_ancilla.rs
./crates/devvani-reversible/src/tests/test_dvr_format.rs
./crates/devvani-reversible/src/tests/test_engine.rs
./crates/devvani-reversible/src/tests/test_lakara_reversible.rs
./crates/devvani-reversible/src/tests/test_operation_log.rs
./crates/devvani-reversible/src/tests/test_ram_buffer.rs
./crates/devvani-reversible/src/tests/test_ssd_tier.rs
./crates/devvani-reversible/src/tests/test_sutra.rs
./crates/devvani-reversible/src/tests/test_vedic_batch.rs
./crates/devvani-reversible/src/tests/test_window.rs
./crates/devvani-stdlib/src/lib.rs
./crates/devvani-stdlib/src/string.rs
./crates/devvani-typesystem/src/krit.rs
./crates/devvani-typesystem/src/lakara.rs
./crates/devvani-typesystem/src/linga.rs
./crates/devvani-typesystem/src/samasa.rs
./crates/devvani-typesystem/src/symbol.rs
./crates/devvani-typesystem/src/taddhita.rs
./crates/devvani-typesystem/src/upasarga.rs
./crates/devvani-typesystem/src/vaak.rs
./crates/devvani-typesystem/src/vacana.rs
```

## STEP 7 — Cargo workspace summary

### Root Cargo.toml (workspace members)
```toml
[workspace]
members = [
    "crates/devvani-lexer",
    "crates/devvani-parser", 
    "crates/devvani-ast",
    "crates/devvani-number",
    "crates/devvani-codegen",
    "crates/devvani-compiler",
    "crates/devvani-cli",
    "crates/devvani-typesystem",
    "crates/devvani-llvm",
    "crates/devvani-stdlib",
    "crates/devvani-module",
    "crates/devvani-reversible",
]
resolver = "2"
```

### [dependencies] sections by crate

**devvani-ast:**
```toml
devvani-lexer = { path = "../devvani-lexer" }
serde = { version = "1.0", features = ["derive"] }
```

**devvani-codegen:**
```toml
devvani-ast = { path = "../devvani-ast" }
devvani-typesystem = { path = "../devvani-typesystem" }
```

**devvani-compiler:**
```toml
devvani-lexer = { path = "../devvani-lexer" }
devvani-parser = { path = "../devvani-parser" }
devvani-ast = { path = "../devvani-ast" }
devvani-codegen = { path = "../devvani-codegen" }
devvani-typesystem = { path = "../devvani-typesystem" }
devvani-stdlib = { path = "../devvani-stdlib" }
devvani-module = { path = "../devvani-module" }
devvani-reversible = { path = "../devvani-reversible" }
```

**devvani-lexer:**
```toml
devvani-number = { path = "../devvani-number" }
unicode-segmentation = "1.10.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
```

**devvani-module:**
```toml
toml = "0.8"
serde = { version = "1", features = ["derive"] }
ed25519-dalek = "2"
thiserror = "1"
dirs = "5"
```

**devvani-number:**
```toml
libm = { version = "0.2", optional = true }
```

**devvani-parser:**
```toml
devvani-lexer = { path = "../devvani-lexer" }
devvani-ast = { path = "../devvani-ast" }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**devvani-reversible:**
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
```

**devvani-stdlib:**
```toml
devvani-ast       = { path = "../devvani-ast" }
devvani-typesystem = { path = "../devvani-typesystem" }
thiserror = "1.0"
```

**devvani-typesystem:**
```toml
devvani-ast = { path = "../devvani-ast" }
```

**devvani-llvm:**
```toml
inkwell = { git = "https://github.com/TheDan64/inkwell", branch = "master", features = ["llvm14-0"] }
llvm-sys = { version = "140.1", features = ["force-dynamic"] }
devvani-ast = { path = "../devvani-ast" }
devvani-typesystem = { path = "../devvani-typesystem" }
thiserror = "1.0"
```