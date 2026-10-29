use crate::theme;
use crate::theme::{
    lerp_color, with_alpha, BG_ACTIVE, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, MODRINTH_BG,
    MODRINTH_GREEN, MODRINTH_SURFACE, MODRINTH_SURFACE_HIGH, MODRINTH_SURFACE_HOVER,
    MODRINTH_TEXT_CONTRAST, MODRINTH_TEXT_SECONDARY, RUBY, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED,
    TEXT_PRIMARY,
};
use crate::widgets::choice_chip;
use crate::widgets::icons::{self, icon_button, Icon};
use crate::widgets::modrinth_ui;
use crate::widgets::pixel;
use amc_auth::AccountManager;
use amc_core::config::{AppMode, LauncherConfig};
use amc_core::paths::LauncherPaths;
use amc_core::Language;
use egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Rect, Rounding, ScrollArea, Sense, Stroke,
    TextEdit, Ui,
};

/// Settings page language. Aleph keeps the original compact composition;
/// Modrinth uses the reference settings modal's vertical section rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSkin {
    #[default]
    Amc,
    Modrinth,
    Millida,
    Multimc,
    Labymod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSubTab {
    #[default]
    General,
    Java,
    Accounts,
}

/// Localized (name, description) pair for a visual style row. Shared by the
/// Settings page and the first-run wizard so both stay in sync.
pub fn mode_labels(lang: Language, mode: AppMode) -> (&'static str, &'static str) {
    match mode {
        AppMode::Simple => (
            lang.settings_mode_simple(),
            lang.settings_mode_simple_desc(),
        ),
        AppMode::Professional => (lang.settings_mode_pro(), lang.settings_mode_pro_desc()),
        AppMode::Multimc => (
            lang.settings_mode_multimc(),
            lang.settings_mode_multimc_desc(),
        ),
        AppMode::Ultra => (lang.settings_mode_ultra(), lang.settings_mode_ultra_desc()),
        AppMode::Labymod => (
            lang.settings_mode_labymod(),
            lang.settings_mode_labymod_desc(),
        ),
        AppMode::Millida => (
            lang.settings_mode_millida(),
            lang.settings_mode_millida_desc(),
        ),
        AppMode::Modrinth => (
            lang.settings_mode_modrinth(),
            lang.settings_mode_modrinth_desc(),
        ),
    }
}

pub struct SettingsPage {
    pub sub_tab: SettingsSubTab,
    skin: SettingsSkin,
    pub custom_jvm_arg_input: String,
    pub custom_game_arg_input: String,
    java_scan: Option<Vec<amc_downloader::SystemJava>>,
    java_scan_rx: Option<std::sync::mpsc::Receiver<Vec<amc_downloader::SystemJava>>>,
    managed_runtimes: Option<Vec<String>>,
    pub update_check_requested: bool,
    pub crash_export_requested: bool,
    pub transfer_export_requested: bool,
    pub transfer_import_requested: bool,
    pub transfer_note: Option<String>,
    pub new_group_name: String,
    pub update_note: Option<String>,
    pub server_start_requested: bool,
    pub server_stop_requested: bool,
    pub server_eula_draft: bool,
    /// Accounts settings empty state asked to open the login modal.
    pub login_request: bool,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            sub_tab: SettingsSubTab::General,
            skin: SettingsSkin::Amc,
            custom_jvm_arg_input: String::new(),
            custom_game_arg_input: String::new(),
            java_scan: None,
            java_scan_rx: None,
            managed_runtimes: None,
            update_check_requested: false,
            crash_export_requested: false,
            transfer_export_requested: false,
            transfer_import_requested: false,
            transfer_note: None,
            new_group_name: String::new(),
            update_note: None,
            server_start_requested: false,
            server_stop_requested: false,
            server_eula_draft: false,
            login_request: false,
        }
    }
}

