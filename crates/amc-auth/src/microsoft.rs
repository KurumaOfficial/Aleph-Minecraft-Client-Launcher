use chrono::{Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use amc_core::error::{LauncherError, Result};
use crate::types::Account;

pub const DEFAULT_CLIENT_ID: &str = "00000000402b5328"; // Standard Minecraft public client ID for Azure

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct XboxLiveAuthRequest<'a> {
    #[serde(rename = "Properties")]
    properties: XboxLiveProperties<'a>,
    #[serde(rename = "RelyingParty")]
    relying_party: &'a str,
    #[serde(rename = "TokenType")]
    token_type: &'a str,
}

#[derive(Debug, Serialize)]
struct XboxLiveProperties<'a> {
    #[serde(rename = "AuthMethod")]
    auth_method: &'a str,
    #[serde(rename = "SiteName")]
    site_name: &'a str,
    #[serde(rename = "RpsTicket")]
    rps_ticket: &'a str,
}

#[derive(Debug, Deserialize)]
struct XboxLiveAuthResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
struct DisplayClaims {
    xui: Vec<XuiClaim>,
}

#[derive(Debug, Deserialize)]
struct XuiClaim {
    uhs: String,
}

#[derive(Debug, Serialize)]
struct XstsAuthRequest<'a> {
    #[serde(rename = "Properties")]
    properties: XstsProperties<'a>,
    #[serde(rename = "RelyingParty")]
    relying_party: &'a str,
    #[serde(rename = "TokenType")]
    token_type: &'a str,
}

#[derive(Debug, Serialize)]
struct XstsProperties<'a> {
    #[serde(rename = "SandboxId")]
    sandbox_id: &'a str,
    #[serde(rename = "UserTokens")]
    user_tokens: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
}

pub struct MicrosoftAuthFlow {
    client: Client,
    client_id: String,
}

impl Default for MicrosoftAuthFlow {
    fn default() -> Self {
        Self::new(DEFAULT_CLIENT_ID)
    }
}

impl MicrosoftAuthFlow {
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(StdDuration::from_secs(30))
                .build()
                .unwrap_or_default(),
            client_id: client_id.into(),
        }
    }

    /// Step 1: Request device authorization code
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("scope", "XboxLive.signin offline_access"),
        ];

        let res = self
            .client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&params)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Device code request failed: {e}")))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(LauncherError::Auth(format!("Device code error: {body}")));
        }

        let resp = res
            .json::<DeviceCodeResponse>()
            .await
            .map_err(|e| LauncherError::Network(format!("Failed to parse device code response: {e}")))?;

        Ok(resp)
    }

    /// Step 2: Poll token endpoint until user confirms or timeout
    pub async fn poll_device_token(&self, device_code: &str) -> Result<Option<Account>> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ];

        let res = self
            .client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Token poll failed: {e}")))?;

        let token_resp: MsTokenResponse = res
            .json()
            .await
            .map_err(|e| LauncherError::Network(format!("Failed to parse MS token response: {e}")))?;

        if let Some(err) = token_resp.error {
            if err == "authorization_pending" {
                return Ok(None);
            }
            return Err(LauncherError::Auth(format!("MS auth failed: {err}")));
        }

        // Complete Minecraft Auth flow with the MS access token
        let account = self.complete_minecraft_auth(token_resp).await?;
        Ok(Some(account))
    }

    /// Internal: Complete Xbox Live -> XSTS -> Mojang Auth -> Profile
    async fn complete_minecraft_auth(&self, ms_token: MsTokenResponse) -> Result<Account> {
        let rps_ticket = format!("d={}", ms_token.access_token);

        // 1. Xbox Live Authenticate
        let xbl_req = XboxLiveAuthRequest {
            properties: XboxLiveProperties {
                auth_method: "RPS",
                site_name: "user.auth.xboxlive.com",
                rps_ticket: &rps_ticket,
            },
            relying_party: "http://auth.xboxlive.com",
            token_type: "JWT",
        };

        let xbl_res = self
            .client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&xbl_req)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Xbox Live auth failed: {e}")))?;

        let xbl_body: XboxLiveAuthResponse = xbl_res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse Xbox Live response: {e}")))?;

        let user_hash = xbl_body
            .display_claims
            .xui
            .first()
            .ok_or_else(|| LauncherError::Auth("Missing Xbox Live user hash".into()))?
            .uhs
            .clone();

        // 2. XSTS Authorize
        let xsts_req = XstsAuthRequest {
            properties: XstsProperties {
                sandbox_id: "RETAIL",
                user_tokens: vec![&xbl_body.token],
            },
            relying_party: "rp://api.minecraftservices.com/",
            token_type: "JWT",
        };

        let xsts_res = self
            .client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&xsts_req)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("XSTS authorize failed: {e}")))?;

        let xsts_body: XboxLiveAuthResponse = xsts_res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse XSTS response: {e}")))?;

        // 3. Mojang Login with Xbox
        let mc_payload = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_body.token)
        });

        let mc_res = self
            .client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&mc_payload)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Minecraft services login failed: {e}")))?;

        let mc_auth: MinecraftAuthResponse = mc_res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse Minecraft login response: {e}")))?;

        // 4. Fetch Minecraft Profile
        let profile_res = self
            .client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .bearer_auth(&mc_auth.access_token)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Minecraft profile request failed: {e}")))?;

        let profile: MinecraftProfileResponse = profile_res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse Minecraft profile: {e}")))?;

        let expires_at = ms_token
            .expires_in
            .map(|sec| Utc::now() + Duration::seconds(sec));

        Ok(Account::new_microsoft(
            profile.name,
            profile.id,
            mc_auth.access_token,
            ms_token.refresh_token,
            expires_at,
        ))
    }
}
