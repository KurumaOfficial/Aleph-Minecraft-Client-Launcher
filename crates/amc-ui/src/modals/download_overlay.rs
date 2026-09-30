use crate::theme::{RUBY, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use amc_core::Language;
use amc_downloader::DownloadProgress;
use egui::{vec2, ProgressBar, Rounding};

pub struct DownloadOverlay;

impl DownloadOverlay {
    pub fn show(
        ctx: &egui::Context,
        lang: Language,
        progress: &DownloadProgress,
        title: &str,
        paused: bool,
        on_pause: impl FnOnce(),
        on_resume: impl FnOnce(),
        on_cancel: impl FnOnce(),
    ) {
        let mut pause_clicked = false;
        let mut resume_clicked = false;
        let mut cancel_clicked = false;

        egui::Window::new(lang.overlay_title())
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

                if paused {
                    ui.label(
                        egui::RichText::new(lang.status_download_paused())
                            .font(egui::FontId::proportional(12.0))
                            .color(RUBY_LIGHT),
                    );
                }

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
                        egui::RichText::new(lang.overlay_file(&progress.current_file))
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

                ui.horizontal(|ui| {
                    if ui
                        .button(if paused {
                            lang.overlay_btn_resume()
                        } else {
                            lang.overlay_btn_pause()
                        })
                        .clicked()
                    {
                        if paused {
                            resume_clicked = true;
                        } else {
                            pause_clicked = true;
                        }
                    }
                    if ui.button(lang.overlay_btn_cancel()).clicked() {
                        cancel_clicked = true;
                    }
                });
            });

        if pause_clicked {
            on_pause();
        }
        if resume_clicked {
            on_resume();
        }
        if cancel_clicked {
            on_cancel();
        }
    }
}
