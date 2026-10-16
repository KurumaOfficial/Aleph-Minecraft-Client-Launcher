//! MultiMC / Prism Launcher-inspired shell (layout only).
//!
//! Credits:
//! - MultiMC Launcher — MultiMC Contributors, Apache-2.0 —
//!   <https://github.com/MultiMC/Launcher>
//! - Prism Launcher — PrismLauncher Contributors, GPL-3.0 —
//!   <https://github.com/PrismLauncher/PrismLauncher>
//!
//! Layout reference (from `launcher/ui/MainWindow.ui`): a main toolbar on
//! top (add instance / folders / settings / accounts on the left, launch on
//! the right), the instance list as the central widget, and a status bar at
//! the bottom carrying status text and download progress.
//!
//! AMC mapping: the central widget stays the shared pages (this mode opens
//! on the instance list), the toolbar carries the page shortcuts plus the
//! account chip and Launch button, and the status bar repeats the ready
//! state and the launcher version.

use crate::shells::{tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Rounding, Sense, Stroke, Ui};

/// Toolbar height (Prism: QToolBar-style 40px strip).
pub const TOOLBAR_HEIGHT: f32 = 40.0;
/// Status bar height (Prism: QStatusBar 24px strip).
pub const STATUS_HEIGHT: f32 = 24.0;
const PAD: f32 = 8.0;

/// Page shortcuts shown as toolbar icons. `Add instance` opens the instance
/// list (see `test_toolbar_covers_every_tab`), so every `NavTab` stays
/// reachable from this shell.
pub const TOOLBAR_TABS: [(NavTab, Icon); 5] = [
    (NavTab::Home, Icon::Home),
    (NavTab::Mods, Icon::Puzzle),
    (NavTab::Skins, Icon::Diamond),
    (NavTab::Profile, Icon::Person),
    (NavTab::Settings, Icon::Gear),
];

/// Vertical hairline between toolbar groups.
fn toolbar_separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(13.0, 18.0), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(rect.center().x, rect.top()),
            pos2(rect.center().x, rect.bottom()),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );
}

/// Main toolbar: instance-list shortcut and page shortcuts on the left,
/// account chip and Launch on the right (the reference's instanceToolBar).
pub fn show_toolbar(
    ui: &mut Ui,
    lang: Language,
    current_tab: NavTab,
    account_name: Option<&str>,
    launching: bool,
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

        // Add instance: the prominent action of the reference toolbar. It
        // opens the shared instance list tab.
        if icons::icon_text_button(
            ui,
            Icon::Package,
            None,
            lang.shell_add_instance(),
            theme::TEXT,
            theme::mono(11.5),
            theme::ACCENT_SOFT,
        )
        .on_hover_text(tab_label(NavTab::Modpacks, lang))
        .clicked()
        {
            action.tab = Some(NavTab::Modpacks);
        }

        toolbar_separator(ui);

        // Page shortcuts (icon-only, tooltips carry the labels).
        for (tab, icon) in TOOLBAR_TABS {
            let active = tab == current_tab;
            let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 26.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let fill = if active {
                theme::with_alpha(theme::ACCENT, 55)
            } else {
                theme::with_alpha(theme::CARD_HOVER, (hover * 255.0) as u8)
            };
            ui.painter().rect_filled(rect, Rounding::ZERO, fill);
            let glyph = if active {
                theme::HEAD
            } else {
                theme::lerp_color(theme::MUTED, theme::HEAD, hover)
            };
            icons::draw(ui.painter(), icon, rect.shrink2(vec2(8.0, 6.0)), glyph, 1.4);
            let resp = resp.on_hover_text(tab_label(tab, lang));
            if resp.clicked() {
                action.tab = Some(tab);
            }
        }

        // Right side: Launch (instanceToolBar position) + account chip.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD);
            let launch_label = if launching {
                lang.bottom_btn_launching()
            } else {
                lang.shell_launch_instance()
            };
            let launch_fill = if launching {
                theme::ACCENT_DIM
            } else {
                theme::RUBY
            };
            if icons::icon_text_button(
                ui,
                Icon::Play,
                None,
                launch_label,
                egui::Color32::WHITE,
                theme::mono(11.5),
                launch_fill,
            )
            .clicked()
                && !launching
            {
                action.launch = true;
            }

            ui.add_space(6.0);

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

/// Status bar: ready state on the left, launcher version on the right.
pub fn show_status_bar(ui: &mut Ui, lang: Language, version: &str) {
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
        ui.add_space(PAD + 2.0);
        ui.label(
            egui::RichText::new(lang.status_ready())
                .font(theme::mono(10.5))
                .color(theme::MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD + 2.0);
            ui.label(
                egui::RichText::new(format!("{} {}", lang.shell_version(), version))
                    .font(theme::mono(10.5))
                    .color(theme::MUTED),
            );
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_covers_every_tab() {
        let mut seen: Vec<NavTab> = TOOLBAR_TABS.iter().map(|(tab, _)| *tab).collect();
        // The Add instance button opens the instance list.
        seen.push(NavTab::Modpacks);
        for tab in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Profile,
            NavTab::Settings,
        ] {
            assert!(
                seen.contains(&tab),
                "{tab:?} is unreachable from the toolbar"
            );
        }
    }
}
