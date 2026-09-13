use reqwest::Client;
use serde::Deserialize;
use amc_core::error::{LauncherError, Result};

const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
const NEOFORGE_META: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Deserialize)]
struct NeoForgeVersions {
    versions: Vec<String>,
}

pub struct NeoForgeLoader;

impl NeoForgeLoader {
    pub async fn get_latest_version(client: &Client, mc_version: &str) -> Result<String> {
        let res = client
            .get(NEOFORGE_META)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("NeoForge meta error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader("Failed to fetch NeoForge versions".into()));
        }

        let data: NeoForgeVersions = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("NeoForge json error: {e}")))?;

        let prefix = if mc_version.starts_with("1.20.") {
            "20."
        } else if mc_version.starts_with("1.21.") {
            "21."
        } else {
            ""
        };

        let found = data
            .versions
            .into_iter()
            .filter(|v| v.starts_with(prefix))
            .last()
            .ok_or_else(|| LauncherError::Loader(format!("No NeoForge version found for {mc_version}")))?;

        Ok(found)
    }

    pub fn installer_url(neoforge_version: &str) -> String {
        format!("{NEOFORGE_MAVEN}/net/neoforged/neoforge/{neoforge_version}/neoforge-{neoforge_version}-installer.jar")
    }

    pub fn version_id(mc_version: &str, neoforge_version: &str) -> String {
        format!("{mc_version}-neoforge-{neoforge_version}")
    }
}
