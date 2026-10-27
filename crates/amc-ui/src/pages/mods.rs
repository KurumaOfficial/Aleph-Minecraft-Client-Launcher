use crate::theme;
use crate::theme::{
    lerp_color, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, SUCCESS,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, icon_button, Icon};
use crate::widgets::pixel;
use amc_core::Language;
use amc_mods::types::{LocalMod, ModCategory, ModSearchResult, ModSource, SearchSort};
use egui::{pos2, vec2, Color32, ScrollArea, Sense, Stroke, TextEdit, Ui};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModsSubTab {
    #[default]
    Local,
    Search,
}

/// Visual skin for the shared mods page. The Modrinth shell uses a rounded
/// segmented switch and sentence-case labels from the reference app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModsSkin {
    #[default]
    Amc,
    Modrinth,
    Millida,
    Multimc,
    Labymod,
}

/// Outcome of one update check (CONCEPT P4): the fresh local scan
/// plus the offers derived from it. The scan runs off the UI thread
/// (jar manifest parsing), so both travel in one message.
#[derive(Debug, Clone, Default)]
pub struct ModUpdateResult {
    pub mods: Vec<LocalMod>,
    pub offers: Vec<UpdateOffer>,
}

/// One-click update offer for an installed mod (CONCEPT P4).
#[derive(Debug, Clone)]
pub struct UpdateOffer {
    pub slug: String,
    pub title: String,
    pub version: String,
    pub url: String,
    pub filename: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub old_filename: String,
    /// Author changelog of the new version, when the source provides one.
    pub changelog: Option<String>,
}

/// Tooltip body for an update offer (CONCEPT "обновление показывает
/// список изменений"): new version plus a collapsed changelog preview.
/// Pure function, unit-tested below.
pub fn changelog_preview(version: &str, changelog: &Option<String>) -> String {
    const LIMIT: usize = 280;
    let Some(notes) = changelog else {
        return format!("→ {version}");
    };
    let collapsed: String = notes
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if collapsed.is_empty() {
        return format!("→ {version}");
    }
    let short: String = collapsed.chars().take(LIMIT).collect();
    let tail = if collapsed.chars().count() > LIMIT {
        "…"
    } else {
        ""
    };
    format!("→ {version}\n{short}{tail}")
}

/// Cache key for a marketplace artwork texture: source-scoped so equal
/// numeric ids from Modrinth and CurseForge never collide.
pub fn icon_key(source: &ModSource, id: &str) -> String {
    let tag = match source {
        ModSource::Modrinth => "mr",
        ModSource::CurseForge => "cf",
        ModSource::Local => "local",
    };
    format!("{tag}:{id}")
}

/// Disk filename for a cached icon: the key with filesystem-hostile
/// characters flattened (notably `:` on Windows).
pub fn icon_cache_file(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub struct ModsPage {
    pub sub_tab: ModsSubTab,
    pub search_provider: ModSource,
    pub search_category: ModCategory,
    pub search_sort: SearchSort,
    pub search_query: String,
    pub is_searching: bool,
    pub search_results: Vec<ModSearchResult>,
    pub local_mods: Vec<LocalMod>,
    /// Folder the cached local-mod list was scanned from. The app rescans
    /// when the selected instance (or its `mods/` folder) changes.
    pub(crate) local_source: Option<std::path::PathBuf>,
    pub installing_ids: HashSet<String>,
    pub installed_titles: HashSet<String>,
    pub status_message: Option<(String, bool)>,
    pub selected_local: HashSet<String>,
    pub update_check_requested: bool,
    pub update_checking: bool,
    pub update_offers: HashMap<String, UpdateOffer>,
    pub pending_update_old: Option<String>,
    /// "Update all" button state (CONCEPT P4).
    pub update_all_requested: bool,
    /// Version pins, mirrored from config each frame; slug toggle requests.
    pub pinned_keys: Vec<String>,
    pub pin_toggle: Option<String>,
    /// Rollback of replaced files stashed by updates (CONCEPT P4).
    pub rollback_available: bool,
    pub rollback_requested: bool,
    /// OptiFine-as-mod install request (CONCEPT P4: OptiFine is a
    /// regular mod, not a loader). Handled by the app.
    pub optifine_install_requested: bool,
    /// Compat context for per-row hints, set by the app each frame.
    pub compat_mc: Option<String>,
    pub compat_loader: String,
    /// Marketplace artwork cache (CONCEPT P10 previews): live textures
    /// plus in-flight/failed keys so each icon fetches at most once.
    pub icon_textures: HashMap<String, egui::TextureHandle>,
    pub icon_pending: HashSet<String>,
    pub icon_failed: HashSet<String>,
    /// Lowercased local mod names for O(1) installed checks without
    /// per-row `to_lowercase` (rebuilt when `local_mods` changes).
    pub(crate) local_lower: HashSet<String>,
    pub(crate) local_lower_rev: u64,
    /// Lowercased search titles parallel to `search_results` (rebuilt
    /// when the result set changes).
    pub(crate) search_lower: Vec<String>,
    pub(crate) search_rev: (usize, String, String),
}

impl Default for ModsPage {
    fn default() -> Self {
        Self {
            sub_tab: ModsSubTab::Local,
            search_provider: ModSource::Modrinth,
            search_category: ModCategory::Mod,
            search_sort: SearchSort::Relevance,
            search_query: String::new(),
            is_searching: false,
            search_results: Vec::new(),
            local_mods: Vec::new(),
            local_source: None,
            installing_ids: HashSet::new(),
            installed_titles: HashSet::new(),
            status_message: None,
            selected_local: HashSet::new(),
            update_check_requested: false,
            update_checking: false,
            update_offers: HashMap::new(),
            pending_update_old: None,
            update_all_requested: false,
            pinned_keys: Vec::new(),
            pin_toggle: None,
            rollback_available: false,
            rollback_requested: false,
            optifine_install_requested: false,
            compat_mc: None,
            compat_loader: "minecraft".to_string(),
            icon_textures: HashMap::new(),
            icon_pending: HashSet::new(),
            icon_failed: HashSet::new(),
            local_lower: HashSet::new(),
            local_lower_rev: 0,
            search_lower: Vec::new(),
            search_rev: (0, String::new(), String::new()),
        }
    }
}

/// Arguments for [`ModsPage::show`], bundled for the argument-count lint.
pub struct ModsActions<'a, S, I> {
    pub wishlist: &'a mut Vec<String>,
    pub update_install: &'a mut Option<(ModSearchResult, String)>,
    pub on_search: S,
    pub on_install: I,
}

