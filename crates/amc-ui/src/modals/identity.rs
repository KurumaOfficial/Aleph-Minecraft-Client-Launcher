use amc_auth::{Account, AccountType};
use amc_core::Language;
use egui::{vec2, Align2, Color32, Rounding, Sense, Stroke, Ui};

use crate::theme::{BG_ELEVATED, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED};
use crate::widgets::choice_chip;

/// Pre-launch identity picker (CONCEPT "Аккаунты", P6): shown when the user
/// asked to choose on every launch and several identities exist.
/// Returns the picked account id plus whether to remember it.
pub struct IdentityPicker {
    pub open: bool,
    picked_id: Option<uuid::Uuid>,
    remember: bool,
}

impl Default for IdentityPicker {
    fn default() -> Self {
        Self {
            open: false,
            picked_id: None,
            remember: true,
        }
    }
}

pub enum PickerAction {
    None,
    Picked {
        account_id: uuid::Uuid,
        remember: bool,
    },
    Offline {
        nickname: String,
        remember: bool,
    },
    Cancelled,
}

impl IdentityPicker {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        accounts: &[Account],
        offline_nickname: &mut String,
        lang: Language,
    ) -> PickerAction {
        if !self.open {
            return PickerAction::None;
        }
        let mut action = PickerAction::None;
        egui::Window::new(lang.picker_title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(420.0)
            .show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                for acc in accounts {
                    let label = match acc.account_type {
                        AccountType::Microsoft => format!("{} (Microsoft)", acc.username),
                        AccountType::WetId => format!("{} (WetID)", acc.username),
                        AccountType::Offline => acc.username.clone(),
                    };
                    let active = self.picked_id == Some(acc.id);
                    if choice_chip(ui, &label, 380.0, 34.0, 12.0, active) {
                        self.picked_id = Some(acc.id);
                    }
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(lang.profile_nickname()).color(TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(offline_nickname).desired_width(200.0));
                });
                ui.add_space(8.0);
                let mut remember = self.remember;
                ui.checkbox(&mut remember, lang.picker_remember());
                self.remember = remember;
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let go = egui::Button::new(
                        egui::RichText::new(lang.bottom_btn_play())
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(RUBY)
                    .stroke(Stroke::new(1.0, RUBY_LIGHT))
                    .min_size(vec2(140.0, 36.0));
                    if ui.add(go).clicked() {
                        let nick = offline_nickname.trim().to_string();
                        if !nick.is_empty() {
                            action = PickerAction::Offline {
                                nickname: nick,
                                remember: self.remember,
                            };
                        } else if let Some(id) = self.picked_id {
                            action = PickerAction::Picked {
                                account_id: id,
                                remember: self.remember,
                            };
                        } else if let Some(first) = accounts.first() {
                            action = PickerAction::Picked {
                                account_id: first.id,
                                remember: self.remember,
                            };
                        }
                    }
                    if ui.button(lang.home_drawer_close()).clicked() {
                        action = PickerAction::Cancelled;
                    }
                });
            });
        if !matches!(action, PickerAction::None) {
            self.open = false;
        }
        action
    }
}

/// Readable account type tag.
pub fn account_tag(ui: &mut Ui, account_type: AccountType) {
    let (label, col) = match account_type {
        AccountType::Microsoft => ("Microsoft", Color32::from_rgb(0x70, 0x90, 0xC8)),
        AccountType::WetId => ("WetID", RUBY_LIGHT),
        AccountType::Offline => ("Offline", TEXT_MUTED),
    };
    let (rect, _) = ui.allocate_exact_size(vec2(86.0, 24.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
    ui.painter()
        .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, BORDER_DEFAULT));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        col,
    );
}
