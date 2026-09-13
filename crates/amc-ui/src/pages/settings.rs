use egui::{vec2, Color32, Rounding, ScrollArea, Stroke, TextEdit, Ui};
use amc_auth::AccountManager;
use amc_core::config::LauncherConfig;
use amc_core::paths::LauncherPaths;
use amc_core::Language;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSubTab {
    #[default]
    General,
    Java,
    Accounts,
}

pub struct SettingsPage {
    pub sub_tab: SettingsSubTab,
    pub custom_jvm_arg_input: String,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            sub_tab: SettingsSubTab::General,
            custom_jvm_arg_input: String::new(),
        }
    }
}

impl SettingsPage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        config: &mut LauncherConfig,
        account_mgr: &mut AccountManager,
        paths: &LauncherPaths,
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
                SettingsSubTab::General => self.show_general(ui, config, paths),
                SettingsSubTab::Java => self.show_java(ui, config),
                SettingsSubTab::Accounts => self.show_accounts(ui, account_mgr, lang),
            });
    }

    fn subtab_btn(&mut self, ui: &mut Ui, label: &str, tab: SettingsSubTab) {
        let is_active = self.sub_tab == tab;
        let (bg, stroke, text_color) = if is_active {
            (RUBY, Stroke::new(1.0, RUBY_LIGHT), Color32::WHITE)
        } else {
            (BG_ELEVATED, Stroke::new(1.0, BORDER_DEFAULT), TEXT_MUTED)
        };

        let btn = egui::Button::new(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(11.0))
                .strong()
                .color(text_color),
        )
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::ZERO)
        .min_size(vec2(130.0, 32.0));

        if ui.add(btn).clicked() {
            self.sub_tab = tab;
        }
    }

    fn show_general(&mut self, ui: &mut Ui, config: &mut LauncherConfig, paths: &LauncherPaths) {
        let lang = config.ui.language();

        // Language selection group
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_language_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                for l in Language::ALL {
                    let is_active = lang == l;
                    let (bg, stroke, text_color) = if is_active {
                        (RUBY, Stroke::new(1.0, RUBY_LIGHT), Color32::WHITE)
                    } else {
                        (BG_HOVER, Stroke::new(1.0, BORDER_DEFAULT), TEXT_MUTED)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(l.display_with_flag())
                            .font(egui::FontId::proportional(12.0))
                            .strong()
                            .color(text_color),
                    )
                    .fill(bg)
                    .stroke(stroke)
                    .rounding(Rounding::ZERO)
                    .min_size(vec2(130.0, 32.0));

                    if ui.add(btn).clicked() {
                        config.ui.set_language(l);
                    }
                }
            });
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_behavior_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.checkbox(
                &mut config.ui.close_after_launch,
                egui::RichText::new(lang.settings_close_after_launch()).color(TEXT_PRIMARY),
            );

            ui.add_space(6.0);

            ui.checkbox(
                &mut config.ui.show_snapshots,
                egui::RichText::new(lang.settings_show_snapshots()).color(TEXT_PRIMARY),
            );

            ui.add_space(6.0);

            ui.checkbox(
                &mut config.ui.show_old,
                egui::RichText::new(lang.settings_show_old()).color(TEXT_PRIMARY),
            );
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_resolution_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.settings_width()).color(TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut config.default_launch_options.window_width).speed(10));
                ui.add_space(20.0);
                ui.label(egui::RichText::new(lang.settings_height()).color(TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut config.default_launch_options.window_height).speed(10));
            });

            ui.add_space(6.0);
            ui.checkbox(
                &mut config.default_launch_options.fullscreen,
                egui::RichText::new(lang.settings_fullscreen()).color(TEXT_PRIMARY),
            );
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_folders_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button(lang.settings_folder_root()).clicked() {
                    let _ = open::that(&paths.root_dir);
                }
                if ui.button(lang.settings_folder_logs()).clicked() {
                    let _ = open::that(paths.logs_dir());
                }
                if ui.button(lang.settings_folder_instances()).clicked() {
                    let _ = open::that(paths.instances_dir());
                }
            });
        });
    }

    fn show_java(&mut self, ui: &mut Ui, config: &mut LauncherConfig) {
        let lang = config.ui.language();

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_ram_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(lang.settings_min_ram(config.default_launch_options.memory_min_mb))
                    .color(TEXT_PRIMARY),
            );
            ui.add(
                egui::Slider::new(
                    &mut config.default_launch_options.memory_min_mb,
                    512..=8192,
                )
                .step_by(256.0),
            );

            ui.add_space(10.0);

            ui.label(
                egui::RichText::new(lang.settings_max_ram(config.default_launch_options.memory_max_mb))
                    .color(TEXT_PRIMARY),
            );
            ui.add(
                egui::Slider::new(
                    &mut config.default_launch_options.memory_max_mb,
                    1024..=32768,
                )
                .step_by(512.0),
            );
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_java_title())
                    .font(egui::FontId::proportional(16.0))
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
                let btn = egui::Button::new(
                    egui::RichText::new(lang.settings_btn_select_java())
                        .font(egui::FontId::proportional(11.0))
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(RUBY)
                .stroke(Stroke::new(1.0, RUBY_LIGHT));

                if ui.add(btn).clicked() {
                    let ext_filter: &[&str] = if cfg!(windows) { &["exe"] } else { &["*"] };
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Java Executable", ext_filter)
                        .pick_file()
                    {
                        config.default_launch_options.java_path = Some(path);
                    }
                }

                if config.default_launch_options.java_path.is_some() {
                    if ui.button(lang.settings_btn_reset_java()).clicked() {
                        config.default_launch_options.java_path = None;
                    }
                }
            });
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_jvm_args_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let mut to_remove = None;
            for (idx, arg) in config.default_launch_options.custom_jvm_args.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(arg).font(egui::FontId::monospace(12.0)).color(TEXT_PRIMARY));
                    if ui.button(egui::RichText::new("✕").color(TEXT_MUTED)).clicked() {
                        to_remove = Some(idx);
                    }
                });
            }

            if let Some(idx) = to_remove {
                config.default_launch_options.custom_jvm_args.remove(idx);
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.custom_jvm_arg_input)
                        .hint_text("-XX:+...")
                        .desired_width(300.0),
                );
                if ui.button(lang.settings_btn_add_arg()).clicked() && !self.custom_jvm_arg_input.trim().is_empty() {
                    config.default_launch_options.custom_jvm_args.push(self.custom_jvm_arg_input.trim().to_string());
                    self.custom_jvm_arg_input.clear();
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new(lang.settings_gc_presets_title()).font(egui::FontId::proportional(12.0)).color(TEXT_MUTED));
            ui.horizontal(|ui| {
                if ui.button(lang.settings_btn_preset_g1gc())
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

                if ui.button(lang.settings_btn_preset_zgc())
                    .on_hover_text(lang.settings_tooltip_preset_zgc())
                    .clicked()
                {
                    config.default_launch_options.custom_jvm_args = vec![
                        "-XX:+UseZGC".to_string(),
                        "-XX:+AlwaysPreTouch".to_string(),
                        "-XX:+UnlockExperimentalVMOptions".to_string(),
                    ];
                }

                if ui.button(lang.settings_btn_reset_args())
                    .on_hover_text(lang.settings_tooltip_reset_args())
                    .clicked()
                {
                    config.default_launch_options.custom_jvm_args.clear();
                }
            });
        });
    }

    fn show_accounts(&mut self, ui: &mut Ui, account_mgr: &mut AccountManager, lang: Language) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.settings_accounts_title())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            let accounts = account_mgr.accounts().to_vec();
            let active_id = account_mgr.active_account().map(|a| a.id);

            let mut set_active_id = None;
            let mut remove_id = None;

            for acc in accounts {
                let is_active = active_id == Some(acc.id);

                ui.horizontal(|ui| {
                    let badge = match acc.account_type {
                        amc_auth::AccountType::Microsoft => "MICROSOFT",
                        amc_auth::AccountType::WetId => "WETID",
                        amc_auth::AccountType::Offline => "OFFLINE",
                    };

                    ui.label(
                        egui::RichText::new(badge)
                            .font(egui::FontId::proportional(10.0))
                            .strong()
                            .color(RUBY_LIGHT),
                    );

                    ui.label(
                        egui::RichText::new(&acc.username)
                            .font(egui::FontId::proportional(14.0))
                            .strong()
                            .color(if is_active { Color32::WHITE } else { TEXT_PRIMARY }),
                    );

                    if is_active {
                        ui.label(egui::RichText::new(lang.settings_account_active()).color(Color32::from_rgb(0x50, 0xB0, 0x50)));
                    } else if ui.button(lang.settings_btn_set_active()).clicked() {
                        set_active_id = Some(acc.id);
                    }

                    if ui.button(egui::RichText::new(lang.settings_btn_delete_acc()).color(TEXT_MUTED)).clicked() {
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
        });
    }
}
