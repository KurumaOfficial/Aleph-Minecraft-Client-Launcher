use crate::progress::{DownloadProgress, ProgressTracker};
use crate::verifier::file_sha1;
use amc_core::error::{LauncherError, Result};
use futures_util::StreamExt;
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, Semaphore};

/// Cooperative cancellation token for downloads. Cheap to clone (shared
/// flag): the UI cancels the current operation, workers observe the flag
/// between chunks and before retries.
#[derive(Debug, Clone, Default)]
pub struct DownloadCancel {
    flag: Arc<AtomicBool>,
}

impl DownloadCancel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

/// Backoff before retry attempt `attempt` (0-based). `None` means give up.
/// Three tries total: immediate, +500ms, +1500ms.
pub fn retry_delay(attempt: u32) -> Option<Duration> {
    match attempt {
        0 => Some(Duration::from_millis(500)),
        1 => Some(Duration::from_millis(1500)),
        _ => None,
    }
}

/// HTTP statuses worth a retry: rate limits, timeouts, server errors.
/// Anything else (notably other 4xx) fails immediately instead of burning
/// retries on a request that will never succeed.
pub fn should_retry_http(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status.is_server_error()
}

/// Internal attempt outcome: transient failures sleep and retry while
/// attempts remain, anything else (including cancellation) returns at once.
enum AttemptError {
    Retryable(LauncherError),
    Fatal(LauncherError),
}

fn retryable(err: LauncherError) -> AttemptError {
    AttemptError::Retryable(err)
}

