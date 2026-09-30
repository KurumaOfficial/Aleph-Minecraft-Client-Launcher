use amc_core::error::{LauncherError, Result};
use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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
        let n = file
            .read(&mut buffer)
            .await
            .map_err(|e| LauncherError::Io {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hex_vector() {
        // Well-known SHA-1 test vector.
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[tokio::test]
    async fn test_hash_file_prefix_matches_full_hash() {
        use sha1::Digest;
        let dir = std::env::temp_dir().join(format!(
            "amc_prefix_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("prefix.bin");
        std::fs::write(&file, b"hello world, this is a resume probe").unwrap();

        let (hasher, len) = crate::engine::hash_file_prefix(&file).await;
        assert_eq!(len, 35);
        let mut hex = String::with_capacity(40);
        for byte in hasher.finalize().iter() {
            hex.push_str(&format!("{byte:02x}"));
        }
        assert_eq!(hex, file_sha1(&file).await.unwrap());

        // Missing file restarts from scratch.
        let (_, zero) = crate::engine::hash_file_prefix(&dir.join("missing.bin")).await;
        assert_eq!(zero, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_file_sha1_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "amc_verify_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("hello.bin");
        std::fs::write(&file, b"hello world").unwrap();

        let hex = file_sha1(&file).await.unwrap();
        assert_eq!(hex, sha1_hex(b"hello world"));
        assert!(verify_file_sha1(&file, &hex).await.unwrap());
        assert!(
            !verify_file_sha1(&file, "0000000000000000000000000000000000000000")
                .await
                .unwrap()
        );
        assert!(!verify_file_sha1(&dir.join("missing.bin"), &hex)
            .await
            .unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
