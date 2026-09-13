use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::{LauncherError, Result};
use crate::types::LaunchOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub ui_scale: f32,
    pub close_after_launch: bool,
    pub show_snapshots: bool,
    pub show_betas: bool,
    pub show_alphas: bool,
    pub show_old: bool,
    pub language: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "aleph-dark".to_string(),
            ui_scale: 1.0,
            close_after_launch: false,
            show_snapshots: false,
            show_betas: false,
            show_alphas: false,
            show_old: false,
            language: "ru".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub ui: UiSettings,
    pub default_launch_options: LaunchOptions,
    pub custom_game_dir: Option<PathBuf>,
    pub selected_version: Option<String>,
    pub selected_instance: Option<uuid::Uuid>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            default_launch_options: LaunchOptions::default(),
            custom_game_dir: None,
            selected_version: None,
            selected_instance: None,
        }
    }
}

impl LauncherConfig {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            let config = Self::default();
            config.save_to_path(path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(path).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let config = serde_json::from_str(&content).map_err(LauncherError::Json)?;
        Ok(config)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(LauncherError::Json)?;
        fs::write(path, content).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }
}
