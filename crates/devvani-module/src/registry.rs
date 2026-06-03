use crate::manifest::KoshaManifest;

const OFFICIAL_REGISTRY_URL: &str = "https://registry.kosha.dev";

// Placeholder 32-byte public key (zeroed) — real key added later
#[allow(dead_code)]
const OFFICIAL_PUBLIC_KEY_BYTES: [u8; 32] = [0u8; 32];

pub struct PackageRegistry;

impl PackageRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn registry_url(&self) -> &str {
        OFFICIAL_REGISTRY_URL
    }

    pub fn verify_signature(&self, manifest: &KoshaManifest) -> bool {
        if !manifest.official {
            return false;
        }
        if manifest.signature.is_none() {
            return false;
        }
        // For now: if official=true and signature exists → return true
        // Real ed25519 verify will be wired in Part 3
        true
    }

    pub fn is_trusted(&self, manifest: &KoshaManifest) -> bool {
        manifest.is_official() && self.verify_signature(manifest)
    }
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
