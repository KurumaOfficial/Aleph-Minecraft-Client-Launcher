use crate::types::{ModCategory, ModDownloadFile, ModSearchResult, ModSource};
use amc_core::error::{LauncherError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

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

/// Single-file endpoint shape (`GET /mods/{id}/files/{fileId}`).
#[derive(Debug, Deserialize)]
struct SingleResponse {
    data: FileItem,
}

#[derive(Debug, Deserialize)]
struct FileItem {
    id: u64,
    #[serde(rename = "fileName", default)]
    file_name: String,
    #[serde(rename = "downloadUrl", default)]
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
        let mut url = format!(
            "{API_BASE}/mods/search?gameId={GAME_ID}&pageSize={page_size}&searchFilter={query}"
        );

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
            return Err(LauncherError::Network(format!(
                "CurseForge HTTP {}",
                res.status()
            )));
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
                rating: 0, // CurseTools search carries no rating; UI stays silent.
                description: item.summary,
                icon_url: item.logo.map(|l| l.url),
                category: ModCategory::Mod,
                // CurseTools search items carry no version metadata.
                game_versions: Vec::new(),
                loaders: Vec::new(),
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
            return Err(LauncherError::Network(format!(
                "CurseForge HTTP {}",
                res.status()
            )));
        }

        let resp: FilesResponse = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge files parse error: {e}")))?;

        let file =
            resp.data.into_iter().next().ok_or_else(|| {
                LauncherError::Custom("No files for this Minecraft version".into())
            })?;

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

    /// Resolve one pinned pack file of a CurseForge `.zip` manifest
    /// (`projectID` + `fileID`) to a direct download (CONCEPT P4 import).
    /// Accepts both single-object and list response shapes; falls back to
    /// the `download-url` endpoint when the payload has no direct URL.
    pub async fn get_pack_file(&self, project_id: u64, file_id: u64) -> Result<ModDownloadFile> {
        let url = format!("{API_BASE}/mods/{project_id}/files/{file_id}");
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge pack file error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "CurseForge HTTP {}",
                res.status()
            )));
        }

        let text = res
            .text()
            .await
            .map_err(|e| LauncherError::Network(format!("CurseForge pack file read error: {e}")))?;

        let file: FileItem = serde_json::from_str::<SingleResponse>(&text)
            .map(|r| r.data)
            .or_else(|_| {
                serde_json::from_str::<FilesResponse>(&text).and_then(|resp| {
                    resp.data
                        .into_iter()
                        .find(|f| f.id == file_id)
                        .ok_or_else(|| serde::de::Error::custom("pack file not in response"))
                })
            })
            .map_err(|e| {
                LauncherError::Network(format!("CurseForge pack file parse error: {e}"))
            })?;

        if file.file_name.is_empty() {
            return Err(LauncherError::Custom(
                "CurseForge pack file has no name".into(),
            ));
        }

        let mut download_url = file.download_url;
        if download_url.is_none() {
            let alt = format!("{API_BASE}/mods/{project_id}/files/{file_id}/download-url");
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
            version: file_id.to_string(),
            sha1: None,
            size: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE: &str = r#"{"data": {"id": 456, "fileName": "sodium.jar",
        "downloadUrl": "https://example.com/sodium.jar"}}"#;

    const LIST: &str = r#"{"data": [{"id": 456, "fileName": "sodium.jar",
        "downloadUrl": "https://example.com/sodium.jar"}]}"#;

    #[test]
    fn test_pack_file_single_shape() {
        let file: FileItem = serde_json::from_str::<SingleResponse>(SINGLE)
            .map(|r| r.data)
            .expect("single-object shape parses");
        assert_eq!(file.id, 456);
        assert_eq!(file.file_name, "sodium.jar");
    }

    #[test]
    fn test_pack_file_list_shape() {
        let resp: FilesResponse = serde_json::from_str(LIST).expect("list shape parses");
        let file = resp
            .data
            .into_iter()
            .find(|f| f.id == 456)
            .expect("pack file found by id");
        assert_eq!(file.file_name, "sodium.jar");
    }
}
