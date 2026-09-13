use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_mods::types::{LocalMod, ModCategory, ModSearchResult, ModSource};
use amc_core::Language;
use std::collections::HashSet;
use std::path::Path;
use crate::theme::{
    lerp_color, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, SUCCESS,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModsSubTab {
    #[default]
    Local,
    Search,
}

pub struct ModsPage {
    pub sub_tab: ModsSubTab,
    pub search_provider: ModSource,
    pub search_category: ModCategory,
    pub search_query: String,
    pub is_searching: bool,
    pub search_results: Vec<ModSearchResult>,
    pub local_mods: Vec<LocalMod>,
    pub installing_ids: HashSet<String>,
    pub installed_titles: HashSet<String>,
    pub status_message: Option<(String, bool)>,
}

impl Default for ModsPage {
    fn default() -> Self {
        Self {
            sub_tab: ModsSubTab::Local,
            search_provider: ModSource::Modrinth,
            search_category: ModCategory::Mod,
            search_query: String::new(),
            is_searching: false,
            search_results: Vec::new(),
            local_mods: Vec::new(),
            installing_ids: HashSet::new(),
            installed_titles: HashSet::new(),
            status_message: None,
        }
    }
}

impl ModsPage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        mods_dir: &Path,
        lang: Language,
        on_search: impl FnOnce(String, ModSource, ModCategory),
        on_install: impl FnOnce(ModSearchResult),
    ) {
        ui.add_space(20.0);

        // Header
        ui.label(
            egui::RichText::new(lang.mods_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Status banner if present
        if let Some((msg, is_ok)) = &self.status_message {
            let col = if *is_ok { SUCCESS } else { RUBY };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("ℹ {msg}")).color(col).strong());
            });
            ui.add_space(8.0);
        }

        // Subtabs: Local / Search
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

            let local_active = self.sub_tab == ModsSubTab::Local;
            let search_active = self.sub_tab == ModsSubTab::Search;

            // Local Tab Button
            let (loc_rect, loc_resp) = ui.allocate_exact_size(vec2(130.0, 32.0), Sense::click());
            let loc_hover = ui.ctx().animate_bool_responsive(loc_resp.id, loc_resp.hovered());
            let loc_act = ui.ctx().animate_bool(loc_resp.id.with("act"), local_active);
            let loc_bg = lerp_color(lerp_color(Color32::TRANSPARENT, BG_HOVER, loc_hover), RUBY, loc_act);
            let loc_stroke = lerp_color(lerp_color(BORDER_DEFAULT, RUBY, loc_hover), RUBY_LIGHT, loc_act);
            let loc_text = lerp_color(lerp_color(TEXT_MUTED, TEXT_PRIMARY, loc_hover), Color32::WHITE, loc_act);

            ui.painter().rect_filled(loc_rect, Rounding::ZERO, loc_bg);
            ui.painter().rect_stroke(loc_rect, Rounding::ZERO, Stroke::new(1.0, loc_stroke));
            ui.painter().text(loc_rect.center(), egui::Align2::CENTER_CENTER, lang.mods_tab_installed(), egui::FontId::proportional(12.0), loc_text);

            if loc_resp.clicked() {
                self.sub_tab = ModsSubTab::Local;
            }

            // Search Tab Button
            let search_label = match self.search_provider {
                ModSource::Modrinth => lang.mods_tab_search_modrinth(),
                ModSource::CurseForge => lang.mods_tab_search_curseforge(),
                _ => lang.mods_tab_search_generic(),
            };

            let (sea_rect, sea_resp) = ui.allocate_exact_size(vec2(160.0, 32.0), Sense::click());
            let sea_hover = ui.ctx().animate_bool_responsive(sea_resp.id, sea_resp.hovered());
            let sea_act = ui.ctx().animate_bool(sea_resp.id.with("act"), search_active);
            let sea_bg = lerp_color(lerp_color(Color32::TRANSPARENT, BG_HOVER, sea_hover), RUBY, sea_act);
            let sea_stroke = lerp_color(lerp_color(BORDER_DEFAULT, RUBY, sea_hover), RUBY_LIGHT, sea_act);
            let sea_text = lerp_color(lerp_color(TEXT_MUTED, TEXT_PRIMARY, sea_hover), Color32::WHITE, sea_act);

            ui.painter().rect_filled(sea_rect, Rounding::ZERO, sea_bg);
            ui.painter().rect_stroke(sea_rect, Rounding::ZERO, Stroke::new(1.0, sea_stroke));
            ui.painter().text(sea_rect.center(), egui::Align2::CENTER_CENTER, &search_label, egui::FontId::proportional(12.0), sea_text);

            if sea_resp.clicked() {
                self.sub_tab = ModsSubTab::Search;
            }

            ui.add_space(16.0);

            if ui.button(lang.mods_btn_open_folder()).clicked() {
                let _ = std::fs::create_dir_all(mods_dir);
                let _ = open::that(mods_dir);
            }
        });

        ui.add_space(14.0);

        match self.sub_tab {
            ModsSubTab::Local => self.show_local_mods(ui, lang),
            ModsSubTab::Search => self.show_search(ui, lang, on_search, on_install),
        }
    }

    fn show_local_mods(&mut self, ui: &mut Ui, lang: Language) {
        if self.local_mods.is_empty() {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(lang.mods_no_installed())
                        .font(egui::FontId::proportional(15.0))
                        .color(TEXT_MUTED),
                );
            });
            return;
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                let mut to_toggle = None;
                let mut to_delete = None;

                for (idx, m) in self.local_mods.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 62.0),
                        Sense::click(),
                    );
                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let stroke_col = lerp_color(BORDER_DEFAULT, RUBY, hover_t);

                    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, stroke_col));

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(14.0);

                    // Mod Icon
                    child.label(egui::RichText::new("🧩").font(egui::FontId::proportional(20.0)));
                    child.add_space(10.0);

                    // Name + version with truncation
                    let max_text_w = (child.available_width() - 150.0).max(120.0);
                    child.vertical(|ui| {
                        ui.set_max_width(max_text_w);
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
                            egui::RichText::new(format!("v{} • {:.1} KB", m.version, m.file_size as f64 / 1024.0))
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );
                    });

                    // Controls on the right
                    let controls_width = 130.0;
                    let avail = child.available_width() - controls_width - 16.0;
                    if avail > 0.0 {
                        child.add_space(avail);
                    }

                    // Toggle Button
                    let toggle_text = if m.enabled { lang.mods_status_enabled() } else { lang.mods_status_disabled() };
                    let toggle_color = if m.enabled { SUCCESS } else { TEXT_MUTED };
                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(toggle_text)
                            .font(egui::FontId::proportional(11.0))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(toggle_color)
                    .min_size(vec2(60.0, 26.0));

                    if child.add(toggle_btn).clicked() {
                        to_toggle = Some(idx);
                    }

                    child.add_space(8.0);

                    // Delete button
                    if child.button(egui::RichText::new("🗑").color(TEXT_MUTED)).clicked() {
                        to_delete = Some(idx);
                    }
                }

                if let Some(idx) = to_toggle {
                    if let Ok(new_path) = amc_mods::LocalModManager::toggle_mod(&self.local_mods[idx].path) {
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
        on_search: impl FnOnce(String, ModSource, ModCategory),
        on_install: impl FnOnce(ModSearchResult),
    ) {
        ui.horizontal(|ui| {
            // Source switch buttons
            let mr_active = self.search_provider == ModSource::Modrinth;
            let cf_active = self.search_provider == ModSource::CurseForge;

            let mr_btn = egui::Button::new(
                egui::RichText::new("Modrinth")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if mr_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if mr_active { RUBY } else { BG_HOVER })
            .min_size(vec2(80.0, 30.0));

            if ui.add(mr_btn).clicked() {
                self.search_provider = ModSource::Modrinth;
            }

            let cf_btn = egui::Button::new(
                egui::RichText::new("CurseForge")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if cf_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if cf_active { RUBY } else { BG_HOVER })
            .min_size(vec2(90.0, 30.0));

            if ui.add(cf_btn).clicked() {
                self.search_provider = ModSource::CurseForge;
                self.search_category = ModCategory::Mod;
            }

            ui.add_space(8.0);

            let search_width = (ui.available_width() - 100.0).max(180.0);
            let hint = match self.search_provider {
                ModSource::Modrinth => lang.mods_search_hint("Modrinth"),
                ModSource::CurseForge => lang.mods_search_hint("CurseForge"),
                _ => lang.mods_search_hint("Mods"),
            };

            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new(hint).color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(14.0));

            let resp = ui.add(search_edit);
            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.add_space(8.0);

            if ui
                .button(egui::RichText::new(lang.mods_btn_search()).strong().color(Color32::WHITE))
                .clicked()
                || enter_pressed
            {
                on_search(self.search_query.clone(), self.search_provider, self.search_category);
            }
        });

        // Category selection row for Modrinth
        if self.search_provider == ModSource::Modrinth {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let cat_mod = self.search_category == ModCategory::Mod;
                let cat_rp = self.search_category == ModCategory::ResourcePack;
                let cat_sh = self.search_category == ModCategory::Shader;

                let b_mod = egui::Button::new(
                    egui::RichText::new(format!("🧩 {}", lang.mods_cat_mods()))
                        .font(egui::FontId::proportional(11.0))
                        .color(if cat_mod { Color32::WHITE } else { TEXT_MUTED }),
                )
                .fill(if cat_mod { RUBY } else { BG_ELEVATED });
                if ui.add(b_mod).clicked() {
                    self.search_category = ModCategory::Mod;
                }

                let b_rp = egui::Button::new(
                    egui::RichText::new(format!("🎨 {}", lang.mods_cat_resourcepacks()))
                        .font(egui::FontId::proportional(11.0))
                        .color(if cat_rp { Color32::WHITE } else { TEXT_MUTED }),
                )
                .fill(if cat_rp { RUBY } else { BG_ELEVATED });
                if ui.add(b_rp).clicked() {
                    self.search_category = ModCategory::ResourcePack;
                }

                let b_sh = egui::Button::new(
                    egui::RichText::new(format!("☀️ {}", lang.mods_cat_shaders()))
                        .font(egui::FontId::proportional(11.0))
                        .color(if cat_sh { Color32::WHITE } else { TEXT_MUTED }),
                )
                .fill(if cat_sh { RUBY } else { BG_ELEVATED });
                if ui.add(b_sh).clicked() {
                    self.search_category = ModCategory::Shader;
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

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                let mut to_install = None;

                for item in &self.search_results {
                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 70.0),
                        Sense::click(),
                    );
                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let stroke_col = lerp_color(BORDER_DEFAULT, RUBY, hover_t);

                    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, stroke_col));

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(14.0);

                    // Icon placeholder
                    let icon_glyph = match item.source {
                        ModSource::Modrinth => "🌐",
                        ModSource::CurseForge => "🔥",
                        ModSource::Local => "🧩",
                    };
                    child.label(egui::RichText::new(icon_glyph).font(egui::FontId::proportional(22.0)));
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
                        });

                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&item.description)
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            )
                            .truncate(),
                        );
                    });

                    // Install button or Installed status
                    let is_downloading = self.installing_ids.contains(&item.id);
                    let is_installed = self.installed_titles.contains(&item.title.to_lowercase())
                        || self.local_mods.iter().any(|lm| {
                            let lm_lower = lm.name.to_lowercase();
                            let it_lower = item.title.to_lowercase();
                            lm_lower == it_lower || lm_lower.contains(&it_lower)
                        });

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
                        let (ib_rect, ib_resp) = child.allocate_exact_size(vec2(90.0, 30.0), Sense::click());
                        let ib_hover = child.ctx().animate_bool_responsive(ib_resp.id, ib_resp.hovered());
                        let ib_bg = lerp_color(RUBY, RUBY_LIGHT, ib_hover);
                        let ib_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, ib_hover);

                        child.painter().rect_filled(ib_rect, Rounding::ZERO, ib_bg);
                        child.painter().rect_stroke(ib_rect, Rounding::ZERO, Stroke::new(1.0, ib_stroke));
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
