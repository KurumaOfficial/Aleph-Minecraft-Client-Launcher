use crate::error::{LauncherError, Result};
use crate::types::Instance;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Soft-delete for instances (CONCEPT "Целостность и удаление", P2):
/// deleting moves the game directory into `trash/<id>/` together with a
/// snapshot of the `Instance` record, so removal is always undoable.
/// Instances with a `custom_dir` outside the launcher root are never moved
/// (that may be user data elsewhere) — only their list record is dropped.
pub struct TrashedInstance {
    pub instance: Instance,
    pub dir: PathBuf,
}

const RECORD_FILE: &str = "instance.json";

fn io_err(path: &Path, source: std::io::Error) -> LauncherError {
    LauncherError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Move a file tree, falling back to copy+delete across devices/mounts.
fn move_tree(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_dir(src, dst)?;
            std::fs::remove_dir_all(src).map_err(|e| io_err(src, e))?;
            Ok(())
        }
    }
}

/// Recursive directory copy (pure fs, unit-tested). Symlinks are skipped.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| io_err(dst, e))?;
    let entries = std::fs::read_dir(src).map_err(|e| io_err(src, e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_symlink() {
            continue;
        }
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else if path.is_file() {
            std::fs::copy(&path, &dest).map_err(|e| io_err(&path, e))?;
        }
    }
    Ok(())
}

/// Is `dir` the launcher-managed folder for this instance (not an outside
/// custom path)? Only managed folders are moved to trash.
fn is_managed_dir(instances_dir: &Path, dir: &Path) -> bool {
    dir.starts_with(instances_dir)
}

/// Trash an instance. Returns the trash entry dir when the game directory
/// was moved, or `None` for outside custom dirs (record-only removal).
pub fn trash_instance(
    instances_dir: &Path,
    trash_dir: &Path,
    inst: &Instance,
) -> Result<Option<PathBuf>> {
    let game_dir = inst.get_game_dir(instances_dir);
    let entry_dir = trash_dir.join(inst.id.to_string());
    std::fs::create_dir_all(&entry_dir).map_err(|e| io_err(&entry_dir, e))?;
    let record = serde_json::to_string_pretty(inst).map_err(LauncherError::Json)?;
    std::fs::write(entry_dir.join(RECORD_FILE), record)
        .map_err(|e| io_err(&entry_dir.join(RECORD_FILE), e))?;
    if game_dir.is_dir() && is_managed_dir(instances_dir, &game_dir) {
        move_tree(&game_dir, &entry_dir.join("data"))?;
        Ok(Some(entry_dir))
    } else {
        Ok(None)
    }
}

/// Every trashed instance that still has its record file.
pub fn list_trash(trash_dir: &Path) -> Vec<TrashedInstance> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(trash_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let record_path = dir.join(RECORD_FILE);
        let Ok(text) = std::fs::read_to_string(&record_path) else {
            continue;
        };
        if let Ok(instance) = serde_json::from_str::<Instance>(&text) {
            out.push(TrashedInstance { instance, dir });
        }
    }
    out.sort_by(|a, b| a.instance.name.cmp(&b.instance.name));
    out
}

/// Move a trashed instance back. On game-dir collision the restored copy
/// gets a numeric suffix so nothing is ever overwritten.
pub fn restore_instance(instances_dir: &Path, trash_dir: &Path, id: Uuid) -> Result<Instance> {
    let entry_dir = trash_dir.join(id.to_string());
    let record_path = entry_dir.join(RECORD_FILE);
    let text = std::fs::read_to_string(&record_path).map_err(|e| io_err(&record_path, e))?;
    let mut inst: Instance = serde_json::from_str(&text).map_err(LauncherError::Json)?;
    let data_dir = entry_dir.join("data");
    if data_dir.is_dir() {
        let base = inst.get_game_dir(instances_dir);
        let mut target = base.clone();
        if target.exists() {
            // Spaces survive get_game_dir sanitizing; parentheses do not,
            // so the collision suffix uses plain words.
            let stem = base
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("instance")
                .to_string();
            let mut n = 2;
            loop {
                let cand = base.with_file_name(format!("{stem} restored {n}"));
                if !cand.exists() {
                    target = cand;
                    break;
                }
                n += 1;
                if n > 999 {
                    return Err(LauncherError::Custom(
                        "No free folder name for restore".to_string(),
                    ));
                }
            }
        }
        move_tree(&data_dir, &target)?;
        if target != base {
            // Point a default-layout instance at the suffixed folder.
            if inst.custom_dir.is_none() {
                if let Some(name) = target.file_name().and_then(|n| n.to_str()) {
                    inst.name = name.to_string();
                }
            } else {
                inst.custom_dir = Some(target);
            }
        }
    }
    std::fs::remove_dir_all(&entry_dir).map_err(|e| io_err(&entry_dir, e))?;
    Ok(inst)
}

