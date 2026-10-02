use amc_auth::Account;
use amc_core::Language;
use egui::{vec2, Color32, Rounding, Sense, Ui};
use image::RgbaImage;

use crate::theme::{BG_ELEVATED, BORDER_DEFAULT, TEXT_HEADING, TEXT_MUTED};

/// Player profile tab (CONCEPT "Профиль игрока", P7): nickname, skin and
/// cape in one place. Editing reuses the login modal (nickname) and the
/// Skins tab, so this page stays a thin honest summary.
#[derive(Default)]
pub struct ProfilePage {
    tex: Option<egui::TextureHandle>,
    tex_key: Option<(bool, usize)>,
}

pub enum ProfileAction {
    None,
    OpenLogin,
    OpenSkins,
}

impl ProfilePage {
    fn ensure_tex(&mut self, ctx: &egui::Context, skin: Option<&RgbaImage>, slim: bool) {
        let key = (slim, skin.map(|s| s.len()).unwrap_or(0));
        if self.tex_key == Some(key) {
            return;
        }
        let avatar = crate::pages::skins::extract_head_avatar(skin, slim);
        self.tex = Some(ctx.load_texture("profile_avatar", avatar, egui::TextureOptions::NEAREST));
        self.tex_key = Some(key);
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        skin: Option<&RgbaImage>,
        slim: bool,
        lang: Language,
    ) -> ProfileAction {
        let mut action = ProfileAction::None;
        self.ensure_tex(ui.ctx(), skin, slim);

        ui.add_space(20.0);
        ui.label(
            egui::RichText::new(lang.profile_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            // Avatar card.
            let (av_rect, _) = ui.allocate_exact_size(vec2(120.0, 120.0), Sense::hover());
            ui.painter()
                .rect_filled(av_rect, Rounding::ZERO, BG_ELEVATED);
            ui.painter().rect_stroke(
                av_rect,
                Rounding::ZERO,
                egui::Stroke::new(1.0_f32, BORDER_DEFAULT),
            );
            if let Some(tex) = &self.tex {
                ui.painter().image(
                    tex.id(),
                    av_rect.shrink(8.0),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            ui.add_space(20.0);

            ui.vertical(|ui| {
                let name = account.map(|a| a.username.as_str()).unwrap_or("—");
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(lang.profile_nickname())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(name)
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(TEXT_HEADING),
                    );
                    if let Some(acc) = account {
                        super::super::modals::identity::account_tag(ui, acc.account_type);
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(lang.skins_title())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    if ui.button(lang.profile_title()).clicked() {
                        action = ProfileAction::OpenSkins;
                    }
                    if ui.button(lang.bottom_btn_login()).clicked() {
                        action = ProfileAction::OpenLogin;
                    }
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(lang.profile_cape_none())
                        .font(egui::FontId::proportional(12.0))
                        .color(TEXT_MUTED),
                );
            });
        });

        if let Some(acc) = account {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("UUID: {}", acc.uuid))
                    .font(egui::FontId::monospace(11.0))
                    .color(TEXT_MUTED),
            );
        }

        action
    }
}
