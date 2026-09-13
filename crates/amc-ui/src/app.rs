use eframe::App;
use egui::{CentralPanel, Context, TopBottomPanel};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use amc_auth::AccountManager;
use amc_core::config::LauncherConfig;
use amc_core::paths::LauncherPaths;
use amc_core::types::{GameVersion, Instance};
use amc_downloader::{DownloadEngine, DownloadProgress};
use amc_minecraft::{GameEvent, VersionManifest};

use crate::modals::{DownloadOverlay, LoginModal};
use crate::pages::{HomePage, InstancesPage, ModsPage, SettingsPage, SkinsPage};
use crate::theme::apply_aleph_theme;
use crate::widgets::{BottomBar, NavTab, Sidebar, TitleBar};

pub struct LauncherApp {
    paths: LauncherPaths,
    config: LauncherConfig,
    account_mgr: AccountManager,
    #[allow(dead_code)]
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
    download_progress_rx: Option<watch::Receiver<DownloadProgress>>,
    game_events_rx: Option<mpsc::Receiver<GameEvent>>,
}

impl LauncherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let paths = LauncherPaths::default_paths().expect("Failed to initialize launcher paths");
        let config = LauncherConfig::load_from_path(&paths.config_file()).unwrap_or_default();
        let account_mgr = AccountManager::load_from_path(paths.accounts_file()).unwrap_or_else(|_| {
            AccountManager::load_from_path("accounts.json").unwrap_or_else(|_| panic!("Failed to load accounts"))
        });

        let download_engine = Arc::new(DownloadEngine::new(16));

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
            mods_page: ModsPage::default(),
            skins_page: SkinsPage::default(),
            settings_page: SettingsPage::default(),
            login_modal: LoginModal::default(),
            download_progress_rx: None,
            game_events_rx: None,
        }
    }

    pub fn load_initial_manifest(&mut self) {
        let client = reqwest::Client::new();
        let cache_dir = self.paths.cache_dir();

        // Spawn version manifest fetch
        tokio::spawn(async move {
            if let Ok(manifest) = VersionManifest::fetch(&client, Some(&cache_dir)).await {
                let _versions = manifest.to_game_versions();
                tracing::info!("Loaded {} versions from Mojang manifest", _versions.len());
            }
        });
    }

    fn handle_launch(&mut self) {
        let Some(session) = self.account_mgr.get_session() else {
            self.login_modal.is_open = true;
            return;
        };

        let ver_id = self.selected_version.clone().unwrap_or_else(|| "1.20.1".to_string());
        tracing::info!("Launching Minecraft version {} for user {}", ver_id, session.username);
        self.is_launching = true;
    }
}

impl App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_aleph_theme(ctx);

        // Check incoming download progress updates
        if let Some(rx) = &self.download_progress_rx {
            let progress = rx.borrow().clone();
            DownloadOverlay::show(ctx, &progress, "Загрузка компонентов...", || {
                // Cancel callback
            });
        }

        // Check game events
        if let Some(rx) = &mut self.game_events_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    GameEvent::Started { pid } => {
                        tracing::info!("Minecraft process started with PID {pid}");
                        self.is_launching = false;
                        if self.config.ui.close_after_launch {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

        // Top custom titlebar
        TopBottomPanel::top("title_bar")
            .exact_height(36.0)
            .show(ctx, |ui| {
                TitleBar::show(ui, "Aleph Launcher");
            });

        // Bottom control bar
        TopBottomPanel::bottom("bottom_bar")
            .exact_height(76.0)
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

        // Left sidebar
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let exit_clicked = Sidebar::show(ui, &mut self.current_tab);
                if exit_clicked {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Main Page Content Area
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    match self.current_tab {
                        NavTab::Home => {
                            self.home_page.show(ui, &self.versions, &mut self.selected_version);
                        }
                        NavTab::Modpacks => {
                            self.instances_page.show(
                                ui,
                                &mut self.instances,
                                &mut self.selected_instance,
                            );
                        }
                        NavTab::Mods => {
                            let mods_dir = self.paths.root_dir.join("mods");
                            self.mods_page.show(
                                ui,
                                &mods_dir,
                                |_query| {},
                                |_result| {},
                            );
                        }
                        NavTab::Skins => {
                            let acc = self.account_mgr.active_account();
                            self.skins_page.show(ui, acc);
                        }
                        NavTab::Settings => {
                            self.settings_page.show(ui, &mut self.config, &mut self.account_mgr);
                        }
                    }
                });
            });
        });

        // Login Modal Dialog
        self.login_modal.show(
            ctx,
            |account| {
                let _ = self.account_mgr.add_or_update_account(account);
            },
            || {
                // Request Microsoft device code
            },
        );

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
