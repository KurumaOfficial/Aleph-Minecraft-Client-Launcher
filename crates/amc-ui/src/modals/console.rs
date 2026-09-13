use egui::{vec2, Color32, Rounding, ScrollArea, Sense, Stroke, TextEdit};
use crate::theme::{lerp_color, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY};
use amc_core::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashCategory {
    OutOfMemory,
    JavaVersionMismatch,
    FabricMissingDependency,
    MixinConflict,
    GraphicsDriverGlfw,
}

#[derive(Debug, Clone)]
pub struct CrashDiagnosis {
    pub category: CrashCategory,
    pub title: String,
    pub description: String,
    pub solution: String,
    pub raw_cause: String,
}

pub struct ConsoleModal {
    pub is_open: bool,
    pub logs: Vec<String>,
    pub filter: String,
    pub autoscroll: bool,
    pub manual_diagnosis: Option<CrashDiagnosis>,
}

impl Default for ConsoleModal {
    fn default() -> Self {
        Self {
            is_open: false,
            logs: Vec::new(),
            filter: String::new(),
            autoscroll: true,
            manual_diagnosis: None,
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

    pub fn set_crash_message(&mut self, msg: String, lang: Language) {
        self.is_open = true;
        self.logs.push(format!("[AMC CRASH] {msg}"));
        if let Some(diag) = self.analyze_crash(lang) {
            self.manual_diagnosis = Some(diag);
        }
    }

    pub fn analyze_crash(&self, lang: Language) -> Option<CrashDiagnosis> {
        for line in self.logs.iter().rev() {
            if line.contains("java.lang.OutOfMemoryError") || line.contains("OutOfMemory") {
                return Some(CrashDiagnosis {
                    category: CrashCategory::OutOfMemory,
                    title: lang.diag_oom_title().to_string(),
                    description: lang.diag_oom_desc().to_string(),
                    solution: lang.diag_oom_solution().to_string(),
                    raw_cause: line.trim().to_string(),
                });
            }
            if line.contains("UnsupportedClassVersionError")
                || line.contains("has been compiled by a more recent version of the Java Runtime")
            {
                return Some(CrashDiagnosis {
                    category: CrashCategory::JavaVersionMismatch,
                    title: lang.diag_java_title().to_string(),
                    description: lang.diag_java_desc().to_string(),
                    solution: lang.diag_java_solution().to_string(),
                    raw_cause: line.trim().to_string(),
                });
            }
            if line.contains("net.fabricmc.loader.impl.FormattedException")
                || line.contains("ModResolutionException")
            {
                return Some(CrashDiagnosis {
                    category: CrashCategory::FabricMissingDependency,
                    title: lang.diag_fabric_dep_title().to_string(),
                    description: lang.diag_fabric_dep_desc().to_string(),
                    solution: lang.diag_fabric_dep_solution().to_string(),
                    raw_cause: line.trim().to_string(),
                });
            }
            if line.contains("org.spongepowered.asm.mixin.throwables.MixinApplyError")
                || line.contains("MixinTransformationException")
            {
                return Some(CrashDiagnosis {
                    category: CrashCategory::MixinConflict,
                    title: lang.diag_mixin_title().to_string(),
                    description: lang.diag_mixin_desc().to_string(),
                    solution: lang.diag_mixin_solution().to_string(),
                    raw_cause: line.trim().to_string(),
                });
            }
            if line.contains("GLFW error 65542")
                || line.contains("Pixel format not accelerated")
                || line.contains("WGL:")
            {
                return Some(CrashDiagnosis {
                    category: CrashCategory::GraphicsDriverGlfw,
                    title: lang.diag_gpu_title().to_string(),
                    description: lang.diag_gpu_desc().to_string(),
                    solution: lang.diag_gpu_solution().to_string(),
                    raw_cause: line.trim().to_string(),
                });
            }
        }
        None
    }

    pub fn show(&mut self, ctx: &egui::Context, lang: Language) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        let diagnosis = self.manual_diagnosis.clone().or_else(|| self.analyze_crash(lang));

        egui::Window::new(lang.console_title())
            .open(&mut is_open)
            .default_size(vec2(820.0, 520.0))
            .min_size(vec2(540.0, 320.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.filter)
                            .hint_text(egui::RichText::new(lang.console_filter_hint()).color(TEXT_MUTED))
                            .desired_width(220.0),
                    );

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.autoscroll, lang.console_autoscroll());

                    ui.add_space(12.0);

                    // Copy Button
                    let (cp_rect, cp_resp) = ui.allocate_exact_size(vec2(78.0, 28.0), Sense::click());
                    let cp_hover = ui.ctx().animate_bool_responsive(cp_resp.id, cp_resp.hovered());
                    let cp_bg = lerp_color(BG_ELEVATED, BG_HOVER, cp_hover);
                    let cp_stroke = lerp_color(BORDER_DEFAULT, RUBY, cp_hover);
                    let cp_text = lerp_color(TEXT_PRIMARY, Color32::WHITE, cp_hover);

                    ui.painter().rect_filled(cp_rect, Rounding::ZERO, cp_bg);
                    ui.painter().rect_stroke(cp_rect, Rounding::ZERO, Stroke::new(1.0, cp_stroke));
                    ui.painter().text(cp_rect.center(), egui::Align2::CENTER_CENTER, lang.console_copy(), egui::FontId::proportional(11.0), cp_text);

                    if cp_resp.clicked() {
                        let full_text = self.logs.join("\n");
                        ui.ctx().copy_text(full_text);
                    }

                    // Clear Button
                    let (cl_rect, cl_resp) = ui.allocate_exact_size(vec2(78.0, 28.0), Sense::click());
                    let cl_hover = ui.ctx().animate_bool_responsive(cl_resp.id, cl_resp.hovered());
                    let cl_bg = lerp_color(BG_ELEVATED, BG_HOVER, cl_hover);
                    let cl_stroke = lerp_color(BORDER_DEFAULT, RUBY, cl_hover);
                    let cl_text = lerp_color(TEXT_MUTED, TEXT_PRIMARY, cl_hover);

                    ui.painter().rect_filled(cl_rect, Rounding::ZERO, cl_bg);
                    ui.painter().rect_stroke(cl_rect, Rounding::ZERO, Stroke::new(1.0, cl_stroke));
                    ui.painter().text(cl_rect.center(), egui::Align2::CENTER_CENTER, lang.console_clear(), egui::FontId::proportional(11.0), cl_text);

                    if cl_resp.clicked() {
                        self.logs.clear();
                        self.manual_diagnosis = None;
                    }

                    // Export Button
                    let (ex_rect, ex_resp) = ui.allocate_exact_size(vec2(86.0, 28.0), Sense::click());
                    let ex_hover = ui.ctx().animate_bool_responsive(ex_resp.id, ex_resp.hovered());
                    let ex_bg = lerp_color(BG_ELEVATED, RUBY, ex_hover);
                    let ex_stroke = lerp_color(RUBY, RUBY_LIGHT, ex_hover);
                    let ex_text = lerp_color(RUBY_LIGHT, Color32::WHITE, ex_hover);

                    ui.painter().rect_filled(ex_rect, Rounding::ZERO, ex_bg);
                    ui.painter().rect_stroke(ex_rect, Rounding::ZERO, Stroke::new(1.0, ex_stroke));
                    ui.painter().text(ex_rect.center(), egui::Align2::CENTER_CENTER, lang.console_export(), egui::FontId::proportional(11.0), ex_text);

                    if ex_resp.clicked() {
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

                // Crash Diagnostic Card (if detected)
                if let Some(diag) = &diagnosis {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(26, 12, 16))
                        .stroke(Stroke::new(1.5, RUBY))
                        .rounding(Rounding::ZERO)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠️").font(egui::FontId::proportional(22.0)));
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(lang.diag_header_label())
                                                .font(egui::FontId::proportional(11.0))
                                                .strong()
                                                .color(RUBY_LIGHT),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("• {}", diag.title))
                                                .font(egui::FontId::proportional(13.0))
                                                .strong()
                                                .color(Color32::WHITE),
                                        );
                                    });
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(&diag.description)
                                            .font(egui::FontId::proportional(12.0))
                                            .color(TEXT_PRIMARY),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(format!("💡 {}", diag.solution))
                                            .font(egui::FontId::proportional(12.0))
                                            .strong()
                                            .color(Color32::from_rgb(255, 204, 100)),
                                    );
                                });
                            });
                        });
                    ui.add_space(6.0);
                }

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
                                || line.contains("[AMC CRASH]")
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
        self.is_open = is_open;
    }
}