impl SettingsPage {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
        skin: SettingsSkin,
    ) {
        self.skin = skin;
        if skin == SettingsSkin::Modrinth {
            self.show_modrinth(
                ui,
                config,
                account_mgr,
                paths,
                server_running,
                server_status,
            );
        } else if skin == SettingsSkin::Millida {
            self.show_millida(
                ui,
                config,
                account_mgr,
                paths,
                server_running,
                server_status,
            );
        } else if skin == SettingsSkin::Multimc {
            self.show_multimc(
                ui,
                config,
                account_mgr,
                paths,
                server_running,
                server_status,
            );
        } else if skin == SettingsSkin::Labymod {
            self.show_labymod(
                ui,
                config,
                account_mgr,
                paths,
                server_running,
                server_status,
            );
        } else {
            self.show_amc(
                ui,
                config,
                account_mgr,
                paths,
                server_running,
                server_status,
            );
        }
    }

    fn show_amc(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        ui.add_space(20.0);

        ui.label(
            egui::RichText::new(lang.settings_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Subtabs
        ui.horizontal(|ui| {
            self.subtab_btn(ui, lang.settings_tab_general(), SettingsSubTab::General);
            self.subtab_btn(ui, lang.settings_tab_java(), SettingsSubTab::Java);
            self.subtab_btn(ui, lang.settings_tab_accounts(), SettingsSubTab::Accounts);
        });

        ui.add_space(16.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.sub_tab {
                SettingsSubTab::General => {
                    self.show_general(ui, config, paths, server_running, server_status)
                }
                SettingsSubTab::Java => self.show_java(ui, config, paths),
                SettingsSubTab::Accounts => self.show_accounts(ui, account_mgr, config, lang),
            });
    }

    /// LabyMod settings: hero header, blue section pills and the shared
    /// group-box forms on the dark palette.
    fn show_labymod(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        ui.add_space(12.0);
        labymod_settings_header(ui, lang);
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            self.subtab_btn(ui, lang.settings_tab_general(), SettingsSubTab::General);
            self.subtab_btn(ui, lang.settings_tab_java(), SettingsSubTab::Java);
            self.subtab_btn(ui, lang.settings_tab_accounts(), SettingsSubTab::Accounts);
        });

        ui.add_space(16.0);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.sub_tab {
                SettingsSubTab::General => {
                    self.show_general(ui, config, paths, server_running, server_status)
                }
                SettingsSubTab::Java => self.show_java(ui, config, paths),
                SettingsSubTab::Accounts => self.show_accounts(ui, account_mgr, config, lang),
            });
    }

    /// Millida settings: display heading, cut-corner section tabs and the
    /// existing form controls squared off to the pixel language.
    fn show_millida(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            let (mark, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            pixel::plate(
                ui.painter(),
                mark,
                3.0,
                theme::ACCENT.into(),
                theme::ACCENT_LIGHT.into(),
            );
            ui.label(
                egui::RichText::new(lang.settings_title())
                    .font(theme::display(26.0))
                    .color(TEXT_HEADING),
            );
        });
        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (tab, label) in [
                (SettingsSubTab::General, lang.settings_tab_general()),
                (SettingsSubTab::Java, lang.settings_tab_java()),
                (SettingsSubTab::Accounts, lang.settings_tab_accounts()),
            ] {
                let active = self.sub_tab == tab;
                let galley = ui.painter().layout_no_wrap(
                    label.to_owned(),
                    egui::FontId::new(11.5, egui::FontFamily::Name("bold".into())),
                    Color32::PLACEHOLDER,
                );
                let width = (galley.size().x + 34.0).max(132.0);
                let (rect, resp) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                let fill = if active {
                    theme::ACCENT.into()
                } else {
                    theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover)
                };
                let outline = if active {
                    theme::ACCENT_LIGHT.into()
                } else {
                    theme::BORDER_STRONG.into()
                };
                pixel::plate(ui.painter(), rect, 5.0, fill, outline);
                ui.painter().galley(
                    pos2(
                        rect.center().x - galley.size().x / 2.0,
                        rect.center().y - galley.size().y / 2.0,
                    ),
                    galley,
                    if active {
                        theme::BG.into()
                    } else {
                        theme::TEXT.into()
                    },
                );
                if resp.clicked() {
                    self.sub_tab = tab;
                }
            }
        });

        ui.add_space(12.0);
        ScrollArea::vertical()
            .id_salt(("millida_settings_scroll", self.sub_tab as u8))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.scope(|ui| {
                    apply_millida_settings_style(ui);
                    match self.sub_tab {
                        SettingsSubTab::General => {
                            self.show_general(ui, config, paths, server_running, server_status)
                        }
                        SettingsSubTab::Java => self.show_java(ui, config, paths),
                        SettingsSubTab::Accounts => {
                            self.show_accounts(ui, account_mgr, config, lang)
                        }
                    }
                });
                ui.add_space(18.0);
            });
    }

    /// MultiMC/Prism settings: a Qt-dialog-like page heading, flat push
    /// tabs and the existing group-box form.
    fn show_multimc(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(lang.settings_title())
                .font(egui::FontId::proportional(16.0))
                .strong()
                .color(TEXT_HEADING),
        );
        let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter()
            .rect_filled(line, egui::Rounding::ZERO, BORDER_DEFAULT);
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (tab, label) in [
                (SettingsSubTab::General, lang.modrinth_settings_general()),
                (SettingsSubTab::Java, lang.modrinth_settings_java()),
                (SettingsSubTab::Accounts, lang.modrinth_settings_accounts()),
            ] {
                let width = (ui
                    .painter()
                    .layout_no_wrap(
                        label.to_owned(),
                        egui::FontId::proportional(12.0),
                        Color32::PLACEHOLDER,
                    )
                    .size()
                    .x
                    + 32.0)
                    .max(118.0);
                if multimc_settings_tab(ui, label, self.sub_tab == tab, width) {
                    self.sub_tab = tab;
                }
            }
        });

        ui.add_space(12.0);
        ScrollArea::vertical()
            .id_salt(("multimc_settings_scroll", self.sub_tab as u8))
            .auto_shrink([false, false])
            .show(ui, |ui| match self.sub_tab {
                SettingsSubTab::General => {
                    self.show_general(ui, config, paths, server_running, server_status)
                }
                SettingsSubTab::Java => self.show_java(ui, config, paths),
                SettingsSubTab::Accounts => self.show_accounts(ui, account_mgr, config, lang),
            });
    }

    /// Modrinth App settings composition: a vertical section rail beside the
    /// scrolling form, on the same gray/green surface as the other pages.
    fn show_modrinth(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(lang.settings_title())
                .font(egui::FontId::proportional(24.0))
                .strong()
                .color(MODRINTH_TEXT_CONTRAST),
        );
        ui.add_space(14.0);

        let full_w = ui.available_width();
        let nav_w = (full_w * 0.30).clamp(148.0, 184.0);
        let content_w = (full_w - nav_w - 29.0).max(220.0);
        let body_h = ui.available_height().max(260.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.allocate_ui_with_layout(
                vec2(nav_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 4.0);
                    self.modrinth_nav_item(
                        ui,
                        lang.modrinth_settings_general(),
                        Icon::Gear,
                        SettingsSubTab::General,
                    );
                    self.modrinth_nav_item(
                        ui,
                        lang.modrinth_settings_java(),
                        Icon::Bolt,
                        SettingsSubTab::Java,
                    );
                    self.modrinth_nav_item(
                        ui,
                        lang.modrinth_settings_accounts(),
                        Icon::Person,
                        SettingsSubTab::Accounts,
                    );
                },
            );
            ui.add_space(12.0);
            let (rule, _) = ui.allocate_exact_size(vec2(1.0, body_h), Sense::hover());
            ui.painter()
                .rect_filled(rule, Rounding::ZERO, MODRINTH_SURFACE_HIGH);
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(content_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt(("modrinth_settings_scroll", self.sub_tab as u8))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.scope(|ui| {
                                apply_modrinth_settings_style(ui);
                                match self.sub_tab {
                                    SettingsSubTab::General => self.show_general(
                                        ui,
                                        config,
                                        paths,
                                        server_running,
                                        server_status,
                                    ),
                                    SettingsSubTab::Java => self.show_java(ui, config, paths),
                                    SettingsSubTab::Accounts => {
                                        self.show_accounts(ui, account_mgr, config, lang)
                                    }
                                }
                            });
                            ui.add_space(18.0);
                        });
                },
            );
        });
    }

    fn modrinth_nav_item(&mut self, ui: &mut Ui, label: &str, icon: Icon, tab: SettingsSubTab) {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
        let active = self.sub_tab == tab;
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active || hover > 0.01 {
            let fill = if active {
                with_alpha(MODRINTH_GREEN, 28)
            } else {
                with_alpha(MODRINTH_SURFACE_HOVER, (hover * 210.0) as u8)
            };
            ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
        }
        let glyph = if active {
            MODRINTH_GREEN
        } else {
            lerp_color(MODRINTH_TEXT_SECONDARY, MODRINTH_TEXT_CONTRAST, hover)
        };
        crate::widgets::icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(rect.left() + 18.0, rect.center().y), vec2(16.0, 16.0)),
            glyph,
            1.4,
        );
        ui.painter().text(
            pos2(rect.left() + 34.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::new(
                13.0,
                if active {
                    FontFamily::Name("bold".into())
                } else {
                    FontFamily::Proportional
                },
            ),
            glyph,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.sub_tab = tab;
        }
    }

    fn subtab_btn(&mut self, ui: &mut Ui, label: &str, tab: SettingsSubTab) {
        let is_active = self.sub_tab == tab;
        let (rect, resp) = ui.allocate_exact_size(vec2(130.0, 32.0), Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let act_t = ui.ctx().animate_bool(resp.id.with("act"), is_active);

        let bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, hover_t), RUBY, act_t);
        let stroke_col = lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, act_t);
        let text_col = lerp_color(
            lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
            Color32::WHITE,
            act_t,
        );

        ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
        ui.painter().rect_stroke(
            rect,
            crate::theme::rounding(),
            Stroke::new(1.0_f32, stroke_col),
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.5),
            text_col,
        );

        if resp.clicked() {
            self.sub_tab = tab;
        }
    }
    fn show_general(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        paths: &LauncherPaths,
        server_running: bool,
        server_status: &Option<String>,
    ) {
        let lang = config.ui.language();
        let mr = self.skin == SettingsSkin::Modrinth;

        // Language selection group
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_language_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            if mr {
                ui.horizontal_wrapped(|ui| {
                    for l in Language::ALL {
                        if choice_chip(ui, l.display_with_flag(), 112.0, 32.0, 11.5, lang == l) {
                            config.ui.set_language(l);
                        }
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    for l in Language::ALL {
                        if choice_chip(ui, l.display_with_flag(), 130.0, 34.0, 12.0, lang == l) {
                            config.ui.set_language(l);
                        }
                    }
                });
            }
        });

        ui.add_space(14.0);

        // Operation mode group (CONCEPT modes, ROADMAP P1).
        // Switchable here at any time; the first-run wizard will ask it once.
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_mode_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            for mode in AppMode::ALL {
                let (name, desc) = mode_labels(lang, mode);
                if mr {
                    ui.vertical(|ui| {
                        let chip_w = ui.available_width().min(230.0);
                        if choice_chip(ui, name, chip_w, 36.0, 12.5, config.app_mode == mode) {
                            config.app_mode = mode;
                        }
                        ui.add_space(2.0);
                        ui.add(egui::Label::new(
                            egui::RichText::new(desc)
                                .font(egui::FontId::proportional(12.0))
                                .color(TEXT_MUTED),
                        ));
                    });
                } else {
                    ui.horizontal(|ui| {
                        if choice_chip(ui, name, 170.0, 36.0, 12.5, config.app_mode == mode) {
                            config.app_mode = mode;
                        }
                        if self.skin == SettingsSkin::Labymod {
                            ui.label(
                                egui::RichText::new(desc)
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            );
                        } else {
                            ui.label(egui::RichText::new(desc).color(TEXT_MUTED));
                        }
                    });
                }
                ui.add_space(6.0);
            }
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_behavior_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(lang.settings_after_launch_title())
                    .font(egui::FontId::proportional(13.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(4.0);
            let after_launch_row = |ui: &mut Ui| {
                for (mode, label) in [
                    (
                        amc_core::config::AfterLaunch::Close,
                        lang.settings_after_close(),
                    ),
                    (
                        amc_core::config::AfterLaunch::Minimize,
                        lang.settings_after_minimize(),
                    ),
                    (
                        amc_core::config::AfterLaunch::Keep,
                        lang.settings_after_keep(),
                    ),
                ] {
                    if crate::widgets::choice_chip(
                        ui,
                        label,
                        if mr { 110.0 } else { 130.0 },
                        32.0,
                        12.0,
                        config.ui.after_launch == mode,
                    ) {
                        config.ui.after_launch = mode;
                    }
                }
            };
            if mr {
                ui.horizontal_wrapped(after_launch_row);
            } else {
                ui.horizontal(after_launch_row);
            }

            ui.add_space(6.0);

            bool_setting(
                self.skin,
                ui,
                &mut config.ui.show_snapshots,
                lang.settings_show_snapshots(),
            );

            ui.add_space(6.0);

            bool_setting(
                self.skin,
                ui,
                &mut config.ui.show_old,
                lang.settings_show_old(),
            );

            ui.add_space(6.0);

            let mut autostart_want = config.ui.autostart;
            bool_setting(
                self.skin,
                ui,
                &mut autostart_want,
                lang.settings_autostart(),
            );
            if autostart_want != config.ui.autostart {
                match amc_core::autostart::set_enabled(autostart_want) {
                    Ok(()) => config.ui.autostart = autostart_want,
                    Err(e) => tracing::error!("Autostart change failed: {e}"),
                }
            }
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_speed_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            ui.label(egui::RichText::new(lang.settings_speed_unlimited()).color(TEXT_MUTED));
            let mut kbps = config.download_speed_limit_kbps.unwrap_or(0).min(10240) as u32;
            ui.add(
                egui::Slider::new(&mut kbps, 0..=10240)
                    .step_by(128.0)
                    .text(lang.settings_speed_unit()),
            );
            config.download_speed_limit_kbps = if kbps == 0 { None } else { Some(kbps as u64) };
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_resolution_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.settings_width()).color(TEXT_MUTED));
                ui.add(
                    egui::DragValue::new(&mut config.default_launch_options.window_width).speed(10),
                );
                ui.add_space(20.0);
                ui.label(egui::RichText::new(lang.settings_height()).color(TEXT_MUTED));
                ui.add(
                    egui::DragValue::new(&mut config.default_launch_options.window_height)
                        .speed(10),
                );
            });

            ui.add_space(6.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.default_launch_options.fullscreen,
                lang.settings_fullscreen(),
            );
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_folders_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button(lang.settings_folder_root()).clicked() {
                    let _ = open::that(&paths.root_dir);
                }
                if ui.button(lang.settings_folder_logs()).clicked() {
                    let _ = open::that(paths.logs_dir());
                }
                if ui.button(lang.settings_folder_instances()).clicked() {
                    let _ = open::that(paths.instances_dir());
                }
                if ui.button(lang.settings_root_pick()).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        config.root_override = Some(dir);
                    }
                }
                if config.root_override.is_some()
                    && icon_button(ui, Icon::Close, TEXT_MUTED).clicked()
                {
                    config.root_override = None;
                }
            });
            if config.root_override.is_some() {
                ui.label(egui::RichText::new(lang.settings_root_restart()).color(TEXT_MUTED));
            }
        });

        ui.add_space(14.0);

        // World backups folder + schedule (CONCEPT backups, P9).
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.worlds_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let current = config
                    .world_backup_dir
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| paths.backups_dir().to_string_lossy().to_string());
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&current)
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_PRIMARY),
                    )
                    .truncate(),
                )
                .on_hover_text(&current);
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button(lang.world_pick_folder()).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        config.world_backup_dir = Some(dir);
                    }
                }
                if config.world_backup_dir.is_some()
                    && icon_button(ui, Icon::Close, TEXT_MUTED).clicked()
                {
                    config.world_backup_dir = None;
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.world_auto_label()).color(TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut config.world_backup_days).speed(1));
            });
        });

        ui.add_space(14.0);

        // Notifications per type (CONCEPT notifications).
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.notif_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.notifications.updates,
                lang.notif_updates(),
            );
            ui.add_space(6.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.notifications.downloads,
                lang.notif_downloads(),
            );
            ui.add_space(6.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.notifications.launcher,
                lang.notif_launcher(),
            );
        });

        ui.add_space(14.0);

        // Rebindable hotkeys (CONCEPT hotkeys, P10).
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.hotkey_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.hotkey_launch()).color(TEXT_PRIMARY));
                egui::ComboBox::from_id_salt("hotkey_launch_select")
                    .selected_text(config.launch_hotkey.label())
                    .show_ui(ui, |ui| {
                        for key in amc_core::config::LaunchHotkey::ALL {
                            if ui
                                .selectable_label(config.launch_hotkey == key, key.label())
                                .clicked()
                            {
                                config.launch_hotkey = key;
                            }
                        }
                    });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.hotkey_tabs()).color(TEXT_PRIMARY));
                egui::ComboBox::from_id_salt("hotkey_tabs_select")
                    .selected_text(config.tab_hotkey_mod.label())
                    .show_ui(ui, |ui| {
                        for key in amc_core::config::TabHotkeyMod::ALL {
                            if ui
                                .selectable_label(config.tab_hotkey_mod == key, key.label())
                                .clicked()
                            {
                                config.tab_hotkey_mod = key;
                            }
                        }
                    });
                ui.label(
                    egui::RichText::new(lang.hotkey_tabs_suffix())
                        .font(egui::FontId::proportional(11.0))
                        .color(TEXT_MUTED),
                );
            });
        });

        ui.add_space(14.0);

        // Launcher updates, crash reports, diagnostics export.
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.update_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(lang.update_check()).clicked() {
                    self.update_check_requested = true;
                }
                if let Some(note) = &self.update_note {
                    ui.label(egui::RichText::new(note).color(TEXT_MUTED));
                }
            });
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.crash_title())
                    .font(egui::FontId::proportional(14.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(4.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.crash_reports,
                lang.crash_enable(),
            );
            ui.add_space(6.0);
            if ui.button(lang.crash_export()).clicked() {
                self.crash_export_requested = true;
            }
        });

        ui.add_space(14.0);

        // Manual settings transfer for moving to another PC
        // (CONCEPT cloud sync; no cloud in v1.0).
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.transfer_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(lang.transfer_desc())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(lang.transfer_export()).clicked() {
                    self.transfer_export_requested = true;
                }
                if ui.button(lang.transfer_import()).clicked() {
                    self.transfer_import_requested = true;
                }
                if let Some(note) = &self.transfer_note {
                    ui.label(egui::RichText::new(note).color(TEXT_MUTED));
                }
            });
        });

        ui.add_space(14.0);

        // Local dedicated server for the selected instance (CONCEPT P13).
        // Stops with the launcher (kill_on_drop); no LAN tunneling in v1.0.
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.server_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);
            if server_running {
                if let Some(status) = server_status {
                    ui.label(egui::RichText::new(status).color(TEXT_PRIMARY));
                    ui.add_space(4.0);
                }
                if ui.button(lang.server_stop()).clicked() {
                    self.server_stop_requested = true;
                }
            } else {
                bool_setting(
                    self.skin,
                    ui,
                    &mut self.server_eula_draft,
                    lang.server_eula(),
                );
                ui.add_space(4.0);
                if ui.button(lang.server_create()).clicked() {
                    self.server_start_requested = true;
                }
                if let Some(status) = server_status {
                    ui.label(egui::RichText::new(status).color(TEXT_MUTED));
                }
            }
        });
    }

    fn show_java(&mut self, ui: &mut Ui, config: &mut LauncherConfig, paths: &LauncherPaths) {
        let lang = config.ui.language();
        let mr = self.skin == SettingsSkin::Modrinth;

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_ram_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(
                    lang.settings_min_ram(config.default_launch_options.memory_min_mb),
                )
                .color(TEXT_PRIMARY),
            );
            if self.skin == SettingsSkin::Multimc {
                multimc_ram_slider(
                    ui,
                    &mut config.default_launch_options.memory_min_mb,
                    512,
                    8192,
                    256,
                    "min_ram",
                );
            } else {
                ui.add(
                    egui::Slider::new(&mut config.default_launch_options.memory_min_mb, 512..=8192)
                        .step_by(256.0),
                );
            }

            ui.add_space(10.0);

            ui.label(
                egui::RichText::new(
                    lang.settings_max_ram(config.default_launch_options.memory_max_mb),
                )
                .color(TEXT_PRIMARY),
            );
            if self.skin == SettingsSkin::Multimc {
                multimc_ram_slider(
                    ui,
                    &mut config.default_launch_options.memory_max_mb,
                    1024,
                    32768,
                    512,
                    "max_ram",
                );
            } else {
                ui.add(
                    egui::Slider::new(
                        &mut config.default_launch_options.memory_max_mb,
                        1024..=32768,
                    )
                    .step_by(512.0),
                );
            }
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_java_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let java_display = config
                .default_launch_options
                .java_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| lang.settings_java_auto().to_string());

            ui.label(
                egui::RichText::new(lang.settings_java_path(&java_display))
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_PRIMARY),
            );

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let pick_clicked = if mr {
                    modrinth_ui::button(
                        ui,
                        lang.modrinth_settings_select_java(),
                        Some(Icon::Folder),
                        modrinth_ui::ModrinthButtonKind::Primary,
                        0.0,
                    )
                    .clicked()
                } else {
                    let btn = egui::Button::new(
                        egui::RichText::new(lang.settings_btn_select_java())
                            .font(egui::FontId::proportional(11.0))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(RUBY)
                    .stroke(Stroke::new(1.0_f32, RUBY_LIGHT));
                    ui.add(btn).clicked()
                };

                if pick_clicked {
                    let ext_filter: &[&str] = if cfg!(windows) { &["exe"] } else { &["*"] };
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Java Executable", ext_filter)
                        .pick_file()
                    {
                        config.default_launch_options.java_path = Some(path);
                    }
                }

                if config.default_launch_options.java_path.is_some()
                    && ui.button(lang.settings_btn_reset_java()).clicked()
                {
                    config.default_launch_options.java_path = None;
                }
            });
        });

        ui.add_space(14.0);

        // Shared Java/RAM groups (CONCEPT P2): one edit applies to every
        // member instance at launch. Instances join by name in their card.
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.group_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(lang.group_desc())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
            ui.add_space(8.0);

            if config.instance_groups.is_empty() {
                ui.label(
                    egui::RichText::new(lang.group_empty())
                        .font(egui::FontId::proportional(12.0))
                        .color(TEXT_MUTED),
                );
                ui.add_space(4.0);
            }
            let mut delete_group = None;
            let mut names: Vec<String> = config.instance_groups.keys().cloned().collect();
            names.sort();
            for name in names {
                let Some(group) = config.instance_groups.get_mut(&name) else {
                    continue;
                };
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&name)
                            .font(egui::FontId::proportional(13.0))
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(ui, Icon::Trash, TEXT_MUTED).clicked() {
                            delete_group = Some(name.clone());
                        }
                    });
                });
                let mut has_ram = group.ram_mb.is_some();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut has_ram, lang.group_ram());
                    if has_ram {
                        let ram = group.ram_mb.get_or_insert(4096);
                        ui.add(egui::Slider::new(ram, 1024..=32768).step_by(512.0));
                    }
                });
                if !has_ram {
                    group.ram_mb = None;
                }
                let java_display = group
                    .java_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| lang.group_java_auto().to_string());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(java_display)
                            .font(egui::FontId::proportional(11.0))
                            .color(TEXT_MUTED),
                    );
                    if ui.button(lang.group_java_select()).clicked() {
                        let ext_filter: &[&str] = if cfg!(windows) { &["exe"] } else { &["*"] };
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Java Executable", ext_filter)
                            .pick_file()
                        {
                            group.java_path = Some(path);
                        }
                    }
                    if group.java_path.is_some()
                        && ui.button(lang.settings_btn_reset_java()).clicked()
                    {
                        group.java_path = None;
                    }
                });
                ui.add_space(8.0);
            }
            if let Some(name) = delete_group {
                config.instance_groups.remove(&name);
            }
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.new_group_name)
                        .hint_text(lang.group_name_hint())
                        .desired_width(220.0),
                );
                if ui.button(lang.group_add()).clicked() {
                    let name = self.new_group_name.trim().to_string();
                    if !name.is_empty() && !config.instance_groups.contains_key(&name) {
                        config
                            .instance_groups
                            .insert(name, amc_core::types::InstanceGroup::default());
                        self.new_group_name.clear();
                    }
                }
            });
        });

        ui.add_space(14.0);

        // Java manager: discovered system runtimes + launcher runtimes.
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.settings_java_found_title())
                        .font(egui::FontId::proportional(
                            if self.skin == SettingsSkin::Modrinth {
                                18.0
                            } else {
                                16.0
                            },
                        ))
                        .strong()
                        .color(TEXT_HEADING),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, Icon::Refresh, TEXT_PRIMARY).clicked() {
                        self.java_scan = None;
                        self.java_scan_rx = None;
                        self.managed_runtimes = None;
                    }
                });
            });
            ui.add_space(8.0);

            // System scan runs on a worker thread (probes can take seconds).
            if self.java_scan.is_none() && self.java_scan_rx.is_none() {
                let (tx, rx) = std::sync::mpsc::channel();
                self.java_scan_rx = Some(rx);
                std::thread::spawn(move || {
                    let found = amc_downloader::discover_system_java();
                    let _ = tx.send(found);
                });
            }
            if let Some(rx) = &self.java_scan_rx {
                if let Ok(found) = rx.try_recv() {
                    self.java_scan = Some(found);
                    self.java_scan_rx = None;
                }
            }

            match &self.java_scan {
                None => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(lang.settings_java_scanning()).color(TEXT_MUTED),
                        );
                    });
                }
                Some(found) if found.is_empty() => {
                    ui.label(
                        egui::RichText::new(lang.settings_java_none_found()).color(TEXT_MUTED),
                    );
                }
                Some(found) => {
                    for java in found {
                        ui.horizontal(|ui| {
                            let major = match java.major {
                                Some(m) => format!("Java {m}"),
                                None => "?".to_string(),
                            };
                            ui.label(
                                egui::RichText::new(major)
                                    .font(egui::FontId::proportional(12.0))
                                    .strong()
                                    .color(TEXT_HEADING),
                            );
                            let short = java
                                .path
                                .parent()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| java.path.to_string_lossy().to_string());
                            ui.label(
                                egui::RichText::new(short)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_MUTED),
                            )
                            .on_hover_text(java.path.to_string_lossy());
                            ui.label(
                                egui::RichText::new(&java.source)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(RUBY_LIGHT),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(lang.settings_btn_use_java()).clicked() {
                                        config.default_launch_options.java_path =
                                            Some(java.path.clone());
                                    }
                                },
                            );
                        });
                    }
                }
            }

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(lang.settings_runtimes_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(6.0);

            if self.managed_runtimes.is_none() {
                let mut names = Vec::new();
                if let Ok(entries) = std::fs::read_dir(paths.runtimes_dir()) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            names.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                names.sort();
                self.managed_runtimes = Some(names);
            }
            let mut delete_runtime = None;
            if let Some(names) = &self.managed_runtimes {
                if names.is_empty() {
                    ui.label(
                        egui::RichText::new(lang.settings_java_none_found()).color(TEXT_MUTED),
                    );
                }
                for name in names {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(name)
                                .font(egui::FontId::monospace(12.0))
                                .color(TEXT_PRIMARY),
                        );
                        if icon_button(ui, Icon::Close, TEXT_MUTED).clicked() {
                            delete_runtime = Some(name.clone());
                        }
                    });
                }
            }
            if let Some(name) = delete_runtime {
                let _ = std::fs::remove_dir_all(paths.runtimes_dir().join(&name));
                self.managed_runtimes = None;
            }
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_jvm_args_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let mut to_remove = None;
            for (idx, arg) in config
                .default_launch_options
                .custom_jvm_args
                .iter()
                .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(arg)
                                .font(egui::FontId::monospace(12.0))
                                .color(TEXT_PRIMARY),
                        )
                        .truncate(),
                    )
                    .on_hover_text(arg);
                    if icon_button(ui, Icon::Close, TEXT_MUTED).clicked() {
                        to_remove = Some(idx);
                    }
                });
            }

            if let Some(idx) = to_remove {
                config.default_launch_options.custom_jvm_args.remove(idx);
            }

            ui.add_space(6.0);
            let input_w = if mr {
                (ui.available_width() - 104.0).max(140.0)
            } else {
                300.0
            };
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.custom_jvm_arg_input)
                        .hint_text("-XX:+...")
                        .desired_width(input_w),
                );
                if ui.button(lang.settings_btn_add_arg()).clicked()
                    && !self.custom_jvm_arg_input.trim().is_empty()
                {
                    config
                        .default_launch_options
                        .custom_jvm_args
                        .push(self.custom_jvm_arg_input.trim().to_string());
                    self.custom_jvm_arg_input.clear();
                }
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.settings_gc_presets_title())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(lang.settings_btn_preset_g1gc())
                    .on_hover_text(lang.settings_tooltip_preset_g1gc())
                    .clicked()
                {
                    config.default_launch_options.custom_jvm_args = vec![
                        "-XX:+UseG1GC".to_string(),
                        "-XX:+ParallelRefProcEnabled".to_string(),
                        "-XX:MaxGCPauseMillis=200".to_string(),
                        "-XX:+UnlockExperimentalVMOptions".to_string(),
                        "-XX:+DisableExplicitGC".to_string(),
                        "-XX:+AlwaysPreTouch".to_string(),
                        "-XX:G1NewSizePercent=30".to_string(),
                        "-XX:G1MaxNewSizePercent=40".to_string(),
                        "-XX:G1ReservePercent=20".to_string(),
                        "-XX:G1HeapWastePercent=5".to_string(),
                    ];
                }

                if ui
                    .button(lang.settings_btn_preset_zgc())
                    .on_hover_text(lang.settings_tooltip_preset_zgc())
                    .clicked()
                {
                    config.default_launch_options.custom_jvm_args = vec![
                        "-XX:+UseZGC".to_string(),
                        "-XX:+AlwaysPreTouch".to_string(),
                        "-XX:+UnlockExperimentalVMOptions".to_string(),
                    ];
                }

                if ui
                    .button(lang.settings_btn_reset_args())
                    .on_hover_text(lang.settings_tooltip_reset_args())
                    .clicked()
                {
                    config.default_launch_options.custom_jvm_args.clear();
                }
            });
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_game_args_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let mut game_arg_to_remove = None;
            for (idx, arg) in config
                .default_launch_options
                .custom_game_args
                .iter()
                .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(arg)
                                .font(egui::FontId::monospace(12.0))
                                .color(TEXT_PRIMARY),
                        )
                        .truncate(),
                    )
                    .on_hover_text(arg);
                    if icon_button(ui, Icon::Close, TEXT_MUTED).clicked() {
                        game_arg_to_remove = Some(idx);
                    }
                });
            }

            if let Some(idx) = game_arg_to_remove {
                config.default_launch_options.custom_game_args.remove(idx);
            }

            ui.add_space(6.0);
            let input_w = if mr {
                (ui.available_width() - 104.0).max(140.0)
            } else {
                300.0
            };
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.custom_game_arg_input)
                        .hint_text("--fullscreen")
                        .desired_width(input_w),
                );
                if ui.button(lang.settings_btn_add_arg()).clicked()
                    && !self.custom_game_arg_input.trim().is_empty()
                {
                    config
                        .default_launch_options
                        .custom_game_args
                        .push(self.custom_game_arg_input.trim().to_string());
                    self.custom_game_arg_input.clear();
                }
            });
        });
    }

    fn show_accounts(
        &mut self,
        ui: &mut Ui,
        account_mgr: &mut AccountManager,
        config: &mut LauncherConfig,
        lang: Language,
    ) {
        let mr = self.skin == SettingsSkin::Modrinth;
        ui.group(|ui| {
            if self.skin != SettingsSkin::Amc {
                ui.set_min_width(ui.available_width());
            }
            ui.label(
                egui::RichText::new(lang.settings_accounts_title())
                    .font(egui::FontId::proportional(
                        if self.skin == SettingsSkin::Modrinth {
                            18.0
                        } else {
                            16.0
                        },
                    ))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let accounts = account_mgr.accounts().to_vec();
            let active_id = account_mgr.active_account().map(|a| a.id);
            let has_accounts = !accounts.is_empty();

            if accounts.is_empty() && self.skin != SettingsSkin::Amc {
                ui.add_space(4.0);
                let clicked = if self.skin == SettingsSkin::Labymod {
                    labymod_empty_accounts(ui, lang)
                } else if self.skin == SettingsSkin::Millida {
                    millida_empty_accounts(ui, lang)
                } else {
                    let mut clicked = false;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(lang.settings_no_accounts())
                                .font(egui::FontId::proportional(12.5))
                                .color(TEXT_MUTED),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let button = if mr {
                                modrinth_ui::button(
                                    ui,
                                    lang.modrinth_sign_in(),
                                    Some(Icon::Person),
                                    modrinth_ui::ModrinthButtonKind::Primary,
                                    150.0,
                                )
                            } else {
                                ui.button(lang.modrinth_sign_in())
                            };
                            if button.clicked() {
                                clicked = true;
                            }
                        });
                    });
                    clicked
                };
                if clicked {
                    self.login_request = true;
                }
                ui.add_space(4.0);
                ui.separator();
            }

            let mut set_active_id = None;
            let mut remove_id = None;

            for acc in accounts {
                let is_active = active_id == Some(acc.id);

                if matches!(self.skin, SettingsSkin::Labymod | SettingsSkin::Millida) {
                    let (set_active, remove) =
                        shell_account_card(ui, self.skin, &acc, is_active, lang);
                    if set_active {
                        set_active_id = Some(acc.id);
                    }
                    if remove {
                        remove_id = Some(acc.id);
                    }
                    ui.add_space(6.0);
                    continue;
                }

                ui.horizontal(|ui| {
                    let (badge, accent) = match acc.account_type {
                        amc_auth::AccountType::Microsoft => (lang.account_tag_ms(), true),
                        amc_auth::AccountType::WetId => (lang.account_tag_wetid(), true),
                        amc_auth::AccountType::Offline => (lang.account_tag_offline(), false),
                    };

                    if mr {
                        modrinth_ui::pill(ui, badge, accent);
                    } else {
                        ui.label(
                            egui::RichText::new(badge.to_uppercase())
                                .font(egui::FontId::proportional(10.0))
                                .strong()
                                .color(RUBY_LIGHT),
                        );
                    }

                    let name_limit = if mr { 36 } else { 28 };
                    let name_text = crate::shells::short_name(&acc.username, name_limit);
                    let name_resp = ui.label(
                        egui::RichText::new(name_text)
                            .font(egui::FontId::proportional(14.0))
                            .strong()
                            .color(if is_active {
                                Color32::WHITE
                            } else {
                                TEXT_PRIMARY.into()
                            }),
                    );
                    if acc.username.chars().count() > name_limit {
                        name_resp.on_hover_text(&acc.username);
                    }

                    if is_active {
                        if mr {
                            let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                            ui.painter()
                                .circle_filled(dot.center(), 3.5, MODRINTH_GREEN);
                            ui.label(
                                egui::RichText::new(lang.modrinth_settings_active())
                                    .font(egui::FontId::proportional(11.5))
                                    .color(MODRINTH_GREEN),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(lang.settings_account_active())
                                    .color(Color32::from_rgb(0x50, 0xB0, 0x50)),
                            );
                        }
                    } else if ui.button(lang.settings_btn_set_active()).clicked() {
                        set_active_id = Some(acc.id);
                    }

                    if mr {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if modrinth_ui::icon_button(
                                ui,
                                Icon::Trash,
                                18.0,
                                lang.settings_btn_delete_acc(),
                            )
                            .clicked()
                            {
                                remove_id = Some(acc.id);
                            }
                        });
                    } else if ui
                        .button(
                            egui::RichText::new(lang.settings_btn_delete_acc()).color(TEXT_MUTED),
                        )
                        .clicked()
                    {
                        remove_id = Some(acc.id);
                    }
                });
                ui.separator();
            }

            if let Some(id) = set_active_id {
                let _ = account_mgr.set_active(id);
            }
            if let Some(id) = remove_id {
                let _ = account_mgr.remove_account(id);
            }

            if has_accounts && self.skin != SettingsSkin::Amc {
                ui.add_space(6.0);
                let clicked = if self.skin == SettingsSkin::Labymod {
                    labymod_account_button(
                        ui,
                        lang.settings_add_account(),
                        Some(Icon::Plus),
                        false,
                        170.0,
                        36.0,
                    )
                } else if self.skin == SettingsSkin::Millida {
                    millida_account_button(
                        ui,
                        lang.settings_add_account(),
                        Some(Icon::Plus),
                        false,
                        170.0,
                        32.0,
                    )
                } else if mr {
                    modrinth_ui::button(
                        ui,
                        lang.settings_add_account(),
                        Some(Icon::Plus),
                        modrinth_ui::ModrinthButtonKind::Secondary,
                        170.0,
                    )
                    .clicked()
                } else {
                    ui.button(lang.settings_add_account()).clicked()
                };
                if clicked {
                    self.login_request = true;
                }
            }

            ui.add_space(10.0);
            bool_setting(
                self.skin,
                ui,
                &mut config.ask_identity_each_launch,
                lang.settings_ask_identity(),
            );
            if config.remembered_identity.is_some()
                && ui.button(lang.settings_forget_identity()).clicked()
            {
                config.remembered_identity = None;
            }
        });
    }
}

