use eframe::App;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel, ViewportCommand};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use amc_auth::{Account, AccountManager, DeviceCodeResponse, MicrosoftAuthFlow};
use amc_core::config::LauncherConfig;
use amc_core::paths::LauncherPaths;
use amc_core::types::{GameVersion, Instance, LoaderType};
use amc_downloader::{AdoptiumInstaller, DownloadEngine, DownloadProgress};
use amc_minecraft::{FabricLoader, GameEvent, MinecraftLauncher, QuiltLoader, VersionDetails, VersionManifest};
use amc_mods::{CurseForgeClient, LocalModManager, ModSearchResult, ModSource, ModrinthClient};

use crate::modals::{ConsoleModal, DownloadOverlay, LoginModal};
use crate::pages::{HomePage, InstanceAction, InstancesPage, ModsPage, SettingsPage, SkinsPage};
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

    // Async channels
    versions_rx: Option<mpsc::Receiver<Vec<GameVersion>>>,
    mod_search_rx: Option<mpsc::Receiver<Vec<ModSearchResult>>>,
    mod_install_rx: Option<mpsc::Receiver<Result<(String, String), String>>>,
    progress_init_rx: Option<mpsc::Receiver<Option<watch::Receiver<DownloadProgress>>>>,
    launch_status_rx: Option<mpsc::Receiver<String>>,
    download_progress_rx: Option<watch::Receiver<DownloadProgress>>,
    game_events_rx: Option<mpsc::Receiver<GameEvent>>,
    ms_code_rx: Option<mpsc::Receiver<Result<DeviceCodeResponse, String>>>,
    ms_poll_rx: Option<mpsc::Receiver<Result<Account, String>>>,
}

