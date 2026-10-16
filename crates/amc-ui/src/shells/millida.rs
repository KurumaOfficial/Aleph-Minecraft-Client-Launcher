//! Millida Launcher-inspired shell (layout only).
//!
//! Credits:
//! - Millida Launcher — Millida contributors, GPL-3.0 —
//!   <https://github.com/millida/launcher>
//!
//! Layout reference: a slim icon navigation rail on the left, and a lobby
//! (`playhub`) home screen built from a hero/continue card plus a grid of
//! tiles; clicking a tile opens the matching content page.
//!
//! AMC mapping: the rail carries the shared `NavTab` shortcuts (account chip
//! pinned at the bottom), the Home tab becomes the lobby (selected instance
//! as the continue card, hub tiles for Play / versions / mods / skins), and
//! every other page keeps its shared layout. All colors stay in the Aleph
//! palette; no third-party artwork or code is reused.

use crate::shells::{short_name, tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, Sense, Stroke, Ui};

/// Left navigation rail width (reference: slim icon strip).
pub const NAV_WIDTH: f32 = 56.0;

/// Rail shortcuts in display order: Home opens the lobby itself.
pub const NAV_TABS: [(NavTab, Icon); 6] = [
    (NavTab::Home, Icon::Home),
    (NavTab::Modpacks, Icon::Package),
    (NavTab::Mods, Icon::Puzzle),
    (NavTab::Skins, Icon::Diamond),
    (NavTab::Profile, Icon::Person),
    (NavTab::Settings, Icon::Gear),
];

/// Hub tiles under the hero card: icon plus page target (`None` launches the
/// selected instance — the reference's quick-play tile).
pub const TILE_TARGETS: [(Icon, Option<NavTab>); 4] = [
    (Icon::Play, None),
    (Icon::Package, Some(NavTab::Modpacks)),
    (Icon::Puzzle, Some(NavTab::Mods)),
    (Icon::Diamond, Some(NavTab::Skins)),
];

/// Localized caption for hub tile `index` (matches `TILE_TARGETS` order).
fn tile_label(index: usize, lang: Language) -> &'static str {
    match index {
        0 => lang.shell_play_tile(),
        1 => lang.shell_versions_tile(),
        2 => tab_label(NavTab::Mods, lang),
        3 => tab_label(NavTab::Skins, lang),
        _ => lang.shell_lobby_title(),
    }
}

/// Wide quick-play button for the hero card; returns true on a click while
/// not already launching.
fn quick_play_button(ui: &mut Ui, lang: Language, launching: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(220.0, 46.0), Sense::click());
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

    let label = if launching {
        lang.bottom_btn_launching()
    } else {
        lang.shell_quick_play()
    };
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), theme::display(15.0), Color32::WHITE);
    let content_w = 17.0 + 10.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        Icon::Play,
        Rect::from_center_size(pos2(left + 8.5, rect.center().y), vec2(17.0, 17.0)),
        Color32::WHITE,
        1.5,
    );
    ui.painter().galley(
        pos2(left + 27.0, rect.center().y - galley.size().y / 2.0),
        galley,
        Color32::WHITE,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked() && !launching
}

/// Slim icon rail: page shortcuts on top, account chip pinned to the bottom.
pub fn show_nav(
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
            pos2(bar.right() - 0.5, bar.top()),
            pos2(bar.right() - 0.5, bar.bottom()),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.add_space(8.0);
    for (tab, icon) in NAV_TABS {
        let (row, _) = ui.allocate_exact_size(vec2(bar.width(), 42.0), Sense::hover());
        let rect = Rect::from_center_size(row.center(), vec2(40.0, 36.0));
        let resp = ui
            .interact(
                rect,
                ui.id().with(("millida_nav", tab as usize)),
                Sense::click(),
            )
            .on_hover_text(tab_label(tab, lang));
        let active = tab == current_tab;
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active || hover > 0.02 {
            let fill = if active {
                theme::with_alpha(theme::ACCENT, 55)
            } else {
                theme::with_alpha(theme::CARD_HOVER, (hover * 255.0) as u8)
            };
            ui.painter().rect_filled(rect, Rounding::ZERO, fill);
        }
        let glyph = if active {
            theme::HEAD
        } else {
            theme::lerp_color(theme::MUTED, theme::HEAD, hover)
        };
        icons::draw(
            ui.painter(),
            icon,
            rect.shrink2(vec2(11.0, 9.0)),
            glyph,
            1.45,
        );
        if resp.clicked() {
            action.tab = Some(tab);
        }
    }

    // Account chip: initial letter (or a person glyph while signed out).
    let remaining = ui.available_height() - 54.0;
    if remaining > 0.0 {
        ui.add_space(remaining);
    }
    let (chip, resp) = ui.allocate_exact_size(vec2(bar.width(), 46.0), Sense::click());
    let display = account_name.unwrap_or_else(|| lang.shell_nickname());
    let resp = resp.on_hover_text(display);
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let center = chip.center();
    ui.painter().circle_filled(
        center,
        14.0,
        theme::lerp_color(theme::ACCENT_DIM, theme::RUBY, hover),
    );
    ui.painter()
        .circle_stroke(center, 14.0, Stroke::new(1.0_f32, theme::BORDER_STRONG));
    match account_name.and_then(|name| name.chars().next()) {
        Some(letter) => {
            ui.painter().text(
                center,
                Align2::CENTER_CENTER,
                letter.to_uppercase(),
                theme::display(13.0),
                Color32::WHITE,
            );
        }
        None => {
            icons::draw(
                ui.painter(),
                Icon::Person,
                Rect::from_center_size(center, vec2(16.0, 16.0)),
                Color32::WHITE,
                1.4,
            );
        }
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        action.login = true;
    }

    action
}

