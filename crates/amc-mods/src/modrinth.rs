use crate::types::{ModCategory, ModDownloadFile, ModSearchResult, ModSource};
use amc_core::error::{LauncherError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const API_BASE: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    slug: String,
    title: String,
    author: String,
    downloads: u64,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModDependency {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub dependency_type: String,
}

/// Full project version as returned by `/project/{id}/version`.
/// All filter fields default to empty so old/partial payloads keep parsing.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectVersionFull {
    #[serde(default)]
    pub version_number: String,
    #[serde(default)]
    pub files: Vec<VersionFile>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
}

impl ProjectVersionFull {
    /// True when this version runs on the given game + loader.
    /// An empty loader list (datapacks and the like) matches everything.
    pub fn supports(&self, mc_version: &str, loader: &str) -> bool {
        if !self.game_versions.iter().any(|v| v == mc_version) {
            return false;
        }
        let want = loader.to_lowercase();
        self.loaders.is_empty()
            || self.loaders.iter().any(|l| {
                let l = l.to_lowercase();
                l == want || l == "minecraft"
            })
    }

    /// Project ids this version strictly requires (depth-1 autodeps).
    pub fn required_dependencies(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|d| d.dependency_type == "required" && !d.project_id.is_empty())
            .map(|d| d.project_id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Hashes {
    pub sha1: Option<String>,
}

pub struct ModrinthClient {
    client: Client,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(20))
                .user_agent("AMCLauncher/2.1 (contact@aleph.network)")
                .build()
                .unwrap_or_default(),
        }
    }
}

impl ModrinthClient {
    pub async fn search(
        &self,
        query: &str,
        loader: Option<&str>,
        mc_version: Option<&str>,
        category: ModCategory,
        limit: usize,
    ) -> Result<Vec<ModSearchResult>> {
        Self::search_sorted(
            &self.client,
            query,
            loader,
            mc_version,
            category,
            limit,
            "relevance",
        )
        .await
    }

