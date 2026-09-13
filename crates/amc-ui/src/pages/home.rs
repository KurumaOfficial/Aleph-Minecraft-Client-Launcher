use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_core::types::{GameVersion, ReleaseType};
use amc_minecraft::ServerStatus;
use crate::theme::{
    lerp_color, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, RUBY_LIGHT,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::draw_custom_badge;

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
        }
    }
}

impl HomePage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        lang: amc_core::Language,
    ) {
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
            let sort_hover = ui.ctx().animate_bool_responsive(sort_resp.id, sort_resp.hovered());
            let sort_bg = lerp_color(BG_ELEVATED, BG_HOVER, sort_hover);
            let sort_stroke = lerp_color(BORDER_DEFAULT, RUBY, sort_hover);
            let sort_text = lerp_color(TEXT_PRIMARY, Color32::WHITE, sort_hover);

            ui.painter().rect_filled(sort_rect, Rounding::ZERO, sort_bg);
            ui.painter().rect_stroke(sort_rect, Rounding::ZERO, Stroke::new(1.0, sort_stroke));
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
                    v.id.to_lowercase().contains(&self.search_query.to_lowercase())
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

                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 50.0),
                        Sense::click(),
                    );

                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

                    let base_bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let bg = lerp_color(base_bg, RUBY_DIM, sel_t);

                    let stroke_col = lerp_color(
                        lerp_color(BORDER_DEFAULT, RUBY, hover_t),
                        RUBY_LIGHT,
                        sel_t,
                    );
                    let border_width = if is_selected { 1.5 } else { 1.0 };

                    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(border_width, stroke_col));

                    // Active left indicator bar (Square motif)
                    if sel_t > 0.01 {
                        let bar_rect = egui::Rect::from_min_max(rect.left_top(), egui::pos2(rect.left() + 3.0 * sel_t, rect.bottom()));
                        ui.painter().rect_filled(bar_rect, Rounding::ZERO, RUBY_LIGHT);
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
                    child.label(egui::RichText::new(icon_glyph).font(egui::FontId::proportional(18.0)));
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
                    }
                }
            });
    }

    fn filter_tab_btn(&mut self, ui: &mut Ui, label: &str, tab: FilterTab) {
        let is_active = self.active_filter == tab;
        let (rect, resp) = ui.allocate_exact_size(vec2(78.0, 28.0), Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active_t = ui.ctx().animate_bool(resp.id.with("act"), is_active);

        let bg = lerp_color(
            lerp_color(BG_ELEVATED, BG_HOVER, hover_t),
            RUBY,
            active_t,
        );
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
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, stroke_col));
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
