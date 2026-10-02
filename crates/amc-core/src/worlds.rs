use crate::error::{LauncherError, Result};
use crate::trash::copy_dir;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// World backup/restore (CONCEPT "Управление мирами", P9): manual backups
/// from the launcher, restore in one click, optional automatic backups
/// every N days, and a user-chosen backup folder. Backups are plain
/// directory copies (`<root>/<instance>/<world>_<stamp>/`) so they stay
/// readable without the launcher.
/// Keep folder names filesystem-safe.
fn sanitize(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = clean.trim().to_string();
    if trimmed.is_empty() {
        "world".to_string()
    } else {
        trimmed
    }
}

fn stamp() -> String {
    // chrono is already a core dependency; UTC keeps names sortable.
    chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

/// Save folders of an instance: subdirectories of `saves/` holding level.dat.
pub fn list_saves(game_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(game_dir.join("saves")) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("level.dat").is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out
}

/// Copy a world into a fresh timestamped backup folder. Returns its path.
pub fn backup_world(
    backups_root: &Path,
    instance_name: &str,
    world: &str,
    saves_dir: &Path,
) -> Result<PathBuf> {
    let src = saves_dir.join(world);
    if !src.join("level.dat").is_file() {
        return Err(LauncherError::Custom(format!(
            "World '{world}' has no level.dat"
        )));
    }
    let dest =
        backups_root
            .join(sanitize(instance_name))
            .join(format!("{}_{}", sanitize(world), stamp()));
    copy_dir(&src, &dest)?;
    Ok(dest)
}

/// Backups for one world, newest first.
pub fn list_backups(backups_root: &Path, instance_name: &str, world: &str) -> Vec<PathBuf> {
    let dir = backups_root.join(sanitize(instance_name));
    let prefix = format!("{}_", sanitize(world));
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix))
        })
        .collect();
    out.sort();
    out.reverse();
    out
}

/// Restore a backup over the live world. The current world is first moved
/// aside (`<world>.pre-restore-<stamp>`) so restore never destroys data.
pub fn restore_world(backup_dir: &Path, saves_dir: &Path, world: &str) -> Result<()> {
    if !backup_dir.join("level.dat").is_file() {
        return Err(LauncherError::Custom("Backup has no level.dat".to_string()));
    }
    let live = saves_dir.join(world);
    if live.is_dir() {
        let aside = saves_dir.join(format!("{world}.pre-restore-{}", stamp()));
        std::fs::rename(&live, &aside).map_err(|e| LauncherError::Io {
            path: live.clone(),
            source: e,
        })?;
    }
    copy_dir(backup_dir, &live)?;
    Ok(())
}

/// Delete a single backup folder.
pub fn delete_backup(backup_dir: &Path) -> Result<()> {
    if backup_dir.is_dir() {
        std::fs::remove_dir_all(backup_dir).map_err(|e| LauncherError::Io {
            path: backup_dir.to_path_buf(),
            source: e,
        })?;
    }
    Ok(())
}

/// Modification time of the newest backup, if any.
pub fn newest_backup_mtime(
    backups_root: &Path,
    instance_name: &str,
    world: &str,
) -> Option<SystemTime> {
    list_backups(backups_root, instance_name, world)
        .iter()
        .filter_map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
        .max()
}

/// True when automatic backups are due: enabled (`days > 0`) and no backup
/// newer than the interval.
pub fn needs_auto_backup(
    backups_root: &Path,
    instance_name: &str,
    world: &str,
    days: u64,
    now: SystemTime,
) -> bool {
    if days == 0 {
        return false;
    }
    match newest_backup_mtime(backups_root, instance_name, world) {
        None => true,
        Some(mtime) => now
            .duration_since(mtime)
            .map(|age| age.as_secs() >= days * 86_400)
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "amc_worlds_{tag}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_world(saves: &Path, name: &str) {
        let dir = saves.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("level.dat"), b"fake-level").unwrap();
    }

    #[test]
    fn test_list_saves_only_level_dat() {
        let root = tmp_root("list");
        let saves = root.join("saves");
        make_world(&saves, "Alpha");
        std::fs::create_dir_all(saves.join("empty")).unwrap();
        assert_eq!(list_saves(&root), vec!["Alpha".to_string()]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_backup_restore_roundtrip() {
        let root = tmp_root("roundtrip");
        let saves = root.join("saves");
        make_world(&saves, "Beta");
        let backups = root.join("backups");

        let backup = backup_world(&backups, "Inst", "Beta", &saves).unwrap();
        assert!(backup.join("level.dat").is_file());

        // Play on: change the live world, then restore.
        std::fs::write(saves.join("Beta").join("new.dat"), b"progress").unwrap();
        restore_world(&backup, &saves, "Beta").unwrap();
        assert!(!saves.join("Beta").join("new.dat").is_file());
        assert!(saves.join("Beta").join("level.dat").is_file());
        // The pre-restore copy kept the progress.
        let aside: Vec<_> = std::fs::read_dir(&saves)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("Beta.pre-restore-"))
            .collect();
        assert_eq!(aside.len(), 1);

        assert_eq!(list_backups(&backups, "Inst", "Beta"), vec![backup]);
        delete_backup(&list_backups(&backups, "Inst", "Beta")[0]).unwrap();
        assert!(list_backups(&backups, "Inst", "Beta").is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_needs_auto_backup() {
        use std::time::Duration;
        let root = tmp_root("auto");
        let saves = root.join("saves");
        make_world(&saves, "Gamma");
        let backups = root.join("backups");
        let now = SystemTime::now();

        assert!(!needs_auto_backup(&backups, "I", "Gamma", 0, now));
        assert!(needs_auto_backup(&backups, "I", "Gamma", 7, now));
        backup_world(&backups, "I", "Gamma", &saves).unwrap();
        assert!(!needs_auto_backup(&backups, "I", "Gamma", 7, now));
        let future = now + Duration::from_secs(8 * 86_400);
        assert!(needs_auto_backup(&backups, "I", "Gamma", 7, future));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("My/World: Nether*"), "My_World_ Nether_");
        assert_eq!(sanitize(""), "world");
    }
}
