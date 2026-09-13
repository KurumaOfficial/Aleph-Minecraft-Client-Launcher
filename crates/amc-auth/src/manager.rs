use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use amc_core::error::{LauncherError, Result};
use crate::types::{Account, AuthSession};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountDatabase {
    pub accounts: Vec<Account>,
    pub active_account_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AccountManager {
    db_path: PathBuf,
    db: AccountDatabase,
}

impl AccountManager {
    pub fn load_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let db_path = path.into();
        let db = if db_path.exists() {
            let content = fs::read_to_string(&db_path).map_err(|e| LauncherError::Io {
                path: db_path.clone(),
                source: e,
            })?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AccountDatabase::default()
        };

        Ok(Self { db_path, db })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let json = serde_json::to_string_pretty(&self.db).map_err(LauncherError::Json)?;
        fs::write(&self.db_path, json).map_err(|e| LauncherError::Io {
            path: self.db_path.clone(),
            source: e,
        })?;
        Ok(())
    }

    pub fn accounts(&self) -> &[Account] {
        &self.db.accounts
    }

    pub fn active_account(&self) -> Option<&Account> {
        self.db
            .active_account_id
            .and_then(|id| self.db.accounts.iter().find(|a| a.id == id))
            .or_else(|| self.db.accounts.first())
    }

    pub fn set_active(&mut self, id: Uuid) -> Result<()> {
        if self.db.accounts.iter().any(|a| a.id == id) {
            self.db.active_account_id = Some(id);
            self.save()?;
            Ok(())
        } else {
            Err(LauncherError::Auth("Account not found".into()))
        }
    }

    pub fn add_or_update_account(&mut self, account: Account) -> Result<()> {
        let id = account.id;
        if let Some(pos) = self.db.accounts.iter().position(|a| a.username == account.username && a.account_type == account.account_type) {
            self.db.accounts[pos] = account;
            self.db.active_account_id = Some(self.db.accounts[pos].id);
        } else {
            self.db.accounts.push(account);
            self.db.active_account_id = Some(id);
        }
        self.save()
    }

    pub fn remove_account(&mut self, id: Uuid) -> Result<()> {
        self.db.accounts.retain(|a| a.id != id);
        if self.db.active_account_id == Some(id) {
            self.db.active_account_id = self.db.accounts.first().map(|a| a.id);
        }
        self.save()
    }

    pub fn get_session(&self) -> Option<AuthSession> {
        self.active_account().map(AuthSession::from_account)
    }
}
