//! Headless CLI for the Aleph Minecraft Client Launcher (CONCEPT "CLI-режим", P14).
//! Same backend as the GUI: manifest → Java → download → launch.
//! Exit codes: 0 ok, 1 usage/error, 2 launch failure.

use amc_core::paths::LauncherPaths;
use amc_core::{init_logging, LauncherConfig};
use amc_downloader::{AdoptiumInstaller, DownloadCancel, DownloadEngine};
use amc_minecraft::{MinecraftLauncher, VersionDetails};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() -> &'static str {
    "Aleph Minecraft Client Launcher (headless CLI)\n\
     \n\
     Usage:\n  \
       amc --help                    this help\n  \
       amc --version                 launcher version\n  \
       amc --list                    instances and versions\n  \
       amc --launch <name-or-id>     launch an instance (or version id) and wait\n  \
       amc --launch <id> --server <host[:port]>   quick-play connect\n\
     \n\
     The CLI never opens a window; game output streams to stdout."
}

fn resolve_paths() -> LauncherPaths {
    if let Ok(root) = std::env::var("AMC_ROOT") {
        if let Ok(paths) = LauncherPaths::custom(root) {
            return paths;
        }
    }
    LauncherPaths::default_paths().unwrap_or_else(|_| {
        LauncherPaths::custom("./AlephLauncher").expect("unwritable fallback root")
    })
}

fn find_instance<'a>(
    instances: &'a [amc_core::types::Instance],
    key: &str,
) -> Option<&'a amc_core::types::Instance> {
    instances
        .iter()
        .find(|i| i.id.to_string() == key || i.name.eq_ignore_ascii_case(key))
}

