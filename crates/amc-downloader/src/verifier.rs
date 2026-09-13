use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use amc_core::error::{LauncherError, Result};

pub fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(40);
    for byte in digest.iter() {
        let _ = write!(out, "{:02x}", byte);
    }
    out
}

pub async fn file_sha1(path: &Path) -> Result<String> {
    let mut file = File::open(path).await.map_err(|e| LauncherError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer).await.map_err(|e| LauncherError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let digest = hasher.finalize();
    let mut out = String::with_capacity(40);
    for byte in digest.iter() {
        let _ = write!(out, "{:02x}", byte);
    }
    Ok(out)
}

pub async fn verify_file_sha1(path: &Path, expected_sha1: &str) -> Result<bool> {
    if !path.is_file() {
        return Ok(false);
    }
    match file_sha1(path).await {
        Ok(actual) => Ok(actual.eq_ignore_ascii_case(expected_sha1)),
        Err(_) => Ok(false),
    }
}
