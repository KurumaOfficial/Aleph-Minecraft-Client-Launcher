use egui::{vec2, Color32, Pos2, Rect, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_core::types::{GameVersion, ReleaseType};
use amc_minecraft::ServerStatus;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_DIM, RUBY_LIGHT, SUCCESS,
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
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_filter: FilterTab::Releases,
            sort_order: SortOrder::Newest,
            featured_server: None,
            is_pinging: false,
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
        ui.add_space(14.0);

        // 1. Hero Banner Card
        self.draw_hero_banner(ui);

        ui.add_space(16.0);

        // 2. Section Header: Versions
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Версии Minecraft")
                    .font(egui::FontId::proportional(22.0))
                    .strong()
                    .color(TEXT_HEADING),
            );

            if let Some(sel) = selected_version.as_deref() {
                ui.add_space(12.0);
                draw_custom_badge(ui, &format!("Выбрана: {sel}"), RUBY);
            }
        });

        ui.add_space(12.0);

        // Control Row (Search + Sort)
        ui.horizontal(|ui| {
            // Search Input
            let search_width = (ui.available_width() - 170.0).max(180.0);
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new("🔍 Поиск версий...").color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(13.0));

            ui.add(search_edit);

            ui.add_space(8.0);

            // Sort Toggle Button
            let sort_label = match self.sort_order {
                SortOrder::Newest => "Сначала новые ▾",
                SortOrder::Oldest => "Сначала старые ▴",
            };

            let sort_btn = egui::Button::new(
                egui::RichText::new(sort_label)
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_PRIMARY),
            )
            .fill(BG_ELEVATED)
            .stroke(Stroke::new(1.0, BORDER_DEFAULT))
            .min_size(vec2(140.0, 32.0));

            if ui.add(sort_btn).clicked() {
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
            self.filter_tab_btn(ui, "ВСЕ", FilterTab::All);
            self.filter_tab_btn(ui, "РЕЛИЗЫ", FilterTab::Releases);
            self.filter_tab_btn(ui, "СНАПШОТЫ", FilterTab::Snapshots);
            self.filter_tab_btn(ui, "БЕТА", FilterTab::Betas);
            self.filter_tab_btn(ui, "АЛЬФА", FilterTab::Alphas);
            self.filter_tab_btn(ui, "СТАРЫЕ", FilterTab::Old);
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
                    ui.label(
                        egui::RichText::new("Загрузка списка версий от Mojang...")
                            .font(egui::FontId::proportional(15.0))
                            .color(TEXT_MUTED),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Версии не найдены")
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
                    child.label(
                        egui::RichText::new(&v.id)
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(if is_selected { Color32::WHITE } else { TEXT_HEADING }),
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
                            .color(TEXT_MUTED),
                    );

                    if resp.clicked() {
                        *selected_version = Some(v.id.clone());
                    }
                }
            });
    }

    fn draw_hero_banner(&self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 94.0), Sense::hover());

        // Dark Ruby Gradient card
        ui.painter().rect_filled(rect, Rounding::ZERO, Color32::from_rgb(0x13, 0x0B, 0x0D));
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, BORDER_STRONG));

        // Ruby accent bar on left
        let accent = Rect::from_min_max(rect.left_top(), Pos2::new(rect.left() + 4.0, rect.bottom()));
        ui.painter().rect_filled(accent, Rounding::ZERO, RUBY);

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.add_space(18.0);

        // Emblem
        let (logo_rect, _) = child.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
        child.painter().rect_filled(logo_rect, Rounding::ZERO, RUBY_DIM);
        child.painter().rect_stroke(logo_rect, Rounding::ZERO, Stroke::new(1.5, RUBY));
        child.painter().text(
            logo_rect.center(),
            egui::Align2::CENTER_CENTER,
            "ℵ",
            egui::FontId::proportional(30.0),
            RUBY_LIGHT,
        );

        child.add_space(14.0);

        child.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("ALEPH MINECRAFT CLIENT")
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(TEXT_HEADING),
                );
                draw_custom_badge(ui, "v0.2.0 OPEN-SOURCE", RUBY);
            });

            ui.add_space(2.0);

            ui.label(
                egui::RichText::new("Высокопроизводительный модульный лаунчер с поддержкой Fabric, Quilt, Forge, скинов и модов")
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚡ 60-144+ FPS").font(egui::FontId::proportional(11.0)).color(RUBY_LIGHT));
                ui.label(egui::RichText::new("•").color(TEXT_MUTED));
                ui.label(egui::RichText::new("🛡 WetID & Microsoft").font(egui::FontId::proportional(11.0)).color(TEXT_PRIMARY));
                ui.label(egui::RichText::new("•").color(TEXT_MUTED));
                ui.label(egui::RichText::new("🌐 Modrinth & CurseForge").font(egui::FontId::proportional(11.0)).color(SUCCESS));
            });
        });

        // Right side: Featured Server Status Widget
        let srv_width = 210.0;
        let space = child.available_width() - srv_width - 16.0;
        if space > 0.0 {
            child.add_space(space);
        }

        let (srv_box, _) = child.allocate_exact_size(vec2(srv_width, 68.0), Sense::hover());
        child.painter().rect_filled(srv_box, Rounding::ZERO, Color32::from_rgb(0x1B, 0x10, 0x13));
        child.painter().rect_stroke(srv_box, Rounding::ZERO, Stroke::new(1.0, BORDER_DEFAULT));

        let mut srv_ui = child.new_child(
            egui::UiBuilder::new()
                .max_rect(srv_box)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        srv_ui.add_space(8.0);

        if let Some(srv) = &self.featured_server {
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(&srv.host).font(egui::FontId::proportional(13.0)).strong().color(TEXT_HEADING));
            });
            srv_ui.add_space(2.0);
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                let (dot, col) = if srv.is_online { ("🟢", SUCCESS) } else { ("🔴", RUBY) };
                ui.label(egui::RichText::new(format!("{dot} {} мс", srv.ping_ms)).font(egui::FontId::proportional(11.0)).color(col));
                ui.label(egui::RichText::new("•").color(TEXT_MUTED));
                ui.label(egui::RichText::new(format!("{} онлайн", srv.online_players)).font(egui::FontId::proportional(11.0)).color(TEXT_PRIMARY));
            });
        } else if self.is_pinging {
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("mc.hypixel.net").font(egui::FontId::proportional(13.0)).strong().color(TEXT_HEADING));
            });
            srv_ui.add_space(4.0);
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("⏳ Пинг сервера...").font(egui::FontId::proportional(11.0)).color(TEXT_MUTED));
            });
        } else {
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("mc.hypixel.net").font(egui::FontId::proportional(13.0)).strong().color(TEXT_HEADING));
            });
            srv_ui.add_space(4.0);
            srv_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("🔴 Недоступен").font(egui::FontId::proportional(11.0)).color(RUBY));
            });
        }
    }

    fn filter_tab_btn(&mut self, ui: &mut Ui, label: &str, tab: FilterTab) {
        let is_active = self.active_filter == tab;
        let (bg, stroke, text_color) = if is_active {
            (RUBY, Stroke::new(1.0, RUBY_LIGHT), Color32::WHITE)
        } else {
            (BG_ELEVATED, Stroke::new(1.0, BORDER_DEFAULT), TEXT_MUTED)
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
        .min_size(vec2(72.0, 26.0));

        if ui.add(btn).clicked() {
            self.active_filter = tab;
        }
    }
}
