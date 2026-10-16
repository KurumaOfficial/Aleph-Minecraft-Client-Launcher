//! Modrinth App-inspired shell (layout only).
//!
//! Credits:
//! - Modrinth App (Theseus) — Modrinth / Theseus contributors, GPL-3.0 —
//!   <https://github.com/modrinth/code> (`apps/app-frontend`)
//!
//! Layout reference: a left navigation sidebar over flat gray surfaces with
//! a green accent in place of the Aleph ruby. The colors follow the app's
//! public dark-theme CSS variables (`--surface-1` canvas #16181c,
//! `--surface-3` raised rows #27292e, muted text #b0bac5, brand green
//! #1bd96a); no assets or code are reused.
//!
//! AMC mapping: the sidebar carries the shared `NavTab` rows, the green Play
//! button (the reference's primary action) and the account chip. The
//! gray/green palette is applied to the egui visuals through
//! [`crate::theme::apply_modrinth_theme`] while this shell is active; the
//! shared pages keep their Aleph colors.

use crate::shells::{tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, Sense, Stroke, Ui};

/// Sidebar width (reference: fixed navigation rail).
pub const SIDEBAR_WIDTH: f32 = 190.0;
const PAD: f32 = 10.0;

/// Sidebar shortcuts in display order.
pub const NAV_TABS: [(NavTab, Icon); 6] = [
    (NavTab::Home, Icon::Home),
    (NavTab::Modpacks, Icon::Package),
    (NavTab::Mods, Icon::Puzzle),
    (NavTab::Skins, Icon::Diamond),
    (NavTab::Profile, Icon::Person),
    (NavTab::Settings, Icon::Gear),
];

