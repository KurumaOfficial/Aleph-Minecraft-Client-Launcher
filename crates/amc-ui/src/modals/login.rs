use egui::{vec2, Color32, Rounding, Stroke, TextEdit, Ui};
use amc_auth::{login_offline, Account, DeviceCodeResponse};
use amc_core::Language;
use crate::theme::{
    BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoginMode {
    #[default]
    Offline,
    WetId,
    Microsoft,
}

pub struct LoginModal {
    pub is_open: bool,
    pub mode: LoginMode,

    // Offline fields
    pub offline_nickname: String,

    // WetID fields
    pub wetid_login: String,
    pub wetid_pass: String,

    // Microsoft fields
    pub ms_device_code: Option<DeviceCodeResponse>,
    pub ms_polling: bool,

    pub error_msg: Option<String>,
}

impl Default for LoginModal {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: LoginMode::Offline,
            offline_nickname: String::new(),
            wetid_login: String::new(),
            wetid_pass: String::new(),
            ms_device_code: None,
            ms_polling: false,
            error_msg: None,
        }
    }
}

impl LoginModal {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        lang: Language,
        on_success: impl FnOnce(Account),
        on_request_ms_code: impl FnOnce(),
    ) {
        if !self.is_open {
            return;
        }

        let mut success_account = None;
        let mut request_ms = false;

        egui::Window::new(lang.login_modal_title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(440.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Mode Tabs
                ui.horizontal(|ui| {
                    self.tab_btn(ui, lang.login_tab_offline(), LoginMode::Offline);
                    self.tab_btn(ui, lang.login_tab_wetid(), LoginMode::WetId);
                    self.tab_btn(ui, lang.login_tab_microsoft(), LoginMode::Microsoft);
                });

                ui.add_space(16.0);

                if let Some(err) = &self.error_msg {
                    ui.label(egui::RichText::new(format!("⚠ {err}")).color(Color32::from_rgb(0xE8, 0x40, 0x40)));
                    ui.add_space(8.0);
                }

                match self.mode {
                    LoginMode::Offline => {
                        ui.label(egui::RichText::new(lang.login_offline_prompt()).color(TEXT_PRIMARY));
                        let resp = ui.add(
                            TextEdit::singleline(&mut self.offline_nickname)
                                .hint_text("Player")
                                .desired_width(400.0)
                                .font(egui::FontId::proportional(15.0)),
                        );

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(lang.login_offline_hint())
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );

                        ui.add_space(16.0);

                        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let btn = egui::Button::new(
                            egui::RichText::new(lang.login_btn_play())
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(RUBY)
                        .stroke(Stroke::new(1.0, RUBY_LIGHT))
                        .min_size(vec2(ui.available_width(), 40.0));

                        if (ui.add(btn).clicked() || enter) && !self.offline_nickname.trim().is_empty() {
                            match login_offline(self.offline_nickname.trim()) {
                                Ok(acc) => {
                                    success_account = Some(acc);
                                    self.is_open = false;
                                }
                                Err(e) => {
                                    self.error_msg = Some(e.to_string());
                                }
                            }
                        }
                    }
                    LoginMode::WetId => {
                        ui.label(egui::RichText::new(lang.login_wetid_login()).color(TEXT_PRIMARY));
                        ui.add(TextEdit::singleline(&mut self.wetid_login).desired_width(400.0));

                        ui.add_space(10.0);

                        ui.label(egui::RichText::new(lang.login_wetid_pass()).color(TEXT_PRIMARY));
                        ui.add(TextEdit::singleline(&mut self.wetid_pass).password(true).desired_width(400.0));

                        ui.add_space(6.0);
                        let wetid_hint = match lang {
                            Language::English => "Access to AlephTrust servers, cloud skins and stats",
                            Language::Russian => "Доступ к серверам AlephTrust, облачным скинам и статистике",
                            Language::Ukrainian => "Доступ до серверів AlephTrust, хмарних скінів та статистики",
                        };
                        ui.label(
                            egui::RichText::new(wetid_hint)
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );

                        ui.add_space(16.0);

                        let btn = egui::Button::new(
                            egui::RichText::new(lang.login_btn_wetid())
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(RUBY)
                        .stroke(Stroke::new(1.0, RUBY_LIGHT))
                        .min_size(vec2(ui.available_width(), 40.0));

                        if ui.add(btn).clicked() {
                            let dummy_acc = amc_auth::Account::new_wetid(
                                self.wetid_login.clone(),
                                "wetid-uuid".to_string(),
                                "mock-token".to_string(),
                                None,
                            );
                            success_account = Some(dummy_acc);
                            self.is_open = false;
                        }
                    }
                    LoginMode::Microsoft => {
                        if let Some(code_data) = &self.ms_device_code {
                            ui.label(
                                egui::RichText::new(lang.login_ms_instructions(&code_data.verification_uri))
                                    .font(egui::FontId::proportional(14.0))
                                    .color(TEXT_PRIMARY),
                            );

                            ui.add_space(10.0);

                            // User code in large monospace box
                            ui.vertical_centered(|ui| {
                                let (rect, _) = ui.allocate_exact_size(vec2(260.0, 50.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, Rounding::ZERO, BG_HOVER);
                                ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &code_data.user_code,
                                    egui::FontId::monospace(22.0),
                                    Color32::WHITE,
                                );
                            });

                            ui.add_space(14.0);

                            if ui.button(lang.login_btn_open_browser()).clicked() {
                                let _ = open::that(&code_data.verification_uri);
                            }

                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(egui::RichText::new(lang.login_ms_waiting()).color(TEXT_MUTED));
                            });
                        } else {
                            let ms_prompt = match lang {
                                Language::English => "Sign in via official Microsoft OAuth2 Device Code Flow:",
                                Language::Russian => "Вход через официальный OAuth2 Device Code Flow:",
                                Language::Ukrainian => "Вхід через офіційний OAuth2 Device Code Flow:",
                            };
                            let ms_hint = match lang {
                                Language::English => "Required for playing on official licensed servers (Hypixel, etc.)",
                                Language::Russian => "Требуется для игры на официальных лицензионных серверах (Hypixel и др.)",
                                Language::Ukrainian => "Потрібно для гри на офіційних ліцензійних серверах (Hypixel тощо)",
                            };
                            ui.label(
                                egui::RichText::new(ms_prompt)
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(ms_hint)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_MUTED),
                            );

                            ui.add_space(16.0);

                            let btn = egui::Button::new(
                                egui::RichText::new(lang.login_btn_get_code())
                                    .font(egui::FontId::proportional(13.0))
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(RUBY)
                            .stroke(Stroke::new(1.0, RUBY_LIGHT))
                            .min_size(vec2(ui.available_width(), 40.0));

                            if ui.add(btn).clicked() {
                                request_ms = true;
                            }
                        }
                    }
                }

                ui.add_space(12.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(lang.login_btn_cancel()).clicked() {
                        self.is_open = false;
                        self.error_msg = None;
                    }
                });
            });

        if let Some(acc) = success_account {
            on_success(acc);
        }

        if request_ms {
            on_request_ms_code();
        }
    }

    fn tab_btn(&mut self, ui: &mut Ui, label: &str, mode: LoginMode) {
        let is_active = self.mode == mode;
        let (bg, stroke, text_color) = if is_active {
            (RUBY, Stroke::new(1.0, RUBY_LIGHT), Color32::WHITE)
        } else {
            (BG_HOVER, Stroke::new(1.0, BORDER_DEFAULT), TEXT_MUTED)
        };

        let btn = egui::Button::new(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(10.0))
                .strong()
                .color(text_color),
        )
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::ZERO)
        .min_size(vec2(120.0, 28.0));

        if ui.add(btn).clicked() {
            self.mode = mode;
            self.error_msg = None;
        }
    }
}
