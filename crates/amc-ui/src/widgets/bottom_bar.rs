use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui};
use amc_auth::Account;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BottomBarResponse {
    pub login_clicked: bool,
    pub launch_clicked: bool,
    pub options_clicked: bool,
}

pub struct BottomBar;

impl BottomBar {
    pub fn show(
        ui: &mut Ui,
        active_account: Option<&Account>,
        is_launching: bool,
        avatar_texture: Option<&egui::TextureHandle>,
    ) -> BottomBarResponse {
        let mut response = BottomBarResponse::default();
        let bar_height = 76.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), bar_height));

        // Background and top border
        ui.painter().rect_filled(bar_rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().line_segment(
            [bar_rect.left_top(), bar_rect.right_top()],
            Stroke::new(1.0, BORDER_DEFAULT),
        );

        let right_width = 280.0;

        // --- Left Area: Avatar + User Info + Login Button ---
        let left_area = Rect::from_min_max(
            bar_rect.left_top(),
            Pos2::new((bar_rect.right() - right_width).max(bar_rect.left()), bar_rect.bottom()),
        );

        let mut left_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(left_area)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        left_ui.add_space(20.0);

        // Player Avatar square
        let av_size = 42.0;
        let (av_rect, _) = left_ui.allocate_exact_size(vec2(av_size, av_size), Sense::hover());
        left_ui.painter().rect_filled(av_rect, Rounding::ZERO, RUBY_DIM);
        left_ui.painter().rect_stroke(av_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));

        if let Some(tex) = avatar_texture {
            left_ui.painter().image(
                tex.id(),
                av_rect.shrink(2.0),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            left_ui.painter().text(
                av_rect.center(),
                egui::Align2::CENTER_CENTER,
                "👤",
                egui::FontId::proportional(20.0),
                RUBY_LIGHT,
            );
        }

        left_ui.add_space(14.0);

        // User text column
        let (name_text, sub_text) = if let Some(acc) = active_account {
            (
                acc.username.as_str(),
                match acc.account_type {
                    amc_auth::AccountType::Microsoft => "Microsoft License",
                    amc_auth::AccountType::WetId => "WetID AlephTrust",
                    amc_auth::AccountType::Offline => "Offline Free",
                },
            )
        } else {
            ("Не авторизован", "Войдите для игры")
        };

        left_ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(name_text)
                    .font(egui::FontId::proportional(14.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.label(
                egui::RichText::new(sub_text)
                    .font(egui::FontId::proportional(11.0))
                    .color(TEXT_MUTED),
            );
        });

        left_ui.add_space(16.0);

        // Login / Switch account button
        let btn_label = if active_account.is_some() { "СМЕНИТЬ" } else { "ВОЙТИ" };
        let login_btn = egui::Button::new(
            egui::RichText::new(btn_label)
                .font(egui::FontId::proportional(11.0))
                .strong()
                .color(TEXT_PRIMARY),
        )
        .fill(BG_HOVER)
        .stroke(Stroke::new(1.0, BORDER_STRONG))
        .rounding(Rounding::ZERO)
        .min_size(vec2(86.0, 34.0));

        if left_ui.add(login_btn).clicked() {
            response.login_clicked = true;
        }

        // --- Right Area: Big Ruby Play Button + Options ---
        let play_area = Rect::from_min_max(
            Pos2::new(bar_rect.right() - right_width, bar_rect.top()),
            bar_rect.right_bottom(),
        );

        let mut right_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(play_area)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );

        right_ui.add_space(20.0);

        // Options dropdown button (48x48)
        let (pa_rect, pa_resp) = right_ui.allocate_exact_size(vec2(48.0, 48.0), Sense::click());
        let pa_bg = if pa_resp.hovered() { RUBY } else { Color32::TRANSPARENT };
        let pa_fg = if pa_resp.hovered() { Color32::WHITE } else { RUBY };

        if pa_bg != Color32::TRANSPARENT {
            right_ui.painter().rect_filled(pa_rect, Rounding::ZERO, pa_bg);
        }
        right_ui.painter().rect_stroke(pa_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        right_ui.painter().text(
            pa_rect.center(),
            egui::Align2::CENTER_CENTER,
            "▾",
            egui::FontId::proportional(16.0),
            pa_fg,
        );

        if pa_resp.clicked() {
            response.options_clicked = true;
        }

        right_ui.add_space(8.0);

        // Main PLAY button (190x48)
        let play_text = if is_launching { "ЗАПУСК..." } else { "▶  ИГРАТЬ" };
        let (pm_rect, pm_resp) = right_ui.allocate_exact_size(vec2(190.0, 48.0), Sense::click());

        let pm_bg = if is_launching {
            RUBY_DIM
        } else if pm_resp.is_pointer_button_down_on() {
            RUBY_DIM
        } else if pm_resp.hovered() {
            RUBY_LIGHT
        } else {
            RUBY
        };

        right_ui.painter().rect_filled(pm_rect, Rounding::ZERO, pm_bg);
        right_ui.painter().rect_stroke(pm_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        right_ui.painter().text(
            pm_rect.center(),
            egui::Align2::CENTER_CENTER,
            play_text,
            egui::FontId::proportional(15.0),
            Color32::WHITE,
        );

        if pm_resp.clicked() && !is_launching {
            response.launch_clicked = true;
        }

        ui.advance_cursor_after_rect(bar_rect);
        response
    }
}
