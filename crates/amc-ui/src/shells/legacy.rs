//! Legacy Minecraft Launcher-inspired shell (layout only).
//!
//! Credits:
//! - Legacy Minecraft Launcher (Mojang Studios, proprietary): the layout is
//!   reconstructed from public screenshots only; no assets or code reused.
//!
//! Layout reference: a strip of classic folder tabs on top (the active tab
//! is a white sheet merged with the page, the launcher wordmark fills the
//! logo slot on the right), a bottom bar with the "Profile" / "Version"
//! comboboxes and the green Play button, and a white page canvas between
//! them. Metrics follow the 2013 launcher: 32px tabs, 24px comboboxes,
//! 48px bottom bar.

use crate::shells::{short_name, tab_label, ShellAction};
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::types::Instance;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Response, Rounding, ScrollArea, Sense, Stroke, Ui};
use uuid::Uuid;

/// Tab strip height (reference: folder tabs above the white page).
pub const TABS_HEIGHT: f32 = 32.0;
/// Bottom bar height (reference: profile/version combos + Play).
pub const PLAY_HEIGHT: f32 = 48.0;
const PAD: f32 = 12.0;
const COMBO_H: f32 = 24.0;
/// Selection blue of the native menus of the era.
const MENU_SELECTION: Color32 = Color32::from_rgb(0x31, 0x6A, 0xC5);

/// Tabs in the reference order: content first, settings last.
pub const TABS: [NavTab; 6] = [
    NavTab::Home,
    NavTab::Modpacks,
    NavTab::Mods,
    NavTab::Skins,
    NavTab::Profile,
    NavTab::Settings,
];

/// Ambient data of the bottom bar.
pub struct PlayBarCtx<'a> {
    /// Instances listed by the reference "Profile" combobox.
    pub instances: &'a [Instance],
    /// Currently selected instance.
    pub selected: Option<Uuid>,
}

/// What the profile combobox menu returned.
enum ComboPick {
    /// Existing instance by index.
    Entry(usize),
    /// "New profile…" entry.
    New,
}

/// Top strip: classic folder tabs; the active one is a white sheet that
/// merges with the page canvas below, the rest sit flat on the window gray.
pub fn show_tabs(ui: &mut Ui, lang: Language, current_tab: NavTab) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, Rounding::ZERO, theme::BG);
    // Baseline under the inactive tabs; the active sheet paints over it.
    painter.line_segment(
        [
            pos2(bar.left(), bar.bottom() - 0.5),
            pos2(bar.right(), bar.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
        for tab in TABS {
            let active = tab == current_tab;
            let label = tab_label(tab, lang);
            let galley = ui.painter().layout_no_wrap(
                label.to_owned(),
                egui::FontId::proportional(12.5),
                Color32::PLACEHOLDER,
            );
            let (rect, resp) =
                ui.allocate_exact_size(vec2(galley.size().x + 26.0, bar.height()), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let sheet = Rect::from_min_max(
                pos2(rect.left(), rect.top() + 4.0),
                pos2(rect.right(), rect.bottom()),
            );
            if active {
                let rounding = Rounding {
                    nw: 2.0,
                    ne: 2.0,
                    sw: 0.0,
                    se: 0.0,
                };
                ui.painter().rect_filled(sheet, rounding, theme::FIELD);
                ui.painter()
                    .rect_stroke(sheet, rounding, Stroke::new(1.0_f32, theme::BORDER));
                // Erase the baseline under the sheet so it merges with the
                // white page canvas.
                ui.painter().line_segment(
                    [
                        pos2(sheet.left() + 1.0, sheet.bottom() - 0.5),
                        pos2(sheet.right() - 1.0, sheet.bottom() - 0.5),
                    ],
                    Stroke::new(1.0_f32, theme::FIELD),
                );
            } else if hover > 0.02 {
                ui.painter().rect_filled(
                    sheet,
                    Rounding::ZERO,
                    theme::with_alpha(theme::FIELD, (hover * 80.0) as u8),
                );
            }
            let color = if active {
                theme::HEAD.into()
            } else {
                theme::lerp_color(theme::MUTED, theme::TEXT, hover)
            };
            let text_y = sheet.top() + (sheet.height() - galley.size().y) / 2.0;
            ui.painter().galley(
                pos2(rect.center().x - galley.size().x / 2.0, text_y),
                galley,
                color,
            );
            if resp.clicked() {
                action.tab = Some(tab);
            }
        }

        // Launcher wordmark in the reference logo slot.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(PAD);
            let galley = ui.painter().layout_no_wrap(
                "ALEPH".to_owned(),
                theme::display(16.0),
                Color32::PLACEHOLDER,
            );
            let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
            ui.painter().galley(
                pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                galley,
                theme::HEAD.into(),
            );
        });
    });

    action
}