    /// Search with an explicit index (`relevance` or `downloads` for Discover).
    pub async fn search_sorted(
        client: &Client,
        query: &str,
        loader: Option<&str>,
        mc_version: Option<&str>,
        category: ModCategory,
        limit: usize,
        index: &str,
    ) -> Result<Vec<ModSearchResult>> {
        let proj_type = match category {
            ModCategory::Mod => "project_type:mod",
            ModCategory::ResourcePack => "project_type:resourcepack",
            ModCategory::Shader => "project_type:shader",
        };
        let mut facets: Vec<Vec<String>> = vec![vec![proj_type.to_string()]];

        if let Some(l) = loader {
            if !l.is_empty() && category == ModCategory::Mod {
                facets.push(vec![format!("categories:{}", l.to_lowercase())]);
            }
        }

        if let Some(v) = mc_version {
            if !v.is_empty() {
                facets.push(vec![format!("versions:{v}")]);
            }
        }

        let res = client
            .get(&format!("{API_BASE}/search"))
            .query(&[
                ("query", query),
                (
                    "facets",
                    &serde_json::to_string(&facets).unwrap_or_default(),
                ),
                ("limit", &limit.to_string()),
                ("index", &index.to_string()),
            ])
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth search failed: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Modrinth HTTP {}",
                res.status()
            )));
        }

        let resp: SearchResponse = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth JSON parse error: {e}")))?;

        Ok(resp
            .hits
            .into_iter()
            .map(|h| ModSearchResult {
                source: ModSource::Modrinth,
                id: h.slug,
                title: h.title,
                author: h.author,
                downloads: h.downloads,
                description: h.description,
                icon_url: h.icon_url,
                category,
            })
            .collect())
    }

    /// Discover: most-downloaded projects of a category (CONCEPT, P4).
    pub async fn search_trending(
        &self,
        category: ModCategory,
        limit: usize,
    ) -> Result<Vec<ModSearchResult>> {
        Self::search_sorted(&self.client, "", None, None, category, limit, "downloads").await
    }

    /// All versions of a project, newest first (API default order).
    pub async fn list_versions(&self, slug_or_id: &str) -> Result<Vec<ProjectVersionFull>> {
        let url = format!("{API_BASE}/project/{slug_or_id}/version");
        let res =
            self.client.get(&url).send().await.map_err(|e| {
                LauncherError::Network(format!("Modrinth versions query failed: {e}"))
            })?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Modrinth versions HTTP {}",
                res.status()
            )));
        }

        res.json()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth versions parse error: {e}")))
    }

    /// Newest version supporting the given game + loader, if any.
    pub fn select_compatible<'a>(
        versions: &'a [ProjectVersionFull],
        mc_version: &str,
        loader: &str,
    ) -> Option<&'a ProjectVersionFull> {
        versions.iter().find(|v| v.supports(mc_version, loader))
    }

    fn file_of(ver: &ProjectVersionFull) -> Result<ModDownloadFile> {
        let file = ver
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| ver.files.first())
            .ok_or_else(|| LauncherError::Custom("Mod version has no files".into()))?;

        Ok(ModDownloadFile {
            filename: file.filename.clone(),
            url: file.url.clone(),
            version: ver.version_number.clone(),
            sha1: file.hashes.sha1.clone(),
            size: Some(file.size),
        })
    }

    pub async fn get_download_file(
        &self,
        slug_or_id: &str,
        loader: Option<&str>,
        mc_version: Option<&str>,
    ) -> Result<ModDownloadFile> {
        let mut url = format!("{API_BASE}/project/{slug_or_id}/version");
        let mut params = Vec::new();

        if let Some(v) = mc_version {
            if !v.is_empty() {
                params.push(format!("game_versions=[\"{v}\"]"));
            }
        }

        if let Some(l) = loader {
            if !l.is_empty() {
                params.push(format!("loaders=[\"{}\"]", l.to_lowercase()));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let res =
            self.client.get(&url).send().await.map_err(|e| {
                LauncherError::Network(format!("Modrinth versions query failed: {e}"))
            })?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Modrinth versions HTTP {}",
                res.status()
            )));
        }

        let versions: Vec<ProjectVersionFull> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth versions parse error: {e}")))?;

        let ver = versions
            .into_iter()
            .next()
            .ok_or_else(|| LauncherError::Custom("No compatible mod version found".into()))?;

        Self::file_of(&ver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<ProjectVersionFull> {
        serde_json::from_str(
            r#"[
            {"version_number":"2.0","files":[],"game_versions":["1.20.1"],"loaders":["fabric"],
             "dependencies":[{"project_id":"abc","version_id":null,"dependency_type":"required"}]},
            {"version_number":"1.0","files":[],"game_versions":["1.19.2"],"loaders":["fabric"],"dependencies":[]},
            {"version_number":"0.9","files":[],"game_versions":["1.20.1"],"loaders":[],"dependencies":[]}
        ]"#,
        )
        .unwrap()
    }

    #[test]
    fn test_select_compatible_prefers_newest_match() {
        let versions = fixture();
        let picked = ModrinthClient::select_compatible(&versions, "1.20.1", "fabric").unwrap();
        assert_eq!(picked.version_number, "2.0");
        assert_eq!(picked.required_dependencies(), vec!["abc".to_string()]);
    }

    #[test]
    fn test_select_compatible_empty_loaders_match() {
        let versions = fixture();
        let only_pack: Vec<ProjectVersionFull> = serde_json::from_str(
            r#"[{"version_number":"1","files":[],"game_versions":["1.20.1"],"loaders":[]}]"#,
        )
        .unwrap();
        assert!(ModrinthClient::select_compatible(&only_pack, "1.20.1", "forge").is_some());
        assert!(versions[1].supports("1.19.2", "fabric"));
        assert!(!versions[1].supports("1.20.1", "fabric"));
        assert!(!versions[0].supports("1.20.1", "forge"));
    }

    #[test]
    fn test_select_compatible_none() {
        let versions = fixture();
        assert!(ModrinthClient::select_compatible(&versions, "1.18.2", "fabric").is_none());
    }
}
