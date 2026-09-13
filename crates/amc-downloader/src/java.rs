use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use zip::ZipArchive;
use amc_core::error::{LauncherError, Result};
use crate::engine::{DownloadEngine, DownloadItem};
use crate::progress::ProgressTracker;

pub struct AdoptiumInstaller;

impl AdoptiumInstaller {
    pub fn os_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        }
    }

    pub fn arch_name() -> &'static str {
        if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "x64"
        }
    }

    pub fn java_executable(runtime_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            runtime_dir.join("bin").join("java.exe")
        } else {
            runtime_dir.join("bin").join("java")
        }
    }

    pub async fn ensure_java(
        runtimes_dir: &Path,
        major: u32,
        engine: &DownloadEngine,
        tracker: Option<&ProgressTracker>,
    ) -> Result<PathBuf> {
        let dest_dir = runtimes_dir.join(format!("java{}", major));
        let bin = Self::java_executable(&dest_dir);

        if bin.is_file() {
            tracing::info!("Java {} already installed at {}", major, bin.display());
            return Ok(bin);
        }

        let is_windows = cfg!(target_os = "windows");
        let ext = if is_windows { "zip" } else { "tar.gz" };
        let archive_path = dest_dir.with_extension(ext);

        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{major}/ga/{}/{}/jre/hotspot/normal/eclipse",
            Self::os_name(),
            Self::arch_name()
        );

        tracing::info!("Downloading Java {} from Adoptium: {}", major, url);

        let item = DownloadItem::new(&url, &archive_path);
        engine.download_one(&item, tracker).await?;

        tracing::info!("Extracting Java {} archive...", major);
        tokio::task::spawn_blocking({
            let archive_path = archive_path.clone();
            let dest_dir = dest_dir.clone();
            move || -> Result<()> {
                fs::create_dir_all(&dest_dir).map_err(|e| LauncherError::Io {
                    path: dest_dir.clone(),
                    source: e,
                })?;

                if is_windows {
                    let file = File::open(&archive_path).map_err(|e| LauncherError::Io {
                        path: archive_path.clone(),
                        source: e,
                    })?;
                    let mut zip = ZipArchive::new(file).map_err(|e| {
                        LauncherError::Java(format!("Failed to open Java zip: {e}"))
                    })?;

                    for i in 0..zip.len() {
                        let mut entry = zip.by_index(i).map_err(|e| {
                            LauncherError::Java(format!("Failed to read zip entry: {e}"))
                        })?;

                        let Some(enclosed) = entry.enclosed_name() else {
                            continue;
                        };

                        // Strip top-level directory e.g. "jdk-17.0.1+12-jre/"
                        let mut components = enclosed.components();
                        components.next();
                        let relative: PathBuf = components.collect();
                        if relative.as_os_str().is_empty() {
                            continue;
                        }

                        let out = dest_dir.join(relative);
                        if entry.is_dir() {
                            fs::create_dir_all(&out).map_err(|e| LauncherError::Io {
                                path: out.clone(),
                                source: e,
                            })?;
                        } else {
                            if let Some(parent) = out.parent() {
                                fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                                    path: parent.to_path_buf(),
                                    source: e,
                                })?;
                            }
                            let mut target = File::create(&out).map_err(|e| LauncherError::Io {
                                path: out.clone(),
                                source: e,
                            })?;
                            copy(&mut entry, &mut target).map_err(|e| LauncherError::Io {
                                path: out.clone(),
                                source: e,
                            })?;
                        }
                    }
                } else {
                    let file = File::open(&archive_path).map_err(|e| LauncherError::Io {
                        path: archive_path.clone(),
                        source: e,
                    })?;
                    let tar = GzDecoder::new(file);
                    let mut archive = TarArchive::new(tar);

                    for entry_res in archive.entries().map_err(|e| LauncherError::Java(format!("Tar error: {e}")))? {
                        let mut entry = entry_res.map_err(|e| LauncherError::Java(format!("Tar entry error: {e}")))?;
                        let path = entry.path().map_err(|e| LauncherError::Java(format!("Tar path error: {e}")))?;

                        let mut components = path.components();
                        components.next();
                        let relative: PathBuf = components.collect();
                        if relative.as_os_str().is_empty() {
                            continue;
                        }

                        let out = dest_dir.join(relative);
                        if let Some(parent) = out.parent() {
                            fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                                path: parent.to_path_buf(),
                                source: e,
                            })?;
                        }
                        entry.unpack(&out).map_err(|e| LauncherError::Io {
                            path: out.clone(),
                            source: e,
                        })?;
                    }
                }

                let _ = fs::remove_file(&archive_path);
                Ok(())
            }
        })
        .await
        .map_err(|e| LauncherError::Custom(format!("Extraction thread panicked: {e}")))?
        ?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&bin) {
                let mut perm = meta.permissions();
                perm.set_mode(0o755);
                let _ = fs::set_permissions(&bin, perm);
            }
        }

        if !bin.is_file() {
            return Err(LauncherError::Java(format!(
                "Java binary not found at {}",
                bin.display()
            )));
        }

        tracing::info!("Java {} installed successfully at {}", major, bin.display());
        Ok(bin)
    }
}
