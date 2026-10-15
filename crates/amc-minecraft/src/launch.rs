use crate::args::ArgumentBuilder;
use crate::version::VersionDetails;
use amc_auth::AuthSession;
use amc_core::error::{LauncherError, Result};
use amc_core::types::LaunchOptions;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum GameEvent {
    Started {
        pid: u32,
    },
    LogLine(String),
    Exited {
        code: Option<i32>,
    },
    Crashed {
        message: String,
    },
    /// User-cancelled operation (e.g. download cancel). Not a crash: no
    /// diagnostics, no playtime accounting — just back to idle.
    Cancelled,
    /// User-paused download (P3). The overlay stays open with Resume;
    /// resuming re-runs the launch, picking up partial files via Range.
    DownloadPaused,
}

pub struct MinecraftLauncher;

/// Extra creation flags for the game process. Windows-only for now,
/// see the call site for the Unix story.
#[cfg(windows)]
pub fn game_creation_flags() -> u32 {
    // ABOVE_NORMAL_PRIORITY_CLASS.
    0x0000_8000
}

#[cfg(not(windows))]
pub fn game_creation_flags() -> u32 {
    0
}

impl MinecraftLauncher {
    pub async fn launch(
        java_bin: &Path,
        game_dir: &Path,
        assets_dir: &Path,
        natives_dir: &Path,
        libraries: &[PathBuf],
        client_jar: &Path,
        details: &VersionDetails,
        session: &AuthSession,
        options: &LaunchOptions,
    ) -> Result<mpsc::Receiver<GameEvent>> {
        if !java_bin.is_file() {
            return Err(LauncherError::Java(format!(
                "Java binary not found at {}",
                java_bin.display()
            )));
        }

        let features: HashMap<String, bool> = HashMap::new();
        let classpath = ArgumentBuilder::build_classpath(libraries, client_jar);

        let jvm_args =
            ArgumentBuilder::build_jvm_args(options, natives_dir, &classpath, details, &features);

        let game_args = ArgumentBuilder::build_game_args(
            options, game_dir, assets_dir, details, session, &features,
        );

        let mut cmd = Command::new(java_bin);
        cmd.current_dir(game_dir);
        cmd.args(&jvm_args);
        cmd.arg(&details.main_class);
        cmd.args(&game_args);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // CONCEPT "автоматически повышает приоритет процесса игры":
        // above-normal on Windows (HIGH would starve the launcher and
        // input on 2-core machines). Unix elevation needs a design of
        // its own (setpriority/nice differ per distro) — tracked openly,
        // the game simply launches at normal priority there for now.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(game_creation_flags());
        }

        tracing::info!(
            "Spawning Minecraft with {} arguments...",
            jvm_args.len() + game_args.len() + 1
        );

        let mut child = cmd
            .spawn()
            .map_err(|e| LauncherError::Launch(format!("Failed to spawn Minecraft: {e}")))?;

        let pid = child.id().unwrap_or(0);
        let (tx, rx) = mpsc::channel(256);

        let _ = tx.send(GameEvent::Started { pid }).await;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Stdout reader
        if let Some(out) = stdout {
            let tx_out = tx.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = tx_out.send(GameEvent::LogLine(line)).await;
                }
            });
        }

        // Stderr reader
        if let Some(err) = stderr {
            let tx_err = tx.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = tx_err
                        .send(GameEvent::LogLine(format!("[ERROR] {line}")))
                        .await;
                }
            });
        }

        // Process exit watcher
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let code = status.code();
                    if status.success() {
                        let _ = tx.send(GameEvent::Exited { code }).await;
                    } else {
                        let _ = tx
                            .send(GameEvent::Crashed {
                                message: format!("Process exited with status {:?}", code),
                            })
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(GameEvent::Crashed {
                            message: format!("Process wait error: {e}"),
                        })
                        .await;
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation_flags() {
        // Above-normal on Windows so the game wins CPU contention;
        // zero (no override) anywhere else by explicit design.
        #[cfg(windows)]
        assert_eq!(game_creation_flags(), 0x0000_8000);
        #[cfg(not(windows))]
        assert_eq!(game_creation_flags(), 0);
    }
}
