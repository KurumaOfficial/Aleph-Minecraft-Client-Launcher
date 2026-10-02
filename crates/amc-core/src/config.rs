use crate::error::{LauncherError, Result};
use crate::types::LaunchOptions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub ui_scale: f32,
    /// What to do with the launcher window once the game starts.
    /// Old `true`/`false` configs migrate to Close/Keep.
    #[serde(default, deserialize_with = "de_after_launch")]
    pub after_launch: AfterLaunch,
    pub show_snapshots: bool,
    pub show_betas: bool,
    pub show_alphas: bool,
    pub show_old: bool,
    pub language: String,
    /// Start with the OS (CONCEPT "Автозапуск"). Applied on toggle; may drift
    /// if the OS entry is removed externally.
    #[serde(default)]
    pub autostart: bool,
}

/// Launcher window behavior after game start (CONCEPT "После запуска игры").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AfterLaunch {
    Close,
    Minimize,
    #[default]
    Keep,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BoolOrAfterLaunch {
    Bool(bool),
    After(AfterLaunch),
}

fn de_after_launch<'de, D>(deserializer: D) -> std::result::Result<AfterLaunch, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match BoolOrAfterLaunch::deserialize(deserializer)? {
        BoolOrAfterLaunch::Bool(true) => Ok(AfterLaunch::Close),
        BoolOrAfterLaunch::Bool(false) => Ok(AfterLaunch::Keep),
        BoolOrAfterLaunch::After(mode) => Ok(mode),
    }
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
            after_launch: AfterLaunch::Keep,
            show_snapshots: false,
            show_betas: false,
            show_alphas: false,
            show_old: false,
            language: "ru".to_string(),
            autostart: false,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Custom storage root from Settings (CONCEPT "Хранение на диске").
    /// Applied on next launcher start.
    #[serde(default)]
    pub root_override: Option<PathBuf>,
    /// Per-type notification switches (CONCEPT "Уведомления").
    #[serde(default)]
    pub notifications: NotifSettings,
    /// Anonymous crash reports, ON by default, disable in Settings (opt-out).
    #[serde(default = "default_true")]
    pub crash_reports: bool,
    /// World backup folder override; default = launcher backups dir.
    #[serde(default)]
    pub world_backup_dir: Option<PathBuf>,
    /// Automatic world backups every N days (0 = manual only).
    #[serde(default)]
    pub world_backup_days: u64,
    /// Remembered launch identity (account UUID string); ask dialog when unset.
    #[serde(default)]
    pub remembered_identity: Option<String>,
    /// Ask which identity to play with on every launch (CONCEPT "Аккаунты").
    #[serde(default)]
    pub ask_identity_each_launch: bool,
    /// Tutorial hints already dismissed.
    #[serde(default)]
    pub tour_seen: bool,
    /// `.minecraft` reuse offer already resolved.
    #[serde(default)]
    pub mc_reuse_done: bool,
    /// Mod wishlist: `"source:id"` entries not yet installed.
    #[serde(default)]
    pub wishlist: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Per-type notification switches; everything on by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifSettings {
    pub updates: bool,
    pub downloads: bool,
    pub launcher: bool,
}

impl Default for NotifSettings {
    fn default() -> Self {
        Self {
            updates: true,
            downloads: true,
            launcher: true,
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            default_launch_options: LaunchOptions::default(),
            custom_game_dir: None,
            selected_version: None,
            selected_instance: None,
            app_mode: AppMode::default(),
            first_run: false,
            download_speed_limit_kbps: None,
            root_override: None,
            notifications: NotifSettings::default(),
            crash_reports: true,
            world_backup_dir: None,
            world_backup_days: 0,
            remembered_identity: None,
            ask_identity_each_launch: false,
            tour_seen: false,
            mc_reuse_done: false,
            wishlist: Vec::new(),
        }
    }
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
    fn test_after_launch_bool_migration() {
        // Old configs stored a bare bool.
        let closed: UiSettings =
            serde_json::from_str(r#"{"theme":"x","ui_scale":1.0,"after_launch":true,"show_snapshots":false,"show_betas":false,"show_alphas":false,"show_old":false,"language":"en"}"#)
                .unwrap();
        assert_eq!(closed.after_launch, AfterLaunch::Close);
        let kept: UiSettings =
            serde_json::from_str(r#"{"theme":"x","ui_scale":1.0,"after_launch":false,"show_snapshots":false,"show_betas":false,"show_alphas":false,"show_old":false,"language":"en"}"#)
                .unwrap();
        assert_eq!(kept.after_launch, AfterLaunch::Keep);
        // New string form round-trips.
        let minimized: UiSettings =
            serde_json::from_str(r#"{"theme":"x","ui_scale":1.0,"after_launch":"minimize","show_snapshots":false,"show_betas":false,"show_alphas":false,"show_old":false,"language":"en"}"#)
                .unwrap();
        assert_eq!(minimized.after_launch, AfterLaunch::Minimize);
        // Missing field defaults to Keep, never fails old files.
        let legacy: UiSettings =
            serde_json::from_str(r#"{"theme":"x","ui_scale":1.0,"show_snapshots":false,"show_betas":false,"show_alphas":false,"show_old":false,"language":"en"}"#)
                .unwrap();
        assert_eq!(legacy.after_launch, AfterLaunch::Keep);
    }

    #[test]
    fn test_fresh_defaults_reflect_concept() {
        let fresh = LauncherConfig::default();
        assert!(fresh.crash_reports);
        assert!(fresh.notifications.updates);
        assert!(fresh.notifications.downloads);
        assert!(fresh.notifications.launcher);
        assert_eq!(fresh.world_backup_days, 0);
        assert!(fresh.wishlist.is_empty());
        assert!(!fresh.ask_identity_each_launch);
        // Legacy file without any new keys still parses.
        let legacy: LauncherConfig =
            serde_json::from_str(r#"{"ui":{"theme":"x","ui_scale":1.0,"show_snapshots":false,"show_betas":false,"show_alphas":false,"show_old":false,"language":"en"},"default_launch_options":{"memory_min_mb":1024,"memory_max_mb":4096,"java_path":null,"custom_jvm_args":[],"custom_game_args":[],"window_width":854,"window_height":480,"fullscreen":false,"quick_play_server":null,"quick_play_port":null},"custom_game_dir":null,"selected_version":null,"selected_instance":null}"#)
                .unwrap();
        assert!(legacy.crash_reports);
        assert_eq!(legacy.world_backup_days, 0);
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
        assert!(config.default_launch_options.custom_game_args.is_empty());
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
