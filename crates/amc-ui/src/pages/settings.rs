use egui::{vec2, Color32, Rounding, ScrollArea, Stroke, TextEdit, Ui};
use amc_auth::AccountManager;
use amc_core::config::LauncherConfig;
use crate::theme::{
    BG_ELEVATED, BORDER_DEFAULT, RUBY, RUBY_LIGHT,
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
    ) {
        ui.add_space(20.0);

        ui.label(
            egui::RichText::new("Настройки")
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Subtabs
        ui.horizontal(|ui| {
            self.subtab_btn(ui, "ОСНОВНЫЕ", SettingsSubTab::General);
            self.subtab_btn(ui, "JAVA И ПАМЯТЬ", SettingsSubTab::Java);
            self.subtab_btn(ui, "АККАУНТЫ", SettingsSubTab::Accounts);
        });

        ui.add_space(16.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.sub_tab {
                SettingsSubTab::General => self.show_general(ui, config),
                SettingsSubTab::Java => self.show_java(ui, config),
                SettingsSubTab::Accounts => self.show_accounts(ui, account_mgr),
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

    fn show_general(&mut self, ui: &mut Ui, config: &mut LauncherConfig) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Поведение лаунчера")
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.checkbox(
                &mut config.ui.close_after_launch,
                egui::RichText::new("Закрывать лаунчер после запуска Minecraft").color(TEXT_PRIMARY),
            );

            ui.add_space(6.0);

            ui.checkbox(
                &mut config.ui.show_snapshots,
                egui::RichText::new("Отображать снапшоты в списке версий").color(TEXT_PRIMARY),
            );

            ui.add_space(6.0);

            ui.checkbox(
                &mut config.ui.show_old,
                egui::RichText::new("Отображать старые версии (Alpha / Beta)").color(TEXT_PRIMARY),
            );
        });

        ui.add_space(14.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Разрешение окна игры")
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Ширина:").color(TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut config.default_launch_options.window_width).speed(10));
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Высота:").color(TEXT_MUTED));
                ui.add(egui::DragValue::new(&mut config.default_launch_options.window_height).speed(10));
            });

            ui.add_space(6.0);
            ui.checkbox(
                &mut config.default_launch_options.fullscreen,
                egui::RichText::new("Запускать в полноэкранном режиме").color(TEXT_PRIMARY),
            );
        });
    }

    fn show_java(&mut self, ui: &mut Ui, config: &mut LauncherConfig) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Оперативная память (RAM)")
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(format!(
                    "Минимальная память: {} МБ ({:.1} ГБ)",
                    config.default_launch_options.memory_min_mb,
                    config.default_launch_options.memory_min_mb as f64 / 1024.0
                ))
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
                egui::RichText::new(format!(
                    "Максимальная память: {} МБ ({:.1} ГБ)",
                    config.default_launch_options.memory_max_mb,
                    config.default_launch_options.memory_max_mb as f64 / 1024.0
                ))
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
                egui::RichText::new("Пользовательские JVM аргументы")
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
                if ui.button("+ Добавить").clicked() && !self.custom_jvm_arg_input.trim().is_empty() {
                    config.default_launch_options.custom_jvm_args.push(self.custom_jvm_arg_input.trim().to_string());
                    self.custom_jvm_arg_input.clear();
                }
            });
        });
    }

    fn show_accounts(&mut self, ui: &mut Ui, account_mgr: &mut AccountManager) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Сохраненные аккаунты")
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
                        ui.label(egui::RichText::new("✔ (Активен)").color(Color32::from_rgb(0x50, 0xB0, 0x50)));
                    } else if ui.button("Сделать активным").clicked() {
                        set_active_id = Some(acc.id);
                    }

                    if ui.button(egui::RichText::new("Удалить").color(TEXT_MUTED)).clicked() {
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
