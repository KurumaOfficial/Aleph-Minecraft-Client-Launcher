use crate::types::Instance;
use std::path::Path;

/// Instance integrity (CONCEPT "Целостность и удаление", P2): verify that
/// the expected folders and launcher-installed files are in place, roll
/// them into one 0–100 score, and repair what can be recreated offline.
/// Anything needing downloads (missing mod jars) is reported, not fetched.
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Default)]
pub struct HealthReport {
    pub checks: Vec<HealthCheck>,
}

impl HealthReport {
    /// Whole-instance score: share of passing checks, 0–100.
    pub fn score(&self) -> u8 {
        if self.checks.is_empty() {
            return 0;
        }
        let ok = self.checks.iter().filter(|c| c.ok).count();
        ((ok * 100) / self.checks.len()) as u8
    }

    pub fn all_ok(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|c| c.ok)
    }

    fn check(name: &str, ok: bool, detail: String) -> HealthCheck {
        HealthCheck {
            name: name.to_string(),
            ok,
            detail,
        }
    }

    /// Verify an instance against its game directory.
    pub fn verify(instances_dir: &Path, inst: &Instance) -> Self {
        let game_dir = inst.get_game_dir(instances_dir);
        let mut checks = Vec::new();

        checks.push(Self::check(
            "game_dir",
            game_dir.is_dir(),
            game_dir.to_string_lossy().to_string(),
        ));
        for sub in ["mods", "config", "saves", "resourcepacks"] {
            let dir = game_dir.join(sub);
            let ok = dir.is_dir();
            checks.push(Self::check(
                sub,
                ok || sub == "saves",
                // A fresh instance legitimately has no saves yet.
                if ok {
                    String::new()
                } else if sub == "saves" {
                    "no saves yet".to_string()
                } else {
                    format!("missing {}", dir.to_string_lossy())
                },
            ));
        }

        let mods_dir = game_dir.join("mods");
        let missing: Vec<String> = inst
            .installed_by_launcher
            .iter()
            .filter(|name| !mods_dir.join(name).is_file())
            .cloned()
            .collect();
        checks.push(Self::check(
            "installed_mods",
            missing.is_empty(),
            if missing.is_empty() {
                format!("{} tracked", inst.installed_by_launcher.len())
            } else {
                format!("missing: {}", missing.join(", "))
            },
        ));

        Self { checks }
    }

    /// Recreate whatever `verify` can fix offline (missing folders).
    /// Returns human-readable action descriptions.
    pub fn repair(instances_dir: &Path, inst: &Instance) -> Vec<String> {
        let game_dir = inst.get_game_dir(instances_dir);
        let mut done = Vec::new();
        for sub in ["mods", "config", "saves", "resourcepacks"] {
            let dir = game_dir.join(sub);
            if !dir.is_dir() && std::fs::create_dir_all(&dir).is_ok() {
                done.push(format!("created {}", dir.to_string_lossy()));
            }
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LoaderType;
    use std::path::PathBuf;

    fn setup(tag: &str) -> (PathBuf, Instance) {
        let root = std::env::temp_dir().join(format!(
            "amc_health_{tag}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&root);
        let instances_dir = root.join("instances");
        let mut inst = Instance::new("Healthy", "1.20.1", LoaderType::Fabric);
        let game_dir = inst.get_game_dir(&instances_dir);
        for sub in ["mods", "config"] {
            std::fs::create_dir_all(game_dir.join(sub)).unwrap();
        }
        std::fs::write(game_dir.join("mods").join("a.jar"), b"fake").unwrap();
        inst.installed_by_launcher = vec!["a.jar".to_string(), "gone.jar".to_string()];
        (instances_dir, inst)
    }

    #[test]
    fn test_score_math() {
        let report = HealthReport { checks: vec![] };
        assert_eq!(report.score(), 0);
        let report = HealthReport {
            checks: vec![
                HealthCheck {
                    name: "a".to_string(),
                    ok: true,
                    detail: String::new(),
                },
                HealthCheck {
                    name: "b".to_string(),
                    ok: false,
                    detail: String::new(),
                },
            ],
        };
        assert_eq!(report.score(), 50);
        assert!(!report.all_ok());
    }

    #[test]
    fn test_verify_flags_missing_mod() {
        let (instances_dir, inst) = setup("verify");
        let report = HealthReport::verify(&instances_dir, &inst);
        // game_dir + mods + config + saves(info-ok) + resourcepacks(missing) + mods-tracked(missing one)
        assert!(!report.all_ok());
        let tracked = report
            .checks
            .iter()
            .find(|c| c.name == "installed_mods")
            .unwrap();
        assert!(!tracked.ok);
        assert!(tracked.detail.contains("gone.jar"));
        let root = instances_dir.parent().unwrap().to_path_buf();
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_repair_creates_dirs() {
        let (instances_dir, inst) = setup("repair");
        let done = HealthReport::repair(&instances_dir, &inst);
        assert!(!done.is_empty());
        let game_dir = inst.get_game_dir(&instances_dir);
        assert!(game_dir.join("saves").is_dir());
        assert!(game_dir.join("resourcepacks").is_dir());
        let root = instances_dir.parent().unwrap().to_path_buf();
        let _ = std::fs::remove_dir_all(&root);
    }
}
