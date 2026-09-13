use eframe::App;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel, ViewportCommand};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use amc_auth::{Account, AccountManager, DeviceCodeResponse, MicrosoftAuthFlow};
use amc_core::config::LauncherConfig;
use amc_core::paths::LauncherPaths;
use amc_core::types::{GameVersion, Instance};
use amc_downloader::{AdoptiumInstaller, DownloadEngine, DownloadProgress};
use amc_minecraft::{GameEvent, MinecraftLauncher, VersionManifest};
use amc_mods::{LocalModManager, ModSearchResult, ModrinthClient};

use crate::modals::{DownloadOverlay, LoginModal};
use crate::pages::{HomePage, InstancesPage, ModsPage, SettingsPage, SkinsPage};
use crate::theme::{apply_aleph_theme, BG};
use crate::widgets::{BottomBar, NavTab, Sidebar, TitleBar};

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

    // Pages
    home_page: HomePage,
    instances_page: InstancesPage,
    mods_page: ModsPage,
    skins_page: SkinsPage,
    settings_page: SettingsPage,

    // Modals
    login_modal: LoginModal,

    // Async channels
    versions_rx: Option<mpsc::Receiver<Vec<GameVersion>>>,
    mod_search_rx: Option<mpsc::Receiver<Vec<ModSearchResult>>>,
    download_progress_rx: Option<watch::Receiver<DownloadProgress>>,
    game_events_rx: Option<mpsc::Receiver<GameEvent>>,
    ms_code_rx: Option<mpsc::Receiver<Result<DeviceCodeResponse, String>>>,
    ms_poll_rx: Option<mpsc::Receiver<Result<Account, String>>>,
}

impl LauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply theme ONCE at startup to avoid per-frame allocations
        apply_aleph_theme(&cc.egui_ctx);

        let paths = LauncherPaths::default_paths().expect("Failed to initialize launcher paths");
        let config = LauncherConfig::load_from_path(&paths.config_file()).unwrap_or_default();
        let account_mgr = AccountManager::load_from_path(paths.accounts_file()).unwrap_or_else(|_| {
            AccountManager::load_from_path("accounts.json").unwrap_or_else(|_| panic!("Failed to load accounts"))
        });

        let download_engine = Arc::new(DownloadEngine::new(16));

        // Pre-scan local mods
        let mods_dir = paths.root_dir.join("mods");
        let local_mods = LocalModManager::scan_mods(&mods_dir).unwrap_or_default();

        let mut mods_page = ModsPage::default();
        mods_page.local_mods = local_mods;

        Self {
            paths,
            config,
            account_mgr,
            download_engine,
            current_tab: NavTab::Home,
            selected_version: Some("1.20.1".to_string()),
            selected_instance: None,
            versions: Vec::new(),
            instances: Vec::new(),
            is_launching: false,
            home_page: HomePage::default(),
            instances_page: InstancesPage::default(),
            mods_page,
            skins_page: SkinsPage::default(),
            settings_page: SettingsPage::default(),
            login_modal: LoginModal::default(),
            versions_rx: None,
            mod_search_rx: None,
            download_progress_rx: None,
            game_events_rx: None,
            ms_code_rx: None,
            ms_poll_rx: None,
        }
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

    fn search_modrinth(&mut self, query: String) {
        self.mods_page.is_searching = true;
        let (tx, rx) = mpsc::channel(1);
        self.mod_search_rx = Some(rx);

        tokio::spawn(async move {
            let client = ModrinthClient::default();
            match client.search(&query, None, None, 30).await {
                Ok(results) => {
                    let _ = tx.send(results).await;
                }
                Err(e) => {
                    tracing::error!("Modrinth search failed: {e}");
                    let _ = tx.send(Vec::new()).await;
                }
            }
        });
    }

    fn install_mod(&mut self, mod_item: ModSearchResult) {
        let mods_dir = self.paths.root_dir.join("mods");
        let engine = self.download_engine.clone();

        tokio::spawn(async move {
            let client = ModrinthClient::default();
            tracing::info!("Resolving mod download for {}", mod_item.title);
            if let Ok(file_info) = client.get_download_file(&mod_item.id, None, None).await {
                let dest = mods_dir.join(&file_info.filename);
                let item = amc_downloader::DownloadItem::new(&file_info.url, dest);
                if let Err(e) = engine.download_one(&item, None).await {
                    tracing::error!("Failed to download mod {}: {e}", file_info.filename);
                } else {
                    tracing::info!("Mod {} installed successfully!", file_info.filename);
                }
            }
        });
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
        let Some(session) = self.account_mgr.get_session() else {
            self.login_modal.is_open = true;
            return;
        };

        let ver_id = self.selected_version.clone().unwrap_or_else(|| "1.20.1".to_string());
        tracing::info!("Preparing Minecraft launch for version {} (User: {})...", ver_id, session.username);
        self.is_launching = true;

        let paths = self.paths.clone();
        let engine = self.download_engine.clone();
        let options = self.config.default_launch_options.clone();

        let (game_tx, game_rx) = mpsc::channel(256);
        self.game_events_rx = Some(game_rx);

        tokio::spawn(async move {
            let runtimes_dir = paths.runtimes_dir();
            let java_bin = match AdoptiumInstaller::ensure_java(&runtimes_dir, 21, &engine, None).await {
                Ok(bin) => bin,
                Err(e) => {
                    tracing::error!("Java installation failed: {e}");
                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Java error: {e}") }).await;
                    return;
                }
            };

            tracing::info!("Java ready: {}", java_bin.display());

            let game_dir = paths.instances_dir().join(&ver_id);
            let assets_dir = paths.assets_dir();
            let natives_dir = paths.libraries_dir().join("natives");
            let client_jar = paths.versions_dir().join(&ver_id).join(format!("{ver_id}.jar"));

            let dummy_details = amc_minecraft::VersionDetails {
                id: ver_id.clone(),
                downloads: None,
                asset_index: None,
                libraries: Vec::new(),
                main_class: "net.minecraft.client.main.Main".to_string(),
                arguments: None,
                minecraft_arguments: None,
                java_version: None,
                inherits_from: None,
            };

            let _ = tokio::fs::create_dir_all(&game_dir).await;
            let _ = tokio::fs::create_dir_all(&natives_dir).await;

            match MinecraftLauncher::launch(
                &java_bin,
                &game_dir,
                &assets_dir,
                &natives_dir,
                &[],
                &client_jar,
                &dummy_details,
                &session,
                &options,
            ).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        let _ = game_tx.send(event).await;
                    }
                }
                Err(e) => {
                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Launch error: {e}") }).await;
                }
            }
        });
    }
}

