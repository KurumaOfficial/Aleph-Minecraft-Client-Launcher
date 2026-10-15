use crate::error::{LauncherError, Result};
use std::path::{Path, PathBuf};

/// Write `bytes` atomically: `tmp` next to `path` + `rename`.
///
/// A crash mid-save must never leave a truncated JSON (instances,
/// accounts, favorites, config). `tmp` lives next to `path` so `rename`
/// stays on one filesystem.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, bytes).map_err(|e| LauncherError::Io {
        path: tmp_path.clone(),
        source: e,
    })?;
    std::fs::rename(&tmp_path, path).map_err(|e| LauncherError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

/// Copy a corrupt document to `<name>.corrupt-<nanos>.bak` next to it.
///
/// Best-effort: never fails the caller. Returns the backup path when
/// a backup was created, so a later overwrite cannot silently wipe
/// user data (instances, accounts, favorites).
#[must_use]
pub fn backup_corrupt_file(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let backup_name = format!("{file_name}.corrupt-{nanos}.bak");
    let backup = path.with_file_name(backup_name);
    std::fs::copy(path, &backup).ok()?;
    Some(backup)
}

/// Sanitize a single remote file name (`CurseForge`, `OptiFine`, ...).
///
/// Only the final path component is kept; traversal, separators,
/// drive letters and empty names are rejected.
#[must_use]
pub fn sanitize_remote_filename(raw: &str) -> Option<String> {
    let normalized = raw.replace('\\', "/");
    let name = normalized.rsplit('/').next()?.trim();
    if name.is_empty() || name == "." || name == ".." {
        return None;
    }
    if name.contains('\0') || name.contains(':') {
        return None;
    }
    if name.contains('/') || name.contains("..") {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_atomic_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "amc_atomic_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("doc.json");
        write_atomic(&path, b"{\"a\":1}").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"a\":1}");
        assert!(!path.with_extension("tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_remote_filename() {
        assert_eq!(
            sanitize_remote_filename("sodium-1.0.jar"),
            Some("sodium-1.0.jar".to_string())
        );
        assert_eq!(
            sanitize_remote_filename("../../evil.jar"),
            Some("evil.jar".to_string())
        );
        assert!(sanitize_remote_filename("").is_none());
        assert!(sanitize_remote_filename("..").is_none());
        assert!(sanitize_remote_filename("C:evil.jar").is_none());
        assert!(sanitize_remote_filename("a/b.jar").unwrap() == "b.jar");
    }

    #[test]
    fn test_backup_missing_is_none() {
        let missing = std::env::temp_dir().join("amc_no_such_file_xyz.json");
        assert!(backup_corrupt_file(&missing).is_none());
    }
}
