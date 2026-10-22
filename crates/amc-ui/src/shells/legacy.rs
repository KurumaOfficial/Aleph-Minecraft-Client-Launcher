//! Legacy Minecraft Launcher-inspired shell (layout only).
//!
//! Credits:
//! - Legacy Minecraft Launcher (Mojang Studios, proprietary): the layout is
//!   reconstructed from public screenshots only; no assets or code reused.
//!
//! Layout reference (1.6.94): a light text-menu strip on top (Update Notes /
//! Launcher Log / Profile Editor in the original), the news canvas in the
//! middle, and a bottom bar with the profile picker, New/Edit Profile on the
//! left, a large white Play button in the centre and the account welcome plus
//! Switch User on the right.

use crate::shells::{short_name, tab_label, ShellAction};
use crate::theme;
use crate::widgets::brand;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::types::Instance;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, ScrollArea, Sense, Stroke, Ui};
use uuid::Uuid;

/// Top text-menu strip height (reference: a slim light-gray menu bar).
pub const TABS_HEIGHT: f32 = 30.0;
/// Bottom bar height (reference: profile picker + Play + account block).
pub const PLAY_HEIGHT: f32 = 48.0;
const PAD: f32 = 10.0;
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
    /// Launch version shown in the "Ready to play" line.
    pub version: Option<&'a str>,
    /// Active account name shown in the welcome line.
    pub account: Option<&'a str>,
}

/// What the profile combobox menu returned.
enum ComboPick {
    /// Existing instance by index.
    Entry(usize),
    /// "New profile…" entry.
    New,
}

/// Top strip: plain text menu links, exactly like the reference's
/// Update Notes / Launcher Log / Profile Editor row, plus the AMC identity
/// in the logo slot on the right.
pub fn show_tabs(ui: &mut Ui, lang: Language, current_tab: NavTab) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, Rounding::ZERO, theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left(), bar.bottom() - 0.5),
            pos2(bar.right(), bar.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let mut x = bar.left() + PAD;
    for (idx, tab) in TABS.into_iter().enumerate() {
        let active = tab == current_tab;
        let label = tab_label(tab, lang);
        let galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(12.5),
            Color32::PLACEHOLDER,
        );
        let rect = Rect::from_min_size(
            pos2(x, bar.top()),
            vec2(galley.size().x + 20.0, bar.height()),
        );
        let resp = ui.interact(rect, ui.id().with(("legacy_menu", idx)), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active {
            painter.rect_filled(
                Rect::from_min_max(
                    pos2(rect.left() + 10.0, rect.bottom() - 2.0),
                    pos2(rect.right() - 10.0, rect.bottom()),
                ),
                Rounding::ZERO,
                theme::ACCENT,
            );
        }
        let color = if active {
            theme::HEAD.into()
        } else {
            theme::lerp_color(theme::TEXT, theme::INFO, hover)
        };
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.5),
            color,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            action.tab = Some(tab);
        }
        x = rect.right();
    }

    // AMC identity in the reference logo slot.
    let galley = ui.painter().layout_no_wrap(
        brand::APP_NAME_UPPER.to_owned(),
        theme::display(13.0),
        Color32::PLACEHOLDER,
    );
    let right = bar.right() - PAD;
    let text_left = right - galley.size().x;
    let mark = Rect::from_min_size(
        pos2(text_left - 19.0, bar.center().y - 5.0),
        vec2(10.0, 10.0),
    );
    brand::hollow_square(ui.painter(), mark, brand::BRAND_RED);
    ui.painter().galley(
        pos2(text_left, bar.center().y - galley.size().y / 2.0),
        galley,
        theme::HEAD.into(),
    );

    action
}

/// Paints the classic combobox of the era: white field, 1px border, a right
/// cell with the drop-down triangle.
fn paint_combo(painter: &egui::Painter, rect: Rect, text: &str, leading: Option<Icon>) {
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
    let galley = painter.layout_no_wrap(
        short_name(text, max_chars),
        egui::FontId::proportional(12.5),
        Color32::PLACEHOLDER,
    );
    painter.galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        theme::TEXT.into(),
    );
}

/// Profile combobox: lists the instances like the reference profile picker
/// (checked row = current profile) plus a "New profile…" entry; the popup
/// opens upward because the combobox sits in the bottom bar.
fn profile_combo_at(
    ui: &mut Ui,
    rect: Rect,
    ctx: &PlayBarCtx<'_>,
    lang: Language,
) -> Option<ComboPick> {
    let selected = ctx
        .selected
        .and_then(|id| ctx.instances.iter().find(|i| i.id == id));
    let text = selected
        .map(|i| i.name.as_str())
        .unwrap_or_else(|| lang.shell_no_profile());
    paint_combo(ui.painter(), rect, text, Some(Icon::Person));

    let resp = ui.interact(rect, ui.id().with("legacy_profile_combo"), Sense::click());
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
            let inside = pos.is_some_and(|p| rect.contains(p) || menu.response.rect.contains(p));
            if !inside {
                open = false;
            }
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(open_id, open));
    picked
}

