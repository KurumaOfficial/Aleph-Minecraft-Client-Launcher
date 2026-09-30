use super::installer::{run_client_installer, INSTALLER_TIMEOUT};
use crate::version::VersionDetails;
use amc_core::error::{LauncherError, Result};
use amc_core::paths::LauncherPaths;
use amc_downloader::{DownloadCancel, DownloadEngine, DownloadItem};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::mpsc;

const MAVEN_FORGE: &str = "https://maven.minecraftforge.net";
/// Official Forge promotions: `{mc}-recommended` / `{mc}-latest` pins.
const FORGE_PROMOTIONS: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

#[derive(Debug, Clone, Deserialize)]
struct PromotionsFile {
    #[serde(default)]
    promos: HashMap<String, String>,
}

pub struct ForgeLoader;

impl ForgeLoader {
    /// Newest Forge for an MC version from the official promotions file.
    /// `recommended` wins over `latest`; unknown MC versions are an error.
    pub async fn get_latest_version(client: &Client, mc_version: &str) -> Result<String> {
        let res = client
            .get(FORGE_PROMOTIONS)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Forge promotions error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "Forge promotions HTTP {}",
                res.status()
            )));
        }

        let file: PromotionsFile = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Forge promotions json error: {e}")))?;

        select_forge_version(&file.promos, mc_version).ok_or_else(|| {
            LauncherError::Loader(format!("No Forge release found for MC {mc_version}"))
        })
    }

    pub fn installer_url(mc_version: &str, forge_version: &str) -> String {
        format!("{MAVEN_FORGE}/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar")
    }

    pub fn version_id(mc_version: &str, forge_version: &str) -> String {
        format!("{mc_version}-forge-{forge_version}")
    }

    /// Full one-time setup: download the official installer, run it
    /// headlessly, cache the produced version json. Returns UNMERGED details —
    /// the caller merges the vanilla parent. A cached json makes relaunches
    /// instant (installer versions are immutable).
    pub async fn resolve(
        client: &Client,
        engine: &DownloadEngine,
        mc_version: &str,
        java_bin: &Path,
        paths: &LauncherPaths,
        status: mpsc::Sender<String>,
    ) -> Result<VersionDetails> {
        let forge_version = Self::get_latest_version(client, mc_version).await?;
        let version_id = Self::version_id(mc_version, &forge_version);

        let cached = paths
            .versions_dir()
            .join(&version_id)
            .join(format!("{version_id}.json"));
        if cached.is_file() {
            if let Ok(text) = tokio::fs::read_to_string(&cached).await {
                if let Ok(details) = VersionDetails::parse(&text) {
                    return Ok(details);
                }
            }
        }

        let _ = status
            .send(format!("Downloading Forge {forge_version} installer..."))
            .await;
        let installer_path = paths
            .cache_dir()
            .join("installers")
            .join(format!("forge-{version_id}-installer.jar"));
        if !installer_path.is_file() {
            if let Some(parent) = installer_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| LauncherError::Io {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
            }
            let item = DownloadItem::new(
                Self::installer_url(mc_version, &forge_version),
                &installer_path,
            );
            // Installer phase is not UI-cancellable yet (P3): uncancellable token.
            engine
                .download_all_with_progress(vec![item], None, DownloadCancel::default())
                .await
                .map_err(|e| {
                    LauncherError::Loader(format!("Forge installer download failed: {e}"))
                })?;
        }

        let _ = status
            .send("Running Forge installer (one-time setup)...".to_string())
            .await;
        let staging = paths
            .cache_dir()
            .join("installers")
            .join(format!("forge-{version_id}-work"));
        let produced = run_client_installer(
            java_bin,
            &installer_path,
            &staging,
            Some(&version_id),
            &status,
            INSTALLER_TIMEOUT,
        )
        .await?;

        let text = tokio::fs::read_to_string(&produced)
            .await
            .map_err(|e| LauncherError::Io {
                path: produced.clone(),
                source: e,
            })?;
        let details = VersionDetails::parse(&text)?;

        if let Some(parent) = cached.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let _ = tokio::fs::write(&cached, &text).await;
        let _ = tokio::fs::remove_dir_all(&staging).await; // best-effort cleanup
        Ok(details)
    }
}

/// Pure selection over a promotions map (unit-tested, no network).
fn select_forge_version(promos: &HashMap<String, String>, mc_version: &str) -> Option<String> {
    let key = |kind: &str| format!("{mc_version}-{kind}");
    promos
        .get(&key("recommended"))
        .or_else(|| promos.get(&key("latest")))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> HashMap<String, String> {
        HashMap::from([
            ("1.20.1-recommended".to_string(), "47.2.0".to_string()),
            ("1.20.1-latest".to_string(), "47.2.20".to_string()),
            ("1.19.2-latest".to_string(), "43.2.0".to_string()),
        ])
    }

    #[test]
    fn test_recommended_wins_over_latest() {
        assert_eq!(
            select_forge_version(&fixture(), "1.20.1").as_deref(),
            Some("47.2.0")
        );
    }

    #[test]
    fn test_latest_fallback_and_unknown() {
        assert_eq!(
            select_forge_version(&fixture(), "1.19.2").as_deref(),
            Some("43.2.0")
        );
        assert_eq!(select_forge_version(&fixture(), "1.16.5"), None);
    }

    #[test]
    fn test_url_and_id_format() {
        assert_eq!(
            ForgeLoader::installer_url("1.20.1", "47.2.0"),
            "https://maven.minecraftforge.net/net/minecraftforge/forge/1.20.1-47.2.0/forge-1.20.1-47.2.0-installer.jar"
        );
        assert_eq!(
            ForgeLoader::version_id("1.20.1", "47.2.0"),
            "1.20.1-forge-47.2.0"
        );
    }
}
