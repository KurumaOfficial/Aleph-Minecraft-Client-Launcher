use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_mods::types::{LocalMod, ModSearchResult, ModSource};
use std::collections::HashSet;
use std::path::Path;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, SUCCESS,
    TEXT_HEADING, TEXT_MUTED,
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
        on_search: impl FnOnce(String, ModSource),
        on_install: impl FnOnce(ModSearchResult),
    ) {
        ui.add_space(20.0);

        // Header
        ui.label(
            egui::RichText::new("Управление модами")
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

        // Subtabs: Локальные / Поиск
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

            let local_active = self.sub_tab == ModsSubTab::Local;
            let search_active = self.sub_tab == ModsSubTab::Search;

            let btn_local = egui::Button::new(
                egui::RichText::new("УСТАНОВЛЕННЫЕ")
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(if local_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if local_active { RUBY } else { Color32::TRANSPARENT })
            .stroke(Stroke::new(1.0, if local_active { RUBY } else { BORDER_DEFAULT }))
            .min_size(vec2(130.0, 32.0));

            if ui.add(btn_local).clicked() {
                self.sub_tab = ModsSubTab::Local;
            }

            let search_label = match self.search_provider {
                ModSource::Modrinth => "ПОИСК (MODRINTH)",
                ModSource::CurseForge => "ПОИСК (CURSEFORGE)",
                _ => "ПОИСК МОДОВ",
            };

            let btn_search = egui::Button::new(
                egui::RichText::new(search_label)
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(if search_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if search_active { RUBY } else { Color32::TRANSPARENT })
            .stroke(Stroke::new(1.0, if search_active { RUBY } else { BORDER_DEFAULT }))
            .min_size(vec2(160.0, 32.0));

            if ui.add(btn_search).clicked() {
                self.sub_tab = ModsSubTab::Search;
            }

            ui.add_space(16.0);

            if ui.button("📂 Открыть папку mods/").clicked() {
                let _ = std::fs::create_dir_all(mods_dir);
                let _ = open::that(mods_dir);
            }
        });

        ui.add_space(14.0);

        match self.sub_tab {
            ModsSubTab::Local => self.show_local_mods(ui),
            ModsSubTab::Search => self.show_search(ui, on_search, on_install),
        }
    }

    fn show_local_mods(&mut self, ui: &mut Ui) {
        if self.local_mods.is_empty() {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Папка mods/ пуста")
                        .font(egui::FontId::proportional(16.0))
                        .color(TEXT_MUTED),
                );
                ui.label(
                    egui::RichText::new("Перейдите на вкладку Поиск или перетащите .jar файлы в папку mods")
                        .font(egui::FontId::proportional(13.0))
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
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 62.0),
                        Sense::hover(),
                    );

                    ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, BORDER_DEFAULT));

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(14.0);

                    // Mod Icon
                    child.label(egui::RichText::new("🧩").font(egui::FontId::proportional(20.0)));
                    child.add_space(10.0);

                    // Name + version
                    child.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&m.name)
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(TEXT_HEADING),
                        );
                        ui.label(
                            egui::RichText::new(format!("v{} • {:.1} КБ", m.version, m.file_size as f64 / 1024.0))
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );
                    });

                    // Controls on the right
                    let controls_width = 110.0;
                    let avail = child.available_width() - controls_width - 16.0;
                    if avail > 0.0 {
                        child.add_space(avail);
                    }

                    // Toggle Button
                    let toggle_text = if m.enabled { "ВКЛ" } else { "ВЫКЛ" };
                    let toggle_color = if m.enabled { SUCCESS } else { TEXT_MUTED };
                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(toggle_text)
                            .font(egui::FontId::proportional(11.0))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(toggle_color)
                    .min_size(vec2(50.0, 26.0));

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
        on_search: impl FnOnce(String, ModSource),
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
            }

            ui.add_space(8.0);

            let search_width = (ui.available_width() - 100.0).max(180.0);
            let hint = match self.search_provider {
                ModSource::Modrinth => "🔍 Поиск в Modrinth...",
                ModSource::CurseForge => "🔍 Поиск в CurseForge...",
                _ => "🔍 Поиск модов...",
            };

            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new(hint).color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(14.0));

            let resp = ui.add(search_edit);
            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.add_space(8.0);

            if ui
                .button(egui::RichText::new("НАЙТИ").strong().color(Color32::WHITE))
                .clicked()
                || enter_pressed
            {
                on_search(self.search_query.clone(), self.search_provider);
            }
        });

        ui.add_space(14.0);

        if self.is_searching {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new("Поиск модов...").color(TEXT_MUTED));
            });
            return;
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                let mut to_install = None;

                for item in &self.search_results {
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 70.0),
                        Sense::hover(),
                    );

                    ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, BORDER_DEFAULT));

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

                    // Title & Description
                    child.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&item.title)
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(TEXT_HEADING),
                            );
                            ui.label(
                                egui::RichText::new(format!("от {}", item.author))
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

                        ui.label(
                            egui::RichText::new(&item.description)
                                .font(egui::FontId::proportional(12.0))
                                .color(TEXT_MUTED),
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

                    let btn_width = 110.0;
                    let avail = child.available_width() - btn_width - 16.0;
                    if avail > 0.0 {
                        child.add_space(avail);
                    }

                    if is_downloading {
                        let btn = egui::Button::new(
                            egui::RichText::new("⏳ СКАЧИВАНИЕ")
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        )
                        .fill(BG_HOVER)
                        .min_size(vec2(100.0, 30.0));
                        child.add(btn);
                    } else if is_installed {
                        let badge = egui::Button::new(
                            egui::RichText::new("✓ УСТАНОВЛЕН")
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(SUCCESS)
                        .min_size(vec2(100.0, 30.0));
                        child.add(badge);
                    } else {
                        let btn = egui::Button::new(
                            egui::RichText::new("СКАЧАТЬ")
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(RUBY)
                        .stroke(Stroke::new(1.0, RUBY_LIGHT))
                        .min_size(vec2(90.0, 30.0));

                        if child.add(btn).clicked() {
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
