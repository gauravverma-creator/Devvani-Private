use std::fmt;

use devvani_ast::node::{ASTNode, UpasargaDirective, UpasargaNode};

#[derive(Debug, Clone, PartialEq)]
pub enum UpasargaError {
    UpasargaSangharsha {
        a: UpasargaDirective,
        b: UpasargaDirective,
    },
    UpasargaAyogya {
        directive: UpasargaDirective,
        target: String,
    },
}

impl fmt::Display for UpasargaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpasargaError::UpasargaSangharsha { a, b } => {
                write!(f, "उपसर्ग संघर्ष: {:?} और {:?} एकसाथ अमान्य हैं", a, b)
            }
            UpasargaError::UpasargaAyogya { directive, .. } => {
                write!(f, "उपसर्ग अयोग्य: {:?} इस लक्ष्य पर लागू नहीं", directive)
            }
        }
    }
}

pub struct UpasargaChecker;

impl UpasargaChecker {
    pub fn check(&self, node: &UpasargaNode) -> Result<(), UpasargaError> {
        for i in 0..node.directives.len() {
            for j in (i + 1)..node.directives.len() {
                let a = &node.directives[i];
                let b = &node.directives[j];
                if conflicts(a, b) {
                    return Err(UpasargaError::UpasargaSangharsha {
                        a: a.clone(),
                        b: b.clone(),
                    });
                }
            }
        }

        for d in &node.directives {
            if !valid_target(d, &node.target) {
                return Err(UpasargaError::UpasargaAyogya {
                    directive: d.clone(),
                    target: format!("{:?}", node.target),
                });
            }
        }

        Ok(())
    }
}

fn conflicts(a: &UpasargaDirective, b: &UpasargaDirective) -> bool {
    matches!(
        (a, b),
        (UpasargaDirective::Export, UpasargaDirective::Private)
            | (UpasargaDirective::Private, UpasargaDirective::Export)
            | (UpasargaDirective::Inline, UpasargaDirective::Override)
            | (UpasargaDirective::Override, UpasargaDirective::Inline)
    )
}

fn valid_target(_directive: &UpasargaDirective, _target: &ASTNode) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_ast::node::Span;

    #[test]
    fn test_valid_single_upasarga_on_dhatu() {
        let n = UpasargaNode {
            directives: vec![UpasargaDirective::Export],
            target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
            span: Span { line: 0, col: 0, len: 0 },
        };
        assert!(UpasargaChecker.check(&n).is_ok());
    }

    #[test]
    fn test_valid_multi_upasarga_pra_su() {
        let n = UpasargaNode {
            directives: vec![UpasargaDirective::Export, UpasargaDirective::Inline],
            target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
            span: Span { line: 0, col: 0, len: 0 },
        };
        assert!(UpasargaChecker.check(&n).is_ok());
    }

    #[test]
    fn test_invalid_pra_ni_conflict() {
        let n = UpasargaNode {
            directives: vec![UpasargaDirective::Export, UpasargaDirective::Private],
            target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
            span: Span { line: 0, col: 0, len: 0 },
        };
        match UpasargaChecker.check(&n) {
            Err(UpasargaError::UpasargaSangharsha { .. }) => {}
            other => panic!("expected sangharsha error, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_su_vi_conflict() {
        let n = UpasargaNode {
            directives: vec![UpasargaDirective::Inline, UpasargaDirective::Override],
            target: Box::new(ASTNode::KaryakramNode { shareera: vec![] }),
            span: Span { line: 0, col: 0, len: 0 },
        };
        match UpasargaChecker.check(&n) {
            Err(UpasargaError::UpasargaSangharsha { .. }) => {}
            other => panic!("expected sangharsha error, got {:?}", other),
        }
    }
}
