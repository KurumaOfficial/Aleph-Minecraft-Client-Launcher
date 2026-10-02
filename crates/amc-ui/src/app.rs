use amc_auth::{Account, AccountManager, DeviceCodeResponse, MicrosoftAuthFlow};
use amc_core::config::LauncherConfig;
use amc_core::error::LauncherError;
use amc_core::paths::LauncherPaths;
use amc_core::types::{GameVersion, Instance, LaunchOptions, LoaderType};
use amc_core::HardwareReport;
use amc_downloader::{AdoptiumInstaller, DownloadCancel, DownloadEngine, DownloadProgress};
use amc_minecraft::{
    FabricLoader, ForgeLoader, GameEvent, MinecraftLauncher, NeoForgeLoader, OptiFineLoader,
    QuiltLoader, VersionDetails, VersionManifest,
};
use amc_mods::{
    CurseForgeClient, LocalModManager, ModCategory, ModDownloadFile, ModSearchResult, ModSource,
    ModrinthClient,
};
use eframe::App;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel, ViewportCommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

use crate::modals::{
    ConsoleModal, DownloadOverlay, HwNoticeAction, IdentityPicker, LoginModal, LoginMode,
    OverlayAction, PickerAction, ScreensModal, WizardAction, WizardFlow,
};
use crate::pages::{
    HomeContext, HomePage, InstanceAction, InstancesContext, InstancesPage, ModsPage,
    ProfileAction, ProfilePage, SettingsPage, SkinsPage,
};
use crate::theme::{apply_aleph_theme, BG};
use crate::widgets::{
    push_toast, show_toasts, topbar, BottomBar, NavTab, Sidebar, TitleBar, Toast, ToastKind,
};

pub struct LauncherApp {
    paths: LauncherPaths,
    config: LauncherConfig,
    account_mgr: AccountManager,
    download_engine: Arc<DownloadEngine>,

    // App state
    current_tab: NavTab,
    selected_version: Option<String>,
    selected_instance: Option<uuid::Uuid>,
    versions: Vec<GameVersion>,
    instances: Vec<Instance>,
    is_launching: bool,
    launch_status_text: String,

    // Pages
    home_page: HomePage,
    instances_page: InstancesPage,
    mods_page: ModsPage,
    skins_page: SkinsPage,
    settings_page: SettingsPage,

    // Modals
    login_modal: LoginModal,
    console_modal: ConsoleModal,
    screens_modal: ScreensModal,

    // First-run wizard (ROADMAP P1) and the one-time hardware notice.
    wizard: Option<WizardFlow>,
    hw_notice: Option<HardwareReport>,
    /// Legacy `.minecraft` reuse offer (CONCEPT P1).
    mcreuse_path: Option<PathBuf>,
    mcreuse_rx: Option<mpsc::Receiver<(u64, u64)>>,

    // Dynamic textures
    avatar_texture: Option<egui::TextureHandle>,

    // Session stats & tracking
    session_start_time: Option<std::time::Instant>,
    launched_instance_id: Option<uuid::Uuid>,
    pending_direct_connect: Option<(String, u16)>,

    // Async channels
    versions_rx: Option<mpsc::Receiver<Vec<GameVersion>>>,
    server_ping_rx: Option<mpsc::Receiver<amc_minecraft::ServerStatus>>,
    mod_search_rx: Option<mpsc::Receiver<Vec<ModSearchResult>>>,
    mod_install_rx: Option<mpsc::Receiver<Result<InstallReport, String>>>,
    wishlist_snapshot: Vec<String>,
    favorites_snapshot: Vec<amc_minecraft::FavoriteServerEntry>,
    mod_update_rx: Option<mpsc::Receiver<Vec<crate::pages::mods::UpdateOffer>>>,
    fav_ping_rx: Option<mpsc::Receiver<Vec<(String, amc_minecraft::ServerStatus)>>>,
    update_rx: Option<mpsc::Receiver<Option<String>>>,
    toasts: Vec<Toast>,
    pending_identity_pick: bool,
    identity_picker: IdentityPicker,
    picker_nickname: String,
    favorites: Vec<amc_minecraft::FavoriteServerEntry>,
    profile_page: ProfilePage,
    server_child: Option<tokio::process::Child>,
    server_status: Option<String>,
    server_rx: Option<mpsc::Receiver<Result<tokio::process::Child, String>>>,
    progress_init_rx: Option<mpsc::Receiver<Option<watch::Receiver<DownloadProgress>>>>,
    launch_status_rx: Option<mpsc::Receiver<String>>,
    download_progress_rx: Option<watch::Receiver<DownloadProgress>>,
    game_events_rx: Option<mpsc::Receiver<GameEvent>>,
    /// Cancel token for the current launch download (overlay Cancel button).
    download_cancel: Option<DownloadCancel>,
    /// Set while a download is paused (overlay stays open with Resume).
    download_paused: bool,
    /// Clean-exit session summary modal (CONCEPT "После запуска игры").
    session_summary: Option<(String, u64)>,
    ms_code_rx: Option<mpsc::Receiver<Result<DeviceCodeResponse, String>>>,
    ms_poll_rx: Option<mpsc::Receiver<Result<Account, String>>>,
}

/// Outcome of one mod installation (CONCEPT P4): downloaded files plus
/// provenance for the local-mod badge and old-file cleanup on updates.
struct InstallReport {
    mod_id: String,
    title: String,
    filenames: Vec<String>,
    replaced_old: Option<String>,
    instance_id: Option<uuid::Uuid>,
    target_dir: std::path::PathBuf,
    mc_version: Option<String>,
    loader_name: String,
}

impl LauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: LauncherPaths) -> Self {
        apply_aleph_theme(&cc.egui_ctx);

        let config = LauncherConfig::load_from_path(&paths.config_file()).unwrap_or_default();
        let account_mgr = Self::load_accounts(&paths);

        let download_engine = Arc::new(DownloadEngine::new(16));

        // Pre-scan local mods
        let mods_dir = paths.root_dir.join("mods");
        let local_mods = LocalModManager::scan_mods(&mods_dir).unwrap_or_default();
        let mut mods_page = ModsPage::default();
        mods_page.local_mods = local_mods;

        // Load persisted instances
        let instances = Instance::load_all(&paths.instances_file()).unwrap_or_default();
        let selected_instance = instances.first().map(|i| i.id);
        let selected_version = instances
            .first()
            .map(|i| i.game_version.clone())
            .or_else(|| Some("1.20.1".to_string()));

        let lang = config.ui.language();

        // Professional mode opens on instances (computed before `config` moves
        // into the app struct below).
        let initial_tab = if config.app_mode == amc_core::config::AppMode::Professional {
            NavTab::Modpacks
        } else {
            NavTab::Home
        };

        // A fresh profile (no config file yet) starts with the setup wizard.
        let wizard = if config.first_run {
            Some(WizardFlow::new(config.ui.language(), config.app_mode))
        } else {
            None
        };

        // Values derived from `config`/`paths` must be cloned before the
        // struct literal moves them.
        let wishlist_snapshot = config.wishlist.clone();
        let favorites = amc_minecraft::load_favorites(&paths.root_dir);