/// Delete one trash entry (game data + record) forever.
pub fn purge_trash_entry(trash_dir: &Path, id: Uuid) -> Result<()> {
    let entry_dir = trash_dir.join(id.to_string());
    if entry_dir.is_dir() {
        std::fs::remove_dir_all(&entry_dir).map_err(|e| io_err(&entry_dir, e))?;
    }
    Ok(())
}

/// Delete every trash entry. Returns the number of removed entries.
pub fn purge_trash_all(trash_dir: &Path) -> usize {
    let mut n = 0;
    for entry in list_trash(trash_dir) {
        if purge_trash_entry(trash_dir, entry.instance.id).is_ok() {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LoaderType;

    fn tmp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "amc_trash_{tag}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("instances")).unwrap();
        std::fs::create_dir_all(dir.join("trash")).unwrap();
        dir
    }

    fn sample_instance(name: &str) -> Instance {
        Instance::new(name, "1.20.1", LoaderType::Fabric)
    }

    #[test]
    fn test_trash_restore_roundtrip() {
        let root = tmp_root("roundtrip");
        let instances_dir = root.join("instances");
        let trash_dir = root.join("trash");
        let inst = sample_instance("Survival");
        let game_dir = inst.get_game_dir(&instances_dir);
        std::fs::create_dir_all(game_dir.join("saves")).unwrap();
        std::fs::write(game_dir.join("saves").join("level.dat"), b"fake").unwrap();

        let moved = trash_instance(&instances_dir, &trash_dir, &inst).unwrap();
        assert!(moved.is_some());
        assert!(!game_dir.is_dir());
        assert_eq!(list_trash(&trash_dir).len(), 1);

        let restored = restore_instance(&instances_dir, &trash_dir, inst.id).unwrap();
        assert_eq!(restored.id, inst.id);
        assert_eq!(restored.name, inst.name);
        assert!(game_dir.join("saves").join("level.dat").is_file());
        assert!(list_trash(&trash_dir).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_trash_outside_custom_dir_keeps_data() {
        let root = tmp_root("custom");
        let instances_dir = root.join("instances");
        let trash_dir = root.join("trash");
        let outside = root.join("elsewhere");
        std::fs::create_dir_all(&outside).unwrap();
        let mut inst = sample_instance("Custom");
        inst.custom_dir = Some(outside.clone());

        let moved = trash_instance(&instances_dir, &trash_dir, &inst).unwrap();
        assert!(moved.is_none());
        assert!(outside.is_dir());
        // Record exists so the entry is listed (restore re-adds the record).
        assert_eq!(list_trash(&trash_dir).len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_restore_collision_suffixes() {
        let root = tmp_root("collision");
        let instances_dir = root.join("instances");
        let trash_dir = root.join("trash");
        let inst = sample_instance("Dup");
        let game_dir = inst.get_game_dir(&instances_dir);
        std::fs::create_dir_all(&game_dir).unwrap();
        trash_instance(&instances_dir, &trash_dir, &inst).unwrap();
        // A new folder appears at the old place before restoring.
        std::fs::create_dir_all(&game_dir).unwrap();
        let restored = restore_instance(&instances_dir, &trash_dir, inst.id).unwrap();
        assert_ne!(restored.get_game_dir(&instances_dir), game_dir);
        assert!(restored.get_game_dir(&instances_dir).is_dir());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_purge() {
        let root = tmp_root("purge");
        let instances_dir = root.join("instances");
        let trash_dir = root.join("trash");
        let inst = sample_instance("Gone");
        std::fs::create_dir_all(inst.get_game_dir(&instances_dir)).unwrap();
        trash_instance(&instances_dir, &trash_dir, &inst).unwrap();
        purge_trash_entry(&trash_dir, inst.id).unwrap();
        assert!(list_trash(&trash_dir).is_empty());
        // Purging twice is fine.
        purge_trash_entry(&trash_dir, inst.id).unwrap();
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_copy_dir_roundtrip() {
        let root = tmp_root("copy");
        let src = root.join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), b"a").unwrap();
        std::fs::write(src.join("sub").join("b.txt"), b"b").unwrap();
        let dst = root.join("dst");
        copy_dir(&src, &dst).unwrap();
        assert_eq!(std::fs::read(dst.join("a.txt")).unwrap(), b"a");
        assert_eq!(std::fs::read(dst.join("sub").join("b.txt")).unwrap(), b"b");
        let _ = std::fs::remove_dir_all(&root);
    }
}
