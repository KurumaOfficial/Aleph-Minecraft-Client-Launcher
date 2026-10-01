use crate::theme::{self, RUBY_DIM, TEXT_MUTED};
use crate::widgets::full::{nav_link, red_button};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rounding, Sense, Stroke, Ui, Vec2};

/// Full reference metrics: 76px bar, 20px page padding.
pub const TOPBAR_HEIGHT: f32 = 76.0;
const PAD: f32 = 20.0;

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

/// Full-style top navigation (1:1 metrics): logo block left, mono links
/// center, account chip + red Play right. `slide_px` shifts content up for
/// the mode-switch slide-down animation (0 = in place).
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
    ui.painter().rect_filled(bg_rect, Rounding::ZERO, theme::BG);
    ui.painter().line_segment(
        [
            pos2(bg_rect.left(), bg_rect.bottom() - 0.5),
            pos2(bg_rect.right(), bg_rect.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    if slide_px > 0.5 {
        ui.add_space(-slide_px);
    }

    ui.add_space(18.0);
    ui.horizontal(|ui| {
        ui.add_space(PAD);

        // Logo block 210x40: ruby square outline + wordmark, click → Home.
        let logo_resp = ui.allocate_response(vec2(210.0, 40.0), Sense::click());
        {
            let p = ui.painter().clone();
            let sq = egui::Rect::from_min_size(logo_resp.rect.min, Vec2::splat(22.0));
            p.rect_stroke(sq, Rounding::ZERO, Stroke::new(2.0_f32, theme::ACCENT));
            p.text(
                pos2(sq.right() + 12.0, sq.center().y),
                Align2::LEFT_CENTER,
                "ALEPH LAUNCHER",
                theme::display(12.0),
                theme::HEAD,
            );
        }
        if logo_resp.clicked() {
            resp.tab_clicked = Some(NavTab::Home);
        }

        ui.add_space(30.0);

        // Mono nav links with Full underline.
        for tab in pro_tabs() {
            if nav_link(ui, tab_label(tab, lang), tab == current_tab).clicked() {
                resp.tab_clicked = Some(tab);
            }
            ui.add_space(22.0);
        }

        // Right side: account chip + red Play.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD);
            let play_label = if is_launching {
                lang.bottom_btn_launching()
            } else {
                lang.bottom_btn_play()
            };
            if red_button(ui, play_label, 150.0, 40.0).clicked() && !is_launching {
                resp.play_clicked = true;
            }
            ui.add_space(10.0);

            let display = account_name.unwrap_or_else(|| lang.bottom_btn_login());
            let name_w =
                crate::widgets::full::text_width(ui, &display.to_uppercase(), &theme::mono(11.0))
                    .clamp(30.0, 150.0);
            let w = 8.0 + 30.0 + 10.0 + name_w + 12.0;
            let (rect, response) = ui.allocate_exact_size(vec2(w, 40.0), Sense::click());
            let anim =
                ui.ctx()
                    .animate_bool_with_time(ui.id().with("account"), response.hovered(), 0.15);
            let painter = ui.painter().clone();
            painter.rect_filled(
                rect,
                Rounding::ZERO,
                theme::lerp_color(theme::BG, theme::CARD_HOVER, anim),
            );
            painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, theme::BORDER));
            let avatar_rect =
                egui::Rect::from_min_size(rect.min + vec2(5.0, 5.0), Vec2::splat(30.0));
            match avatar {
                Some(tex) => {
                    painter.image(
                        tex.id(),
                        avatar_rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    painter.rect_stroke(
                        avatar_rect,
                        Rounding::ZERO,
                        Stroke::new(1.0_f32, theme::BORDER),
                    );
                }
                None => {
                    painter.rect_filled(avatar_rect, Rounding::ZERO, RUBY_DIM);
                    let initial = display
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    painter.text(
                        avatar_rect.center(),
                        Align2::CENTER_CENTER,
                        initial,
                        theme::mono(14.0),
                        Color32::WHITE,
                    );
                }
            }
            painter.text(
                pos2(rect.min.x + 45.0, rect.center().y),
                Align2::LEFT_CENTER,
                display.to_uppercase(),
                theme::mono(11.0),
                theme::lerp_color(TEXT_MUTED, theme::HEAD, anim),
            );
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
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