impl App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Receive version manifest updates
        if let Some(rx) = &mut self.versions_rx {
            if let Ok(versions) = rx.try_recv() {
                self.versions = versions;
                self.versions_rx = None;
            }
        }

        // Receive Modrinth search results
        if let Some(rx) = &mut self.mod_search_rx {
            if let Ok(results) = rx.try_recv() {
                self.mods_page.search_results = results;
                self.mods_page.is_searching = false;
                self.mod_search_rx = None;
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
                        let _ = self.account_mgr.add_or_update_account(acc);
                        self.login_modal.is_open = false;
                        self.login_modal.ms_polling = false;
                        self.login_modal.ms_device_code = None;
                        self.ms_poll_rx = None;
                    }
                    Err(err) => {
                        self.login_modal.error_msg = Some(err);
                        self.login_modal.ms_polling = false;
                        self.ms_poll_rx = None;
                    }
                }
            }
        }

        // Check incoming download progress updates
        if let Some(rx) = &self.download_progress_rx {
            let progress = rx.borrow().clone();
            DownloadOverlay::show(ctx, &progress, "Загрузка компонентов...", || {
                // Cancel callback
            });
            // Keep rendering at 60 FPS while download is active
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // Check game events
        if let Some(rx) = &mut self.game_events_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    GameEvent::Started { pid } => {
                        tracing::info!("Minecraft process started with PID {pid}");
                        self.is_launching = false;
                        if self.config.ui.close_after_launch {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    }
                    GameEvent::LogLine(line) => {
                        tracing::debug!("[MC] {line}");
                    }
                    GameEvent::Exited { code } => {
                        tracing::info!("Minecraft process exited with code {:?}", code);
                        self.is_launching = false;
                    }
                    GameEvent::Crashed { message } => {
                        tracing::error!("Minecraft process crashed: {message}");
                        self.is_launching = false;
                    }
                }
            }
        }

        // 1. Top custom titlebar
        TopBottomPanel::top("title_bar")
            .exact_height(36.0)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                TitleBar::show(ui, "Aleph Launcher");
            });

        // 2. Bottom control bar
        TopBottomPanel::bottom("bottom_bar")
            .exact_height(76.0)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let active_acc = self.account_mgr.active_account();
                let is_launching = self.is_launching;

                let bbar_resp = BottomBar::show(ui, active_acc, is_launching);
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

        // 3. Left sidebar dock panel
        SidePanel::left("sidebar_panel")
            .exact_width(170.0)
            .resizable(false)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let exit_clicked = Sidebar::show(ui, &mut self.current_tab);
                if exit_clicked {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });

        // 4. Central content panel (fills all remaining area cleanly)
        CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                // Generous inner margins for the pages
                let inner_rect = ui.available_rect_before_wrap().shrink2(egui::vec2(24.0, 12.0));
                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inner_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                match self.current_tab {
                    NavTab::Home => {
                        self.home_page.show(&mut content_ui, &self.versions, &mut self.selected_version);
                    }
                    NavTab::Modpacks => {
                        self.instances_page.show(
                            &mut content_ui,
                            &mut self.instances,
                            &mut self.selected_instance,
                        );
                    }
                    NavTab::Mods => {
                        let mods_dir = self.paths.root_dir.join("mods");
                        let mut query_to_search = None;
                        let mut mod_to_install = None;

                        self.mods_page.show(
                            &mut content_ui,
                            &mods_dir,
                            |query| {
                                query_to_search = Some(query);
                            },
                            |result| {
                                mod_to_install = Some(result);
                            },
                        );

                        if let Some(q) = query_to_search {
                            self.search_modrinth(q);
                        }
                        if let Some(item) = mod_to_install {
                            self.install_mod(item);
                        }
                    }
                    NavTab::Skins => {
                        let acc = self.account_mgr.active_account();
                        self.skins_page.show(&mut content_ui, acc);
                    }
                    NavTab::Settings => {
                        self.settings_page.show(&mut content_ui, &mut self.config, &mut self.account_mgr);
                    }
                }
            });

        // Login Modal Dialog
        let mut request_ms = false;
        self.login_modal.show(
            ctx,
            |account| {
                let _ = self.account_mgr.add_or_update_account(account);
            },
            || {
                request_ms = true;
            },
        );

        if request_ms {
            self.request_ms_device_code();
        }
    }
}
