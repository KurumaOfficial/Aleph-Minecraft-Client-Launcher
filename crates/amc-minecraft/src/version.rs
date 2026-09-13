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
}
