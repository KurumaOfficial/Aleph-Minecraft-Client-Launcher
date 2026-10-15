use super::installer::{
    find_new_version_json, run_client_installer, snapshot_versions, INSTALLER_TIMEOUT,
};
use crate::version::VersionDetails;
use amc_core::error::{LauncherError, Result};
use amc_core::paths::LauncherPaths;
use amc_downloader::{DownloadCancel, DownloadEngine, DownloadItem};
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

        let versions: NeoForgeVersions = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("NeoForge json error: {e}")))?;

        select_neoforge_version(&versions.versions, mc_version).ok_or_else(|| {
            LauncherError::Loader(format!("No NeoForge version found for MC {mc_version}"))
        })
    }

    pub fn installer_url(neoforge_version: &str) -> String {
        format!("{NEOFORGE_MAVEN}/net/neoforged/neoforge/{neoforge_version}/neoforge-{neoforge_version}-installer.jar")
    }

    pub fn version_id(mc_version: &str, neoforge_version: &str) -> String {
        format!("{mc_version}-neoforge-{neoforge_version}")
    }

    /// Full one-time setup, mirroring [`crate::loaders::ForgeLoader::resolve`]: official
    /// installer run headlessly, produced version json cached. Returns
    /// UNMERGED details plus the resolved NeoForge version for pinning
    /// (`None` on a cache hit or a caller-supplied pin). A stored pin
    /// wins verbatim (reproducible, offline-friendly).
    pub async fn resolve(
        client: &Client,
        engine: &DownloadEngine,
        mc_version: &str,
        java_bin: &Path,
        paths: &LauncherPaths,
        status: mpsc::Sender<String>,
        pinned: Option<String>,
    ) -> Result<(VersionDetails, Option<String>)> {
        let neoforge_version = match pinned.filter(|v| !v.trim().is_empty()) {
            Some(v) => v,
            None => Self::get_latest_version(client, mc_version).await?,
        };
        let version_id = Self::version_id(mc_version, &neoforge_version);
        // Id format drifted across NeoForge lines (`{mc}-neoforge-{v}`
        // vs `neoforge-{v}`): probe every known shape before downloading.
        let mut id_candidates = vec![version_id.clone()];
        let drifted = format!("neoforge-{neoforge_version}");
        if drifted != version_id {
            id_candidates.push(drifted);
        }
        for candidate in &id_candidates {
            let cached = paths
                .versions_dir()
                .join(candidate)
                .join(format!("{candidate}.json"));
            if cached.is_file() {
                if let Ok(text) = tokio::fs::read_to_string(&cached).await {
                    if let Ok(details) = VersionDetails::parse(&text) {
                        return Ok((details, None));
                    }
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
            // Installer phase is not UI-cancellable yet (P3): uncancellable token.
            engine
                .download_all_with_progress(vec![item], None, DownloadCancel::default())
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
        // Start from a clean staging dir so the product diff below is
        // exact (a previous failed run must not look like output).
        let _ = tokio::fs::remove_dir_all(&staging).await;
        // The installer downloads vanilla files next to its own output,
        // so the product is found by directory diff, not by counting.
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
            // Exact id missed (format drift): diff-based fallback.
            find_new_version_json(&staging.join("versions"), &before, Some(&version_id)).map_err(
                |e| LauncherError::Launch(format!("NeoForge installer product not found: {e}")),
            )
        })?;

        let text = tokio::fs::read_to_string(&produced)
            .await
            .map_err(|e| LauncherError::Io {
                path: produced.clone(),
                source: e,
            })?;
        let details = VersionDetails::parse(&text)?;

        // Cache under the produced id (the truth after format drift),
        // so relaunches hit the fast path above.
        let cache_dir = paths.versions_dir().join(&details.id);
        if tokio::fs::create_dir_all(&cache_dir).await.is_ok() {
            let _ = tokio::fs::write(cache_dir.join(format!("{}.json", details.id)), &text).await;
        }
        let _ = tokio::fs::remove_dir_all(&staging).await; // best-effort cleanup
        Ok((details, Some(neoforge_version)))
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

    /// LIVE end-to-end verification (manual only — needs network and a
    /// Java 21+ binary; never runs in CI):
    /// `cargo test -p amc-minecraft -- --ignored live_neoforge_installer`
    /// Downloads the real installer, runs it headlessly into a temp
    /// root, and asserts a parseable version json with libraries and a
    /// main class comes out.
    #[tokio::test]
    #[ignore]
    async fn live_neoforge_installer() {
        let java_bin = find_java_21_plus();
        let temp_root = std::env::temp_dir().join(format!(
            "amc_live_neoforge_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&temp_root);
        let paths = LauncherPaths::custom(&temp_root).expect("temp launcher root");
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
        let (details, persist_ver) = NeoForgeLoader::resolve(
            &client, &engine, "1.21.4", &java_bin, &paths, status_tx, None,
        )
        .await
        .expect("NeoForge 1.21.4 resolves end to end");
        assert!(
            persist_ver.as_deref().is_some_and(|v| v.starts_with("21.")),
            "fresh resolve reports its version for pinning"
        );
        assert!(
            details.id.contains("neoforge"),
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
        let _ = std::fs::remove_dir_all(&temp_root);
    }

    /// System Java for the live test: `JAVA_HOME` first, else PATH.
    /// Asserts Java 21+ by parsing `java -version` output.
    fn find_java_21_plus() -> std::path::PathBuf {
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
        // Legacy "1.8.0_xx" scheme reports major 1 with minor 8.
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
            effective >= 21,
            "live NeoForge 1.21 test needs Java 21+, got: {}",
            text.lines().next().unwrap_or("?")
        );
        candidate
    }
}
