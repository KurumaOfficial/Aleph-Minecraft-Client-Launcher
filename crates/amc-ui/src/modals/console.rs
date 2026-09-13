use egui::{vec2, Color32, ScrollArea, Stroke, TextEdit};
use crate::theme::{BG_ELEVATED, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY};

pub struct ConsoleModal {
    pub is_open: bool,
    pub logs: Vec<String>,
    pub filter: String,
    pub autoscroll: bool,
}

impl Default for ConsoleModal {
    fn default() -> Self {
        Self {
            is_open: false,
            logs: Vec::new(),
            filter: String::new(),
            autoscroll: true,
        }
    }
}

impl ConsoleModal {
    pub fn push_line(&mut self, line: String) {
        if self.logs.len() >= 5000 {
            self.logs.remove(0);
        }
        self.logs.push(line);
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        egui::Window::new("Консоль игры (Логи процесса)")
            .open(&mut self.is_open)
            .default_size(vec2(780.0, 500.0))
            .min_size(vec2(500.0, 300.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.filter)
                            .hint_text(egui::RichText::new("🔍 Фильтр логов...").color(TEXT_MUTED))
                            .desired_width(220.0),
                    );

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.autoscroll, "Автоскролл вниз");

                    ui.add_space(12.0);

                    let copy_btn = egui::Button::new(
                        egui::RichText::new("📋 Копировать").color(TEXT_PRIMARY),
                    )
                    .fill(BG_ELEVATED)
                    .stroke(Stroke::new(1.0, BORDER_DEFAULT));

                    if ui.add(copy_btn).clicked() {
                        let full_text = self.logs.join("\n");
                        ui.ctx().copy_text(full_text);
                    }

                    let clear_btn = egui::Button::new(
                        egui::RichText::new("🧹 Очистить").color(TEXT_MUTED),
                    )
                    .fill(BG_ELEVATED)
                    .stroke(Stroke::new(1.0, BORDER_DEFAULT));

                    if ui.add(clear_btn).clicked() {
                        self.logs.clear();
                    }

                    let export_btn = egui::Button::new(
                        egui::RichText::new("💾 Экспорт").color(RUBY_LIGHT),
                    )
                    .fill(BG_ELEVATED)
                    .stroke(Stroke::new(1.0, RUBY));

                    if ui.add(export_btn).clicked() {
                        if let Some(dest) = rfd::FileDialog::new()
                            .set_file_name("minecraft-latest.log")
                            .save_file()
                        {
                            let full_text = self.logs.join("\n");
                            let _ = std::fs::write(dest, full_text);
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Logs list inside dark console box
                let filter_query = self.filter.trim().to_lowercase();
                let filtered: Vec<&String> = self
                    .logs
                    .iter()
                    .filter(|l| filter_query.is_empty() || l.to_lowercase().contains(&filter_query))
                    .collect();

                let row_height = 18.0;
                let num_rows = filtered.len();

                ScrollArea::vertical()
                    .stick_to_bottom(self.autoscroll)
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, num_rows, |ui, row_range| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                        for i in row_range {
                            let line = filtered[i];
                            let color = if line.contains("[ERROR]")
                                || line.contains("Exception")
                                || line.contains("Caused by")
                                || line.contains("Crash")
                            {
                                Color32::from_rgb(230, 80, 80)
                            } else if line.contains("[WARN]") {
                                Color32::from_rgb(240, 160, 40)
                            } else if line.contains("[DEBUG]") {
                                Color32::from_rgb(120, 110, 110)
                            } else {
                                TEXT_PRIMARY
                            };

                            ui.label(
                                egui::RichText::new(line)
                                    .font(egui::FontId::monospace(11.0))
                                    .color(color),
                            );
                        }
                    });
            });
    }
}
