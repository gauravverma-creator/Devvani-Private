pub struct DevvaniPrelude {
    pub registry: crate::registry::DhatuRegistry,
}

impl DevvaniPrelude {
    /// Called once at compiler startup
    pub fn init() -> Self {
        Self {
            registry: crate::registry::DhatuRegistry::new(),
        }
    }

    /// Check if a name is a stdlib Dhatu
    pub fn is_stdlib_dhatu(&self, name: &str) -> bool {
        self.registry.get(name).is_some()
    }

    /// Execute a stdlib Dhatu by name
    pub fn call(
        &self,
        name: &str,
        args: Vec<crate::DvnValue>,
    ) -> Result<crate::DvnValue, crate::StdlibError> {
        self.registry.call(name, args)
    }

    /// List all available Dhatu names (for autocomplete/help)
    pub fn available_dhatus(&self) -> Vec<&'static str> {
        self.registry.list_all()
    }
}

/// Global prelude accessor — call this from devvani-compiler
pub fn devvani_prelude() -> DevvaniPrelude {
    DevvaniPrelude::init()
}