/// LabyMod account action: blue primary or dark secondary, rounded 10.
fn labymod_account_button(
    ui: &mut Ui,
    label: &str,
    icon: Option<Icon>,
    primary: bool,
    min_width: f32,
    height: f32,
) -> bool {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(13.0),
        Color32::PLACEHOLDER,
    );
    let width = (galley.size().x + if icon.is_some() { 24.0 } else { 0.0 } + 24.0)
        .max(min_width)
        .max(32.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill = if primary {
        lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover)
    } else {
        lerp_color(BG_HOVER, BG_ELEVATED, hover)
    };
    ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
    if !primary {
        ui.painter().rect_stroke(
            rect,
            Rounding::same(10.0),
            Stroke::new(1.0_f32, BORDER_DEFAULT),
        );
    }
    let content = if primary {
        Color32::WHITE
    } else {
        theme::TEXT_HEADING.into()
    };
    let content_w = galley.size().x + if icon.is_some() { 24.0 } else { 0.0 };
    let left = rect.center().x - content_w / 2.0;
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(left + 8.0, rect.center().y), vec2(16.0, 16.0)),
            content,
            1.5,
        );
    }
    ui.painter().galley(
        pos2(
            left + if icon.is_some() { 24.0 } else { 0.0 },
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        content,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Millida account action: cut-corner plate with the mint accent.
fn millida_account_button(
    ui: &mut Ui,
    label: &str,
    icon: Option<Icon>,
    primary: bool,
    min_width: f32,
    height: f32,
) -> bool {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.5),
        Color32::PLACEHOLDER,
    );
    let width = (galley.size().x + if icon.is_some() { 22.0 } else { 0.0 } + 22.0)
        .max(min_width)
        .max(32.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let (fill, outline) = if primary {
        (
            lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
            theme::ACCENT_LIGHT.into(),
        )
    } else {
        (
            lerp_color(BG_HOVER, BG_ACTIVE, hover),
            BORDER_DEFAULT.into(),
        )
    };
    pixel::plate(ui.painter(), rect, 4.0, fill, outline);
    let content = if primary {
        theme::BG.into()
    } else {
        theme::TEXT_HEADING.into()
    };
    let content_w = galley.size().x + if icon.is_some() { 22.0 } else { 0.0 };
    let left = rect.center().x - content_w / 2.0;
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(left + 7.0, rect.center().y), vec2(14.0, 14.0)),
            content,
            1.5,
        );
    }
    ui.painter().galley(
        pos2(
            left + if icon.is_some() { 22.0 } else { 0.0 },
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        content,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Empty accounts card for LabyMod: dark surface, blue avatar, sign-in action.
fn labymod_empty_accounts(ui: &mut Ui, lang: Language) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 92.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(14.0), BG_CARD);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(14.0),
        Stroke::new(1.0_f32, Color32::from(BORDER_DEFAULT)),
    );
    let avatar =
        Rect::from_center_size(pos2(rect.left() + 44.0, rect.center().y), vec2(48.0, 48.0));
    ui.painter()
        .circle_filled(avatar.center(), 24.0, Color32::from(theme::ACCENT_SOFT));
    icons::draw_centered(
        ui.painter(),
        Icon::Person,
        avatar.center(),
        22.0,
        theme::ACCENT_LIGHT,
    );
    ui.painter().text(
        pos2(rect.left() + 80.0, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        lang.settings_no_accounts(),
        egui::FontId::proportional(14.0),
        theme::TEXT_HEADING.into(),
    );
    ui.painter().text(
        pos2(rect.left() + 80.0, rect.center().y + 12.0),
        Align2::LEFT_CENTER,
        lang.settings_accounts_empty_hint(),
        egui::FontId::proportional(11.5),
        TEXT_MUTED.into(),
    );
    let button = Rect::from_min_size(
        pos2(rect.right() - 148.0, rect.center().y - 19.0),
        vec2(132.0, 38.0),
    );
    let resp = ui.interact(
        button,
        ui.id().with("labymod_accounts_empty_login"),
        Sense::click(),
    );
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    ui.painter().rect_filled(
        button,
        Rounding::same(10.0),
        lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
    );
    icons::draw(
        ui.painter(),
        Icon::Person,
        Rect::from_center_size(
            pos2(button.left() + 30.0, button.center().y),
            vec2(16.0, 16.0),
        ),
        Color32::WHITE,
        1.5,
    );
    ui.painter().text(
        pos2(button.left() + 46.0, button.center().y),
        Align2::LEFT_CENTER,
        lang.modrinth_sign_in(),
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Empty accounts card for Millida: cut-corner plate, mint avatar and action.
fn millida_empty_accounts(ui: &mut Ui, lang: Language) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 92.0), Sense::hover());
    pixel::plate(
        ui.painter(),
        rect,
        6.0,
        BG_CARD.into(),
        BORDER_DEFAULT.into(),
    );
    let avatar =
        Rect::from_center_size(pos2(rect.left() + 44.0, rect.center().y), vec2(46.0, 46.0));
    pixel::plate(
        ui.painter(),
        avatar,
        5.0,
        theme::ACCENT_SOFT.into(),
        theme::ACCENT.into(),
    );
    icons::draw_centered(
        ui.painter(),
        Icon::Person,
        avatar.center(),
        21.0,
        theme::ACCENT_LIGHT,
    );
    ui.painter().text(
        pos2(rect.left() + 80.0, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        lang.settings_no_accounts(),
        egui::FontId::proportional(13.5),
        theme::TEXT_HEADING.into(),
    );
    ui.painter().text(
        pos2(rect.left() + 80.0, rect.center().y + 12.0),
        Align2::LEFT_CENTER,
        lang.settings_accounts_empty_hint(),
        egui::FontId::proportional(11.0),
        TEXT_MUTED.into(),
    );
    let button = Rect::from_min_size(
        pos2(rect.right() - 146.0, rect.center().y - 17.0),
        vec2(130.0, 34.0),
    );
    let resp = ui.interact(
        button,
        ui.id().with("millida_accounts_empty_login"),
        Sense::click(),
    );
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    pixel::plate(
        ui.painter(),
        button,
        4.0,
        lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
        Color32::from(theme::ACCENT_LIGHT),
    );
    icons::draw(
        ui.painter(),
        Icon::Person,
        Rect::from_center_size(
            pos2(button.left() + 28.0, button.center().y),
            vec2(14.0, 14.0),
        ),
        Color32::from(theme::BG),
        1.5,
    );
    ui.painter().text(
        pos2(button.left() + 42.0, button.center().y),
        Align2::LEFT_CENTER,
        lang.modrinth_sign_in(),
        egui::FontId::proportional(12.5),
        theme::BG.into(),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// One account row for LabyMod/Millida: shell-styled card with avatar, badge,
/// active marker and shell-styled actions. Returns (set active, remove).
fn shell_account_card(
    ui: &mut Ui,
    skin: SettingsSkin,
    acc: &amc_auth::Account,
    is_active: bool,
    lang: Language,
) -> (bool, bool) {
    let labymod = skin == SettingsSkin::Labymod;
    let (rect, card_resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 66.0), Sense::hover());
    if labymod {
        ui.painter()
            .rect_filled(rect, Rounding::same(14.0), BG_CARD);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(14.0),
            Stroke::new(1.0_f32, Color32::from(BORDER_DEFAULT)),
        );
    } else {
        pixel::plate(
            ui.painter(),
            rect,
            6.0,
            BG_CARD.into(),
            BORDER_DEFAULT.into(),
        );
    }
    let avatar =
        Rect::from_center_size(pos2(rect.left() + 38.0, rect.center().y), vec2(40.0, 40.0));
    if labymod {
        ui.painter()
            .circle_filled(avatar.center(), 20.0, Color32::from(theme::ACCENT_SOFT));
    } else {
        pixel::plate(
            ui.painter(),
            avatar,
            5.0,
            theme::ACCENT_SOFT.into(),
            theme::ACCENT.into(),
        );
    }
    icons::draw_centered(
        ui.painter(),
        Icon::Person,
        avatar.center(),
        19.0,
        theme::ACCENT_LIGHT,
    );

    let badge = match acc.account_type {
        amc_auth::AccountType::Microsoft => lang.account_tag_ms(),
        amc_auth::AccountType::WetId => lang.account_tag_wetid(),
        amc_auth::AccountType::Offline => lang.account_tag_offline(),
    };
    ui.painter().text(
        pos2(rect.left() + 68.0, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        crate::shells::short_name(&acc.username, 28),
        egui::FontId::new(14.0, egui::FontFamily::Name("bold".into())),
        theme::TEXT_HEADING.into(),
    );
    ui.painter().text(
        pos2(rect.left() + 68.0, rect.center().y + 12.0),
        Align2::LEFT_CENTER,
        badge,
        egui::FontId::proportional(10.5),
        Color32::from(theme::ACCENT_LIGHT),
    );

    let mut set_active = false;
    let mut remove = false;
    let inner = rect.shrink2(vec2(12.0, 8.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            let delete = icons::icon_button(ui, Icon::Trash, TEXT_MUTED);
            if delete
                .on_hover_text(lang.settings_btn_delete_acc())
                .clicked()
            {
                remove = true;
            }
            ui.add_space(4.0);
            if is_active {
                let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), 3.5, Color32::from(theme::ACCENT_LIGHT));
                ui.label(
                    egui::RichText::new(lang.modrinth_settings_active())
                        .font(egui::FontId::proportional(11.5))
                        .color(theme::ACCENT_LIGHT),
                );
            } else if labymod {
                if labymod_account_button(
                    ui,
                    lang.settings_btn_set_active(),
                    None,
                    false,
                    150.0,
                    32.0,
                ) {
                    set_active = true;
                }
            } else if millida_account_button(
                ui,
                lang.settings_btn_set_active(),
                None,
                false,
                150.0,
                30.0,
            ) {
                set_active = true;
            }
        },
    );
    card_resp.on_hover_text(&acc.username);
    (set_active, remove)
}

