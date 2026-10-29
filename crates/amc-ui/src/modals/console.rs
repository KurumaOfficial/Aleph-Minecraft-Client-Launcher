use crate::theme::{lerp_color, BG_HOVER, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY};
use crate::widgets::icons::{self, icon_button, Icon};
use amc_core::Language;
use egui::{pos2, vec2, Color32, Rect, ScrollArea, Sense, Stroke, TextEdit, Ui};

/// Long log lines stay one fixed-height row. Their hover card is bounded so
/// a stack trace cannot turn the tooltip into a full-screen wall of text.
fn tooltip_line(line: &str) -> String {
    const MAX: usize = 600;
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        format!("{}…", line.chars().take(MAX).collect::<String>())
    }
}

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
    lower_logs: Vec<String>,
    diag_cache: Option<CrashDiagnosis>,
    diag_key: Option<(usize, Language)>,
    filter_cache: Vec<usize>,
    filter_key: Option<(String, usize)>,
    copied_at: Option<std::time::Instant>,
}

impl Default for ConsoleModal {
    fn default() -> Self {
        Self {
            is_open: false,
            logs: Vec::new(),
            filter: String::new(),
            autoscroll: true,
            manual_diagnosis: None,
            lower_logs: Vec::new(),
            diag_cache: None,
            diag_key: None,
            filter_cache: Vec::new(),
            filter_key: None,
            copied_at: None,
        }
    }
}

