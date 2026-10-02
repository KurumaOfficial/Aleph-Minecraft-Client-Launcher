use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoaderType {
    #[default]
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    NeoForge,
    OptiFine,
}

impl LoaderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
            Self::OptiFine => "OptiFine",
        }
    }
}

impl fmt::Display for LoaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseType {
    #[default]
    Release,
    Snapshot,
    Beta,
    Alpha,
    Old,
}

impl ReleaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "Release",
            Self::Snapshot => "Snapshot",
            Self::Beta => "Beta",
            Self::Alpha => "Alpha",
            Self::Old => "Old",
        }
    }
}

impl fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub id: String,
    pub release_type: ReleaseType,
    pub url: String,
    pub release_time: DateTime<Utc>,
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub game_version: String,
    pub loader: LoaderType,
    pub loader_version: Option<String>,
    pub custom_dir: Option<PathBuf>,
    pub ram_mb: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    pub total_played_minutes: u64,
    /// Preset icon key (`InstanceIcon::ALL`), `None` = loader glyph.
    /// A custom `icon.png` in the game dir wins over any preset.
    #[serde(default)]
    pub icon: Option<String>,
    /// Free-form tags for grouping/filtering (CONCEPT, P2).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Pinned instances sort above the rest.
    #[serde(default)]
    pub pinned: bool,
    /// Default account for this instance (CONCEPT: no per-launch picker).
    /// Missing or unknown ids fall back to the active account.
    #[serde(default)]
    pub default_account: Option<Uuid>,
    /// Personal notes (seed, who we play with…).
    #[serde(default)]
    pub notes: String,
    /// Custom command before launch (CONCEPT pre-launch scripts).
    #[serde(default)]
    pub pre_launch_cmd: String,
    /// Custom command after the game exits (CONCEPT post-exit scripts).
    #[serde(default)]
    pub post_exit_cmd: String,
    /// Mod jars installed by the launcher itself (provenance for the
    /// local-mod badge; anything else counts as local).
    #[serde(default)]
    pub installed_by_launcher: Vec<String>,
}

/// Preset instance icons (key, glyph). AMC-team set; custom art goes
/// through `icon.png`, no user icon packs in v1.0.
pub struct InstanceIcon;

impl InstanceIcon {
    pub const ALL: [(&'static str, &'static str); 12] = [
        ("pickaxe", "⛏"),
        ("sword", "🗡"),
        ("shield", "🛡"),
        ("bow", "🏹"),
        ("potion", "🧪"),
        ("dragon", "🐉"),
        ("castle", "🏰"),
        ("wheat", "🌾"),
        ("gear", "⚙"),
        ("alien", "👾"),
        ("art", "🎨"),
        ("fire", "🔥"),
    ];
    pub const CUSTOM_ICON_FILE: &'static str = "icon.png";

    pub fn glyph(key: &str) -> Option<&'static str> {
        Self::ALL
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, glyph)| *glyph)
    }
}

use crate::error::{LauncherError, Result};

