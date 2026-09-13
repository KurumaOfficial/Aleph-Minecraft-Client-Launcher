use serde::Deserialize;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;
use amc_core::error::{LauncherError, Result};
use crate::types::LocalMod;

#[derive(Debug, Deserialize)]
struct FabricModMetadata {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McModInfoEntry {
    modid: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

pub struct LocalModManager;

impl LocalModManager {
    pub fn scan_mods(mods_dir: &Path) -> Result<Vec<LocalMod>> {
        if !mods_dir.is_dir() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(mods_dir).map_err(|e| LauncherError::Io {
            path: mods_dir.to_path_buf(),
            source: e,
        })?;

        let mut mods = Vec::new();

        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            let filename = match path.file_name().and_then(|f| f.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let is_jar = filename.ends_with(".jar");
            let is_disabled = filename.ends_with(".jar.disabled");

            if !is_jar && !is_disabled {
                continue;
            }

            let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let parsed_info = Self::read_jar_metadata(&path);

            let (name, id, version, description) = match parsed_info {
                Some((n, i, v, d)) => (n, i, v, d),
                None => {
                    let fallback_name = filename
                        .trim_end_matches(".disabled")
                        .trim_end_matches(".jar")
                        .to_string();
                    (fallback_name.clone(), fallback_name, "unknown".into(), String::new())
                }
            };

            mods.push(LocalMod {
                filename,
                path,
                name,
                id,
                version,
                description,
                enabled: is_jar,
                file_size,
            });
        }

        mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(mods)
    }

    fn read_jar_metadata(path: &Path) -> Option<(String, String, String, String)> {
        let file = File::open(path).ok()?;
        let mut archive = ZipArchive::new(file).ok()?;

        // 1. Try fabric.mod.json
        if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() {
                if let Ok(meta) = serde_json::from_str::<FabricModMetadata>(&content) {
                    let name = meta.name.unwrap_or_else(|| meta.id.clone());
                    let ver = meta.version.unwrap_or_else(|| "1.0.0".into());
                    let desc = meta.description.unwrap_or_default();
                    return Some((name, meta.id, ver, desc));
                }
            }
        }

        // 2. Try META-INF/mods.toml (Forge / NeoForge)
        if let Ok(mut entry) = archive.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() {
                // Simple regex-free key extraction
                let mut mod_id = String::new();
                let mut display_name = String::new();
                let mut version = String::new();
                let mut description = String::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("modId=") {
                        mod_id = line.trim_start_matches("modId=").trim_matches('"').trim().to_string();
                    } else if line.starts_with("displayName=") {
                        display_name = line.trim_start_matches("displayName=").trim_matches('"').trim().to_string();
                    } else if line.starts_with("version=") {
                        version = line.trim_start_matches("version=").trim_matches('"').trim().to_string();
                    } else if line.starts_with("description=") {
                        description = line.trim_start_matches("description=").trim_matches('"').trim().to_string();
                    }
                }

                if !mod_id.is_empty() {
                    let name = if display_name.is_empty() { mod_id.clone() } else { display_name };
                    let ver = if version.is_empty() { "1.0.0".into() } else { version };
                    return Some((name, mod_id, ver, description));
                }
            }
        }

        // 3. Try mcmod.info (Legacy Forge)
        if let Ok(mut entry) = archive.by_name("mcmod.info") {
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() {
                if let Ok(list) = serde_json::from_str::<Vec<McModInfoEntry>>(&content) {
                    if let Some(m) = list.first() {
                        let name = m.name.clone().unwrap_or_else(|| m.modid.clone());
                        let ver = m.version.clone().unwrap_or_else(|| "1.0.0".into());
                        let desc = m.description.clone().unwrap_or_default();
                        return Some((name, m.modid.clone(), ver, desc));
                    }
                }
            }
        }

        None
    }

    pub fn toggle_mod(path: &Path) -> Result<PathBuf> {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| LauncherError::Custom("Invalid mod filename".into()))?;

        let parent = path.parent().unwrap_or(Path::new("."));

        let new_path = if filename.ends_with(".jar") {
            parent.join(format!("{filename}.disabled"))
        } else if filename.ends_with(".jar.disabled") {
            parent.join(filename.trim_end_matches(".disabled"))
        } else {
            return Err(LauncherError::Custom("File is not a recognized mod".into()));
        };

        fs::rename(path, &new_path).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        Ok(new_path)
    }

    pub fn delete_mod(path: &Path) -> Result<()> {
        fs::remove_file(path).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    pub fn import_mod(source: &Path, mods_dir: &Path) -> Result<PathBuf> {
        let filename = source
            .file_name()
            .ok_or_else(|| LauncherError::Custom("Invalid source file".into()))?;

        fs::create_dir_all(mods_dir).map_err(|e| LauncherError::Io {
            path: mods_dir.to_path_buf(),
            source: e,
        })?;

        let destination = mods_dir.join(filename);
        fs::copy(source, &destination).map_err(|e| LauncherError::Io {
            path: destination.clone(),
            source: e,
        })?;

        Ok(destination)
    }
}
