use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModCategory {
    #[default]
    Mod,
    ResourcePack,
    Shader,
}

/// Marketplace result ordering (CONCEPT P4 "фильтр по популярности").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchSort {
    /// Provider default (Modrinth `relevance`).
    #[default]
    Relevance,
    /// Most downloaded first.
    Downloads,
    /// Most followed first (Modrinth `follows`).
    Follows,
    /// Recently published first (Modrinth `newest`).
    Newest,
    /// Recently updated first (Modrinth `updated`).
    Updated,
}

impl SearchSort {
    /// Modrinth `search` index name for this ordering.
    pub fn modrinth_index(self) -> &'static str {
        match self {
            Self::Relevance => "relevance",
            Self::Downloads => "downloads",
            Self::Follows => "follows",
            Self::Newest => "newest",
            Self::Updated => "updated",
        }
    }

    /// CurseTools exposes no ordering parameter. Relevance and downloads
    /// (sorted by the caller) are meaningful; the rest fall back to
    /// relevance instead of pretending to work.
    pub fn curseforge_supported(self) -> bool {
        matches!(self, Self::Relevance | Self::Downloads)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub source: ModSource,
    pub id: String,
    pub title: String,
    pub author: String,
    pub downloads: u64,
    pub description: String,
    pub icon_url: Option<String>,
    pub category: ModCategory,
    /// Rating proxy from the marketplace (Modrinth `follows`).
    /// Zero means unknown — the UI stays silent instead of guessing.
    #[serde(default)]
    pub rating: u64,
    /// Game versions from the search hit (empty = unknown).
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// Loader slugs from the search hit (empty = unknown).
    #[serde(default)]
    pub loaders: Vec<String>,
}

impl ModSearchResult {
    /// True when this result is known to fit the game + loader.
    /// Unknown (empty) metadata never claims a fit.
    pub fn fits(&self, mc_version: &str, loader: &str) -> bool {
        if self.game_versions.is_empty() || mc_version.is_empty() {
            return false;
        }
        if !self.game_versions.iter().any(|v| v == mc_version) {
            return false;
        }
        let want = loader.to_lowercase();
        self.loaders.is_empty()
            || self
                .loaders
                .iter()
                .any(|l| l.to_lowercase() == want || l.to_lowercase() == "minecraft")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDownloadFile {
    pub filename: String,
    pub url: String,
    pub version: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMod {
    pub filename: String,
    pub path: PathBuf,
    pub name: String,
    pub id: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub file_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> ModSearchResult {
        ModSearchResult {
            source: ModSource::Modrinth,
            id: "sodium".to_string(),
            title: "Sodium".to_string(),
            author: "JellySquid".to_string(),
            downloads: 1,
            rating: 42,
            description: String::new(),
            icon_url: None,
            category: ModCategory::Mod,
            game_versions: vec!["1.20.1".to_string(), "1.20.4".to_string()],
            loaders: vec!["fabric".to_string()],
        }
    }

    #[test]
    fn test_fits() {
        let r = sample_result();
        assert!(r.fits("1.20.1", "fabric"));
        assert!(r.fits("1.20.4", "Fabric"));
        assert!(!r.fits("1.19.4", "fabric"));
        assert!(!r.fits("1.20.1", "forge"));
    }

    #[test]
    fn test_fits_unknown_never_claims() {
        let mut r = sample_result();
        r.game_versions.clear();
        r.loaders.clear();
        assert!(!r.fits("1.20.1", "fabric"));
    }

    #[test]
    fn test_rating_defaults_to_unknown() {
        let v: ModSearchResult = serde_json::from_value(serde_json::json!({
            "source": "Modrinth",
            "id": "x",
            "title": "X",
            "author": "A",
            "downloads": 5,
            "description": "",
            "icon_url": null,
            "category": "Mod",
        }))
        .expect("old payloads keep parsing");
        assert_eq!(v.rating, 0);
    }

    #[test]
    fn test_search_sort_index() {
        assert_eq!(SearchSort::default(), SearchSort::Relevance);
        assert_eq!(SearchSort::Relevance.modrinth_index(), "relevance");
        assert_eq!(SearchSort::Downloads.modrinth_index(), "downloads");
        assert_eq!(SearchSort::Follows.modrinth_index(), "follows");
        assert_eq!(SearchSort::Newest.modrinth_index(), "newest");
        assert_eq!(SearchSort::Updated.modrinth_index(), "updated");
    }

    #[test]
    fn test_curseforge_sort_support() {
        assert!(SearchSort::Relevance.curseforge_supported());
        assert!(SearchSort::Downloads.curseforge_supported());
        assert!(!SearchSort::Follows.curseforge_supported());
        assert!(!SearchSort::Newest.curseforge_supported());
        assert!(!SearchSort::Updated.curseforge_supported());
    }
}
