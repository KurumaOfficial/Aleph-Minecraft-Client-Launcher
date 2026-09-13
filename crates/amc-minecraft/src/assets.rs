use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use amc_core::error::{LauncherError, Result};
use amc_downloader::DownloadItem;
use crate::version::AssetIndexRef;

pub const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

impl AssetIndex {
    pub async fn fetch_or_load(
        client: &Client,
        asset_ref: &AssetIndexRef,
        assets_dir: &Path,
    ) -> Result<Self> {
        let index_file = assets_dir.join("indexes").join(format!("{}.json", asset_ref.id));

        if index_file.is_file() {
            if let Ok(content) = fs::read_to_string(&index_file).await {
                if let Ok(index) = serde_json::from_str::<Self>(&content) {
                    return Ok(index);
                }
            }
        }

        if let Some(parent) = index_file.parent() {
            fs::create_dir_all(parent).await.map_err(|e| LauncherError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let res = client
            .get(&asset_ref.url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Asset index fetch failed: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Asset index HTTP {}",
                res.status()
            )));
        }

        let text = res
            .text()
            .await
            .map_err(|e| LauncherError::Network(format!("Asset index text error: {e}")))?;

        let index: Self = serde_json::from_str(&text).map_err(LauncherError::Json)?;
        let _ = fs::write(&index_file, &text).await;

        Ok(index)
    }

    pub fn to_download_items(&self, assets_dir: &Path) -> Vec<DownloadItem> {
        let objects_dir = assets_dir.join("objects");
        let mut items = Vec::with_capacity(self.objects.len());

        for object in self.objects.values() {
            if object.hash.len() < 2 {
                continue;
            }
            let prefix = &object.hash[..2];
            let url = format!("{RESOURCES_BASE_URL}/{prefix}/{}", object.hash);
            let dest = objects_dir.join(prefix).join(&object.hash);

            items.push(
                DownloadItem::new(url, dest)
                    .with_sha1(&object.hash)
                    .with_size(object.size),
            );
        }

        items
    }
}
