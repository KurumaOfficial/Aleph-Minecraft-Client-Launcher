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
    pub fn ratio(&self) -> f32 {
        if self.total_bytes > 0 {
            (self.downloaded_bytes as f32 / self.total_bytes as f32).clamp(0.0, 1.0)
        } else if self.total_files > 0 {
            (self.completed_files as f32 / self.total_files as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn percent(&self) -> f32 {
        self.ratio() * 100.0
    }

    pub fn formatted_speed(&self) -> String {
        let speed = self.speed_bytes_per_sec;
        if speed >= 1024.0 * 1024.0 {
            format!("{:.2} МБ/с", speed / (1024.0 * 1024.0))
        } else if speed >= 1024.0 {
            format!("{:.1} КБ/с", speed / 1024.0)
        } else {
            format!("{:.0} Б/с", speed)
        }
    }

    pub fn formatted_bytes(&self) -> String {
        let dl = self.downloaded_bytes as f64 / (1024.0 * 1024.0);
        let total = self.total_bytes as f64 / (1024.0 * 1024.0);
        if self.total_bytes > 0 {
            format!("{:.1} / {:.1} МБ", dl, total)
        } else {
            format!("{:.1} МБ", dl)
        }
    }

    pub fn formatted_eta(&self) -> String {
        if let Some(secs) = self.eta_seconds {
            if secs >= 60 {
                format!("{} мин {} сек", secs / 60, secs % 60)
            } else {
                format!("{} сек", secs)
            }
        } else {
            "--".to_string()
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
    pub fn new(total_files: usize, total_bytes: u64) -> (Arc<Self>, watch::Receiver<DownloadProgress>) {
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

    pub fn add_total_bytes(&self, bytes: u64) {
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

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

    pub fn on_file_completed(&self) {
        let completed = self.completed_files.fetch_add(1, Ordering::Relaxed) + 1;
        self.sender.send_modify(|p| {
            p.completed_files = completed;
        });
    }
}
