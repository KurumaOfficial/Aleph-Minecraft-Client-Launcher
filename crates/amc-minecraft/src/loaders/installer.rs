use amc_core::error::{LauncherError, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// How long a loader installer may run. Processors are slow on HDDs and the
/// installer downloads its own files — 10 minutes is generous but finite.
pub const INSTALLER_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Run an official loader installer headlessly:
///
/// `java -jar <installer.jar> --installClient <target_dir>`
///
/// Installer stdout/stderr are streamed to `status` (visible in the download
/// overlay). Returns the produced `<version>.json`. Any genuine failure —
/// spawn error, timeout, non-zero exit, missing json — is an error.
/// A silent fallback to vanilla is forbidden by design.
pub async fn run_client_installer(
    java_bin: &Path,
    installer_jar: &Path,
    target_dir: &Path,
    expected_version_id: Option<&str>,
    status: &mpsc::Sender<String>,
    timeout: Duration,
) -> Result<PathBuf> {
    use std::process::Stdio;

    tokio::fs::create_dir_all(target_dir)
        .await
        .map_err(|e| LauncherError::Io {
            path: target_dir.to_path_buf(),
            source: e,
        })?;

    // NeoForge (and some Forge lines) refuse an empty target:
    // "no minecraft launcher profile ... run the launcher first".
    // Seed a minimal profile so headless installs proceed; the
    // installer fills in its own entries.
    let profiles = target_dir.join("launcher_profiles.json");
    if !profiles.is_file() {
        let seed = r#"{"profiles": {}, "selectedProfile": "(Default)", "authenticationDatabase": {}, "launcherVersion": {}}"#;
        tokio::fs::write(&profiles, seed)
            .await
            .map_err(|e| LauncherError::Io {
                path: profiles.clone(),
                source: e,
            })?;
    }

    let mut child = tokio::process::Command::new(java_bin)
        .arg("-jar")
        .arg(installer_jar)
        .arg("--installClient")
        .arg(target_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| LauncherError::Launch(format!("Failed to start loader installer: {e}")))?;

    let tail = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let mut readers = Vec::new();
    if let Some(out) = child.stdout.take() {
        readers.push(tokio::spawn(stream_pipe(out, status.clone(), tail.clone())));
    }
    if let Some(err) = child.stderr.take() {
        readers.push(tokio::spawn(stream_pipe(err, status.clone(), tail.clone())));
    }

    let wait_result = tokio::time::timeout(timeout, child.wait()).await;
    for reader in readers {
        let _ = reader.await;
    }

    let exit = match wait_result {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            return Err(LauncherError::Launch(format!(
                "Loader installer wait failed: {e}\n{}",
                tail_text(&tail)
            )));
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err(LauncherError::Launch(format!(
                "Loader installer timed out after {}s\n{}",
                timeout.as_secs(),
                tail_text(&tail)
            )));
        }
    };

    if !exit.success() {
        return Err(LauncherError::Launch(format!(
            "Loader installer exited with {exit}\n{}",
            tail_text(&tail)
        )));
    }

    find_installed_json(&target_dir.join("versions"), expected_version_id)
}

fn tail_text(tail: &std::sync::Arc<std::sync::Mutex<Vec<String>>>) -> String {
    tail.lock()
        .map(|lines| {
            let start = lines.len().saturating_sub(20);
            lines[start..].join("\n")
        })
        .unwrap_or_default()
}

async fn stream_pipe<T>(
    pipe: T,
    status: mpsc::Sender<String>,
    tail: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) where
    T: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(pipe).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if let Ok(mut guard) = tail.lock() {
            guard.push(line.clone());
        }
        let _ = status.send(line).await;
    }
}

/// Locate the version json an installer produced under `<target>/versions`.
/// With an expected id the exact file must exist; otherwise the directory
/// must contain exactly one version (anything else is ambiguous → error).
pub fn find_installed_json(
    versions_dir: &Path,
    expected_version_id: Option<&str>,
) -> Result<PathBuf> {
    if let Some(id) = expected_version_id {
        let exact = versions_dir.join(id).join(format!("{id}.json"));
        if exact.is_file() {
            return Ok(exact);
        }
    }

    let mut found = Vec::new();
    let entries = std::fs::read_dir(versions_dir).map_err(|e| LauncherError::Io {
        path: versions_dir.to_path_buf(),
        source: e,
    })?;
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let candidate = dir.join(format!("{name}.json"));
        if candidate.is_file() {
            found.push(candidate);
        }
    }

    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(LauncherError::Launch(format!(
            "Installer produced no version json under {}",
            versions_dir.display()
        ))),
        _ => Err(LauncherError::Launch(format!(
            "Installer produced {} version jsons, ambiguous without an expected id",
            found.len()
        ))),
    }
}

