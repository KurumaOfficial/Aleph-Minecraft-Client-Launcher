use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use amc_core::types::{Instance, LoaderType};
use std::path::Path;
use uuid::Uuid;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone)]
pub enum InstanceAction {
    None,
    Created(Instance),
    Updated(Instance),
    Deleted(Uuid),
    Selected(Uuid),
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
}

impl Default for InstancesPage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            show_create_modal: false,
            new_instance_name: "Новая сборка".to_string(),
            new_instance_version: "1.20.1".to_string(),
            new_instance_loader: LoaderType::Fabric,
            new_instance_ram: 4096,

            show_edit_modal: false,
            edit_instance_id: None,
            edit_instance_name: String::new(),
            edit_instance_version: String::new(),
            edit_instance_loader: LoaderType::Vanilla,
            edit_instance_ram: 4096,
        }
    }
}

impl InstancesPage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        instances: &mut Vec<Instance>,
        selected_instance: &mut Option<Uuid>,
        instances_dir: &Path,
    ) -> InstanceAction {
        let mut action = InstanceAction::None;
        ui.add_space(20.0);

        // Header
        ui.label(
            egui::RichText::new("Сборки Minecraft")
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
                    .hint_text(egui::RichText::new("🔍 Поиск сборок...").color(TEXT_MUTED))
                    .desired_width(search_width)
                    .font(egui::FontId::proportional(14.0)),
            );

            ui.add_space(10.0);

            let create_btn = egui::Button::new(
                egui::RichText::new("+ СОЗДАТЬ СБОРКУ")
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(RUBY)
            .stroke(Stroke::new(1.0, RUBY_LIGHT))
            .min_size(vec2(160.0, 36.0));

            if ui.add(create_btn).clicked() {
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
                            egui::RichText::new("У вас пока нет созданных сборок")
                                .font(egui::FontId::proportional(16.0))
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(10.0);
                        if ui
                            .button(egui::RichText::new("Создать первую сборку").color(RUBY_LIGHT))
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
                            egui::RichText::new("Сборки по запросу не найдены")
                                .font(egui::FontId::proportional(14.0))
                                .color(TEXT_MUTED),
                        );
                    });
                    return;
                }

                for &idx in &filtered_indices {
                    let inst = &instances[idx];
                    let is_selected = selected_instance.as_ref() == Some(&inst.id);

                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 64.0),
                        Sense::click(),
                    );

                    let bg = if is_selected {
                        RUBY_DIM
                    } else if resp.hovered() {
                        BG_HOVER
                    } else {
                        BG_ELEVATED
                    };

                    let border = if is_selected {
                        Stroke::new(1.5, RUBY)
                    } else if resp.hovered() {
                        Stroke::new(1.0, RUBY)
                    } else {
                        Stroke::new(1.0, BORDER_DEFAULT)
                    };

                    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
                    ui.painter().rect_stroke(rect, Rounding::ZERO, border);

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child.add_space(16.0);

                    // Dynamic loader icon
                    let icon_glyph = match inst.loader {
                        LoaderType::Vanilla => "🟩",
                        LoaderType::Fabric => "🧵",
                        LoaderType::Quilt => "🪡",
                        LoaderType::Forge => "🔨",
                        LoaderType::NeoForge => "⚡",
                        LoaderType::OptiFine => "✨",
                    };
                    child.label(egui::RichText::new(icon_glyph).font(egui::FontId::proportional(22.0)));
                    child.add_space(12.0);

                    // Name + version
                    child.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&inst.name)
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(TEXT_HEADING),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("MC {}", inst.game_version))
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            );
                            ui.label(
                                egui::RichText::new(format!("• {}", inst.loader.as_str()))
                                    .font(egui::FontId::proportional(12.0))
                                    .color(RUBY_LIGHT),
                            );
                            if let Some(ram) = inst.ram_mb {
                                ui.label(
                                    egui::RichText::new(format!("• {} МБ RAM", ram))
                                        .font(egui::FontId::proportional(12.0))
                                        .color(TEXT_MUTED),
                                );
                            }

                            let play_time_str = if inst.total_played_minutes == 0 {
                                "Не запускалась".to_string()
                            } else if inst.total_played_minutes < 60 {
                                format!("{} мин", inst.total_played_minutes)
                            } else {
                                format!("{} ч {} мин", inst.total_played_minutes / 60, inst.total_played_minutes % 60)
                            };
                            ui.label(
                                egui::RichText::new(format!("• ⏱ {play_time_str}"))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_MUTED),
                            );
                        });
                    });

                    // Action buttons on the right
                    let actions_width = 205.0;
                    let avail = child.available_width() - actions_width - 16.0;
                    if avail > 0.0 {
                        child.add_space(avail);
                    }

                    // Open folder
                    let inst_dir = inst.get_game_dir(instances_dir);
                    if child
                        .button(egui::RichText::new("📁").color(TEXT_PRIMARY))
                        .on_hover_text("Открыть папку сборки")
                        .clicked()
                    {
                        let _ = std::fs::create_dir_all(&inst_dir);
                        let _ = open::that(&inst_dir);
                    }

                    child.add_space(6.0);

                    // Edit instance button
                    if child
                        .button(egui::RichText::new("⚙").color(TEXT_PRIMARY))
                        .on_hover_text("Редактировать сборку")
                        .clicked()
                    {
                        self.edit_instance_id = Some(inst.id);
                        self.edit_instance_name = inst.name.clone();
                        self.edit_instance_version = inst.game_version.clone();
                        self.edit_instance_loader = inst.loader;
                        self.edit_instance_ram = inst.ram_mb.unwrap_or(4096);
                        self.show_edit_modal = true;
                    }

                    child.add_space(6.0);

                    // Quick launch
                    let play_btn = egui::Button::new(
                        egui::RichText::new("▶ ИГРАТЬ")
                            .font(egui::FontId::proportional(11.0))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(RUBY)
                    .stroke(Stroke::new(1.0, RUBY_LIGHT))
                    .min_size(vec2(76.0, 28.0));

                    if child.add(play_btn).clicked() {
                        action = InstanceAction::Launch(inst.id);
                    }

                    child.add_space(6.0);

                    // Delete button
                    if child
                        .button(egui::RichText::new("🗑").color(TEXT_MUTED))
                        .on_hover_text("Удалить сборку")
                        .clicked()
                    {
                        action = InstanceAction::Deleted(inst.id);
                    }

                    if resp.clicked() {
                        *selected_instance = Some(inst.id);
                        action = InstanceAction::Selected(inst.id);
                    }
                }
            });

        // Create Instance Modal Dialog
        if self.show_create_modal {
            egui::Window::new("Создание новой сборки")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Название сборки:").color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.new_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Версия Minecraft:").color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.new_instance_version).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Загрузчик модов:").color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("loader_select")
                        .selected_text(self.new_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::Vanilla, "Vanilla");
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::Fabric, "Fabric");
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::Quilt, "Quilt");
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::Forge, "Forge");
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::NeoForge, "NeoForge");
                            ui.selectable_value(&mut self.new_instance_loader, LoaderType::OptiFine, "OptiFine");
                        });

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(format!("Выделение RAM: {} МБ", self.new_instance_ram)).color(TEXT_PRIMARY));
                    ui.add(egui::Slider::new(&mut self.new_instance_ram, 1024..=16384).step_by(512.0));

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("СОЗДАТЬ").strong().color(Color32::WHITE))
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

                        if ui.button("Отмена").clicked() {
                            self.show_create_modal = false;
                        }
                    });
                });
        }

        // Edit Instance Modal Dialog
        if self.show_edit_modal {
            let mut close_edit = false;
            let mut save_edit = false;

            egui::Window::new("Настройки сборки")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Название сборки:").color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.edit_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Версия Minecraft:").color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.edit_instance_version).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Загрузчик модов:").color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("edit_loader_select")
                        .selected_text(self.edit_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::Vanilla, "Vanilla");
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::Fabric, "Fabric");
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::Quilt, "Quilt");
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::Forge, "Forge");
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::NeoForge, "NeoForge");
                            ui.selectable_value(&mut self.edit_instance_loader, LoaderType::OptiFine, "OptiFine");
                        });

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(format!("Выделение RAM: {} МБ", self.edit_instance_ram)).color(TEXT_PRIMARY));
                    ui.add(egui::Slider::new(&mut self.edit_instance_ram, 1024..=32768).step_by(512.0));

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("СОХРАНИТЬ").strong().color(Color32::WHITE))
                            .clicked()
                        {
                            save_edit = true;
                        }

                        if ui.button("Отмена").clicked() {
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
