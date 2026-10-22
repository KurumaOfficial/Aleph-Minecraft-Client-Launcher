//! Saved skin library.
//!
//! Skins are copied into `<root>/skins/` and indexed by `<root>/skins.json`.
//! The format is additive: a missing or corrupt index starts an empty library
//! and never blocks the launcher. Files are addressed by generated UUIDs, so
//! user-supplied names cannot escape the skin directory.

use crate::error::{LauncherError, Result};
use crate::fs::write_atomic;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// One locally saved skin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSkin {
    /// Stable identifier; also names the PNG on disk.
    pub id: Uuid,
    /// Display name shown in the library.
    pub name: String,
    /// File name inside the `skins/` directory (never a path).
    pub file: String,
    /// Slim (3 px arms) model flag.
    #[serde(default)]
    pub slim: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SkinIndex {
    #[serde(default)]
    skins: Vec<SavedSkin>,
}

/// Persistent library rooted at the launcher data directory.
#[derive(Debug, Clone, Default)]
pub struct SkinLibrary {
    root: PathBuf,
    skins: Vec<SavedSkin>,
}

impl SkinLibrary {
    /// Loads `<root>/skins.json`. Missing or malformed data yields an empty
    /// library (the UI still works and a later save rewrites the index).
    pub fn load(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let index = std::fs::read(root.join("skins.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SkinIndex>(&bytes).ok())
            .unwrap_or_default();
        Self {
            root,
            skins: index
                .skins
                .into_iter()
                .filter(|skin| safe_file_name(&skin.file))
                .collect(),
        }
    }

    /// Saved skins in display order.
    pub fn skins(&self) -> &[SavedSkin] {
        &self.skins
    }

    pub fn is_empty(&self) -> bool {
        self.skins.is_empty()
    }

    /// Absolute PNG path of a saved skin.
    pub fn path_of(&self, skin: &SavedSkin) -> PathBuf {
        self.root.join("skins").join(&skin.file)
    }

    /// Copies PNG bytes into the library and persists the index.
    pub fn add(&mut self, name: &str, slim: bool, png: &[u8]) -> Result<SavedSkin> {
        if png.len() < 8 || &png[..8] != b"\x89PNG\r\n\x1a\n" {
            return Err(LauncherError::Config(
                "saved skin is not a PNG image".to_string(),
            ));
        }
        // Keep the index bounded: a skin is at most a few hundred KiB, and
        // anything in the megabytes is not a Minecraft skin.
        if png.len() > 4 * 1024 * 1024 {
            return Err(LauncherError::Config("saved skin is too large".to_string()));
        }

        let id = Uuid::new_v4();
        let file = format!("{id}.png");
        let dir = self.root.join("skins");
        std::fs::create_dir_all(&dir).map_err(|source| LauncherError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = dir.join(&file);
        std::fs::write(&path, png).map_err(|source| LauncherError::Io {
            path: path.clone(),
            source,
        })?;

        let skin = SavedSkin {
            id,
            name: unique_name(&normalized_name(name), &self.skins),
            file,
            slim,
        };
        self.skins.push(skin.clone());
        if let Err(err) = self.save() {
            // Roll back the in-memory and on-disk addition so the library
            // stays consistent with the index that failed to write.
            self.skins.pop();
            let _ = std::fs::remove_file(&path);
            return Err(err);
        }
        Ok(skin)
    }

    /// Deletes one saved skin and its PNG. Unknown ids are a no-op.
    pub fn remove(&mut self, id: Uuid) -> Result<()> {
        let Some(pos) = self.skins.iter().position(|skin| skin.id == id) else {
            return Ok(());
        };
        let skin = self.skins.remove(pos);
        let path = self.path_of(&skin);
        // The index entry is removed even when the file is already gone.
        let _ = std::fs::remove_file(&path);
        self.save()
    }

    /// Writes the index atomically under `<root>/skins.json`.
    pub fn save(&self) -> Result<()> {
        let index = SkinIndex {
            skins: self.skins.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&index)?;
        write_atomic(&self.root.join("skins.json"), &bytes)
    }
}

fn safe_file_name(file: &str) -> bool {
    let path = Path::new(file);
    path.components().count() == 1
        && path.file_name().and_then(|name| name.to_str()) == Some(file)
        && file.ends_with(".png")
}

fn unique_name(base: &str, existing: &[SavedSkin]) -> String {
    if !existing.iter().any(|skin| skin.name == base) {
        return base.to_string();
    }
    for suffix in 2.. {
        let suffix = format!(" {suffix}");
        let keep = 48usize.saturating_sub(suffix.chars().count());
        let candidate = format!("{}{suffix}", base.chars().take(keep).collect::<String>());
        if !existing.iter().any(|skin| skin.name == candidate) {
            return candidate;
        }
    }
    base.to_string()
}

fn normalized_name(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "Skin".to_string();
    }
    let mut cut: String = name.chars().take(48).collect();
    if cut.is_empty() {
        cut = "Skin".to_string();
    }
    cut
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot-a-real-image";

    fn temp_root(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("amc_skins_{tag}_{}_{nonce}", std::process::id()))
    }

    #[test]
    fn test_add_load_remove_roundtrip() {
        let root = temp_root("roundtrip");
        let _ = std::fs::remove_dir_all(&root);
        let mut library = SkinLibrary::load(&root);
        assert!(library.is_empty());

        let saved = library.add("My Skin", true, PNG).unwrap();
        assert_eq!(saved.name, "My Skin");
        assert!(saved.slim);
        assert!(library.path_of(&saved).is_file());
        assert!(root.join("skins.json").is_file());

        let reloaded = SkinLibrary::load(&root);
        assert_eq!(reloaded.skins(), library.skins());

        library.remove(saved.id).unwrap();
        assert!(library.is_empty());
        assert!(!library.path_of(&saved).exists());
        assert!(SkinLibrary::load(&root).is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_add_rejects_non_png_and_blank_names_default() {
        let root = temp_root("validation");
        let _ = std::fs::remove_dir_all(&root);
        let mut library = SkinLibrary::load(&root);
        assert!(library.add("Skin", false, b"not png").is_err());
        let saved = library.add("   ", false, PNG).unwrap();
        assert_eq!(saved.name, "Skin");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_missing_or_corrupt_index_is_empty() {
        let root = temp_root("corrupt");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("skins.json"), b"{broken").unwrap();
        assert!(SkinLibrary::load(&root).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_duplicate_names_get_a_suffix() {
        let root = temp_root("duplicates");
        let _ = std::fs::remove_dir_all(&root);
        let mut library = SkinLibrary::load(&root);
        assert_eq!(library.add("Same", false, PNG).unwrap().name, "Same");
        assert_eq!(library.add("Same", false, PNG).unwrap().name, "Same 2");
        assert_eq!(library.add("Same", false, PNG).unwrap().name, "Same 3");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_index_rejects_path_traversal() {
        assert!(safe_file_name("123.png"));
        assert!(!safe_file_name("../123.png"));
        assert!(!safe_file_name("nested/123.png"));
        assert!(!safe_file_name("123.txt"));
    }

    #[test]
    fn test_name_is_trimmed_and_bounded() {
        let name = " x ".to_string() + &"a".repeat(80);
        let normalized = normalized_name(&name);
        assert_eq!(normalized.chars().count(), 48);
        assert!(!normalized.starts_with(' '));
    }
}
