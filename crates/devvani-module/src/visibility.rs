#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Prakataḥ, // public  — prakaṭaḥ keyword in .dvn files
    Guptaḥ,   // private — guptaḥ keyword in .dvn files
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Guptaḥ
    }
}

#[derive(Debug, Clone)]
pub struct SymbolVisibility {
    pub symbol_name: String,
    pub visibility: Visibility,
    pub defined_in_module: String,
}

impl SymbolVisibility {
    pub fn new(symbol_name: &str, visibility: Visibility, module: &str) -> Self {
        Self {
            symbol_name: symbol_name.to_string(),
            visibility,
            defined_in_module: module.to_string(),
        }
    }

    pub fn is_accessible_from(&self, requesting_module: &str) -> bool {
        match self.visibility {
            Visibility::Prakataḥ => true,
            Visibility::Guptaḥ => self.defined_in_module == requesting_module,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_accessible_from_any_module() {
        let sym = SymbolVisibility::new("test_func", Visibility::Prakataḥ, "module_a");
        assert!(sym.is_accessible_from("module_a"));
        assert!(sym.is_accessible_from("module_b"));
    }

    #[test]
    fn test_private_only_accessible_from_own_module() {
        let sym = SymbolVisibility::new("secret_var", Visibility::Guptaḥ, "module_a");
        assert!(sym.is_accessible_from("module_a"));
        assert!(!sym.is_accessible_from("module_b"));
    }

    #[test]
    fn test_default_visibility_is_private() {
        assert_eq!(Visibility::default(), Visibility::Guptaḥ);
    }
}
