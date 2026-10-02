pub mod autostart;
pub mod config;
pub mod error;
pub mod hardware;
pub mod health;
pub mod i18n;
pub mod logging;
pub mod paths;
pub mod trash;
pub mod types;
pub mod worlds;

pub use config::{AfterLaunch, AppMode, LauncherConfig, NotifSettings, UiSettings};
pub use error::{LauncherError, Result};
pub use hardware::{free_disk_mb, HardwareReport};
pub use health::{HealthCheck, HealthReport};
pub use i18n::Language;
pub use logging::init_logging;
pub use paths::looks_like_minecraft;
pub use paths::LauncherPaths;
pub use trash::{
    copy_dir, list_trash, purge_trash_all, purge_trash_entry, restore_instance, trash_instance,
    TrashedInstance,
};
pub use types::{GameVersion, Instance, LaunchOptions, LoaderType, ReleaseType};
pub use worlds::{
    backup_world, delete_backup, list_backups, list_saves, needs_auto_backup, newest_backup_mtime,
    restore_world,
};