impl Instance {
    pub fn new(
        name: impl Into<String>,
        game_version: impl Into<String>,
        loader: LoaderType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            game_version: game_version.into(),
            loader,
            loader_version: None,
            custom_dir: None,
            ram_mb: None,
            created_at: Utc::now(),
            last_played: None,
            total_played_minutes: 0,
            icon: None,
            tags: Vec::new(),
            pinned: false,
            default_account: None,
            notes: String::new(),
            pre_launch_cmd: String::new(),
            post_exit_cmd: String::new(),
            installed_by_launcher: Vec::new(),
        }
    }

    pub fn load_all(path: &std::path::Path) -> Result<Vec<Self>> {
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        serde_json::from_str(&content).map_err(LauncherError::Json)
    }

    pub fn save_all(path: &std::path::Path, instances: &[Self]) -> Result<()> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(instances).map_err(LauncherError::Json)?;
        std::fs::write(path, content).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    pub fn get_game_dir(&self, base_instances_dir: &std::path::Path) -> PathBuf {
        if let Some(custom) = &self.custom_dir {
            custom.clone()
        } else {
            let sanitized: String = self
                .name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let folder_name = if sanitized.trim().is_empty() {
                self.id.to_string()
            } else {
                sanitized.trim().to_string()
            };
            base_instances_dir.join(folder_name)
        }
    }

    /// Total bytes under the instance game directory (CONCEPT "Место на диске").
    /// Missing directories count as 0, unreadable entries are skipped and
    /// symlinks are never followed — measurement never fails.
    pub fn disk_usage(&self, base_instances_dir: &std::path::Path) -> u64 {
        Self::disk_usage_dir(&self.get_game_dir(base_instances_dir))
    }

    /// Size of an arbitrary directory tree. See [`disk_usage`](Self::disk_usage).
    pub fn disk_usage_dir(dir: &std::path::Path) -> u64 {
        let mut total = 0u64;
        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            let entries = match std::fs::read_dir(&current) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_symlink() {
                    continue;
                } else if path.is_dir() {
                    stack.push(path);
                } else if let Ok(meta) = entry.metadata() {
                    total = total.saturating_add(meta.len());
                }
            }
        }
        total
    }

    pub fn clone_instance(
        &self,
        new_name: &str,
        base_instances_dir: &std::path::Path,
    ) -> Result<Self> {
        let mut cloned = self.clone();
        cloned.id = Uuid::new_v4();
        cloned.name = new_name.to_string();
        cloned.created_at = Utc::now();
        cloned.last_played = None;
        cloned.total_played_minutes = 0;
        cloned.custom_dir = None;

        let src_dir = self.get_game_dir(base_instances_dir);
        let dst_dir = cloned.get_game_dir(base_instances_dir);

        if src_dir.is_dir() {
            let _ = std::fs::create_dir_all(&dst_dir);
            for sub in &["config", "mods", "resourcepacks", "shaderpacks"] {
                let src_sub = src_dir.join(sub);
                let dst_sub = dst_dir.join(sub);
                if src_sub.is_dir() {
                    let _ = copy_dir_recursive(&src_sub, &dst_sub);
                }
            }
        } else {
            cloned.ensure_directories(base_instances_dir)?;
        }

        Ok(cloned)
    }

    pub fn ensure_directories(&self, base_instances_dir: &std::path::Path) -> Result<PathBuf> {
        let game_dir = self.get_game_dir(base_instances_dir);
        let subdirs = [
            "mods",
            "saves",
            "resourcepacks",
            "shaderpacks",
            "config",
            "screenshots",
        ];
        for sub in subdirs {
            let p = game_dir.join(sub);
            let _ = std::fs::create_dir_all(&p);
        }
        Ok(game_dir)
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub java_path: Option<PathBuf>,
    pub custom_jvm_args: Vec<String>,
    /// Extra game (not JVM) arguments, appended last so they can override
    /// anything above. `#[serde(default)]` keeps pre-P2 configs loadable.
    #[serde(default)]
    pub custom_game_args: Vec<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub quick_play_server: Option<String>,
    pub quick_play_port: Option<u16>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            memory_min_mb: 1024,
            memory_max_mb: 4096,
            java_path: None,
            custom_jvm_args: vec![
                "-XX:+UnlockExperimentalVMOptions".into(),
                "-XX:+UseG1GC".into(),
                "-XX:G1NewSizePercent=20".into(),
                "-XX:G1ReservePercent=20".into(),
                "-XX:MaxGCPauseMillis=50".into(),
                "-XX:G1HeapRegionSize=32m".into(),
            ],
            custom_game_args: Vec::new(),
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            quick_play_server: None,
            quick_play_port: None,
        }
    }
}

/// Quick-start preset for the create-instance dialog (CONCEPT, P2).
/// AMC-team templates only in v1.0 — no user/community templates.
/// A template prefills loader, RAM and (when untouched) the name;
/// the game version stays user-chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceTemplate {
    pub id: &'static str,
    pub loader: LoaderType,
    pub ram_mb: u32,
}

impl InstanceTemplate {
    pub const EMPTY: Self = Self {
        id: "empty",
        loader: LoaderType::Vanilla,
        ram_mb: 2048,
    };
    pub const VANILLA_PLUS: Self = Self {
        id: "vanilla_plus",
        loader: LoaderType::Vanilla,
        ram_mb: 4096,
    };
    pub const OPTIMIZED: Self = Self {
        id: "optimized",
        loader: LoaderType::Fabric,
        ram_mb: 6144,
    };
    pub const ALL: [Self; 3] = [Self::EMPTY, Self::VANILLA_PLUS, Self::OPTIMIZED];

