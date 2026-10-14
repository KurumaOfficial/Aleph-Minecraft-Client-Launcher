use crate::theme::{TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use amc_downloader::DownloadRecord;
use egui::{vec2, ScrollArea};

/// Download manager modal (CONCEPT P3): finished-download history from
/// the engine (newest first) with a clear button. Active transfers keep
/// living in the download overlay and the bottom progress bar — this
/// window answers "what already happened".
pub struct DownloadManager {
    pub open: bool,
}

/// Manager outcome for the app shell.
#[derive(Debug, Default)]
pub enum ManagerAction {
    #[default]
    None,
    Clear,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self { open: false }
    }
}

/// Human size for one history row. Pure for tests.
pub fn format_history_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

impl DownloadManager {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        history: &[DownloadRecord],
        lang: amc_core::Language,
    ) -> ManagerAction {
        if !self.open {
            return ManagerAction::None;
        }
        let mut close = false;
        let mut action = ManagerAction::None;
        egui::Window::new(lang.dlman_title())
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(520.0)
            .default_size(vec2(560.0, 420.0))
            .show(ctx, |ui| {
                if history.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(lang.dlman_empty()).color(TEXT_MUTED));
                    });
                }
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 6.0);
                        for record in history.iter().rev() {
                            ui.horizontal(|ui| {
                                let (glyph, col) = if record.ok {
                                    ("✓", TEXT_MUTED)
                                } else {
                                    ("✗", TEXT_PRIMARY)
                                };
                                ui.label(
                                    egui::RichText::new(glyph)
                                        .font(egui::FontId::proportional(12.0))
                                        .color(col),
                                );
                                let mut name = record.filename.clone();
                                if name.chars().count() > 44 {
                                    name =
                                        format!("{}…", name.chars().take(44).collect::<String>());
                                }
                                ui.label(
                                    egui::RichText::new(name)
                                        .font(egui::FontId::proportional(12.0))
                                        .color(TEXT_HEADING),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format_history_bytes(record.bytes))
                                                .font(egui::FontId::proportional(11.0))
                                                .color(TEXT_MUTED),
                                        );
                                    },
                                );
                            });
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(lang.home_drawer_close()).clicked() {
                        close = true;
                    }
                    if !history.is_empty() && ui.button(lang.dlman_clear()).clicked() {
                        action = ManagerAction::Clear;
                    }
                });
            });
        if close {
            self.open = false;
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_history_bytes() {
        assert_eq!(format_history_bytes(0), "0 B");
        assert_eq!(format_history_bytes(512), "512 B");
        assert_eq!(format_history_bytes(2048), "2.0 KB");
        assert_eq!(format_history_bytes(5_242_880), "5.0 MB");
    }
}
