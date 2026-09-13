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

    pub fn ensure_directories(&self, base_instances_dir: &std::path::Path) -> Result<PathBuf> {
        let game_dir = self.get_game_dir(base_instances_dir);
        let subdirs = ["mods", "saves", "resourcepacks", "shaderpacks", "config"];
        for sub in subdirs {
            let p = game_dir.join(sub);
            let _ = std::fs::create_dir_all(&p);
        }
        Ok(game_dir)
    }
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
        }
    }
}
