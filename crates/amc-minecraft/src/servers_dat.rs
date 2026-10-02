use amc_core::error::{LauncherError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Minimal NBT writer for `servers.dat` (CONCEPT P8): favorites are injected
/// into fresh instances so players never add them by hand in-game.
/// Format: uncompressed big-endian NBT, root compound with a `servers` list
/// of compounds (`name` + `ip` strings). Existing files are never touched —
/// injection only seeds instances without a `servers.dat`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteServerEntry {
    pub name: String,
    pub address: String,
}

fn write_str(buf: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    buf.extend_from_slice(bytes);
}

/// Encode favorites as a `servers.dat` blob. Deterministic, unit-tested
/// against hand-computed golden bytes.
pub fn encode_servers_dat(servers: &[FavoriteServerEntry]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x0A); // TAG_Compound, root
    buf.extend_from_slice(&[0x00, 0x00]); // empty root name
    buf.push(0x09); // TAG_List "servers"
    write_str(&mut buf, "servers");
    buf.push(0x0A); // element type: compound
    buf.extend_from_slice(&(servers.len() as i32).to_be_bytes());
    for server in servers {
        buf.push(0x08); // TAG_String "ip"
        write_str(&mut buf, "ip");
        write_str(&mut buf, &server.address);
        buf.push(0x08); // TAG_String "name"
        write_str(&mut buf, "name");
        write_str(&mut buf, &server.name);
        buf.push(0x00); // TAG_End
    }
    buf.push(0x00); // root TAG_End
    buf
}

/// Write favorites into `<game_dir>/servers.dat`, but only when the player
/// has no file yet. Returns `true` when the file was created.
pub fn inject_favorites(game_dir: &Path, servers: &[FavoriteServerEntry]) -> Result<bool> {
    if servers.is_empty() {
        return Ok(false);
    }
    let target = game_dir.join("servers.dat");
    if target.is_file() {
        return Ok(false);
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| LauncherError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    let blob = encode_servers_dat(servers);
    std::fs::write(&target, blob).map_err(|e| LauncherError::Io {
        path: target.clone(),
        source: e,
    })?;
    Ok(true)
}

/// Parse `host[:port]` user input into a normalized `(host, port)` pair.
pub fn parse_server_address(raw: &str) -> Option<(String, u16)> {
    let text = raw.trim().trim_start_matches("mc://");
    if text.is_empty() || text.contains(char::is_whitespace) {
        return None;
    }
    match text.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() => {
            let port: u16 = port.parse().ok()?;
            Some((host.to_string(), port))
        }
        // ":port" with an empty host is not a valid address.
        Some(_) => None,
        _ => Some((text.to_string(), 25565)),
    }
}

pub fn favorites_file(root_dir: &Path) -> PathBuf {
    root_dir.join("favorites.json")
}

pub fn load_favorites(root_dir: &Path) -> Vec<FavoriteServerEntry> {
    let path = favorites_file(root_dir);
    if !path.is_file() {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str::<HashMap<String, Vec<FavoriteServerEntry>>>(&text)
        .ok()
        .and_then(|mut map| map.remove("servers"))
        .unwrap_or_default()
}

pub fn save_favorites(root_dir: &Path, servers: &[FavoriteServerEntry]) -> Result<()> {
    let mut map = HashMap::new();
    map.insert("servers".to_string(), servers.to_vec());
    let text = serde_json::to_string_pretty(&map).map_err(LauncherError::Json)?;
    std::fs::write(favorites_file(root_dir), text).map_err(|e| LauncherError::Io {
        path: favorites_file(root_dir),
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_servers_dat_golden() {
        let blob = encode_servers_dat(&[FavoriteServerEntry {
            name: "Test".to_string(),
            address: "play.example.com:25565".to_string(),
        }]);
        let mut expected = vec![0x0A, 0x00, 0x00, 0x09];
        expected.extend_from_slice(&7u16.to_be_bytes());
        expected.extend_from_slice(b"servers");
        expected.push(0x0A);
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.push(0x08);
        expected.extend_from_slice(&2u16.to_be_bytes());
        expected.extend_from_slice(b"ip");
        expected.extend_from_slice(&22u16.to_be_bytes());
        expected.extend_from_slice(b"play.example.com:25565");
        expected.push(0x08);
        expected.extend_from_slice(&4u16.to_be_bytes());
        expected.extend_from_slice(b"name");
        expected.extend_from_slice(&4u16.to_be_bytes());
        expected.extend_from_slice(b"Test");
        expected.push(0x00);
        expected.push(0x00);
        assert_eq!(blob, expected);
    }

    #[test]
    fn test_encode_empty_list() {
        let blob = encode_servers_dat(&[]);
        assert_eq!(
            blob,
            vec![
                0x0A, 0x00, 0x00, 0x09, 0x00, 0x07, b's', b'e', b'r', b'v', b'e', b'r', b's', 0x0A,
                0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_parse_server_address() {
        assert_eq!(
            parse_server_address("play.example.com:25565"),
            Some(("play.example.com".to_string(), 25565))
        );
        assert_eq!(
            parse_server_address("  hypixel.net "),
            Some(("hypixel.net".to_string(), 25565))
        );
        assert_eq!(parse_server_address("bad host:12"), None);
        assert_eq!(parse_server_address(""), None);
        assert_eq!(parse_server_address(":25565"), None);
    }

    #[test]
    fn test_inject_only_when_missing() {
        let dir = std::env::temp_dir().join(format!(
            "amc_servers_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let servers = vec![FavoriteServerEntry {
            name: "T".to_string(),
            address: "h:1".to_string(),
        }];
        assert!(inject_favorites(&dir, &servers).unwrap());
        assert!(dir.join("servers.dat").is_file());
        // Existing file is never overwritten.
        assert!(!inject_favorites(&dir, &servers).unwrap());
        assert!(!inject_favorites(&dir, &[]).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_favorites_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "amc_fav_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_favorites(&dir).is_empty());
        let servers = vec![FavoriteServerEntry {
            name: "T".to_string(),
            address: "h:1".to_string(),
        }];
        save_favorites(&dir, &servers).unwrap();
        assert_eq!(load_favorites(&dir), servers);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
