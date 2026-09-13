use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModCategory {
    #[default]
    Mod,
    ResourcePack,
    Shader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub source: ModSource,
    pub id: String,
    pub title: String,
    pub author: String,
    pub downloads: u64,
    pub description: String,
    pub icon_url: Option<String>,
    pub category: ModCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDownloadFile {
    pub filename: String,
    pub url: String,
    pub version: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMod {
    pub filename: String,
    pub path: PathBuf,
    pub name: String,
    pub id: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub file_size: u64,
}
