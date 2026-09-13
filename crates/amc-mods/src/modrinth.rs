use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use amc_core::error::{LauncherError, Result};
use crate::types::{ModDownloadFile, ModSearchResult, ModSource};

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

#[derive(Debug, Deserialize)]
struct ProjectVersion {
    version_number: String,
    files: Vec<VersionFile>,
}

#[derive(Debug, Deserialize)]
struct VersionFile {
    url: String,
    filename: String,
    primary: bool,
    size: u64,
    hashes: Hashes,
}

#[derive(Debug, Deserialize)]
struct Hashes {
    sha1: Option<String>,
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
        limit: usize,
    ) -> Result<Vec<ModSearchResult>> {
        let mut facets: Vec<Vec<String>> = vec![vec!["project_type:mod".to_string()]];

        if let Some(l) = loader {
            if !l.is_empty() {
                facets.push(vec![format!("categories:{}", l.to_lowercase())]);
            }
        }

        if let Some(v) = mc_version {
            if !v.is_empty() {
                facets.push(vec![format!("versions:{v}")]);
            }
        }

        let res = self
            .client
            .get(&format!("{API_BASE}/search"))
            .query(&[
                ("query", query),
                ("facets", &serde_json::to_string(&facets).unwrap_or_default()),
                ("limit", &limit.to_string()),
                ("index", &"relevance".to_string()),
            ])
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth search failed: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!("Modrinth HTTP {}", res.status())));
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
            })
            .collect())
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

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth versions query failed: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Modrinth versions HTTP {}",
                res.status()
            )));
        }

        let versions: Vec<ProjectVersion> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Modrinth versions parse error: {e}")))?;

        let ver = versions
            .into_iter()
            .next()
            .ok_or_else(|| LauncherError::Custom("No compatible mod version found".into()))?;

        let file = ver
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| ver.files.first())
            .ok_or_else(|| LauncherError::Custom("Mod version has no files".into()))?;

        Ok(ModDownloadFile {
            filename: file.filename.clone(),
            url: file.url.clone(),
            version: ver.version_number,
            sha1: file.hashes.sha1.clone(),
            size: Some(file.size),
        })
    }
}