impl ModsPage {
    pub fn show<S, I>(
        &mut self,
        ui: &mut Ui,
        mods_dir: &Path,
        lang: Language,
        skin: ModsSkin,
        instance_meta: Option<&str>,
        actions: ModsActions<'_, S, I>,
    ) where
        S: FnOnce(String, ModSource, ModCategory, SearchSort),
        I: FnOnce(ModSearchResult),
    {
        ui.add_space(if skin == ModsSkin::Millida {
            14.0
        } else if skin == ModsSkin::Labymod {
            12.0
        } else {
            20.0
        });

        // Header
        let millida = skin == ModsSkin::Millida;
        let multimc = skin == ModsSkin::Multimc;
        if skin == ModsSkin::Labymod {
            labymod_mods_header(ui, lang);
        } else {
            let title_size = if skin == ModsSkin::Modrinth {
                24.0
            } else if millida {
                28.0
            } else if multimc {
                16.0
            } else {
                26.0
            };
            let title = if skin == ModsSkin::Modrinth {
                lang.nav_mods()
            } else if millida {
                lang.millida_mods_title()
            } else {
                lang.mods_title()
            };
            let title_font = if millida {
                crate::theme::display(title_size)
            } else {
                egui::FontId::proportional(title_size)
            };
            ui.label(
                egui::RichText::new(title)
                    .font(title_font)
                    .strong()
                    .color(TEXT_HEADING),
            );
            if multimc {
                let (line, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                ui.painter()
                    .rect_filled(line, egui::Rounding::ZERO, BORDER_DEFAULT);
            }
            if skin == ModsSkin::Modrinth {
                if let Some(meta) = instance_meta {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(meta)
                            .font(egui::FontId::proportional(12.0))
                            .color(crate::theme::MODRINTH_TEXT_SECONDARY),
                    );
                }
            } else if millida {
                let count = match self.sub_tab {
                    ModsSubTab::Local => self.local_mods.len(),
                    ModsSubTab::Search => self.search_results.len(),
                };
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let (mark, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter().rect_filled(mark, egui::Rounding::ZERO, RUBY);
                    ui.label(
                        egui::RichText::new(lang.millida_mods_count(count))
                            .font(egui::FontId::proportional(12.5))
                            .color(TEXT_MUTED),
                    );
                    if let Some(meta) = instance_meta {
                        ui.label(
                            egui::RichText::new(meta)
                                .font(egui::FontId::proportional(12.0))
                                .color(TEXT_MUTED),
                        );
                    }
                });
            }
        }

        ui.add_space(if millida { 12.0 } else { 14.0 });