/// Bottom bar: profile picker and New/Edit Profile on the left, the large
/// white Play button in the centre, and the account welcome plus Switch User
/// on the right — the 1.6.94 composition.
pub fn show_play_bar(
    ui: &mut Ui,
    lang: Language,
    ctx: &PlayBarCtx<'_>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, Rounding::ZERO, theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let cy = bar.center().y;
    let left = bar.left() + PAD;
    let profile_w = (bar.width() * 0.14).clamp(130.0, 190.0);
    let profile_rect =
        Rect::from_min_size(pos2(left, cy - COMBO_H / 2.0), vec2(profile_w, COMBO_H));
    if let Some(pick) = profile_combo_at(ui, profile_rect, ctx, lang) {
        match pick {
            ComboPick::Entry(idx) => action.select_instance = Some(ctx.instances[idx].id),
            ComboPick::New => action.new_instance = true,
        }
    }

    let gap = 5.0;
    let new_label = lang.legacy_new_profile();
    let new_rect = Rect::from_min_size(
        pos2(profile_rect.right() + gap, cy - COMBO_H / 2.0),
        vec2(bar_button_width(ui, new_label).max(84.0), COMBO_H),
    );
    if bar_button(ui, new_rect, new_label, true) {
        action.new_instance = true;
    }
    let edit_label = lang.shell_edit_profile();
    let edit_rect = Rect::from_min_size(
        pos2(new_rect.right() + gap, cy - COMBO_H / 2.0),
        vec2(bar_button_width(ui, edit_label).max(92.0), COMBO_H),
    );
    if bar_button(ui, edit_rect, edit_label, ctx.selected.is_some()) && ctx.selected.is_some() {
        action.edit_instance = true;
    }

    let play_w = (bar.width() * 0.22).clamp(190.0, 280.0);
    let play_rect = Rect::from_center_size(pos2(bar.center().x, cy), vec2(play_w, 32.0));
    let play_label = if launching {
        lang.legacy_launching()
    } else {
        lang.legacy_play()
    };
    if play_button_at(ui, play_rect, play_label, launching) {
        action.launch = true;
    }

    let switch_label = lang.legacy_switch_user();
    let switch_w = bar_button_width(ui, switch_label).max(104.0);
    let switch_rect = Rect::from_min_size(
        pos2(bar.right() - PAD - switch_w, cy - COMBO_H / 2.0),
        vec2(switch_w, COMBO_H),
    );
    if bar_button(ui, switch_rect, switch_label, true) {
        action.login = true;
    }

    // Welcome block sits just left of Switch User. It disappears on very
    // narrow windows rather than colliding with the centred Play button.
    let text_right = switch_rect.left() - 10.0;
    if text_right > play_rect.right() + 12.0 {
        let account = ctx.account.unwrap_or_else(|| lang.legacy_guest());
        let welcome = lang.legacy_welcome(account);
        let version = ctx.version.unwrap_or_else(|| lang.shell_use_latest());
        let ready = lang.legacy_ready(version);
        let ready = short_name(&ready, 42);
        ui.painter().text(
            pos2(text_right, cy - 8.0),
            Align2::RIGHT_CENTER,
            short_name(&welcome, 30),
            egui::FontId::proportional(11.5),
            theme::HEAD.into(),
        );
        ui.painter().text(
            pos2(text_right, cy + 8.0),
            Align2::RIGHT_CENTER,
            ready,
            egui::FontId::proportional(10.0),
            theme::MUTED.into(),
        );
    }

    action
}

fn bar_button_width(ui: &Ui, label: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(10.5),
            Color32::PLACEHOLDER,
        )
        .size()
        .x
        + 16.0
}

/// Flat classic button: white field, 1px border, dark label; disabled buttons
/// keep the frame but dim the text.
fn bar_button(ui: &mut Ui, rect: Rect, label: &str, enabled: bool) -> bool {
    let resp = ui.interact(
        rect,
        ui.id().with(("legacy_bar_button", label)),
        Sense::click(),
    );
    let hover = if enabled {
        ui.ctx().animate_bool_responsive(resp.id, resp.hovered())
    } else {
        0.0
    };
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::ZERO, theme::FIELD);
    painter.rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );
    if hover > 0.02 {
        painter.rect_filled(
            rect,
            Rounding::ZERO,
            theme::with_alpha(theme::BG_HOVER, (hover * 210.0) as u8),
        );
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.5),
        if enabled {
            theme::TEXT.into()
        } else {
            theme::MUTED_DIM.into()
        },
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    enabled && resp.clicked()
}

/// The reference's large white Play button: flat white fill, gray border,
/// dark label, no icon.
fn play_button_at(ui: &mut Ui, rect: Rect, label: &str, launching: bool) -> bool {
    let resp = ui.interact(rect, ui.id().with("legacy_play_button"), Sense::click());
    let enabled = !launching;
    let hover = if enabled {
        ui.ctx().animate_bool_responsive(resp.id, resp.hovered())
    } else {
        0.0
    };
    let painter = ui.painter();
    let fill: Color32 = if launching {
        theme::CARD.into()
    } else {
        theme::lerp_color(theme::FIELD, theme::BG_HOVER, hover * 0.7)
    };
    painter.rect_filled(rect, crate::theme::rounding(), fill);
    painter.rect_stroke(
        rect,
        crate::theme::rounding(),
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        if enabled {
            theme::HEAD.into()
        } else {
            theme::MUTED.into()
        },
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    enabled && resp.clicked()
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