/// Console toolbar plate with an icon and label. `None` disables the action
/// and drops the hover feedback instead of leaving a clickable no-op.
fn console_action(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    width: f32,
    enabled: bool,
    primary: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        vec2(width, 28.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hover = if enabled {
        ui.ctx().animate_bool_responsive(resp.id, resp.hovered())
    } else {
        0.0
    };
    let (bg, stroke, icon_color, text_color) = if !enabled {
        (
            Color32::from(crate::theme::BG_ELEVATED),
            Color32::from(crate::theme::BORDER_DEFAULT),
            Color32::from(TEXT_MUTED),
            Color32::from(TEXT_MUTED),
        )
    } else if primary {
        (
            lerp_color(crate::theme::BG_ELEVATED, RUBY, hover),
            lerp_color(RUBY, RUBY_LIGHT, hover),
            lerp_color(RUBY_LIGHT, Color32::WHITE, hover),
            lerp_color(RUBY_LIGHT, Color32::WHITE, hover),
        )
    } else {
        (
            lerp_color(
                Color32::from(crate::theme::BG_ELEVATED),
                Color32::from(BG_HOVER),
                hover,
            ),
            lerp_color(crate::theme::BORDER_DEFAULT, RUBY, hover),
            lerp_color(Color32::from(TEXT_PRIMARY), Color32::WHITE, hover),
            lerp_color(Color32::from(TEXT_PRIMARY), Color32::WHITE, hover),
        )
    };
    ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
    ui.painter()
        .rect_stroke(rect, crate::theme::rounding(), Stroke::new(1.0_f32, stroke));
    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 14.0, rect.center().y), vec2(13.0, 13.0));
    icons::draw(ui.painter(), icon, icon_rect, icon_color, 1.3);
    ui.painter().text(
        pos2(rect.left() + 25.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        text_color,
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

impl ConsoleModal {
    pub fn push_line(&mut self, line: String) {
        if self.logs.len() >= 5000 {
            self.logs.remove(0);
            self.lower_logs.remove(0);
        }
        self.lower_logs.push(line.to_lowercase());
        self.logs.push(line);
        // New output may change the diagnosis; recomputed lazily in `show`.
        self.diag_key = None;
    }

    pub fn clear(&mut self) {
        self.logs.clear();
        self.lower_logs.clear();
        self.manual_diagnosis = None;
        self.diag_cache = None;
        self.diag_key = None;
        self.filter_cache.clear();
        self.filter_key = None;
        self.copied_at = None;
    }

    pub fn set_crash_message(&mut self, msg: String, lang: Language) {
        self.is_open = true;
        self.push_line(format!("[AMC CRASH] {msg}"));
        if let Some(diag) = self.analyze_crash(lang) {
            self.manual_diagnosis = Some(diag);
        }
        // Manual diagnosis wins; drop the auto cache so `show` prefers it.
        self.diag_cache = None;
        self.diag_key = None;
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
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.is_open = false;
            return;
        }

        let mut is_open = self.is_open;
        let max_h = (ctx.screen_rect().height() - 160.0).max(320.0);
        // Auto diagnosis memoized by (logs_len, lang); manual wins.
        let diag_key = (self.logs.len(), lang);
        if self.diag_key != Some(diag_key) {
            self.diag_cache = self.analyze_crash(lang);
            self.diag_key = Some(diag_key);
        }
        let diagnosis = self
            .manual_diagnosis
            .clone()
            .or_else(|| self.diag_cache.clone());

        // Filter memoized by (query, logs_len) over pre-lowercased lines —
        // no per-line `to_lowercase` per frame, and the counter in the
        // toolbar always describes the rows that are actually visible.
        let filter_query = self.filter.trim().to_lowercase();
        let filter_key = (filter_query.clone(), self.logs.len());
        if self.filter_key.as_ref() != Some(&filter_key) {
            if filter_query.is_empty() {
                self.filter_cache.clear();
            } else {
                self.filter_cache = self
                    .lower_logs
                    .iter()
                    .enumerate()
                    .filter(|(_, lower)| lower.contains(&filter_query))
                    .map(|(idx, _)| idx)
                    .collect();
            }
            self.filter_key = Some(filter_key);
        }
        let use_filter = !self.filter.trim().is_empty();
        let visible_count = if use_filter {
            self.filter_cache.len()
        } else {
            self.logs.len()
        };
        let has_logs = !self.logs.is_empty();

        egui::Window::new(lang.console_title())
            .open(&mut is_open)
            .default_size(vec2(820.0, (max_h - 30.0).max(320.0)))
            .max_height(max_h)
            .min_size(vec2(540.0, 320.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    let filter_id = ui.id().with("console_filter");
                    let filter_w = if self.filter.is_empty() { 220.0 } else { 190.0 };
                    ui.add(
                        TextEdit::singleline(&mut self.filter)
                            .id(filter_id)
                            .hint_text(
                                egui::RichText::new(lang.console_filter_hint()).color(TEXT_MUTED),
                            )
                            .desired_width(filter_w),
                    );
                    if !self.filter.is_empty()
                        && icon_button(ui, Icon::Close, TEXT_MUTED)
                            .on_hover_text(lang.console_clear_filter())
                            .clicked()
                    {
                        self.filter.clear();
                        ui.memory_mut(|m| m.request_focus(filter_id));
                    }

                    ui.add_space(4.0);
                    ui.checkbox(&mut self.autoscroll, lang.console_autoscroll());
                    ui.add_space(6.0);

                    let copied = self
                        .copied_at
                        .is_some_and(|at| at.elapsed() < std::time::Duration::from_millis(1500));
                    let copy_label = if copied {
                        lang.console_copied()
                    } else {
                        lang.console_copy()
                    };
                    let copy_resp =
                        console_action(ui, Icon::Clipboard, copy_label, 112.0, has_logs, false);
                    if !has_logs {
                        copy_resp.clone().on_hover_text(lang.console_empty_title());
                    }
                    if copy_resp.clicked() {
                        ui.ctx().copy_text(self.logs.join("\n"));
                        self.copied_at = Some(std::time::Instant::now());
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_millis(1600));
                    }

                    ui.add_space(6.0);
                    let clear_resp = console_action(
                        ui,
                        Icon::Trash,
                        lang.console_clear(),
                        86.0,
                        has_logs,
                        false,
                    );
                    if !has_logs {
                        clear_resp.clone().on_hover_text(lang.console_empty_title());
                    }
                    if clear_resp.clicked() {
                        self.clear();
                    }

                    ui.add_space(6.0);
                    let export_resp = console_action(
                        ui,
                        Icon::Download,
                        lang.console_export(),
                        92.0,
                        has_logs,
                        true,
                    );
                    if !has_logs {
                        export_resp
                            .clone()
                            .on_hover_text(lang.console_empty_title());
                    }
                    if export_resp.clicked() {
                        if let Some(dest) = rfd::FileDialog::new()
                            .set_file_name("minecraft-latest.log")
                            .save_file()
                        {
                            let _ = std::fs::write(dest, self.logs.join("\n"));
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(
                                lang.console_line_count(visible_count, self.logs.len()),
                            )
                            .font(egui::FontId::proportional(10.5))
                            .color(TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(8.0);

                // Crash Diagnostic Card (if detected)
                if let Some(diag) = &diagnosis {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(26, 12, 16))
                        .stroke(Stroke::new(1.5_f32, RUBY))
                        .rounding(crate::theme::rounding())
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (warn_rect, _) =
                                    ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                                icons::draw_centered(
                                    ui.painter(),
                                    Icon::Warning,
                                    warn_rect.center(),
                                    20.0,
                                    RUBY_LIGHT,
                                );
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
                                    icons::icon_label(
                                        ui,
                                        Icon::Bulb,
                                        &diag.solution,
                                        Color32::from_rgb(255, 204, 100),
                                        egui::FontId::proportional(12.0),
                                    );
                                });
                            });
                        });
                    ui.add_space(6.0);
                }

                ui.separator();
                ui.add_space(4.0);

                let row_height = 18.0;
                if !has_logs {
                    let empty_h = (max_h - 190.0).max(160.0);
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), empty_h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.add_space(38.0);
                            let (icon_rect, _) =
                                ui.allocate_exact_size(vec2(54.0, 54.0), Sense::hover());
                            ui.painter().rect_filled(
                                icon_rect,
                                crate::theme::rounding(),
                                Color32::from(crate::theme::BG_CARD),
                            );
                            icons::draw_centered(
                                ui.painter(),
                                Icon::Console,
                                icon_rect.center(),
                                23.0,
                                TEXT_MUTED,
                            );
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new(lang.console_empty_title())
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add_space(3.0);
                            ui.label(
                                egui::RichText::new(lang.console_empty_hint())
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            );
                        },
                    );
                } else if visible_count == 0 {
                    let empty_h = (max_h - 190.0).max(160.0);
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), empty_h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.add_space(38.0);
                            let (icon_rect, _) =
                                ui.allocate_exact_size(vec2(54.0, 54.0), Sense::hover());
                            ui.painter().rect_filled(
                                icon_rect,
                                crate::theme::rounding(),
                                Color32::from(crate::theme::BG_CARD),
                            );
                            icons::draw_centered(
                                ui.painter(),
                                Icon::Search,
                                icon_rect.center(),
                                23.0,
                                TEXT_MUTED,
                            );
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new(lang.console_no_matches())
                                    .font(egui::FontId::proportional(13.0))
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add_space(8.0);
                            if ui.button(lang.console_clear_filter()).clicked() {
                                self.filter.clear();
                            }
                        },
                    );
                } else {
                    let logs = &self.logs;
                    let indices = &self.filter_cache;
                    ScrollArea::vertical()
                        .stick_to_bottom(self.autoscroll)
                        .max_height((max_h - 150.0).max(160.0))
                        .auto_shrink([false, false])
                        .show_rows(ui, row_height, visible_count, |ui, row_range| {
                            // One virtualized row is exactly `row_height` high:
                            // no item gaps, otherwise the row range and the
                            // scrollbar would drift apart.
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                            let font = egui::FontId::monospace(11.0);
                            let available = ui.available_width();

                            for i in row_range {
                                let line: &String = if use_filter {
                                    &logs[indices[i]]
                                } else {
                                    &logs[i]
                                };
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
                                    TEXT_PRIMARY.into()
                                };

                                let full_width = ui
                                    .painter()
                                    .layout_no_wrap(line.clone(), font.clone(), color)
                                    .size()
                                    .x;
                                let resp = ui
                                    .allocate_ui_with_layout(
                                        vec2(available, row_height),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(line)
                                                        .font(font.clone())
                                                        .color(color),
                                                )
                                                .truncate()
                                                .sense(Sense::hover()),
                                            )
                                        },
                                    )
                                    .inner;
                                if full_width > available {
                                    resp.on_hover_text(tooltip_line(line));
                                }
                            }
                        });
                }
            });
        self.is_open = is_open;
    }
}