        // Status banner if present
        if let Some((msg, is_ok)) = &self.status_message {
            let col = if *is_ok { SUCCESS } else { RUBY };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("ℹ {msg}")).color(col).strong());
            });
            ui.add_space(8.0);
        }

        // Subtabs: Local / Search
        if skin == ModsSkin::Modrinth {
            self.show_modrinth_tabs(ui, lang, mods_dir);
        } else if millida {
            self.show_millida_tabs(ui, lang, mods_dir);
        } else if multimc {
            self.show_multimc_tabs(ui, lang, mods_dir);
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

                let local_active = self.sub_tab == ModsSubTab::Local;
                let search_active = self.sub_tab == ModsSubTab::Search;

                // Local Tab Button
                let (loc_rect, loc_resp) =
                    ui.allocate_exact_size(vec2(130.0, 32.0), Sense::click());
                let loc_hover = ui
                    .ctx()
                    .animate_bool_responsive(loc_resp.id, loc_resp.hovered());
                let loc_act = ui.ctx().animate_bool(loc_resp.id.with("act"), local_active);
                let loc_bg = lerp_color(
                    lerp_color(Color32::TRANSPARENT, BG_HOVER, loc_hover),
                    RUBY,
                    loc_act,
                );
                let loc_stroke = lerp_color(
                    lerp_color(BORDER_DEFAULT, RUBY, loc_hover),
                    RUBY_LIGHT,
                    loc_act,
                );
                let loc_text = lerp_color(
                    lerp_color(TEXT_MUTED, TEXT_PRIMARY, loc_hover),
                    Color32::WHITE,
                    loc_act,
                );

                ui.painter()
                    .rect_filled(loc_rect, crate::theme::rounding(), loc_bg);
                ui.painter().rect_stroke(
                    loc_rect,
                    crate::theme::rounding(),
                    Stroke::new(1.0_f32, loc_stroke),
                );
                ui.painter().text(
                    loc_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    lang.mods_tab_installed(),
                    egui::FontId::proportional(12.0),
                    loc_text,
                );

                if loc_resp.clicked() {
                    self.sub_tab = ModsSubTab::Local;
                }

                // Search Tab Button
                let search_label = match self.search_provider {
                    ModSource::Modrinth => lang.mods_tab_search_modrinth(),
                    ModSource::CurseForge => lang.mods_tab_search_curseforge(),
                    _ => lang.mods_tab_search_generic(),
                };

                let (sea_rect, sea_resp) =
                    ui.allocate_exact_size(vec2(160.0, 32.0), Sense::click());
                let sea_hover = ui
                    .ctx()
                    .animate_bool_responsive(sea_resp.id, sea_resp.hovered());
                let sea_act = ui
                    .ctx()
                    .animate_bool(sea_resp.id.with("act"), search_active);
                let sea_bg = lerp_color(
                    lerp_color(Color32::TRANSPARENT, BG_HOVER, sea_hover),
                    RUBY,
                    sea_act,
                );
                let sea_stroke = lerp_color(
                    lerp_color(BORDER_DEFAULT, RUBY, sea_hover),
                    RUBY_LIGHT,
                    sea_act,
                );
                let sea_text = lerp_color(
                    lerp_color(TEXT_MUTED, TEXT_PRIMARY, sea_hover),
                    Color32::WHITE,
                    sea_act,
                );

                ui.painter()
                    .rect_filled(sea_rect, crate::theme::rounding(), sea_bg);
                ui.painter().rect_stroke(
                    sea_rect,
                    crate::theme::rounding(),
                    Stroke::new(1.0_f32, sea_stroke),
                );
                ui.painter().text(
                    sea_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    search_label,
                    egui::FontId::proportional(12.0),
                    sea_text,
                );

                if sea_resp.clicked() {
                    self.sub_tab = ModsSubTab::Search;
                }

                ui.add_space(16.0);

                if ui.button(lang.mods_btn_open_folder()).clicked() {
                    let _ = std::fs::create_dir_all(mods_dir);
                    let _ = open::that(mods_dir);
                }
            });
        }

        ui.add_space(14.0);

        match self.sub_tab {
            ModsSubTab::Local => self.show_local_mods(ui, lang, skin, actions.update_install),
            ModsSubTab::Search => self.show_search(
                ui,
                lang,
                skin,
                actions.wishlist,
                actions.on_search,
                actions.on_install,
            ),
        }
    }

    /// MultiMC/Prism treatment: flat Qt push buttons with the Fusion blue
    /// selection and sentence-case labels.
    fn show_multimc_tabs(&mut self, ui: &mut Ui, lang: Language, mods_dir: &Path) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (idx, (label, active)) in [
                (
                    lang.mods_tab_installed_plain(),
                    self.sub_tab == ModsSubTab::Local,
                ),
                (
                    lang.mods_tab_search_plain(),
                    self.sub_tab == ModsSubTab::Search,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let width = (ui
                    .painter()
                    .layout_no_wrap(
                        label.to_owned(),
                        egui::FontId::proportional(12.0),
                        Color32::PLACEHOLDER,
                    )
                    .size()
                    .x
                    + 30.0)
                    .max(112.0);
                if multimc_button(ui, label, active, width) {
                    self.sub_tab = if idx == 0 {
                        ModsSubTab::Local
                    } else {
                        ModsSubTab::Search
                    };
                }
            }
            if multimc_button(ui, lang.mods_open_folder_plain(), false, 156.0) {
                let _ = std::fs::create_dir_all(mods_dir);
                let _ = open::that(mods_dir);
            }
        });
    }

    /// Millida catalog header: cut-corner tabs with the shell's mint active
    /// plate and a matched folder action.
    fn show_millida_tabs(&mut self, ui: &mut Ui, lang: Language, mods_dir: &Path) {
        let font = egui::FontId::new(12.0, egui::FontFamily::Name("bold".into()));
        let installed = lang.mods_tab_installed_plain().to_owned();
        let search = lang.mods_tab_search_plain().to_owned();
        let folder = lang.mods_open_folder_plain().to_owned();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (idx, (label, active)) in [
                (installed, self.sub_tab == ModsSubTab::Local),
                (search, self.sub_tab == ModsSubTab::Search),
            ]
            .into_iter()
            .enumerate()
            {
                let galley =
                    ui.painter()
                        .layout_no_wrap(label.clone(), font.clone(), Color32::PLACEHOLDER);
                let width = (galley.size().x + 32.0).max(122.0);
                let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                let fill = if active {
                    theme::ACCENT.into()
                } else {
                    theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover)
                };
                let outline = if active {
                    theme::ACCENT_LIGHT.into()
                } else {
                    theme::BORDER_STRONG.into()
                };
                millida_pixel_plate(ui.painter(), rect, 5.0, fill, outline);
                ui.painter().galley(
                    pos2(
                        rect.center().x - galley.size().x / 2.0,
                        rect.center().y - galley.size().y / 2.0,
                    ),
                    galley,
                    if active {
                        theme::BG.into()
                    } else {
                        theme::TEXT_PRIMARY.into()
                    },
                );
                if resp.clicked() {
                    self.sub_tab = if idx == 0 {
                        ModsSubTab::Local
                    } else {
                        ModsSubTab::Search
                    };
                }
            }

            let galley = ui
                .painter()
                .layout_no_wrap(folder, font, Color32::PLACEHOLDER);
            let width = (galley.size().x + 48.0).max(156.0);
            let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            millida_pixel_plate(
                ui.painter(),
                rect,
                5.0,
                theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover),
                theme::BORDER_STRONG.into(),
            );
            icons::draw_centered(
                ui.painter(),
                Icon::Folder,
                pos2(rect.left() + 18.0, rect.center().y),
                15.0,
                theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
            );
            ui.painter().galley(
                pos2(rect.left() + 32.0, rect.center().y - galley.size().y / 2.0),
                galley,
                theme::TEXT_PRIMARY.into(),
            );
            if resp.clicked() {
                let _ = std::fs::create_dir_all(mods_dir);
                let _ = open::that(mods_dir);
            }
        });
    }

    /// Reference `NavTabs` treatment for the Modrinth shell: a rounded
    /// surface tray with a green selected pill and sentence-case labels.
    fn show_modrinth_tabs(&mut self, ui: &mut Ui, lang: Language, mods_dir: &Path) {
        let font = egui::FontId::new(13.0, egui::FontFamily::Name("bold".into()));
        let installed = lang.mods_tab_installed_plain().to_owned();
        let search = lang.mods_tab_search_plain().to_owned();
        let installed_w = (ui
            .painter()
            .layout_no_wrap(installed.clone(), font.clone(), Color32::PLACEHOLDER)
            .size()
            .x
            + 32.0)
            .max(118.0);
        let search_w = (ui
            .painter()
            .layout_no_wrap(search.clone(), font.clone(), Color32::PLACEHOLDER)
            .size()
            .x
            + 32.0)
            .max(134.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            let (tray, _) =
                ui.allocate_exact_size(vec2(installed_w + search_w + 8.0, 40.0), Sense::hover());
            ui.painter().rect_filled(
                tray,
                egui::Rounding::same(20.0),
                crate::theme::MODRINTH_SURFACE,
            );
            let mut x = tray.left() + 4.0;
            for (idx, (label, width, active)) in [
                (installed, installed_w, self.sub_tab == ModsSubTab::Local),
                (search, search_w, self.sub_tab == ModsSubTab::Search),
            ]
            .into_iter()
            .enumerate()
            {
                let rect =
                    egui::Rect::from_min_size(egui::pos2(x, tray.top() + 4.0), vec2(width, 32.0));
                let resp = ui.interact(
                    rect,
                    ui.id().with(("modrinth_mods_tab", idx)),
                    Sense::click(),
                );
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                if active {
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(16.0),
                        crate::theme::MODRINTH_GREEN,
                    );
                } else if hover > 0.02 {
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(16.0),
                        crate::theme::lerp_color(
                            Color32::TRANSPARENT,
                            crate::theme::MODRINTH_SURFACE_HIGH,
                            hover,
                        ),
                    );
                }
                let color = if active {
                    crate::theme::MODRINTH_BG
                } else {
                    crate::theme::lerp_color(
                        crate::theme::MODRINTH_TEXT,
                        crate::theme::MODRINTH_TEXT_CONTRAST,
                        hover,
                    )
                };
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &label,
                    font.clone(),
                    color,
                );
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    self.sub_tab = if idx == 0 {
                        ModsSubTab::Local
                    } else {
                        ModsSubTab::Search
                    };
                }
                x += width;
            }

            ui.add_space(8.0);
            let folder_label = lang.mods_open_folder_plain();
            let label_galley = ui.painter().layout_no_wrap(
                folder_label.to_owned(),
                egui::FontId::proportional(12.5),
                Color32::PLACEHOLDER,
            );
            let button_w = (label_galley.size().x + 54.0).max(150.0);
            let (button, resp) = ui.allocate_exact_size(vec2(button_w, 40.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            ui.painter().rect_filled(
                button,
                egui::Rounding::same(20.0),
                crate::theme::lerp_color(
                    Color32::TRANSPARENT,
                    crate::theme::MODRINTH_SURFACE,
                    hover,
                ),
            );
            ui.painter().rect_stroke(
                button,
                egui::Rounding::same(20.0),
                Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
            );
            icons::draw(
                ui.painter(),
                Icon::Folder,
                egui::Rect::from_center_size(
                    egui::pos2(button.left() + 22.0, button.center().y),
                    vec2(15.0, 15.0),
                ),
                crate::theme::MODRINTH_TEXT,
                1.3,
            );
            ui.painter().galley(
                egui::pos2(
                    button.left() + 36.0,
                    button.center().y - label_galley.size().y / 2.0,
                ),
                label_galley,
                crate::theme::MODRINTH_TEXT,
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                let _ = std::fs::create_dir_all(mods_dir);
                let _ = open::that(mods_dir);
            }
        });
    }

    fn show_millida_empty(&mut self, ui: &mut Ui, lang: Language) {
        ui.add_space(22.0);
        let width = ui.available_width();
        let (card, _) = ui.allocate_exact_size(vec2(width, 222.0), Sense::hover());
        millida_pixel_plate(
            ui.painter(),
            card,
            10.0,
            theme::BG_CARD.into(),
            theme::BORDER_DEFAULT.into(),
        );
        let icon = egui::Rect::from_center_size(
            pos2(card.center().x, card.top() + 58.0),
            vec2(58.0, 58.0),
        );
        millida_pixel_plate(
            ui.painter(),
            icon,
            8.0,
            theme::BG_CARD.into(),
            theme::ACCENT.into(),
        );
        icons::draw_centered(
            ui.painter(),
            Icon::Puzzle,
            icon.center(),
            27.0,
            theme::ACCENT,
        );
        let title = ui.painter().layout_no_wrap(
            lang.mods_empty_title().to_owned(),
            crate::theme::display(17.0),
            theme::TEXT_HEADING.into(),
        );
        ui.painter().galley(
            pos2(card.center().x - title.size().x / 2.0, card.top() + 100.0),
            title,
            theme::TEXT_HEADING.into(),
        );
        let hint = ui.painter().layout(
            lang.mods_empty_hint().to_owned(),
            egui::FontId::proportional(12.5),
            theme::MUTED.into(),
            (width - 56.0).max(120.0),
        );
        ui.painter().galley(
            pos2(card.center().x - hint.size().x / 2.0, card.top() + 126.0),
            hint,
            theme::MUTED.into(),
        );

        let action = egui::Rect::from_center_size(
            pos2(card.center().x, card.bottom() - 38.0),
            vec2(168.0, 36.0),
        );
        let resp = ui
            .interact(action, ui.id().with("millida_browse_mods"), Sense::click())
            .on_hover_text(lang.mods_browse_action());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        millida_pixel_plate(
            ui.painter(),
            action,
            5.0,
            theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
            theme::ACCENT_LIGHT.into(),
        );
        ui.painter().text(
            action.center(),
            egui::Align2::CENTER_CENTER,
            lang.mods_browse_action(),
            egui::FontId::new(12.0, egui::FontFamily::Name("bold".into())),
            theme::BG.into(),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.sub_tab = ModsSubTab::Search;
        }
    }

    fn show_labymod_empty(&mut self, ui: &mut Ui, lang: Language) {
        ui.add_space(22.0);
        let width = ui.available_width();
        let (card, _) = ui.allocate_exact_size(vec2(width, 232.0), Sense::hover());
        ui.painter()
            .rect_filled(card, egui::Rounding::same(16.0), BG_CARD);
        ui.painter().rect_stroke(
            card,
            egui::Rounding::same(16.0),
            Stroke::new(1.0_f32, BORDER_DEFAULT),
        );
        let icon = egui::Rect::from_center_size(
            pos2(card.center().x, card.top() + 58.0),
            vec2(64.0, 64.0),
        );
        ui.painter().rect_filled(
            icon,
            egui::Rounding::same(16.0),
            theme::with_alpha(theme::ACCENT, 38),
        );
        ui.painter().rect_stroke(
            icon,
            egui::Rounding::same(16.0),
            Stroke::new(1.0_f32, theme::with_alpha(theme::ACCENT, 130)),
        );
        icons::draw_centered(
            ui.painter(),
            Icon::Puzzle,
            icon.center(),
            28.0,
            theme::ACCENT_LIGHT,
        );
        let title = ui.painter().layout_no_wrap(
            lang.mods_empty_title().to_owned(),
            egui::FontId::new(18.0, egui::FontFamily::Name("bold".into())),
            TEXT_HEADING.into(),
        );
        ui.painter().galley(
            pos2(card.center().x - title.size().x / 2.0, card.top() + 104.0),
            title,
            TEXT_HEADING.into(),
        );
        let hint = ui.painter().layout(
            lang.mods_empty_hint().to_owned(),
            egui::FontId::proportional(12.5),
            TEXT_MUTED.into(),
            (width - 56.0).max(120.0),
        );
        ui.painter().galley(
            pos2(card.center().x - hint.size().x / 2.0, card.top() + 132.0),
            hint,
            TEXT_MUTED.into(),
        );

        let action = egui::Rect::from_center_size(
            pos2(card.center().x, card.bottom() - 38.0),
            vec2(180.0, 36.0),
        );
        let resp = ui
            .interact(action, ui.id().with("laby_browse_mods"), Sense::click())
            .on_hover_text(lang.mods_browse_action());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        ui.painter().rect_filled(
            action,
            egui::Rounding::same(10.0),
            theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
        );
        ui.painter().rect_stroke(
            action,
            egui::Rounding::same(10.0),
            Stroke::new(1.0_f32, theme::ACCENT_LIGHT),
        );
        ui.painter().text(
            action.center(),
            egui::Align2::CENTER_CENTER,
            lang.mods_browse_action(),
            egui::FontId::new(12.0, egui::FontFamily::Name("bold".into())),
            Color32::WHITE,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.sub_tab = ModsSubTab::Search;
        }
    }

    fn show_local_mods(
        &mut self,
        ui: &mut Ui,
        lang: Language,
        skin: ModsSkin,
        update_install: &mut Option<(ModSearchResult, String)>,
    ) {
        if self.local_mods.is_empty() {
            if skin == ModsSkin::Multimc {
                // Qt empty state: the reference offers the next action
                // instead of leaving a dead-end sentence.
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(lang.mods_empty_title())
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(lang.mods_empty_hint())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(12.0);
                    let button = egui::Button::new(
                        egui::RichText::new(lang.mods_browse_action())
                            .font(egui::FontId::proportional(12.0)),
                    )
                    .min_size(vec2(140.0, 32.0));
                    if ui.add(button).clicked() {
                        self.sub_tab = ModsSubTab::Search;
                    }
                });
                return;
            }
            if skin == ModsSkin::Millida {
                self.show_millida_empty(ui, lang);
                return;
            }
            if skin == ModsSkin::Labymod {
                self.show_labymod_empty(ui, lang);
                return;
            }
            if skin == ModsSkin::Modrinth {
                ui.add_space(36.0);
                ui.vertical_centered(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
                    ui.painter().rect_filled(
                        icon_rect,
                        egui::Rounding::same(16.0),
                        crate::theme::MODRINTH_SURFACE,
                    );
                    icons::draw_centered(
                        ui.painter(),
                        Icon::Puzzle,
                        icon_rect.center(),
                        28.0,
                        crate::theme::MODRINTH_GREEN,
                    );
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new(lang.mods_empty_title())
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(crate::theme::MODRINTH_TEXT_CONTRAST),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(lang.mods_empty_hint())
                            .font(egui::FontId::proportional(12.5))
                            .color(crate::theme::MODRINTH_TEXT_SECONDARY),
                    );
                    ui.add_space(14.0);
                    let button = egui::Button::new(
                        egui::RichText::new(lang.mods_browse_action())
                            .font(egui::FontId::proportional(13.0))
                            .strong()
                            .color(crate::theme::MODRINTH_BG),
                    )
                    .fill(crate::theme::MODRINTH_GREEN)
                    .rounding(egui::Rounding::same(10.0))
                    .min_size(vec2(154.0, 38.0));
                    if ui.add(button).clicked() {
                        self.sub_tab = ModsSubTab::Search;
                    }
                });
                return;
            }
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(lang.mods_no_installed())
                        .font(egui::FontId::proportional(15.0))
                        .color(TEXT_MUTED),
                );
                ui.add_space(12.0);
                let button = egui::Button::new(
                    egui::RichText::new(lang.mods_browse_action())
                        .font(egui::FontId::proportional(12.0))
                        .color(Color32::WHITE),
                )
                .fill(RUBY)
                .min_size(vec2(140.0, 32.0));
                if ui.add(button).clicked() {
                    self.sub_tab = ModsSubTab::Search;
                }
            });
            return;
        }

        let millida = skin == ModsSkin::Millida;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                // Batch operations bar. Wraps on the 960 px minimum instead
                // of clipping the last action under the window edge.
                ui.horizontal_wrapped(|ui| {
                    let action = |ui: &mut Ui, label: &str| -> bool {
                        if millida {
                            millida_action_button(ui, label, false)
                        } else {
                            ui.button(label).clicked()
                        }
                    };
                    let all_selected = !self.local_mods.is_empty()
                        && self.selected_local.len() == self.local_mods.len();
                    let batch_icon = if all_selected {
                        Icon::Close
                    } else {
                        Icon::Check
                    };
                    let batch_font = egui::TextStyle::Button.resolve(ui.style());
                    let batch_fill = ui.visuals().widgets.inactive.weak_bg_fill;
                    let select_clicked = if millida {
                        millida_action_button(ui, lang.batch_select_all(), false)
                    } else {
                        icons::icon_text_button(
                            ui,
                            batch_icon,
                            None,
                            lang.batch_select_all(),
                            TEXT_PRIMARY,
                            batch_font,
                            batch_fill,
                        )
                        .clicked()
                    };
                    if select_clicked {
                        if all_selected {
                            self.selected_local.clear();
                        } else {
                            self.selected_local = self
                                .local_mods
                                .iter()
                                .map(|m| m.path.to_string_lossy().to_string())
                                .collect();
                        }
                    }
                    if action(ui, lang.batch_enable()) {
                        for m in self.local_mods.iter_mut() {
                            let key = m.path.to_string_lossy().to_string();
                            if self.selected_local.contains(&key) && !m.enabled {
                                if let Ok(new_path) = amc_mods::LocalModManager::toggle_mod(&m.path)
                                {
                                    m.path = new_path;
                                    m.enabled = true;
                                }
                            }
                        }
                    }
                    if action(ui, lang.batch_disable()) {
                        for m in self.local_mods.iter_mut() {
                            let key = m.path.to_string_lossy().to_string();
                            if self.selected_local.contains(&key) && m.enabled {
                                if let Ok(new_path) = amc_mods::LocalModManager::toggle_mod(&m.path)
                                {
                                    m.path = new_path;
                                    m.enabled = false;
                                }
                            }
                        }
                    }
                    if action(ui, lang.batch_delete()) {
                        let doomed: Vec<String> = self.selected_local.iter().cloned().collect();
                        for key in &doomed {
                            if let Some(m) = self
                                .local_mods
                                .iter()
                                .find(|m| m.path.to_string_lossy() == *key)
                            {
                                let _ = amc_mods::LocalModManager::delete_mod(&m.path);
                            }
                        }
                        self.local_mods
                            .retain(|m| !doomed.contains(&m.path.to_string_lossy().to_string()));
                        self.selected_local.clear();
                    }
                    ui.add_space(16.0);
                    if self.update_checking {
                        ui.spinner();
                    } else if action(ui, lang.mods_check_updates()) {
                        self.update_check_requested = true;
                    }
                    if !self.update_offers.is_empty() && action(ui, lang.mods_update_all()) {
                        self.update_all_requested = true;
                    }
                    if self.rollback_available && action(ui, lang.mods_rollback()) {
                        self.rollback_requested = true;
                    }
                    if action(ui, lang.mods_install_optifine()) {
                        self.optifine_install_requested = true;
                    }
                });

                ui.add_space(8.0);

                let mut to_toggle = None;
                let mut to_delete = None;

                for (idx, m) in self.local_mods.iter().enumerate() {
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::click());
                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let stroke_col = lerp_color(BORDER_DEFAULT, RUBY, hover_t);

                    ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
                    ui.painter().rect_stroke(
                        rect,
                        crate::theme::rounding(),
                        Stroke::new(1.0_f32, stroke_col),
                    );

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(14.0);

                    // Batch selection checkbox.
                    let row_key = m.path.to_string_lossy().to_string();
                    let mut row_sel = self.selected_local.contains(&row_key);
                    let toggled = if millida {
                        millida_check(&mut child, row_sel)
                    } else {
                        child.checkbox(&mut row_sel, "").clicked()
                    };
                    if toggled {
                        row_sel = !row_sel;
                    }
                    if row_sel {
                        self.selected_local.insert(row_key);
                    } else {
                        self.selected_local
                            .remove(&m.path.to_string_lossy().to_string());
                    }

                    // Mod icon (painter vector: the bundled fonts carry no
                    // emoji, so a glyph would render as a tofu box).
                    let (mod_icon_rect, _) =
                        child.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                    icons::draw_centered(
                        child.painter(),
                        Icon::Puzzle,
                        mod_icon_rect.center(),
                        18.0,
                        TEXT_PRIMARY,
                    );
                    child.add_space(10.0);

                    // Name + version in a fixed text column so the action
                    // controls always keep their reserved space at 960 px.
                    let has_offer = self.update_offers.contains_key(&m.name);
                    let controls_width = if has_offer { 258.0 } else { 116.0 };
                    let text_w = (child.available_width() - controls_width).max(120.0);
                    child.allocate_ui_with_layout(
                        vec2(text_w, 44.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&m.name)
                                        .font(egui::FontId::proportional(14.0))
                                        .strong()
                                        .color(TEXT_HEADING),
                                )
                                .truncate(),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "v{} • {:.1} KB",
                                    m.version,
                                    m.file_size as f64 / 1024.0
                                ))
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                            );
                        },
                    );

                    // Controls on the right, laid out from the right edge so
                    // they cannot be pushed out by a long mod name.
                    child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(ui, Icon::Trash, TEXT_MUTED).clicked() {
                            to_delete = Some(idx);
                        }
                        ui.add_space(8.0);

                        let toggle_text = if m.enabled {
                            lang.mods_status_enabled()
                        } else {
                            lang.mods_status_disabled()
                        };
                        let toggle_color: Color32 = if m.enabled {
                            if matches!(skin, ModsSkin::Labymod | ModsSkin::Multimc) {
                                crate::theme::ACCENT.into()
                            } else {
                                SUCCESS.into()
                            }
                        } else {
                            TEXT_MUTED.into()
                        };
                        let toggle_btn = egui::Button::new(
                            egui::RichText::new(toggle_text)
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(toggle_color)
                        .min_size(vec2(60.0, 26.0));
                        let toggled = if millida {
                            millida_action_button(ui, toggle_text, m.enabled)
                        } else {
                            ui.add(toggle_btn).clicked()
                        };
                        if toggled {
                            to_toggle = Some(idx);
                        }

                        // One-click update when the checker found a newer build.
                        if let Some(offer) = self.update_offers.get(&m.name).cloned() {
                            ui.add_space(8.0);
                            let pinned = self.pinned_keys.contains(&offer.slug);
                            if icon_button(
                                ui,
                                if pinned { Icon::Pin } else { Icon::PinOutline },
                                if pinned { RUBY_LIGHT } else { TEXT_MUTED },
                            )
                            .on_hover_text(if pinned {
                                lang.mods_unpin()
                            } else {
                                lang.mods_pin()
                            })
                            .clicked()
                            {
                                self.pin_toggle = Some(offer.slug.clone());
                            }
                            if !pinned
                                && ui
                                    .button(
                                        egui::RichText::new(lang.mods_update_available())
                                            .color(RUBY_LIGHT),
                                    )
                                    .on_hover_text(changelog_preview(
                                        &offer.version,
                                        &offer.changelog,
                                    ))
                                    .clicked()
                            {
                                *update_install = Some((
                                    ModSearchResult {
                                        source: ModSource::Modrinth,
                                        id: offer.slug.clone(),
                                        title: offer.title.clone(),
                                        author: String::new(),
                                        downloads: 0,
                                        rating: 0,
                                        description: String::new(),
                                        icon_url: None,
                                        category: ModCategory::Mod,
                                        game_versions: Vec::new(),
                                        loaders: Vec::new(),
                                    },
                                    offer.old_filename.clone(),
                                ));
                            }
                        }
                    });
                }

                if let Some(idx) = to_toggle {
                    if let Ok(new_path) =
                        amc_mods::LocalModManager::toggle_mod(&self.local_mods[idx].path)
                    {
                        self.local_mods[idx].path = new_path;
                        self.local_mods[idx].enabled = !self.local_mods[idx].enabled;
                    }
                }

                if let Some(idx) = to_delete {
                    let _ = amc_mods::LocalModManager::delete_mod(&self.local_mods[idx].path);
                    self.local_mods.remove(idx);
                }
            });
    }

    fn show_search(
        &mut self,
        ui: &mut Ui,
        lang: Language,
        skin: ModsSkin,
        wishlist: &mut Vec<String>,
        on_search: impl FnOnce(String, ModSource, ModCategory, SearchSort),
        on_install: impl FnOnce(ModSearchResult),
    ) {
        let millida = skin == ModsSkin::Millida;
        let chip = |ui: &mut Ui, icon: Icon, label: &str, active: bool| -> bool {
            if millida {
                millida_action_button(ui, label, active)
            } else {
                icons::icon_text_button(
                    ui,
                    icon,
                    None,
                    label,
                    if active {
                        Color32::WHITE
                    } else {
                        TEXT_MUTED.into()
                    },
                    egui::FontId::proportional(11.0),
                    if active { RUBY } else { BG_ELEVATED },
                )
                .clicked()
            }
        };
        ui.horizontal_wrapped(|ui| {
            let mr_active = self.search_provider == ModSource::Modrinth;
            let cf_active = self.search_provider == ModSource::CurseForge;

            let mr_btn = egui::Button::new(
                egui::RichText::new("Modrinth")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if mr_active {
                        Color32::WHITE
                    } else {
                        TEXT_MUTED.into()
                    }),
            )
            .fill(if mr_active { RUBY } else { BG_HOVER })
            .min_size(vec2(80.0, 30.0));

            let mr_clicked = if millida {
                millida_action_button(ui, "Modrinth", mr_active)
            } else {
                ui.add(mr_btn).clicked()
            };
            if mr_clicked {
                self.search_provider = ModSource::Modrinth;
            }

            let cf_btn = egui::Button::new(
                egui::RichText::new("CurseForge")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if cf_active {
                        Color32::WHITE
                    } else {
                        TEXT_MUTED.into()
                    }),
            )
            .fill(if cf_active { RUBY } else { BG_HOVER })
            .min_size(vec2(90.0, 30.0));

            let cf_clicked = if millida {
                millida_action_button(ui, "CurseForge", cf_active)
            } else {
                ui.add(cf_btn).clicked()
            };
            if cf_clicked {
                self.search_provider = ModSource::CurseForge;
                self.search_category = ModCategory::Mod;
            }

            ui.add_space(8.0);

            let search_width = (ui.available_width()
                - 240.0
                - if self.search_query.is_empty() {
                    0.0
                } else {
                    30.0
                })
            .max(160.0);
            let hint = match self.search_provider {
                ModSource::Modrinth => lang.mods_search_hint("Modrinth"),
                ModSource::CurseForge => lang.mods_search_hint("CurseForge"),
                _ => lang.mods_search_hint("Mods"),
            };

            let search_id = egui::Id::new("mods_search_field");
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .id(search_id)
                .hint_text(egui::RichText::new(hint).color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(14.0));

            let resp = ui.add(search_edit);
            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if !self.search_query.is_empty()
                && icons::icon_button(ui, Icon::Close, TEXT_MUTED)
                    .on_hover_text(lang.mods_search_clear())
                    .clicked()
            {
                self.search_query.clear();
                ui.memory_mut(|m| m.request_focus(search_id));
            }
            if ui.memory(|m| m.has_focus(search_id))
                && ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.search_query.clear();
            }

            ui.add_space(8.0);

            let mut on_search = Some(on_search);
            let search_clicked = if millida {
                millida_action_button(ui, lang.mods_btn_search(), false)
            } else {
                ui.button(
                    egui::RichText::new(lang.mods_btn_search())
                        .strong()
                        .color(Color32::WHITE),
                )
                .clicked()
            };
            if search_clicked || enter_pressed {
                if let Some(f) = on_search.take() {
                    f(
                        self.search_query.clone(),
                        self.search_provider,
                        self.search_category,
                        self.search_sort,
                    );
                }
            }

            if self.search_provider == ModSource::Modrinth {
                ui.add_space(8.0);
                let btn_font = egui::TextStyle::Button.resolve(ui.style());
                let btn_fill = ui.visuals().widgets.inactive.weak_bg_fill;
                let trending_clicked = if millida {
                    millida_action_button(ui, lang.discover_title(), false)
                } else {
                    icons::icon_text_button(
                        ui,
                        Icon::Flame,
                        None,
                        lang.discover_title(),
                        TEXT_PRIMARY,
                        btn_font,
                        btn_fill,
                    )
                    .clicked()
                };
                if trending_clicked {
                    self.search_query.clear();
                    if let Some(f) = on_search.take() {
                        f(
                            String::new(),
                            self.search_provider,
                            self.search_category,
                            self.search_sort,
                        );
                    }
                }
            }
        });

        // Category selection row (Modrinth) + result ordering (both
        // providers: Modrinth sorts server-side, CurseForge client-side).
        {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if self.search_provider == ModSource::Modrinth {
                    let cat_mod = self.search_category == ModCategory::Mod;
                    let cat_rp = self.search_category == ModCategory::ResourcePack;
                    let cat_sh = self.search_category == ModCategory::Shader;

                    if chip(ui, Icon::Puzzle, lang.mods_cat_mods(), cat_mod) {
                        self.search_category = ModCategory::Mod;
                    }

                    if chip(ui, Icon::Palette, lang.mods_cat_resourcepacks(), cat_rp) {
                        self.search_category = ModCategory::ResourcePack;
                    }

                    if chip(ui, Icon::Sun, lang.mods_cat_shaders(), cat_sh) {
                        self.search_category = ModCategory::Shader;
                    }
                }

                // Result ordering (CONCEPT P4 "фильтр по популярности"):
                // Modrinth sorts server-side, CurseForge client-side.
                ui.add_space(12.0);
                let sort_rel = self.search_sort == SearchSort::Relevance;
                let sort_dl = self.search_sort == SearchSort::Downloads;
                if chip(ui, Icon::List, lang.mods_sort_relevance(), sort_rel) {
                    self.search_sort = SearchSort::Relevance;
                }
                if chip(ui, Icon::Download, lang.mods_sort_downloads(), sort_dl) {
                    self.search_sort = SearchSort::Downloads;
                }
            });
        }

        ui.add_space(14.0);

        if self.is_searching {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new(lang.mods_searching()).color(TEXT_MUTED));
            });
            return;
        }

        if self.search_results.is_empty() {
            let has_query = !self.search_query.trim().is_empty();
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
                if millida {
                    pixel::plate(
                        ui.painter(),
                        icon_rect,
                        6.0,
                        Color32::from(BG_CARD),
                        Color32::from(BORDER_DEFAULT),
                    );
                } else {
                    ui.painter()
                        .rect_filled(icon_rect, theme::rounding(), BG_CARD);
                    ui.painter().rect_stroke(
                        icon_rect,
                        theme::rounding(),
                        Stroke::new(1.0_f32, BORDER_DEFAULT),
                    );
                }
                icons::draw_centered(
                    ui.painter(),
                    Icon::Search,
                    icon_rect.center(),
                    24.0,
                    Color32::from(theme::ACCENT),
                );
                ui.add_space(12.0);
                let (title, hint) = if has_query {
                    (
                        lang.mods_search_no_results_title(),
                        lang.mods_search_no_results_hint(),
                    )
                } else {
                    (
                        lang.mods_search_prompt_title(),
                        lang.mods_search_prompt_hint(),
                    )
                };
                ui.label(
                    egui::RichText::new(title)
                        .font(egui::FontId::proportional(15.0))
                        .strong()
                        .color(TEXT_HEADING),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(hint)
                        .font(egui::FontId::proportional(12.0))
                        .color(TEXT_MUTED),
                );
                if has_query {
                    ui.add_space(12.0);
                    if chip(ui, Icon::Close, lang.mods_search_clear(), true) {
                        self.search_query.clear();
                    }
                }
            });
            return;
        }

        // Rebuild lowercased helpers only when inputs change (no per-row
        // `to_lowercase` per frame).
        {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            self.local_mods.len().hash(&mut hasher);
            for lm in &self.local_mods {
                lm.name.hash(&mut hasher);
            }
            let rev = hasher.finish();
            if rev != self.local_lower_rev {
                self.local_lower = self
                    .local_mods
                    .iter()
                    .map(|lm| lm.name.to_lowercase())
                    .collect();
                self.local_lower_rev = rev;
            }
            let first = self
                .search_results
                .first()
                .map(|r| r.id.as_str())
                .unwrap_or("");
            let last = self
                .search_results
                .last()
                .map(|r| r.id.as_str())
                .unwrap_or("");
            let srev = (
                self.search_results.len(),
                first.to_string(),
                last.to_string(),
            );
            if srev != self.search_rev {
                self.search_lower = self
                    .search_results
                    .iter()
                    .map(|r| r.title.to_lowercase())
                    .collect();
                self.search_rev = srev;
            }
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                let mut to_install = None;

                for (idx, item) in self.search_results.iter().enumerate() {
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 70.0), Sense::click());
                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let stroke_col = lerp_color(BORDER_DEFAULT, RUBY, hover_t);

                    ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
                    ui.painter().rect_stroke(
                        rect,
                        crate::theme::rounding(),
                        Stroke::new(1.0_f32, stroke_col),
                    );

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(14.0);

                    // Marketplace artwork (CONCEPT P10 previews): cached
                    // texture when fetched, source glyph otherwise.
                    let key = icon_key(&item.source, &item.id);
                    if let Some(tex) = self.icon_textures.get(&key) {
                        let (irect, _) =
                            child.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                        child.painter().image(
                            tex.id(),
                            irect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    } else {
                        let source_icon = match item.source {
                            ModSource::Modrinth => Icon::Globe,
                            ModSource::CurseForge => Icon::Flame,
                            ModSource::Local => Icon::Puzzle,
                        };
                        let (src_rect, _) =
                            child.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                        icons::draw_centered(
                            child.painter(),
                            source_icon,
                            src_rect.center(),
                            22.0,
                            TEXT_PRIMARY,
                        );
                    }
                    child.add_space(12.0);

                    // Title & Description with truncation
                    let max_card_text_w = (child.available_width() - 140.0).max(120.0);
                    child.vertical(|ui| {
                        ui.set_max_width(max_card_text_w);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&item.title)
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(TEXT_HEADING),
                            );
                            ui.label(
                                egui::RichText::new(format!("by {}", item.author))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_MUTED),
                            );
                            let src_tag = match item.source {
                                ModSource::Modrinth => "Modrinth",
                                ModSource::CurseForge => "CurseForge",
                                ModSource::Local => "Local",
                            };
                            ui.label(
                                egui::RichText::new(format!("• {src_tag}"))
                                    .font(egui::FontId::proportional(10.0))
                                    .color(RUBY_LIGHT),
                            );
                            icons::icon_label(
                                ui,
                                Icon::Download,
                                &item.downloads.to_string(),
                                TEXT_MUTED,
                                egui::FontId::proportional(10.0),
                            );
                            // Rating from the marketplace (Modrinth follows);
                            // unknown (0) stays silent instead of guessing.
                            if item.rating > 0 {
                                icons::icon_label(
                                    ui,
                                    Icon::Star,
                                    &item.rating.to_string(),
                                    TEXT_MUTED,
                                    egui::FontId::proportional(10.0),
                                );
                            }
                        });

                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&item.description)
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            )
                            .truncate(),
                        );
                        // Compat hint (CONCEPT P4): known-good builds get a
                        // check against the selected instance; unknown
                        // metadata stays silent instead of guessing.
                        if !item.game_versions.is_empty() {
                            let mut versions = item.game_versions.join(" ");
                            if versions.chars().count() > 42 {
                                versions =
                                    format!("{}…", versions.chars().take(42).collect::<String>());
                            }
                            let fits = self
                                .compat_mc
                                .as_deref()
                                .is_some_and(|mc| item.fits(mc, &self.compat_loader));
                            let mut hint = versions;
                            if !item.loaders.is_empty() {
                                hint.push_str(&format!(" • {}", item.loaders.join(" ")));
                            }
                            let hint_color = if fits { SUCCESS } else { TEXT_MUTED };
                            if fits {
                                icons::icon_label(
                                    ui,
                                    Icon::Check,
                                    &hint,
                                    hint_color,
                                    egui::FontId::proportional(10.0),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(hint)
                                        .font(egui::FontId::proportional(10.0))
                                        .color(hint_color),
                                );
                            }
                        }
                    });

                    // Install button or Installed status
                    let is_downloading = self.installing_ids.contains(&item.id);
                    let wish_key = match item.source {
                        ModSource::Modrinth => Some(format!("modrinth:{}", item.id)),
                        ModSource::CurseForge => Some(format!("curseforge:{}", item.id)),
                        ModSource::Local => None,
                    };
                    if let Some(key) = wish_key {
                        let wished = wishlist.contains(&key);
                        if icon_button(
                            &mut child,
                            if wished {
                                Icon::Heart
                            } else {
                                Icon::HeartOutline
                            },
                            if wished { RUBY_LIGHT } else { TEXT_MUTED },
                        )
                        .on_hover_text(if wished {
                            lang.wishlist_remove()
                        } else {
                            lang.wishlist_add()
                        })
                        .clicked()
                        {
                            if wished {
                                wishlist.retain(|k| k != &key);
                            } else {
                                wishlist.push(key);
                            }
                        }
                        child.add_space(6.0);
                    }
                    let is_installed = {
                        // `search_lower` is rebuilt above in lockstep with
                        // `search_results`; indexing avoids per-row allocs.
                        let title_lower = self
                            .search_lower
                            .get(idx)
                            .map(String::as_str)
                            .unwrap_or(item.title.as_str());
                        // Fallback path lowercases once (only on cache skew).
                        let owned;
                        let title_lower = if self.search_lower.get(idx).is_some() {
                            title_lower
                        } else {
                            owned = item.title.to_lowercase();
                            owned.as_str()
                        };
                        self.installed_titles.contains(title_lower)
                            || self.local_lower.iter().any(|lm_lower| {
                                lm_lower == title_lower || lm_lower.contains(title_lower)
                            })
                    };

                    let btn_width = 120.0;
                    let avail = child.available_width() - btn_width - 16.0;
                    if avail > 0.0 {
                        child.add_space(avail);
                    }

                    if is_downloading {
                        let btn = egui::Button::new(
                            egui::RichText::new(lang.mods_btn_downloading())
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        )
                        .fill(BG_HOVER)
                        .min_size(vec2(100.0, 30.0));
                        child.add(btn);
                    } else if is_installed {
                        let badge = egui::Button::new(
                            egui::RichText::new(lang.mods_badge_installed())
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(SUCCESS)
                        .min_size(vec2(100.0, 30.0));
                        child.add(badge);
                    } else {
                        let (ib_rect, ib_resp) =
                            child.allocate_exact_size(vec2(90.0, 30.0), Sense::click());
                        let ib_hover = child
                            .ctx()
                            .animate_bool_responsive(ib_resp.id, ib_resp.hovered());
                        let ib_bg = lerp_color(RUBY, RUBY_LIGHT, ib_hover);
                        let ib_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, ib_hover);

                        child
                            .painter()
                            .rect_filled(ib_rect, crate::theme::rounding(), ib_bg);
                        child.painter().rect_stroke(
                            ib_rect,
                            crate::theme::rounding(),
                            Stroke::new(1.0_f32, ib_stroke),
                        );
                        child.painter().text(
                            ib_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            lang.mods_btn_install(),
                            egui::FontId::proportional(11.5),
                            Color32::WHITE,
                        );

                        if ib_resp.clicked() {
                            self.installing_ids.insert(item.id.clone());
                            to_install = Some(item.clone());
                        }
                    }
                }

                if let Some(item) = to_install {
                    on_install(item);
                }
            });
    }
}