/// Qt/Fusion-style RAM slider for the MultiMC settings page: flat groove,
/// rectangular handle and a spinbox-like value plate. Other shells keep the
/// native egui slider so their visuals stay unchanged.
fn multimc_ram_slider(
    ui: &mut Ui,
    value: &mut u32,
    min: u32,
    max: u32,
    step: u32,
    salt: &str,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
    let plate_w = 86.0;
    let groove_w = (width - plate_w - 14.0).max(80.0);
    let groove = Rect::from_center_size(
        pos2(rect.left() + groove_w / 2.0, rect.center().y),
        vec2(groove_w, 4.0),
    );
    let resp = ui.interact(
        Rect::from_min_max(
            pos2(rect.left(), rect.top()),
            pos2(groove.right() + 8.0, rect.bottom()),
        ),
        ui.id().with(("multimc_ram_slider", salt)),
        Sense::click_and_drag(),
    );
    if resp.dragged() || resp.clicked() {
        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
            let t = ((pointer.x - groove.left()) / groove.width()).clamp(0.0, 1.0);
            let raw = min as f32 + t * (max - min) as f32;
            let stepped = ((raw / step as f32).round() as u32) * step;
            *value = stepped.clamp(min, max);
        }
    }
    let value = (*value).clamp(min, max);
    let t = (value - min) as f32 / (max - min) as f32;
    ui.painter()
        .rect_filled(groove, Rounding::ZERO, theme::BORDER_SOFT);
    ui.painter().rect_filled(
        Rect::from_min_size(groove.min, vec2(groove.width() * t, groove.height())),
        Rounding::ZERO,
        theme::ACCENT,
    );
    let handle = Rect::from_center_size(
        pos2(groove.left() + groove.width() * t, rect.center().y),
        vec2(10.0, 18.0),
    );
    ui.painter()
        .rect_filled(handle, Rounding::ZERO, theme::ACCENT);
    ui.painter().rect_stroke(
        handle,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );
    let plate = Rect::from_min_size(
        pos2(rect.right() - plate_w, rect.center().y - 12.0),
        vec2(plate_w, 24.0),
    );
    ui.painter()
        .rect_filled(plate, Rounding::ZERO, theme::FIELD);
    ui.painter().rect_stroke(
        plate,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::BORDER_DEFAULT),
    );
    ui.painter().text(
        plate.center(),
        Align2::CENTER_CENTER,
        format!("{value} MB"),
        egui::FontId::proportional(11.0),
        theme::TEXT.into(),
    );
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    resp
}

