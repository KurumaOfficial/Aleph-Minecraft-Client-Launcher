//! MultiMC-inspired shell (layout only).
//!
//! Credits:
//! - MultiMC Launcher — MultiMC Contributors, Apache-2.0 —
//!   <https://github.com/MultiMC/Launcher>
//!
//! Layout reference (`launcher/ui/MainWindow.cpp`, `ui/instanceview/*`): a
//! text-beside-icon toolbar on top (Add Instance | Folders, Settings, Help,
//! Update | Support, Meow … Accounts), a text-only instance toolbar docked
//! on the right, the icon-grid instance view as the central widget, a news
//! strip above the status bar and permanent status labels on its right.
//! Metrics (toolbar row, 22px menu rows, 26px buttons, icon-grid cells)
//! follow the Qt widgets of the reference. Only layout ideas and public
//! color values are borrowed — no assets or code.

use crate::shells::ShellAction;
use crate::theme;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Response, Rounding, Sense, Stroke, Ui};
use std::path::Path;
use uuid::Uuid;

/// Main toolbar height (Qt toolbar with 16px icons, text beside them).
pub const TOOLBAR_HEIGHT: f32 = 32.0;
/// News strip height (Qt toolbar above the status bar).
pub const NEWS_HEIGHT: f32 = 24.0;
/// Status bar height.
pub const STATUS_HEIGHT: f32 = 22.0;
/// Width of the right instance toolbar (the reference sizes it to its
/// longest text-only label).
pub const INSTANCE_BAR_WIDTH: f32 = 136.0;

/// The highlighted-text role of the Qt "Bright" theme (`#eff0f1`).
const HIGHLIGHTED_TEXT: Color32 = Color32::from_rgb(0xEF, 0xF0, 0xF1);
/// Project links (our own repository, never the reference's pages).
const PROJECT_URL: &str = "https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher";

/// Ambient data of the main toolbar.
pub struct ToolbarCtx<'a> {
    /// All stored accounts (accounts dropdown of the reference).
    pub accounts: &'a [amc_auth::Account],
    /// Currently active account id.
    pub active_account: Option<Uuid>,
    /// True while the shared launch flow is running.
    pub launching: bool,
    /// Folder shown by "View Instance Folder".
    pub instances_dir: &'a Path,
    /// Folder shown by "View Central Mods Folder".
    pub central_mods_dir: &'a Path,
}

/// Selected instance shown in the right instance toolbar.
pub struct InstanceSel {
    /// Instance display name (the name button renames from it).
    pub name: String,
    /// Instance icon stand-in (preset/loader badge of the instance).
    pub badge: Icon,
    /// Color of that badge.
    pub badge_color: Color32,
    /// The instance folder (Minecraft / Config / Instance Folder buttons).
    pub game_dir: std::path::PathBuf,
}

// ============================================================
// Main toolbar
// ============================================================

/// One text-beside-icon toolbar button (Qt `ToolButtonTextBesideIcon`):
/// 16px icon, 12px label, hover tint, optional dropdown caret. `checked`
/// paints the sunken state of a checkable tool button.
fn tb_button(ui: &mut Ui, icon: Icon, label: &str, menu: bool, checked: bool) -> Response {
    let font = egui::FontId::proportional(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let width = 14.0 + 16.0 + 6.0 + galley.size().x + 10.0 + if menu { 11.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 26.0), Sense::click());
    let painter = ui.painter();
    let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
    if checked {
        painter.rect_filled(rect, Rounding::ZERO, theme::BORDER_SOFT);
        painter.rect_stroke(
            rect,
            Rounding::ZERO,
            Stroke::new(1.0_f32, theme::BORDER_DEFAULT),
        );
    } else if hover > 0.02 {
        painter.rect_filled(
            rect,
            Rounding::ZERO,
            theme::with_alpha(theme::ACCENT, (hover * 56.0) as u8),
        );
    }
    icons::draw(
        painter,
        icon,
        Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(16.0, 16.0)),
        theme::TEXT,
        1.5,
    );
    painter.galley(
        pos2(rect.left() + 34.0, rect.center().y - galley.size().y / 2.0),
        galley,
        theme::TEXT.into(),
    );
    if menu {
        icons::draw(
            painter,
            Icon::CaretDown,
            Rect::from_center_size(pos2(rect.right() - 10.0, rect.center().y), vec2(9.0, 9.0)),
            theme::TEXT,
            1.3,
        );
    }
    resp
}

/// Vertical toolbar separator (Qt `QToolBar::addSeparator`).
fn tb_separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 18.0), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(rect.center().x, rect.top()),
            pos2(rect.center().x, rect.bottom()),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );
}