fn multimc_button(ui: &mut Ui, label: &str, active: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill = if active {
        RUBY.into()
    } else {
        lerp_color(BG_ELEVATED, BG_HOVER, hover)
    };
    let stroke = if active {
        RUBY.into()
    } else {
        lerp_color(BORDER_DEFAULT, RUBY, hover)
    };
    ui.painter()
        .rect_filled(rect, crate::theme::rounding(), fill);
    ui.painter()
        .rect_stroke(rect, crate::theme::rounding(), Stroke::new(1.0_f32, stroke));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if active {
            Color32::WHITE
        } else {
            TEXT_PRIMARY.into()
        },
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn millida_pixel_rect(painter: &egui::Painter, rect: egui::Rect, cut: f32, fill: Color32) {
    let c = cut
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5)
        .max(0.0);
    painter.rect_filled(
        egui::Rect::from_min_max(
            pos2(rect.left() + c, rect.top()),
            pos2(rect.right() - c, rect.top() + c),
        ),
        egui::Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            pos2(rect.left(), rect.top() + c),
            pos2(rect.right(), rect.bottom() - c),
        ),
        egui::Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            pos2(rect.left() + c, rect.bottom() - c),
            pos2(rect.right() - c, rect.bottom()),
        ),
        egui::Rounding::ZERO,
        fill,
    );
}

