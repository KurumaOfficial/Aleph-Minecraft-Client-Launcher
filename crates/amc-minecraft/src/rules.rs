use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

pub fn current_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    }
}

fn rule_matches(rule: &Rule, features: &HashMap<String, bool>) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != current_os() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch != current_arch() {
                return false;
            }
        }
    }
    if let Some(feats) = &rule.features {
        for (key, expected) in feats {
            let actual = features.get(key).copied().unwrap_or(false);
            if actual != *expected {
                return false;
            }
        }
    }
    true
}

pub fn allows(rules: Option<&[Rule]>, features: &HashMap<String, bool>) -> bool {
    let Some(rules) = rules else {
        return true;
    };
    if rules.is_empty() {
        return true;
    }
    let mut allow = false;
    for rule in rules {
        if rule_matches(rule, features) {
            allow = rule.action == "allow";
        }
    }
    allow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(action: &str, os: Option<&str>) -> Rule {
        Rule {
            action: action.to_string(),
            os: os.map(|name| OsRule {
                name: Some(name.to_string()),
                arch: None,
                version: None,
            }),
            features: None,
        }
    }

    #[test]
    fn test_allows_matrix() {
        let empty: HashMap<String, bool> = HashMap::new();
        // No rules at all: allowed.
        assert!(allows(None, &empty));
        assert!(allows(Some(&[]), &empty));
        // Matching allow wins; last matching rule decides.
        let rules = vec![rule("allow", None)];
        assert!(allows(Some(&rules), &empty));
        let rules = vec![rule("allow", None), rule("disallow", None)];
        assert!(!allows(Some(&rules), &empty));
        // OS-gated rules only fire on their platform (no match = deny).
        let other = if current_os() == "windows" {
            "linux"
        } else {
            "windows"
        };
        let rules = vec![rule("disallow", Some(other))];
        assert!(!allows(Some(&rules), &empty));
        let rules = vec![rule("disallow", Some(current_os()))];
        assert!(!allows(Some(&rules), &empty));
        let rules = vec![rule("allow", Some(current_os()))];
        assert!(allows(Some(&rules), &empty));
    }
}
