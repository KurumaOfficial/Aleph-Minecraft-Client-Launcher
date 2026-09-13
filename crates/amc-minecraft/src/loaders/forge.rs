use reqwest::Client;
use serde::Deserialize;
use amc_core::error::{LauncherError, Result};

const MAVEN_FORGE: &str = "https://maven.minecraftforge.net";
const FORGE_API_MIRROR: &str = "https://bmclapi2.bangbang93.com/forge/minecraft";

#[derive(Debug, Clone, Deserialize)]
struct ForgeBuild {
    version: String,
    build: u64,
}

pub struct ForgeLoader;

impl ForgeLoader {
    pub async fn get_latest_version(client: &Client, mc_version: &str) -> Result<String> {
        let url = format!("{FORGE_API_MIRROR}/{mc_version}");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Forge list error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "No Forge versions found for MC {mc_version}"
            )));
        }

        let list: Vec<ForgeBuild> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Forge json error: {e}")))?;

        list.into_iter()
            .max_by_key(|b| b.build)
            .map(|b| b.version)
            .ok_or_else(|| LauncherError::Loader("Empty Forge list".into()))
    }

    pub fn installer_url(mc_version: &str, forge_version: &str) -> String {
        format!("{MAVEN_FORGE}/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar")
    }

    pub fn version_id(mc_version: &str, forge_version: &str) -> String {
        format!("{mc_version}-forge-{forge_version}")
    }
}
