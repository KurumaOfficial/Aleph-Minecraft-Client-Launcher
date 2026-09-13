use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Microsoft,
    WetId,
    Offline,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Microsoft => "Microsoft",
            Self::WetId => "WetID",
            Self::Offline => "Offline",
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub account_type: AccountType,
    pub username: String,
    pub uuid: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub skin_url: Option<String>,
    pub active: bool,
}

impl Account {
    pub fn new_offline(username: String, uuid: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_type: AccountType::Offline,
            username,
            uuid,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            skin_url: None,
            active: true,
        }
    }

    pub fn new_microsoft(
        username: String,
        uuid: String,
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_type: AccountType::Microsoft,
            username,
            uuid,
            access_token: Some(access_token),
            refresh_token,
            expires_at,
            skin_url: None,
            active: true,
        }
    }

    pub fn new_wetid(
        username: String,
        uuid: String,
        token: String,
        skin_url: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_type: AccountType::WetId,
            username,
            uuid,
            access_token: Some(token),
            refresh_token: None,
            expires_at: None,
            skin_url,
            active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() >= exp
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub xuid: Option<String>,
}

impl AuthSession {
    pub fn from_account(account: &Account) -> Self {
        match account.account_type {
            AccountType::Microsoft => Self {
                username: account.username.clone(),
                uuid: account.uuid.clone(),
                access_token: account.access_token.clone().unwrap_or_else(|| "0".to_string()),
                user_type: "msa".to_string(),
                xuid: None,
            },
            AccountType::WetId => Self {
                username: account.username.clone(),
                uuid: account.uuid.clone(),
                access_token: account.access_token.clone().unwrap_or_else(|| "0".to_string()),
                user_type: "wetid".to_string(),
                xuid: None,
            },
            AccountType::Offline => Self {
                username: account.username.clone(),
                uuid: account.uuid.clone(),
                access_token: "0".to_string(),
                user_type: "legacy".to_string(),
                xuid: None,
            },
        }
    }
}
