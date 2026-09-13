use egui::{vec2, ProgressBar, Rounding};
use amc_downloader::DownloadProgress;
use crate::theme::{RUBY, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};

pub struct DownloadOverlay;

impl DownloadOverlay {
    pub fn show(
        ctx: &egui::Context,
        progress: &DownloadProgress,
        title: &str,
        on_cancel: impl FnOnce(),
    ) {
        let mut cancel_clicked = false;

        egui::Window::new("Загрузка компонентов")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(460.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(title)
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(TEXT_HEADING),
                );

                ui.add_space(10.0);

                // Progress bar
                let ratio = progress.ratio();
                let bar = ProgressBar::new(ratio)
                    .text(format!("{:.1}%", progress.percent()))
                    .fill(RUBY)
                    .rounding(Rounding::ZERO);

                ui.add(bar);

                ui.add_space(10.0);

                // Current file
                if !progress.current_file.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("Файл: {}", progress.current_file))
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                }

                ui.add_space(6.0);

                // Speed, Bytes, ETA in one row
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("💾 {}", progress.formatted_bytes()))
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_PRIMARY),
                    );

                    ui.add_space(16.0);

                    ui.label(
                        egui::RichText::new(format!("⚡ {}", progress.formatted_speed()))
                            .font(egui::FontId::proportional(12.0))
                            .color(RUBY_LIGHT),
                    );

                    ui.add_space(16.0);

                    ui.label(
                        egui::RichText::new(format!("⏳ {}", progress.formatted_eta()))
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                });

                ui.add_space(14.0);

                let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(100.0, 30.0), egui::Sense::click());
                let btn_hover = ui.ctx().animate_bool_responsive(btn_resp.id, btn_resp.hovered());
                let btn_bg = crate::theme::lerp_color(crate::theme::BG_ELEVATED, crate::theme::BG_HOVER, btn_hover);
                let btn_stroke = crate::theme::lerp_color(crate::theme::BORDER_DEFAULT, RUBY, btn_hover);
                let btn_text = crate::theme::lerp_color(TEXT_MUTED, TEXT_PRIMARY, btn_hover);

                ui.painter().rect_filled(btn_rect, Rounding::ZERO, btn_bg);
                ui.painter().rect_stroke(btn_rect, Rounding::ZERO, egui::Stroke::new(1.0, btn_stroke));
                ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, "Отмена", egui::FontId::proportional(11.5), btn_text);

                if btn_resp.clicked() {
                    cancel_clicked = true;
                }
            });

        if cancel_clicked {
            on_cancel();
        }
    }
}
