use super::installer::{run_client_installer, INSTALLER_TIMEOUT};
use crate::version::VersionDetails;
use amc_core::error::{LauncherError, Result};
use amc_core::paths::LauncherPaths;
use amc_downloader::{DownloadEngine, DownloadItem};
use reqwest::Client;
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::Path;
use tokio::sync::mpsc;

const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
const NEOFORGE_META: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Deserialize)]
struct NeoForgeVersions {
    versions: Vec<String>,
}

pub struct NeoForgeLoader;

impl NeoForgeLoader {
    /// Newest NeoForge for an MC version. Only MC generations NeoForge
    /// actually ships (1.20+) resolve — anything older is an honest error,
    /// never a wrong-version match.
    pub async fn get_latest_version(client: &Client, mc_version: &str) -> Result<String> {
        let res = client
            .get(NEOFORGE_META)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("NeoForge meta error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "NeoForge meta HTTP {}",
                res.status()
            )));
        }

        let data: NeoForgeVersions = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("NeoForge json error: {e}")))?;

        select_neoforge_version(&data.versions, mc_version).ok_or_else(|| {
            LauncherError::Loader(format!("No NeoForge version found for MC {mc_version}"))
        })
    }

    pub fn installer_url(neoforge_version: &str) -> String {
        format!("{NEOFORGE_MAVEN}/net/neoforged/neoforge/{neoforge_version}/neoforge-{neoforge_version}-installer.jar")
    }

    pub fn version_id(mc_version: &str, neoforge_version: &str) -> String {
        format!("{mc_version}-neoforge-{neoforge_version}")
    }

    /// Full one-time setup, mirroring [`ForgeLoader::resolve`]: official
    /// installer run headlessly, produced version json cached. UNMERGED.
    pub async fn resolve(
        client: &Client,
        engine: &DownloadEngine,
        mc_version: &str,
        java_bin: &Path,
        paths: &LauncherPaths,
        status: mpsc::Sender<String>,
    ) -> Result<VersionDetails> {
        let neoforge_version = Self::get_latest_version(client, mc_version).await?;
        let version_id = Self::version_id(mc_version, &neoforge_version);

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
            .send(format!(
                "Downloading NeoForge {neoforge_version} installer..."
            ))
            .await;
        let installer_path = paths
            .cache_dir()
            .join("installers")
            .join(format!("neoforge-{version_id}-installer.jar"));
        if !installer_path.is_file() {
            if let Some(parent) = installer_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| LauncherError::Io {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
            }
            let item = DownloadItem::new(Self::installer_url(&neoforge_version), &installer_path);
            engine
                .download_all_with_progress(vec![item], None)
                .await
                .map_err(|e| {
                    LauncherError::Loader(format!("NeoForge installer download failed: {e}"))
                })?;
        }

        let _ = status
            .send("Running NeoForge installer (one-time setup)...".to_string())
            .await;
        let staging = paths
            .cache_dir()
            .join("installers")
            .join(format!("neoforge-{version_id}-work"));
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

/// NeoForge major for an MC version (`1.20.x` → `20.`, `1.21.x` → `21.`).
/// Generations NeoForge never shipped return `None` — callers error out
/// instead of matching an unrelated version.
fn mc_to_prefix(mc_version: &str) -> Option<&str> {
    // Note: bare "1.21" (no patch) is a real MC version id.
    if mc_version == "1.20" || mc_version.starts_with("1.20.") {
        Some("20.")
    } else if mc_version == "1.21" || mc_version.starts_with("1.21.") {
        Some("21.")
    } else {
        None
    }
}

/// Newest version for an MC generation (unit-tested, no network).
fn select_neoforge_version(versions: &[String], mc_version: &str) -> Option<String> {
    let prefix = mc_to_prefix(mc_version)?;
    versions
        .iter()
        .filter(|v| v.starts_with(prefix))
        .max_by(|a, b| compare_versions(a, b))
        .cloned()
}

/// Numeric-aware version comparison (`21.0.167` > `21.0.9`).
/// A `-suffix` (beta/rc) always sorts below its release, while an extra
/// numeric part (`21.0.167.1`) sorts above.
fn compare_versions(a: &str, b: &str) -> Ordering {
    let pa = split_parts(a);
    let pb = split_parts(b);
    let common = pa.len().min(pb.len());
    for i in 0..common {
        match pa[i].cmp(&pb[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    match (pa.get(common), pb.get(common)) {
        (None, None) => Ordering::Equal,
        (None, Some(Part::Text(_))) => Ordering::Greater, // `a` is the release
        (None, Some(Part::Number(_))) => Ordering::Less,  // `b` has a newer patch
        (Some(Part::Text(_)), None) => Ordering::Less,
        (Some(Part::Number(_)), None) => Ordering::Greater,
        // Unreachable: `common` is the shorter length, so both sides cannot
        // have an element there — but exhaustiveness requires the arm.
        (Some(_), Some(_)) => Ordering::Equal,
    }
}

fn split_parts(v: &str) -> Vec<Part> {
    v.split(['.', '-'])
        .map(|p| {
            p.parse::<u64>()
                .map(Part::Number)
                .unwrap_or_else(|_| Part::Text(p.to_string()))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    Number(u64),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<String> {
        vec![
            "20.4.167".to_string(),
            "20.4.190".to_string(),
            "20.4.9".to_string(),
            "21.0.167-beta".to_string(),
            "21.0.167".to_string(),
            "21.0.9".to_string(),
        ]
    }

    #[test]
    fn test_selects_newest_of_generation() {
        assert_eq!(
            select_neoforge_version(&fixture(), "1.20.1").as_deref(),
            Some("20.4.190")
        );
        // Numeric compare: 167 > 9, and release > beta.
        assert_eq!(
            select_neoforge_version(&fixture(), "1.21").as_deref(),
            Some("21.0.167")
        );
    }

    #[test]
    fn test_unsupported_generation_is_none() {
        assert_eq!(select_neoforge_version(&fixture(), "1.19.2"), None);
        assert_eq!(select_neoforge_version(&fixture(), "1.16.5"), None);
        assert_eq!(select_neoforge_version(&[], "1.20.1"), None);
    }

    #[test]
    fn test_url_and_id_format() {
        assert_eq!(
            NeoForgeLoader::installer_url("21.0.167"),
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/21.0.167/neoforge-21.0.167-installer.jar"
        );
        assert_eq!(
            NeoForgeLoader::version_id("1.21", "21.0.167"),
            "1.21-neoforge-21.0.167"
        );
    }
}
