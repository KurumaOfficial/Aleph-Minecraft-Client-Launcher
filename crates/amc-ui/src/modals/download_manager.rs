use crate::theme::{TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use crate::widgets::icons::{self, Icon};
use amc_downloader::DownloadRecord;
use egui::{vec2, Color32, ScrollArea, Sense};

/// Download manager modal (CONCEPT P3): finished-download history from
/// the engine (newest first) with a clear button. Active transfers keep
/// living in the download overlay and the bottom progress bar — this
/// window answers "what already happened".
#[derive(Default)]
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
        let screen = ctx.screen_rect();
        let max_h = (screen.height() - 120.0).max(280.0);
        let mut open = true;
        egui::Window::new(lang.dlman_title())
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(520.0)
            .default_size(vec2(560.0, 320.0))
            .max_height(max_h)
            .show(ctx, |ui| {
                if history.is_empty() {
                    ui.add_space(36.0);
                    ui.vertical_centered(|ui| {
                        let (icon, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
                        ui.painter().rect_filled(
                            icon,
                            crate::theme::rounding(),
                            Color32::from(crate::theme::BG_CARD),
                        );
                        icons::draw_centered(
                            ui.painter(),
                            Icon::Download,
                            icon.center(),
                            24.0,
                            TEXT_MUTED,
                        );
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(lang.dlman_empty())
                                .font(egui::FontId::proportional(13.0))
                                .color(TEXT_MUTED),
                        );
                    });
                } else {
                    ScrollArea::vertical()
                        .max_height((max_h - 100.0).max(160.0))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 6.0);
                            for record in history.iter().rev() {
                                ui.horizontal(|ui| {
                                    let (state_rect, _) = ui.allocate_exact_size(
                                        vec2(12.0, 12.0),
                                        egui::Sense::hover(),
                                    );
                                    let (icon, col) = if record.ok {
                                        (Icon::Check, TEXT_MUTED)
                                    } else {
                                        (Icon::Close, TEXT_PRIMARY)
                                    };
                                    icons::draw(ui.painter(), icon, state_rect, col, 1.4);
                                    let mut name = record.filename.clone();
                                    if name.chars().count() > 44 {
                                        name = format!(
                                            "{}…",
                                            name.chars().take(44).collect::<String>()
                                        );
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
                                                egui::RichText::new(lang.disk_size(record.bytes))
                                                    .font(egui::FontId::proportional(11.0))
                                                    .color(TEXT_MUTED),
                                            );
                                        },
                                    );
                                });
                            }
                        });
                }
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
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if !open {
            close = true;
        }
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
    fn test_manager_action_default_is_none() {
        assert!(matches!(ManagerAction::default(), ManagerAction::None));
    }
}
