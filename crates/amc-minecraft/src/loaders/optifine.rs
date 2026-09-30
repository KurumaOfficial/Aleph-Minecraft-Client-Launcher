use crate::version::{
    ArgumentValue, FileDownload, Library, LibraryDownloads, VersionArguments, VersionDetails,
};
use amc_core::error::{LauncherError, Result};
use reqwest::Client;
use serde::Deserialize;

const BMCLAPI_OPTIFINE: &str = "https://bmclapi2.bangbang93.com/optifine";
const LAUNCHWRAPPER_MAVEN: &str = "https://libraries.minecraft.net/";
const TWEAK_CLASS: &str = "optifine.OptiFineTweaker";

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
    pub async fn get_versions_for_mc(
        client: &Client,
        mc_version: &str,
    ) -> Result<Vec<OptiFineVersion>> {
        let url = format!("{BMCLAPI_OPTIFINE}/{mc_version}");
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("OptiFine list error: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Loader(format!(
                "OptiFine HTTP {} for MC {mc_version}",
                res.status()
            )));
        }

        let list: Vec<OptiFineVersion> = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("OptiFine json error: {e}")))?;

        if list.is_empty() {
            return Err(LauncherError::Loader(format!(
                "No OptiFine builds published for MC {mc_version}"
            )));
        }

        Ok(list)
    }

    /// Newest stable patch: pre-releases (`pre`/`alpha`/`beta`/`rc`) lose to
    /// the plain release; otherwise lexical max. Pure, unit-tested.
    pub fn select_patch(versions: &[OptiFineVersion]) -> Option<&OptiFineVersion> {
        let is_prerelease = |patch: &str| {
            let lower = patch.to_lowercase();
            ["pre", "alpha", "beta", "rc"]
                .iter()
                .any(|marker| lower.contains(marker))
        };
        versions
            .iter()
            .filter(|v| !is_prerelease(&v.patch))
            .max_by(|a, b| a.patch.cmp(&b.patch))
            .or_else(|| versions.iter().max_by(|a, b| a.patch.cmp(&b.patch)))
    }

    pub fn download_url(filename: &str) -> String {
        format!(
            "{BMCLAPI_OPTIFINE}/download?filename={}",
            url_encode(filename)
        )
    }

    /// Hand-built version details mirroring what the OptiFine installer
    /// generates: launchwrapper main class + tweak class, own jar described
    /// as a maven-layout library so the standard downloader fetches it.
    pub fn build_details(mc_version: &str, of: &OptiFineVersion) -> VersionDetails {
        let version_part = format!("{mc_version}_{}", of.patch);
        let coord = format!("optifine:OptiFine:{version_part}");
        VersionDetails {
            id: format!("{mc_version}-OptiFine_{}", of.patch),
            downloads: None,
            asset_index: None,
            libraries: vec![
                Library {
                    name: coord.clone(),
                    downloads: Some(LibraryDownloads {
                        artifact: Some(FileDownload {
                            sha1: None,
                            size: None,
                            url: Self::download_url(&of.filename),
                            path: Some(Library::maven_to_path(&coord)),
                        }),
                        classifiers: None,
                    }),
                    rules: None,
                    natives: None,
                    url: None,
                },
                Library {
                    name: "net.minecraft:launchwrapper:1.12".to_string(),
                    downloads: None,
                    rules: None,
                    natives: None,
                    url: Some(LAUNCHWRAPPER_MAVEN.to_string()),
                },
            ],
            main_class: "net.minecraft.launchwrapper.Launch".to_string(),
            arguments: Some(VersionArguments {
                game: vec![
                    ArgumentValue::Simple("--tweakClass".to_string()),
                    ArgumentValue::Simple(TWEAK_CLASS.to_string()),
                ],
                jvm: Vec::new(),
            }),
            minecraft_arguments: None,
            java_version: None,
            inherits_from: Some(mc_version.to_string()),
        }
    }

    /// Legacy (≤1.12) vanilla carries `minecraftArguments`, which a merged
    /// modern `arguments` block would shadow in the builder — fold the tweak
    /// into the legacy string instead. Modern parents are untouched.
    pub fn normalize_legacy_tweak(merged: &mut VersionDetails) {
        if let Some(legacy) = merged.minecraft_arguments.clone() {
            merged.minecraft_arguments = Some(format!("{legacy} --tweakClass {TWEAK_CLASS}"));
            merged.arguments = None;
        }
    }

    /// Network part of the pipeline: list → select → build. The OptiFine jar
    /// itself flows through the standard library downloader afterwards, so no
    /// Java or installer run is needed here.
    pub async fn resolve(client: &Client, mc_version: &str) -> Result<VersionDetails> {
        let versions = Self::get_versions_for_mc(client, mc_version).await?;
        let selected = Self::select_patch(&versions).ok_or_else(|| {
            LauncherError::Loader(format!("No OptiFine builds published for MC {mc_version}"))
        })?;
        Ok(Self::build_details(mc_version, selected))
    }
}

