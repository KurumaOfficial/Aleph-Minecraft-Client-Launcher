use crate::theme::{
    lerp_color, ACCENT_DIM, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY,
    RUBY_DIM, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::draw_custom_badge;
use crate::widgets::full::{ghost_link, image_cover, kicker, red_button, red_square_deco, tag};
use amc_core::types::{GameVersion, ReleaseType};
use amc_minecraft::ServerStatus;
use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterTab {
    All,
    #[default]
    Releases,
    Snapshots,
    Betas,
    Alphas,
    Old,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Newest,
    Oldest,
}

pub struct HomePage {
    pub search_query: String,
    pub active_filter: FilterTab,
    pub sort_order: SortOrder,
    pub featured_server: Option<ServerStatus>,
    pub is_pinging: bool,
    pub direct_connect_request: Option<(String, u16)>,
    pub hero_play_request: bool,
    pub drawer_play_request: Option<String>,
    drawer_open: bool,
    drawer_version: Option<String>,
    hero_texture: Option<egui::TextureHandle>,
    hero_failed: bool,
    installed_cache: HashSet<String>,
    installed_for: usize,
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_filter: FilterTab::Releases,
            sort_order: SortOrder::Newest,
            featured_server: None,
            is_pinging: false,
            direct_connect_request: None,
            hero_play_request: false,
            drawer_play_request: None,
            drawer_open: false,
            drawer_version: None,
            hero_texture: None,
            hero_failed: false,
            installed_cache: HashSet::new(),
            installed_for: usize::MAX,
        }
    }
}

/// Ambient data the Home page needs beyond the version list itself.
/// Bundled so `show` stays under the argument-count lint.
pub struct HomeContext<'a> {
    pub versions_dir: &'a Path,
    pub instances_count: usize,
    pub memory_mb: u32,
}

