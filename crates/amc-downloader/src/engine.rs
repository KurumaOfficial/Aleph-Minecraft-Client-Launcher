use crate::progress::{DownloadProgress, ProgressTracker};
use crate::verifier::file_sha1;
use amc_core::error::{LauncherError, Result};
use futures_util::StreamExt;
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, Semaphore};

/// Cooperative control token for downloads. Cheap to clone (shared flags):
/// the UI cancels or pauses the current operation, workers observe the flags
/// between chunks and before retries. Pause preserves the partial `.amctmp`
/// file for Range-resume; cancel deletes it.
#[derive(Debug, Clone, Default)]
pub struct DownloadCancel {
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
}

impl DownloadCancel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn pause(&self) {
        self.pause.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
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

/// `Range` header value resuming after `have` bytes.
fn range_value(have: u64) -> String {
    format!("bytes={have}-")
}

/// Stream-hash an existing partial `.amctmp` file so a resumed download
/// continues with a correct running hash. A missing or unreadable file
/// restarts from scratch — never fails the download.
pub(crate) async fn hash_file_prefix(path: &Path) -> (Sha1, u64) {
    use tokio::io::AsyncReadExt;
    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(_) => return (Sha1::new(), 0),
    };
    let mut hasher = Sha1::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        match file.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            Err(_) => return (Sha1::new(), 0),
        }
    }
    (hasher, total)
}

/// Global download speed limiter (token bucket). Shared across all workers
/// so the cap holds for the whole operation, not per connection.
#[derive(Debug)]
struct ThrottleState {
    tokens: f64,
    last: Instant,
}

#[derive(Debug)]
pub struct SpeedLimiter {
    limit_bps: AtomicU64,
    state: Mutex<ThrottleState>,
}

impl SpeedLimiter {
    fn new() -> Self {
        Self {
            limit_bps: AtomicU64::new(0),
            state: Mutex::new(ThrottleState {
                tokens: 0.0,
                last: Instant::now(),
            }),
        }
    }

    /// Cap in bytes per second. `None` (or 0) disables limiting.
    pub fn set_limit_bps(&self, limit: Option<u64>) {
        self.limit_bps.store(limit.unwrap_or(0), Ordering::Relaxed);
    }

    async fn take(&self, n: u64) {
        let limit = self.limit_bps.load(Ordering::Relaxed);
        if limit == 0 || n == 0 {
            return;
        }
        let delay_secs = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            consume(&mut state, Instant::now(), n, limit)
        };
        if delay_secs > 0.0 {
            tokio::time::sleep(Duration::from_secs_f64(delay_secs)).await;
        }
    }
}

/// Pure bucket step: returns seconds to wait before consuming `n` bytes.
/// Fully unit-tested; the async wrapper only sleeps.
fn consume(state: &mut ThrottleState, now: Instant, n: u64, limit_bps: u64) -> f64 {
    let elapsed = now.saturating_duration_since(state.last).as_secs_f64();
    state.last = now;
    // One second of burst, at least 256 KiB so small files never stall.
    let capacity = (limit_bps as f64).max(262_144.0);
    state.tokens = (state.tokens + elapsed * limit_bps as f64).min(capacity);
    if state.tokens >= n as f64 {
        state.tokens -= n as f64;
        0.0
    } else {
        let wait = (n as f64 - state.tokens) / limit_bps as f64;
        state.tokens = 0.0;
        wait
    }
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
    throttle: Arc<SpeedLimiter>,
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
            throttle: Arc::new(SpeedLimiter::new()),
        }
    }

    /// Global speed cap in KiB/s (`None` = unlimited). Shared across all
    /// workers and all downloads through this engine. Applied by the app
    /// from settings on every launch.
    pub fn set_speed_limit_kbps(&self, kbps: Option<u64>) {
        self.throttle
            .set_limit_bps(kbps.map(|k| k.saturating_mul(1024)));
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
        if cancel.is_paused() {
            return Err(LauncherError::Paused);
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
                    if cancel.is_paused() {
                        return Err(LauncherError::Paused);
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
        let tmp_path = item.destination.with_extension("amctmp");

        // Resume: hash the existing partial file, then ask for the rest.
        let (mut hasher, have) = hash_file_prefix(&tmp_path).await;
        let mut request = self.client.get(&item.url);
        if have > 0 {
            request = request.header("Range", range_value(have));
        }
        let response = request.send().await.map_err(|e| {
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

        // 206 = resume accepted (append, keep the seeded hash); anything else
        // (200, 416, …) restarts the file from scratch.
        let append = have > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        if !append {
            hasher = Sha1::new();
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&tmp_path)
            .await
            .map_err(|e| {
                retryable(LauncherError::Io {
                    path: tmp_path.clone(),
                    source: e,
                })
            })?;

        let mut stream = response.bytes_stream();

        while let Some(chunk_res) = stream.next().await {
            if cancel.is_cancelled() {
                drop(file);
                let _ = fs::remove_file(&tmp_path).await;
                return Err(fatal(LauncherError::Cancelled));
            }
            if cancel.is_paused() {
                drop(file);
                return Err(fatal(LauncherError::Paused));
            }
            let chunk = chunk_res
                .map_err(|e| retryable(LauncherError::Network(format!("Chunk error: {e}"))))?;
            self.throttle.take(chunk.len() as u64).await;
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
                throttle: self.throttle.clone(),
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

    #[tokio::test]
    async fn test_prepaused_download_returns_paused() {
        let engine = DownloadEngine::new(1);
        let cancel = DownloadCancel::new();
        cancel.pause();
        assert!(cancel.is_paused());
        cancel.resume();
        assert!(!cancel.is_paused());
        cancel.pause();
        let item = DownloadItem::new(
            "http://127.0.0.1:1/amc-pause-probe.jar",
            std::env::temp_dir().join("amc-pause-probe.jar"),
        );
        let err = engine.download_one(&item, None, cancel).await.unwrap_err();
        assert!(matches!(err, LauncherError::Paused));
    }

    #[test]
    fn test_range_value() {
        assert_eq!(range_value(0), "bytes=0-");
        assert_eq!(range_value(123456), "bytes=123456-");
    }

    #[test]
    fn test_throttle_consume_math() {
        let now = Instant::now();
        // Full bucket: instant.
        let mut state = ThrottleState {
            tokens: 10_000.0,
            last: now,
        };
        assert_eq!(consume(&mut state, now, 1000, 1000), 0.0);
        assert_eq!(state.tokens, 9000.0);
        // Empty bucket: wait the deficit.
        let mut state = ThrottleState {
            tokens: 0.0,
            last: now,
        };
        let wait = consume(&mut state, now, 2000, 1000);
        assert!((wait - 2.0).abs() < 1e-9);
        assert_eq!(state.tokens, 0.0);
        // Refill over idle time, capped at one second of burst.
        let later = now + Duration::from_secs(10);
        let mut state = ThrottleState {
            tokens: 0.0,
            last: now,
        };
        assert_eq!(consume(&mut state, later, 500, 1000), 0.0);
        assert_eq!(state.tokens, 9500.0);
    }
}