        Self {
            paths,
            config,
            account_mgr,
            download_engine,
            // Professional mode opens on instances, Simple on versions (ROADMAP P1:
            // different home screens start here; full split follows in P2).
            current_tab: initial_tab,
            selected_version,
            selected_instance,
            versions: Vec::new(),
            instances,
            is_launching: false,
            launch_status_text: lang.status_preparing().to_string(),
            home_page: HomePage::default(),
            instances_page: InstancesPage::default(),
            mods_page,
            skins_page: SkinsPage::default(),
            settings_page: SettingsPage::default(),
            login_modal: LoginModal::default(),
            console_modal: ConsoleModal::default(),
            screens_modal: ScreensModal::default(),
            wizard,
            hw_notice: None,
            mcreuse_path: None,
            mcreuse_rx: None,
            avatar_texture: None,
            session_start_time: None,
            launched_instance_id: None,
            versions_rx: None,
            server_ping_rx: None,
            mod_search_rx: None,
            mod_install_rx: None,
            wishlist_snapshot,
            favorites_snapshot: favorites.clone(),
            mod_update_rx: None,
            fav_ping_rx: None,
            update_rx: None,
            toasts: Vec::new(),
            pending_identity_pick: false,
            identity_picker: IdentityPicker::default(),
            picker_nickname: String::new(),
            favorites,
            profile_page: ProfilePage::default(),
            server_child: None,
            server_status: None,
            server_rx: None,
            progress_init_rx: None,
            launch_status_rx: None,
            download_progress_rx: None,
            game_events_rx: None,
            download_cancel: None,
            download_paused: false,
            session_summary: None,
            ms_code_rx: None,
            ms_poll_rx: None,
            pending_direct_connect: None,
        }
    }

    /// Store an account, surfacing storage errors (e.g. a second WetID,
    /// which the concept forbids) in the login modal instead of dropping
    /// them silently. Returns success.
    fn store_account(&mut self, account: Account) -> bool {
        match self.account_mgr.add_or_update_account(account) {
            Ok(()) => true,
            Err(e) => {
                self.login_modal.error_msg = Some(e.to_string());
                self.login_modal.is_open = true;
                false
            }
        }
    }

    /// Add playtime to an instance, merging with whatever is on disk first
    /// so parallel sessions (including the same instance ×N) never lose
    /// minutes to a last-writer-wins overwrite.
    fn add_playtime(
        paths: &LauncherPaths,
        instances: &mut [Instance],
        inst_id: uuid::Uuid,
        mins: u64,
    ) {
        let now = chrono::Utc::now();
        let instances_file = paths.instances_file();
        let mut disk = Instance::load_all(&instances_file).unwrap_or_default();
        let mut applied = false;
        if let Some(inst) = disk.iter_mut().find(|i| i.id == inst_id) {
            inst.total_played_minutes += mins;
            inst.last_played = Some(now);
            applied = true;
        }
        if applied {
            let _ = Instance::save_all(&instances_file, &disk);
        }
        if let Some(inst) = instances.iter_mut().find(|i| i.id == inst_id) {
            if applied {
                if let Some(saved) = disk.iter().find(|i| i.id == inst_id) {
                    inst.total_played_minutes = saved.total_played_minutes;
                    inst.last_played = saved.last_played;
                }
            } else {
                inst.total_played_minutes += mins;
                inst.last_played = Some(now);
                let _ = Instance::save_all(&instances_file, instances);
            }
        }
    }

    /// Archive a crash log locally when reports are enabled (opt-out).
    /// Nothing leaves the machine — upload needs a future endpoint.
    fn archive_crash(paths: &LauncherPaths, message: &str) {
        let dir = paths.logs_dir().join("crashes");
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        // Keep the archive bounded: drop oldest beyond 20 files.
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut logs: Vec<_> = entries.flatten().collect();
            if logs.len() >= 20 {
                logs.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                for stale in logs.iter().take(logs.len().saturating_sub(19)) {
                    let _ = std::fs::remove_file(stale.path());
                }
            }
        }
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let _ = std::fs::write(dir.join(format!("crash-{stamp}.log")), message);
    }

    /// Load stored accounts, degrading gracefully to an empty (signed-out)
    /// manager when the storage is missing or unreadable — never a crash.
    fn load_accounts(paths: &LauncherPaths) -> AccountManager {
        if let Ok(mgr) = AccountManager::load_from_path(paths.accounts_file()) {
            return mgr;
        }
        tracing::warn!("Primary accounts file unreadable, trying ./accounts.json");
        AccountManager::load_from_path("accounts.json").unwrap_or_else(|_| {
            tracing::error!("Accounts storage unreadable, starting signed-out");
            AccountManager::new("accounts.json")
        })
    }

    /// Persist the wizard choices, clear the first-run flag, and schedule the
    /// weak-hardware notice when the machine is below target. Never blocks.
    fn complete_wizard(&mut self) {
        if let Some(flow) = self.wizard.take() {
            self.config.ui.set_language(flow.language);
            self.config.app_mode = flow.mode;
            self.current_tab = if flow.mode == amc_core::config::AppMode::Professional {
                NavTab::Modpacks
            } else {
                NavTab::Home
            };
        }
        self.config.first_run = false;
        let _ = self.config.save_to_path(&self.paths.config_file());
        let report = HardwareReport::gather(&self.paths.root_dir);
        if report.is_weak() || report.is_low_disk() {
            self.hw_notice = Some(report);
        }
        // Offer to reuse another launcher's `.minecraft` instead of
        // re-downloading shared files (CONCEPT "Первый запуск", P1).
        if !self.config.mc_reuse_done {
            let legacy = LauncherPaths::default_minecraft_dir();
            if legacy.is_dir() && amc_core::looks_like_minecraft(&legacy) {
                self.mcreuse_path = Some(legacy);
            } else {
                self.config.mc_reuse_done = true;
                let _ = self.config.save_to_path(&self.paths.config_file());
            }
        }
    }

    pub fn ping_featured_server(&mut self) {
        self.home_page.is_pinging = true;
        let (tx, rx) = mpsc::channel(1);
        self.server_ping_rx = Some(rx);

        tokio::spawn(async move {
            if let Ok(status) = amc_minecraft::ServerPinger::ping("mc.hypixel.net", 25565).await {
                let _ = tx.send(status).await;
            }
        });
    }

    pub fn load_initial_manifest(&mut self) {
        let client = reqwest::Client::new();
        let cache_dir = self.paths.cache_dir();
        let (tx, rx) = mpsc::channel(1);
        self.versions_rx = Some(rx);

        tokio::spawn(async move {
            if let Ok(manifest) = VersionManifest::fetch(&client, Some(&cache_dir)).await {
                let versions = manifest.to_game_versions();
                tracing::info!("Loaded {} versions from Mojang manifest", versions.len());
                let _ = tx.send(versions).await;
            }
        });
    }

    /// Check installed mods against Modrinth (CONCEPT P4): exact-slug match,
    /// compatible with the selected instance, newer version number = offer.
    /// Heuristic and documented as such — a wrong guess just shows no offer.
    fn check_mod_updates(&mut self) {
        if self.mods_page.update_checking {
            return;
        }
        self.mods_page.update_checking = true;
        let (tx, rx) = mpsc::channel(1);
        self.mod_update_rx = Some(rx);
        let local: Vec<(String, String, String)> = self
            .mods_page
            .local_mods
            .iter()
            .map(|m| {
                (
                    m.name.clone(),
                    m.version.clone(),
                    m.path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                )
            })
            .collect();
        let (mc_version, loader_name) = if let Some(id) = self.selected_instance {
            if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                (
                    Some(inst.game_version.clone()),
                    inst.loader.as_str().to_string(),
                )
            } else {
                (None, "minecraft".to_string())
            }
        } else {
            (None, "minecraft".to_string())
        };

        tokio::spawn(async move {
            let client = ModrinthClient::default();
            let mut offers = Vec::new();
            for (name, version, old_filename) in local {
                let slug_guess = name.to_lowercase().replace(' ', "-");
                let hits = match client.search(&name, None, None, ModCategory::Mod, 3).await {
                    Ok(h) => h,
                    Err(_) => continue,
                };
                let hit = match hits
                    .into_iter()
                    .find(|h| h.id == slug_guess || h.title.eq_ignore_ascii_case(&name))
                {
                    Some(h) => h,
                    None => continue,
                };
                let versions = match client.list_versions(&hit.id).await {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let picked = match &mc_version {
                    Some(mc) => {
                        ModrinthClient::select_compatible(&versions, mc, &loader_name).cloned()
                    }
                    None => versions.into_iter().next(),
                };
                let ver = match picked {
                    Some(v) => v,
                    None => continue,
                };
                if ver.version_number == version {
                    continue;
                }
                let Some(file) = ver
                    .files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| ver.files.first())
                else {
                    continue;
                };
                offers.push(crate::pages::mods::UpdateOffer {
                    slug: hit.id,
                    title: hit.title,
                    version: ver.version_number,
                    url: file.url.clone(),
                    filename: file.filename.clone(),
                    sha1: file.hashes.sha1.clone(),
                    size: Some(file.size),
                    old_filename,
                });
            }
            let _ = tx.send(offers).await;
        });
    }

    /// Check GitHub Releases for a newer launcher (CONCEPT P11).
    /// Result arrives as a toast + settings note; download is manual.
    pub fn check_launcher_update(&mut self) {
        let (tx, rx) = mpsc::channel(1);
        self.update_rx = Some(rx);
        tokio::spawn(async move {
            #[derive(serde::Deserialize)]
            struct Release {
                #[serde(default)]
                tag_name: String,
            }
            let found: Option<String> = async {
                let client = reqwest::Client::new();
                let res = client
                    .get("https://api.github.com/repos/KurumaOfficial/Aleph-Minecraft-Client-Launcher/releases/latest")
                    .header("User-Agent", "amc-launcher")
                    .send()
                    .await
                    .ok()?;
                if !res.status().is_success() {
                    return None;
                }
                let rel: Release = res.json().await.ok()?;
                let latest = rel.tag_name.trim_start_matches(['v', 'V']).to_string();
                if latest.is_empty() || latest == env!("CARGO_PKG_VERSION") {
                    None
                } else {
                    Some(latest)
                }
            }
            .await;
            let _ = tx.send(found).await;
        });
    }

    /// Export logs + config + instances as a support bundle (P12).
    /// Runs inline: small text files only.
    fn export_diagnostics(&mut self) {
        let lang = self.config.ui.language();
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let dest = self.paths.root_dir.join(format!("diag-{stamp}.zip"));
        let result: Result<PathBuf, String> = (|| {
            let file = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let mut add_tree = |dir: &PathBuf, prefix: &str| -> Result<(), String> {
                let mut stack = vec![dir.clone()];
                while let Some(current) = stack.pop() {
                    let entries = std::fs::read_dir(&current).map_err(|e| e.to_string())?;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else if path.is_file() {
                            if entry.metadata().map(|m| m.len()).unwrap_or(0) > 5 * 1024 * 1024 {
                                continue;
                            }
                            let rel = path.strip_prefix(&self.paths.root_dir).unwrap_or(&path);
                            let name =
                                format!("{prefix}/{}", rel.to_string_lossy().replace('\\', "/"));
                            zip.start_file(name, opts).map_err(|e| e.to_string())?;
                            let mut src = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                            std::io::copy(&mut src, &mut zip).map_err(|e| e.to_string())?;
                        }
                    }
                }
                Ok(())
            };
            add_tree(&self.paths.logs_dir(), "logs")?;
            for single in [
                self.paths.config_file(),
                self.paths.instances_file(),
                self.paths.accounts_file(),
            ] {
                if single.is_file() {
                    let name = format!(
                        "root/{}",
                        single.file_name().unwrap_or_default().to_string_lossy()
                    );
                    zip.start_file(name, opts).map_err(|e| e.to_string())?;
                    let mut src = std::fs::File::open(&single).map_err(|e| e.to_string())?;
                    std::io::copy(&mut src, &mut zip).map_err(|e| e.to_string())?;
                }
            }
            zip.finish().map_err(|e| e.to_string())?;
            Ok(dest)
        })();
        match result {
            Ok(path) => push_toast(
                &mut self.toasts,
                &self.config.notifications,
                ToastKind::Launcher,
                format!("{}: {}", lang.crash_export(), path.display()),
            ),
            Err(e) => tracing::error!("Diagnostics export failed: {e}"),
        }
    }

    /// Start a local dedicated server for the selected instance (CONCEPT P13).
    /// Downloads `server.jar`, writes `eula.txt` only after explicit UI
    /// consent, resolves Java like the game does. The owned child uses
    /// `kill_on_drop`, so the server always stops with the launcher.
    fn start_local_server(&mut self, eula_ok: bool) {
        // A fresh start always kills the previous server first.
        self.server_child = None;
        self.server_status = None;
        let (tx, rx) = mpsc::channel(1);
        self.server_rx = Some(rx);

        let instances_dir = self.paths.instances_dir();
        let picked = self
            .selected_instance
            .and_then(|id| self.instances.iter().find(|i| i.id == id));
        let (game_dir, ver_id, mem_mb) = match picked {
            Some(inst) => (
                inst.get_game_dir(&instances_dir),
                inst.game_version.clone(),
                inst.ram_mb.unwrap_or(2048),
            ),
            None => {
                let ver = self
                    .selected_version
                    .clone()
                    .unwrap_or_else(|| "1.20.1".to_string());
                (instances_dir.join(&ver), ver, 2048)
            }
        };

        let engine = self.download_engine.clone();
        let paths = self.paths.clone();
        let java_override = self.config.default_launch_options.java_path.clone();
        tokio::spawn(async move {
            let outcome: Result<tokio::process::Child, String> = async {
                let client = reqwest::Client::new();
                let details =
                    VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir())
                        .await
                        .map_err(|e| e.to_string())?;
                let server_url = details
                    .downloads
                    .as_ref()
                    .and_then(|d| d.server.as_ref())
                    .map(|f| f.url.clone())
                    .unwrap_or_else(|| amc_minecraft::DedicatedServer::mojang_server_url(&ver_id));
                let server_dir = amc_minecraft::DedicatedServer::dir(&game_dir);
                let jar = amc_minecraft::DedicatedServer::ensure_server_jar(
                    &client,
                    &server_url,
                    &server_dir,
                )
                .await
                .map_err(|e| e.to_string())?;
                if !amc_minecraft::DedicatedServer::eula_accepted(&server_dir) {
                    if !eula_ok {
                        return Err("Accept the Mojang EULA to start the server".to_string());
                    }
                    amc_minecraft::DedicatedServer::write_eula(&server_dir)
                        .map_err(|e| e.to_string())?;
                }
                let java_bin = if let Some(custom) = java_override {
                    custom
                } else if let Some(sys) =
                    amc_downloader::find_system_java(details.required_java_major()).await
                {
                    sys
                } else {
                    AdoptiumInstaller::ensure_java(
                        &paths.runtimes_dir(),
                        details.required_java_major(),
                        &engine,
                        None,
                    )
                    .await
                    .map_err(|e| e.to_string())?
                };
                amc_minecraft::DedicatedServer::start(&java_bin, &server_dir, &jar, mem_mb, 25565)
                    .map_err(|e| e.to_string())
            }
            .await;
            let _ = tx.send(outcome).await;
        });
    }

    fn search_mods(&mut self, query: String, provider: ModSource, category: ModCategory) {
        self.mods_page.is_searching = true;
        let (tx, rx) = mpsc::channel(1);
        self.mod_search_rx = Some(rx);

        tokio::spawn(async move {
            match provider {
                ModSource::CurseForge => {
                    let client = CurseForgeClient::default();
                    match client.search(&query, None, None, 30).await {
                        Ok(results) => {
                            let _ = tx.send(results).await;
                        }
                        Err(e) => {
                            tracing::error!("CurseForge search failed: {e}");
                            let _ = tx.send(Vec::new()).await;
                        }
                    }
                }
                _ => {
                    let client = ModrinthClient::default();
                    // Empty query = Discover: most downloaded first.
                    let result = if query.trim().is_empty() {
                        client.search_trending(category, 20).await
                    } else {
                        client.search(&query, None, None, category, 30).await
                    };
                    match result {
                        Ok(results) => {
                            let _ = tx.send(results).await;
                        }
                        Err(e) => {
                            tracing::error!("Modrinth search failed: {e}");
                            let _ = tx.send(Vec::new()).await;
                        }
                    }
                }
            }
        });
    }

    /// Drag-and-drop entry: `.mrpack` imports a pack, `.jar` installs a mod
    /// file into the selected instance, a folder with `level.dat` imports
    /// a world. Everything else is ignored with a toast.
    fn handle_dropped_files(&mut self, files: Vec<std::path::PathBuf>) {
        for path in files {
            if path.is_dir() {
                if path.join("level.dat").is_file() {
                    self.import_dropped_world(path);
                }
                continue;
            }
            match path.extension().and_then(|e| e.to_str()) {
                Some(ext) if ext.eq_ignore_ascii_case("mrpack") => self.import_mrpack(path),
                Some(ext) if ext.eq_ignore_ascii_case("jar") => self.install_loose_jar(path),
                _ => {}
            }
        }
    }

    fn content_base_dir(&self) -> std::path::PathBuf {
        if let Some(id) = self.selected_instance {
            if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                return inst.get_game_dir(&self.paths.instances_dir());
            }
        }
        self.paths.root_dir.clone()
    }

    /// Copy a loose `.jar` into the active mods folder. Local by definition,
    /// so it is deliberately NOT recorded as launcher-installed.
    fn install_loose_jar(&mut self, path: std::path::PathBuf) {
        let Some(filename) = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
        else {
            return;
        };
        let target = self.content_base_dir().join("mods");
        let _ = std::fs::create_dir_all(&target);
        let lang = self.config.ui.language();
        match std::fs::copy(&path, target.join(&filename)) {
            Ok(_) => {
                self.mods_page.local_mods = LocalModManager::scan_mods(&target).unwrap_or_default();
                self.instances_page.refresh_scans();
                push_toast(
                    &mut self.toasts,
                    &self.config.notifications,
                    ToastKind::Downloads,
                    format!("Мод \"{filename}\" установлен!"),
                );
                let _ = lang;
            }
            Err(e) => {
                self.mods_page.status_message =
                    Some((format!("Не удалось скопировать {filename}: {e}"), false));
            }
        }
    }

    /// Import a dropped world folder into the active instance `saves/`.
    fn import_dropped_world(&mut self, folder: std::path::PathBuf) {
        let name = folder
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "world".to_string());
        let base = self.content_base_dir();
        let saves = base.join("saves");
        let _ = std::fs::create_dir_all(&saves);
        let mut dest = saves.join(&name);
        let mut n = 1;
        while dest.exists() {
            n += 1;
            dest = saves.join(format!("{name}-{n}"));
        }
        let lang = self.config.ui.language();
        match copy_dir_recursive(&folder, &dest) {
            Ok(()) => push_toast(
                &mut self.toasts,
                &self.config.notifications,
                ToastKind::Downloads,
                format!(
                    "Мир \"{}\" импортирован!",
                    dest.file_name().unwrap_or_default().to_string_lossy()
                ),
            ),
            Err(e) => {
                self.mods_page.status_message =
                    Some((format!("Не удалось импортировать мир: {e}"), false));
                let _ = lang;
            }
        }
    }

    /// Import a Modrinth `.mrpack`: download files, extract overrides.
    fn import_mrpack(&mut self, pack_path: std::path::PathBuf) {
        let base_dir = self.content_base_dir();
        let engine = self.download_engine.clone();
        let (tx, rx) = mpsc::channel(1);
        self.mod_install_rx = Some(rx);
        let lang = self.config.ui.language();
        let _ = lang;
        tokio::spawn(async move {
            let index = match amc_mods::parse_mrpack_index(&pack_path) {
                Ok(index) => index,
                Err(e) => {
                    let _ = tx.send(Err(format!("Bad .mrpack: {e}"))).await;
                    return;
                }
            };
            let mut ok_count = 0usize;
            for file in &index.files {
                let Some(rel) = amc_mods::MrpackIndex::sanitize_path(&file.path) else {
                    continue;
                };
                let Some(url) = file.downloads.first() else {
                    continue;
                };
                let dest = base_dir.join(&rel);
                if let Some(parent) = dest.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let mut item = amc_downloader::DownloadItem::new(url, &dest);
                if let Some(sha1) = file.hashes.as_ref().and_then(|h| h.sha1.as_ref()) {
                    item = item.with_sha1(sha1);
                }
                if let Some(size) = file.file_size {
                    item = item.with_size(size);
                }
                if engine
                    .download_one(&item, None, DownloadCancel::default())
                    .await
                    .is_ok()
                {
                    ok_count += 1;
                }
            }
            // Overrides tree (configs, resourcepacks, …).
            let overrides_done = (|| {
                let zip_file = std::fs::File::open(&pack_path).ok()?;
                let mut archive = zip::ZipArchive::new(zip_file).ok()?;
                for i in 0..archive.len() {
                    let mut entry = archive.by_index(i).ok()?;
                    let name = entry.name().to_string();
                    let Some(rest) = name.strip_prefix("overrides/") else {
                        continue;
                    };
                    let Some(rel) = amc_mods::MrpackIndex::sanitize_path(rest) else {
                        continue;
                    };
                    if name.ends_with('/') {
                        continue;
                    }
                    let dest = base_dir.join(&rel);
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent).ok()?;
                    }
                    let mut out = std::fs::File::create(&dest).ok()?;
                    std::io::copy(&mut entry, &mut out).ok()?;
                }
                Some(())
            })();
            let _ = overrides_done;
            let _ = tx
                .send(Ok(InstallReport {
                    mod_id: format!("mrpack:{}", index.name),
                    title: index.name.clone(),
                    filenames: Vec::new(),
                    replaced_old: None,
                    instance_id: None,
                    target_dir: base_dir.join("mods"),
                    mc_version: None,
                    loader_name: "minecraft".to_string(),
                }))
                .await;
            let _ = ok_count;
        });
    }

    /// One-click Sodium install for weak hardware (CONCEPT P1/P4).
    /// Reuses the standard install pipeline (compat check, hash verify,
    /// provenance), replacing any installed Sodium in place.
    fn install_sodium(&mut self) {
        let lang = self.config.ui.language();
        let loader = self
            .selected_instance
            .and_then(|id| self.instances.iter().find(|i| i.id == id))
            .map(|inst| inst.loader);
        let ok = matches!(loader, Some(LoaderType::Fabric | LoaderType::Quilt));
        if !ok {
            let msg = if loader.is_none() {
                lang.wizard_hw_no_instance().to_string()
            } else {
                lang.wizard_hw_bad_loader().to_string()
            };
            push_toast(
                &mut self.toasts,
                &self.config.notifications,
                ToastKind::Downloads,
                msg,
            );
            return;
        }
        // Replace the installed Sodium file instead of duplicating it.
        let old = self
            .mods_page
            .local_mods
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case("sodium"))
            .and_then(|m| m.path.file_name().map(|n| n.to_string_lossy().to_string()));
        self.install_mod(
            ModSearchResult {
                source: ModSource::Modrinth,
                id: "sodium".to_string(),
                title: "Sodium".to_string(),
                author: "JellySquid".to_string(),
                downloads: 0,
                description: String::new(),
                icon_url: None,
                category: ModCategory::Mod,
            },
            old,
        );
    }

    fn install_mod(&mut self, mod_item: ModSearchResult, replaced_old: Option<String>) {
        let base_dir = if let Some(id) = self.selected_instance {
            if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                inst.get_game_dir(&self.paths.instances_dir())
            } else {
                self.paths.root_dir.clone()
            }
        } else {
            self.paths.root_dir.clone()
        };

        let target_dir = match mod_item.category {
            ModCategory::ResourcePack => base_dir.join("resourcepacks"),
            ModCategory::Shader => base_dir.join("shaderpacks"),
            ModCategory::Mod => base_dir.join("mods"),
        };
        let _ = std::fs::create_dir_all(&target_dir);

        // Compatibility context from the selected instance (CONCEPT P4).
        // Without it we install latest and say compatibility is unknown.
        let (mc_version, loader_name) = if let Some(id) = self.selected_instance {
            if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                (
                    Some(inst.game_version.clone()),
                    inst.loader.as_str().to_string(),
                )
            } else {
                (None, "minecraft".to_string())
            }
        } else {
            (None, "minecraft".to_string())
        };
        let install_instance = self.selected_instance;
        let install_target = target_dir.clone();

        let engine = self.download_engine.clone();
        let (tx, rx) = mpsc::channel(1);
        self.mod_install_rx = Some(rx);

        tokio::spawn(async move {
            let outcome: Result<Vec<(ModDownloadFile, String)>, String> = async {
                let mut queue: Vec<(String, ModSource, ModCategory, Option<String>)> = vec![(
                    mod_item.id.clone(),
                    mod_item.source,
                    mod_item.category,
                    None,
                )];
                let mut installed: Vec<(ModDownloadFile, String)> = Vec::new();
                let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
                // Depth-1 autodeps: the root mod plus what it strictly requires.
                while let Some((slug, source, category, old_file)) = queue.pop() {
                    if !seen.insert(format!("{slug:?}{source:?}")) {
                        continue;
                    }
                    let (file, deps) =
                        resolve_mod_file(&slug, source, category, &mc_version, &loader_name)
                            .await?;
                    if installed.len() >= 8 {
                        break;
                    }
                    installed.push((file, old_file.unwrap_or_default()));
                    if installed.len() == 1 {
                        for dep in deps.into_iter().take(4) {
                            queue.push((dep, ModSource::Modrinth, ModCategory::Mod, None));
                        }
                    }
                }
                if installed.is_empty() {
                    return Err("Nothing to install".to_string());
                }
                Ok(installed)
            }
            .await;

            match outcome {
                Ok(files) => {
                    let mut names = Vec::new();
                    for (file_info, _old) in &files {
                        let dest = target_dir.join(&file_info.filename);
                        let mut item = amc_downloader::DownloadItem::new(&file_info.url, &dest);
                        if let Some(sha1) = &file_info.sha1 {
                            item = item.with_sha1(sha1);
                        }
                        if let Some(size) = file_info.size {
                            item = item.with_size(size);
                        }
                        if let Err(e) = engine
                            .download_one(&item, None, DownloadCancel::default())
                            .await
                        {
                            tracing::error!("Failed to download {}: {e}", file_info.filename);
                            let _ = tx
                                .send(Err(format!("Ошибка загрузки {}: {e}", file_info.filename)))
                                .await;
                            return;
                        }
                        names.push(file_info.filename.clone());
                    }
                    tracing::info!("Items {:?} installed successfully!", names);
                    let _ = tx
                        .send(Ok(InstallReport {
                            mod_id: mod_item.id.clone(),
                            title: mod_item.title.clone(),
                            filenames: names,
                            replaced_old,
                            instance_id: install_instance,
                            target_dir: install_target,
                            mc_version,
                            loader_name,
                        }))
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to get file info: {e}");
                    let _ = tx.send(Err(format!("Не удалось получить файл: {e}"))).await;
                }
            }

            async fn resolve_mod_file(
                slug: &str,
                source: ModSource,
                category: ModCategory,
                mc_version: &Option<String>,
                loader_name: &str,
            ) -> Result<(ModDownloadFile, Vec<String>), String> {
                match source {
                    ModSource::CurseForge => {
                        let client = CurseForgeClient::default();
                        let mc = mc_version.as_deref();
                        // CurseTools filters server-side; empty means incompatible.
                        let file = client
                            .get_download_file(slug, Some(loader_name), mc)
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok((file, Vec::new()))
                    }
                    _ => {
                        let client = ModrinthClient::default();
                        if category != ModCategory::Mod {
                            let file = client
                                .get_download_file(slug, None, mc_version.as_deref())
                                .await
                                .map_err(|e| e.to_string())?;
                            return Ok((file, Vec::new()));
                        }
                        let versions = client
                            .list_versions(slug)
                            .await
                            .map_err(|e| e.to_string())?;
                        match mc_version {
                            Some(mc) => {
                                let ver = amc_mods::ModrinthClient::select_compatible(
                                    &versions,
                                    mc,
                                    loader_name,
                                )
                                .ok_or_else(|| format!("No build for Minecraft {mc}"))?;
                                let file = mod_file_of(ver)?;
                                Ok((file, ver.required_dependencies()))
                            }
                            None => {
                                let ver = versions
                                    .into_iter()
                                    .next()
                                    .ok_or_else(|| "No compatible mod version found".to_string())?;
                                let file = mod_file_of(&ver)?;
                                Ok((file, ver.required_dependencies()))
                            }
                        }
                    }
                }
            }

            fn mod_file_of(ver: &amc_mods::ProjectVersionFull) -> Result<ModDownloadFile, String> {
                ver.files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| ver.files.first())
                    .map(|file| ModDownloadFile {
                        filename: file.filename.clone(),
                        url: file.url.clone(),
                        version: ver.version_number.clone(),
                        sha1: file.hashes.sha1.clone(),
                        size: Some(file.size),
                    })
                    .ok_or_else(|| "Mod version has no files".to_string())
            }
        });
    }

    /// Root folder for world backups (CONCEPT, P9): the user override
    /// from settings, or the launcher default `backups/` folder.
    fn world_backups_root(&self) -> PathBuf {
        self.config
            .world_backup_dir
            .clone()
            .unwrap_or_else(|| self.paths.backups_dir())
    }

    /// Copy shared files from a foreign `.minecraft` (versions, assets,
    /// libraries) so they are not re-downloaded (CONCEPT P1). Only missing
    /// files are copied, never overwriting. Heavy work runs blocking-style
    /// off the UI thread; the result lands in `mcreuse_rx`.
    fn import_legacy_minecraft(&mut self, src: PathBuf) {
        self.mcreuse_path = None;
        self.config.mc_reuse_done = true;
        let _ = self.config.save_to_path(&self.paths.config_file());
        let (tx, rx) = mpsc::channel(1);
        self.mcreuse_rx = Some(rx);
        let paths = self.paths.clone();
        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                let mut files = 0u64;
                let mut bytes = 0u64;
                for (sub, dest_root) in [
                    ("versions", paths.versions_dir()),
                    ("assets", paths.assets_dir()),
                    ("libraries", paths.libraries_dir()),
                ] {
                    let from = src.join(sub);
                    if !from.is_dir() {
                        continue;
                    }
                    let mut stack = vec![from.clone()];
                    while let Some(dir) = stack.pop() {
                        let Ok(entries) = std::fs::read_dir(&dir) else {
                            continue;
                        };
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_symlink() {
                                continue;
                            }
                            if path.is_dir() {
                                stack.push(path);
                                continue;
                            }
                            let Ok(rel) = path.strip_prefix(&from) else {
                                continue;
                            };
                            let dest = dest_root.join(rel);
                            if dest.is_file() {
                                continue;
                            }
                            if let Some(parent) = dest.parent() {
                                if std::fs::create_dir_all(parent).is_err() {
                                    continue;
                                }
                            }
                            if std::fs::copy(&path, &dest).is_ok() {
                                files += 1;
                                bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
                            }
                        }
                    }
                }
                (files, bytes)
            })
            .await
            .unwrap_or((0, 0));
            let _ = tx.send(result).await;
        });
    }

    fn handle_direct_connect(&mut self, server: String, port: u16) {
        self.pending_direct_connect = Some((server, port));
        self.handle_launch();
    }

    fn request_ms_device_code(&mut self) {
        let (tx, rx) = mpsc::channel(1);
        self.ms_code_rx = Some(rx);

        tokio::spawn(async move {
            let flow = MicrosoftAuthFlow::default();
            match flow.request_device_code().await {
                Ok(resp) => {
                    let _ = tx.send(Ok(resp)).await;
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string())).await;
                }
            }
        });
    }

    fn start_ms_poll(&mut self, device_code: String) {
        let (tx, rx) = mpsc::channel(1);
        self.ms_poll_rx = Some(rx);
        self.login_modal.ms_polling = true;

        tokio::spawn(async move {
            let flow = MicrosoftAuthFlow::default();
            let mut attempts = 60;
            while attempts > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                match flow.poll_device_token(&device_code).await {
                    Ok(Some(acc)) => {
                        let _ = tx.send(Ok(acc)).await;
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string())).await;
                        break;
                    }
                }
                attempts -= 1;
            }
        });
    }

    fn handle_launch(&mut self) {
        // Per-launch identity (CONCEPT "Аккаунты"): a remembered choice or
        // the instance default wins outright; otherwise, when the user asked
        // to choose every time and several identities exist, open the picker
        // and continue after they pick.
        let remembered = self.config.remembered_identity.clone();
        let usable = self.account_mgr.accounts().to_vec();
        if self.config.ask_identity_each_launch
            && remembered.is_none()
            && usable.len() > 1
            && !self.pending_identity_pick
        {
            self.pending_identity_pick = true;
            self.identity_picker.open = true;
            self.picker_nickname.clear();
            return;
        }
        self.pending_identity_pick = false;
        // Instance default account wins over the active one; unknown ids
        // fall back to active, then to the remembered choice.
        let session = self
            .selected_instance
            .and_then(|id| {
                self.instances
                    .iter()
                    .find(|i| i.id == id)
                    .and_then(|inst| inst.default_account)
                    .and_then(|acc_id| self.account_mgr.accounts().iter().find(|a| a.id == acc_id))
                    .map(amc_auth::AuthSession::from_account)
            })
            .or_else(|| {
                remembered
                    .as_ref()
                    .and_then(|id| {
                        self.account_mgr
                            .accounts()
                            .iter()
                            .find(|a| a.id.to_string() == *id)
                    })
                    .map(amc_auth::AuthSession::from_account)
            })
            .or_else(|| self.account_mgr.get_session());
        let Some(session) = session else {
            self.login_modal.is_open = true;
            return;
        };

        self.launched_instance_id = self.selected_instance;

        let paths = self.paths.clone();
        // P9: scheduled world backups run inside the launch task below.
        let backup_days = self.config.world_backup_days;
        let backups_root = self.world_backups_root();
        let backup_inst_name = self
            .selected_instance
            .and_then(|id| self.instances.iter().find(|i| i.id == id))
            .map(|inst| inst.name.clone());
        let (game_dir, ver_id, ram_override, loader) = if let Some(inst_id) = self.selected_instance
        {
            if let Some(inst) = self.instances.iter().find(|i| i.id == inst_id) {
                let _ = inst.ensure_directories(&paths.instances_dir());
                (
                    inst.get_game_dir(&paths.instances_dir()),
                    inst.game_version.clone(),
                    inst.ram_mb,
                    inst.loader,
                )
            } else {
                let ver = self
                    .selected_version
                    .clone()
                    .unwrap_or_else(|| "1.20.1".to_string());
                (
                    paths.instances_dir().join(&ver),
                    ver,
                    None,
                    LoaderType::Vanilla,
                )
            }
        } else {
            let ver = self
                .selected_version
                .clone()
                .unwrap_or_else(|| "1.20.1".to_string());
            (
                paths.instances_dir().join(&ver),
                ver,
                None,
                LoaderType::Vanilla,
            )
        };

        tracing::info!(
            "Preparing Minecraft launch: Version={}, Loader={}, User={}...",
            ver_id,
            loader.as_str(),
            session.username
        );

        // P8: seed launcher favorites into the instance (never overwrites
        // the player's own servers.dat).
        match amc_minecraft::inject_favorites(&game_dir, &self.favorites) {
            Ok(true) => tracing::info!("Injected favorite servers into {}", game_dir.display()),
            Ok(false) => {}
            Err(e) => tracing::warn!("Favorite servers inject failed: {e}"),
        }

        let lang = self.config.ui.language();
        self.is_launching = true;
        self.launch_status_text = lang.status_preparing_version(&ver_id);

        let engine = self.download_engine.clone();
        engine.set_speed_limit_kbps(self.config.download_speed_limit_kbps);
        let mut options = self.config.default_launch_options.clone();
        if let Some(ram) = ram_override {
            options.memory_max_mb = ram;
        }
        if let Some((srv, port)) = self.pending_direct_connect.take() {
            options.quick_play_server = Some(srv);
            options.quick_play_port = Some(port);
        }

        let download_cancel = DownloadCancel::new();
        let (game_tx, game_rx) = mpsc::channel(256);
        let (status_tx, status_rx) = mpsc::channel(32);
        let (prog_tx, prog_rx) = mpsc::channel(8);

        self.game_events_rx = Some(game_rx);
        self.launch_status_rx = Some(status_rx);
        self.progress_init_rx = Some(prog_rx);
        self.download_cancel = Some(download_cancel.clone());

        tokio::spawn(async move {
            let client = reqwest::Client::new();

            // 0. Scheduled world backups (CONCEPT, P9): due worlds are
            // copied before anything is downloaded or started.
            if backup_days > 0 {
                if let Some(inst_name) = backup_inst_name {
                    let _ = status_tx
                        .send(lang.status_backup_worlds().to_string())
                        .await;
                    let now = std::time::SystemTime::now();
                    for world in amc_core::list_saves(&game_dir) {
                        if amc_core::needs_auto_backup(
                            &backups_root,
                            &inst_name,
                            &world,
                            backup_days,
                            now,
                        ) {
                            match amc_core::backup_world(
                                &backups_root,
                                &inst_name,
                                &world,
                                &game_dir.join("saves"),
                            ) {
                                Ok(path) => {
                                    tracing::info!("Auto-backup {} -> {}", world, path.display())
                                }
                                Err(e) => {
                                    tracing::warn!("Auto-backup failed: {e}")
                                }
                            }
                        }
                    }
                }
            }

            // 1. Resolve Version Details
            let _ = status_tx
                .send(lang.status_fetching_manifest().to_string())
                .await;
            let details = match loader {
                LoaderType::Fabric => {
                    let _ = status_tx.send("Fabric Loader...".to_string()).await;
                    match FabricLoader::get_latest_loader_version(&client, &ver_id).await {
                        Ok(loader_ver) => {
                            let _ = status_tx
                                .send(format!("Загрузка профиля Fabric {loader_ver}..."))
                                .await;
                            match FabricLoader::fetch_profile_json(&client, &ver_id, &loader_ver)
                                .await
                            {
                                Ok(mut fab_details) => {
                                    if let Ok(vanilla) = VersionDetails::fetch_or_load(
                                        &client,
                                        &ver_id,
                                        None,
                                        &paths.versions_dir(),
                                    )
                                    .await
                                    {
                                        fab_details.merge_parent(vanilla);
                                    }
                                    fab_details
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Fabric profile fetch error: {e}, falling back to Vanilla"
                                    );
                                    match VersionDetails::fetch_or_load(
                                        &client,
                                        &ver_id,
                                        None,
                                        &paths.versions_dir(),
                                    )
                                    .await
                                    {
                                        Ok(d) => d,
                                        Err(err) => {
                                            let _ = game_tx
                                                .send(GameEvent::Crashed {
                                                    message: format!(
                                                        "Ошибка загрузки версии {ver_id}: {err}"
                                                    ),
                                                })
                                                .await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Fabric loader lookup error: {e}, falling back to Vanilla"
                            );
                            match VersionDetails::fetch_or_load(
                                &client,
                                &ver_id,
                                None,
                                &paths.versions_dir(),
                            )
                            .await
                            {
                                Ok(d) => d,
                                Err(err) => {
                                    let _ = game_tx
                                        .send(GameEvent::Crashed {
                                            message: format!(
                                                "Ошибка загрузки версии {ver_id}: {err}"
                                            ),
                                        })
                                        .await;
                                    return;
                                }
                            }
                        }
                    }
                }
                LoaderType::Quilt => {
                    let _ = status_tx.send("Поиск Quilt Loader...".to_string()).await;
                    match QuiltLoader::get_latest_loader_version(&client, &ver_id).await {
                        Ok(loader_ver) => {
                            let _ = status_tx
                                .send(format!("Загрузка профиля Quilt {loader_ver}..."))
                                .await;
                            match QuiltLoader::fetch_profile_json(&client, &ver_id, &loader_ver)
                                .await
                            {
                                Ok(mut quilt_details) => {
                                    if let Ok(vanilla) = VersionDetails::fetch_or_load(
                                        &client,
                                        &ver_id,
                                        None,
                                        &paths.versions_dir(),
                                    )
                                    .await
                                    {
                                        quilt_details.merge_parent(vanilla);
                                    }
                                    quilt_details
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Quilt profile fetch error: {e}, falling back to Vanilla"
                                    );
                                    match VersionDetails::fetch_or_load(
                                        &client,
                                        &ver_id,
                                        None,
                                        &paths.versions_dir(),
                                    )
                                    .await
                                    {
                                        Ok(d) => d,
                                        Err(err) => {
                                            let _ = game_tx
                                                .send(GameEvent::Crashed {
                                                    message: format!(
                                                        "Ошибка загрузки версии {ver_id}: {err}"
                                                    ),
                                                })
                                                .await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Quilt loader lookup error: {e}, falling back to Vanilla"
                            );
                            match VersionDetails::fetch_or_load(
                                &client,
                                &ver_id,
                                None,
                                &paths.versions_dir(),
                            )
                            .await
                            {
                                Ok(d) => d,
                                Err(err) => {
                                    let _ = game_tx
                                        .send(GameEvent::Crashed {
                                            message: format!(
                                                "Ошибка загрузки версии {ver_id}: {err}"
                                            ),
                                        })
                                        .await;
                                    return;
                                }
                            }
                        }
                    }
                }
                LoaderType::Forge => {
                    let _ = status_tx
                        .send(lang.status_resolving_loader("Forge", &ver_id))
                        .await;
                    let vanilla =
                        match fetch_vanilla_details(&client, &ver_id, &paths, &game_tx).await {
                            Some(v) => v,
                            None => return,
                        };
                    let java_bin =
                        match resolve_java_for_install(&vanilla, &options, &engine, &paths).await {
                            Ok(bin) => bin,
                            Err(err) => {
                                let _ = game_tx
                                    .send(GameEvent::Crashed {
                                        message: lang.err_loader_failed("Forge", &err),
                                    })
                                    .await;
                                return;
                            }
                        };
                    match ForgeLoader::resolve(
                        &client,
                        &engine,
                        &ver_id,
                        &java_bin,
                        &paths,
                        status_tx.clone(),
                    )
                    .await
                    {
                        Ok(mut forge_details) => {
                            forge_details.merge_parent(vanilla);
                            forge_details
                        }
                        Err(e) => {
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: lang.err_loader_failed("Forge", &e.to_string()),
                                })
                                .await;
                            return;
                        }
                    }
                }
                LoaderType::NeoForge => {
                    let _ = status_tx
                        .send(lang.status_resolving_loader("NeoForge", &ver_id))
                        .await;
                    let vanilla =
                        match fetch_vanilla_details(&client, &ver_id, &paths, &game_tx).await {
                            Some(v) => v,
                            None => return,
                        };
                    let java_bin =
                        match resolve_java_for_install(&vanilla, &options, &engine, &paths).await {
                            Ok(bin) => bin,
                            Err(err) => {
                                let _ = game_tx
                                    .send(GameEvent::Crashed {
                                        message: lang.err_loader_failed("NeoForge", &err),
                                    })
                                    .await;
                                return;
                            }
                        };
                    match NeoForgeLoader::resolve(
                        &client,
                        &engine,
                        &ver_id,
                        &java_bin,
                        &paths,
                        status_tx.clone(),
                    )
                    .await
                    {
                        Ok(mut neoforge_details) => {
                            neoforge_details.merge_parent(vanilla);
                            neoforge_details
                        }
                        Err(e) => {
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: lang.err_loader_failed("NeoForge", &e.to_string()),
                                })
                                .await;
                            return;
                        }
                    }
                }
                LoaderType::OptiFine => {
                    let _ = status_tx
                        .send(lang.status_resolving_loader("OptiFine", &ver_id))
                        .await;
                    let vanilla =
                        match fetch_vanilla_details(&client, &ver_id, &paths, &game_tx).await {
                            Some(v) => v,
                            None => return,
                        };
                    match OptiFineLoader::resolve(&client, &ver_id).await {
                        Ok(mut of_details) => {
                            of_details.merge_parent(vanilla);
                            OptiFineLoader::normalize_legacy_tweak(&mut of_details);
                            of_details
                        }
                        Err(e) => {
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: lang.err_loader_failed("OptiFine", &e.to_string()),
                                })
                                .await;
                            return;
                        }
                    }
                }
                _ => {
                    match VersionDetails::fetch_or_load(
                        &client,
                        &ver_id,
                        None,
                        &paths.versions_dir(),
                    )
                    .await
                    {
                        Ok(d) => d,
                        Err(err) => {
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: format!("Ошибка загрузки версии {ver_id}: {err}"),
                                })
                                .await;
                            return;
                        }
                    }
                }
            };

            // 2. Resolve Java Runtime
            let req_java = details.required_java_major();
            let _ = status_tx
                .send(lang.status_checking_java().to_string())
                .await;
            let java_bin = if let Some(custom_java) = &options.java_path {
                if custom_java.is_file() {
                    custom_java.clone()
                } else {
                    AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), req_java, &engine, None)
                        .await
                        .unwrap_or_else(|_| custom_java.clone())
                }
            } else if let Some(sys) = amc_downloader::find_system_java(req_java).await {
                // P5 auto-offer: a matching system Java wins over downloading.
                tracing::info!("Using system Java {} for Java {req_java}", sys.display());
                sys
            } else {
                match AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), req_java, &engine, None)
                    .await
                {
                    Ok(bin) => bin,
                    Err(e) => {
                        tracing::error!("Java installation failed: {e}");
                        let _ = game_tx
                            .send(GameEvent::Crashed {
                                message: format!("Ошибка установки Java {req_java}: {e}"),
                            })
                            .await;
                        return;
                    }
                }
            };

            // 3. Collect download items
            let _ = status_tx
                .send(lang.status_checking_assets().to_string())
                .await;
            let mut download_items: Vec<amc_downloader::DownloadItem> = Vec::new();
            let client_jar = paths
                .versions_dir()
                .join(&ver_id)
                .join(format!("{ver_id}.jar"));

            if let Some(downloads) = &details.downloads {
                if let Some(client_file) = &downloads.client {
                    if !client_jar.is_file() {
                        let mut item =
                            amc_downloader::DownloadItem::new(&client_file.url, &client_jar);
                        if let Some(sha1) = &client_file.sha1 {
                            item = item.with_sha1(sha1);
                        }
                        if let Some(size) = client_file.size {
                            item = item.with_size(size);
                        }
                        download_items.push(item);
                    }
                }
            }

            let features = std::collections::HashMap::new();
            let mut classpath_libs: Vec<std::path::PathBuf> = Vec::new();
            let mut natives_jars: Vec<std::path::PathBuf> = Vec::new();

            for lib in &details.libraries {
                if !lib.is_allowed(&features) {
                    continue;
                }
                if let Some(item) = lib.to_download_item(&paths.libraries_dir()) {
                    let dest = item.destination.clone();
                    classpath_libs.push(dest.clone());
                    if lib.natives.is_some() || dest.to_string_lossy().contains("natives") {
                        natives_jars.push(dest.clone());
                    }
                    if !dest.is_file() {
                        download_items.push(item);
                    }
                }
            }

            if let Some(asset_ref) = &details.asset_index {
                let _ = status_tx
                    .send(lang.status_checking_assets().to_string())
                    .await;
                if let Ok(idx) = amc_minecraft::AssetIndex::fetch_or_load(
                    &client,
                    asset_ref,
                    &paths.assets_dir(),
                )
                .await
                {
                    let all_assets = idx.to_download_items(&paths.assets_dir());
                    for a in all_assets {
                        if !a.destination.is_file() {
                            download_items.push(a);
                        }
                    }
                }
            }

            // 4. Download missing components
            if !download_items.is_empty() {
                let count = download_items.len();
                let _ = status_tx.send(lang.status_downloading_files(count)).await;
                // CONCEPT "Место на диске": warn before downloading more than fits.
                let needed: u64 = download_items.iter().filter_map(|item| item.size).sum();
                if needed > 0 {
                    if let Some(free_mb) = amc_core::free_disk_mb(&game_dir) {
                        let free_bytes = free_mb.saturating_mul(1024 * 1024);
                        if needed > free_bytes {
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: lang.err_low_disk(
                                        &lang.disk_size(needed),
                                        &lang.disk_size(free_bytes),
                                    ),
                                })
                                .await;
                            return;
                        }
                    }
                }
                let (tracker, progress_rx) = engine.create_tracker(&download_items);
                let _ = prog_tx.send(Some(progress_rx)).await;

                if let Err(e) = engine
                    .download_all_with_progress(download_items, Some(tracker), download_cancel)
                    .await
                {
                    let _ = prog_tx.send(None).await;
                    match e {
                        LauncherError::Cancelled => {
                            let _ = game_tx.send(GameEvent::Cancelled).await;
                        }
                        _ => {
                            tracing::error!("Component download failed: {e}");
                            let _ = game_tx
                                .send(GameEvent::Crashed {
                                    message: format!("Ошибка скачивания компонентов: {e}"),
                                })
                                .await;
                        }
                    }
                    return;
                }
                let _ = prog_tx.send(None).await;
            }

            // 5. Unpack natives
            let natives_dir = paths.libraries_dir().join("natives").join(&ver_id);
            let _ = tokio::fs::create_dir_all(&natives_dir).await;
            let _ = status_tx
                .send(lang.status_extracting_natives().to_string())
                .await;
            for nat_jar in &natives_jars {
                let _ = VersionDetails::extract_natives(nat_jar, &natives_dir);
            }

            // 6. Launch Minecraft
            let _ = status_tx.send(lang.status_launching().to_string()).await;
            let assets_dir = paths.assets_dir();

            match MinecraftLauncher::launch(
                &java_bin,
                &game_dir,
                &assets_dir,
                &natives_dir,
                &classpath_libs,
                &client_jar,
                &details,
                &session,
                &options,
            )
            .await
            {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        let _ = game_tx.send(event).await;
                    }
                }
                Err(e) => {
                    let _ = game_tx
                        .send(GameEvent::Crashed {
                            message: format!("Ошибка запуска: {e}"),
                        })
                        .await;
                }
            }
        });
    }
}

