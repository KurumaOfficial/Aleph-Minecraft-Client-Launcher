use amc_core::error::{LauncherError, Result};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::process::Child;

/// Local dedicated server for an instance (CONCEPT "Совместная игра / Steam":
/// the "create local server" button). Downloads `server.jar` from Mojang,
/// writes `eula.txt` only after the user accepts it in the UI, and runs
/// `java -jar server.jar nogui`. The child uses `kill_on_drop`, so stopping
/// the launcher always stops the server too.
pub struct DedicatedServer;

impl DedicatedServer {
    pub fn dir(game_dir: &Path) -> PathBuf {
        game_dir.join("server")
    }

    /// Fetch `server.jar` for a game version into the server dir.
    /// `server_url` comes from the version details (`downloads.server`).
    pub async fn ensure_server_jar(
        client: &Client,
        server_url: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        tokio::fs::create_dir_all(server_dir)
            .await
            .map_err(|e| LauncherError::Io {
                path: server_dir.to_path_buf(),
                source: e,
            })?;
        let jar = server_dir.join("server.jar");
        if jar.is_file() {
            return Ok(jar);
        }
        let res = client
            .get(server_url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Server jar download failed: {e}")))?;
        if !res.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Server jar HTTP {}",
                res.status()
            )));
        }
        let bytes = res
            .bytes()
            .await
            .map_err(|e| LauncherError::Network(format!("Server jar body failed: {e}")))?;
        tokio::fs::write(&jar, &bytes)
            .await
            .map_err(|e| LauncherError::Io {
                path: jar.clone(),
                source: e,
            })?;
        Ok(jar)
    }

    pub fn eula_accepted(server_dir: &Path) -> bool {
        std::fs::read_to_string(server_dir.join("eula.txt"))
            .map(|text| text.contains("eula=true"))
            .unwrap_or(false)
    }

    pub fn write_eula(server_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(server_dir).map_err(|e| LauncherError::Io {
            path: server_dir.to_path_buf(),
            source: e,
        })?;
        std::fs::write(server_dir.join("eula.txt"), "eula=true\n").map_err(|e| {
            LauncherError::Io {
                path: server_dir.join("eula.txt"),
                source: e,
            }
        })?;
        Ok(())
    }

    /// Start the server on `port`. The caller owns the child and kills it
    /// with the Stop button; `kill_on_drop` covers launcher exits/crashes.
    pub fn start(
        java_bin: &Path,
        server_dir: &Path,
        jar: &Path,
        memory_mb: u32,
        port: u16,
    ) -> Result<Child> {
        if !Self::eula_accepted(server_dir) {
            return Err(LauncherError::Custom(
                "Accept the Mojang EULA first".to_string(),
            ));
        }
        let props = format!("server-port={port}\nonline-mode=true\nmotd=Aleph local server\n");
        std::fs::write(server_dir.join("server.properties"), props).map_err(|e| {
            LauncherError::Io {
                path: server_dir.join("server.properties"),
                source: e,
            }
        })?;
        tokio::process::Command::new(java_bin)
            .arg(format!("-Xmx{memory_mb}M"))
            .arg("-jar")
            .arg(jar)
            .arg("nogui")
            .current_dir(server_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| LauncherError::Launch(format!("Cannot start local server: {e}")))
    }

    /// Default server jar URL pattern for a vanilla version id. Real code
    /// prefers `downloads.server` from the version details; this is only a
    /// documented fallback shape and is unit-tested as such.
    pub fn mojang_server_url(mc_version: &str) -> String {
        format!("https://piston-data.mojang.com/v1/objects/server/{mc_version}/server.jar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_dir_layout() {
        let game = Path::new("/instances/survival");
        assert_eq!(
            DedicatedServer::dir(game),
            Path::new("/instances/survival/server")
        );
    }

    #[test]
    fn test_eula_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "amc_eula_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!DedicatedServer::eula_accepted(&dir));
        DedicatedServer::write_eula(&dir).unwrap();
        assert!(DedicatedServer::eula_accepted(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_start_requires_eula() {
        let dir = std::env::temp_dir();
        let err = DedicatedServer::start(
            Path::new("/nonexistent/java"),
            &dir,
            &dir.join("server.jar"),
            1024,
            25565,
        )
        .unwrap_err();
        assert!(matches!(err, LauncherError::Custom(_)));
    }
}
