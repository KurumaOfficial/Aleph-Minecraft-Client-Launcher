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
}

use crate::error::{LauncherError, Result};

impl Instance {
    pub fn new(name: impl Into<String>, game_version: impl Into<String>, loader: LoaderType) -> Self {
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
                .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' { c } else { '_' })
                .collect();
            let folder_name = if sanitized.trim().is_empty() {
                self.id.to_string()
            } else {
                sanitized.trim().to_string()
            };
            base_instances_dir.join(folder_name)
        }
    }

    pub fn clone_instance(&self, new_name: &str, base_instances_dir: &std::path::Path) -> Result<Self> {
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
        let subdirs = ["mods", "saves", "resourcepacks", "shaderpacks", "config", "screenshots"];
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
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            quick_play_server: None,
            quick_play_port: None,
        }
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

        let cloned = original.clone_instance("Test Pack (Copy)", &temp_dir).unwrap();
        assert_ne!(original.id, cloned.id);
        assert_eq!(cloned.name, "Test Pack (Copy)");
        assert_eq!(cloned.game_version, "1.20.1");
        assert_eq!(cloned.loader, LoaderType::Fabric);

        let _ = std::fs::remove_dir_all(&temp_dir);
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
        assert_eq!(deserialized.quick_play_server.as_deref(), Some("hypixel.net"));
        assert_eq!(deserialized.quick_play_port, Some(25565));
    }
}
