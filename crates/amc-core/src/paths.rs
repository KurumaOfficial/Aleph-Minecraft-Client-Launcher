use std::fs;
use std::path::PathBuf;
use crate::error::{LauncherError, Result};

#[derive(Debug, Clone)]
pub struct LauncherPaths {
    pub root_dir: PathBuf,
}

impl LauncherPaths {
    pub fn default_paths() -> Result<Self> {
        let root = if cfg!(target_os = "windows") {
            dirs::data_dir()
                .map(|p| p.join("AlephLauncher"))
                .unwrap_or_else(|| PathBuf::from("./AlephLauncher"))
        } else if cfg!(target_os = "macos") {
            dirs::data_dir()
                .map(|p| p.join("AlephLauncher"))
                .unwrap_or_else(|| PathBuf::from("./AlephLauncher"))
        } else {
            dirs::data_local_dir()
                .map(|p| p.join("aleph-launcher"))
                .unwrap_or_else(|| PathBuf::from("./.aleph-launcher"))
        };

        let paths = Self { root_dir: root };
        paths.ensure_directories()?;
        Ok(paths)
    }

    pub fn custom(root: impl Into<PathBuf>) -> Result<Self> {
        let paths = Self { root_dir: root.into() };
        paths.ensure_directories()?;
        Ok(paths)
    }

    pub fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            self.root_dir.clone(),
            self.instances_dir(),
            self.versions_dir(),
            self.assets_dir(),
            self.libraries_dir(),
            self.runtimes_dir(),
            self.cache_dir(),
            self.logs_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                fs::create_dir_all(&dir).map_err(|e| LauncherError::Io {
                    path: dir.clone(),
                    source: e,
                })?;
            }
        }
        Ok(())
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.root_dir.join("instances")
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root_dir.join("versions")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root_dir.join("assets")
    }

    pub fn asset_indexes_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    pub fn asset_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.root_dir.join("libraries")
    }

    pub fn runtimes_dir(&self) -> PathBuf {
        self.root_dir.join("runtimes")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root_dir.join("cache")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root_dir.join("logs")
    }

    pub fn config_file(&self) -> PathBuf {
        self.root_dir.join("config.json")
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.root_dir.join("accounts.json")
    }

    pub fn instances_file(&self) -> PathBuf {
        self.root_dir.join("instances.json")
    }

    pub fn default_minecraft_dir() -> PathBuf {
        if cfg!(target_os = "windows") {
            dirs::data_dir()
                .map(|p| p.join(".minecraft"))
                .unwrap_or_else(|| PathBuf::from("./.minecraft"))
        } else if cfg!(target_os = "macos") {
            dirs::data_dir()
                .map(|p| p.join("minecraft"))
                .unwrap_or_else(|| PathBuf::from("./minecraft"))
        } else {
            dirs::home_dir()
                .map(|p| p.join(".minecraft"))
                .unwrap_or_else(|| PathBuf::from("./.minecraft"))
        }
    }
}
