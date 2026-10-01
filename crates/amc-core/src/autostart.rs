//! OS autostart (CONCEPT "Автозапуск", P11): register or remove the launcher
//! in the OS login items. Windows uses HKCU `...\\Run`, Linux writes an
//! XDG `.desktop` file. Best-effort: every failure is a `String`, never a
//! panic. Other OSes report unsupported.

use std::path::PathBuf;

pub const APP_ID: &str = "AlephLauncher";

/// Currently running executable, if determinable.
pub fn launcher_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

#[cfg(target_os = "windows")]
mod platform {
    use super::APP_ID;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ,
    };

    const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn is_enabled() -> bool {
        // SAFETY: fixed-size UTF-16 buffer, length passed in bytes.
        unsafe {
            let mut buf = [0u16; 4096];
            let mut len = (buf.len() * 2) as u32;
            RegGetValueW(
                HKEY_CURRENT_USER,
                wide(RUN_SUBKEY).as_ptr(),
                wide(APP_ID).as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                &mut len,
            ) == 0
        }
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        // SAFETY: all pointers are valid NUL-terminated UTF-16; handles closed.
        // WIN32_ERROR compares via its `.0` payload; 2 is ERROR_FILE_NOT_FOUND.
        unsafe {
            if !enabled {
                let mut key: HKEY = std::ptr::null_mut();
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    wide(RUN_SUBKEY).as_ptr(),
                    0,
                    KEY_SET_VALUE,
                    &mut key,
                ) != 0
                {
                    return Ok(()); // No Run key at all — already effectively off.
                }
                let status = RegDeleteValueW(key, wide(APP_ID).as_ptr());
                RegCloseKey(key);
                if status != 0 && status != 2 {
                    return Err(format!("Failed to remove Run value (code {})", status));
                }
                return Ok(());
            }
            let exe =
                super::launcher_exe().ok_or_else(|| "Current exe path unknown".to_string())?;
            let value = format!("\"{}\"", exe.display());
            let data: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
            let mut key: HKEY = std::ptr::null_mut();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide(RUN_SUBKEY).as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut key,
            ) != 0
            {
                return Err("Cannot open HKCU Run key".to_string());
            }
            let status = RegSetValueExW(
                key,
                wide(APP_ID).as_ptr(),
                0,
                REG_SZ,
                data.as_ptr() as *const u8,
                (data.len() * 2) as u32,
            );
            RegCloseKey(key);
            if status != 0 {
                return Err(format!("Failed to write Run value (code {})", status));
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::APP_ID;

    fn desktop_file() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(
            PathBuf::from(home)
                .join(".config/autostart")
                .join(format!("{}.desktop", APP_ID.to_lowercase())),
        )
    }

    pub fn is_enabled() -> bool {
        desktop_file().map(|p| p.is_file()).unwrap_or(false)
    }

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let file = desktop_file().ok_or_else(|| "HOME is not set".to_string())?;
        if !enabled {
            let _ = std::fs::remove_file(&file);
            return Ok(());
        }
        let exe = super::launcher_exe().ok_or_else(|| "Current exe path unknown".to_string())?;
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName=Aleph Launcher\nExec=\"{}\"\nHidden=false\nX-GNOME-Autostart-enabled=true\n",
            exe.display()
        );
        std::fs::write(&file, entry).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
    pub fn is_enabled() -> bool {
        false
    }

    pub fn set_enabled(_enabled: bool) -> Result<(), String> {
        Err("Autostart is not supported on this OS".to_string())
    }
}

pub fn is_enabled() -> bool {
    platform::is_enabled()
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    platform::set_enabled(enabled)
}