/// Recursive directory copy (world import, backups). Skips nothing,
/// fails on the first IO error.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Fetch the vanilla parent for a loader branch, reporting a crash event.
/// New loader arms share it instead of duplicating the boilerplate.
async fn fetch_vanilla_details(
    client: &reqwest::Client,
    ver_id: &str,
    paths: &LauncherPaths,
    game_tx: &mpsc::Sender<GameEvent>,
) -> Option<VersionDetails> {
    match VersionDetails::fetch_or_load(client, ver_id, None, &paths.versions_dir()).await {
        Ok(d) => Some(d),
        Err(err) => {
            let _ = game_tx
                .send(GameEvent::Crashed {
                    message: format!("Ошибка загрузки версии {ver_id}: {err}"),
                })
                .await;
            None
        }
    }
}

/// Resolve a runnable `java` binary for installer-based loaders, mirroring
/// the main launch flow: explicit user path first, Adoptium otherwise.
async fn resolve_java_for_install(
    vanilla: &VersionDetails,
    options: &LaunchOptions,
    engine: &DownloadEngine,
    paths: &LauncherPaths,
) -> Result<std::path::PathBuf, String> {
    if let Some(custom_java) = &options.java_path {
        if custom_java.is_file() {
            return Ok(custom_java.clone());
        }
        let major = vanilla.required_java_major();
        match AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), major, engine, None).await {
            Ok(bin) => Ok(bin),
            Err(_) => Ok(custom_java.clone()),
        }
    } else {
        let major = vanilla.required_java_major();
        if let Some(sys) = amc_downloader::find_system_java(major).await {
            return Ok(sys);
        }
        AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), major, engine, None)
            .await
            .map_err(|e| format!("Java {major}: {e}"))
    }
}

