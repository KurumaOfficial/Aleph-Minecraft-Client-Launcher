use super::installer::{
    find_new_version_json, run_client_installer, snapshot_versions, INSTALLER_TIMEOUT,
};
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
        // Start from a clean staging dir so the product diff below is
        // exact (a previous failed run must not look like output).
        let _ = tokio::fs::remove_dir_all(&staging).await;
        // Like NeoForge, the installer downloads vanilla files next to
        // its own output: find the product by directory diff.
        let before = snapshot_versions(&staging.join("versions"));
        let produced = run_client_installer(
            java_bin,
            &installer_path,
            &staging,
            Some(&version_id),
            &status,
            INSTALLER_TIMEOUT,
        )
        .await
        .or_else(|_| {
            find_new_version_json(&staging.join("versions"), &before, Some(&version_id)).map_err(
                |e| LauncherError::Launch(format!("Forge installer product not found: {e}")),
            )
        })?;

        let text = tokio::fs::read_to_string(&produced)
            .await
            .map_err(|e| LauncherError::Io {
                path: produced.clone(),
                source: e,
            })?;
        let details = VersionDetails::parse(&text)?;

        // Cache under the produced id (the truth after any format
        // drift), so relaunches hit the fast path above.
        let cache_dir = paths.versions_dir().join(&details.id);
        if tokio::fs::create_dir_all(&cache_dir).await.is_ok() {
            let _ = tokio::fs::write(cache_dir.join(format!("{}.json", details.id)), &text).await;
        }
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

/// Whether a Forge line is old enough to deserve an unsafe-loader warning
/// (CONCEPT "Загрузчики модов"). The 1.12-and-older Forge line ended in 2018
/// and ships a pre-2.17 log4j — warn, never block. Pure, unit-tested.
pub fn is_legacy_unsafe_line(mc_version: &str) -> bool {
    mc_version.starts_with("1.7.")
        || mc_version.starts_with("1.8.")
        || mc_version.starts_with("1.9.")
        || mc_version.starts_with("1.10.")
        || mc_version.starts_with("1.11.")
        || mc_version.starts_with("1.12.")
        || mc_version == "1.7"
        || mc_version == "1.8"
        || mc_version == "1.9"
        || mc_version == "1.10"
        || mc_version == "1.11"
        || mc_version == "1.12"
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
    fn test_legacy_unsafe_line() {
        for mc in ["1.7.10", "1.8.9", "1.12.2", "1.12", "1.7"] {
            assert!(is_legacy_unsafe_line(mc), "{mc} should warn");
        }
        for mc in ["1.13", "1.16.5", "1.19.2", "1.20.1", "1.21", ""] {
            assert!(!is_legacy_unsafe_line(mc), "{mc} should not warn");
        }
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

    /// LIVE end-to-end verification (manual only — needs network and a
    /// Java 17+ binary; never runs in CI):
    /// `cargo test -p amc-minecraft -- --ignored live_forge_installer`
    /// Downloads the real installer, runs it headlessly into a temp
    /// root, and asserts a parseable version json with libraries and a
    /// main class comes out.
    #[tokio::test]
    #[ignore]
    async fn live_forge_installer() {
        let java_bin = find_java_17_plus();
        let tmp = std::env::temp_dir().join(format!(
            "amc_live_forge_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let paths = LauncherPaths::custom(&tmp).expect("temp launcher root");
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("reqwest client");
        let engine = DownloadEngine::new(4);
        let (status_tx, mut status_rx) = mpsc::channel::<String>(64);
        tokio::spawn(async move {
            while let Some(line) = status_rx.recv().await {
                eprintln!("[installer] {line}");
            }
        });
        let details =
            ForgeLoader::resolve(&client, &engine, "1.20.1", &java_bin, &paths, status_tx)
                .await
                .expect("Forge 1.20.1 resolves end to end");
        assert!(
            details.id.contains("forge"),
            "unexpected version id: {}",
            details.id
        );
        assert!(
            !details.libraries.is_empty(),
            "produced version has no libraries"
        );
        assert!(
            !details.main_class.is_empty(),
            "produced version has no main class"
        );
        let cached = paths
            .versions_dir()
            .join(&details.id)
            .join(format!("{}.json", details.id));
        assert!(cached.is_file(), "resolved json is cached for relaunches");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// System Java for the live test: `JAVA_HOME` first, else PATH.
    /// Forge 1.20.1 needs Java 17+.
    fn find_java_17_plus() -> std::path::PathBuf {
        let candidate = std::env::var_os("JAVA_HOME")
            .map(|home| {
                let mut p = std::path::PathBuf::from(home);
                p.push("bin");
                #[cfg(target_os = "windows")]
                p.push("java.exe");
                #[cfg(not(target_os = "windows"))]
                p.push("java");
                p
            })
            .filter(|p| p.is_file())
            .unwrap_or_else(|| std::path::PathBuf::from("java"));
        let probe = std::process::Command::new(&candidate)
            .arg("-version")
            .output()
            .expect("a Java binary is required for the live installer test");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&probe.stdout),
            String::from_utf8_lossy(&probe.stderr)
        );
        let major: u32 = text
            .lines()
            .find_map(|line| {
                let q1 = line.find('"')? + 1;
                let rest = &line[q1..];
                let end = rest.find(['"', ' ', '.'])?;
                rest[..end].parse::<u32>().ok()
            })
            .unwrap_or(0);
        let minor: u32 = text
            .lines()
            .find_map(|line| {
                let q1 = line.find('"')? + 1;
                let rest = &line[q1..];
                let mut parts = rest.split(['"', ' ', '.', '_']);
                let _one: u32 = parts.next()?.parse().ok()?;
                parts.next()?.parse().ok()
            })
            .unwrap_or(0);
        let effective = if major == 1 { minor } else { major };
        assert!(
            effective >= 17,
            "live Forge 1.20.1 test needs Java 17+, got: {}",
            text.lines().next().unwrap_or("?")
        );
        candidate
    }
}
