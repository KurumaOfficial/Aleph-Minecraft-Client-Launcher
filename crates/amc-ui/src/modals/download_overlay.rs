use crate::theme::{RUBY, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use amc_core::Language;
use amc_downloader::DownloadProgress;
use egui::{vec2, ProgressBar, Rounding};

pub struct DownloadOverlay;

/// Overlay button pressed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    Pause,
    Resume,
    Cancel,
}

impl DownloadOverlay {
    pub fn show(
        ctx: &egui::Context,
        lang: Language,
        progress: &DownloadProgress,
        title: &str,
        paused: bool,
        on_action: impl FnOnce(OverlayAction),
    ) {
        let mut action = None;

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
                    let resume_label = if paused {
                        lang.overlay_btn_resume()
                    } else {
                        lang.overlay_btn_pause()
                    };
                    if ui.button(resume_label).clicked() {
                        action = Some(if paused {
                            OverlayAction::Resume
                        } else {
                            OverlayAction::Pause
                        });
                    }
                    if ui.button(lang.overlay_btn_cancel()).clicked() {
                        action = Some(OverlayAction::Cancel);
                    }
                });
            });

        if let Some(a) = action {
            on_action(a);
        }
    }
}