/// One boolean setting: the reference switch on Modrinth, the original
/// checkbox elsewhere (so Aleph layouts stay untouched).
fn bool_setting(skin: SettingsSkin, ui: &mut Ui, value: &mut bool, label: &str) -> egui::Response {
    if skin == SettingsSkin::Modrinth {
        modrinth_ui::toggle(ui, value, label)
    } else {
        ui.checkbox(value, egui::RichText::new(label).color(TEXT_PRIMARY))
    }
}

/// Qt-style flat push button used by the MultiMC settings page.
fn multimc_settings_tab(ui: &mut Ui, label: &str, active: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill = if active {
        Color32::from(theme::ACCENT)
    } else {
        lerp_color(BG_ELEVATED, BG_HOVER, hover)
    };
    let stroke = if active {
        Color32::from(theme::ACCENT)
    } else {
        lerp_color(BORDER_DEFAULT, theme::ACCENT, hover)
    };
    ui.painter()
        .rect_filled(rect, crate::theme::rounding(), fill);
    ui.painter()
        .rect_stroke(rect, crate::theme::rounding(), Stroke::new(1.0_f32, stroke));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if active {
            Color32::WHITE
        } else {
            Color32::from(theme::TEXT)
        },
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Scoped square controls for the Millida settings form.
fn apply_millida_settings_style(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    ui.spacing_mut().button_padding = vec2(12.0, 7.0);
    ui.spacing_mut().slider_width = 190.0;
    let visuals = ui.visuals_mut();
    visuals.widgets.noninteractive.bg_fill = Color32::from(theme::BG_CARD);
    visuals.widgets.noninteractive.weak_bg_fill = Color32::from(theme::BG_CARD);
    visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0_f32, Color32::from(theme::BORDER_STRONG));
    visuals.widgets.noninteractive.rounding = Rounding::ZERO;
    visuals.widgets.noninteractive.fg_stroke.color = Color32::from(theme::TEXT);

    visuals.widgets.inactive.bg_fill = Color32::from(theme::BG_HOVER);
    visuals.widgets.inactive.weak_bg_fill = Color32::from(theme::BG_HOVER);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from(theme::BORDER_STRONG));
    visuals.widgets.inactive.rounding = Rounding::ZERO;
    visuals.widgets.inactive.fg_stroke.color = Color32::from(theme::TEXT);

    visuals.widgets.hovered.bg_fill = Color32::from(theme::ACCENT_DIM);
    visuals.widgets.hovered.weak_bg_fill = Color32::from(theme::ACCENT_DIM);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from(theme::ACCENT));
    visuals.widgets.hovered.rounding = Rounding::ZERO;
    visuals.widgets.hovered.fg_stroke.color = Color32::from(theme::HEAD);

    visuals.widgets.active.bg_fill = Color32::from(theme::ACCENT);
    visuals.widgets.active.weak_bg_fill = Color32::from(theme::ACCENT);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from(theme::ACCENT_LIGHT));
    visuals.widgets.active.rounding = Rounding::ZERO;
    visuals.widgets.active.fg_stroke.color = Color32::from(theme::BG);

    visuals.selection.bg_fill = with_alpha(theme::ACCENT, 100);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from(theme::ACCENT_LIGHT));
    visuals.hyperlink_color = Color32::from(theme::ACCENT_LIGHT);
    visuals.extreme_bg_color = Color32::from(theme::BG);
    visuals.faint_bg_color = Color32::from(theme::BG_CARD);
    visuals.window_fill = Color32::from(theme::BG_CARD);
    visuals.panel_fill = Color32::from(theme::BG);
}

