use std::fmt::Write as _;
use md5::{Digest, Md5};
use amc_core::error::{LauncherError, Result};
use crate::types::Account;

pub fn validate_nickname(name: &str) -> Result<()> {
    let len = name.chars().count();
    if !(3..=16).contains(&len) {
        return Err(LauncherError::Auth(
            "Никнейм должен содержать от 3 до 16 символов".to_string(),
        ));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(LauncherError::Auth(
            "Никнейм может содержать только латиницу, цифры и символ подчеркивания".to_string(),
        ));
    }
    Ok(())
}

pub fn offline_uuid(name: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", name).as_bytes());
    let mut digest = hasher.finalize();
    digest[6] = (digest[6] & 0x0f) | 0x30; // UUID version 3
    digest[8] = (digest[8] & 0x3f) | 0x80; // Variant RFC 4122
    let mut out = String::with_capacity(32);
    for byte in digest.iter() {
        let _ = write!(out, "{:02x}", byte);
    }
    out
}

pub fn dashed_uuid(uuid: &str) -> String {
    if uuid.len() != 32 {
        return uuid.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &uuid[0..8],
        &uuid[8..12],
        &uuid[12..16],
        &uuid[16..20],
        &uuid[20..32]
    )
}

pub fn login_offline(name: &str) -> Result<Account> {
    validate_nickname(name)?;
    let raw_uuid = offline_uuid(name);
    let formatted_uuid = dashed_uuid(&raw_uuid);
    Ok(Account::new_offline(name.to_string(), formatted_uuid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_uuid_generation() {
        let uuid = offline_uuid("Steve");
        assert_eq!(uuid.len(), 32);
        let dashed = dashed_uuid(&uuid);
        assert_eq!(dashed.len(), 36);
        assert_eq!(dashed.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_nickname_validation() {
        assert!(validate_nickname("Player123").is_ok());
        assert!(validate_nickname("ab").is_err());
        assert!(validate_nickname("VeryLongNicknameExceedingLimit").is_err());
        assert!(validate_nickname("Invalid Nick").is_err());
    }
}
