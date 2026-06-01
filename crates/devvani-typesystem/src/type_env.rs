use std::collections::HashMap;
use crate::vibhakti::DevvaniType;
use crate::symbol::Symbol;
use crate::vacana::Vacana;
use crate::linga::Linga;

#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: HashMap<String, Symbol>,
    parent: Option<Box<TypeEnv>>,
    scope_name: String,
}

impl TypeEnv {
    pub fn new(scope_name: &str) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
            scope_name: scope_name.to_string(),
        }
    }

    pub fn with_parent(scope_name: &str, parent: TypeEnv) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
            scope_name: scope_name.to_string(),
        }
    }

    pub fn define_symbol(&mut self, name: &str, symbol: Symbol) {
        self.bindings.insert(name.to_string(), symbol);
    }

    pub fn define(&mut self, name: &str, ty: DevvaniType) {
        let symbol = Symbol::new(name, ty, &Vacana::Eka, &Linga::Pullinga, "unknown");
        self.define_symbol(name, symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        if let Some(symbol) = self.bindings.get(name) {
            Some(symbol)
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    pub fn lookup_type(&self, name: &str) -> Option<&DevvaniType> {
        self.lookup(name).map(|s| &s.devvani_type)
    }

    pub fn enter_scope(&self, name: &str) -> TypeEnv {
        TypeEnv::with_parent(name, self.clone())
    }

    pub fn scope_name(&self) -> &str {
        &self.scope_name
    }
}
