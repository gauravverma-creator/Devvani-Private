use std::path::PathBuf;
use crate::{KoshaManifest, ModuleError, registry::PackageRegistry};

#[derive(Debug)]
pub struct LoadedModule {
    pub name: String,
    pub version: String,
    pub source_files: Vec<PathBuf>,
    pub is_official: bool,
}

pub struct ModuleLoader {
    registry: PackageRegistry,
    cache_dir: PathBuf,
}

impl ModuleLoader {
    pub fn new() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".devvani")
            .join("packages");
        
        Self {
            registry: PackageRegistry::new(),
            cache_dir,
        }
    }

    pub fn cache_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }

    pub fn load(&self, manifest: &KoshaManifest) -> Result<LoadedModule, ModuleError> {
        let is_official = if manifest.is_official() {
            if self.registry.is_trusted(manifest) {
                true
            } else {
                return Err(ModuleError::ApramanikaPaksha(manifest.name.clone()));
            }
        } else {
            if self.cache_path(&manifest.name).exists() {
                false
            } else {
                return Err(ModuleError::KoshaNaPraptah(manifest.name.clone()));
            }
        };

        Ok(LoadedModule {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            source_files: vec![],
            is_official,
        })
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cache_path_structure() {
        let loader = ModuleLoader::new();
        let path = loader.cache_path("deva-ai");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".devvani"));
        assert!(path_str.contains("packages"));
        assert!(path_str.contains("deva-ai"));
    }

    #[test]
    fn test_load_community_missing_returns_error() {
        let loader = ModuleLoader::new();
        let manifest = KoshaManifest {
            name: "nonexistent-pkg".to_string(),
            version: "1.0.0".to_string(),
            official: false,
            signature: None,
            dependencies: HashMap::new(),
        };
        let result = loader.load(&manifest);
        assert!(matches!(result, Err(ModuleError::KoshaNaPraptah(_))));
    }
}
