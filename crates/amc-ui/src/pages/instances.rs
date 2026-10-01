use crate::theme::{
    lerp_color, BG_CARD, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::choice_chip;
use amc_core::types::{Instance, InstanceTemplate, LoaderType};
use amc_core::Language;
use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum InstanceAction {
    None,
    Created(Instance),
    Updated(Instance),
    Deleted(Uuid),
    Selected(Uuid),
    Cloned(Uuid),
    Launch(Uuid),
}

pub struct InstancesPage {
    pub search_query: String,
    pub show_create_modal: bool,
    pub new_instance_name: String,
    pub new_instance_version: String,
    pub new_instance_loader: LoaderType,
    pub new_instance_ram: u32,

    // Edit modal
    pub show_edit_modal: bool,
    pub edit_instance_id: Option<Uuid>,
    pub edit_instance_name: String,
    pub edit_instance_version: String,
    pub edit_instance_loader: LoaderType,
    pub edit_instance_ram: u32,

    // Background disk-usage scan (CONCEPT "Место на диске").
    disk_sizes: HashMap<Uuid, u64>,
    disk_scanned_for: Vec<Uuid>,
    disk_rx: Option<std::sync::mpsc::Receiver<Vec<(Uuid, u64)>>>,
}

impl Default for InstancesPage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            show_create_modal: false,
            new_instance_name: "My Instance".to_string(),
            new_instance_version: "1.20.1".to_string(),
            new_instance_loader: LoaderType::Fabric,
            new_instance_ram: 4096,

            show_edit_modal: false,
            edit_instance_id: None,
            edit_instance_name: String::new(),
            edit_instance_version: String::new(),
            edit_instance_loader: LoaderType::Vanilla,
            edit_instance_ram: 4096,

            disk_sizes: HashMap::new(),
            disk_scanned_for: Vec::new(),
            disk_rx: None,
        }
    }
}

/// Shorten a card title to one line (pure, unit-tested).
fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        name.to_string()
    } else {
        format!("{}…", name.chars().take(max_chars).collect::<String>())
    }
}

impl InstancesPage {
    /// Poll a finished background scan, or (re)start one when the instance
    /// set changed. Measuring runs on a worker thread — the UI never blocks.
    fn poll_disk_sizes(&mut self, instances: &[Instance], instances_dir: &Path) {
        if let Some(rx) = &self.disk_rx {
            match rx.try_recv() {
                Ok(sizes) => {
                    self.disk_sizes = sizes.into_iter().collect();
                    self.disk_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.disk_rx = None;
                }
            }
            return;
        }
        let ids: Vec<Uuid> = instances.iter().map(|i| i.id).collect();
        if ids == self.disk_scanned_for {
            return;
        }
        self.disk_scanned_for = ids;
        let pairs: Vec<(Uuid, PathBuf)> = instances
            .iter()
            .map(|i| (i.id, i.get_game_dir(instances_dir)))
            .collect();
        let (tx, rx) = std::sync::mpsc::channel();
        self.disk_rx = Some(rx);
        std::thread::spawn(move || {
            let sizes: Vec<(Uuid, u64)> = pairs
                .into_iter()
                .map(|(id, dir)| (id, Instance::disk_usage_dir(&dir)))
                .collect();
            let _ = tx.send(sizes);
        });
    }

