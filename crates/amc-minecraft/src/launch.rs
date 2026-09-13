use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use amc_auth::AuthSession;
use amc_core::error::{LauncherError, Result};
use amc_core::types::LaunchOptions;
use crate::args::ArgumentBuilder;
use crate::version::VersionDetails;

#[derive(Debug, Clone)]
pub enum GameEvent {
    Started { pid: u32 },
    LogLine(String),
    Exited { code: Option<i32> },
    Crashed { message: String },
}

pub struct MinecraftLauncher;

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

        let jvm_args = ArgumentBuilder::build_jvm_args(
            options,
            natives_dir,
            &classpath,
            details,
            &features,
        );

        let game_args = ArgumentBuilder::build_game_args(
            options,
            game_dir,
            assets_dir,
            details,
            session,
            &features,
        );

        let mut cmd = Command::new(java_bin);
        cmd.current_dir(game_dir);
        cmd.args(&jvm_args);
        cmd.arg(&details.main_class);
        cmd.args(&game_args);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(windows)]
        {
            // Creation flags on Windows: 0x08000000 = CREATE_NO_WINDOW if desired, or default
        }

        tracing::info!("Spawning Minecraft with {} arguments...", jvm_args.len() + game_args.len() + 1);

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
                    let _ = tx_err.send(GameEvent::LogLine(format!("[ERROR] {line}"))).await;
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
