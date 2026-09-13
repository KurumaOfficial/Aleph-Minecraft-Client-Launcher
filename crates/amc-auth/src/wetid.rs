use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use amc_core::error::{LauncherError, Result};
use crate::types::Account;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WetIdLoginRequest {
    pub login: String,
    pub password: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WetIdProfile {
    pub uuid: String,
    pub username: String,
    pub token: String,
    pub skin_url: Option<String>,
    pub trust_level: u32,
}

pub struct WetIdClient {
    client: Client,
    api_url: String,
}

impl Default for WetIdClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            api_url: "https://api.aleph.network/v1/auth/wetid".to_string(),
        }
    }
}

impl WetIdClient {
    pub fn new(api_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            api_url: api_url.into(),
        }
    }

    pub async fn authenticate(&self, login: &str, password: &str) -> Result<Account> {
        let req = WetIdLoginRequest {
            login: login.to_string(),
            password: Some(password.to_string()),
            token: None,
        };

        let res = self
            .client
            .post(&format!("{}/login", self.api_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("WetID authentication request failed: {e}")))?;

        if !res.status().is_success() {
            let msg = res.text().await.unwrap_or_default();
            return Err(LauncherError::Auth(format!("WetID login failed: {msg}")));
        }

        let profile: WetIdProfile = res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse WetID profile: {e}")))?;

        Ok(Account::new_wetid(
            profile.username,
            profile.uuid,
            profile.token,
            profile.skin_url,
        ))
    }

    pub async fn authenticate_with_token(&self, token: &str) -> Result<Account> {
        let res = self
            .client
            .get(&format!("{}/verify", self.api_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("WetID verify request failed: {e}")))?;

        if !res.status().is_success() {
            return Err(LauncherError::Auth("WetID token expired or invalid".into()));
        }

        let profile: WetIdProfile = res
            .json()
            .await
            .map_err(|e| LauncherError::Auth(format!("Failed to parse WetID profile: {e}")))?;

        Ok(Account::new_wetid(
            profile.username,
            profile.uuid,
            profile.token,
            profile.skin_url,
        ))
    }
}