fn labymod_settings_header(ui: &mut Ui, lang: Language) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(16.0), BG_CARD);
    painter.rect_stroke(
        rect,
        Rounding::same(16.0),
        Stroke::new(1.0_f32, BORDER_DEFAULT),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 22.0),
        egui::Align2::LEFT_CENTER,
        lang.nav_settings().to_uppercase(),
        FontId::proportional(10.5),
        theme::ACCENT_LIGHT.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 47.0),
        egui::Align2::LEFT_CENTER,
        lang.settings_title(),
        FontId::new(27.0, FontFamily::Name("bold".into())),
        TEXT_HEADING.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 74.0),
        egui::Align2::LEFT_CENTER,
        lang.labymod_settings_subtitle(),
        FontId::proportional(11.5),
        TEXT_MUTED.into(),
    );
}

/// Scoped visuals for the Modrinth settings form. Existing controls keep
/// their behavior; groups become raised cards and buttons/inputs use the
/// reference's gray surfaces with the green focus accent.
fn apply_modrinth_settings_style(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    ui.spacing_mut().button_padding = vec2(12.0, 7.0);
    ui.spacing_mut().slider_width = 190.0;
    let visuals = ui.visuals_mut();
    visuals.widgets.noninteractive.bg_fill = MODRINTH_SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = MODRINTH_SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
    visuals.widgets.noninteractive.rounding = Rounding::same(12.0);

    visuals.widgets.inactive.bg_fill = MODRINTH_SURFACE_HOVER;
    visuals.widgets.inactive.weak_bg_fill = MODRINTH_SURFACE_HOVER;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    visuals.widgets.hovered.bg_fill = with_alpha(MODRINTH_GREEN, 26);
    visuals.widgets.hovered.weak_bg_fill = with_alpha(MODRINTH_GREEN, 26);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, with_alpha(MODRINTH_GREEN, 110));
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    visuals.widgets.active.bg_fill = with_alpha(MODRINTH_GREEN, 42);
    visuals.widgets.active.weak_bg_fill = with_alpha(MODRINTH_GREEN, 42);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, MODRINTH_GREEN);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    visuals.selection.bg_fill = with_alpha(MODRINTH_GREEN, 70);
    visuals.selection.stroke = Stroke::new(1.0_f32, MODRINTH_GREEN);
    visuals.hyperlink_color = MODRINTH_GREEN;
    visuals.extreme_bg_color = MODRINTH_BG;
    visuals.faint_bg_color = MODRINTH_SURFACE;
    visuals.window_fill = MODRINTH_SURFACE;
    visuals.panel_fill = MODRINTH_BG;
}