/// One entry of a Qt-style dropdown menu.
enum QtItem<'a> {
    /// A separator line between groups.
    Separator,
    /// A clickable row (with an optional check mark and icon).
    Entry {
        label: &'a str,
        icon: Option<Icon>,
        checked: bool,
    },
}

/// Qt-style dropdown anchored under `button` (the reference's instant-popup
/// tool buttons). Open state lives in egui memory; returns the clicked
/// entry index (separators not counted).
fn qt_dropdown(ctx: &egui::Context, button: &Response, items: &[QtItem<'_>]) -> Option<usize> {
    let open_id = button.id.with("qt_open");
    let mut open: bool = ctx.data_mut(|d| *d.get_temp_mut_or(open_id, false));
    if button.clicked() {
        open = !open;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    let mut clicked: Option<usize> = None;
    if open {
        let font = egui::FontId::proportional(12.0);
        let widest = items.iter().fold(0.0_f32, |w, item| match item {
            QtItem::Separator => w,
            QtItem::Entry { label, icon, .. } => {
                let text = ctx.fonts(|f| {
                    f.layout_no_wrap(label.to_string(), font.clone(), Color32::PLACEHOLDER)
                        .size()
                        .x
                });
                let icon_w = if icon.is_some() { 22.0 } else { 0.0 };
                w.max(14.0 + icon_w + text)
            }
        });
        let width = (widest + 14.0).max(120.0);
        let area = egui::Area::new(button.id.with("qt_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(button.rect.left(), button.rect.bottom() + 1.0))
            .interactable(true);
        let menu = area.show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme::CARD.into())
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .show(ui, |ui| {
                    ui.set_min_width(width);
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    let mut entry = 0usize;
                    for item in items {
                        match item {
                            QtItem::Separator => {
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(width, 7.0), Sense::hover());
                                ui.painter().line_segment(
                                    [
                                        pos2(rect.left() + 4.0, rect.center().y),
                                        pos2(rect.right() - 4.0, rect.center().y),
                                    ],
                                    Stroke::new(1.0_f32, theme::BORDER_SOFT),
                                );
                            }
                            QtItem::Entry {
                                label,
                                icon,
                                checked,
                            } => {
                                let (rect, resp) =
                                    ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
                                let selected = resp.hovered();
                                if selected {
                                    ui.painter()
                                        .rect_filled(rect, Rounding::ZERO, theme::ACCENT);
                                }
                                let fg = if selected {
                                    HIGHLIGHTED_TEXT
                                } else {
                                    theme::TEXT.into()
                                };
                                let mut x = rect.left() + 6.0;
                                if *checked {
                                    icons::draw(
                                        ui.painter(),
                                        Icon::Check,
                                        Rect::from_center_size(
                                            pos2(x + 5.0, rect.center().y),
                                            vec2(11.0, 11.0),
                                        ),
                                        fg,
                                        1.6,
                                    );
                                }
                                x += 14.0;
                                if let Some(icon) = icon {
                                    icons::draw(
                                        ui.painter(),
                                        *icon,
                                        Rect::from_center_size(
                                            pos2(x + 8.0, rect.center().y),
                                            vec2(15.0, 15.0),
                                        ),
                                        fg,
                                        1.5,
                                    );
                                    x += 22.0;
                                }
                                ui.painter().text(
                                    pos2(x, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    *label,
                                    font.clone(),
                                    fg,
                                );
                                if resp.clicked() {
                                    clicked = Some(entry);
                                }
                                entry += 1;
                            }
                        }
                    }
                });
        });
        if clicked.is_some() {
            open = false;
        }
        // Close on a click anywhere outside the button and the menu.
        if ctx.input(|i| i.pointer.any_click()) {
            let pos = ctx.input(|i| i.pointer.interact_pos());
            let inside =
                pos.is_some_and(|p| button.rect.contains(p) || menu.response.rect.contains(p));
            if !inside {
                open = false;
            }
        }
    }
    ctx.data_mut(|d| d.insert_temp(open_id, open));
    clicked
}

/// The main toolbar of the reference: Add Instance | Folders, Settings,
/// Help, Update | Support, Meow … Accounts.
pub fn show_toolbar(ui: &mut Ui, lang: Language, ctx: &ToolbarCtx<'_>) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, theme::rounding(), theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left(), bar.bottom() - 0.5),
            pos2(bar.right(), bar.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
        ui.add_space(4.0);

        // Add Instance.
        if tb_button(ui, Icon::Plus, lang.shell_add_instance(), false, false).clicked() {
            action.new_instance = true;
        }
        tb_separator(ui);

        // Folders (dropdown): both entries open the real folders.
        let folders_btn = tb_button(ui, Icon::Folder, lang.shell_folders(), true, false);
        let folders_items = [
            QtItem::Entry {
                label: lang.shell_view_instance_folder(),
                icon: Some(Icon::Folder),
                checked: false,
            },
            QtItem::Entry {
                label: lang.shell_view_central_mods(),
                icon: Some(Icon::Package),
                checked: false,
            },
        ];
        if let Some(pick) = qt_dropdown(ui.ctx(), &folders_btn, &folders_items) {
            let dir = if pick == 0 {
                ctx.instances_dir
            } else {
                ctx.central_mods_dir
            };
            let _ = std::fs::create_dir_all(dir);
            let _ = open::that(dir);
        }

        // Settings.
        if tb_button(ui, Icon::Gear, lang.nav_settings(), false, false).clicked() {
            action.tab = Some(NavTab::Settings);
        }

        // Help (dropdown): our own project links.
        let help_btn = tb_button(ui, Icon::Bulb, lang.shell_help(), true, false);
        let help_items = [
            QtItem::Entry {
                label: lang.shell_help_report_bug(),
                icon: Some(Icon::Warning),
                checked: false,
            },
            QtItem::Entry {
                label: lang.shell_help_github(),
                icon: Some(Icon::Link),
                checked: false,
            },
        ];
        if let Some(pick) = qt_dropdown(ui.ctx(), &help_btn, &help_items) {
            if pick == 0 {
                let _ = open::that(format!("{PROJECT_URL}/issues"));
            } else {
                let _ = open::that(PROJECT_URL);
            }
        }

        // Update (the reference updates in place; AMC ships its own exe).
        let _ = tb_button(ui, Icon::Refresh, lang.shell_update(), false, false);

        tb_separator(ui);

        // Support (visual entry of the reference toolbar).
        let _ = tb_button(ui, Icon::Heart, lang.shell_support(), false, false);

        // Meow: checkable easter egg of the reference.
        let meow_id = ui.id().with("multimc_meow");
        let meow = ui.ctx().data_mut(|d| *d.get_temp_mut_or(meow_id, false));
        if tb_button(ui, Icon::Star, lang.shell_meow(), false, meow).clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(meow_id, !meow));
        }

        // Right side: the accounts dropdown (face + profile name).
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            let label = ctx
                .accounts
                .iter()
                .find(|a| Some(a.id) == ctx.active_account)
                .map(|a| a.username.as_str())
                .unwrap_or_else(|| lang.shell_accounts());
            let account_btn = tb_button(ui, Icon::Person, label, true, false);
            if let Some(pick) = qt_dropdown(ui.ctx(), &account_btn, &account_items(lang, ctx)) {
                if pick < ctx.accounts.len() {
                    action.account = Some(ctx.accounts[pick].id);
                } else {
                    action.tab = Some(NavTab::Profile);
                }
            }
        });
    });

    action
}

