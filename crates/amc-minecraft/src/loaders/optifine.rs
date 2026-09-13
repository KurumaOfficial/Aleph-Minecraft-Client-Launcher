use reqwest::Client;
use serde::Deserialize;
use amc_core::error::{LauncherError, Result};

const BMCLAPI_OPTIFINE: &str = "https://bmclapi2.bangbang93.com/optifine";

#[derive(Debug, Clone, Deserialize)]
pub struct OptiFineVersion {
    #[serde(rename = "mcversion")]
    pub mc_version: String,
    #[serde(rename = "type")]
    pub patch_type: String,
    pub patch: String,
    pub filename: String,
}

pub struct OptiFineLoader;

impl OptiFineLoader {
    pub async fn get_versions_for_mc(client: &Client, mc_version: &str) -> Result<Vec<OptiFineVersion>> {
        let url = format!("{BMCLAPI_OPTIFINE}/{mc_version}");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("OptiFine list error: {e}")))?;

        if !res.status().is_success() {
            return Ok(Vec::new());
        }

        let list: Vec<OptiFineVersion> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("OptiFine json error: {e}")))?;

        Ok(list)
    }

    pub fn download_url(filename: &str) -> String {
        format!("{BMCLAPI_OPTIFINE}/download?filename={filename}")
    }
}