    /// Template whose settings exactly match a loader/RAM pair, if any.
    pub fn matching(loader: LoaderType, ram_mb: u32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|t| t.loader == loader && t.ram_mb == ram_mb)
    }

    /// Suggested instance name in the UI language.
    pub fn default_name(&self, lang: crate::i18n::Language) -> String {
        lang.tmpl_name(self.id).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation_and_clone() {
        let temp_dir = std::env::temp_dir().join(format!("amc_test_{}", Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let original = Instance::new("Test Pack", "1.20.1", LoaderType::Fabric);
        let _ = original.ensure_directories(&temp_dir);

        let cloned = original
            .clone_instance("Test Pack (Copy)", &temp_dir)
            .unwrap();
        assert_ne!(original.id, cloned.id);
        assert_eq!(cloned.name, "Test Pack (Copy)");
        assert_eq!(cloned.game_version, "1.20.1");
        assert_eq!(cloned.loader, LoaderType::Fabric);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_legacy_instance_without_p2_fields_uses_defaults() {
        let legacy = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Old Pack",
            "game_version": "1.19.2",
            "loader": "forge",
            "loader_version": null,
            "custom_dir": null,
            "ram_mb": 4096,
            "created_at": "2024-01-01T00:00:00Z",
            "last_played": null,
            "total_played_minutes": 0
        }"#;
        let inst: Instance = serde_json::from_str(legacy).unwrap();
        assert_eq!(inst.icon, None);
        assert!(inst.tags.is_empty());
        assert!(!inst.pinned);
        assert_eq!(inst.default_account, None);
        assert!(inst.notes.is_empty());
        assert!(inst.pre_launch_cmd.is_empty());
        assert!(inst.post_exit_cmd.is_empty());
        assert!(inst.installed_by_launcher.is_empty());
    }

    #[test]
    fn test_instance_icon_keys_unique_and_known() {
        let mut keys = std::collections::HashSet::new();
        for (key, glyph) in InstanceIcon::ALL {
            assert!(keys.insert(key), "duplicate icon key: {key}");
            assert!(!glyph.is_empty());
            assert_eq!(InstanceIcon::glyph(key), Some(glyph));
        }
        assert_eq!(InstanceIcon::glyph("nope"), None);
        assert_eq!(InstanceIcon::CUSTOM_ICON_FILE, "icon.png");
    }

    #[test]
    fn test_disk_usage_counts_nested_files() {
        let root = std::env::temp_dir().join(format!("amc_disktest_{}", Uuid::new_v4()));
        let inst = Instance::new("Disk", "1.20.1", LoaderType::Vanilla);
        let game_dir = inst.get_game_dir(&root);
        std::fs::create_dir_all(game_dir.join("mods")).unwrap();
        std::fs::write(game_dir.join("x.jar"), vec![0u8; 100]).unwrap();
        std::fs::write(game_dir.join("mods").join("y.jar"), vec![0u8; 300]).unwrap();
        assert_eq!(inst.disk_usage(&root), 400);
        assert_eq!(Instance::disk_usage_dir(&game_dir.join("mods")), 300);
        // Missing trees measure zero instead of failing.
        assert_eq!(inst.disk_usage(&root.join("nonexistent-base")), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_instance_templates() {
        use crate::i18n::Language;
        assert_eq!(InstanceTemplate::ALL.len(), 3);
        let ids: Vec<&str> = InstanceTemplate::ALL.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec!["empty", "vanilla_plus", "optimized"]);
        for tmpl in InstanceTemplate::ALL {
            assert!((1024..=16384).contains(&tmpl.ram_mb));
            assert!(!Language::English.tmpl_name(tmpl.id).is_empty());
            assert!(!Language::Russian.tmpl_desc(tmpl.id).is_empty());
        }
        assert_eq!(
            InstanceTemplate::matching(LoaderType::Fabric, 6144).map(|t| t.id),
            Some("optimized")
        );
        assert_eq!(
            InstanceTemplate::matching(LoaderType::Forge, 4096).map(|t| t.id),
            None
        );
        assert_eq!(
            InstanceTemplate::OPTIMIZED.default_name(Language::Russian),
            "Оптимизированная"
        );
    }

    #[test]
    fn test_launch_options_quick_play() {
        let mut opts = LaunchOptions::default();
        assert!(opts.quick_play_server.is_none());
        opts.quick_play_server = Some("hypixel.net".to_string());
        opts.quick_play_port = Some(25565);

        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("hypixel.net"));
        let deserialized: LaunchOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.quick_play_server.as_deref(),
            Some("hypixel.net")
        );
        assert_eq!(deserialized.quick_play_port, Some(25565));
    }
}
