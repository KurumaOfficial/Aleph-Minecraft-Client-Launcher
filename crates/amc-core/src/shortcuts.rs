use crate::error::{LauncherError, Result};
use std::path::{Path, PathBuf};

/// Desktop shortcuts that launch one instance headlessly
/// (`amc --launch <id>`, CONCEPT P11/P14). No new dependencies:
/// Windows goes through `powershell` + `WScript.Shell`, Linux writes a
/// freedesktop `.desktop` file. macOS is not a v1.0 target.
pub struct InstanceShortcut {
    /// File stem shown on the desktop.
    pub file_name: String,
    /// Launcher binary to invoke.
    pub exe_path: PathBuf,
    /// Instance id argument.
    pub instance_id: String,
    /// Working directory for the child.
    pub work_dir: PathBuf,
}

impl InstanceShortcut {
    /// Build from an instance name/id. The file stem is sanitized for
    /// desktop filesystems; the id (a UUID) is shell-safe as-is.
    pub fn new(name: &str, instance_id: &str, exe_path: PathBuf, work_dir: PathBuf) -> Self {
        Self {
            file_name: sanitize_file_stem(name),
            exe_path,
            instance_id: instance_id.to_string(),
            work_dir,
        }
    }

    /// Arguments passed to the launcher binary.
    pub fn launch_args(&self) -> String {
        format!("--launch {}", self.instance_id)
    }

    /// Create the shortcut on the current OS desktop.
    /// Returns the written file path.
    pub fn create(&self) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            Self::create_windows_lnk(self)
        }
        #[cfg(target_os = "linux")]
        {
            Self::create_linux_desktop(self)
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err(LauncherError::Custom(
                "Desktop shortcuts are supported on Windows and Linux only".to_string(),
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn create_windows_lnk(shortcut: &Self) -> Result<PathBuf> {
        let desktop = dirs::desktop_dir()
            .ok_or_else(|| LauncherError::Custom("Desktop folder not found".to_string()))?;
        let link = desktop.join(format!("{}.lnk", shortcut.file_name));
        let script = windows_lnk_script(
            &link,
            &shortcut.exe_path,
            &shortcut.launch_args(),
            &shortcut.work_dir,
        );
        let out = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .output()
            .map_err(|e| {
                LauncherError::Custom(format!("Shortcut powershell failed to start: {e}"))
            })?;
        if !out.status.success() {
            return Err(LauncherError::Custom(format!(
                "Shortcut creation failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        if !link.is_file() {
            return Err(LauncherError::Custom(
                "Shortcut file did not appear after powershell run".to_string(),
            ));
        }
        Ok(link)
    }

    #[cfg(target_os = "linux")]
    fn create_linux_desktop(shortcut: &Self) -> Result<PathBuf> {
        let desktop = dirs::desktop_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join("Desktop"))
                .unwrap_or_else(|| PathBuf::from("./Desktop"))
        });
        std::fs::create_dir_all(&desktop).map_err(|e| LauncherError::Io {
            path: desktop.clone(),
            source: e,
        })?;
        let file = desktop.join(format!("{}.desktop", shortcut.file_name));
        std::fs::write(&file, linux_desktop_entry(shortcut)).map_err(|e| LauncherError::Io {
            path: file.clone(),
            source: e,
        })?;
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&file)
                .map_err(|e| LauncherError::Io {
                    path: file.clone(),
                    source: e,
                })?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&file, perms).map_err(|e| LauncherError::Io {
                path: file.clone(),
                source: e,
            })?;
        }
        Ok(file)
    }
}

/// Keep desktop-safe characters only; never empty.
pub fn sanitize_file_stem(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().to_string();
    if trimmed.is_empty() {
        "AMC instance".to_string()
    } else {
        format!("{trimmed} (AMC)")
    }
}

/// PowerShell `WScript.Shell` snippet creating one `.lnk`.
/// Pure for tests; single quotes in paths are doubled (PS escaping).
pub fn windows_lnk_script(link: &Path, target: &Path, args: &str, work_dir: &Path) -> String {
    fn esc(path: &Path) -> String {
        path.to_string_lossy().replace('\'', "''")
    }
    format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');\
         $s.TargetPath='{}';$s.Arguments='{}';$s.WorkingDirectory='{}';$s.Save()",
        esc(link),
        esc(target),
        args.replace('\'', "''"),
        esc(work_dir)
    )
}

/// Freedesktop entry for one instance. Pure for tests.
pub fn linux_desktop_entry(shortcut: &InstanceShortcut) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nVersion=1.0\nName={} (AMC)\nComment=Launch Minecraft instance {}\nExec=\"{}\" {}\nPath=\"{}\"\nTerminal=false\nCategories=Game;\n",
        shortcut.file_name,
        shortcut.instance_id,
        shortcut.exe_path.to_string_lossy(),
        shortcut.launch_args(),
        shortcut.work_dir.to_string_lossy()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_file_stem() {
        assert_eq!(sanitize_file_stem("My Pack/1:20?"), "My Pack_1_20_ (AMC)");
        assert_eq!(sanitize_file_stem("  "), "AMC instance");
        assert_eq!(sanitize_file_stem(""), "AMC instance");
        assert_eq!(sanitize_file_stem("SMP (Copy)"), "SMP (Copy) (AMC)");
    }

    #[test]
    fn test_launch_args() {
        let s = InstanceShortcut::new(
            "Pack",
            "123e4567-e89b-12d3-a456-426614174000",
            PathBuf::from("C:/x/amc.exe"),
            PathBuf::from("C:/x"),
        );
        assert_eq!(
            s.launch_args(),
            "--launch 123e4567-e89b-12d3-a456-426614174000"
        );
    }

    #[test]
    fn test_windows_lnk_script_escapes_quotes() {
        let script = windows_lnk_script(
            Path::new("C:/D/a'b.lnk"),
            Path::new("C:/x/amc.exe"),
            "--launch id",
            Path::new("C:/x"),
        );
        assert!(script.contains("a''b.lnk"));
        assert!(script.contains("TargetPath='C:/x/amc.exe'"));
        assert!(script.contains("Arguments='--launch id'"));
    }

    #[test]
    fn test_linux_desktop_entry() {
        let s = InstanceShortcut::new(
            "Pack",
            "abc-id",
            PathBuf::from("/opt/amc/amc-launcher"),
            PathBuf::from("/opt/amc"),
        );
        let entry = linux_desktop_entry(&s);
        assert!(entry.contains("Type=Application"));
        assert!(entry.contains("Exec=\"/opt/amc/amc-launcher\" --launch abc-id"));
        assert!(entry.contains("Path=\"/opt/amc\""));
    }

    /// LIVE desktop roundtrip (manual only — writes a probe shortcut to
    /// the real desktop and deletes it; never in CI):
    /// `cargo test -p amc-core -- --ignored live_desktop_shortcut`
    #[test]
    #[ignore]
    fn live_desktop_shortcut_roundtrip() {
        let exe = std::env::current_exe().expect("test binary path");
        let probe = InstanceShortcut::new(
            "AMC probe delete me",
            "00000000-0000-0000-0000-000000000000",
            exe,
            std::env::temp_dir(),
        );
        let path = probe.create().expect("shortcut creates on this OS");
        assert!(path.is_file(), "shortcut file appears");
        std::fs::remove_file(&path).expect("probe cleaned up");
        assert!(!path.exists(), "probe gone");
    }
}