/// Snapshot of version directory names, taken before an installer runs.
/// The installer also downloads vanilla files next to its own output,
/// so "exactly one version" is unreliable — diffing is not.
pub fn snapshot_versions(versions_dir: &Path) -> std::collections::HashSet<String> {
    std::fs::read_dir(versions_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Locate the version json an installer produced, robust to id format
/// drift (NeoForge emits `neoforge-<v>`, older lines `{mc}-neoforge-<v>`;
/// vanilla files land next to the product):
/// exact expected id first, else the single NEW directory whose json
/// declares `inheritsFrom` (loader products always do, vanilla never).
pub fn find_new_version_json(
    versions_dir: &Path,
    before: &std::collections::HashSet<String>,
    expected_version_id: Option<&str>,
) -> Result<PathBuf> {
    if let Some(id) = expected_version_id {
        let exact = versions_dir.join(id).join(format!("{id}.json"));
        if exact.is_file() {
            return Ok(exact);
        }
    }

    let fresh: Vec<String> = std::fs::read_dir(versions_dir)
        .map_err(|e| LauncherError::Io {
            path: versions_dir.to_path_buf(),
            source: e,
        })?
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| !before.contains(name))
        .collect();
    let mut loader_made = Vec::new();
    for name in &fresh {
        let candidate = versions_dir.join(name).join(format!("{name}.json"));
        let inherits = std::fs::read_to_string(&candidate)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|json| {
                json.get("inheritsFrom")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            });
        if inherits.is_some() {
            loader_made.push(candidate);
        }
    }
    match loader_made.len() {
        1 => Ok(loader_made.remove(0)),
        _ => Err(LauncherError::Launch(format!(
            "Installer produced {} loader versions (new dirs: {}), expected exactly one",
            loader_made.len(),
            fresh.join(", ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_dir(suite: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "amc_installer_test_{}_{}_{}",
            suite,
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn write_json(dir: &Path, version_id: &str) {
        write_json_body(dir, version_id, r#"{"id":"x"}"#);
    }

    fn write_json_body(dir: &Path, version_id: &str, body: &str) {
        let version_dir = dir.join(version_id);
        std::fs::create_dir_all(&version_dir).unwrap();
        std::fs::write(version_dir.join(format!("{version_id}.json")), body).unwrap();
    }

    #[test]
    fn test_find_exact_id() {
        let root = unique_dir("exact");
        let versions = root.join("versions");
        write_json(&versions, "1.20.1-forge-47.2.0");
        write_json(&versions, "1.19.2-forge-43.2.0");
        let found = find_installed_json(&versions, Some("1.20.1-forge-47.2.0")).unwrap();
        assert!(found.ends_with(Path::new("1.20.1-forge-47.2.0/1.20.1-forge-47.2.0.json")));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_find_single_fallback() {
        let root = unique_dir("single");
        let versions = root.join("versions");
        write_json(&versions, "whatever");
        let found = find_installed_json(&versions, None).unwrap();
        assert!(found.ends_with(Path::new("whatever/whatever.json")));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_find_new_version_json_by_diff() {
        use std::collections::HashSet;
        let root = unique_dir("diff");
        let versions = root.join("versions");
        // Vanilla downloaded next to the product must not confuse it:
        // only loader products declare inheritsFrom.
        write_json(&versions, "1.21.11");
        let before = snapshot_versions(&versions);
        // NeoForge drifted id format: no MC prefix.
        write_json_body(
            &versions,
            "neoforge-21.11.45",
            r#"{"id":"neoforge-21.11.45","inheritsFrom":"1.21.11"}"#,
        );
        let found =
            find_new_version_json(&versions, &before, Some("1.21.4-neoforge-21.11.45")).unwrap();
        assert!(found.ends_with(Path::new("neoforge-21.11.45/neoforge-21.11.45.json")));
        // Exact expected id still wins when present.
        write_json(&versions, "1.21.4-neoforge-21.11.45");
        let found =
            find_new_version_json(&versions, &before, Some("1.21.4-neoforge-21.11.45")).unwrap();
        assert!(found.ends_with(Path::new(
            "1.21.4-neoforge-21.11.45/1.21.4-neoforge-21.11.45.json"
        )));
        // Nothing new, nothing exact: error, not a guess.
        let empty_before: HashSet<String> = ["1.21.11".to_string()].into_iter().collect();
        let root2 = unique_dir("diff_none");
        let versions2 = root2.join("versions");
        write_json(&versions2, "1.21.11");
        assert!(find_new_version_json(&versions2, &empty_before, Some("nope")).is_err());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&root2);
    }

    #[test]
    fn test_find_none_and_ambiguous_fail() {
        let root = unique_dir("fail");
        let versions = root.join("versions");
        std::fs::create_dir_all(&versions).unwrap();
        assert!(find_installed_json(&versions, None).is_err());
        write_json(&versions, "a");
        write_json(&versions, "b");
        assert!(find_installed_json(&versions, None).is_err());
        // ...but an expected id still resolves when present.
        assert!(find_installed_json(&versions, Some("a")).is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }
}
