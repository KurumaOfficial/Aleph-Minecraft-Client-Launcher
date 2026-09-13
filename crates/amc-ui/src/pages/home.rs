use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_core::types::{GameVersion, ReleaseType};
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::draw_badge;

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
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_filter: FilterTab::Releases,
            sort_order: SortOrder::Newest,
        }
    }
}

impl HomePage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
    ) {
        ui.add_space(20.0);

        // Page title
        ui.label(
            egui::RichText::new("Версии Minecraft")
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Control Row (Search + Sort)
        ui.horizontal(|ui| {
            // Search Input
            let search_width = (ui.available_width() - 170.0).max(200.0);
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new("🔍 Поиск версий...").color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(14.0));

            ui.add(search_edit);

            ui.add_space(10.0);

            // Sort Toggle Button
            let sort_label = match self.sort_order {
                SortOrder::Newest => "Сначала новые ▾",
                SortOrder::Oldest => "Сначала старые ▴",
            };

            let sort_btn = egui::Button::new(
                egui::RichText::new(sort_label)
                    .font(egui::FontId::proportional(13.0))
                    .color(TEXT_PRIMARY),
            )
            .fill(BG_ELEVATED)
            .stroke(Stroke::new(1.0, BORDER_DEFAULT))
            .min_size(vec2(150.0, 36.0));

            if ui.add(sort_btn).clicked() {
                self.sort_order = match self.sort_order {
                    SortOrder::Newest => SortOrder::Oldest,
                    SortOrder::Oldest => SortOrder::Newest,
                };
            }
        });

        ui.add_space(12.0);

        // Filter Tabs
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
            self.filter_tab_btn(ui, "ВСЕ", FilterTab::All);
            self.filter_tab_btn(ui, "РЕЛИЗЫ", FilterTab::Releases);
            self.filter_tab_btn(ui, "СНАПШОТЫ", FilterTab::Snapshots);
            self.filter_tab_btn(ui, "БЕТА", FilterTab::Betas);
            self.filter_tab_btn(ui, "АЛЬФА", FilterTab::Alphas);
            self.filter_tab_btn(ui, "СТАРЫЕ", FilterTab::Old);
        });

        ui.add_space(12.0);

        // Filter and sort version list
        let mut filtered: Vec<&GameVersion> = versions
            .iter()
            .filter(|v| {
                // Filter tab
                let matches_filter = match self.active_filter {
                    FilterTab::All => true,
                    FilterTab::Releases => v.release_type == ReleaseType::Release,
                    FilterTab::Snapshots => v.release_type == ReleaseType::Snapshot,
                    FilterTab::Betas => v.release_type == ReleaseType::Beta,
                    FilterTab::Alphas => v.release_type == ReleaseType::Alpha,
                    FilterTab::Old => v.release_type == ReleaseType::Old,
                };

                // Search query
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

        // Versions List
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                if filtered.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Версии не найдены")
                                .font(egui::FontId::proportional(16.0))
                                .color(TEXT_MUTED),
                        );
                    });
                    return;
                }

                for version in filtered {
                    let is_selected = selected_version.as_deref() == Some(&version.id);
                    if self.version_row(ui, version, is_selected) {
                        *selected_version = Some(version.id.clone());
                    }
                }
            });
    }

    fn filter_tab_btn(&mut self, ui: &mut Ui, label: &str, tab: FilterTab) {
        let is_active = self.active_filter == tab;
        let (bg, stroke, text_color) = if is_active {
            (RUBY, Stroke::new(2.0, RUBY), Color32::WHITE)
        } else {
            (Color32::TRANSPARENT, Stroke::new(1.0, BORDER_DEFAULT), TEXT_MUTED)
        };

        let btn = egui::Button::new(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(11.0))
                .strong()
                .color(text_color),
        )
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::ZERO)
        .min_size(vec2(76.0, 30.0));

        if ui.add(btn).clicked() {
            self.active_filter = tab;
        }
    }

    fn version_row(&self, ui: &mut Ui, version: &GameVersion, is_selected: bool) -> bool {
        let row_height = 46.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), row_height), Sense::click());

        let bg = if is_selected {
            RUBY_DIM
        } else if resp.hovered() {
            BG_HOVER
        } else {
            BG_ELEVATED
        };

        let border_stroke = if is_selected {
            Stroke::new(1.5, RUBY)
        } else if resp.hovered() {
            Stroke::new(1.0, RUBY)
        } else {
            Stroke::new(1.0, BORDER_DEFAULT)
        };

        ui.painter().rect_filled(rect, Rounding::ZERO, bg);
        ui.painter().rect_stroke(rect, Rounding::ZERO, border_stroke);

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.add_space(14.0);

        // Icon placeholder
        child.label(egui::RichText::new("🟩").font(egui::FontId::proportional(16.0)));
        child.add_space(10.0);

        // Version ID
        child.label(
            egui::RichText::new(&version.id)
                .font(egui::FontId::proportional(15.0))
                .strong()
                .color(TEXT_HEADING),
        );

        child.add_space(12.0);

        // Badge
        draw_badge(&mut child, version.release_type);

        // Date right aligned
        let date_str = version.release_time.format("%d.%m.%Y").to_string();
        let date_width = 80.0;
        let available = child.available_width() - date_width - 16.0;
        if available > 0.0 {
            child.add_space(available);
        }

        child.label(
            egui::RichText::new(date_str)
                .font(egui::FontId::proportional(12.0))
                .color(TEXT_MUTED),
        );

        resp.clicked()
    }
}
