use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use amc_core::error::{LauncherError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct ServerVersionInfo {
    pub name: Option<String>,
    pub protocol: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerPlayersInfo {
    pub max: Option<i32>,
    pub online: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ServerDescription {
    Text(String),
    Object { text: Option<String> },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStatusRaw {
    pub version: Option<ServerVersionInfo>,
    pub players: Option<ServerPlayersInfo>,
    pub description: Option<ServerDescription>,
}

#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub host: String,
    pub port: u16,
    pub motd: String,
    pub version: String,
    pub online_players: i32,
    pub max_players: i32,
    pub ping_ms: u64,
    pub is_online: bool,
}

pub struct ServerPinger;

impl ServerPinger {
    pub async fn ping(host: &str, port: u16) -> Result<ServerStatus> {
        let addr = format!("{host}:{port}");
        let start = Instant::now();

        // 3 second timeout for ping
        let status_res = timeout(Duration::from_secs(3), async {
            let mut stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| LauncherError::Network(format!("Connection to {addr} failed: {e}")))?;

            // 1. Handshake packet
            let mut handshake_payload = Vec::new();
            write_varint(&mut handshake_payload, 0x00); // Packet ID: Handshake
            write_varint(&mut handshake_payload, 765); // Protocol version
            write_string(&mut handshake_payload, host);
            handshake_payload.extend_from_slice(&port.to_be_bytes());
            write_varint(&mut handshake_payload, 1); // Next state: Status

            let mut handshake_packet = Vec::new();
            write_varint(&mut handshake_packet, handshake_payload.len() as i32);
            handshake_packet.extend_from_slice(&handshake_payload);

            stream
                .write_all(&handshake_packet)
                .await
                .map_err(|e| LauncherError::Network(format!("Write handshake: {e}")))?;

            // 2. Status Request packet
            let mut status_req = Vec::new();
            write_varint(&mut status_req, 1);
            write_varint(&mut status_req, 0x00);

            stream
                .write_all(&status_req)
                .await
                .map_err(|e| LauncherError::Network(format!("Write status req: {e}")))?;

            // 3. Read Status Response
            let _packet_len = read_varint(&mut stream)
                .await
                .map_err(|e| LauncherError::Network(format!("Read length: {e}")))?;
            let packet_id = read_varint(&mut stream)
                .await
                .map_err(|e| LauncherError::Network(format!("Read packet ID: {e}")))?;

            if packet_id != 0x00 {
                return Err(LauncherError::Network("Invalid SLP packet ID".into()));
            }

            let json_len = read_varint(&mut stream)
                .await
                .map_err(|e| LauncherError::Network(format!("Read string len: {e}")))? as usize;

            if json_len > 32768 {
                return Err(LauncherError::Network("Server response too large".into()));
            }

            let mut json_bytes = vec![0u8; json_len];
            stream
                .read_exact(&mut json_bytes)
                .await
                .map_err(|e| LauncherError::Network(format!("Read body: {e}")))?;

            let json_str = String::from_utf8_lossy(&json_bytes);
            let raw: ServerStatusRaw = serde_json::from_str(&json_str).map_err(LauncherError::Json)?;

            Ok(raw)
        })
        .await;

        let ping_ms = start.elapsed().as_millis() as u64;

        match status_res {
            Ok(Ok(raw)) => {
                let motd = match raw.description {
                    Some(ServerDescription::Text(t)) => strip_minecraft_color_codes(&t),
                    Some(ServerDescription::Object { text }) => {
                        strip_minecraft_color_codes(&text.unwrap_or_default())
                    }
                    None => "Minecraft Server".to_string(),
                };

                let version = raw
                    .version
                    .and_then(|v| v.name)
                    .unwrap_or_else(|| "1.20.4".to_string());

                let (online, max) = match raw.players {
                    Some(p) => (p.online.unwrap_or(0), p.max.unwrap_or(0)),
                    None => (0, 0),
                };

                Ok(ServerStatus {
                    host: host.to_string(),
                    port,
                    motd,
                    version,
                    online_players: online,
                    max_players: max,
                    ping_ms,
                    is_online: true,
                })
            }
            _ => Ok(ServerStatus {
                host: host.to_string(),
                port,
                motd: "Сервер недоступен".to_string(),
                version: "--".to_string(),
                online_players: 0,
                max_players: 0,
                ping_ms: 999,
                is_online: false,
            }),
        }
    }
}

fn write_varint(buf: &mut Vec<u8>, mut val: i32) {
    loop {
        if (val & !0x7F) == 0 {
            buf.push(val as u8);
            return;
        }
        buf.push(((val & 0x7F) | 0x80) as u8);
        val >>= 7;
    }
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

async fn read_varint<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<i32> {
    let mut num_read = 0;
    let mut result = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).await?;
        let value = (byte[0] & 0x7F) as i32;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "VarInt too big",
            ));
        }
        if (byte[0] & 0x80) == 0 {
            break;
        }
    }
    Ok(result)
}

fn strip_minecraft_color_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' || c == '&' {
            if let Some(&next) = chars.peek() {
                if next.is_ascii_hexdigit()
                    || matches!(next, 'k' | 'l' | 'm' | 'n' | 'o' | 'r' | 'K' | 'L' | 'M' | 'N' | 'O' | 'R')
                {
                    chars.next();
                    continue;
                }
            }
        }
        out.push(c);
    }
    out.trim().to_string()
}
