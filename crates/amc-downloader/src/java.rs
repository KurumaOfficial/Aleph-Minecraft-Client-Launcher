use crate::engine::{DownloadCancel, DownloadEngine, DownloadItem};
use crate::progress::ProgressTracker;
use amc_core::error::{LauncherError, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use zip::ZipArchive;

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
        // Runtime download is not UI-cancellable yet (P3): uncancellable token.
        engine
            .download_one(&item, tracker, DownloadCancel::default())
            .await?;

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

                    for entry_res in archive
                        .entries()
                        .map_err(|e| LauncherError::Java(format!("Tar error: {e}")))?
                    {
                        let mut entry = entry_res
                            .map_err(|e| LauncherError::Java(format!("Tar entry error: {e}")))?;
                        // Reject symlinks/hardlinks: they could point outside `dest_dir`.
                        if entry.header().entry_type().is_symlink()
                            || entry.header().entry_type().is_hard_link()
                        {
                            continue;
                        }
                        let path = entry
                            .path()
                            .map_err(|e| LauncherError::Java(format!("Tar path error: {e}")))?;

                        // Reject absolute paths and `..` before joining.
                        if path.is_absolute()
                            || path.components().any(|c| {
                                matches!(
                                    c,
                                    std::path::Component::ParentDir
                                        | std::path::Component::RootDir
                                        | std::path::Component::Prefix(_)
                                )
                            })
                        {
                            continue;
                        }
                        let mut components = path.components();
                        components.next();
                        let relative: PathBuf = components.collect();
                        if relative.as_os_str().is_empty() {
                            continue;
                        }
                        // Reject `..` that survived stripping the top-level dir.
                        if relative.components().any(|c| {
                            matches!(
                                c,
                                std::path::Component::ParentDir
                                    | std::path::Component::RootDir
                                    | std::path::Component::Prefix(_)
                            )
                        }) {
                            continue;
                        }

                        let out = dest_dir.join(&relative);
                        // Final containment check after join.
                        if !out.starts_with(&dest_dir) {
                            continue;
                        }
                        if entry.header().entry_type().is_dir() {
                            fs::create_dir_all(&out).map_err(|e| LauncherError::Io {
                                path: out.clone(),
                                source: e,
                            })?;
                            continue;
                        }
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
        .map_err(|e| LauncherError::Custom(format!("Extraction thread panicked: {e}")))??;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_names_are_known() {
        assert!(["windows", "mac", "linux"].contains(&AdoptiumInstaller::os_name()));
        assert!(["x64", "aarch64", "x86"].contains(&AdoptiumInstaller::arch_name()));
    }

    #[test]
    fn test_java_executable_layout() {
        let exe = AdoptiumInstaller::java_executable(Path::new("/rt"));
        assert_eq!(exe.parent().and_then(|p| p.file_name()).unwrap(), "bin");
        #[cfg(windows)]
        assert_eq!(exe.extension().and_then(|e| e.to_str()), Some("exe"));
        #[cfg(not(windows))]
        assert_eq!(exe.extension(), None);
    }
}
