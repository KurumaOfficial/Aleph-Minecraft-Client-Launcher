use amc_core::error::{LauncherError, Result};
use serde::Deserialize;

/// CurseForge `.zip` pack import (CONCEPT "Импорт/экспорт сборок", P4).
/// Only the standard shape is supported: `manifest.json`
/// (`manifestType: "minecraftModpack"`) with a `files[]` list of
/// `(projectID, fileID)` pairs plus an `overrides/` tree.
/// File URLs resolve through `CurseForgeClient::get_pack_file` in the app
/// task; this module owns the parsing and layout rules, fully unit-tested
/// with fixture zips.
#[derive(Debug, Clone, Deserialize)]
pub struct CurseModLoader {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurseMinecraft {
    #[serde(default)]
    pub version: String,
    #[serde(rename = "modLoaders", default)]
    pub mod_loaders: Vec<CurseModLoader>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursePackFile {
    #[serde(rename = "projectID")]
    pub project_id: u64,
    #[serde(rename = "fileID")]
    pub file_id: u64,
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurseManifest {
    #[serde(rename = "manifestType", default)]
    pub manifest_type: String,
    #[serde(rename = "manifestVersion", default)]
    pub manifest_version: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub overrides: String,
    #[serde(default)]
    pub minecraft: Option<CurseMinecraft>,
    #[serde(default)]
    pub files: Vec<CursePackFile>,
}

impl CurseManifest {
    /// Overrides folder name, defaulting to the spec standard.
    pub fn overrides_dir(&self) -> &str {
        if self.overrides.is_empty() {
            "overrides"
        } else {
            &self.overrides
        }
    }
}

/// Read `manifest.json` from a CurseForge pack `.zip`.
pub fn parse_curse_manifest(pack_path: &std::path::Path) -> Result<CurseManifest> {
    let file = std::fs::File::open(pack_path).map_err(|e| LauncherError::Io {
        path: pack_path.to_path_buf(),
        source: e,
    })?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| LauncherError::Custom(format!("Not a readable pack zip: {e}")))?;
    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|_| LauncherError::Custom("manifest.json not found in pack".to_string()))?;
    let mut text = String::new();
    std::io::Read::read_to_string(&mut manifest_file, &mut text).map_err(|e| {
        LauncherError::Io {
            path: pack_path.to_path_buf(),
            source: e,
        }
    })?;
    let manifest: CurseManifest = serde_json::from_str(&text).map_err(LauncherError::Json)?;
    if manifest.manifest_type != "minecraftModpack" {
        return Err(LauncherError::Custom(format!(
            "Not a CurseForge modpack (manifestType: {})",
            manifest.manifest_type
        )));
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(dir: &std::path::Path, manifest_json: &str, with_override: bool) {
        let pack = dir.join("pack.zip");
        let file = std::fs::File::create(&pack).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("manifest.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(manifest_json.as_bytes()).unwrap();
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
            "amc_cursepack_test_{tag}_{}_{}",
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

    const MANIFEST: &str = r#"{
        "manifestType": "minecraftModpack",
        "manifestVersion": 1,
        "version": "2.0",
        "name": "Test Pack",
        "overrides": "overrides",
        "minecraft": {
            "version": "1.20.1",
            "modLoaders": [{"id": "forge-47.2.0", "primary": true}]
        },
        "files": [
            {"projectID": 123, "fileID": 456, "required": true},
            {"projectID": 789, "fileID": 101, "required": false}
        ]
    }"#;

    #[test]
    fn test_parse_manifest_and_files() {
        let dir = temp_dir("ok");
        write_fixture(&dir, MANIFEST, true);
        let manifest = parse_curse_manifest(&dir.join("pack.zip")).unwrap();
        assert_eq!(manifest.name, "Test Pack");
        assert_eq!(manifest.overrides_dir(), "overrides");
        let mc = manifest.minecraft.expect("minecraft section parses");
        assert_eq!(mc.version, "1.20.1");
        assert_eq!(mc.mod_loaders.len(), 1);
        assert!(mc.mod_loaders[0].primary);
        assert_eq!(manifest.files.len(), 2);
        assert_eq!(manifest.files[0].project_id, 123);
        assert_eq!(manifest.files[0].file_id, 456);
        assert!(manifest.files[0].required);
        assert!(!manifest.files[1].required);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rejects_wrong_type_and_missing_manifest() {
        let dir = temp_dir("bad");
        write_fixture(
            &dir,
            r#"{"manifestType": "other", "manifestVersion": 1}"#,
            false,
        );
        assert!(parse_curse_manifest(&dir.join("pack.zip")).is_err());
        std::fs::write(dir.join("pack.zip"), b"not a zip").unwrap();
        assert!(parse_curse_manifest(&dir.join("pack.zip")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_overrides_dir_defaults() {
        let manifest: CurseManifest =
            serde_json::from_str(r#"{"manifestType": "minecraftModpack"}"#).unwrap();
        assert_eq!(manifest.overrides_dir(), "overrides");
        assert!(manifest.minecraft.is_none());
        assert!(manifest.files.is_empty());
    }
}
