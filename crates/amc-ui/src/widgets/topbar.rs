use crate::theme::{
    lerp_color, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, FontId, Rounding, Sense, Stroke, Ui};

/// Height of the Full-style professional top navigation bar.
pub const TOPBAR_HEIGHT: f32 = 68.0;

pub struct TopBarResponse {
    pub tab_clicked: Option<NavTab>,
    pub play_clicked: bool,
    pub account_clicked: bool,
}

/// Tabs in Full-style top navigation order.
pub fn pro_tabs() -> [NavTab; 5] {
    [
        NavTab::Home,
        NavTab::Modpacks,
        NavTab::Mods,
        NavTab::Skins,
        NavTab::Settings,
    ]
}

fn tab_label(tab: NavTab, lang: Language) -> &'static str {
    match tab {
        NavTab::Home => lang.nav_home(),
        NavTab::Modpacks => lang.nav_instances(),
        NavTab::Mods => lang.nav_mods(),
        NavTab::Skins => lang.nav_skins(),
        NavTab::Settings => lang.nav_settings(),
    }
}

/// Full-style top navigation: logo left, links center, account + Play right.
/// `slide_px` shifts content up for the mode-switch slide-down animation
/// (0 = in place).
pub fn show_topbar(
    ui: &mut Ui,
    lang: Language,
    current_tab: NavTab,
    account_name: Option<&str>,
    avatar: Option<&egui::TextureHandle>,
    is_launching: bool,
    slide_px: f32,
) -> TopBarResponse {
    let mut resp = TopBarResponse {
        tab_clicked: None,
        play_clicked: false,
        account_clicked: false,
    };

    // Bar background + bottom hairline.
    let bg_rect = ui.max_rect();
    ui.painter()
        .rect_filled(bg_rect, Rounding::ZERO, BG_ELEVATED);
    ui.painter().line_segment(
        [
            pos2(bg_rect.left(), bg_rect.bottom() - 0.5),
            pos2(bg_rect.right(), bg_rect.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, BORDER_DEFAULT),
    );

    if slide_px > 0.5 {
        ui.add_space(-slide_px);
    }

    ui.horizontal(|ui| {
        ui.add_space(16.0);

        // Logo: ruby square outline + wordmark, click goes Home.
        let (logo_rect, logo_resp) = ui.allocate_exact_size(vec2(170.0, 40.0), Sense::click());
        let sq = egui::Rect::from_min_size(
            pos2(logo_rect.min.x, logo_rect.center().y - 10.0),
            vec2(20.0, 20.0),
        );
        ui.painter()
            .rect_stroke(sq, Rounding::ZERO, Stroke::new(2.0_f32, RUBY));
        ui.painter().text(
            pos2(sq.right() + 10.0, logo_rect.center().y),
            Align2::LEFT_CENTER,
            "ALEPH LAUNCHER",
            FontId::proportional(12.0),
            TEXT_HEADING,
        );
        if logo_resp.clicked() {
            resp.tab_clicked = Some(NavTab::Home);
        }

        ui.add_space(24.0);

        // Nav links with animated underline.
        for tab in pro_tabs() {
            let label = tab_label(tab, lang);
            let font = FontId::proportional(13.0);
            let text_w = ui
                .painter()
                .layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE)
                .size()
                .x;
            let (rect, link_resp) =
                ui.allocate_exact_size(vec2(text_w + 8.0, 40.0), Sense::click());
            let hover_t = ui
                .ctx()
                .animate_bool_responsive(link_resp.id, link_resp.hovered());
            let act_t = ui
                .ctx()
                .animate_bool(link_resp.id.with("act"), tab == current_tab);
            let text_col = lerp_color(
                lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
                Color32::WHITE,
                act_t,
            );
            ui.painter().text(
                pos2(rect.center().x, rect.center().y - 3.0),
                Align2::CENTER_CENTER,
                label,
                font,
                text_col,
            );
            let line_w = (text_w + 8.0) * act_t;
            if line_w > 0.5 {
                let y = rect.bottom() - 5.0;
                ui.painter().line_segment(
                    [
                        pos2(rect.center().x - line_w / 2.0, y),
                        pos2(rect.center().x + line_w / 2.0, y),
                    ],
                    Stroke::new(2.0_f32, RUBY_LIGHT),
                );
            }
            if link_resp.clicked() {
                resp.tab_clicked = Some(tab);
            }
            ui.add_space(18.0);
        }

        // Right side: account chip + Play.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);

            let play_label = if is_launching {
                lang.bottom_btn_launching()
            } else {
                lang.bottom_btn_play()
            };
            let (play_rect, play_resp) = ui.allocate_exact_size(vec2(150.0, 40.0), Sense::click());
            let play_hover = ui
                .ctx()
                .animate_bool_responsive(play_resp.id, play_resp.hovered() && !is_launching);
            let play_bg = lerp_color(RUBY_DIM, RUBY, if is_launching { 0.4 } else { 1.0 });
            let play_bg = lerp_color(play_bg, RUBY_LIGHT, play_hover * 0.5);
            ui.painter().rect_filled(play_rect, Rounding::ZERO, play_bg);
            ui.painter().text(
                play_rect.center(),
                Align2::CENTER_CENTER,
                play_label,
                FontId::proportional(13.0),
                Color32::WHITE,
            );
            if play_resp.clicked() && !is_launching {
                resp.play_clicked = true;
            }

            ui.add_space(10.0);

            let display = account_name.unwrap_or_else(|| lang.bottom_btn_login());
            let name_w = ui
                .painter()
                .layout_no_wrap(
                    display.to_string(),
                    FontId::proportional(12.0),
                    Color32::WHITE,
                )
                .size()
                .x
                .clamp(30.0, 140.0);
            let chip_w = 12.0 + 30.0 + 8.0 + name_w + 12.0;
            let (chip_rect, chip_resp) = ui.allocate_exact_size(vec2(chip_w, 40.0), Sense::click());
            let chip_hover = ui
                .ctx()
                .animate_bool_responsive(chip_resp.id, chip_resp.hovered());
            ui.painter().rect_filled(
                chip_rect,
                Rounding::ZERO,
                lerp_color(BG_ELEVATED, BG_HOVER, chip_hover),
            );
            ui.painter().rect_stroke(
                chip_rect,
                Rounding::ZERO,
                Stroke::new(1.0_f32, BORDER_DEFAULT),
            );
            let av_rect = egui::Rect::from_min_size(
                pos2(chip_rect.min.x + 6.0, chip_rect.center().y - 15.0),
                vec2(30.0, 30.0),
            );
            match avatar {
                Some(tex) => {
                    ui.painter().image(
                        tex.id(),
                        av_rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => {
                    ui.painter().rect_filled(av_rect, Rounding::ZERO, RUBY_DIM);
                    let initial = display
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    ui.painter().text(
                        av_rect.center(),
                        Align2::CENTER_CENTER,
                        initial,
                        FontId::proportional(14.0),
                        Color32::WHITE,
                    );
                }
            }
            ui.painter().text(
                pos2(av_rect.right() + 8.0, chip_rect.center().y),
                Align2::LEFT_CENTER,
                display.to_uppercase(),
                FontId::proportional(12.0),
                lerp_color(TEXT_MUTED, TEXT_PRIMARY, chip_hover),
            );
            if chip_resp.clicked() {
                resp.account_clicked = true;
            }
        });
    });

    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pro_tabs_cover_all_nav() {
        let tabs = pro_tabs();
        assert_eq!(tabs.len(), 5);
        for expected in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Settings,
        ] {
            assert!(tabs.contains(&expected));
        }
    }
}