/// The lobby (playhub) shown on the Home tab: a hero/continue card for the
/// selected instance and a row of hub tiles.
pub fn show_lobby(
    ui: &mut Ui,
    lang: Language,
    instance: Option<(&str, &str, &str)>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();

    ui.label(
        egui::RichText::new(lang.shell_lobby_title())
            .font(theme::display(20.0))
            .color(theme::HEAD),
    );
    ui.add_space(10.0);

    // Hero / continue card.
    let (hero_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 118.0), Sense::hover());
    ui.painter()
        .rect_filled(hero_rect, Rounding::ZERO, theme::CARD);
    ui.painter().rect_stroke(
        hero_rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::BORDER),
    );
    ui.painter().rect_filled(
        Rect::from_min_max(
            hero_rect.left_top(),
            pos2(hero_rect.left() + 3.0, hero_rect.bottom()),
        ),
        Rounding::ZERO,
        theme::RUBY,
    );

    let inner = hero_rect.shrink2(vec2(20.0, 14.0));
    let mut hero = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    match instance {
        Some((name, version, loader)) => {
            ui.painter().text(
                pos2(inner.left(), inner.center().y - 9.0),
                Align2::LEFT_CENTER,
                short_name(name, 34),
                theme::display_reg(21.0),
                theme::TEXT,
            );
            ui.painter().text(
                pos2(inner.left(), inner.center().y + 13.0),
                Align2::LEFT_CENTER,
                format!("MC {version} • {loader}"),
                theme::mono(11.0),
                theme::MUTED,
            );
            hero.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if quick_play_button(ui, lang, launching) {
                    action.launch = true;
                }
            });
        }
        None => {
            if icons::icon_text_button(
                &mut hero,
                Icon::Package,
                None,
                lang.shell_add_instance(),
                theme::MUTED,
                theme::mono(12.5),
                theme::ACCENT_SOFT,
            )
            .clicked()
            {
                action.tab = Some(NavTab::Modpacks);
            }
        }
    }

    // Hub tiles.
    ui.add_space(16.0);
    let gap = 12.0;
    let count = TILE_TARGETS.len() as f32;
    let tile_w = ((ui.available_width() - gap * (count - 1.0)) / count).max(90.0);
    let tile_h = 92.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (index, (icon, target)) in TILE_TARGETS.iter().enumerate() {
            let (rect, resp) = ui.allocate_exact_size(vec2(tile_w, tile_h), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            ui.painter().rect_filled(
                rect,
                Rounding::ZERO,
                theme::lerp_color(theme::CARD, theme::CARD_HOVER, hover),
            );
            let stroke = if hover > 0.5 {
                theme::BORDER_STRONG
            } else {
                theme::BORDER
            };
            ui.painter()
                .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, stroke));
            icons::draw(
                ui.painter(),
                *icon,
                Rect::from_center_size(pos2(rect.center().x, rect.top() + 34.0), vec2(22.0, 22.0)),
                theme::lerp_color(theme::MUTED, theme::HEAD, hover),
                1.5,
            );
            let galley = ui.painter().layout_no_wrap(
                tile_label(index, lang).to_owned(),
                theme::mono(10.5),
                Color32::PLACEHOLDER,
            );
            ui.painter().galley(
                pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.bottom() - 22.0 - galley.size().y / 2.0,
                ),
                galley,
                theme::lerp_color(theme::MUTED, theme::TEXT, hover),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                match target {
                    Some(tab) => action.tab = Some(*tab),
                    None if !launching => action.launch = true,
                    None => {}
                }
            }
        }
    });

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_covers_every_page_once() {
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
                "{tab:?} must appear exactly once in the rail"
            );
        }
    }

    #[test]
    fn test_tiles_offer_play_and_versions() {
        let targets: Vec<Option<NavTab>> = TILE_TARGETS.iter().map(|(_, t)| *t).collect();
        assert!(
            targets.contains(&None),
            "the quick-play tile (no target = launch) is missing"
        );
        assert!(
            targets.contains(&Some(NavTab::Modpacks)),
            "the versions/instances tile is missing"
        );
        assert_eq!(targets.len(), 4, "tile labels are indexed by position");
    }
}