/// Classic combobox of the era: white field, 1px border, a right cell with
/// the drop-down triangle; returns the field rect and its response.
fn combo_frame(ui: &mut Ui, text: &str, width: f32, leading: Option<Icon>) -> (Rect, Response) {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, COMBO_H), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::ZERO, theme::FIELD);
    painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, theme::BORDER));
    let arrow = Rect::from_min_max(
        pos2(rect.right() - 19.0, rect.top() + 1.0),
        pos2(rect.right() - 1.0, rect.bottom() - 1.0),
    );
    painter.line_segment(
        [
            pos2(arrow.left(), rect.top() + 1.0),
            pos2(arrow.left(), rect.bottom() - 1.0),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );
    let c = arrow.center();
    painter.add(egui::Shape::convex_polygon(
        vec![
            pos2(c.x - 4.0, c.y - 2.0),
            pos2(c.x + 4.0, c.y - 2.0),
            pos2(c.x, c.y + 3.5),
        ],
        theme::TEXT,
        Stroke::NONE,
    ));

    let mut x = rect.left() + 7.0;
    if let Some(icon) = leading {
        icons::draw(
            painter,
            icon,
            Rect::from_center_size(pos2(x + 8.0, rect.center().y), vec2(14.0, 14.0)),
            theme::MUTED,
            1.5,
        );
        x += 20.0;
    }
    let avail = (arrow.left() - x - 4.0).max(10.0);
    let max_chars = ((avail / 6.2).floor() as usize).max(4);
    let galley = ui.painter().layout_no_wrap(
        short_name(text, max_chars),
        egui::FontId::proportional(12.5),
        Color32::PLACEHOLDER,
    );
    ui.painter().galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        theme::TEXT.into(),
    );
    (rect, resp)
}

