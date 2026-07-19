use crate::token::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    #[error("Unknown character '{ch}' at {span:?}")]
    UnknownCharacter { ch: char, span: Span },

    #[error("Unterminated string at {span:?}")]
    UnterminatedString { span: Span },

    #[error("Unterminated block comment at {span:?}")]
    UnterminatedBlockComment { span: Span },

    #[error("Invalid IAST sequence '{seq}' at {span:?}")]
    InvalidIASTSequence { seq: String, span: Span },

    #[error("Sandhi conflict between {rule1} and {rule2} at {span:?}")]
    SandhiConflict {
        rule1: String,
        rule2: String,
        span: Span,
    },

    #[error("Invalid escape sequence '{ch}' at {span:?}")]
    InvalidEscape { ch: char, span: Span },
}
