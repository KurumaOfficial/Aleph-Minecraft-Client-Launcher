use amc_core::error::{LauncherError, Result};
use serde::Deserialize;

/// Modrinth `.mrpack` import (CONCEPT "Импорт/экспорт сборок", P4).
/// Only the widely-used shape is supported: `modrinth.index.json` with a
/// `files[]` list (path + hashes + downloads) plus an `overrides/` tree.
/// Downloading and extraction happen in the app task; this module owns the
/// parsing and layout rules, fully unit-tested with fixture zips.
#[derive(Debug, Clone, Deserialize)]
pub struct MrpackHashes {
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub sha512: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    #[serde(default)]
    pub hashes: Option<MrpackHashes>,
    #[serde(default)]
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize", default)]
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MrpackIndex {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    #[serde(default)]
    pub game: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub files: Vec<MrpackFile>,
}

impl MrpackIndex {
    /// Reject path traversal (`../`, absolute paths) in entry paths.
    pub fn sanitize_path(raw: &str) -> Option<String> {
        let normalized = raw.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').collect();
        let non_empty = parts.iter().filter(|p| !p.is_empty()).count();
        let cleaned: Vec<&str> = parts
            .into_iter()
            .filter(|part| !part.is_empty() && *part != "." && *part != "..")
            .collect();
        if cleaned.is_empty() || cleaned.len() != non_empty {
            return None;
        }
        // Reject absolute paths and Windows drive letters.
        if raw.starts_with('/') || cleaned[0].ends_with(':') {
            return None;
        }
        Some(cleaned.join("/"))
    }
}

/// Read `modrinth.index.json` from a `.mrpack` (zip) file.
pub fn parse_mrpack_index(pack_path: &std::path::Path) -> Result<MrpackIndex> {
    let file = std::fs::File::open(pack_path).map_err(|e| LauncherError::Io {
        path: pack_path.to_path_buf(),
        source: e,
    })?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| LauncherError::Custom(format!("Not a readable .mrpack zip: {e}")))?;
    let mut index_file = archive
        .by_name("modrinth.index.json")
        .map_err(|_| LauncherError::Custom("modrinth.index.json not found in pack".to_string()))?;
    let mut text = String::new();
    std::io::Read::read_to_string(&mut index_file, &mut text).map_err(|e| LauncherError::Io {
        path: pack_path.to_path_buf(),
        source: e,
    })?;
    let index: MrpackIndex = serde_json::from_str(&text).map_err(LauncherError::Json)?;
    if index.format_version != 1 {
        return Err(LauncherError::Custom(format!(
            "Unsupported .mrpack format version {}",
            index.format_version
        )));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(dir: &std::path::Path, index_json: &str, with_override: bool) {
        let pack = dir.join("pack.mrpack");
        let file = std::fs::File::create(&pack).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(
            "modrinth.index.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(index_json.as_bytes()).unwrap();
        if with_override {
            zip.start_file(
                "overrides/config/example.txt",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(b"example=true").unwrap();
        }
        zip.finish().unwrap();
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "amc_mrpack_test_{tag}_{}_{}",
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

    const INDEX: &str = r#"{
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "1.0",
        "name": "Test Pack",
        "files": [
            {"path": "mods/sodium.jar",
             "hashes": {"sha1": "abc", "sha512": "def"},
             "downloads": ["https://example.com/sodium.jar"],
             "fileSize": 123}
        ]
    }"#;

    #[test]
    fn test_parse_index_and_files() {
        let dir = temp_dir("ok");
        write_fixture(&dir, INDEX, true);
        let index = parse_mrpack_index(&dir.join("pack.mrpack")).unwrap();
        assert_eq!(index.format_version, 1);
        assert_eq!(index.files.len(), 1);
        assert_eq!(index.files[0].path, "mods/sodium.jar");
        assert_eq!(
            index.files[0].downloads,
            vec!["https://example.com/sodium.jar".to_string()]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rejects_bad_format_and_missing_index() {
        let dir = temp_dir("bad");
        write_fixture(&dir, r#"{"formatVersion": 99, "files": []}"#, false);
        assert!(parse_mrpack_index(&dir.join("pack.mrpack")).is_err());
        std::fs::write(dir.join("pack.mrpack"), b"not a zip").unwrap();
        assert!(parse_mrpack_index(&dir.join("pack.mrpack")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_path() {
        assert_eq!(
            MrpackIndex::sanitize_path("mods/sodium.jar").as_deref(),
            Some("mods/sodium.jar")
        );
        assert_eq!(MrpackIndex::sanitize_path("../evil.jar"), None);
        assert_eq!(MrpackIndex::sanitize_path("mods/../../evil.jar"), None);
        assert_eq!(MrpackIndex::sanitize_path("/abs/path.jar"), None);
        assert_eq!(MrpackIndex::sanitize_path("C:/win/path.jar"), None);
        assert_eq!(MrpackIndex::sanitize_path(""), None);
    }
}