async fn run_launch(
    paths: &LauncherPaths,
    config: &LauncherConfig,
    key: &str,
    quick_server: Option<(String, u16)>,
) -> i32 {
    let instances =
        amc_core::types::Instance::load_all(&paths.instances_file()).unwrap_or_default();
    let (game_dir, ver_id, loader, ram_mb) = match find_instance(&instances, key) {
        Some(inst) => {
            let _ = inst.ensure_directories(&paths.instances_dir());
            (
                inst.get_game_dir(&paths.instances_dir()),
                inst.game_version.clone(),
                inst.loader,
                inst.ram_mb,
            )
        }
        None => {
            // Bare version id: vanilla quick launch into a version folder.
            (
                paths.instances_dir().join(key),
                key.to_string(),
                amc_core::types::LoaderType::Vanilla,
                None,
            )
        }
    };

    let client = reqwest::Client::new();
    let engine = DownloadEngine::new(16);
    let cancel = DownloadCancel::new();

    println!("Resolving {ver_id}...");
    let details =
        match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Version resolve failed: {e}");
                return 2;
            }
        };
    if loader != amc_core::types::LoaderType::Vanilla {
        eprintln!("Note: CLI launches vanilla; loader {:?} ignored", loader);
    }

    let mut options = config.default_launch_options.clone();
    if let Some(ram) = ram_mb {
        options.memory_max_mb = ram;
    }
    if let Some((srv, port)) = quick_server {
        options.quick_play_server = Some(srv);
        options.quick_play_port = Some(port);
    }

    let java_bin = if let Some(custom) = &options.java_path {
        custom.clone()
    } else {
        match AdoptiumInstaller::ensure_java(
            &paths.runtimes_dir(),
            details.required_java_major(),
            &engine,
            None,
        )
        .await
        {
            Ok(bin) => bin,
            Err(e) => {
                eprintln!("Java setup failed: {e}");
                return 2;
            }
        }
    };

    let mut items = Vec::new();
    if let Some(downloads) = &details.downloads {
        if let Some(client_file) = &downloads.client {
            let dest = paths
                .versions_dir()
                .join(&ver_id)
                .join(format!("{ver_id}.jar"));
            if !dest.is_file() {
                let mut item = amc_downloader::DownloadItem::new(&client_file.url, dest);
                if let Some(sha1) = &client_file.sha1 {
                    item = item.with_sha1(sha1);
                }
                if let Some(size) = client_file.size {
                    item = item.with_size(size);
                }
                items.push(item);
            }
        }
    }
    for lib in &details.libraries {
        if !lib.is_allowed(&std::collections::HashMap::new()) {
            continue;
        }
        if let Some(item) = lib.to_download_item(&paths.libraries_dir()) {
            if !item.destination.is_file() {
                items.push(item);
            }
        }
    }
    if !items.is_empty() {
        println!("Downloading {} files...", items.len());
        if let Err(e) = engine.download_all_with_progress(items, None, cancel).await {
            eprintln!("Download failed: {e}");
            return 2;
        }
    }

    println!("Starting Minecraft {ver_id}...");
    let session = match amc_auth::AccountManager::load_from_path(paths.accounts_file())
        .ok()
        .and_then(|m| m.get_session())
    {
        Some(s) => s,
        None => {
            eprintln!("No signed-in account; sign in via the GUI first");
            return 2;
        }
    };

    let natives_dir = paths.libraries_dir().join("natives").join(&ver_id);
    let _ = std::fs::create_dir_all(&natives_dir);
    let classpath: Vec<std::path::PathBuf> = details
        .libraries
        .iter()
        .filter_map(|lib| lib.to_download_item(&paths.libraries_dir()))
        .map(|item| item.destination)
        .collect();
    let client_jar = paths
        .versions_dir()
        .join(&ver_id)
        .join(format!("{ver_id}.jar"));

    match MinecraftLauncher::launch(
        &java_bin,
        &game_dir,
        &paths.assets_dir(),
        &natives_dir,
        &classpath,
        &client_jar,
        &details,
        &session,
        &options,
    )
    .await
    {
        Ok(mut rx) => {
            while let Some(event) = rx.recv().await {
                match event {
                    amc_minecraft::GameEvent::LogLine(line) => println!("{line}"),
                    amc_minecraft::GameEvent::Started { pid } => {
                        println!("Minecraft started, pid {pid}")
                    }
                    amc_minecraft::GameEvent::Exited { code } => {
                        println!("Minecraft exited: {code:?}");
                        return if code == Some(0) { 0 } else { 2 };
                    }
                    amc_minecraft::GameEvent::Crashed { message } => {
                        eprintln!("Minecraft crashed: {message}");
                        return 2;
                    }
                    amc_minecraft::GameEvent::Cancelled
                    | amc_minecraft::GameEvent::DownloadPaused => return 2,
                }
            }
            0
        }
        Err(e) => {
            eprintln!("Launch failed: {e}");
            2
        }
    }
}

#[tokio::main]
async fn main() {
    let paths = resolve_paths();
    init_logging(Some(&paths.logs_dir()));
    let args: Vec<String> = std::env::args().skip(1).collect();

    let code = match args
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["--help"] | ["-h"] => {
            println!("{}", usage());
            0
        }
        ["--version"] | ["-V"] => {
            println!("amc {VERSION}");
            0
        }
        ["--list"] => {
            let instances =
                amc_core::types::Instance::load_all(&paths.instances_file()).unwrap_or_default();
            if instances.is_empty() {
                println!("No instances yet");
            }
            for inst in &instances {
                println!(
                    "{}  {}  {} {:?}  {} MB RAM",
                    inst.id,
                    inst.name,
                    inst.game_version,
                    inst.loader,
                    inst.ram_mb.unwrap_or(0)
                );
            }
            0
        }
        ["--launch", key] => {
            let config = LauncherConfig::load_from_path(&paths.config_file()).unwrap_or_default();
            run_launch(&paths, &config, key, None).await
        }
        ["--launch", key, "--server", addr] => {
            let (host, port) = match addr.rsplit_once(':') {
                Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(25565)),
                None => (addr.to_string(), 25565),
            };
            let config = LauncherConfig::load_from_path(&paths.config_file()).unwrap_or_default();
            run_launch(&paths, &config, key, Some((host, port))).await
        }
        _ => {
            eprintln!("{}", usage());
            1
        }
    };
    std::process::exit(code);
}