/// Minimal percent-encoding for the `filename` query value.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(patch: &str) -> OptiFineVersion {
        OptiFineVersion {
            mc_version: "1.20.1".to_string(),
            patch_type: "HD_U".to_string(),
            patch: patch.to_string(),
            filename: format!("OptiFine_1.20.1_{patch}.jar"),
        }
    }

    #[test]
    fn test_select_prefers_stable_over_prerelease() {
        let versions = vec![entry("HD_U_I6"), entry("HD_U_I7_pre1"), entry("HD_U_I7")];
        assert_eq!(
            OptiFineLoader::select_patch(&versions).unwrap().patch,
            "HD_U_I7"
        );
    }

    #[test]
    fn test_select_all_prerelease_falls_back_to_max() {
        let versions = vec![entry("HD_U_I7_pre1"), entry("HD_U_I7_pre2")];
        assert_eq!(
            OptiFineLoader::select_patch(&versions).unwrap().patch,
            "HD_U_I7_pre2"
        );
        let empty: Vec<OptiFineVersion> = Vec::new();
        assert!(OptiFineLoader::select_patch(&empty).is_none());
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(
            OptiFineLoader::download_url("OptiFine 1.20.1_HD_U_I7.jar"),
            "https://bmclapi2.bangbang93.com/optifine/download?filename=OptiFine%201.20.1_HD_U_I7.jar"
        );
        assert_eq!(url_encode("abc-_.~"), "abc-_.~");
    }

    #[test]
    fn test_build_details_shape() {
        let of = entry("HD_U_I7");
        let details = OptiFineLoader::build_details("1.20.1", &of);
        assert_eq!(details.id, "1.20.1-OptiFine_HD_U_I7");
        assert_eq!(details.inherits_from.as_deref(), Some("1.20.1"));
        assert_eq!(details.main_class, "net.minecraft.launchwrapper.Launch");
        assert_eq!(details.libraries.len(), 2);
        let artifact = details.libraries[0]
            .downloads
            .as_ref()
            .and_then(|d| d.artifact.as_ref())
            .unwrap();
        assert!(artifact.url.contains("OptiFine_1.20.1_HD_U_I7.jar"));
        assert_eq!(
            artifact.path.as_deref(),
            Some("optifine/OptiFine/1.20.1_HD_U_I7/OptiFine-1.20.1_HD_U_I7.jar")
        );
    }

    #[test]
    fn test_normalize_legacy_tweak() {
        // Legacy parent: tweak folds into the string, modern block is dropped.
        let mut legacy = OptiFineLoader::build_details("1.12.2", &entry("HD_U_C9"));
        legacy.minecraft_arguments = Some("--username ${auth_player_name}".to_string());
        OptiFineLoader::normalize_legacy_tweak(&mut legacy);
        let args = legacy.minecraft_arguments.unwrap();
        assert!(args.contains("--username ${auth_player_name}"));
        assert!(args.contains("--tweakClass optifine.OptiFineTweaker"));
        assert!(legacy.arguments.is_none());

        // Modern parent: untouched.
        let mut modern = OptiFineLoader::build_details("1.20.1", &entry("HD_U_I7"));
        OptiFineLoader::normalize_legacy_tweak(&mut modern);
        assert!(modern.minecraft_arguments.is_none());
        let game = &modern.arguments.unwrap().game;
        assert!(game.len() == 2);
    }
}
