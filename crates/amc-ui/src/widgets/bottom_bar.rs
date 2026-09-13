use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui};
use amc_auth::Account;
use crate::theme::{
    lerp_color, BG_ACTIVE, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_DIM,
    RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
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
        lang: amc_core::Language,
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
            (lang.bottom_not_authorized(), lang.bottom_login_prompt())
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
        let btn_label = if active_account.is_some() { lang.bottom_btn_switch() } else { lang.bottom_btn_login() };
        let (login_rect, login_resp) = left_ui.allocate_exact_size(vec2(96.0, 34.0), Sense::click());
        let login_hover = left_ui.ctx().animate_bool_responsive(login_resp.id, login_resp.hovered());
        let login_bg = lerp_color(BG_HOVER, BG_ACTIVE, login_hover);
        let login_stroke = lerp_color(BORDER_STRONG, RUBY, login_hover);
        let login_fg = lerp_color(TEXT_PRIMARY, Color32::WHITE, login_hover);

        left_ui.painter().rect_filled(login_rect, Rounding::ZERO, login_bg);
        left_ui.painter().rect_stroke(login_rect, Rounding::ZERO, Stroke::new(1.0, login_stroke));
        left_ui.painter().text(
            login_rect.center(),
            egui::Align2::CENTER_CENTER,
            btn_label,
            egui::FontId::proportional(11.5),
            login_fg,
        );

        if login_resp.clicked() {
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
        let (pa_rect, pa_resp) = right_ui.allocate_exact_size(vec2(44.0, 48.0), Sense::click());
        let pa_hover = right_ui.ctx().animate_bool_responsive(pa_resp.id, pa_resp.hovered());
        let pa_bg = lerp_color(Color32::from_rgba_premultiplied(139, 26, 42, 10), RUBY, pa_hover);
        let pa_fg = lerp_color(RUBY_LIGHT, Color32::WHITE, pa_hover);
        let pa_stroke = lerp_color(RUBY, RUBY_LIGHT, pa_hover);

        right_ui.painter().rect_filled(pa_rect, Rounding::ZERO, pa_bg);
        right_ui.painter().rect_stroke(pa_rect, Rounding::ZERO, Stroke::new(1.5, pa_stroke));
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
        let play_text = if is_launching { lang.bottom_btn_launching() } else { lang.bottom_btn_play() };
        let (pm_rect, pm_resp) = right_ui.allocate_exact_size(vec2(190.0, 48.0), Sense::click());

        let hover_t = right_ui.ctx().animate_bool_responsive(pm_resp.id, pm_resp.hovered());
        let press_t = right_ui.ctx().animate_bool_responsive(pm_resp.id.with("press"), pm_resp.is_pointer_button_down_on());

        let base_bg = lerp_color(RUBY, RUBY_LIGHT, hover_t);
        let pm_bg = if is_launching {
            RUBY_DIM
        } else {
            lerp_color(base_bg, RUBY_DIM, press_t)
        };

        let stroke_col = if is_launching {
            RUBY
        } else {
            lerp_color(RUBY, Color32::from_rgb(0xE8, 0x55, 0x70), hover_t)
        };

        right_ui.painter().rect_filled(pm_rect, Rounding::ZERO, pm_bg);
        right_ui.painter().rect_stroke(pm_rect, Rounding::ZERO, Stroke::new(2.0, stroke_col));

        // When launching: render sleek animated top scan-line (runs only during launch!)
        if is_launching {
            let time = right_ui.input(|i| i.time);
            let cycle = (time * 1.5).fract() as f32;
            let scan_w = 40.0;
            let scan_x = pm_rect.left() + (pm_rect.width() + scan_w) * cycle - scan_w;
            let scan_rect = Rect::from_min_size(
                Pos2::new(scan_x.clamp(pm_rect.left(), pm_rect.right() - scan_w), pm_rect.top()),
                vec2(scan_w, 2.0),
            );
            right_ui.painter().rect_filled(scan_rect, Rounding::ZERO, Color32::WHITE);
            right_ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }

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