impl LauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        // Load persisted instances
        let instances = Instance::load_all(&paths.instances_file()).unwrap_or_default();
        let selected_instance = instances.first().map(|i| i.id);
        let selected_version = instances
            .first()
            .map(|i| i.game_version.clone())
            .or_else(|| Some("1.20.1".to_string()));

        Self {
            paths,
            config,
            account_mgr,
            download_engine,
            current_tab: NavTab::Home,
            selected_version,
            selected_instance,
            versions: Vec::new(),
            instances,
            is_launching: false,
            launch_status_text: "Подготовка к запуску...".to_string(),
            home_page: HomePage::default(),
            instances_page: InstancesPage::default(),
            mods_page,
            skins_page: SkinsPage::default(),
            settings_page: SettingsPage::default(),
            login_modal: LoginModal::default(),
            console_modal: ConsoleModal::default(),
            versions_rx: None,
            mod_search_rx: None,
            mod_install_rx: None,
            progress_init_rx: None,
            launch_status_rx: None,
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

    fn search_mods(&mut self, query: String, provider: ModSource) {
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
                    match client.search(&query, None, None, 30).await {
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

    fn install_mod(&mut self, mod_item: ModSearchResult) {
        let mods_dir = self.paths.root_dir.join("mods");
        let engine = self.download_engine.clone();
        let (tx, rx) = mpsc::channel(1);
        self.mod_install_rx = Some(rx);

        tokio::spawn(async move {
            let file_result = match mod_item.source {
                ModSource::CurseForge => {
                    let client = CurseForgeClient::default();
                    client.get_download_file(&mod_item.id, None, None).await
                }
                _ => {
                    let client = ModrinthClient::default();
                    client.get_download_file(&mod_item.id, None, None).await
                }
            };

            match file_result {
                Ok(file_info) => {
                    let dest = mods_dir.join(&file_info.filename);
                    let item = amc_downloader::DownloadItem::new(&file_info.url, dest);
                    if let Err(e) = engine.download_one(&item, None).await {
                        tracing::error!("Failed to download mod {}: {e}", file_info.filename);
                        let _ = tx.send(Err(format!("Ошибка загрузки {}: {e}", file_info.filename))).await;
                    } else {
                        tracing::info!("Mod {} installed successfully!", file_info.filename);
                        let _ = tx.send(Ok((mod_item.id, mod_item.title))).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get mod file info: {e}");
                    let _ = tx.send(Err(format!("Не удалось получить файл мода: {e}"))).await;
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

        let paths = self.paths.clone();
        let (game_dir, ver_id, ram_override, loader) = if let Some(inst_id) = self.selected_instance {
            if let Some(inst) = self.instances.iter().find(|i| i.id == inst_id) {
                let _ = inst.ensure_directories(&paths.instances_dir());
                (inst.get_game_dir(&paths.instances_dir()), inst.game_version.clone(), inst.ram_mb, inst.loader)
            } else {
                let ver = self.selected_version.clone().unwrap_or_else(|| "1.20.1".to_string());
                (paths.instances_dir().join(&ver), ver, None, LoaderType::Vanilla)
            }
        } else {
            let ver = self.selected_version.clone().unwrap_or_else(|| "1.20.1".to_string());
            (paths.instances_dir().join(&ver), ver, None, LoaderType::Vanilla)
        };

        tracing::info!(
            "Preparing Minecraft launch: Version={}, Loader={}, User={}...",
            ver_id,
            loader.as_str(),
            session.username
        );

        self.is_launching = true;
        self.launch_status_text = format!("Подготовка версии {}...", ver_id);

        let engine = self.download_engine.clone();
        let mut options = self.config.default_launch_options.clone();
        if let Some(ram) = ram_override {
            options.memory_max_mb = ram;
        }

        let (game_tx, game_rx) = mpsc::channel(256);
        let (status_tx, status_rx) = mpsc::channel(32);
        let (prog_tx, prog_rx) = mpsc::channel(8);

        self.game_events_rx = Some(game_rx);
        self.launch_status_rx = Some(status_rx);
        self.progress_init_rx = Some(prog_rx);

        tokio::spawn(async move {
            let client = reqwest::Client::new();

            // 1. Resolve Version Details
            let _ = status_tx.send("Получение информации о версии...".to_string()).await;
            let details = match loader {
                LoaderType::Fabric => {
                    let _ = status_tx.send("Поиск Fabric Loader...".to_string()).await;
                    match FabricLoader::get_latest_loader_version(&client, &ver_id).await {
                        Ok(loader_ver) => {
                            let _ = status_tx.send(format!("Загрузка профиля Fabric {loader_ver}...")).await;
                            match FabricLoader::fetch_profile_json(&client, &ver_id, &loader_ver).await {
                                Ok(mut fab_details) => {
                                    if let Ok(vanilla) = VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                        fab_details.merge_parent(vanilla);
                                    }
                                    fab_details
                                }
                                Err(e) => {
                                    tracing::warn!("Fabric profile fetch error: {e}, falling back to Vanilla");
                                    match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                        Ok(d) => d,
                                        Err(err) => {
                                            let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка загрузки версии {ver_id}: {err}") }).await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Fabric loader lookup error: {e}, falling back to Vanilla");
                            match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                Ok(d) => d,
                                Err(err) => {
                                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка загрузки версии {ver_id}: {err}") }).await;
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
                            let _ = status_tx.send(format!("Загрузка профиля Quilt {loader_ver}...")).await;
                            match QuiltLoader::fetch_profile_json(&client, &ver_id, &loader_ver).await {
                                Ok(mut quilt_details) => {
                                    if let Ok(vanilla) = VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                        quilt_details.merge_parent(vanilla);
                                    }
                                    quilt_details
                                }
                                Err(e) => {
                                    tracing::warn!("Quilt profile fetch error: {e}, falling back to Vanilla");
                                    match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                        Ok(d) => d,
                                        Err(err) => {
                                            let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка загрузки версии {ver_id}: {err}") }).await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Quilt loader lookup error: {e}, falling back to Vanilla");
                            match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                                Ok(d) => d,
                                Err(err) => {
                                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка загрузки версии {ver_id}: {err}") }).await;
                                    return;
                                }
                            }
                        }
                    }
                }
                _ => {
                    match VersionDetails::fetch_or_load(&client, &ver_id, None, &paths.versions_dir()).await {
                        Ok(d) => d,
                        Err(err) => {
                            let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка загрузки версии {ver_id}: {err}") }).await;
                            return;
                        }
                    }
                }
            };

            // 2. Resolve Java Runtime
            let req_java = details.required_java_major();
            let _ = status_tx.send(format!("Проверка Java {req_java}...")).await;
            let java_bin = if let Some(custom_java) = &options.java_path {
                if custom_java.is_file() {
                    custom_java.clone()
                } else {
                    AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), req_java, &engine, None).await.unwrap_or_else(|_| custom_java.clone())
                }
            } else {
                match AdoptiumInstaller::ensure_java(&paths.runtimes_dir(), req_java, &engine, None).await {
                    Ok(bin) => bin,
                    Err(e) => {
                        tracing::error!("Java installation failed: {e}");
                        let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка установки Java {req_java}: {e}") }).await;
                        return;
                    }
                }
            };

            // 3. Collect download items
            let _ = status_tx.send("Проверка файлов Minecraft...".to_string()).await;
            let mut download_items: Vec<amc_downloader::DownloadItem> = Vec::new();
            let client_jar = paths.versions_dir().join(&ver_id).join(format!("{ver_id}.jar"));

            if let Some(downloads) = &details.downloads {
                if let Some(client_file) = &downloads.client {
                    if !client_jar.is_file() {
                        let mut item = amc_downloader::DownloadItem::new(&client_file.url, &client_jar);
                        if let Some(sha1) = &client_file.sha1 { item = item.with_sha1(sha1); }
                        if let Some(size) = client_file.size { item = item.with_size(size); }
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
                let _ = status_tx.send("Проверка ассетов игры...".to_string()).await;
                if let Ok(idx) = amc_minecraft::AssetIndex::fetch_or_load(&client, asset_ref, &paths.assets_dir()).await {
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
                let _ = status_tx.send(format!("Загрузка {count} файлов...").to_string()).await;
                let (tracker, progress_rx) = engine.create_tracker(&download_items);
                let _ = prog_tx.send(Some(progress_rx)).await;

                if let Err(e) = engine.download_all_with_progress(download_items, Some(tracker)).await {
                    tracing::error!("Component download failed: {e}");
                    let _ = prog_tx.send(None).await;
                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка скачивания компонентов: {e}") }).await;
                    return;
                }
                let _ = prog_tx.send(None).await;
            }

            // 5. Unpack natives
            let natives_dir = paths.libraries_dir().join("natives").join(&ver_id);
            let _ = tokio::fs::create_dir_all(&natives_dir).await;
            let _ = status_tx.send("Распаковка natives...".to_string()).await;
            for nat_jar in &natives_jars {
                let _ = VersionDetails::extract_natives(nat_jar, &natives_dir);
            }

            // 6. Launch Minecraft
            let _ = status_tx.send("Запуск процесса Minecraft...".to_string()).await;
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
            ).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        let _ = game_tx.send(event).await;
                    }
                }
                Err(e) => {
                    let _ = game_tx.send(GameEvent::Crashed { message: format!("Ошибка запуска: {e}") }).await;
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
                    Ok((id, title)) => {
                        self.mods_page.installing_ids.remove(&id);
                        self.mods_page.installed_titles.insert(title.to_lowercase());
                        let mods_dir = self.paths.root_dir.join("mods");
                        self.mods_page.local_mods = LocalModManager::scan_mods(&mods_dir).unwrap_or_default();
                        self.mods_page.status_message = Some((format!("Мод \"{title}\" успешно установлен!"), true));
                    }
                    Err(err) => {
                        self.mods_page.status_message = Some((err, false));
                    }
                }
                self.mod_install_rx = None;
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
            DownloadOverlay::show(ctx, &progress, &self.launch_status_text, || {
                // Cancel
            });
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
                        self.is_launching = false;
                        if self.config.ui.close_after_launch {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    }
                    GameEvent::LogLine(line) => {
                        tracing::debug!("[MC] {line}");
                        self.console_modal.push_line(line);
                    }
                    GameEvent::Exited { code } => {
                        let msg = format!("Процесс Minecraft завершен с кодом {:?}", code);
                        tracing::info!("{msg}");
                        self.console_modal.push_line(format!("[ALEPH] {msg}"));
                        self.is_launching = false;
                    }
                    GameEvent::Crashed { message } => {
                        tracing::error!("Ошибка процесса Minecraft: {message}");
                        self.console_modal.push_line(format!("[ERROR] {message}"));
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
                let tb_resp = TitleBar::show(ui, "Aleph Launcher");
                if tb_resp.console_clicked {
                    self.console_modal.is_open = !self.console_modal.is_open;
                }
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

        // 4. Central content panel
        CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
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
                        let action = self.instances_page.show(
                            &mut content_ui,
                            &mut self.instances,
                            &mut self.selected_instance,
                            &self.paths.instances_dir(),
                        );

                        match action {
                            InstanceAction::Created(inst) => {
                                let _ = inst.ensure_directories(&self.paths.instances_dir());
                                self.instances.push(inst);
                                let _ = Instance::save_all(&self.paths.instances_file(), &self.instances);
                            }
                            InstanceAction::Deleted(id) => {
                                self.instances.retain(|i| i.id != id);
                                if self.selected_instance == Some(id) {
                                    self.selected_instance = self.instances.first().map(|i| i.id);
                                }
                                let _ = Instance::save_all(&self.paths.instances_file(), &self.instances);
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
                        let mods_dir = self.paths.root_dir.join("mods");
                        let mut search_req = None;
                        let mut mod_to_install = None;

                        self.mods_page.show(
                            &mut content_ui,
                            &mods_dir,
                            |query, provider| {
                                search_req = Some((query, provider));
                            },
                            |result| {
                                mod_to_install = Some(result);
                            },
                        );

                        if let Some((q, prov)) = search_req {
                            self.search_mods(q, prov);
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
                        self.settings_page.show(&mut content_ui, &mut self.config, &mut self.account_mgr, &self.paths);
                        // Auto-save configuration changes
                        let _ = self.config.save_to_path(&self.paths.config_file());
                    }
                }
            });

        // Modals
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

        // Game Console Modal
        self.console_modal.show(ctx);
    }
}
