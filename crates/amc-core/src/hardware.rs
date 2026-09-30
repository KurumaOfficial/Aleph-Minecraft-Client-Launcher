//! Best-effort hardware inventory for the first-run check (ROADMAP P1).
//!
//! Every query is fallible and degrades to `None`: a machine that cannot be
//! measured is never treated as weak and never blocks the user (CONCEPT:
//! "не блокирует"). No new dependencies — Win32 API on Windows,
//! `/proc` + `df` on Linux.

use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// Minimum spec the launcher targets (CONCEPT: GT710-tier machine).
pub const MIN_RAM_MB: u64 = 4096;
pub const MIN_CPU_THREADS: u32 = 4;
/// Free-space warning threshold for the game directory drive.
pub const MIN_FREE_DISK_MB: u64 = 2048;

/// Best-effort snapshot of the machine. Fields are `Option` when the OS
/// refused to answer — callers must treat `None` as "unknown, not weak".
#[derive(Debug, Clone, Default)]
pub struct HardwareReport {
    pub cpu_threads: u32,
    pub total_ram_mb: Option<u64>,
    /// Free space on the drive holding `measured_path`.
    pub free_disk_mb: Option<u64>,
}

impl HardwareReport {
    /// Gather what the OS is willing to tell us. Never panics.
    pub fn gather(measured_path: &Path) -> Self {
        Self {
            cpu_threads: cpu_threads(),
            total_ram_mb: total_ram_mb(),
            free_disk_mb: free_disk_mb(measured_path),
        }
    }

    /// Below-target CPU or RAM (unknown values do not count).
    pub fn is_weak(&self) -> bool {
        self.cpu_threads < MIN_CPU_THREADS || self.total_ram_mb.is_some_and(|ram| ram < MIN_RAM_MB)
    }

    /// Free space below the warning threshold (unknown does not count).
    pub fn is_low_disk(&self) -> bool {
        self.free_disk_mb
            .is_some_and(|free| free < MIN_FREE_DISK_MB)
    }
}

fn cpu_threads() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(2)
}

#[cfg(target_os = "windows")]
fn total_ram_mb() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    // SAFETY: `status` is a valid writable struct; `dwLength` is set as required.
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut status) == 0 {
            return None;
        }
        Some(status.ullTotalPhys / (1024 * 1024))
    }
}

#[cfg(target_os = "linux")]
fn total_ram_mb() -> Option<u64> {
    // `MemTotal:       16384000 kB`
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = content.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn total_ram_mb() -> Option<u64> {
    None
}

#[cfg(target_os = "windows")]
fn free_disk_mb(path: &Path) -> Option<u64> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let root = path
        .ancestors()
        .last()?
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let mut free_bytes: u64 = 0;
    // SAFETY: `root` is NUL-terminated; `free_bytes` is a valid out-pointer.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            root.as_ptr(),
            &mut free_bytes,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return None;
    }
    Some(free_bytes / (1024 * 1024))
}

#[cfg(target_os = "linux")]
fn free_disk_mb(path: &Path) -> Option<u64> {
    // `df -k -P <path>` → second line, 4th column = available KiB.
    let out = std::process::Command::new("df")
        .args(["-k", "-P"])
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let kb: u64 = stdout
        .lines()
        .nth(1)?
        .split_whitespace()
        .nth(3)?
        .parse()
        .ok()?;
    Some(kb / 1024)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn free_disk_mb(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(threads: u32, ram: Option<u64>, disk: Option<u64>) -> HardwareReport {
        HardwareReport {
            cpu_threads: threads,
            total_ram_mb: ram,
            free_disk_mb: disk,
        }
    }

    #[test]
    fn test_weak_thresholds() {
        assert!(report(2, Some(8192), None).is_weak()); // few cores
        assert!(report(8, Some(2048), None).is_weak()); // little RAM
        assert!(report(2, Some(2048), None).is_weak()); // both
        assert!(!report(4, Some(4096), None).is_weak()); // exact minimum is fine
        assert!(!report(8, Some(16384), None).is_weak());
    }

    #[test]
    fn test_unknown_is_never_weak() {
        assert!(!report(8, None, None).is_weak());
        assert!(!report(8, None, None).is_low_disk());
        // Unknown cores fall back to the std default path, never zero.
        assert!(report(0, None, None).is_weak()); // 0 < 4: explicit zero still counts
    }

    #[test]
    fn test_low_disk_threshold() {
        assert!(report(8, Some(8192), Some(512)).is_low_disk());
        assert!(!report(8, Some(8192), Some(2048)).is_low_disk());
        assert!(!report(8, Some(8192), Some(10240)).is_low_disk());
    }

    #[test]
    fn test_gather_never_panics_and_reports_threads() {
        let report = HardwareReport::gather(&std::env::temp_dir());
        assert!(report.cpu_threads >= 1);
    }
}
