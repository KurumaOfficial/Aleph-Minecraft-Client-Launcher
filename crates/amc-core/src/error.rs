use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("IO error: {0}")]
    IoRaw(#[from] std::io::Error),

    #[error("Serialization / JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network request error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Version error: {0}")]
    Version(String),

    #[error("Loader error: {0}")]
    Loader(String),

    #[error("Download integrity check failed for {file}: expected sha1 {expected}, calculated {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("Java runtime error: {0}")]
    Java(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Process launch error: {0}")]
    Launch(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, LauncherError>;