fn fatal(err: LauncherError) -> AttemptError {
    AttemptError::Fatal(err)
}

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
        cancel: DownloadCancel,
    ) -> Result<()> {
        if cancel.is_cancelled() {
            return Err(LauncherError::Cancelled);
        }

        let filename = item
            .destination
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "download".to_string());

        // A file already on disk counts only if its SHA-1 matches. A bare
        // size match is accepted only when no hash is known — a corrupt file
        // of the right size must never pass.
        if item.destination.is_file() {
            if let Some(expected) = &item.sha1 {
                if let Ok(actual) = file_sha1(&item.destination).await {
                    if actual.eq_ignore_ascii_case(expected) {
                        if let Some(t) = tracker {
                            if let Ok(meta) = fs::metadata(&item.destination).await {
                                t.on_bytes(meta.len(), &filename);
                            }
                            t.on_file_completed();
                        }
                        return Ok(());
                    }
                }
            } else if let Some(expected_size) = item.size {
                if let Ok(meta) = fs::metadata(&item.destination).await {
                    if meta.len() == expected_size {
                        if let Some(t) = tracker {
                            t.on_bytes(expected_size, &filename);
                            t.on_file_completed();
                        }
                        return Ok(());
                    }
                }
            }
        }

        if let Some(parent) = item.destination.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| LauncherError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
        }

        let mut attempt = 0u32;
        loop {
            match self
                .execute_download(item, tracker, &filename, &cancel)
                .await
            {
                Ok(()) => {
                    if let Some(t) = tracker {
                        t.on_file_completed();
                    }
                    return Ok(());
                }
                Err(AttemptError::Fatal(err)) => return Err(err),
                Err(AttemptError::Retryable(err)) => {
                    if cancel.is_cancelled() {
                        return Err(LauncherError::Cancelled);
                    }
                    match retry_delay(attempt) {
                        Some(delay) => {
                            attempt += 1;
                            tokio::time::sleep(delay).await;
                        }
                        None => return Err(err),
                    }
                }
            }
        }
    }

    async fn execute_download(
        &self,
        item: &DownloadItem,
        tracker: Option<&ProgressTracker>,
        filename: &str,
        cancel: &DownloadCancel,
    ) -> std::result::Result<(), AttemptError> {
        let response = self.client.get(&item.url).send().await.map_err(|e| {
            retryable(LauncherError::Network(format!(
                "Request to {} failed: {e}",
                item.url
            )))
        })?;

        if !response.status().is_success() {
            let err =
                LauncherError::Network(format!("HTTP {} for {}", response.status(), item.url));
            if should_retry_http(response.status()) {
                return Err(retryable(err));
            }
            return Err(fatal(err));
        }

        let tmp_path = item.destination.with_extension("amctmp");
        let mut file = File::create(&tmp_path).await.map_err(|e| {
            retryable(LauncherError::Io {
                path: tmp_path.clone(),
                source: e,
            })
        })?;

        // Streamed SHA-1: files of any size hash with O(1) memory.
        let mut hasher = Sha1::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_res) = stream.next().await {
            if cancel.is_cancelled() {
                drop(file);
                let _ = fs::remove_file(&tmp_path).await;
                return Err(fatal(LauncherError::Cancelled));
            }
            let chunk = chunk_res
                .map_err(|e| retryable(LauncherError::Network(format!("Chunk error: {e}"))))?;
            file.write_all(&chunk).await.map_err(|e| {
                retryable(LauncherError::Io {
                    path: tmp_path.clone(),
                    source: e,
                })
            })?;

            hasher.update(&chunk);

            if let Some(t) = tracker {
                t.on_bytes(chunk.len() as u64, filename);
            }
        }

        file.flush().await.map_err(|e| {
            retryable(LauncherError::Io {
                path: tmp_path.clone(),
                source: e,
            })
        })?;
        drop(file);

        // Verify SHA1
        if let Some(expected) = &item.sha1 {
            let digest = hasher.finalize();
            let mut actual = String::with_capacity(40);
            for byte in digest.iter() {
                let _ = write!(actual, "{byte:02x}");
            }
            if !actual.eq_ignore_ascii_case(expected) {
                let _ = fs::remove_file(&tmp_path).await;
                return Err(retryable(LauncherError::ChecksumMismatch {
                    file: filename.to_string(),
                    expected: expected.clone(),
                    actual,
                }));
            }
        }

        // Replace destination file atomically
        if item.destination.exists() {
            let _ = fs::remove_file(&item.destination).await;
        }

        fs::rename(&tmp_path, &item.destination)
            .await
            .map_err(|e| {
                fatal(LauncherError::Io {
                    path: item.destination.clone(),
                    source: e,
                })
            })?;

        Ok(())
    }

    pub fn create_tracker(
        &self,
        items: &[DownloadItem],
    ) -> (Arc<ProgressTracker>, watch::Receiver<DownloadProgress>) {
        let total_items = items.len();
        let total_bytes: u64 = items.iter().filter_map(|i| i.size).sum();
        ProgressTracker::new(total_items, total_bytes)
    }

    pub async fn download_all_with_progress(
        &self,
        items: Vec<DownloadItem>,
        tracker: Option<Arc<ProgressTracker>>,
        cancel: DownloadCancel,
    ) -> Result<()> {
        let total_items = items.len();
        if total_items == 0 {
            return Ok(());
        }

        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let mut handles = Vec::with_capacity(total_items);

        for item in items {
            let sem = semaphore.clone();
            let trk = tracker.clone();
            let tok = cancel.clone();
            let client = self.client.clone();
            let engine = DownloadEngine {
                client,
                max_concurrency: self.max_concurrency,
            };

            handles.push(tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| LauncherError::Custom(format!("Semaphore error: {e}")))?;
                engine.download_one(&item, trk.as_deref(), tok).await
            }));
        }

        for handle in handles {
            let res = handle
                .await
                .map_err(|e| LauncherError::Custom(format!("Download task panicked: {e}")))?;
            res?;
        }

        Ok(())
    }

    pub async fn download_all(
        &self,
        items: Vec<DownloadItem>,
        cancel: DownloadCancel,
    ) -> Result<()> {
        self.download_all_with_progress(items, None, cancel).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_delay_schedule() {
        assert_eq!(retry_delay(0), Some(Duration::from_millis(500)));
        assert_eq!(retry_delay(1), Some(Duration::from_millis(1500)));
        assert_eq!(retry_delay(2), None);
        assert_eq!(retry_delay(99), None);
    }

    #[test]
    fn test_should_retry_http() {
        use reqwest::StatusCode;
        for code in [500, 502, 503, 504, 408, 429] {
            assert!(
                should_retry_http(StatusCode::from_u16(code).unwrap()),
                "{code} should retry"
            );
        }
        for code in [200, 400, 401, 403, 404, 422] {
            assert!(
                !should_retry_http(StatusCode::from_u16(code).unwrap()),
                "{code} should fail immediately"
            );
        }
    }

    #[test]
    fn test_cancel_token() {
        let token = DownloadCancel::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
        // Clones share the flag.
        assert!(token.clone().is_cancelled());
        assert!(!DownloadCancel::default().is_cancelled());
    }

    #[tokio::test]
    async fn test_precancelled_download_returns_cancelled() {
        let engine = DownloadEngine::new(1);
        let cancel = DownloadCancel::new();
        cancel.cancel();
        // Unroutable destination: with the pre-set flag no request may run.
        let item = DownloadItem::new(
            "http://127.0.0.1:1/amc-cancel-probe.jar",
            std::env::temp_dir().join("amc-cancel-probe.jar"),
        );
        let err = engine.download_one(&item, None, cancel).await.unwrap_err();
        assert!(matches!(err, LauncherError::Cancelled));
    }
}
