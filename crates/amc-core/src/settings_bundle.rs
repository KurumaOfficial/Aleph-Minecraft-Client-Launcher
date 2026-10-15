use crate::error::{LauncherError, Result};
use std::path::Path;

/// Manual settings transfer for moving to another PC (CONCEPT
/// "Облачная синхронизация", P10 — no cloud in v1.0).
/// The bundle is a flat `.zip` with the launcher JSON documents only:
/// no worlds, mods, or runtimes. Account tokens travel inside
/// `accounts.json` by design — the file owner moves it themselves.
pub const BUNDLE_FILES: [&str; 4] = [
    "config.json",
    "accounts.json",
    "instances.json",
    "favorites.json",
];

fn bundle_error(message: String) -> LauncherError {
    LauncherError::Custom(message)
}

/// Pack existing JSON documents from `root` into `dest` (a `.zip` path).
/// Missing files are skipped so partial profiles still transfer.
pub fn export_bundle(root: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::create(dest).map_err(|e| LauncherError::Io {
        path: dest.to_path_buf(),
        source: e,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut packed = 0u32;
    for name in BUNDLE_FILES {
        let src = root.join(name);
        if !src.is_file() {
            continue;
        }
        let bytes = std::fs::read(&src).map_err(|e| LauncherError::Io {
            path: src.clone(),
            source: e,
        })?;
        // Skip corrupt documents rather than transferring breakage.
        if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
            continue;
        }
        zip.start_file(name, opts)
            .map_err(|e| bundle_error(format!("Bundle write error: {e}")))?;
        std::io::Write::write_all(&mut zip, &bytes)
            .map_err(|e| bundle_error(format!("Bundle write error: {e}")))?;
        packed += 1;
    }
    zip.finish()
        .map_err(|e| bundle_error(format!("Bundle finish error: {e}")))?;
    if packed == 0 {
        return Err(bundle_error("No settings files to export".to_string()));
    }
    Ok(())
}

/// Restore a bundle into `root`, returning the restored file names.
/// Only the exact allowlisted names land on disk — subdirectory entries
/// and path traversal are rejected. Every document must be valid JSON.
pub fn import_bundle(root: &Path, pack: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(pack).map_err(|e| LauncherError::Io {
        path: pack.to_path_buf(),
        source: e,
    })?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| bundle_error(format!("Not a bundle zip: {e}")))?;
    if archive.is_empty() {
        return Err(bundle_error("Bundle is empty".to_string()));
    }
    // Stage in memory first: nothing is overwritten until the whole
    // bundle proves to be valid JSON under allowlisted names.
    let mut staged: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| bundle_error(format!("Bundle read error: {e}")))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if !BUNDLE_FILES.contains(&name.as_str()) {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes)
            .map_err(|e| bundle_error(format!("Bundle read error: {e}")))?;
        if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
            return Err(bundle_error(format!(
                "Bundle file is not valid JSON: {name}"
            )));
        }
        staged.push((name, bytes));
    }
    if staged.is_empty() {
        return Err(bundle_error("Bundle has no settings files".to_string()));
    }
    let mut restored = Vec::new();
    for (name, bytes) in staged {
        let dest = root.join(&name);
        crate::fs::write_atomic(&dest, &bytes)?;
        restored.push(name);
    }
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "amc_bundle_test_{tag}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn seed(root: &Path) {
        for name in BUNDLE_FILES {
            std::fs::write(root.join(name), format!("{{\"{name}\": true}}")).unwrap();
        }
    }

    #[test]
    fn test_export_import_roundtrip() {
        let src = temp_root("src");
        let dst = temp_root("dst");
        seed(&src);
        let pack = src.join("bundle.zip");
        export_bundle(&src, &pack).expect("export packs all four files");
        let restored = import_bundle(&dst, &pack).expect("import restores");
        assert_eq!(restored.len(), 4);
        for name in BUNDLE_FILES {
            let a = std::fs::read(src.join(name)).unwrap();
            let b = std::fs::read(dst.join(name)).unwrap();
            assert_eq!(a, b, "{name} survives the roundtrip");
        }
        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }

    #[test]
    fn test_export_skips_missing_and_corrupt() {
        let root = temp_root("partial");
        std::fs::write(root.join("config.json"), r#"{"ok": true}"#).unwrap();
        std::fs::write(root.join("accounts.json"), "not json").unwrap();
        let pack = root.join("bundle.zip");
        export_bundle(&root, &pack).expect("one good file is enough");
        let dst = temp_root("partial_dst");
        let restored = import_bundle(&dst, &pack).unwrap();
        assert_eq!(restored, vec!["config.json".to_string()]);
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&dst);
    }

    #[test]
    fn test_export_empty_root_fails() {
        let root = temp_root("empty");
        let pack = root.join("bundle.zip");
        assert!(export_bundle(&root, &pack).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_import_rejects_traversal_and_non_json() {
        let root = temp_root("evil_src");
        let pack = root.join("evil.zip");
        {
            let file = std::fs::File::create(&pack).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("../../evil.json", opts).unwrap();
            zip.write_all(br#"{"x": 1}"#).unwrap();
            zip.start_file("config.json", opts).unwrap();
            zip.write_all(b"broken").unwrap();
            zip.finish().unwrap();
        }
        let dst = temp_root("evil_dst");
        let err = import_bundle(&dst, &pack).expect_err("invalid JSON aborts");
        assert!(err.to_string().contains("config.json"));
        assert!(!dst.join("config.json").is_file());
        assert!(!dst.join("evil.json").is_file());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&dst);
    }
}
