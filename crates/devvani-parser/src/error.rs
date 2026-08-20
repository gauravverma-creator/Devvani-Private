use devvani_ast::{KarakaRole, Lakara};
use devvani_lexer::{Span, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("ParseError: Expected {expected}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: TokenKind,
        span: Span,
    },

    #[error("ParseError: Expected {expected} but reached EOF")]
    UnexpectedEOF { expected: String, span: Span },

    #[error("ParseError: Undefined symbol '{name}'")]
    UndefinedSymbol { name: String, span: Span },

    #[error("ParseError: Duplicate definition of '{name}'")]
    DuplicateDefinition {
        name: String,
        first_at: Span,
        second_at: Span,
    },

    #[error("ParseError: Karaka conflict for role {role:?}")]
    KarakaConflict {
        role: KarakaRole,
        first: Span,
        second: Span,
    },

    #[error("ParseError: Invalid Samasa compound {components:?}")]
    InvalidSamasa { components: Vec<String>, span: Span },

    #[error("ParseError: Missing Karta (subject) in SOV statement")]
    MissingKarta { span: Span },

    #[error("ParseError: Missing Kriya (verb) in SOV statement")]
    MissingKriya { span: Span },

    #[error("ParseError: Invalid Lakara {found:?} in context {context}")]
    InvalidLakara {
        found: Lakara,
        context: String,
        span: Span,
    },

    #[error("ParseError: {0}")]
    Generic(String),

    #[error("ParseError: Assertion '{keyword}' expects exactly {expected} argument(s), but found {found}")]
    AssertionArgCount {
        keyword: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("ParseError: tarka modifier can only be used with parikshaa")]
    TarkaWithoutParikshaa { span: Span },

    #[error("ParseError: Malformed parikshaa: {reason}")]
    MalformedParikshaa { reason: String, span: Span },

    #[error("ParseError: bhashya can only appear at the top of a file, before any other declaration")]
    BhashyaNotAtTop { span: Span },

    #[error("ParseError: only one mrittika block is allowed per file")]
    DuplicateMrittika { span: Span },

    #[error("ParseError: mrittika block requires a non-empty naamadheya version string as its first entry")]
    MissingNaamadheya { span: Span },

    #[error("ParseError: aptavakya must be followed by a string literal (extern block) or dhatu (exported function), found {found:?}")]
    InvalidAptavakyaUsage { found: TokenKind, span: Span },

    #[error("ParseError: bahya-dhatu is only valid inside an aptavakya \"<abi>\" block")]
    BahyaDhatuOutsideAptavakya { span: Span },

    #[error("ParseError: bahya-dhatu must not have a body; it is a signature-only declaration")]
    BahyaDhatuMustNotHaveBody { span: Span },
}