/// Entries of the accounts dropdown: one checkable row per account plus
/// "Manage Accounts" (our profile page).
fn account_items<'a>(lang: Language, ctx: &'a ToolbarCtx<'_>) -> Vec<QtItem<'a>> {
    let mut items: Vec<QtItem<'a>> = Vec::with_capacity(ctx.accounts.len() + 2);
    for account in ctx.accounts {
        items.push(QtItem::Entry {
            label: account.username.as_str(),
            icon: Some(Icon::Person),
            checked: Some(account.id) == ctx.active_account,
        });
    }
    if !ctx.accounts.is_empty() {
        items.push(QtItem::Separator);
    }
    items.push(QtItem::Entry {
        label: lang.shell_manage_accounts(),
        icon: Some(Icon::Gear),
        checked: false,
    });
    items
}

// ============================================================
// News strip and status bar
// ============================================================

/// The flying news toolbar of the reference, above the status bar. AMC has
/// no blog feed, so its empty-state text is shown; the button opens our
/// project page instead of the reference blog.
pub fn show_news_bar(ui: &mut Ui, lang: Language) {
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, theme::rounding(), theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    ui.horizontal_centered(|ui| {
        ui.add_space(6.0);
        let font = egui::FontId::proportional(12.0);
        let galley = ui.painter().layout_no_wrap(
            lang.shell_news_none().to_owned(),
            font,
            Color32::PLACEHOLDER,
        );
        let (rect, resp) = ui.allocate_exact_size(
            vec2(10.0 + 16.0 + 5.0 + galley.size().x + 10.0, 20.0),
            Sense::click(),
        );
        let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
        if hover > 0.02 {
            ui.painter().rect_filled(
                rect,
                Rounding::ZERO,
                theme::with_alpha(theme::ACCENT, (hover * 56.0) as u8),
            );
        }
        icons::draw(
            ui.painter(),
            Icon::Doc,
            Rect::from_center_size(pos2(rect.left() + 17.0, rect.center().y), vec2(15.0, 15.0)),
            theme::MUTED,
            1.4,
        );
        ui.painter().galley(
            pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
            galley,
            theme::MUTED.into(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            let font = egui::FontId::proportional(12.0);
            let galley = ui.painter().layout_no_wrap(
                lang.shell_more_news().to_owned(),
                font,
                Color32::PLACEHOLDER,
            );
            let (rect, resp) =
                ui.allocate_exact_size(vec2(galley.size().x + 18.0, 18.0), Sense::click());
            let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
            if hover > 0.02 {
                ui.painter().rect_filled(
                    rect,
                    Rounding::ZERO,
                    theme::with_alpha(theme::ACCENT, (hover * 56.0) as u8),
                );
            }
            ui.painter().galley(
                pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                theme::TEXT.into(),
            );
            if resp.clicked() {
                let _ = open::that(PROJECT_URL);
            }
        });
    });
}

