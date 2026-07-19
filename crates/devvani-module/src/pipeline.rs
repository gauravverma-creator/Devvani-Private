use crate::{KoshaManifest, ModuleError, ModuleLoader, ModuleResolver};

pub struct ModulePipeline {
    pub resolver: ModuleResolver,
    pub loader: ModuleLoader,
}

impl ModulePipeline {
    pub fn new() -> Self {
        Self {
            resolver: ModuleResolver::new(),
            loader: ModuleLoader::new(),
        }
    }

    pub fn process_manifest(&mut self, manifest: KoshaManifest) -> Result<(), ModuleError> {
        // Step 1: Attempt load
        self.loader.load(&manifest)?;

        // Step 2: Register module
        self.resolver.register_module(manifest);

        // Step 3: Detect circular
        if let Some(cycle) = self.resolver.detect_circular() {
            return Err(ModuleError::ChakraAvalambanam(cycle.join(" → ")));
        }

        Ok(())
    }

    pub fn resolve(
        &self,
        from_module: &str,
        import_name: &str,
    ) -> Result<&KoshaManifest, ModuleError> {
        self.resolver.resolve_import(from_module, import_name)
    }

    pub fn loaded_module_count(&self) -> usize {
        self.resolver.dependency_count()
    }
}

impl Default for ModulePipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pipeline_process_official_manifest() {
        let mut pipeline = ModulePipeline::new();
        let manifest = KoshaManifest {
            name: "deva-ai".to_string(),
            version: "0.1.0".to_string(),
            official: true,
            signature: Some("abc".to_string()),
            dependencies: HashMap::new(),
        };

        pipeline.process_manifest(manifest).unwrap();
        assert_eq!(pipeline.loaded_module_count(), 1);
    }

    #[test]
    fn test_pipeline_detects_circular_dependency() {
        let mut pipeline = ModulePipeline::new();

        // Use Resolver directly to build a cycle since process_manifest checks each step
        // and we don't have a way to process multiple at once without checking.
        // Actually, process_manifest calls register_module which adds to graph.

        // A -> B
        pipeline.resolver.register_module(KoshaManifest {
            name: "A".to_string(),
            version: "0.1.0".to_string(),
            official: false,
            signature: None,
            dependencies: [("B".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        });

        // B -> A
        let _manifest_b = KoshaManifest {
            name: "B".to_string(),
            version: "0.1.0".to_string(),
            official: false,
            signature: None,
            dependencies: [("A".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        };

        // This should fail because it creates a cycle
        // However, loader.load(&manifest_b) will fail because "B" is not in cache (it's community)
        // Let's make them official to bypass loader check if signature is present.

        let mut pipeline = ModulePipeline::new();

        pipeline.resolver.register_module(KoshaManifest {
            name: "A".to_string(),
            version: "0.1.0".to_string(),
            official: true,
            signature: Some("sig".to_string()),
            dependencies: [("B".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        });

        let manifest_b = KoshaManifest {
            name: "B".to_string(),
            version: "0.1.0".to_string(),
            official: true,
            signature: Some("sig".to_string()),
            dependencies: [("A".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        };

        let result = pipeline.process_manifest(manifest_b);
        assert!(matches!(result, Err(ModuleError::ChakraAvalambanam(_))));
    }

    #[test]
    fn test_pipeline_resolve_unknown_returns_error() {
        let pipeline = ModulePipeline::new();
        let result = pipeline.resolve("main", "unknown_pkg");
        assert!(matches!(result, Err(ModuleError::KoshaNaPraptah(_))));
    }
}
