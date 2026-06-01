use std::collections::HashMap;
use devvani_ast::{KarakaRole, Vibhakti, Linga, Vacana, Gana, Lakara, Upasarga};
use devvani_lexer::Span;
use crate::error::ParseError;

pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub karaka: KarakaRole,
    pub vibhakti: Vibhakti,
    pub linga: Linga,
    pub vacana: Vacana,
    pub defined_at: Span,
}

pub enum SymbolKind {
    Dhatu { gana: Gana, lakara: Lakara },
    Nama { linga: Linga },
    Param { role: KarakaRole },
    Upasarga(Upasarga),
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: &str, symbol: Symbol) -> Result<(), ParseError> {
        let current_scope = self.scopes.last_mut().unwrap();
        if let Some(existing) = current_scope.get(name) {
            return Err(ParseError::DuplicateDefinition {
                name: name.to_string(),
                first_at: existing.defined_at,
                second_at: symbol.defined_at,
            });
        }
        current_scope.insert(name.to_string(), symbol);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().unwrap().get(name)
    }
}
