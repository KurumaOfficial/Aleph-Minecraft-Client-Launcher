use crate::error::{LauncherError, Result};
use crate::types::LaunchOptions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

impl UiSettings {
    pub fn language(&self) -> crate::i18n::Language {
        crate::i18n::Language::parse_code(&self.language)
    }

    pub fn set_language(&mut self, lang: crate::i18n::Language) {
        self.language = lang.code().to_string();
    }
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

/// Launcher operation mode (CONCEPT "Режимы работы", ROADMAP P1).
/// Persisted in `config.json`; freely switchable from Settings at any time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Single active setup with a quick-start button for newcomers.
    #[default]
    Simple,
    /// Multiple isolated instances (MultiMC/Prism-grade).
    Professional,
}

impl AppMode {
    pub const ALL: [AppMode; 2] = [AppMode::Simple, AppMode::Professional];

    pub fn toggle(self) -> Self {
        match self {
            Self::Simple => Self::Professional,
            Self::Professional => Self::Simple,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    pub ui: UiSettings,
    pub default_launch_options: LaunchOptions,
    pub custom_game_dir: Option<PathBuf>,
    pub selected_version: Option<String>,
    pub selected_instance: Option<uuid::Uuid>,
    /// Operation mode. `#[serde(default)]` keeps pre-P1 configs loadable.
    #[serde(default)]
    pub app_mode: AppMode,
    /// True until the first-run wizard (ROADMAP P1) completes.
    /// Defaults to `false` so configs written before P1 are never
    /// mistaken for a fresh install.
    #[serde(default)]
    pub first_run: bool,
    /// Global download speed limit in KiB/s (P3). `None` (default) = unlimited.
    #[serde(default)]
    pub download_speed_limit_kbps: Option<u64>,
}

impl LauncherConfig {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            let mut config = Self::default();
            // First launch: guess the UI language from the OS instead of
            // unconditionally assuming Russian (CONCEPT "Первый запуск").
            config
                .ui
                .set_language(crate::i18n::Language::detect_system());
            config.first_run = true;
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
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }

        let content = serde_json::to_string_pretty(self).map_err(LauncherError::Json)?;
        // Atomic write: a crash mid-save must never leave a truncated config.
        // `tmp` lives next to `path`, so `rename` stays on one filesystem.
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, &content).map_err(|e| LauncherError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;
        fs::rename(&tmp_path, path).map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_dir(suite: &str) -> PathBuf {
        let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "amc_config_test_{}_{}_{}",
            suite,
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_app_mode_serde_names() {
        let simple = serde_json::to_string(&AppMode::Simple).unwrap();
        let pro = serde_json::to_string(&AppMode::Professional).unwrap();
        assert_eq!(simple, "\"simple\"");
        assert_eq!(pro, "\"professional\"");
        assert_eq!(
            serde_json::from_str::<AppMode>("\"professional\"").unwrap(),
            AppMode::Professional
        );
        assert_eq!(AppMode::Simple.toggle(), AppMode::Professional);
        assert_eq!(AppMode::Professional.toggle(), AppMode::Simple);
    }

    #[test]
    fn test_legacy_config_without_p1_fields_uses_defaults() {
        // Config written before P1 has no `app_mode` / `first_run` keys.
        let legacy = r#"{
            "ui": {
                "theme": "aleph-dark",
                "ui_scale": 1.0,
                "close_after_launch": false,
                "show_snapshots": false,
                "show_betas": false,
                "show_alphas": false,
                "show_old": false,
                "language": "en"
            },
            "default_launch_options": {
                "memory_min_mb": 1024,
                "memory_max_mb": 4096,
                "java_path": null,
                "custom_jvm_args": [],
                "window_width": 854,
                "window_height": 480,
                "fullscreen": false,
                "quick_play_server": null,
                "quick_play_port": null
            },
            "custom_game_dir": null,
            "selected_version": "1.20.1",
            "selected_instance": null
        }"#;
        let config: LauncherConfig = serde_json::from_str(legacy).unwrap();
        assert_eq!(config.app_mode, AppMode::Simple);
        assert!(!config.first_run);
        assert_eq!(config.ui.language(), crate::i18n::Language::English);
    }

    #[test]
    fn test_fresh_load_marks_first_run_and_roundtrips() {
        let dir = unique_dir("fresh");
        let path = dir.join("config.json");

        let first = LauncherConfig::load_from_path(&path).unwrap();
        assert!(first.first_run);
        assert!(path.exists());

        let second = LauncherConfig::load_from_path(&path).unwrap();
        assert!(second.first_run);
        assert_eq!(second.ui.language, first.ui.language);
        assert_eq!(second.app_mode, AppMode::Simple);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_is_atomic_and_roundtrips() {
        let dir = unique_dir("atomic");
        let path = dir.join("nested").join("config.json");

        let mut config = LauncherConfig::default();
        config.app_mode = AppMode::Professional;
        config.first_run = true;
        config.save_to_path(&path).unwrap();

        // No temp file may linger next to the config.
        assert!(!path.with_extension("tmp").exists());

        let loaded = LauncherConfig::load_from_path(&path).unwrap();
        assert_eq!(loaded.app_mode, AppMode::Professional);
        assert!(loaded.first_run);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