/// Status bar: permanent labels on the right of the bar, like the
/// reference (`No instance selected` / `Total playtime: …`).
pub fn show_status_bar(
    ui: &mut Ui,
    lang: Language,
    selected: Option<(&str, &str)>,
    total_minutes: u64,
) {
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, theme::rounding(), theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left(), bar.top() + 0.5),
            pos2(bar.right(), bar.top() + 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let font = egui::FontId::proportional(12.0);
    ui.horizontal_centered(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "{}: {}",
                    lang.shell_total_playtime(),
                    format_minutes(lang, total_minutes)
                ))
                .font(font.clone())
                .color(theme::MUTED),
            );
            ui.add_space(16.0);
            let left = match selected {
                Some((version, loader)) => format!("Minecraft {version} ({loader})"),
                None => lang.shell_status_no_instance().to_string(),
            };
            ui.label(egui::RichText::new(left).font(font).color(theme::TEXT));
        });
    });
}

/// "45 m" / "2 h 5 m" with the localized unit suffixes.
fn format_minutes(lang: Language, minutes: u64) -> String {
    if minutes < 60 {
        format!("{} {}", minutes, lang.inst_mins_suffix())
    } else {
        format!(
            "{} {} {} {}",
            minutes / 60,
            lang.inst_hours_suffix(),
            minutes % 60,
            lang.inst_mins_suffix()
        )
    }
}

// ============================================================
// Right instance toolbar
// ============================================================

/// Text-only button of the instance toolbar (Qt `ToolButtonTextOnly`).
fn bar_button(ui: &mut Ui, label: &str, enabled: bool) -> bool {
    let font = egui::FontId::proportional(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
    let clicked = enabled && resp.clicked();
    if enabled && resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(vec2(2.0, 1.0)),
            Rounding::ZERO,
            theme::CARD_HOVER,
        );
    }
    let color = if enabled {
        theme::TEXT.into()
    } else {
        theme::with_alpha(theme::TEXT, 110)
    };
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        color,
    );
    clicked
}

/// Horizontal separator between the groups of the instance toolbar.
fn bar_separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 7.0), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(rect.left() + 4.0, rect.center().y),
            pos2(rect.right() - 4.0, rect.center().y),
        ],
        Stroke::new(1.0_f32, theme::BORDER_SOFT),
    );
}