    /// Fill the create dialog from a quick-start template (ROADMAP P2).
    /// Loader and RAM always apply; the name applies only while untouched.
    fn apply_template(&mut self, template: InstanceTemplate, lang: Language) {
        self.new_instance_loader = template.loader;
        self.new_instance_ram = template.ram_mb;
        if self.new_instance_name.trim().is_empty() || self.new_instance_name == "My Instance" {
            self.new_instance_name = template.default_name(lang);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        instances: &mut Vec<Instance>,
        selected_instance: &mut Option<Uuid>,
        instances_dir: &Path,
        lang: Language,
    ) -> InstanceAction {
        let mut action = InstanceAction::None;
        self.poll_disk_sizes(instances, instances_dir);
        ui.add_space(20.0);

        // Header
        ui.label(
            egui::RichText::new(lang.inst_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Control Row
        ui.horizontal(|ui| {
            let search_width = (ui.available_width() - 180.0).max(200.0);
            ui.add(
                TextEdit::singleline(&mut self.search_query)
                    .hint_text(egui::RichText::new(lang.inst_search_hint()).color(TEXT_MUTED))
                    .desired_width(search_width)
                    .font(egui::FontId::proportional(14.0)),
            );

            ui.add_space(10.0);

            let (cr_rect, cr_resp) = ui.allocate_exact_size(vec2(160.0, 36.0), Sense::click());
            let cr_hover = ui
                .ctx()
                .animate_bool_responsive(cr_resp.id, cr_resp.hovered());
            let cr_bg = lerp_color(RUBY, RUBY_LIGHT, cr_hover);
            let cr_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, cr_hover);

            ui.painter().rect_filled(cr_rect, Rounding::ZERO, cr_bg);
            ui.painter()
                .rect_stroke(cr_rect, Rounding::ZERO, Stroke::new(1.0, cr_stroke));
            ui.painter().text(
                cr_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.inst_btn_create(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            if cr_resp.clicked() {
                self.show_create_modal = true;
            }
        });

        ui.add_space(14.0);

        // Filter instances
        let query = self.search_query.trim().to_lowercase();
        let filtered_indices: Vec<usize> = instances
            .iter()
            .enumerate()
            .filter(|(_, inst)| {
                if query.is_empty() {
                    true
                } else {
                    inst.name.to_lowercase().contains(&query)
                        || inst.game_version.to_lowercase().contains(&query)
                        || inst.loader.as_str().to_lowercase().contains(&query)
                }
            })
            .map(|(idx, _)| idx)
            .collect();

        // Instances list
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                if instances.is_empty() {
                    ui.add_space(50.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(lang.inst_empty())
                                .font(egui::FontId::proportional(16.0))
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(10.0);
                        if ui
                            .button(egui::RichText::new(lang.inst_btn_create()).color(RUBY_LIGHT))
                            .clicked()
                        {
                            self.show_create_modal = true;
                        }
                    });
                    return;
                }

                if filtered_indices.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(lang.home_empty())
                                .font(egui::FontId::proportional(14.0))
                                .color(TEXT_MUTED),
                        );
                    });
                    return;
                }

