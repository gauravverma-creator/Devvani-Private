use crate::error::ModuleError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoshaManifest {
    pub name: String,
    pub version: String,
    pub official: bool,
    pub signature: Option<String>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RawManifest {
    package: PackageMetadata,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

#[derive(Deserialize)]
struct PackageMetadata {
    name: String,
    version: String,
    #[serde(default)]
    official: bool,
    signature: Option<String>,
}

impl KoshaManifest {
    pub fn from_file(path: &Path) -> Result<Self, ModuleError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(s: &str) -> Result<Self, ModuleError> {
        let raw: RawManifest =
            toml::from_str(s).map_err(|e| ModuleError::ManifestParseError(e.to_string()))?;

        Ok(KoshaManifest {
            name: raw.package.name,
            version: raw.package.version,
            official: raw.package.official,
            signature: raw.package.signature,
            dependencies: raw.dependencies,
        })
    }

    pub fn is_official(&self) -> bool {
        self.official && self.signature.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_official_manifest() {
        let toml_str = r#"
[package]
name = "deva-ai"
version = "0.1.0"
official = true
signature = "abc123hexsignature"

[dependencies]
devvani-stdlib = "0.3.0"
"#;
        let manifest = KoshaManifest::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "deva-ai");
        assert!(manifest.is_official());
    }

    #[test]
    fn test_parse_community_manifest() {
        let toml_str = r#"
[package]
name = "community-pkg"
version = "1.0.0"
official = false

[dependencies]
"#;
        let manifest = KoshaManifest::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "community-pkg");
        assert!(!manifest.is_official());
    }

    #[test]
    fn test_official_without_signature() {
        let toml_str = r#"
[package]
name = "fake-official"
version = "0.1.0"
official = true
"#;
        let manifest = KoshaManifest::from_str(toml_str).unwrap();
        assert!(!manifest.is_official());
    }

    #[test]
    fn test_dependencies_parsed() {
        let toml_str = r#"
[package]
name = "deps-test"
version = "0.1.0"

[dependencies]
lib1 = "1.0.0"
lib2 = "2.0.0"
"#;
        let manifest = KoshaManifest::from_str(toml_str).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dependencies.get("lib1").unwrap(), "1.0.0");
        assert_eq!(manifest.dependencies.get("lib2").unwrap(), "2.0.0");
    }

    #[test]
    fn test_registry_url() {
        use crate::registry::PackageRegistry;
        let registry = PackageRegistry::new();
        assert_eq!(registry.registry_url(), "https://registry.kosha.dev");
    }
}
