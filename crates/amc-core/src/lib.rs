pub mod autostart;
pub mod config;
pub mod error;
pub mod hardware;
pub mod health;
pub mod i18n;
pub mod logging;
pub mod paths;

pub mod settings_bundle;
pub mod shortcuts;
pub mod trash;
pub mod types;
pub mod worlds;

pub use config::{
    AfterLaunch, AppMode, LaunchHotkey, LauncherConfig, NotifSettings, TabHotkeyMod, UiSettings,
};
pub use error::{LauncherError, Result};
pub use hardware::{free_disk_mb, HardwareReport};
pub use health::{HealthCheck, HealthReport};
pub use i18n::Language;
pub use logging::init_logging;
pub use paths::looks_like_minecraft;
pub use paths::LauncherPaths;
pub use shortcuts::{
    linux_desktop_entry, sanitize_file_stem, windows_lnk_script, InstanceShortcut,
};
pub use trash::{
    copy_dir, list_trash, purge_trash_all, purge_trash_entry, restore_instance, trash_instance,
    TrashedInstance,
};
pub use types::{GameVersion, Instance, InstanceGroup, LaunchOptions, LoaderType, ReleaseType};
pub use worlds::{
    backup_world, delete_backup, list_backups, list_saves, needs_auto_backup, newest_backup_mtime,
    restore_world,
};
