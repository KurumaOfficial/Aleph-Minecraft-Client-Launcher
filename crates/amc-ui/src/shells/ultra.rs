//! Aleph Ultra — the built-in minimal shell (no third-party origin).
//!
//! One screen and nothing else: nickname, version, Play. Navigation lives in
//! a slim centered icon dock at the bottom instead of tabs, so every shared
//! page stays reachable without bringing back the full chrome. AMC's own
//! design language is kept (sharp corners, ruby accent, display/mono faces).

use crate::shells::{tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, Ui};

/// Dock height (slim strip, no text labels).
pub const DOCK_HEIGHT: f32 = 46.0;

/// Dock shortcuts in display order: Home is the Ultra screen itself.
pub const DOCK_TABS: [(NavTab, Icon); 6] = [
    (NavTab::Home, Icon::Home),
    (NavTab::Modpacks, Icon::Package),
    (NavTab::Mods, Icon::Puzzle),
    (NavTab::Skins, Icon::Diamond),
    (NavTab::Profile, Icon::Person),
    (NavTab::Settings, Icon::Gear),
];

/// Wide hero Play button; returns true when a launch was requested.
fn hero_play_button(ui: &mut Ui, lang: Language, launching: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(260.0, 54.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let base = if launching {
        theme::ACCENT_DIM
    } else {
        theme::RUBY
    };
    let fill = theme::lerp_color(base, theme::ACCENT_LIGHT, hover * 0.6);
    ui.painter()
        .rect_filled(rect, crate::theme::rounding(), fill);
    ui.painter().rect_stroke(
        rect,
        crate::theme::rounding(),
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );

    let label = if launching {
        lang.bottom_btn_launching()
    } else {
        lang.bottom_btn_play()
    };
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), theme::display(16.0), Color32::WHITE);
    let content_w = 20.0 + 10.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        Icon::Play,
        Rect::from_center_size(pos2(left + 10.0, rect.center().y), vec2(20.0, 20.0)),
        Color32::WHITE,
        1.6,
    );
    ui.painter().galley(
        pos2(left + 30.0, rect.center().y - galley.size().y / 2.0),
        galley,
        Color32::WHITE,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked() && !launching
}

/// The Ultra screen (shown on the Home tab): wordmark, account, selected
/// instance and the big Play button — nothing else.
pub fn show_screen(
    ui: &mut Ui,
    lang: Language,
    account_name: Option<&str>,
    instance: Option<(&str, &str, &str)>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let rect = ui.available_rect_before_wrap();
    let mut hero = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );

    hero.add_space((rect.height() * 0.18).max(24.0));
    hero.label(
        egui::RichText::new("ALEPH LAUNCHER")
            .font(theme::display(26.0))
            .color(theme::HEAD),
    );
    hero.add_space(8.0);

    let display = account_name.unwrap_or_else(|| lang.shell_nickname());
    if icons::icon_text_button(
        &mut hero,
        Icon::Person,
        None,
        display,
        theme::TEXT,
        theme::mono(12.5),
        theme::FIELD,
    )
    .clicked()
    {
        action.login = true;
    }

    hero.add_space(16.0);
    match instance {
        Some((name, version, loader)) => {
            hero.label(
                egui::RichText::new(name)
                    .font(theme::display_reg(16.0))
                    .color(theme::TEXT),
            );
            hero.add_space(2.0);
            hero.label(
                egui::RichText::new(format!("MC {version} • {loader}"))
                    .font(theme::mono(11.0))
                    .color(theme::MUTED),
            );
        }
        None => {
            if icons::icon_text_button(
                &mut hero,
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

    hero.add_space(24.0);
    if hero_play_button(&mut hero, lang, launching) {
        action.launch = true;
    }
    hero.add_space(10.0);
    hero.label(
        egui::RichText::new(format!(
            "{} {}",
            lang.shell_version(),
            env!("CARGO_PKG_VERSION")
        ))
        .font(theme::mono(10.0))
        .color(theme::MUTED_DIM),
    );

    action
}

/// Slim centered icon dock: navigation for every shared page.
pub fn show_dock(ui: &mut Ui, lang: Language, current_tab: NavTab) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    ui.painter()
        .rect_filled(bar, crate::theme::rounding(), theme::BG);
    ui.painter().line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let item = vec2(36.0, 32.0);
    let spacing = 6.0;
    let total = DOCK_TABS.len() as f32 * item.x + (DOCK_TABS.len() - 1) as f32 * spacing;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = spacing;
        ui.add_space(((bar.width() - total) / 2.0).max(0.0));
        for (tab, icon) in DOCK_TABS {
            let active = tab == current_tab;
            let (rect, resp) = ui.allocate_exact_size(item, Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let fill = if active {
                theme::with_alpha(theme::ACCENT, 55)
            } else {
                theme::with_alpha(theme::CARD_HOVER, (hover * 255.0) as u8)
            };
            ui.painter()
                .rect_filled(rect, crate::theme::rounding(), fill);
            let glyph = if active {
                theme::HEAD.into()
            } else {
                theme::lerp_color(theme::MUTED, theme::HEAD, hover)
            };
            icons::draw(
                ui.painter(),
                icon,
                rect.shrink2(vec2(10.0, 7.0)),
                glyph,
                1.45,
            );
            if resp.on_hover_text(tab_label(tab, lang)).clicked() {
                action.tab = Some(tab);
            }
        }
    });

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_covers_every_tab() {
        for tab in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Profile,
            NavTab::Settings,
        ] {
            assert!(
                DOCK_TABS.iter().any(|(dock_tab, _)| *dock_tab == tab),
                "{tab:?} is unreachable from the dock"
            );
        }
    }
}