impl HomePage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        ctx_data: &HomeContext,
        lang: amc_core::Language,
    ) {
        self.refresh_installed(versions, ctx_data.versions_dir);
        self.ensure_hero_texture(ui.ctx());
        self.show_hero(
            ui,
            versions,
            selected_version,
            ctx_data.instances_count,
            ctx_data.memory_mb,
            lang,
        );

        ui.add_space(20.0);

        // Section Header: Versions + Selected Badge
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.home_title())
                    .font(egui::FontId::proportional(26.0))
                    .strong()
                    .color(TEXT_HEADING),
            );

            if let Some(sel) = selected_version.as_deref() {
                ui.add_space(12.0);
                draw_custom_badge(ui, &lang.home_selected_badge(sel), RUBY);
            }
        });

        ui.add_space(14.0);

        // Control Row (Search + Sort)
        ui.horizontal(|ui| {
            // Search Input
            let search_width = (ui.available_width() - 170.0).max(180.0);
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new(lang.home_search_hint()).color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(13.0));

            ui.add(search_edit);

            ui.add_space(8.0);

            // Sort Toggle Button
            let sort_label = match self.sort_order {
                SortOrder::Newest => format!("{} ▾", lang.home_sort_newest()),
                SortOrder::Oldest => format!("{} ▴", lang.home_sort_oldest()),
            };

            let (sort_rect, sort_resp) = ui.allocate_exact_size(vec2(140.0, 32.0), Sense::click());
            let sort_hover = ui
                .ctx()
                .animate_bool_responsive(sort_resp.id, sort_resp.hovered());
            let sort_bg = lerp_color(BG_ELEVATED, BG_HOVER, sort_hover);
            let sort_stroke = lerp_color(BORDER_DEFAULT, RUBY, sort_hover);
            let sort_text = lerp_color(TEXT_PRIMARY, Color32::WHITE, sort_hover);

            ui.painter().rect_filled(sort_rect, Rounding::ZERO, sort_bg);
            ui.painter()
                .rect_stroke(sort_rect, Rounding::ZERO, Stroke::new(1.0, sort_stroke));
            ui.painter().text(
                sort_rect.center(),
                egui::Align2::CENTER_CENTER,
                &sort_label,
                egui::FontId::proportional(12.0),
                sort_text,
            );

            if sort_resp.clicked() {
                self.sort_order = match self.sort_order {
                    SortOrder::Newest => SortOrder::Oldest,
                    SortOrder::Oldest => SortOrder::Newest,
                };
            }
        });

        ui.add_space(10.0);

        // Filter Tabs
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
            self.filter_tab_btn(ui, lang.home_tab_all(), FilterTab::All);
            self.filter_tab_btn(ui, lang.home_tab_releases(), FilterTab::Releases);
            self.filter_tab_btn(ui, lang.home_tab_snapshots(), FilterTab::Snapshots);
            self.filter_tab_btn(ui, lang.home_tab_betas(), FilterTab::Betas);
            self.filter_tab_btn(ui, lang.home_tab_alphas(), FilterTab::Alphas);
            self.filter_tab_btn(ui, lang.home_tab_old(), FilterTab::Old);
        });

        ui.add_space(10.0);

        // Filter and sort version list
        let mut filtered: Vec<&GameVersion> = versions
            .iter()
            .filter(|v| {
                let matches_filter = match self.active_filter {
                    FilterTab::All => true,
                    FilterTab::Releases => v.release_type == ReleaseType::Release,
                    FilterTab::Snapshots => v.release_type == ReleaseType::Snapshot,
                    FilterTab::Betas => v.release_type == ReleaseType::Beta,
                    FilterTab::Alphas => v.release_type == ReleaseType::Alpha,
                    FilterTab::Old => v.release_type == ReleaseType::Old,
                };

                let matches_search = if self.search_query.trim().is_empty() {
                    true
                } else {
                    v.id.to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                };

                matches_filter && matches_search
            })
            .collect();

        if self.sort_order == SortOrder::Oldest {
            filtered.sort_by_key(|v| v.release_time);
        } else {
            filtered.sort_by_key(|v| std::cmp::Reverse(v.release_time));
        }

        if filtered.is_empty() {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                if versions.is_empty() {
                    ui.spinner();
                    ui.add_space(8.0);
                    let loading_msg = match lang {
                        amc_core::Language::English => "Fetching Minecraft versions from Mojang...",
                        amc_core::Language::Russian => "Загрузка списка версий от Mojang...",
                        amc_core::Language::Ukrainian => "Завантаження списку версій від Mojang...",
                    };
                    ui.label(
                        egui::RichText::new(loading_msg)
                            .font(egui::FontId::proportional(15.0))
                            .color(TEXT_MUTED),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(lang.home_empty())
                            .font(egui::FontId::proportional(15.0))
                            .color(TEXT_MUTED),
                    );
                }
            });
            return;
        }

        // Virtualized ScrollArea
        let row_height = 56.0;
        let num_rows = filtered.len();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                for idx in row_range {
                    let v = filtered[idx];
                    let is_selected = selected_version.as_deref() == Some(&v.id);

                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::click());

                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

                    let base_bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let bg = lerp_color(base_bg, RUBY_DIM, sel_t);

                    let stroke_col =
                        lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, sel_t);
                    let border_width = if is_selected { 1.5 } else { 1.0 };

                    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
                    ui.painter().rect_stroke(
                        rect,
                        Rounding::ZERO,
                        Stroke::new(border_width, stroke_col),
                    );

                    // Active left indicator bar (Square motif)
                    if sel_t > 0.01 {
                        let bar_rect = egui::Rect::from_min_max(
                            rect.left_top(),
                            egui::pos2(rect.left() + 3.0 * sel_t, rect.bottom()),
                        );
                        ui.painter()
                            .rect_filled(bar_rect, Rounding::ZERO, RUBY_LIGHT);
                    }

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );

                    child.add_space(14.0);

                    // Version type icon
                    let icon_glyph = match v.release_type {
                        ReleaseType::Release => "📦",
                        ReleaseType::Snapshot => "🧪",
                        ReleaseType::Beta => "⚙",
                        ReleaseType::Alpha => "🔨",
                        ReleaseType::Old => "📜",
                    };
                    child.label(
                        egui::RichText::new(icon_glyph).font(egui::FontId::proportional(18.0)),
                    );
                    child.add_space(10.0);

                    // Version ID
                    let ver_color = lerp_color(
                        lerp_color(TEXT_HEADING, Color32::WHITE, hover_t),
                        Color32::WHITE,
                        sel_t,
                    );
                    child.label(
                        egui::RichText::new(&v.id)
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(ver_color),
                    );

                    child.add_space(12.0);

                    // Release type badge
                    let badge_color = match v.release_type {
                        ReleaseType::Release => RUBY,
                        ReleaseType::Snapshot => Color32::from_rgb(0xB8, 0x86, 0x0B),
                        _ => Color32::from_rgb(0x4A, 0x55, 0x68),
                    };
                    draw_custom_badge(&mut child, v.release_type.as_str(), badge_color);

                    // Release date on right
                    let date_str = v.release_time.format("%d.%m.%Y").to_string();
                    let date_width = 80.0;
                    let space = child.available_width() - date_width - 16.0;
                    if space > 0.0 {
                        child.add_space(space);
                    }

                    child.label(
                        egui::RichText::new(date_str)
                            .font(egui::FontId::proportional(12.0))
                            .color(lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t)),
                    );

                    if resp.clicked() {
                        *selected_version = Some(v.id.clone());
                        self.open_drawer(Some(v.id.clone()));
                    }
                }
            });

        self.show_drawer(ui, versions, selected_version, lang);
    }

    pub fn open_drawer(&mut self, id: Option<String>) {
        self.drawer_version = id.clone();
        self.drawer_open = id.is_some();
    }

    pub fn close_drawer(&mut self) {
        self.drawer_open = false;
        self.drawer_version = None;
    }

    /// Rescan downloaded versions when the manifest list changes.
    fn refresh_installed(&mut self, versions: &[GameVersion], versions_dir: &Path) {
        if versions.len() == self.installed_for {
            return;
        }
        self.installed_for = versions.len();
        self.installed_cache.clear();
        let entries = match std::fs::read_dir(versions_dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let has_json = std::fs::read_dir(&dir).map(|inner| {
                inner.flatten().any(|f| {
                    f.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
            });
            if has_json.unwrap_or(false) {
                self.installed_cache
                    .insert(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    fn ensure_hero_texture(&mut self, ctx: &egui::Context) {
        if self.hero_texture.is_some() || self.hero_failed {
            return;
        }
        match image::load_from_memory(include_bytes!("../../../../assets/png/back_versions.png")) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                let pixels: Vec<Color32> = rgba
                    .pixels()
                    .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
                    .collect();
                let tex = ctx.load_texture(
                    "home_hero",
                    egui::ColorImage {
                        size: [w, h],
                        pixels,
                    },
                    egui::TextureOptions::LINEAR,
                );
                self.hero_texture = Some(tex);
            }
            Err(_) => self.hero_failed = true,
        }
    }

    /// Full-style hero: cover art, red-square deco, display titles,
    /// big Play + ghost pick link, stats row.
    fn show_hero(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        instances_count: usize,
        memory_mb: u32,
        lang: amc_core::Language,
    ) {
        let width = ui.available_width();
        let hero_h = 300.0;
        let (hero, _) = ui.allocate_exact_size(vec2(width, hero_h), Sense::hover());
        let painter = ui.painter().clone();
        match &self.hero_texture {
            Some(tex) => image_cover(&painter, hero, tex, Color32::WHITE),
            None => {
                painter.rect_filled(hero, Rounding::ZERO, BG_CARD);
            }
        }
        painter.rect_filled(
            hero,
            Rounding::ZERO,
            Color32::from_rgba_unmultiplied(8, 6, 6, 208),
        );

        let deco_side = 360.0_f32.min(hero_h - 60.0);
        let deco = egui::Rect::from_center_size(
            egui::pos2(hero.right() - 90.0, hero.center().y),
            vec2(deco_side, deco_side),
        );
        red_square_deco(&painter, deco);

        let text_rect = egui::Rect::from_min_max(
            egui::pos2(hero.left() + 40.0, hero.top() + 32.0),
            egui::pos2(hero.right() - 40.0, hero.bottom() - 24.0),
        );
        let mut play_clicked = false;
        let mut pick_clicked = false;
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if red_button(ui, lang.bottom_btn_play(), 300.0, 50.0).clicked() {
                        play_clicked = true;
                    }
                    ui.add_space(24.0);
                    if ghost_link(ui, lang.home_hero_pick()).clicked() {
                        pick_clicked = true;
                    }
                });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(lang.home_hero_hint())
                        .font(crate::theme::serif_it(17.0))
                        .color(TEXT_MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("LAUNCHER")
                        .font(crate::theme::display(54.0))
                        .color(RUBY_LIGHT),
                );
                ui.label(
                    egui::RichText::new("ALEPH")
                        .font(crate::theme::display(54.0))
                        .color(TEXT_HEADING),
                );
                ui.add_space(10.0);
                kicker(ui, "Minecraft launcher");
            });
        });
        if play_clicked {
            self.hero_play_request = true;
        }
        if pick_clicked {
            self.open_drawer(
                selected_version
                    .clone()
                    .or_else(|| versions.first().map(|v| v.id.clone())),
            );
        }

        ui.add_space(26.0);

        let stats = [
            (
                "01",
                format!("{:02}", versions.len()),
                lang.home_stat_versions(),
            ),
            (
                "02",
                format!("{:02}", instances_count),
                lang.home_stat_instances(),
            ),
            ("03", format!("{memory_mb} MB"), lang.home_stat_ram()),
        ];
        ui.horizontal(|ui| {
            let w = ((width - 40.0) / 3.0).max(180.0);
            for (num, big, small) in stats {
                ui.vertical(|ui| {
                    ui.set_width(w);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(ui.min_rect().min, vec2(w, 1.0)),
                        Rounding::ZERO,
                        BORDER_STRONG,
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(num)
                            .font(crate::theme::mono(11.0))
                            .color(ACCENT_DIM),
                    );
                    ui.label(
                        egui::RichText::new(big)
                            .font(crate::theme::display(22.0))
                            .color(TEXT_HEADING),
                    );
                    ui.label(
                        egui::RichText::new(small)
                            .font(crate::theme::mono(10.0))
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(20.0);
            }
        });

        ui.add_space(10.0);
    }

    /// Full-style details drawer for the selected version: dim backdrop,
    /// slide-in panel, click-outside to close, Play launches directly.
    fn show_drawer(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        lang: amc_core::Language,
    ) {
        let ctx = ui.ctx().clone();
        let anim = ctx.animate_bool_with_time(egui::Id::new("home_drawer"), self.drawer_open, 0.18);
        if !self.drawer_open && anim <= 0.001 {
            return;
        }
        let screen = ctx.screen_rect();
        let slide = (1.0 - anim) * 60.0;
        let panel = egui::Rect::from_min_max(
            egui::pos2(screen.right() - 400.0 - 14.0 + slide, screen.top() + 14.0),
            egui::pos2(screen.right() - 14.0 + slide, screen.bottom() - 14.0),
        );

        let mut close = false;
        let mut play_id: Option<String> = None;

        egui::Area::new(egui::Id::new("home_drawer_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(&ctx, |ui| {
                ui.set_min_size(screen.size());
                let painter = ui.painter().clone();
                let dim = egui::Rect::from_min_size(screen.min, screen.size());
                painter.rect_filled(
                    dim,
                    Rounding::ZERO,
                    Color32::from_rgba_unmultiplied(0, 0, 0, (120.0 * anim) as u8),
                );
                let dim_resp = ui.interact(dim, ui.id().with("drawer-dim"), Sense::click());
                if dim_resp.clicked() {
                    if let Some(pos) = dim_resp.interact_pointer_pos() {
                        if !panel.contains(pos) {
                            close = true;
                        }
                    }
                }

                painter.rect_filled(panel, Rounding::ZERO, BG_CARD);
                painter.rect_stroke(panel, Rounding::ZERO, Stroke::new(1.0_f32, BORDER_DEFAULT));

                let inner = panel.shrink(18.0);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(lang.home_title())
                                .font(crate::theme::display(20.0))
                                .color(TEXT_HEADING),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if tag(ui, lang.home_drawer_close(), false).clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(12.0);

                    match versions
                        .iter()
                        .find(|v| Some(&v.id) == self.drawer_version.as_ref())
                    {
                        Some(v) => {
                            ui.label(
                                egui::RichText::new(&v.id)
                                    .font(crate::theme::display(20.0))
                                    .color(TEXT_HEADING),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} • {}",
                                    v.release_type.as_str(),
                                    v.release_time.format("%d.%m.%Y")
                                ))
                                .font(crate::theme::mono(11.0))
                                .color(TEXT_MUTED),
                            );
                            ui.add_space(10.0);
                            let installed = self.installed_cache.contains(&v.id);
                            let _ = tag(
                                ui,
                                if installed {
                                    lang.home_drawer_installed()
                                } else {
                                    lang.home_drawer_not_installed()
                                },
                                installed,
                            );
                            ui.add_space(16.0);
                            if red_button(ui, lang.inst_btn_play(), 260.0, 44.0).clicked() {
                                *selected_version = Some(v.id.clone());
                                play_id = Some(v.id.clone());
                                close = true;
                            }
                        }
                        None => {
                            ui.label(
                                egui::RichText::new(lang.home_empty())
                                    .font(crate::theme::serif_it(15.0))
                                    .color(TEXT_MUTED),
                            );
                        }
                    }
                });
            });

        if close {
            self.close_drawer();
        }
        if let Some(id) = play_id {
            self.drawer_play_request = Some(id);
        }
    }

    fn filter_tab_btn(&mut self, ui: &mut Ui, label: &str, tab: FilterTab) {
        let is_active = self.active_filter == tab;
        let (rect, resp) = ui.allocate_exact_size(vec2(78.0, 28.0), Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active_t = ui.ctx().animate_bool(resp.id.with("act"), is_active);

        let bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, hover_t), RUBY, active_t);
        let stroke_col = lerp_color(
            lerp_color(BORDER_DEFAULT, RUBY, hover_t),
            RUBY_LIGHT,
            active_t,
        );
        let text_col = lerp_color(
            lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
            Color32::WHITE,
            active_t,
        );

        ui.painter().rect_filled(rect, Rounding::ZERO, bg);
        ui.painter()
            .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, stroke_col));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_col,
        );

        if resp.clicked() {
            self.active_filter = tab;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_open_close() {
        let mut page = HomePage::default();
        assert!(!page.drawer_open);
        page.open_drawer(Some("1.20.1".to_string()));
        assert!(page.drawer_open);
        assert_eq!(page.drawer_version.as_deref(), Some("1.20.1"));
        page.close_drawer();
        assert!(!page.drawer_open);
        assert_eq!(page.drawer_version, None);
    }

    #[test]
    fn test_drawer_none_stays_closed() {
        let mut page = HomePage::default();
        page.open_drawer(None);
        assert!(!page.drawer_open);
    }
}
