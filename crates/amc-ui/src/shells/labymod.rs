//! LabyMod-inspired shell (layout only).
//!
//! Reference: public Flathub screenshots of the LabyMod launcher. The launcher
//! uses a dark navy canvas, a slim labelled icon rail on the left, a large
//! hero Play card in the centre and a friends/news column on the right. No
//! LabyMod assets, wordmark or logo are used: the identity is AMC everywhere.

use crate::shells::{tab_label, ShellAction};
use crate::theme;
use crate::widgets::brand;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, Sense, Stroke, Ui};

/// Width of the labelled icon rail.
pub const RAIL_WIDTH: f32 = 92.0;
const RAIL_BG: Color32 = Color32::from_rgb(0x08, 0x0B, 0x11);
const ACTIVE_BG: Color32 = Color32::from_rgb(0x16, 0x25, 0x42);

/// Tabs in the rail order: Play first, settings last.
pub const TABS: [NavTab; 6] = [
    NavTab::Home,
    NavTab::Modpacks,
    NavTab::Mods,
    NavTab::Skins,
    NavTab::Profile,
    NavTab::Settings,
];

fn tab_icon(tab: NavTab) -> Icon {
    match tab {
        NavTab::Home => Icon::Play,
        NavTab::Modpacks => Icon::Grid,
        NavTab::Mods => Icon::Puzzle,
        NavTab::Skins => Icon::Palette,
        NavTab::Profile => Icon::Person,
        NavTab::Settings => Icon::Gear,
    }
}

/// Left rail: large labelled buttons on a near-black strip, with the AMC
/// hollow-square mark pinned to the bottom.
pub fn show_rail(ui: &mut Ui, lang: Language, current_tab: NavTab) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, Rounding::ZERO, RAIL_BG);
    painter.line_segment(
        [
            pos2(bar.right() - 0.5, bar.top()),
            pos2(bar.right() - 0.5, bar.bottom()),
        ],
        Stroke::new(1.0_f32, theme::BORDER_SUBTLE),
    );

    let item_w = 76.0;
    let item_h = 60.0;
    let gap = 6.0;
    let mut y = bar.top() + 14.0;
    for (idx, tab) in TABS.into_iter().enumerate() {
        let rect =
            Rect::from_min_size(pos2(bar.center().x - item_w / 2.0, y), vec2(item_w, item_h));
        let resp = ui.interact(rect, ui.id().with(("labymod_rail", idx)), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active = tab == current_tab;
        let label = if tab == NavTab::Home {
            lang.labymod_play_tab()
        } else {
            tab_label(tab, lang)
        };

        if active {
            ui.painter()
                .rect_filled(rect, Rounding::same(theme::rounding().nw), ACTIVE_BG);
            ui.painter().rect_stroke(
                rect,
                Rounding::same(theme::rounding().nw),
                Stroke::new(1.0_f32, theme::with_alpha(theme::ACCENT, 70)),
            );
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(rect.left() - 5.0, rect.top() + 14.0), vec2(3.0, 32.0)),
                Rounding::same(2.0),
                theme::ACCENT,
            );
        } else if hover > 0.02 {
            ui.painter().rect_filled(
                rect,
                Rounding::same(theme::rounding().nw),
                theme::with_alpha(theme::CARD, (hover * 150.0) as u8),
            );
        }

        let fg: Color32 = if active {
            theme::ACCENT_LIGHT.into()
        } else {
            theme::lerp_color(theme::TEXT_DIM, theme::TEXT, hover)
        };
        let icon_rect =
            Rect::from_center_size(pos2(rect.center().x, rect.top() + 24.0), vec2(22.0, 22.0));
        icons::draw(
            ui.painter(),
            tab_icon(tab),
            icon_rect,
            fg,
            if active { 1.8 } else { 1.6 },
        );
        ui.painter().text(
            pos2(rect.center().x, rect.bottom() - 13.0),
            Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.5),
            fg,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            action.tab = Some(tab);
        }
        y = rect.bottom() + gap;
    }

    // AMC identity pinned to the bottom, in the reference logo slot.
    let mark = Rect::from_center_size(pos2(bar.center().x, bar.bottom() - 48.0), vec2(24.0, 24.0));
    brand::hollow_square(ui.painter(), mark, brand::BRAND_RED);
    ui.painter().text(
        pos2(bar.center().x, bar.bottom() - 22.0),
        Align2::CENTER_CENTER,
        "AMC",
        theme::display(13.0),
        theme::HEAD.into(),
    );
    ui.painter().text(
        pos2(bar.center().x, bar.bottom() - 10.0),
        Align2::CENTER_CENTER,
        "LAUNCHER",
        egui::FontId::proportional(7.5),
        theme::MUTED.into(),
    );

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
