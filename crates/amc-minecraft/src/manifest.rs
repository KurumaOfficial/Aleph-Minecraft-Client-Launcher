use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use amc_core::error::{LauncherError, Result};
use amc_core::types::{GameVersion, ReleaseType};

pub const MOJANG_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionManifestEntry>,
}

impl VersionManifest {
    pub async fn fetch(client: &Client, cache_dir: Option<&Path>) -> Result<Self> {
        let cache_file = cache_dir.map(|d| d.join("version_manifest_v2.json"));

        let fetch_online = async {
            let res = client
                .get(MOJANG_MANIFEST_URL)
                .send()
                .await
                .map_err(|e| LauncherError::Network(format!("Manifest fetch error: {e}")))?;

            if !res.status().is_success() {
                return Err(LauncherError::Network(format!(
                    "HTTP error {} fetching manifest",
                    res.status()
                )));
            }

            let text = res
                .text()
                .await
                .map_err(|e| LauncherError::Network(format!("Reading manifest body: {e}")))?;

            let manifest: Self = serde_json::from_str(&text).map_err(LauncherError::Json)?;

            if let Some(cache_path) = &cache_file {
                if let Some(parent) = cache_path.parent() {
                    let _ = fs::create_dir_all(parent).await;
                }
                let _ = fs::write(cache_path, &text).await;
            }

            Ok(manifest)
        };

        match fetch_online.await {
            Ok(m) => Ok(m),
            Err(err) => {
                if let Some(cache_path) = &cache_file {
                    if cache_path.is_file() {
                        if let Ok(text) = fs::read_to_string(cache_path).await {
                            if let Ok(manifest) = serde_json::from_str::<Self>(&text) {
                                tracing::warn!("Using cached version manifest due to network error: {err}");
                                return Ok(manifest);
                            }
                        }
                    }
                }
                Err(err)
            }
        }
    }

    pub fn to_game_versions(&self) -> Vec<GameVersion> {
        self.versions
            .iter()
            .map(|v| {
                let release_type = match v.version_type.as_str() {
                    "release" => ReleaseType::Release,
                    "snapshot" => ReleaseType::Snapshot,
                    "old_beta" => ReleaseType::Beta,
                    "old_alpha" => ReleaseType::Alpha,
                    _ => ReleaseType::Old,
                };

                let parsed_time = DateTime::parse_from_rfc3339(&v.release_time)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                GameVersion {
                    id: v.id.clone(),
                    release_type,
                    url: v.url.clone(),
                    release_time: parsed_time,
                    sha1: v.sha1.clone(),
                }
            })
            .collect()
    }
}
