//! Legacy Minecraft Launcher-inspired shell (layout only).
//!
//! Credits:
//! - Legacy Minecraft Launcher (Mojang Studios, proprietary): the layout is
//!   reconstructed from public screenshots only; no assets or code reused.
//!
//! Layout reference: a row of plain text tabs on top (account name at the
//! right) and a bottom Play panel that pairs the selected profile / version
//! with the launch button. AMC mapping: the tabs mirror the shared pages,
//! the panel shows the selected instance plus the ruby Play button, and all
//! colors stay in the Aleph palette (no Mojang assets involved).

use crate::shells::{short_name, tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, Sense, Stroke, Ui};

/// Tab strip height (reference: plain text row).
pub const TABS_HEIGHT: f32 = 40.0;
/// Bottom Play panel height (reference: profile picker + Play button).
pub const PLAY_HEIGHT: f32 = 64.0;
const PAD: f32 = 14.0;

/// Tabs in the reference order: content first, settings last.
pub const TABS: [NavTab; 6] = [
    NavTab::Home,
    NavTab::Modpacks,
    NavTab::Mods,
    NavTab::Skins,
    NavTab::Profile,
    NavTab::Settings,
];

/// Bottom Play button with a leading vector play mark.
fn play_button(ui: &mut Ui, label: &str, launching: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(210.0, 44.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let base = if launching {
        theme::ACCENT_DIM
    } else {
        theme::RUBY
    };
    let fill = theme::lerp_color(base, theme::ACCENT_LIGHT, hover * 0.6);
    ui.painter().rect_filled(rect, Rounding::ZERO, fill);
    ui.painter().rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );

    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), theme::display(15.0), Color32::WHITE);
    let content_w = 16.0 + 10.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        Icon::Play,
        Rect::from_center_size(pos2(left + 8.0, rect.center().y), vec2(16.0, 16.0)),
        Color32::WHITE,
        1.5,
    );
    ui.painter().galley(
        pos2(left + 26.0, rect.center().y - galley.size().y / 2.0),
        galley,
        Color32::WHITE,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked() && !launching
}

/// Top strip: plain text tabs (active one underlined), account on the right.
pub fn show_tabs(
    ui: &mut Ui,
    lang: Language,
    current_tab: NavTab,
    account_name: Option<&str>,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    ui.painter().rect_filled(bar, Rounding::ZERO, theme::CARD);
    ui.painter().line_segment(
        [
            pos2(bar.left(), bar.bottom() - 0.5),
            pos2(bar.right(), bar.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal_centered(|ui| {
        ui.add_space(PAD);
        for tab in TABS {
            let active = tab == current_tab;
            let label = tab_label(tab, lang).to_uppercase();
            let galley =
                ui.painter()
                    .layout_no_wrap(label, theme::mono(11.5), Color32::PLACEHOLDER);
            let (rect, resp) =
                ui.allocate_exact_size(vec2(galley.size().x + 26.0, 26.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let color = if active {
                theme::HEAD.into()
            } else {
                theme::lerp_color(theme::MUTED, theme::TEXT, hover)
            };
            ui.painter().galley(
                pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                color,
            );
            if active || hover > 0.02 {
                let alpha = if active { 255.0 } else { 150.0 * hover };
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        pos2(rect.left() + 5.0, rect.bottom() - 2.0),
                        pos2(rect.right() - 5.0, rect.bottom() - 1.0),
                    ),
                    Rounding::ZERO,
                    theme::with_alpha(theme::RUBY_LIGHT, alpha as u8),
                );
            }
            if resp.clicked() {
                action.tab = Some(tab);
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD);
            let display = account_name.unwrap_or_else(|| lang.shell_nickname());
            if icons::icon_text_button(
                ui,
                Icon::Person,
                None,
                display,
                theme::TEXT,
                theme::mono(11.5),
                theme::FIELD,
            )
            .clicked()
            {
                action.login = true;
            }
        });
    });

    action
}

/// Bottom panel: the selected instance as a profile block (click opens the
/// list), Play on the right — the reference's version selector + Play row.
pub fn show_play_bar(
    ui: &mut Ui,
    lang: Language,
    instance: Option<(&str, &str, &str)>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    ui.painter().rect_filled(bar, Rounding::ZERO, theme::CARD);
    ui.painter().line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal_centered(|ui| {
        ui.add_space(PAD);
        match instance {
            Some((name, version, loader)) => {
                let (rect, resp) = ui.allocate_exact_size(vec2(280.0, 44.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    theme::lerp_color(theme::FIELD, theme::CARD_HOVER, hover),
                );
                ui.painter()
                    .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, theme::BORDER));
                ui.painter().text(
                    pos2(rect.left() + 12.0, rect.center().y - 8.0),
                    Align2::LEFT_CENTER,
                    short_name(name, 30),
                    theme::display_reg(13.0),
                    theme::TEXT.into(),
                );
                ui.painter().text(
                    pos2(rect.left() + 12.0, rect.center().y + 10.0),
                    Align2::LEFT_CENTER,
                    format!("MC {version} • {loader}"),
                    theme::mono(10.0),
                    theme::MUTED.into(),
                );
                icons::draw(
                    ui.painter(),
                    Icon::CaretDown,
                    Rect::from_center_size(
                        pos2(rect.right() - 15.0, rect.center().y),
                        vec2(10.0, 10.0),
                    ),
                    theme::MUTED,
                    1.3,
                );
                if resp.clicked() {
                    action.tab = Some(NavTab::Modpacks);
                }
            }
            None => {
                if icons::icon_text_button(
                    ui,
                    Icon::Package,
                    None,
                    lang.shell_add_instance(),
                    theme::MUTED,
                    theme::mono(12.0),
                    theme::ACCENT_SOFT,
                )
                .clicked()
                {
                    action.tab = Some(NavTab::Modpacks);
                }
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD);
            let label = if launching {
                lang.bottom_btn_launching()
            } else {
                lang.bottom_btn_play()
            };
            if play_button(ui, label, launching) {
                action.launch = true;
            }
        });
    });

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabs_cover_every_page_once() {
        for tab in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Profile,
            NavTab::Settings,
        ] {
            assert_eq!(
                TABS.iter().filter(|candidate| **candidate == tab).count(),
                1,
                "{tab:?} must appear exactly once"
            );
        }
    }
}
