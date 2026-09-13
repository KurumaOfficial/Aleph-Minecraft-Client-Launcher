use std::collections::HashMap;
use std::path::{Path, PathBuf};
use amc_auth::AuthSession;
use amc_core::types::LaunchOptions;
use crate::rules::allows;
use crate::version::{ArgumentValue, ArgumentValueNested, VersionDetails};

pub struct ArgumentBuilder;

impl ArgumentBuilder {
    pub fn build_jvm_args(
        options: &LaunchOptions,
        natives_dir: &Path,
        classpath: &str,
        details: &VersionDetails,
        features: &HashMap<String, bool>,
    ) -> Vec<String> {
        let mut args = Vec::new();

        // Memory limits
        args.push(format!("-Xms{}M", options.memory_min_mb));
        args.push(format!("-Xmx{}M", options.memory_max_mb));

        // Natives library path
        args.push(format!("-Djava.library.path={}", natives_dir.display()));

        // Modern Java modules compatibility
        args.push("-Dstdout.encoding=UTF-8".to_string());
        args.push("-Dstderr.encoding=UTF-8".to_string());

        // Custom JVM arguments from options
        for custom_arg in &options.custom_jvm_args {
            if !custom_arg.trim().is_empty() {
                args.push(custom_arg.clone());
            }
        }

        // Version-specific JVM arguments (from modern arguments.jvm array)
        if let Some(version_args) = &details.arguments {
            for arg_val in &version_args.jvm {
                match arg_val {
                    ArgumentValue::Simple(s) => {
                        let replaced = Self::replace_jvm_templates(s, natives_dir, classpath);
                        args.push(replaced);
                    }
                    ArgumentValue::Complex { rules, value } => {
                        if allows(rules.as_deref(), features) {
                            match value {
                                ArgumentValueNested::Single(s) => {
                                    let replaced = Self::replace_jvm_templates(s, natives_dir, classpath);
                                    args.push(replaced);
                                }
                                ArgumentValueNested::Multiple(list) => {
                                    for s in list {
                                        let replaced = Self::replace_jvm_templates(s, natives_dir, classpath);
                                        args.push(replaced);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add classpath if not already injected via template
        if !args.iter().any(|a| a == "-cp" || a == "-classpath") {
            args.push("-cp".to_string());
            args.push(classpath.to_string());
        }

        args
    }

    fn replace_jvm_templates(arg: &str, natives_dir: &Path, classpath: &str) -> String {
        let lib_dir = natives_dir
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let cp_sep = if cfg!(windows) { ";" } else { ":" };

        arg.replace("${natives_directory}", &natives_dir.to_string_lossy())
            .replace("${launcher_name}", "AlephLauncher")
            .replace("${launcher_version}", "1.0.0")
            .replace("${classpath}", classpath)
            .replace("${library_directory}", &lib_dir)
            .replace("${classpath_separator}", cp_sep)
    }

    pub fn build_game_args(
        options: &LaunchOptions,
        game_dir: &Path,
        assets_dir: &Path,
        details: &VersionDetails,
        session: &AuthSession,
        features: &HashMap<String, bool>,
    ) -> Vec<String> {
        let mut args = Vec::new();
        let asset_index_name = details
            .asset_index
            .as_ref()
            .map(|a| a.id.as_str())
            .unwrap_or("legacy");

        let mut replacements: HashMap<&str, String> = HashMap::new();
        replacements.insert("${auth_player_name}", session.username.clone());
        replacements.insert("${version_name}", details.id.clone());
        replacements.insert("${game_directory}", game_dir.to_string_lossy().to_string());
        replacements.insert("${assets_root}", assets_dir.to_string_lossy().to_string());
        replacements.insert("${game_assets}", assets_dir.to_string_lossy().to_string());
        replacements.insert("${assets_index_name}", asset_index_name.to_string());
        replacements.insert("${auth_uuid}", session.uuid.clone());
        replacements.insert("${auth_access_token}", session.access_token.clone());
        replacements.insert("${user_type}", session.user_type.clone());
        replacements.insert("${version_type}", "Aleph".to_string());
        replacements.insert("${user_properties}", "{}".to_string());

        if let Some(xuid) = &session.xuid {
            replacements.insert("${auth_xuid}", xuid.clone());
        } else {
            replacements.insert("${auth_xuid}", "0".to_string());
        }

        // 1. Modern arguments structure
        if let Some(version_args) = &details.arguments {
            for arg_val in &version_args.game {
                match arg_val {
                    ArgumentValue::Simple(s) => {
                        args.push(Self::replace_templates(s, &replacements));
                    }
                    ArgumentValue::Complex { rules, value } => {
                        if allows(rules.as_deref(), features) {
                            match value {
                                ArgumentValueNested::Single(s) => {
                                    args.push(Self::replace_templates(s, &replacements));
                                }
                                ArgumentValueNested::Multiple(list) => {
                                    for s in list {
                                        args.push(Self::replace_templates(s, &replacements));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // 2. Legacy minecraftArguments string (1.12 and earlier)
        else if let Some(legacy_args) = &details.minecraft_arguments {
            for token in legacy_args.split_whitespace() {
                args.push(Self::replace_templates(token, &replacements));
            }
        }

        // Screen resolution
        if options.fullscreen {
            args.push("--fullscreen".to_string());
        } else {
            args.push("--width".to_string());
            args.push(options.window_width.to_string());
            args.push("--height".to_string());
            args.push(options.window_height.to_string());
        }

        // Direct Server / Quick Play
        if let Some(server) = &options.quick_play_server {
            let port = options.quick_play_port.unwrap_or(25565);
            // Modern 1.20+ Quick Play argument
            args.push("--quickPlayMultiplayer".to_string());
            args.push(format!("{server}:{port}"));
            // Legacy 1.12-1.19 server/port argument
            args.push("--server".to_string());
            args.push(server.clone());
            args.push("--port".to_string());
            args.push(port.to_string());
        }

        args
    }

    fn replace_templates(text: &str, replacements: &HashMap<&str, String>) -> String {
        let mut result = text.to_string();
        for (key, val) in replacements {
            if result.contains(key) {
                result = result.replace(key, val);
            }
        }
        result
    }

    pub fn build_classpath(libraries: &[PathBuf], client_jar: &Path) -> String {
        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let mut parts = Vec::with_capacity(libraries.len() + 1);

        for lib in libraries {
            if lib.is_file() {
                parts.push(lib.to_string_lossy().to_string());
            }
        }
        parts.push(client_jar.to_string_lossy().to_string());

        parts.join(separator)
    }
}
