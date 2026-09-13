use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use amc_core::error::{LauncherError, Result};
use crate::types::{ModDownloadFile, ModSearchResult, ModSource};

const API_BASE: &str = "https://api.curse.tools/v1/cf";
const GAME_ID: u32 = 432;

fn loader_to_id(loader: &str) -> Option<u32> {
    match loader.to_lowercase().as_str() {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: u64,
    name: String,
    #[serde(default, rename = "downloadCount")]
    downloads: u64,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    authors: Vec<Author>,
    #[serde(default)]
    logo: Option<Logo>,
}

#[derive(Debug, Deserialize)]
struct Author {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Logo {
    url: String,
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    data: Vec<FileItem>,
}

#[derive(Debug, Deserialize)]
struct FileItem {
    id: u64,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UrlResponse {
    data: Option<String>,
}

pub struct CurseForgeClient {
    client: Client,
}

impl Default for CurseForgeClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(20))
                .user_agent("AMCLauncher/2.1")
                .build()
                .unwrap_or_default(),
        }
    }
}

impl CurseForgeClient {
    pub async fn search(
        &self,
        query: &str,
        loader: Option<&str>,
        mc_version: Option<&str>,
        page_size: usize,
    ) -> Result<Vec<ModSearchResult>> {
        let mut url = format!("{API_BASE}/mods/search?gameId={GAME_ID}&pageSize={page_size}&searchFilter={query}");

        if let Some(l) = loader {
            if let Some(id) = loader_to_id(l) {
                url.push_str(&format!("&modLoaderType={id}"));
            }
        }

        if let Some(v) = mc_version {
            if !v.is_empty() {
                url.push_str(&format!("&gameVersion={v}"));
            }
        }

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge search error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!("CurseForge HTTP {}", res.status())));
        }

        let resp: SearchResponse = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge parse error: {e}")))?;

        Ok(resp
            .data
            .into_iter()
            .map(|item| ModSearchResult {
                source: ModSource::CurseForge,
                id: item.id.to_string(),
                title: item.name,
                author: item
                    .authors
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                downloads: item.downloads,
                description: item.summary,
                icon_url: item.logo.map(|l| l.url),
            })
            .collect())
    }

    pub async fn get_download_file(
        &self,
        mod_id: &str,
        loader: Option<&str>,
        mc_version: Option<&str>,
    ) -> Result<ModDownloadFile> {
        let mut url = format!("{API_BASE}/mods/{mod_id}/files?pageSize=20");

        if let Some(l) = loader {
            if let Some(id) = loader_to_id(l) {
                url.push_str(&format!("&modLoaderType={id}"));
            }
        }

        if let Some(v) = mc_version {
            if !v.is_empty() {
                url.push_str(&format!("&gameVersion={v}"));
            }
        }

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge files error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!("CurseForge HTTP {}", res.status())));
        }

        let resp: FilesResponse = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge files parse error: {e}")))?;

        let file = resp
            .data
            .into_iter()
            .next()
            .ok_or_else(|| LauncherError::Custom("No files for this Minecraft version".into()))?;

        let mut download_url = file.download_url;
        if download_url.is_none() {
            let alt = format!("{API_BASE}/mods/{mod_id}/files/{}/download-url", file.id);
            if let Ok(alt_res) = self.client.get(&alt).send().await {
                if let Ok(json) = alt_res.json::<UrlResponse>().await {
                    download_url = json.data;
                }
            }
        }

        let final_url = download_url.ok_or_else(|| {
            LauncherError::Custom("Mod author disabled external distribution".into())
        })?;

        Ok(ModDownloadFile {
            filename: file.file_name,
            url: final_url,
            version: "latest".into(),
            sha1: None,
            size: None,
        })
    }
}
