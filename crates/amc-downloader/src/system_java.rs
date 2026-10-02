//! System Java discovery (ROADMAP P5): find already-installed runtimes
//! (`JAVA_HOME`, `PATH`, vendor directories, other launchers' runtimes) so
//! the user can pick one instead of downloading. Best-effort throughout —
//! an unreadable directory or a hanging probe never fails the scan.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SystemJava {
    /// `java`/`java.exe` binary path.
    pub path: PathBuf,
    /// Probed major version (`None` when the probe failed).
    pub major: Option<u32>,
    /// Where it was found: `JAVA_HOME`, `PATH`, `Adoptium`, …
    pub source: String,
}

pub fn java_exe_name() -> &'static str {
    if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }
}

/// Parse `java -version` output: `openjdk version "21.0.2" …` or legacy
/// `java version "1.8.0_391" …` (1.x maps to 8). Pure, unit-tested.
pub fn parse_java_version(output: &str) -> Option<u32> {
    let first = output.lines().next()?;
    let quoted = first.split('"').nth(1)?;
    let mut parts = quoted.split(['.', '_', '-']);
    let major: u32 = parts.next()?.parse().ok()?;
    if major == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(major)
    }
}

fn sort_java(list: &mut [SystemJava]) {
    list.sort_by(|a, b| match (a.major, b.major) {
        (Some(x), Some(y)) => y.cmp(&x).then_with(|| a.path.cmp(&b.path)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.path.cmp(&b.path),
    });
}

/// Run `java -version` with a timeout. Hanging binaries only delay the
/// background scan, never the UI.
fn probe_java_major(bin: &Path) -> Option<u32> {
    use std::io::Read;
    use std::process::{Command, Stdio};

    let mut child = Command::new(bin)
        .arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }

    let mut text = String::new();
    if let Some(mut out) = stdout {
        let mut buf = Vec::new();
        let _ = out.read_to_end(&mut buf);
        text.push_str(&String::from_utf8_lossy(&buf));
    }
    if let Some(mut err) = stderr {
        let mut buf = Vec::new();
        let _ = err.read_to_end(&mut buf);
        text.push_str(&String::from_utf8_lossy(&buf));
    }
    parse_java_version(&text)
}

/// Collect `bin/java(.exe)` candidates under `root` (depth-bounded).
fn scan_tree(root: &Path, depth: u8, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_symlink() || !path.is_dir() {
            continue;
        }
        let candidate = path.join("bin").join(java_exe_name());
        if candidate.is_file() {
            out.push(candidate);
        } else {
            scan_tree(&path, depth - 1, out);
        }
    }
}

fn mc_runtime_roots() -> Vec<PathBuf> {
    // Official launcher runtimes live here on every OS.
    let base = if cfg!(windows) {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    } else {
        std::env::var("HOME").ok().map(PathBuf::from)
    };
    base.map(|b| b.join(".minecraft").join("runtime"))
        .into_iter()
        .collect()
}

fn vendor_roots() -> Vec<(&'static str, PathBuf, u8)> {
    if cfg!(windows) {
        let pf = Path::new("C:\\Program Files");
        let pf86 = Path::new("C:\\Program Files (x86)");
        vec![
            ("Java", pf.join("Java"), 2),
            ("Java", pf86.join("Java"), 2),
            ("Adoptium", pf.join("Eclipse Adoptium"), 2),
            ("Eclipse", pf.join("Eclipse Foundation"), 2),
            ("Microsoft", pf.join("Microsoft"), 2),
            ("Zulu", pf.join("Zulu"), 2),
            ("BellSoft", pf.join("BellSoft"), 2),
            ("Corretto", pf.join("Amazon Corretto"), 2),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            (
                "system",
                PathBuf::from("/Library/Java/JavaVirtualMachines"),
                4,
            ),
            (
                "user",
                dirs_home().join("Library/Java/JavaVirtualMachines"),
                4,
            ),
        ]
    } else {
        vec![
            ("system", PathBuf::from("/usr/lib/jvm"), 2),
            ("opt", PathBuf::from("/opt"), 2),
            ("SDKMAN", dirs_home().join(".sdkman/candidates/java"), 2),
        ]
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Find system Java installations. Never panics, never fails as a whole —
/// every source is best-effort and an empty list is a valid answer.
pub fn discover_system_java() -> Vec<SystemJava> {
    let mut candidates: Vec<(PathBuf, String)> = Vec::new();

    if let Ok(home) = std::env::var("JAVA_HOME") {
        let bin = Path::new(&home).join("bin").join(java_exe_name());
        if bin.is_file() {
            candidates.push((bin, "JAVA_HOME".to_string()));
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let bin = dir.join(java_exe_name());
            if bin.is_file() {
                candidates.push((bin, "PATH".to_string()));
            }
        }
    }

    for (label, root, depth) in vendor_roots() {
        let mut found = Vec::new();
        scan_tree(&root, depth, &mut found);
        for bin in found {
            candidates.push((bin, label.to_string()));
        }
    }

    for root in mc_runtime_roots() {
        let mut found = Vec::new();
        scan_tree(&root, 5, &mut found);
        for bin in found {
            candidates.push((bin, "Minecraft runtime".to_string()));
        }
    }

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (path, source) in candidates {
        if !path.is_file() {
            continue;
        }
        let key = path.canonicalize().unwrap_or_else(|_| path.clone());
        if !seen.insert(key) {
            continue;
        }
        out.push(SystemJava {
            major: probe_java_major(&path),
            path,
            source,
        });
    }
    sort_java(&mut out);
    out
}

/// Auto-offer (CONCEPT P5): a system Java with exactly `major`, if any.
/// The scan runs off-thread so slow probes never stall async tasks.
/// Returns the `java` binary path ready to launch with.
pub async fn find_system_java(major: u32) -> Option<PathBuf> {
    tokio::task::spawn_blocking(move || {
        discover_system_java()
            .into_iter()
            .find(|j| j.major == Some(major))
            .map(|j| j.path)
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_java_version() {
        assert_eq!(
            parse_java_version("openjdk version \"21.0.2\" 2024-01-16 LTS"),
            Some(21)
        );
        assert_eq!(
            parse_java_version("openjdk version \"17.0.9\" 2023-10-17"),
            Some(17)
        );
        assert_eq!(
            parse_java_version("java version \"1.8.0_391\"\nJava(TM) SE Runtime"),
            Some(8)
        );
        assert_eq!(
            parse_java_version("openjdk version \"17\" 2021-09-14"),
            Some(17)
        );
        assert_eq!(parse_java_version("garbage without quotes"), None);
        assert_eq!(parse_java_version(""), None);
        assert_eq!(parse_java_version("openjdk version \"\""), None);
    }

    #[test]
    fn test_sort_java_major_desc_unknown_last() {
        let mut list = vec![
            SystemJava {
                path: PathBuf::from("/b"),
                major: None,
                source: String::new(),
            },
            SystemJava {
                path: PathBuf::from("/c"),
                major: Some(8),
                source: String::new(),
            },
            SystemJava {
                path: PathBuf::from("/a"),
                major: Some(21),
                source: String::new(),
            },
        ];
        sort_java(&mut list);
        let majors: Vec<Option<u32>> = list.iter().map(|j| j.major).collect();
        assert_eq!(majors, vec![Some(21), Some(8), None]);
    }

    #[test]
    fn test_discover_never_panics() {
        let _ = discover_system_java();
    }
}
