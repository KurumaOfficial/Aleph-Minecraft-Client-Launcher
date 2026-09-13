use reqwest::Client;
use serde::Deserialize;
use amc_core::error::{LauncherError, Result};
use crate::version::VersionDetails;

const QUILT_META: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Deserialize)]
struct QuiltLoaderEntry {
    loader: QuiltLoaderInfo,
}

#[derive(Debug, Clone, Deserialize)]
struct QuiltLoaderInfo {
    version: String,
}

pub struct QuiltLoader;

impl QuiltLoader {
    pub async fn get_latest_loader_version(client: &Client, mc_version: &str) -> Result<String> {
        let url = format!("{QUILT_META}/versions/loader/{mc_version}");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Quilt meta error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "No Quilt loader found for MC {mc_version}"
            )));
        }

        let mut list: Vec<QuiltLoaderEntry> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Quilt json error: {e}")))?;

        if list.is_empty() {
            return Err(LauncherError::Loader(format!(
                "Empty Quilt loader list for MC {mc_version}"
            )));
        }

        Ok(list.remove(0).loader.version)
    }

    pub async fn fetch_profile_json(
        client: &Client,
        mc_version: &str,
        loader_version: &str,
    ) -> Result<VersionDetails> {
        let url = format!("{QUILT_META}/versions/loader/{mc_version}/{loader_version}/profile/json");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Quilt profile error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "HTTP {} fetching Quilt profile",
                res.status()
            )));
        }

        let text = res
            .text()
            .await
            .map_err(|e| LauncherError::Network(format!("Quilt profile text error: {e}")))?;

        VersionDetails::parse(&text)
    }
}
