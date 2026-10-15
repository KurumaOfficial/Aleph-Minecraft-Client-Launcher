use amc_core::error::{LauncherError, Result};
use std::path::Path;
use std::process::Command;

/// Pre-launch / post-exit instance scripts (CONCEPT "Pre-launch/post-exit
/// скрипты", P2). The command runs in the instance game dir through the
/// platform shell (`cmd /C` on Windows, `sh -c` elsewhere) so users can
/// chain commands with `&&`, set env vars, etc.
///
/// An empty command is a no-op. A non-zero exit is an error carrying the
/// script's stderr tail, so the caller can abort (pre-launch) or report
/// (post-exit). No timeout is applied — scripts are user-authored and
/// expected to finish quickly; callers run this off the UI thread.
pub fn run_hook_script(command: &str, work_dir: &Path) -> Result<String> {
    if command.trim().is_empty() {
        return Ok(String::new());
    }
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };
    let output = cmd
        .current_dir(work_dir)
        .output()
        .map_err(|e| LauncherError::Custom(format!("Hook script failed to start: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail: String = stderr
        .lines()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    let code = output
        .status
        .code()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".into());
    Err(LauncherError::Custom(format!(
        "Hook script exited with code {code}{}",
        if tail.is_empty() {
            String::new()
        } else {
            format!(": {tail}")
        }
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_dir() -> std::path::PathBuf {
        std::env::temp_dir()
    }

    #[test]
    fn test_empty_command_is_noop() {
        let out = run_hook_script("   ", &work_dir()).expect("empty hook skips");
        assert!(out.is_empty());
    }

    #[test]
    fn test_echo_reports_stdout() {
        let out = run_hook_script("echo amc-hook-ok", &work_dir()).expect("echo works");
        assert!(out.contains("amc-hook-ok"));
    }

    #[test]
    fn test_failing_command_errors() {
        #[cfg(target_os = "windows")]
        let err = run_hook_script("exit 3", &work_dir()).expect_err("exit 3 fails");
        #[cfg(not(target_os = "windows"))]
        let err = run_hook_script("exit 3", &work_dir()).expect_err("exit 3 fails");
        assert!(err.to_string().contains('3'));
    }

    #[test]
    fn test_missing_work_dir_errors() {
        let missing = Path::new("definitely-not-here-amc-hook-test");
        assert!(run_hook_script("echo hi", missing).is_err());
    }
}
