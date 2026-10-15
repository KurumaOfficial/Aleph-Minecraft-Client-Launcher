use amc_core::Language;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::watch;

#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub current_file: String,
    pub completed_files: usize,
    pub total_files: usize,
    pub eta_seconds: Option<u64>,
}

impl DownloadProgress {
    /// 0.0..=1.0 completion by bytes, falling back to files.
    #[must_use]
    pub fn ratio(&self) -> f32 {
        if self.total_bytes > 0 {
            (self.downloaded_bytes as f32 / self.total_bytes as f32).clamp(0.0, 1.0)
        } else if self.total_files > 0 {
            (self.completed_files as f32 / self.total_files as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Percentage (0..=100) for progress bars.
    #[must_use]
    pub fn percent(&self) -> f32 {
        self.ratio() * 100.0
    }

    /// Localized speed (`2.0 MB/s` / `2,0 МБ/с`).
    #[must_use]
    pub fn formatted_speed(&self, lang: &Language) -> String {
        lang.speed_value(self.speed_bytes_per_sec)
    }

    /// Localized `downloaded / total`.
    #[must_use]
    pub fn formatted_bytes(&self, lang: &Language) -> String {
        if self.total_bytes > 0 {
            format!(
                "{} / {}",
                lang.disk_size(self.downloaded_bytes),
                lang.disk_size(self.total_bytes)
            )
        } else {
            lang.disk_size(self.downloaded_bytes)
        }
    }

    /// Localized ETA (`--` when unknown).
    #[must_use]
    pub fn formatted_eta(&self, lang: &Language) -> String {
        match self.eta_seconds {
            Some(secs) => lang.eta_text(secs),
            None => "--".to_string(),
        }
    }
}

pub struct ProgressTracker {
    total_bytes: AtomicU64,
    downloaded_bytes: AtomicU64,
    completed_files: AtomicUsize,
    total_files: usize,
    start_time: Instant,
    last_check_time: parking_lot::Mutex<Instant>,
    last_check_bytes: AtomicU64,
    sender: watch::Sender<DownloadProgress>,
}

impl ProgressTracker {
    /// Channel-backed tracker; send side is cheap to clone via `Arc`.
    pub fn new(
        total_files: usize,
        total_bytes: u64,
    ) -> (Arc<Self>, watch::Receiver<DownloadProgress>) {
        let initial = DownloadProgress {
            total_bytes,
            downloaded_bytes: 0,
            speed_bytes_per_sec: 0.0,
            current_file: String::new(),
            completed_files: 0,
            total_files,
            eta_seconds: None,
        };

        let (sender, receiver) = watch::channel(initial);
        let now = Instant::now();

        let tracker = Arc::new(Self {
            total_bytes: AtomicU64::new(total_bytes),
            downloaded_bytes: AtomicU64::new(0),
            completed_files: AtomicUsize::new(0),
            total_files,
            start_time: now,
            last_check_time: parking_lot::Mutex::new(now),
            last_check_bytes: AtomicU64::new(0),
            sender,
        });

        (tracker, receiver)
    }

    /// Grow the total when the manifest size was unknown up front.
    pub fn add_total_bytes(&self, bytes: u64) {
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record `bytes` for `current_file`; emits speed/ETA throttled.
    pub fn on_bytes(&self, bytes: u64, current_file: &str) {
        let downloaded = self.downloaded_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes;
        let total = self.total_bytes.load(Ordering::Relaxed);

        let mut last_time = self.last_check_time.lock();
        let elapsed = last_time.elapsed().as_secs_f64();

        // Update speed and ETA at intervals of >= 200ms
        let (speed, eta) = if elapsed >= 0.2 {
            let last_bytes = self.last_check_bytes.swap(downloaded, Ordering::Relaxed);
            let bytes_diff = downloaded.saturating_sub(last_bytes);
            let current_speed = bytes_diff as f64 / elapsed;
            *last_time = Instant::now();

            let eta = if current_speed > 0.0 && total > downloaded {
                Some(((total - downloaded) as f64 / current_speed) as u64)
            } else {
                None
            };
            (current_speed, eta)
        } else {
            let overall_elapsed = self.start_time.elapsed().as_secs_f64();
            let avg_speed = if overall_elapsed > 0.0 {
                downloaded as f64 / overall_elapsed
            } else {
                0.0
            };
            let eta = if avg_speed > 0.0 && total > downloaded {
                Some(((total - downloaded) as f64 / avg_speed) as u64)
            } else {
                None
            };
            (avg_speed, eta)
        };

        let progress = DownloadProgress {
            total_bytes: total,
            downloaded_bytes: downloaded,
            speed_bytes_per_sec: speed,
            current_file: current_file.to_string(),
            completed_files: self.completed_files.load(Ordering::Relaxed),
            total_files: self.total_files,
            eta_seconds: eta,
        };

        let _ = self.sender.send(progress);
    }

    /// Mark one file done (drives file-count fallback progress).
    pub fn on_file_completed(&self) {
        let completed = self.completed_files.fetch_add(1, Ordering::Relaxed) + 1;
        self.sender.send_modify(|p| {
            p.completed_files = completed;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(downloaded: u64, total: u64, speed: f64, eta: Option<u64>) -> DownloadProgress {
        DownloadProgress {
            total_bytes: total,
            downloaded_bytes: downloaded,
            speed_bytes_per_sec: speed,
            current_file: String::new(),
            completed_files: 0,
            total_files: 0,
            eta_seconds: eta,
        }
    }

    #[test]
    fn test_ratio_and_percent() {
        let p = progress(25, 100, 0.0, None);
        assert!((p.ratio() - 0.25).abs() < 1e-6);
        assert!((p.percent() - 25.0).abs() < 1e-4);
        assert_eq!(progress(0, 0, 0.0, None).ratio(), 0.0);
        assert_eq!(progress(200, 100, 0.0, None).ratio(), 1.0);
    }

    #[test]
    fn test_formatted_units_follow_language() {
        let en = Language::English;
        let ru = Language::Russian;
        let p = progress(1_048_576, 2_097_152, 2_097_152.0, Some(90));
        assert_eq!(p.formatted_speed(&en), "2.0 MB/s");
        assert_eq!(p.formatted_speed(&ru), "2,0 МБ/с");
        assert_eq!(p.formatted_bytes(&en), "1.0 MB / 2.0 MB");
        assert_eq!(p.formatted_bytes(&ru), "1,0 МБ / 2,0 МБ");
        assert_eq!(p.formatted_eta(&en), "1 min 30 sec");
        assert_eq!(p.formatted_eta(&ru), "1 мин 30 сек");
        assert_eq!(progress(0, 0, 512.0, None).formatted_eta(&en), "--");
    }

    #[test]
    fn test_tracker_mutators() {
        let (tracker, rx) = ProgressTracker::new(2, 100);
        tracker.add_total_bytes(100);
        tracker.on_bytes(50, "a.jar");
        tracker.on_file_completed();
        let snap = rx.borrow().clone();
        assert_eq!(snap.total_bytes, 200);
        assert_eq!(snap.downloaded_bytes, 50);
        assert_eq!(snap.completed_files, 1);
        assert_eq!(snap.total_files, 2);
        assert_eq!(snap.current_file, "a.jar");
    }
}
