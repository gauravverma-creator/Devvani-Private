use crate::{KoshaManifest, ModuleError};
use std::collections::{HashMap, HashSet};

pub struct ModuleResolver {
    // dependency graph: module_name -> list of its dependencies
    graph: HashMap<String, Vec<String>>,
    // resolved manifests cache
    resolved: HashMap<String, KoshaManifest>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, manifest: KoshaManifest) {
        let deps = manifest.dependencies.keys().cloned().collect();
        self.graph.insert(manifest.name.clone(), deps);
        self.resolved.insert(manifest.name.clone(), manifest);
    }

    pub fn resolve_import(
        &self,
        _from_module: &str,
        import_name: &str,
    ) -> Result<&KoshaManifest, ModuleError> {
        self.resolved
            .get(import_name)
            .ok_or_else(|| ModuleError::KoshaNaPraptah(import_name.to_string()))
    }

    pub fn detect_circular(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) = self.dfs(node, &mut visited, &mut recursion_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(deps) = self.graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs(dep, visited, recursion_stack, path) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(dep) {
                    // Found a cycle. Construct the cycle path.
                    if let Some(pos) = path.iter().position(|x| x == dep) {
                        return Some(path[pos..].to_vec());
                    }
                    return Some(vec![dep.to_string()]); // Fallback
                }
            }
        }

        recursion_stack.remove(node);
        path.pop();
        None
    }

    pub fn dependency_count(&self) -> usize {
        self.graph.len()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_register_and_resolve_module() {
        let mut resolver = ModuleResolver::new();
        let manifest = KoshaManifest {
            name: "deva-ai".to_string(),
            version: "0.1.0".to_string(),
            official: true,
            signature: Some("sig".to_string()),
            dependencies: HashMap::new(),
        };
        resolver.register_module(manifest);

        let resolved = resolver.resolve_import("caller", "deva-ai").unwrap();
        assert_eq!(resolved.name, "deva-ai");
    }

    #[test]
    fn test_resolve_unknown_module_returns_error() {
        let resolver = ModuleResolver::new();
        let result = resolver.resolve_import("caller", "unknown");
        assert!(matches!(result, Err(ModuleError::KoshaNaPraptah(_))));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = ModuleResolver::new();

        // A -> B
        resolver.register_module(KoshaManifest {
            name: "A".to_string(),
            version: "0.1.0".to_string(),
            official: false,
            signature: None,
            dependencies: [("B".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        });

        // B -> C
        resolver.register_module(KoshaManifest {
            name: "B".to_string(),
            version: "0.1.0".to_string(),
            official: false,
            signature: None,
            dependencies: [("C".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        });

        // C -> A
        resolver.register_module(KoshaManifest {
            name: "C".to_string(),
            version: "0.1.0".to_string(),
            official: false,
            signature: None,
            dependencies: [("A".to_string(), "0.1.0".to_string())]
                .into_iter()
                .collect(),
        });

        let cycle = resolver.detect_circular();
        assert!(cycle.is_some());
        let cycle_nodes = cycle.unwrap();
        assert!(cycle_nodes.contains(&"A".to_string()));
        assert!(cycle_nodes.contains(&"B".to_string()));
        assert!(cycle_nodes.contains(&"C".to_string()));
    }
}