impl App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // In-app hotkeys (CONCEPT, P10): Ctrl+1..6 switch tabs, F5 plays.
        // Skipped while typing or when a modal grabs input.
        if !ctx.wants_keyboard_input() && !self.login_modal.is_open {
            let tabs = [
                NavTab::Home,
                NavTab::Modpacks,
                NavTab::Mods,
                NavTab::Skins,
                NavTab::Profile,
                NavTab::Settings,
            ];
            for (i, tab) in tabs.iter().enumerate() {
                let digit = match i {
                    0 => egui::Key::Num1,
                    1 => egui::Key::Num2,
                    2 => egui::Key::Num3,
                    3 => egui::Key::Num4,
                    4 => egui::Key::Num5,
                    _ => egui::Key::Num6,
                };
                if ctx.input(|inp| inp.modifiers.ctrl && inp.key_pressed(digit)) {
                    self.current_tab = *tab;
                }
            }
            if ctx.input(|inp| inp.key_pressed(egui::Key::F5)) {
                self.handle_launch();
            }
        }

        // Drag-and-drop install (CONCEPT P4): .mrpack pack, .jar mod,
        // or a world folder (contains level.dat) into the window.
        if !self.pending_identity_pick && self.wizard.is_none() {
            let dropped: Vec<std::path::PathBuf> = ctx.input(|inp| {
                inp.raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect()
            });
            if !dropped.is_empty() {
                self.handle_dropped_files(dropped);
            }
        }

        // Toast notifications render above all panels.
        show_toasts(ctx, &mut self.toasts);

        // Receive version manifest updates
        if let Some(rx) = &mut self.versions_rx {
            if let Ok(versions) = rx.try_recv() {
                self.versions = versions;
                self.versions_rx = None;
            }
        }

        // Receive mod search results
        if let Some(rx) = &mut self.mod_search_rx {
            if let Ok(results) = rx.try_recv() {
                self.mods_page.search_results = results;
                self.mods_page.is_searching = false;
                self.mod_search_rx = None;
            }
        }

        // Receive Mod installation result
        if let Some(rx) = &mut self.mod_install_rx {
            if let Ok(res) = rx.try_recv() {
                match res {
                    Ok(report) => {
                        let lang = self.config.ui.language();
                        self.mods_page.installing_ids.remove(&report.mod_id);
                        self.mods_page
                            .installed_titles
                            .insert(report.title.to_lowercase());
                        // Provenance for the local-mod badge + old-file
                        // cleanup on one-click updates.
                        if let Some(inst_id) = report.instance_id {
                            if let Some(inst) = self.instances.iter_mut().find(|i| i.id == inst_id)
                            {
                                for name in &report.filenames {
                                    if !inst.installed_by_launcher.contains(name) {
                                        inst.installed_by_launcher.push(name.clone());
                                    }
                                }
                                if let Some(old) = &report.replaced_old {
                                    let old_path = report.target_dir.join(old);
                                    let _ = std::fs::remove_file(&old_path);
                                    inst.installed_by_launcher.retain(|n| n != old);
                                }
                                let _ = Instance::save_all(
                                    &self.paths.instances_file(),
                                    &self.instances,
                                );
                            }
                        }
                        self.mods_page.local_mods =
                            LocalModManager::scan_mods(&report.target_dir).unwrap_or_default();
                        self.instances_page.refresh_scans();
                        let compat = match &report.mc_version {
                            Some(mc) => format!(" • {}", lang.compat_ok(mc, &report.loader_name)),
                            None => format!(" • {}", lang.compat_unknown()),
                        };
                        self.mods_page.status_message = Some((
                            format!("Мод \"{}\" установлен!{compat}", report.title),
                            true,
                        ));
                        push_toast(
                            &mut self.toasts,
                            &self.config.notifications,
                            ToastKind::Downloads,
                            format!("Мод \"{}\" установлен!", report.title),
                        );
                    }
                    Err(err) => {
                        self.mods_page.status_message = Some((err, false));
                    }
                }
                self.mod_install_rx = None;
            }
        }

        // Trigger + receive mod update checks.
        if self.mods_page.update_check_requested {
            self.mods_page.update_check_requested = false;
            self.check_mod_updates();
        }
        if let Some(rx) = &mut self.mod_update_rx {
            if let Ok(offers) = rx.try_recv() {
                self.mods_page.update_checking = false;
                let n = offers.len();
                // Row lookup uses the manifest display name, which is what
                // the heuristic matched on.
                self.mods_page.update_offers =
                    offers.into_iter().map(|o| (o.title.clone(), o)).collect();
                if n > 0 {
                    push_toast(
                        &mut self.toasts,
                        &self.config.notifications,
                        ToastKind::Updates,
                        format!("Моды с обновлениями: {n}"),
                    );
                }
                self.mod_update_rx = None;
            }
        }

        // Launcher update check (manual button; auto-check runs once at
        // startup from main.rs — CONCEPT checks only on launch).
        if self.settings_page.update_check_requested {
            self.settings_page.update_check_requested = false;
            self.check_launcher_update();
        }
        if let Some(rx) = &mut self.update_rx {
            if let Ok(found) = rx.try_recv() {
                let lang = self.config.ui.language();
                let note = match &found {
                    Some(ver) => lang.update_available(ver),
                    None => lang.update_latest().to_string(),
                };
                self.settings_page.update_note = Some(note.clone());
                // Toast only for real updates; "latest" stays a settings note.
                if found.is_some() {
                    push_toast(
                        &mut self.toasts,
                        &self.config.notifications,
                        ToastKind::Updates,
                        note,
                    );
                }
                self.update_rx = None;
            }
        }

        // One-click diagnostics bundle (P12 crash export task).
        if self.settings_page.crash_export_requested {
            self.settings_page.crash_export_requested = false;
            self.export_diagnostics();
        }

        // Local dedicated server (P13): start/stop requests, prep-task
        // results, and reaping a server that exited on its own.
        if self.settings_page.server_start_requested {
            self.settings_page.server_start_requested = false;
            let eula_ok = self.settings_page.server_eula_draft;
            self.start_local_server(eula_ok);
        }
        if self.settings_page.server_stop_requested {
            self.settings_page.server_stop_requested = false;
            // Dropping the child kills the process (kill_on_drop).
            self.server_child = None;
            self.server_status = None;
        }
        if let Some(rx) = &mut self.server_rx {
            if let Ok(outcome) = rx.try_recv() {
                let lang = self.config.ui.language();
                match outcome {
                    Ok(child) => {
                        self.server_child = Some(child);
                        self.server_status = Some(lang.server_running(25565));
                    }
                    Err(err) => {
                        tracing::warn!("Local server failed: {err}");
                        self.server_status = Some(err);
                    }
                }
                self.server_rx = None;
            }
        }
        if let Some(child) = &mut self.server_child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.server_child = None;
                    self.server_status = None;
                }
                Ok(None) => {}
                Err(e) => tracing::warn!("Local server wait failed: {e}"),
            }
        }

        // Server favorites ping pass (P8): status + players without joining.
        if self.home_page.fav_ping_request {
            self.home_page.fav_ping_request = false;
            let targets: Vec<(String, String, u16)> = self
                .favorites
                .iter()
                .filter_map(|f| {
                    amc_minecraft::parse_server_address(&f.address)
                        .map(|(host, port)| (f.address.clone(), host, port))
                })
                .collect();
            let (tx, rx) = mpsc::channel(1);
            self.fav_ping_rx = Some(rx);
            tokio::spawn(async move {
                let mut out = Vec::new();
                for (addr, host, port) in targets {
                    match amc_minecraft::ServerPinger::ping(&host, port).await {
                        Ok(status) => out.push((addr, status)),
                        Err(e) => tracing::warn!("Favorite ping failed: {e}"),
                    }
                }
                let _ = tx.send(out).await;
            });
        }
        if let Some(rx) = &mut self.fav_ping_rx {
            if let Ok(list) = rx.try_recv() {
                self.home_page.fav_status = list.into_iter().collect();
                self.home_page.fav_pinging = false;
                self.fav_ping_rx = None;
            }
        }

        // Receive launch step text
        if let Some(rx) = &mut self.launch_status_rx {
            while let Ok(msg) = rx.try_recv() {
                self.launch_status_text = msg;
            }
        }

        // Receive progress tracker handle
        if let Some(rx) = &mut self.progress_init_rx {
            if let Ok(maybe_rx) = rx.try_recv() {
                self.download_progress_rx = maybe_rx;
            }
        }

        // Receive MS Device Code
        if let Some(rx) = &mut self.ms_code_rx {
            if let Ok(res) = rx.try_recv() {
                match res {
                    Ok(resp) => {
                        let code = resp.device_code.clone();
                        self.login_modal.ms_device_code = Some(resp);
                        self.ms_code_rx = None;
                        self.start_ms_poll(code);
                    }
                    Err(err) => {
                        self.login_modal.error_msg = Some(err);
                        self.ms_code_rx = None;
                    }
                }
            }
        }

        // Receive MS Poll result
        if let Some(rx) = &mut self.ms_poll_rx {
            if let Ok(res) = rx.try_recv() {
                match res {
                    Ok(acc) => {
                        if self.store_account(acc) {
                            self.login_modal.is_open = false;
                            self.login_modal.ms_polling = false;
                            self.login_modal.ms_device_code = None;
                            self.ms_poll_rx = None;
                        } else {
                            self.login_modal.ms_polling = false;
                        }
                    }
                    Err(err) => {
                        self.login_modal.error_msg = Some(err);
                        self.login_modal.ms_polling = false;
                        self.ms_poll_rx = None;
                    }
                }
            }
        }

        // Receive server ping result
        if let Some(rx) = &mut self.server_ping_rx {
            if let Ok(status) = rx.try_recv() {
                self.home_page.featured_server = Some(status);
                self.home_page.is_pinging = false;
                self.server_ping_rx = None;
            }
        }

        // Download progress overlay (pause/resume/cancel).
        if self.download_progress_rx.is_some() {
            let lang = self.config.ui.language();
            let paused = self.download_paused;
            let mut overlay_action = None;
            if let Some(rx) = &self.download_progress_rx {
                let progress = rx.borrow().clone();
                let title = self.launch_status_text.clone();
                DownloadOverlay::show(ctx, lang, &progress, &title, paused, |a| {
                    overlay_action = Some(a)
                });
            }
            match overlay_action {
                Some(OverlayAction::Pause) => {
                    if let Some(cancel) = &self.download_cancel {
                        cancel.pause();
                    }
                }
                Some(OverlayAction::Resume) => {
                    self.download_paused = false;
                    self.handle_launch();
                }
                Some(OverlayAction::Cancel) => {
                    if let Some(cancel) = &self.download_cancel {
                        cancel.cancel();
                    }
                }
                None => {}
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // Check game events
        if let Some(rx) = &mut self.game_events_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    GameEvent::Started { pid } => {
                        let msg = format!("Minecraft успешно запущен с PID {pid}");
                        tracing::info!("{msg}");
                        self.console_modal.push_line(format!("[ALEPH] {msg}"));
                        self.session_start_time = Some(std::time::Instant::now());
                        self.is_launching = false;
                        match self.config.ui.after_launch {
                            amc_core::config::AfterLaunch::Close => {
                                ctx.send_viewport_cmd(ViewportCommand::Close);
                            }
                            amc_core::config::AfterLaunch::Minimize => {
                                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                            }
                            amc_core::config::AfterLaunch::Keep => {}
                        }
                    }
                    GameEvent::LogLine(line) => {
                        tracing::debug!("[MC] {line}");
                        self.console_modal.push_line(line);
                    }
                    GameEvent::Exited { code } => {
                        let lang = self.config.ui.language();
                        let msg = lang.status_process_exited(code);
                        tracing::info!("{msg}");
                        self.console_modal.push_line(format!("[ALEPH] {msg}"));
                        self.is_launching = false;
                        if let Some(start) = self.session_start_time.take() {
                            let elapsed_mins = (start.elapsed().as_secs() / 60).max(1);
                            if let Some(inst_id) = self.launched_instance_id {
                                Self::add_playtime(
                                    &self.paths,
                                    &mut self.instances,
                                    inst_id,
                                    elapsed_mins,
                                );
                                // Clean exit (not a crash): remember the session
                                // for the summary modal (CONCEPT "После запуска игры").
                                if code == Some(0) {
                                    if let Some(inst) =
                                        self.instances.iter().find(|i| i.id == inst_id)
                                    {
                                        self.session_summary =
                                            Some((inst.name.clone(), elapsed_mins));
                                    }
                                }
                            }
                        }
                    }
                    GameEvent::DownloadPaused => {
                        let lang = self.config.ui.language();
                        let msg = lang.status_download_paused();
                        tracing::info!("{msg}");
                        self.console_modal.push_line(format!("[ALEPH] {msg}"));
                        self.download_paused = true;
                    }
                    GameEvent::Cancelled => {
                        let lang = self.config.ui.language();
                        let msg = lang.status_download_cancelled();
                        tracing::info!("{msg}");
                        self.console_modal.push_line(format!("[ALEPH] {msg}"));
                        self.is_launching = false;
                    }
                    GameEvent::Crashed { message } => {
                        let lang = self.config.ui.language();
                        let msg = lang.status_process_crashed(&message);
                        tracing::error!("{msg}");
                        self.console_modal.set_crash_message(msg, lang);
                        self.is_launching = false;
                        if self.config.crash_reports {
                            Self::archive_crash(&self.paths, &message);
                        }
                        if let Some(start) = self.session_start_time.take() {
                            let elapsed_mins = (start.elapsed().as_secs() / 60).max(1);
                            if let Some(inst_id) = self.launched_instance_id {
                                Self::add_playtime(
                                    &self.paths,
                                    &mut self.instances,
                                    inst_id,
                                    elapsed_mins,
                                );
                            }
                        }
                    }
                }
            }
        }

        let lang = self.config.ui.language();

        // Update avatar texture if needed
        if self.skins_page.avatar_dirty || self.avatar_texture.is_none() {
            let avatar_img = crate::pages::skins::extract_head_avatar(
                self.skins_page.skin_image.as_ref(),
                self.skins_page.is_slim_model,
            );
            self.avatar_texture =
                Some(ctx.load_texture("player_avatar", avatar_img, egui::TextureOptions::NEAREST));
            self.skins_page.avatar_dirty = false;
        }

        // First-run wizard (ROADMAP P1) takes over the whole window until done.
        if self.wizard.is_some() {
            let action = self
                .wizard
                .as_mut()
                .map(|flow| flow.show(ctx))
                .unwrap_or(WizardAction::None);
            match action {
                WizardAction::None => {}
                WizardAction::Finished => self.complete_wizard(),
                WizardAction::CreateOffline(account) => {
                    if self.store_account(account) {
                        self.complete_wizard();
                    }
                }
                WizardAction::OpenMicrosoftLogin => {
                    self.login_modal.is_open = true;
                    self.login_modal.mode = LoginMode::Microsoft;
                    self.complete_wizard();
                }
            }
            if self.wizard.is_some() {
                return;
            }
        }

        // One-time weak-hardware notice right after the wizard.
        if let Some(report) = self.hw_notice.take() {
            match crate::modals::wizard::show_hardware_notice(ctx, lang, &report) {
                HwNoticeAction::None => self.hw_notice = Some(report),
                HwNoticeAction::Dismissed => {}
                HwNoticeAction::InstallSodium => self.install_sodium(),
            }
        }

        // .minecraft reuse offer (CONCEPT "Первый запуск", P1).
        if let Some(legacy) = self.mcreuse_path.clone() {
            let mut resolve: Option<bool> = None;
            egui::Window::new(lang.mcreuse_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(
                        lang.mcreuse_body(&legacy.to_string_lossy()),
                    ));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(lang.mcreuse_use()).clicked() {
                            resolve = Some(true);
                        }
                        if ui.button(lang.mcreuse_skip()).clicked() {
                            resolve = Some(false);
                        }
                    });
                });
            match resolve {
                Some(true) => self.import_legacy_minecraft(legacy),
                Some(false) => {
                    self.mcreuse_path = None;
                    self.config.mc_reuse_done = true;
                    let _ = self.config.save_to_path(&self.paths.config_file());
                }
                None => {}
            }
        }
        if let Some(rx) = &mut self.mcreuse_rx {
            if let Ok((files, bytes)) = rx.try_recv() {
                push_toast(
                    &mut self.toasts,
                    &self.config.notifications,
                    ToastKind::Downloads,
                    lang.mcreuse_copied(files, bytes / 1_048_576),
                );
                self.mcreuse_rx = None;
            }
        }

        // Tutorial banners for newcomers (CONCEPT "Первый запуск", P1):
        // one dismissible hint per main tab, shown until dismissed.
        if self.wizard.is_none() && self.mcreuse_path.is_none() && !self.config.tour_seen {
            let hint = match self.current_tab {
                NavTab::Home => Some((lang.tour_home_title(), lang.tour_home_body())),
                NavTab::Modpacks => Some((lang.tour_instances_title(), lang.tour_instances_body())),
                _ => None,
            };
            if let Some((title, body)) = hint {
                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -100.0))
                    .min_width(420.0)
                    .show(ctx, |ui| {
                        ui.add_space(4.0);
                        ui.label(body);
                        ui.add_space(8.0);
                        if ui.button(lang.tour_dismiss()).clicked() {
                            self.config.tour_seen = true;
                            let _ = self.config.save_to_path(&self.paths.config_file());
                        }
                    });
            }
        }

        // Post-exit session summary (clean exits only, never crashes).
        if let Some((name, mins)) = self.session_summary.clone() {
            let mut close_summary = false;
            egui::Window::new(lang.sess_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .min_width(360.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(&name).strong());
                    let play_time_str = if mins == 0 {
                        lang.inst_never_played().to_string()
                    } else if mins < 60 {
                        format!("{} {}", mins, lang.inst_mins_suffix())
                    } else {
                        format!(
                            "{} {} {} {}",
                            mins / 60,
                            lang.inst_hours_suffix(),
                            mins % 60,
                            lang.inst_mins_suffix()
                        )
                    };
                    ui.label(egui::RichText::new(
                        lang.inst_playtime_label(&play_time_str),
                    ));
                    ui.add_space(12.0);
                    if ui.button(lang.wizard_hw_ok()).clicked() {
                        close_summary = true;
                    }
                });
            if close_summary {
                self.session_summary = None;
            }
        }

        // 1. Top custom titlebar
        TopBottomPanel::top("title_bar")
            .exact_height(36.0)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let tb_resp = TitleBar::show(ui, "Aleph Launcher", lang);
                if tb_resp.console_clicked {
                    self.console_modal.is_open = !self.console_modal.is_open;
                }
                if tb_resp.gallery_clicked {
                    self.screens_modal.open = !self.screens_modal.open;
                }
            });

        // 2. Bottom control bar
        TopBottomPanel::bottom("bottom_bar")
            .exact_height(76.0)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let active_acc = self.account_mgr.active_account();
                let is_launching = self.is_launching;

                let bbar_resp = BottomBar::show(
                    ui,
                    active_acc,
                    is_launching,
                    self.avatar_texture.as_ref(),
                    lang,
                );
                if bbar_resp.login_clicked {
                    self.login_modal.is_open = true;
                }
                if bbar_resp.launch_clicked {
                    self.handle_launch();
                }
                if bbar_resp.options_clicked {
                    self.current_tab = NavTab::Settings;
                }
            });

        // 3. Navigation: Full-style top bar in Professional, sidebar in Simple.
        // The slide id is driven in both branches so the animation replays
        // on every mode switch.
        let pro_slide_id = egui::Id::new("pro_topbar_slide");
        let is_pro = self.config.app_mode == amc_core::config::AppMode::Professional;
        if is_pro {
            let slide_t = ctx.animate_bool_responsive(pro_slide_id, true);
            let slide_px = (1.0 - slide_t) * topbar::TOPBAR_HEIGHT;
            let acc_name: Option<String> = self
                .account_mgr
                .active_account()
                .map(|a| a.username.clone());
            let avatar = self.avatar_texture.clone();
            let launching = self.is_launching;
            TopBottomPanel::top("pro_topbar")
                .exact_height(topbar::TOPBAR_HEIGHT)
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    let resp = topbar::show_topbar(
                        ui,
                        lang,
                        self.current_tab,
                        acc_name.as_deref(),
                        avatar.as_ref(),
                        launching,
                        slide_px,
                    );
                    if let Some(tab) = resp.tab_clicked {
                        self.current_tab = tab;
                    }
                    if resp.account_clicked {
                        self.login_modal.is_open = true;
                    }
                    if resp.play_clicked {
                        self.handle_launch();
                    }
                });
        } else {
            ctx.animate_bool_responsive(pro_slide_id, false);
            SidePanel::left("sidebar_panel")
                .exact_width(170.0)
                .resizable(false)
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    let exit_clicked = Sidebar::show(ui, &mut self.current_tab, lang);
                    if exit_clicked {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }
                });
        }

        // 4. Central content panel
        CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                let inner_rect = ui
                    .available_rect_before_wrap()
                    .shrink2(egui::vec2(24.0, 12.0));
                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inner_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                // Pre-launch identity picker (CONCEPT "Аккаунты").
                if self.pending_identity_pick {
                    let accounts = self.account_mgr.accounts().to_vec();
                    let lang = self.config.ui.language();
                    match self.identity_picker.show(
                        &mut content_ui,
                        &accounts,
                        &mut self.picker_nickname,
                        lang,
                    ) {
                        PickerAction::None => {}
                        PickerAction::Cancelled => {
                            self.pending_identity_pick = false;
                        }
                        PickerAction::Picked {
                            account_id,
                            remember,
                        } => {
                            let _ = self.account_mgr.set_active(account_id);
                            if remember {
                                self.config.remembered_identity = Some(account_id.to_string());
                                let _ = self.config.save_to_path(&self.paths.config_file());
                            }
                            self.pending_identity_pick = false;
                            self.handle_launch();
                        }
                        PickerAction::Offline { nickname, remember } => {
                            match amc_auth::login_offline(&nickname) {
                                Ok(acc) => {
                                    let id = acc.id;
                                    if self.store_account(acc) {
                                        let _ = self.account_mgr.set_active(id);
                                        if remember {
                                            self.config.remembered_identity = Some(id.to_string());
                                            let _ =
                                                self.config.save_to_path(&self.paths.config_file());
                                        }
                                        self.pending_identity_pick = false;
                                        self.handle_launch();
                                    }
                                }
                                Err(e) => {
                                    self.login_modal.error_msg = Some(e.to_string());
                                    self.login_modal.is_open = true;
                                    self.pending_identity_pick = false;
                                }
                            }
                        }
                    }
                }

                match self.current_tab {
                    NavTab::Home => {
                        let home_ctx = HomeContext {
                            versions_dir: &self.paths.versions_dir(),
                            instances_count: self.instances.len(),
                            memory_mb: self.config.default_launch_options.memory_max_mb,
                        };
                        self.home_page.show(
                            &mut content_ui,
                            &self.versions,
                            &mut self.selected_version,
                            &home_ctx,
                            &mut self.favorites,
                            lang,
                        );
                        if let Some((server, port)) = self.home_page.direct_connect_request.take() {
                            self.handle_direct_connect(server, port);
                        }
                        if self.home_page.hero_play_request {
                            self.home_page.hero_play_request = false;
                            self.handle_launch();
                        }
                        if let Some(id) = self.home_page.drawer_play_request.take() {
                            self.selected_version = Some(id);
                            self.handle_launch();
                        }
                        // Persist favorite servers.
                        if self.favorites != self.favorites_snapshot {
                            self.favorites_snapshot = self.favorites.clone();
                            if let Err(e) =
                                amc_minecraft::save_favorites(&self.paths.root_dir, &self.favorites)
                            {
                                tracing::warn!("Favorites persist failed: {e}");
                            }
                        }
                    }
                    NavTab::Modpacks => {
                        let instances_ctx = InstancesContext {
                            instances_dir: &self.paths.instances_dir(),
                            trash_dir: &self.paths.trash_dir(),
                            backups_root: &self.world_backups_root(),
                            accounts: self.account_mgr.accounts(),
                        };
                        let action = self.instances_page.show(
                            &mut content_ui,
                            &mut self.instances,
                            &mut self.selected_instance,
                            &instances_ctx,
                            lang,
                        );

                        match action {
                            InstanceAction::Created(inst) => {
                                let _ = inst.ensure_directories(&self.paths.instances_dir());
                                self.instances.push(inst);
                                let _ = Instance::save_all(
                                    &self.paths.instances_file(),
                                    &self.instances,
                                );
                            }
                            InstanceAction::Updated(inst) => {
                                if let Some(existing) =
                                    self.instances.iter_mut().find(|i| i.id == inst.id)
                                {
                                    *existing = inst.clone();
                                }
                                if self.selected_instance == Some(inst.id) {
                                    self.selected_version = Some(inst.game_version.clone());
                                }
                                let _ = Instance::save_all(
                                    &self.paths.instances_file(),
                                    &self.instances,
                                );
                            }
                            InstanceAction::Cloned(id) => {
                                if let Some(inst) =
                                    self.instances.iter().find(|i| i.id == id).cloned()
                                {
                                    let copy_name = match lang {
                                        amc_core::Language::English => {
                                            format!("{} (Copy)", inst.name)
                                        }
                                        amc_core::Language::Russian => {
                                            format!("{} (Копия)", inst.name)
                                        }
                                        amc_core::Language::Ukrainian => {
                                            format!("{} (Копія)", inst.name)
                                        }
                                    };
                                    if let Ok(cloned) =
                                        inst.clone_instance(&copy_name, &self.paths.instances_dir())
                                    {
                                        let new_id = cloned.id;
                                        self.instances.push(cloned);
                                        self.selected_instance = Some(new_id);
                                        let _ = Instance::save_all(
                                            &self.paths.instances_file(),
                                            &self.instances,
                                        );
                                    }
                                }
                            }
                            InstanceAction::Trashed(id) => {
                                if let Some(inst) =
                                    self.instances.iter().find(|i| i.id == id).cloned()
                                {
                                    if let Err(e) = amc_core::trash_instance(
                                        &self.paths.instances_dir(),
                                        &self.paths.trash_dir(),
                                        &inst,
                                    ) {
                                        tracing::error!("Move to trash failed: {e}");
                                    }
                                }
                                self.instances.retain(|i| i.id != id);
                                if self.selected_instance == Some(id) {
                                    self.selected_instance = self.instances.first().map(|i| i.id);
                                }
                                let _ = Instance::save_all(
                                    &self.paths.instances_file(),
                                    &self.instances,
                                );
                            }
                            InstanceAction::Restored(id) => {
                                match amc_core::restore_instance(
                                    &self.paths.instances_dir(),
                                    &self.paths.trash_dir(),
                                    id,
                                ) {
                                    Ok(inst) => {
                                        let new_id = inst.id;
                                        self.instances.push(inst);
                                        self.selected_instance = Some(new_id);
                                        let _ = Instance::save_all(
                                            &self.paths.instances_file(),
                                            &self.instances,
                                        );
                                    }
                                    Err(e) => tracing::error!("Restore failed: {e}"),
                                }
                            }
                            InstanceAction::TrashPurged(id) => {
                                if let Err(e) =
                                    amc_core::purge_trash_entry(&self.paths.trash_dir(), id)
                                {
                                    tracing::error!("Trash purge failed: {e}");
                                }
                            }
                            InstanceAction::TrashEmptied => {
                                let n = amc_core::purge_trash_all(&self.paths.trash_dir());
                                tracing::info!("Emptied trash ({n} entries)");
                            }
                            InstanceAction::Selected(id) => {
                                if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                                    self.selected_version = Some(inst.game_version.clone());
                                }
                            }
                            InstanceAction::Launch(id) => {
                                self.selected_instance = Some(id);
                                if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                                    self.selected_version = Some(inst.game_version.clone());
                                }
                                self.handle_launch();
                            }
                            InstanceAction::None => {}
                        }
                    }
                    NavTab::Mods => {
                        let base_dir = if let Some(id) = self.selected_instance {
                            if let Some(inst) = self.instances.iter().find(|i| i.id == id) {
                                inst.get_game_dir(&self.paths.instances_dir())
                            } else {
                                self.paths.root_dir.clone()
                            }
                        } else {
                            self.paths.root_dir.clone()
                        };
                        let mods_dir = base_dir.join("mods");
                        let mut search_req = None;
                        let mut mod_to_install = None;

                        let mut update_install = None;
                        self.mods_page.show(
                            &mut content_ui,
                            &mods_dir,
                            lang,
                            &mut self.config.wishlist,
                            &mut update_install,
                            |query, provider, category| {
                                search_req = Some((query, provider, category));
                            },
                            |result| {
                                mod_to_install = Some(result);
                            },
                        );

                        if let Some((item, old_filename)) = update_install {
                            self.install_mod(item, Some(old_filename));
                        }
                        // Persist wishlist hearts.
                        if self.config.wishlist != self.wishlist_snapshot {
                            self.wishlist_snapshot = self.config.wishlist.clone();
                            let _ = self.config.save_to_path(&self.paths.config_file());
                        }

                        if let Some((q, prov, cat)) = search_req {
                            self.search_mods(q, prov, cat);
                        }
                        if let Some(item) = mod_to_install {
                            self.install_mod(item, None);
                        }
                    }
                    NavTab::Skins => {
                        let acc = self.account_mgr.active_account();
                        self.skins_page.show(&mut content_ui, acc, lang);
                    }
                    NavTab::Profile => {
                        let acc = self.account_mgr.active_account();
                        let slim = self.skins_page.is_slim_model;
                        let skin = self.skins_page.skin_image.as_ref();
                        match self
                            .profile_page
                            .show(&mut content_ui, acc, skin, slim, lang)
                        {
                            ProfileAction::None => {}
                            ProfileAction::OpenSkins => {
                                self.current_tab = NavTab::Skins;
                            }
                            ProfileAction::OpenLogin => {
                                self.login_modal.is_open = true;
                            }
                        }
                    }
                    NavTab::Settings => {
                        let server_running = self.server_child.is_some();
                        self.settings_page.show(
                            &mut content_ui,
                            &mut self.config,
                            &mut self.account_mgr,
                            &self.paths,
                            server_running,
                            &self.server_status,
                        );
                        // Auto-save configuration changes
                        let _ = self.config.save_to_path(&self.paths.config_file());
                    }
                }
            });

        // Modals
        let mut request_ms = false;
        let mut pending_account = None;
        self.login_modal.show(
            ctx,
            lang,
            |account| {
                pending_account = Some(account);
            },
            || {
                request_ms = true;
            },
        );
        if let Some(account) = pending_account {
            self.store_account(account);
        }

        if request_ms {
            self.request_ms_device_code();
        }

        // Game Console Modal
        self.console_modal.show(ctx, lang);

        // Screenshot gallery (CONCEPT "Скриншоты", P10).
        let instances_dir = self.paths.instances_dir();
        self.screens_modal.show(ctx, &instances_dir, lang);
    }
}
