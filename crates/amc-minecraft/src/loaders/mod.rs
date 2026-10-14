pub mod fabric;
pub mod forge;
pub mod installer;
pub mod neoforge;
pub mod optifine;
pub mod quilt;

pub use fabric::FabricLoader;
pub use forge::ForgeLoader;
pub use installer::{find_installed_json, run_client_installer, INSTALLER_TIMEOUT};
pub use neoforge::NeoForgeLoader;
pub use optifine::OptiFineLoader;
pub use quilt::QuiltLoader;

/// First-try candidates for a Fabric/Quilt resolve: the stored pin as a
/// non-persisting candidate, or empty when unpinned/blank (the caller
/// looks up latest then). Pure for tests; the async task owns the rest.
pub fn pin_candidates(pinned: Option<String>) -> Vec<(String, bool)> {
    pinned
        .filter(|v| !v.trim().is_empty())
        .map(|v| vec![(v, false)])
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_candidates() {
        assert_eq!(
            pin_candidates(Some("0.15.11".to_string())),
            vec![("0.15.11".to_string(), false)]
        );
        assert!(pin_candidates(Some("  ".to_string())).is_empty());
        assert!(pin_candidates(None).is_empty());
    }
}