                // Full-style card grid: fixed 270px cards, auto-fill rows.
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                    for &idx in &filtered_indices {
                        let inst = &instances[idx];
                        instance_card(
                            self,
                            ui,
                            inst,
                            selected_instance,
                            instances_dir,
                            lang,
                            &mut action,
                        );
                    }
                });
            });

        /// Full-style instance card: loader accent strip, icon, name, meta,
        /// quick actions and a Play button. Click selects the instance.
        fn instance_card(
            page: &mut InstancesPage,
            ui: &mut Ui,
            inst: &Instance,
            selected_instance: &mut Option<Uuid>,
            instances_dir: &Path,
            lang: Language,
            action: &mut InstanceAction,
        ) {
            const CARD_W: f32 = 270.0;
            const CARD_H: f32 = 216.0;
            let inst_id = inst.id;
            let is_selected = selected_instance.as_ref() == Some(&inst_id);

            let (rect, resp) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());

            let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

            let base_bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
            let bg = lerp_color(base_bg, RUBY_DIM, sel_t);

            let stroke_col =
                lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, sel_t);
            let border_width = if is_selected { 1.5 } else { 1.0 };

            ui.painter().rect_filled(rect, Rounding::ZERO, bg);

            // Loader accent strip on top (Square motif from aleph.icu)
            let loader_color = match inst.loader {
                LoaderType::Vanilla => Color32::from_rgb(0x38, 0x8E, 0x3C),
                LoaderType::Fabric => RUBY_LIGHT,
                LoaderType::Quilt => Color32::from_rgb(0x9C, 0x27, 0xB0),
                LoaderType::Forge => Color32::from_rgb(0xE6, 0x51, 0x00),
                LoaderType::NeoForge => Color32::from_rgb(0xFF, 0x6D, 0x00),
                LoaderType::OptiFine => Color32::from_rgb(0x00, 0x83, 0x8F),
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.left_top(),
                    egui::pos2(rect.right(), rect.top() + 3.0),
                ),
                Rounding::ZERO,
                loader_color,
            );
            ui.painter()
                .rect_stroke(rect, Rounding::ZERO, Stroke::new(border_width, stroke_col));

            let mut card = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect.shrink(12.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            // Header: loader icon + name + version.
            card.horizontal(|ui| {
                let icon_glyph = match inst.loader {
                    LoaderType::Vanilla => "🟩",
                    LoaderType::Fabric => "🧵",
                    LoaderType::Quilt => "🪡",
                    LoaderType::Forge => "🔨",
                    LoaderType::NeoForge => "⚡",
                    LoaderType::OptiFine => "✨",
                };
                let (icon_rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
                ui.painter()
                    .rect_filled(icon_rect, Rounding::ZERO, RUBY_DIM);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon_glyph,
                    egui::FontId::proportional(22.0),
                    Color32::WHITE,
                );
                ui.add_space(8.0);
                ui.vertical(|inner| {
                    inner.label(
                        egui::RichText::new(truncate_name(&inst.name, 22))
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(TEXT_HEADING),
                    );
                    inner.label(
                        egui::RichText::new(format!("MC {}", inst.game_version))
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    inner.label(
                        egui::RichText::new(inst.loader.as_str())
                            .font(egui::FontId::proportional(12.0))
                            .color(RUBY_LIGHT),
                    );
                });
            });

            card.add_space(6.0);

            // Stats line: playtime, RAM, disk.
            let play_time_str = if inst.total_played_minutes == 0 {
                lang.inst_never_played().to_string()
            } else if inst.total_played_minutes < 60 {
                format!("{} {}", inst.total_played_minutes, lang.inst_mins_suffix())
            } else {
                format!(
                    "{} {} {} {}",
                    inst.total_played_minutes / 60,
                    lang.inst_hours_suffix(),
                    inst.total_played_minutes % 60,
                    lang.inst_mins_suffix()
                )
            };
            let size_text = match page.disk_sizes.get(&inst_id) {
                Some(bytes) => lang.disk_size(*bytes),
                None => "…".to_string(),
            };
            let ram_text = match inst.ram_mb {
                Some(ram) => format!(" • {ram} MB"),
                None => String::new(),
            };
            card.label(
                egui::RichText::new(format!(
                    "• ⏱ {} • 💾 {size_text}{ram_text}",
                    lang.inst_playtime_label(&play_time_str)
                ))
                .font(egui::FontId::proportional(11.0))
                .color(TEXT_MUTED),
            );

            card.add_space(6.0);

            // Quick actions row.
            let inst_dir = inst.get_game_dir(instances_dir);
            card.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("📁").color(TEXT_PRIMARY))
                    .on_hover_text(lang.inst_btn_folder())
                    .clicked()
                {
                    let _ = std::fs::create_dir_all(&inst_dir);
                    let _ = open::that(&inst_dir);
                }
                if ui
                    .button(egui::RichText::new("⚙").color(TEXT_PRIMARY))
                    .on_hover_text(lang.inst_btn_edit())
                    .clicked()
                {
                    page.edit_instance_id = Some(inst_id);
                    page.edit_instance_name = inst.name.clone();
                    page.edit_instance_version = inst.game_version.clone();
                    page.edit_instance_loader = inst.loader;
                    page.edit_instance_ram = inst.ram_mb.unwrap_or(4096);
                    page.show_edit_modal = true;
                }
                if ui
                    .button(egui::RichText::new("📋").color(TEXT_PRIMARY))
                    .on_hover_text(lang.inst_btn_clone())
                    .clicked()
                {
                    *action = InstanceAction::Cloned(inst_id);
                }
                if ui
                    .button(egui::RichText::new("🗑").color(TEXT_MUTED))
                    .on_hover_text(lang.inst_btn_delete())
                    .clicked()
                {
                    *action = InstanceAction::Deleted(inst_id);
                }
            });

            card.add_space(6.0);

            // Play button fills the remaining card height (at least 34px).
            let play_h = (rect.bottom() - 12.0 - card.min_rect().bottom()).max(34.0);
            let play_w = card.available_width();
            let (pb_rect, pb_resp) = card.allocate_exact_size(vec2(play_w, play_h), Sense::click());
            let pb_hover = card
                .ctx()
                .animate_bool_responsive(pb_resp.id, pb_resp.hovered());
            let pb_bg = lerp_color(RUBY, RUBY_LIGHT, pb_hover);
            let pb_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, pb_hover);

            card.painter().rect_filled(pb_rect, Rounding::ZERO, pb_bg);
            card.painter()
                .rect_stroke(pb_rect, Rounding::ZERO, Stroke::new(1.0, pb_stroke));
            card.painter().text(
                pb_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.inst_btn_play(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            if pb_resp.clicked() {
                *action = InstanceAction::Launch(inst_id);
            }

            if resp.clicked() {
                *selected_instance = Some(inst_id);
                *action = InstanceAction::Selected(inst_id);
            }
        }

        // Create Instance Modal Dialog
        if self.show_create_modal {
            egui::Window::new(lang.inst_modal_create_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(lang.tmpl_title())
                            .strong()
                            .color(TEXT_HEADING),
                    );
                    ui.horizontal(|ui| {
                        for tmpl in InstanceTemplate::ALL {
                            let active = InstanceTemplate::matching(
                                self.new_instance_loader,
                                self.new_instance_ram,
                            )
                            .map(|t| t.id)
                                == Some(tmpl.id);
                            if choice_chip(ui, lang.tmpl_name(tmpl.id), 130.0, 32.0, 12.0, active) {
                                self.apply_template(tmpl, lang);
                            }
                        }
                    });
                    let tmpl_desc =
                        InstanceTemplate::matching(self.new_instance_loader, self.new_instance_ram)
                            .map(|t| lang.tmpl_desc(t.id))
                            .unwrap_or_else(|| lang.tmpl_custom_note());
                    ui.label(egui::RichText::new(tmpl_desc).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.new_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.new_instance_version).desired_width(400.0),
                    );

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("loader_select")
                        .selected_text(self.new_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Vanilla,
                                "Vanilla",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Fabric,
                                "Fabric",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Quilt,
                                "Quilt",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Forge,
                                "Forge",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::NeoForge,
                                "NeoForge",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::OptiFine,
                                "OptiFine",
                            );
                        });

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {} MB",
                            lang.inst_modal_ram(),
                            self.new_instance_ram
                        ))
                        .color(TEXT_PRIMARY),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.new_instance_ram, 1024..=16384).step_by(512.0),
                    );

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.inst_modal_btn_create())
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .clicked()
                        {
                            let mut inst = Instance::new(
                                &self.new_instance_name,
                                &self.new_instance_version,
                                self.new_instance_loader,
                            );
                            inst.ram_mb = Some(self.new_instance_ram);
                            *selected_instance = Some(inst.id);
                            action = InstanceAction::Created(inst);
                            self.show_create_modal = false;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            self.show_create_modal = false;
                        }
                    });
                });
        }

        // Edit Instance Modal Dialog
        if self.show_edit_modal {
            let mut close_edit = false;
            let mut save_edit = false;

            egui::Window::new(lang.inst_modal_edit_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.edit_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_version).desired_width(400.0),
                    );

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("edit_loader_select")
                        .selected_text(self.edit_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Vanilla,
                                "Vanilla",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Fabric,
                                "Fabric",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Quilt,
                                "Quilt",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Forge,
                                "Forge",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::NeoForge,
                                "NeoForge",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::OptiFine,
                                "OptiFine",
                            );
                        });

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {} MB",
                            lang.inst_modal_ram(),
                            self.edit_instance_ram
                        ))
                        .color(TEXT_PRIMARY),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.edit_instance_ram, 1024..=32768).step_by(512.0),
                    );

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.inst_modal_btn_save())
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .clicked()
                        {
                            save_edit = true;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            close_edit = true;
                        }
                    });
                });

            if save_edit {
                if let Some(target_id) = self.edit_instance_id {
                    if let Some(inst) = instances.iter_mut().find(|i| i.id == target_id) {
                        inst.name = self.edit_instance_name.clone();
                        inst.game_version = self.edit_instance_version.clone();
                        inst.loader = self.edit_instance_loader;
                        inst.ram_mb = Some(self.edit_instance_ram);
                        action = InstanceAction::Updated(inst.clone());
                    }
                }
                self.show_edit_modal = false;
            }

            if close_edit {
                self.show_edit_modal = false;
            }
        }

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("Short", 22), "Short");
        assert_eq!(
            truncate_name("Exactly twenty-two chars!!", 22),
            "Exactly twenty-two cha…"
        );
        assert_eq!(truncate_name("", 22), "");
    }

    #[test]
    fn test_apply_template_fills_form() {
        let lang = Language::English;
        let mut page = InstancesPage::default();
        page.new_instance_name = String::new();
        page.apply_template(InstanceTemplate::OPTIMIZED, lang);
        assert_eq!(page.new_instance_loader, LoaderType::Fabric);
        assert_eq!(page.new_instance_ram, 6144);
        assert_eq!(page.new_instance_name, "Optimized");
    }

    #[test]
    fn test_apply_template_keeps_custom_name() {
        let lang = Language::English;
        let mut page = InstancesPage::default();
        page.new_instance_name = "My Pack".to_string();
        page.apply_template(InstanceTemplate::EMPTY, lang);
        assert_eq!(page.new_instance_loader, LoaderType::Vanilla);
        assert_eq!(page.new_instance_ram, 2048);
        assert_eq!(page.new_instance_name, "My Pack");
    }
}