fn millida_pixel_plate(
    painter: &egui::Painter,
    rect: egui::Rect,
    cut: f32,
    fill: Color32,
    outline: Color32,
) {
    millida_pixel_rect(painter, rect, cut, outline);
    millida_pixel_rect(painter, rect.shrink(1.0), cut, fill);
}

/// Small cut-corner action used by the Millida mods list.
fn millida_action_button(ui: &mut Ui, label: &str, active: bool) -> bool {
    let font = egui::FontId::new(11.5, egui::FontFamily::Name("bold".into()));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(galley.size().x + 20.0, 30.0), egui::Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let (fill, outline, text): (Color32, Color32, Color32) = if active {
        (
            crate::theme::ACCENT.into(),
            crate::theme::ACCENT_LIGHT.into(),
            crate::theme::BG.into(),
        )
    } else {
        (
            lerp_color(crate::theme::CARD, crate::theme::CARD_HOVER, hover),
            lerp_color(
                crate::theme::BORDER_STRONG,
                crate::theme::ACCENT_LIGHT,
                hover,
            ),
            crate::theme::TEXT.into(),
        )
    };
    millida_pixel_plate(ui.painter(), rect, 4.0, fill, outline);
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Cut-corner checkbox used instead of the native rounded checkbox.
fn millida_check(ui: &mut Ui, selected: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let outline = if selected {
        crate::theme::ACCENT_LIGHT.into()
    } else {
        lerp_color(
            crate::theme::BORDER_STRONG,
            crate::theme::ACCENT_LIGHT,
            hover,
        )
    };
    let fill: Color32 = if selected {
        crate::theme::ACCENT.into()
    } else {
        crate::theme::CARD_HOVER.into()
    };
    millida_pixel_plate(ui.painter(), rect, 4.0, fill, outline);
    if selected {
        let check_color: Color32 = crate::theme::BG.into();
        icons::draw_centered(ui.painter(), Icon::Check, rect.center(), 12.0, check_color);
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn labymod_mods_header(ui: &mut Ui, lang: Language) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::Rounding::same(16.0), BG_CARD);
    painter.rect_stroke(
        rect,
        egui::Rounding::same(16.0),
        Stroke::new(1.0_f32, BORDER_DEFAULT),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 22.0),
        egui::Align2::LEFT_CENTER,
        lang.nav_mods().to_uppercase(),
        egui::FontId::proportional(10.5),
        crate::theme::ACCENT_LIGHT.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 47.0),
        egui::Align2::LEFT_CENTER,
        lang.mods_title(),
        egui::FontId::new(27.0, egui::FontFamily::Name("bold".into())),
        TEXT_HEADING.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 74.0),
        egui::Align2::LEFT_CENTER,
        lang.labymod_mods_subtitle(),
        egui::FontId::proportional(11.5),
        TEXT_MUTED.into(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_key_scopes_sources() {
        assert_eq!(
            icon_key(&ModSource::Modrinth, "42"),
            icon_key(&ModSource::Modrinth, "42")
        );
        assert_ne!(
            icon_key(&ModSource::Modrinth, "42"),
            icon_key(&ModSource::CurseForge, "42")
        );
    }

    #[test]
    fn test_icon_cache_file_flattens() {
        assert_eq!(icon_cache_file("mr:sodium"), "mr_sodium");
        assert_eq!(icon_cache_file("cf:123"), "cf_123");
    }

    #[test]
    fn test_changelog_preview() {
        let arrow = '→';
        let ellipsis = '…';
        assert_eq!(changelog_preview("2.0", &None), format!("{arrow} 2.0"));
        assert!(changelog_preview("2.0", &Some("a b".to_string())).contains("a b"));
        let long = "x".repeat(400);
        let preview = changelog_preview("3.0", &Some(long));
        assert!(preview.starts_with(&format!("{arrow} 3.0")));
        assert!(preview.ends_with(ellipsis));
    }
}