/// Profile combobox: lists the instances like the reference profile picker
/// (checked row = current profile) plus a "New profile…" entry; the popup
/// opens upward because the combobox sits in the bottom bar.
fn profile_combo(ui: &mut Ui, ctx: &PlayBarCtx<'_>, lang: Language) -> Option<ComboPick> {
    let selected = ctx
        .selected
        .and_then(|id| ctx.instances.iter().find(|i| i.id == id));
    let text = selected
        .map(|i| i.name.as_str())
        .unwrap_or_else(|| lang.shell_no_profile());
    let (rect, resp) = combo_frame(ui, text, 210.0, Some(Icon::Person));

    let open_id = resp.id.with("open");
    let mut open: bool = ui.ctx().data_mut(|d| *d.get_temp_mut_or(open_id, false));
    if resp.clicked() {
        open = !open;
    }
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    let mut picked: Option<ComboPick> = None;
    if open {
        let font = egui::FontId::proportional(12.5);
        let width = 250.0;
        let list_h = (ctx.instances.len() as f32 * 22.0).min(264.0);
        let menu_h = list_h + 7.0 + 22.0 + 2.0;
        let area = egui::Area::new(resp.id.with("menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(rect.left(), rect.top() - menu_h - 1.0))
            .interactable(true);
        let menu = area.show(ui.ctx(), |ui| {
            egui::Frame::none()
                .fill(theme::FIELD.into())
                .stroke(Stroke::new(1.0_f32, theme::BORDER_STRONG))
                .show(ui, |ui| {
                    ui.set_min_width(width);
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    ScrollArea::vertical()
                        .max_height(list_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (idx, inst) in ctx.instances.iter().enumerate() {
                                let (row, rr) =
                                    ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
                                let hovered = rr.hovered();
                                if hovered {
                                    ui.painter()
                                        .rect_filled(row, Rounding::ZERO, MENU_SELECTION);
                                }
                                let fg: Color32 = if hovered {
                                    Color32::WHITE
                                } else {
                                    theme::TEXT.into()
                                };
                                if ctx.selected == Some(inst.id) {
                                    icons::draw(
                                        ui.painter(),
                                        Icon::Check,
                                        Rect::from_center_size(
                                            pos2(row.left() + 9.0, row.center().y),
                                            vec2(10.0, 10.0),
                                        ),
                                        fg,
                                        1.6,
                                    );
                                }
                                ui.painter().text(
                                    pos2(row.left() + 20.0, row.center().y),
                                    Align2::LEFT_CENTER,
                                    &inst.name,
                                    font.clone(),
                                    fg,
                                );
                                ui.painter().text(
                                    pos2(row.right() - 8.0, row.center().y),
                                    Align2::RIGHT_CENTER,
                                    &inst.game_version,
                                    egui::FontId::proportional(11.0),
                                    if hovered { fg } else { theme::MUTED.into() },
                                );
                                if rr.clicked() {
                                    picked = Some(ComboPick::Entry(idx));
                                }
                            }
                        });
                    let (sep, _) = ui.allocate_exact_size(vec2(width, 7.0), Sense::hover());
                    ui.painter().line_segment(
                        [
                            pos2(sep.left() + 4.0, sep.center().y),
                            pos2(sep.right() - 4.0, sep.center().y),
                        ],
                        Stroke::new(1.0_f32, theme::BORDER_SOFT),
                    );
                    let (row, rr) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
                    let hovered = rr.hovered();
                    if hovered {
                        ui.painter()
                            .rect_filled(row, Rounding::ZERO, MENU_SELECTION);
                    }
                    let fg: Color32 = if hovered {
                        Color32::WHITE
                    } else {
                        theme::TEXT.into()
                    };
                    ui.painter().text(
                        pos2(row.left() + 20.0, row.center().y),
                        Align2::LEFT_CENTER,
                        lang.shell_new_profile(),
                        font.clone(),
                        fg,
                    );
                    if rr.clicked() {
                        picked = Some(ComboPick::New);
                    }
                });
        });
        if picked.is_some() {
            open = false;
        }
        if ui.ctx().input(|i| i.pointer.any_click()) {
            let pos = ui.ctx().input(|i| i.pointer.interact_pos());
            let inside =
                pos.is_some_and(|p| resp.rect.contains(p) || menu.response.rect.contains(p));
            if !inside {
                open = false;
            }
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(open_id, open));
    picked
}

/// Bottom bar: "Profile" + combobox, "Version" + combobox, Play on the
/// right — the reference's picker row.
pub fn show_play_bar(
    ui: &mut Ui,
    lang: Language,
    ctx: &PlayBarCtx<'_>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, Rounding::ZERO, theme::BG);
    painter.line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        ui.add_space(PAD);

        bar_label(ui, lang.shell_profile_label());
        if let Some(pick) = profile_combo(ui, ctx, lang) {
            match pick {
                ComboPick::Entry(idx) => action.select_instance = Some(ctx.instances[idx].id),
                ComboPick::New => action.new_instance = true,
            }
        }

        ui.add_space(12.0);
        bar_label(ui, lang.shell_version_label());
        let selected = ctx
            .selected
            .and_then(|id| ctx.instances.iter().find(|i| i.id == id));
        let version = selected
            .map(|i| format!("{} ({})", i.game_version, i.loader.as_str()))
            .unwrap_or_else(|| lang.shell_use_latest().to_string());
        let (_, version_resp) = combo_frame(ui, &version, 168.0, None);
        if version_resp.clicked() {
            action.tab = Some(NavTab::Modpacks);
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

/// Small "Profile:" / "Version:" label with its colon.
fn bar_label(ui: &mut Ui, text: &str) {
    let galley = ui.painter().layout_no_wrap(
        format!("{text}:"),
        egui::FontId::proportional(12.5),
        Color32::PLACEHOLDER,
    );
    let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(
        pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
        galley,
        theme::TEXT.into(),
    );
}

/// The classic green Play button (flat fill, 1px darker border, white
/// label, no icon).
fn play_button(ui: &mut Ui, label: &str, launching: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(132.0, 30.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill: Color32 = if launching {
        theme::ACCENT_DIM.into()
    } else {
        theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover * 0.55)
    };
    ui.painter()
        .rect_filled(rect, crate::theme::rounding(), fill);
    ui.painter().rect_stroke(
        rect,
        crate::theme::rounding(),
        Stroke::new(1.0_f32, theme::ACCENT_DIM),
    );
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(14.0),
        Color32::PLACEHOLDER,
    );
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        Color32::WHITE,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked() && !launching
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
