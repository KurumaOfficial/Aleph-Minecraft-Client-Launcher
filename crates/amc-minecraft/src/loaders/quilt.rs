use crate::version::VersionDetails;
use amc_core::error::{LauncherError, Result};
use reqwest::Client;
use serde::Deserialize;

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
    /// Loader list endpoint for one MC version (pure for tests).
    pub fn loader_list_url(mc_version: &str) -> String {
        format!("{QUILT_META}/versions/loader/{mc_version}")
    }

    /// Profile endpoint for one MC + loader version (pure for tests).
    pub fn profile_url(mc_version: &str, loader_version: &str) -> String {
        format!("{QUILT_META}/versions/loader/{mc_version}/{loader_version}/profile/json")
    }

    pub async fn get_latest_loader_version(client: &Client, mc_version: &str) -> Result<String> {
        let url = Self::loader_list_url(mc_version);
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
        let url = Self::profile_url(mc_version, loader_version);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_urls() {
        assert_eq!(
            QuiltLoader::loader_list_url("1.20.1"),
            "https://meta.quiltmc.org/v3/versions/loader/1.20.1"
        );
        assert_eq!(
            QuiltLoader::profile_url("1.20.1", "0.25.0"),
            "https://meta.quiltmc.org/v3/versions/loader/1.20.1/0.25.0/profile/json"
        );
    }
}