/// The instance toolbar docked on the right of the reference: instance
/// icon and name on top, then launch actions, instance editing, folder
/// shortcuts and instance management. Disabled while nothing is selected.
pub fn show_instance_bar(
    ui: &mut Ui,
    lang: Language,
    sel: Option<&InstanceSel>,
    launching: bool,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(bar, theme::rounding(), theme::CARD);
    painter.line_segment(
        [
            pos2(bar.left() + 0.5, bar.top()),
            pos2(bar.left() + 0.5, bar.bottom()),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let enabled = sel.is_some();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.add_space(4.0);

        // Icon button: the reference picks the instance icon here; AMC
        // edits the icon inside the instance dialog.
        let (icon_rect, icon_resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), 60.0), Sense::click());
        if enabled && icon_resp.hovered() {
            ui.painter()
                .rect_filled(icon_rect.shrink(2.0), Rounding::ZERO, theme::CARD_HOVER);
        }
        if let Some(sel) = sel {
            icons::draw_centered(
                ui.painter(),
                sel.badge,
                icon_rect.center(),
                38.0,
                sel.badge_color,
            );
        }
        if enabled && icon_resp.clicked() {
            action.edit_instance = true;
        }

        // Name button: renames the instance in the reference.
        match sel {
            Some(sel) => {
                let galley = ui.painter().layout(
                    sel.name.clone(),
                    egui::FontId::proportional(12.0),
                    Color32::PLACEHOLDER,
                    ui.available_width() - 12.0,
                );
                let (rect, resp) = ui.allocate_exact_size(
                    vec2(ui.available_width(), galley.size().y + 10.0),
                    Sense::click(),
                );
                if resp.hovered() {
                    ui.painter().rect_filled(
                        rect.shrink2(vec2(2.0, 1.0)),
                        Rounding::ZERO,
                        theme::CARD_HOVER,
                    );
                }
                ui.painter().galley(
                    pos2(
                        rect.center().x - galley.size().x / 2.0,
                        rect.center().y - galley.size().y / 2.0,
                    ),
                    galley,
                    theme::TEXT.into(),
                );
                if resp.clicked() {
                    action.edit_instance = true;
                }
            }
            None => ui.add_space(26.0),
        }

        if bar_button(ui, lang.shell_change_group(), enabled) {
            action.edit_instance = true;
        }
        bar_separator(ui);

        // Launch / Launch Offline (the reference turns Launch into Kill
        // while the game runs; AMC has no kill switch, so it disables).
        let launch_label = if launching {
            lang.bottom_btn_launching()
        } else {
            lang.shell_launch_instance()
        };
        if bar_button(ui, launch_label, enabled && !launching) {
            action.launch = true;
        }
        if bar_button(ui, lang.shell_launch_offline(), enabled && !launching) {
            action.launch = true;
        }
        bar_separator(ui);

        if bar_button(ui, lang.shell_edit_instance(), enabled) {
            action.edit_instance = true;
        }
        if bar_button(ui, lang.shell_edit_notes(), enabled) {
            action.edit_instance = true;
        }
        if bar_button(ui, lang.nav_mods(), enabled) {
            action.tab = Some(NavTab::Mods);
        }
        if bar_button(ui, lang.shell_view_worlds(), enabled) {
            action.manage = true;
        }
        if bar_button(ui, lang.shell_manage_screenshots(), enabled) {
            action.manage = true;
        }
        bar_separator(ui);

        if let Some(sel) = sel {
            if bar_button(ui, lang.shell_folder_minecraft(), true) {
                let _ = std::fs::create_dir_all(&sel.game_dir);
                let _ = open::that(&sel.game_dir);
            }
            if bar_button(ui, lang.shell_folder_config(), true) {
                let dir = sel.game_dir.join("config");
                let _ = std::fs::create_dir_all(&dir);
                let _ = open::that(&dir);
            }
            if bar_button(ui, lang.shell_folder_instance(), true) {
                let _ = std::fs::create_dir_all(&sel.game_dir);
                let _ = open::that(&sel.game_dir);
            }
        } else {
            let _ = bar_button(ui, lang.shell_folder_minecraft(), false);
            let _ = bar_button(ui, lang.shell_folder_config(), false);
            let _ = bar_button(ui, lang.shell_folder_instance(), false);
        }
        bar_separator(ui);

        if bar_button(ui, lang.shell_create_shortcut(), enabled) {
            action.shortcut_instance = true;
        }
        // Export is a reference action AMC does not implement yet.
        let _ = bar_button(ui, lang.shell_export_instance(), false);
        if bar_button(ui, lang.shell_delete(), enabled) {
            action.delete_instance = true;
        }
        if bar_button(ui, lang.shell_copy_instance(), enabled) {
            action.clone_instance = true;
        }
    });

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_minutes_splits_hours() {
        let short = format_minutes(Language::English, 42);
        assert!(short.starts_with("42"));
        let long = format_minutes(Language::English, 125);
        assert!(long.starts_with("2"));
        assert!(long.contains("5"));
        // Zero stays readable ("0 m").
        assert!(format_minutes(Language::Russian, 0).starts_with('0'));
    }

    #[test]
    fn test_account_items_end_with_manage() {
        let ctx = ToolbarCtx {
            accounts: &[],
            active_account: None,
            launching: false,
            instances_dir: Path::new("."),
            central_mods_dir: Path::new("."),
        };
        let items = account_items(Language::English, &ctx);
        assert_eq!(items.len(), 1);
        assert!(matches!(
            items[0],
            QtItem::Entry {
                label: "Manage Accounts",
                checked: false,
                ..
            }
        ));
    }
}
