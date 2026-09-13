use futures_util::StreamExt;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, Semaphore};
use amc_core::error::{LauncherError, Result};
use crate::progress::{DownloadProgress, ProgressTracker};
use crate::verifier::{file_sha1, sha1_hex};

#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub url: String,
    pub destination: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

impl DownloadItem {
    pub fn new(url: impl Into<String>, destination: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            destination: destination.into(),
            sha1: None,
            size: None,
        }
    }

    pub fn with_sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into());
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
}

pub struct DownloadEngine {
    client: Client,
    max_concurrency: usize,
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new(16)
    }
}

impl DownloadEngine {
    pub fn new(max_concurrency: usize) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(32)
            .build()
            .unwrap_or_default();

        Self {
            client,
            max_concurrency: max_concurrency.max(1),
        }
    }

    pub async fn download_one(
        &self,
        item: &DownloadItem,
        tracker: Option<&ProgressTracker>,
    ) -> Result<()> {
        let filename = item
            .destination
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "download".to_string());

        // Check if file already exists with valid checksum
        if item.destination.is_file() {
            if let Some(expected) = &item.sha1 {
                if let Ok(actual) = file_sha1(&item.destination).await {
                    if actual.eq_ignore_ascii_case(expected) {
                        if let Some(t) = tracker {
                            if let Some(size) = item.size {
                                t.on_bytes(size, &filename);
                            }
                            t.on_file_completed();
                        }
                        return Ok(());
                    }
                }
            } else if item.size.is_some() {
                if let Ok(meta) = fs::metadata(&item.destination).await {
                    if meta.len() == item.size.unwrap() {
                        if let Some(t) = tracker {
                            t.on_bytes(meta.len(), &filename);
                            t.on_file_completed();
                        }
                        return Ok(());
                    }
                }
            }
        }

        if let Some(parent) = item.destination.parent() {
            fs::create_dir_all(parent).await.map_err(|e| LauncherError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let mut retries = 3;
        while retries > 0 {
            match self.execute_download(item, tracker, &filename).await {
                Ok(()) => {
                    if let Some(t) = tracker {
                        t.on_file_completed();
                    }
                    return Ok(());
                }
                Err(err) => {
                    retries -= 1;
                    if retries == 0 {
                        return Err(err);
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        Err(LauncherError::Network(format!("Failed to download {}", item.url)))
    }

    async fn execute_download(
        &self,
        item: &DownloadItem,
        tracker: Option<&ProgressTracker>,
        filename: &str,
    ) -> Result<()> {
        let response = self
            .client
            .get(&item.url)
            .send()
            .await
            .map_err(|e| LauncherError::Network(format!("Request to {} failed: {e}", item.url)))?;

        if !response.status().is_success() {
            return Err(LauncherError::Network(format!(
                "HTTP {} for {}",
                response.status(),
                item.url
            )));
        }

        let tmp_path = item.destination.with_extension("amctmp");
        let mut file = File::create(&tmp_path).await.map_err(|e| LauncherError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;

        let mut stream = response.bytes_stream();
        let mut downloaded_bytes = Vec::new();
        let compute_sha1 = item.sha1.is_some();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| LauncherError::Network(format!("Chunk error: {e}")))?;
            file.write_all(&chunk).await.map_err(|e| LauncherError::Io {
                path: tmp_path.clone(),
                source: e,
            })?;

            if compute_sha1 {
                downloaded_bytes.extend_from_slice(&chunk);
            }

            if let Some(t) = tracker {
                t.on_bytes(chunk.len() as u64, filename);
            }
        }

        file.flush().await.map_err(|e| LauncherError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;
        drop(file);

        // Verify SHA1
        if let Some(expected) = &item.sha1 {
            let actual = sha1_hex(&downloaded_bytes);
            if !actual.eq_ignore_ascii_case(expected) {
                let _ = fs::remove_file(&tmp_path).await;
                return Err(LauncherError::ChecksumMismatch {
                    file: filename.to_string(),
                    expected: expected.clone(),
                    actual,
                });
            }
        }

        // Replace destination file atomically
        if item.destination.exists() {
            let _ = fs::remove_file(&item.destination).await;
        }

        fs::rename(&tmp_path, &item.destination)
            .await
            .map_err(|e| LauncherError::Io {
                path: item.destination.clone(),
                source: e,
            })?;

        Ok(())
    }

    pub async fn download_all(
        &self,
        items: Vec<DownloadItem>,
    ) -> Result<watch::Receiver<DownloadProgress>> {
        let total_items = items.len();
        let total_bytes: u64 = items.iter().filter_map(|i| i.size).sum();

        let (tracker, rx) = ProgressTracker::new(total_items, total_bytes);
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));

        let mut handles = Vec::with_capacity(total_items);

        for item in items {
            let sem = semaphore.clone();
            let trk = tracker.clone();
            let client = self.client.clone();
            let engine = DownloadEngine {
                client,
                max_concurrency: self.max_concurrency,
            };

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                engine.download_one(&item, Some(&trk)).await
            }));
        }

        for handle in handles {
            let res = handle
                .await
                .map_err(|e| LauncherError::Custom(format!("Download task panicked: {e}")))?;
            res?;
        }

        Ok(rx)
    }
}
