use egui::{vec2, Color32, Rounding, Sense, Stroke, Ui, UiBuilder};
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
    ) -> BottomBarResponse {
        let mut response = BottomBarResponse::default();
        let bar_height = 76.0;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), bar_height), Sense::hover());

        // Background and top border
        ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, BORDER_DEFAULT),
        );

        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        child.add_space(20.0);

        // Player Avatar square
        let av_size = 44.0;
        let (av_rect, _) = child.allocate_exact_size(vec2(av_size, av_size), Sense::hover());
        child.painter().rect_filled(av_rect, Rounding::ZERO, RUBY_DIM);
        child.painter().rect_stroke(av_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        child.painter().text(
            av_rect.center(),
            egui::Align2::CENTER_CENTER,
            "👤",
            egui::FontId::proportional(20.0),
            RUBY_LIGHT,
        );

        child.add_space(14.0);

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

        child.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(egui::RichText::new(name_text).font(egui::FontId::proportional(14.0)).strong().color(TEXT_HEADING));
            ui.label(egui::RichText::new(sub_text).font(egui::FontId::proportional(11.0)).color(TEXT_MUTED));
        });

        child.add_space(14.0);

        // Login / Switch account button
        let login_btn = egui::Button::new(
            egui::RichText::new("ВОЙТИ")
                .font(egui::FontId::proportional(12.0))
                .strong()
                .color(TEXT_PRIMARY),
        )
        .fill(BG_HOVER)
        .stroke(Stroke::new(1.0, BORDER_STRONG))
        .rounding(Rounding::ZERO);

        if child.add(login_btn).clicked() {
            response.login_clicked = true;
        }

        // Push Play buttons to the right
        let right_group_width = 270.0;
        let available_space = child.available_width() - right_group_width;
        if available_space > 0.0 {
            child.add_space(available_space);
        }

        // Main PLAY button
        let play_text = if is_launching { "ЗАПУСК..." } else { "▶  ИГРАТЬ" };
        let (pm_rect, pm_resp) = child.allocate_exact_size(vec2(200.0, 50.0), Sense::click());

        let pm_bg = if is_launching {
            RUBY_DIM
        } else if pm_resp.is_pointer_button_down_on() {
            RUBY_DIM
        } else if pm_resp.hovered() {
            RUBY_LIGHT
        } else {
            RUBY
        };

        child.painter().rect_filled(pm_rect, Rounding::ZERO, pm_bg);
        child.painter().rect_stroke(pm_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        child.painter().text(
            pm_rect.center(),
            egui::Align2::CENTER_CENTER,
            play_text,
            egui::FontId::proportional(15.0),
            Color32::WHITE,
        );

        if pm_resp.clicked() && !is_launching {
            response.launch_clicked = true;
        }

        child.add_space(6.0);

        // Options dropdown button (50x50)
        let (pa_rect, pa_resp) = child.allocate_exact_size(vec2(50.0, 50.0), Sense::click());
        let pa_bg = if pa_resp.hovered() { RUBY } else { Color32::TRANSPARENT };
        let pa_fg = if pa_resp.hovered() { Color32::WHITE } else { RUBY };

        if pa_bg != Color32::TRANSPARENT {
            child.painter().rect_filled(pa_rect, Rounding::ZERO, pa_bg);
        }
        child.painter().rect_stroke(pa_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        child.painter().text(
            pa_rect.center(),
            egui::Align2::CENTER_CENTER,
            "▾",
            egui::FontId::proportional(16.0),
            pa_fg,
        );

        if pa_resp.clicked() {
            response.options_clicked = true;
        }

        response
    }
}
