use reqwest::Client;
use serde::Deserialize;
use amc_core::error::{LauncherError, Result};
use crate::version::VersionDetails;

const FABRIC_META: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Deserialize)]
struct FabricLoaderEntry {
    loader: FabricLoaderInfo,
}

#[derive(Debug, Clone, Deserialize)]
struct FabricLoaderInfo {
    version: String,
    stable: bool,
}

pub struct FabricLoader;

impl FabricLoader {
    pub async fn get_latest_loader_version(client: &Client, mc_version: &str) -> Result<String> {
        let url = format!("{FABRIC_META}/versions/loader/{mc_version}");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Fabric meta error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "No Fabric loader found for MC {mc_version}"
            )));
        }

        let mut list: Vec<FabricLoaderEntry> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Fabric json error: {e}")))?;

        if list.is_empty() {
            return Err(LauncherError::Loader(format!(
                "Empty Fabric loader list for MC {mc_version}"
            )));
        }

        if let Some(pos) = list.iter().position(|e| e.loader.stable) {
            Ok(list[pos].loader.version.clone())
        } else {
            Ok(list.remove(0).loader.version)
        }
    }

    pub async fn fetch_profile_json(
        client: &Client,
        mc_version: &str,
        loader_version: &str,
    ) -> Result<VersionDetails> {
        let url = format!("{FABRIC_META}/versions/loader/{mc_version}/{loader_version}/profile/json");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Fabric profile error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "HTTP {} fetching Fabric profile",
                res.status()
            )));
        }

        let text = res
            .text()
            .await
            .map_err(|e| LauncherError::Network(format!("Fabric profile text error: {e}")))?;

        VersionDetails::parse(&text)
    }
}
