use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use amc_core::error::{LauncherError, Result};
use amc_downloader::DownloadItem;
use crate::rules::{allows, current_os, Rule};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDownload {
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDownloads {
    pub client: Option<FileDownload>,
    pub server: Option<FileDownload>,
    pub client_mappings: Option<FileDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersionInfo {
    pub component: Option<String>,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<FileDownload>,
    pub classifiers: Option<HashMap<String, FileDownload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    pub url: Option<String>,
}

impl Library {
    pub fn is_allowed(&self, features: &HashMap<String, bool>) -> bool {
        allows(self.rules.as_deref(), features)
    }

    pub fn to_download_item(&self, libraries_dir: &Path) -> Option<DownloadItem> {
        // 1. Check direct artifact in downloads
        if let Some(downloads) = &self.downloads {
            if let Some(artifact) = &downloads.artifact {
                let rel_path = artifact
                    .path
                    .clone()
                    .unwrap_or_else(|| Self::maven_to_path(&self.name));
                let dest = libraries_dir.join(rel_path);
                let mut item = DownloadItem::new(&artifact.url, dest);
                if let Some(sha1) = &artifact.sha1 {
                    item = item.with_sha1(sha1);
                }
                if let Some(size) = artifact.size {
                    item = item.with_size(size);
                }
                return Some(item);
            }

            // Check natives classifier if available
            if let Some(classifiers) = &downloads.classifiers {
                let os_name = current_os();
                let native_key = self
                    .natives
                    .as_ref()
                    .and_then(|n| n.get(os_name))
                    .map(|s| s.replace("${arch}", if cfg!(target_arch = "x86_64") { "64" } else { "32" }))
                    .unwrap_or_else(|| format!("natives-{}", os_name));

                if let Some(classifier_art) = classifiers.get(&native_key) {
                    let rel_path = classifier_art
                        .path
                        .clone()
                        .unwrap_or_else(|| Self::maven_to_path(&self.name));
                    let dest = libraries_dir.join(rel_path);
                    let mut item = DownloadItem::new(&classifier_art.url, dest);
                    if let Some(sha1) = &classifier_art.sha1 {
                        item = item.with_sha1(sha1);
                    }
                    if let Some(size) = classifier_art.size {
                        item = item.with_size(size);
                    }
                    return Some(item);
                }
            }
        }

        // 2. Custom maven URL (e.g. Fabric, Forge, OptiFine)
        if let Some(base_url) = &self.url {
            let rel_path = Self::maven_to_path(&self.name);
            let full_url = format!("{}/{}", base_url.trim_end_matches('/'), rel_path);
            let dest = libraries_dir.join(rel_path);
            return Some(DownloadItem::new(full_url, dest));
        }

        None
    }

    pub fn maven_to_path(name: &str) -> String {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 {
            return name.to_string();
        }
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let classifier = if parts.len() >= 4 {
            format!("-{}", parts[3])
        } else {
            String::new()
        };
        format!("{group}/{artifact}/{version}/{artifact}-{version}{classifier}.jar")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Complex {
        #[serde(default)]
        rules: Option<Vec<Rule>>,
        value: ArgumentValueNested,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValueNested {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionArguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    pub downloads: Option<VersionDownloads>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndexRef>,
    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: Option<VersionArguments>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersionInfo>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
}

impl VersionDetails {
    pub fn parse(json_content: &str) -> Result<Self> {
        serde_json::from_str(json_content).map_err(LauncherError::Json)
    }

    pub fn required_java_major(&self) -> u32 {
        if let Some(java) = &self.java_version {
            return java.major_version;
        }

        // Fallback heuristics based on version string
        if self.id.starts_with("1.20") || self.id.starts_with("1.21") {
            21
        } else if self.id.starts_with("1.17") || self.id.starts_with("1.18") || self.id.starts_with("1.19") {
            17
        } else {
            8
        }
    }

    pub async fn fetch_or_load(
        client: &reqwest::Client,
        version_id: &str,
        url: Option<&str>,
        versions_dir: &Path,
    ) -> Result<Self> {
        let version_dir = versions_dir.join(version_id);
        let version_json_path = version_dir.join(format!("{version_id}.json"));

        if version_json_path.is_file() {
            if let Ok(content) = tokio::fs::read_to_string(&version_json_path).await {
                if let Ok(details) = Self::parse(&content) {
                    return Ok(details);
                }
            }
        }

        let fetch_url = if let Some(u) = url {
            u.to_string()
        } else {
            let manifest = crate::manifest::VersionManifest::fetch(client, None).await?;
            let entry = manifest
                .versions
                .into_iter()
                .find(|v| v.id == version_id)
                .ok_or_else(|| {
                    LauncherError::Version(format!("Версия {version_id} не найдена в манифесте"))
                })?;
            entry.url
        };

        let res = client
            .get(&fetch_url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Ошибка загрузки JSON версии: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "HTTP {} при загрузке версии {}",
                res.status(),
                version_id
            )));
        }

        let text = res
            .text()
            .await
            .map_err(|e| LauncherError::Network(format!("Ошибка чтения тела версии: {e}")))?;
        let details = Self::parse(&text)?;

        let _ = tokio::fs::create_dir_all(&version_dir).await;
        let _ = tokio::fs::write(&version_json_path, &text).await;

        Ok(details)
    }

    pub fn merge_parent(&mut self, parent: VersionDetails) {
        if self.downloads.is_none() {
            self.downloads = parent.downloads;
        }
        if self.asset_index.is_none() {
            self.asset_index = parent.asset_index;
        }
        if self.java_version.is_none() {
            self.java_version = parent.java_version;
        }
        if self.minecraft_arguments.is_none() {
            self.minecraft_arguments = parent.minecraft_arguments;
        }

        // Merge arguments
        if let Some(parent_args) = parent.arguments {
            if let Some(my_args) = &mut self.arguments {
                let mut merged_game = parent_args.game;
                merged_game.append(&mut my_args.game);
                my_args.game = merged_game;

                let mut merged_jvm = parent_args.jvm;
                merged_jvm.append(&mut my_args.jvm);
                my_args.jvm = merged_jvm;
            } else {
                self.arguments = Some(parent_args);
            }
        }

        // Merge libraries (avoid duplicate names)
        let mut existing_names: std::collections::HashSet<String> =
            self.libraries.iter().map(|l| l.name.clone()).collect();

        for lib in parent.libraries {
            if !existing_names.contains(&lib.name) {
                existing_names.insert(lib.name.clone());
                self.libraries.push(lib);
            }
        }
    }

    pub fn extract_natives(jar_path: &Path, natives_dir: &Path) -> Result<()> {
        if !jar_path.is_file() {
            return Ok(());
        }

        let file = std::fs::File::open(jar_path).map_err(|e| LauncherError::Io {
            path: jar_path.to_path_buf(),
            source: e,
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            LauncherError::Custom(format!(
                "Не удалось открыть natives jar {}: {e}",
                jar_path.display()
            ))
        })?;

        std::fs::create_dir_all(natives_dir).map_err(|e| LauncherError::Io {
            path: natives_dir.to_path_buf(),
            source: e,
        })?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                LauncherError::Custom(format!("Ошибка чтения zip entry: {e}"))
            })?;

            let Some(name) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
                continue;
            };
            let name_str = name.to_string_lossy();
            if name_str.starts_with("META-INF") || name_str.starts_with('.') {
                continue;
            }

            let ext = name.extension().and_then(|s| s.to_str()).unwrap_or("");
            let is_native = if cfg!(windows) {
                ext == "dll"
            } else if cfg!(target_os = "macos") {
                ext == "dylib" || ext == "jnilib"
            } else {
                ext == "so"
            };

            if is_native {
                let file_name = name.file_name().unwrap_or(name.as_os_str());
                let dest = natives_dir.join(file_name);
                let mut out = std::fs::File::create(&dest).map_err(|e| LauncherError::Io {
                    path: dest.clone(),
                    source: e,
                })?;
                std::io::copy(&mut entry, &mut out).map_err(|e| LauncherError::Io {
                    path: dest,
                    source: e,
                })?;
            }
        }

        Ok(())
    }
}