/// Green Play plate (reference primary button: dark label on green).
/// Returns the plate's interaction response.
fn play_button(ui: &mut Ui, rect: Rect, lang: Language, launching: bool) -> egui::Response {
    let resp = ui.interact(rect, ui.id().with("modrinth_play"), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let base = if launching {
        theme::MODRINTH_GREEN_DIM
    } else {
        theme::MODRINTH_GREEN
    };
    let fill = theme::lerp_color(base, theme::MODRINTH_GREEN_LIGHT, hover * 0.7);
    ui.painter().rect_filled(rect, Rounding::same(4.0), fill);

    let label = if launching {
        lang.bottom_btn_launching()
    } else {
        lang.bottom_btn_play()
    };
    let text_color = theme::MODRINTH_BG;
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), theme::mono(12.5), text_color);
    let content_w = 15.0 + 8.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        Icon::Play,
        Rect::from_center_size(pos2(left + 7.5, rect.center().y), vec2(15.0, 15.0)),
        text_color,
        1.5,
    );
    ui.painter().galley(
        pos2(left + 23.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Navigation sidebar: wordmark, page rows, green Play button and the
/// account chip pinned to the bottom.
pub fn show_sidebar(
    ui: &mut Ui,
    lang: Language,
    current_tab: NavTab,
    account_name: Option<&str>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    ui.painter()
        .rect_filled(bar, crate::theme::rounding(), theme::MODRINTH_BG);
    ui.painter().line_segment(
        [
            pos2(bar.right() - 0.5, bar.top()),
            pos2(bar.right() - 0.5, bar.bottom()),
        ],
        Stroke::new(1.0_f32, theme::MODRINTH_SURFACE),
    );

    // Wordmark header (same brand block as the other shells).
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(PAD);
        ui.label(
            egui::RichText::new("ALEPH LAUNCHER")
                .font(theme::display(12.0))
                .color(theme::MODRINTH_TEXT),
        );
    });
    ui.add_space(12.0);

    // Page rows: green-tinted pill when active, gray hover otherwise.
    for (tab, icon) in NAV_TABS {
        let (row, _) = ui.allocate_exact_size(vec2(bar.width(), 34.0), Sense::hover());
        let rect = row.shrink2(vec2(PAD, 2.0));
        let resp = ui
            .interact(
                rect,
                ui.id().with(("modrinth_nav", tab as usize)),
                Sense::click(),
            )
            .on_hover_text(tab_label(tab, lang));
        let active = tab == current_tab;
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active || hover > 0.02 {
            let fill = if active {
                theme::with_alpha(theme::MODRINTH_GREEN, 34)
            } else {
                theme::with_alpha(theme::MODRINTH_SURFACE_HOVER, (hover * 255.0) as u8)
            };
            ui.painter().rect_filled(rect, Rounding::same(4.0), fill);
        }
        if active {
            ui.painter().rect_filled(
                Rect::from_min_max(
                    pos2(rect.left(), rect.top() + 6.0),
                    pos2(rect.left() + 2.0, rect.bottom() - 6.0),
                ),
                Rounding::same(1.0),
                theme::MODRINTH_GREEN,
            );
        }
        let glyph = if active {
            theme::MODRINTH_GREEN
        } else {
            theme::lerp_color(theme::MODRINTH_MUTED, theme::MODRINTH_TEXT, hover)
        };
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(rect.left() + 16.0, rect.center().y), vec2(15.0, 15.0)),
            glyph,
            1.4,
        );
        let galley = ui.painter().layout_no_wrap(
            tab_label(tab, lang).to_owned(),
            theme::mono(11.5),
            Color32::PLACEHOLDER,
        );
        ui.painter().galley(
            pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
            galley,
            if active {
                theme::MODRINTH_TEXT
            } else {
                theme::lerp_color(theme::MODRINTH_MUTED, theme::MODRINTH_TEXT, hover)
            },
        );
        if resp.clicked() {
            action.tab = Some(tab);
        }
    }

    // Bottom block: Play button above the account chip.
    let remaining = ui.available_height() - 96.0;
    if remaining > 0.0 {
        ui.add_space(remaining);
    }
    let (play_row, _) = ui.allocate_exact_size(vec2(bar.width(), 44.0), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(bar.left() + PAD, play_row.top()),
            pos2(bar.right() - PAD, play_row.top()),
        ],
        Stroke::new(1.0_f32, theme::MODRINTH_SURFACE),
    );
    if play_button(ui, play_row.shrink2(vec2(PAD, 3.0)), lang, launching).clicked() && !launching {
        action.launch = true;
    }

    let (chip_row, _) = ui.allocate_exact_size(vec2(bar.width(), 46.0), Sense::hover());
    let chip = chip_row.shrink2(vec2(PAD, 4.0));
    let display = account_name.unwrap_or_else(|| lang.shell_nickname());
    let resp = ui
        .interact(chip, ui.id().with("modrinth_account"), Sense::click())
        .on_hover_text(display);
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    ui.painter().rect_filled(
        chip,
        Rounding::same(4.0),
        theme::with_alpha(theme::MODRINTH_SURFACE_HOVER, (hover * 255.0) as u8),
    );
    let center = pos2(chip.left() + 15.0, chip.center().y);
    ui.painter()
        .circle_filled(center, 12.0, theme::MODRINTH_SURFACE_HOVER);
    match account_name.and_then(|name| name.chars().next()) {
        Some(letter) => {
            ui.painter().text(
                center,
                Align2::CENTER_CENTER,
                letter.to_uppercase(),
                theme::display(11.5),
                theme::MODRINTH_TEXT,
            );
        }
        None => {
            icons::draw(
                ui.painter(),
                Icon::Person,
                Rect::from_center_size(center, vec2(14.0, 14.0)),
                theme::MODRINTH_TEXT,
                1.3,
            );
        }
    }
    let galley =
        ui.painter()
            .layout_no_wrap(display.to_owned(), theme::mono(11.5), Color32::PLACEHOLDER);
    ui.painter().galley(
        pos2(center.x + 20.0, chip.center().y - galley.size().y / 2.0),
        galley,
        theme::lerp_color(theme::MODRINTH_MUTED, theme::MODRINTH_TEXT, hover),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        action.login = true;
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_covers_every_page_once() {
        for tab in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Profile,
            NavTab::Settings,
        ] {
            assert_eq!(
                NAV_TABS
                    .iter()
                    .filter(|(candidate, _)| *candidate == tab)
                    .count(),
                1,
                "{tab:?} must appear exactly once in the sidebar"
            );
        }
    }
}
