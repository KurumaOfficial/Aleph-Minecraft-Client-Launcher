use crate::theme::{
    with_alpha, BG_CARD, BORDER_SUBTLE, DANGER, SUCCESS, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, Icon};
use amc_downloader::DownloadRecord;
use egui::{vec2, Color32, Label, Rounding, ScrollArea, Sense, Stroke};

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

fn status_text(record: &DownloadRecord, lang: amc_core::Language) -> &'static str {
    if record.ok {
        lang.dlman_status_done()
    } else {
        lang.dlman_status_failed()
    }
}

fn draw_record(ui: &mut egui::Ui, record: &DownloadRecord, lang: amc_core::Language) {
    egui::Frame::none()
        .fill(BG_CARD.into())
        .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
                let accent = if record.ok { SUCCESS } else { DANGER };
                ui.painter()
                    .circle_filled(badge.center(), 13.0, with_alpha(accent, 35));
                icons::draw_centered(
                    ui.painter(),
                    if record.ok { Icon::Check } else { Icon::Close },
                    badge.center(),
                    13.0,
                    accent,
                );
                ui.add_space(6.0);

                let meta_w = 88.0;
                let name_w = (ui.available_width() - meta_w - 20.0).max(80.0);
                ui.vertical(|ui| {
                    ui.set_width(name_w);
                    ui.add_space(1.0);
                    let font = egui::FontId::proportional(12.5);
                    let full_w = ui
                        .painter()
                        .layout_no_wrap(record.filename.clone(), font.clone(), TEXT_HEADING.into())
                        .size()
                        .x;
                    let name = ui
                        .allocate_ui_with_layout(
                            vec2(name_w, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add(
                                    Label::new(
                                        egui::RichText::new(&record.filename)
                                            .font(font)
                                            .color(TEXT_HEADING),
                                    )
                                    .truncate()
                                    .sense(Sense::hover()),
                                )
                            },
                        )
                        .inner;
                    if full_w > name_w {
                        name.on_hover_text(&record.filename);
                    }
                    ui.add_space(1.0);
                    let (status_col, status) = if record.ok {
                        (SUCCESS, status_text(record, lang))
                    } else {
                        (DANGER, status_text(record, lang))
                    };
                    ui.label(
                        egui::RichText::new(status)
                            .font(egui::FontId::proportional(10.5))
                            .color(status_col),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(lang.disk_size(record.bytes))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_PRIMARY),
                            );
                        });
                        ui.add_space(2.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(lang.dlman_duration(record.millis))
                                    .font(egui::FontId::proportional(10.0))
                                    .color(TEXT_MUTED),
                            );
                        });
                    });
                });
            });
        });
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
            .default_size(vec2(560.0, 340.0))
            .max_height(max_h)
            .show(ctx, |ui| {
                if history.is_empty() {
                    ui.add_space(32.0);
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
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(lang.dlman_empty_hint())
                                .font(egui::FontId::proportional(11.0))
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
                                draw_record(ui, record, lang);
                            }
                        });
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(lang.home_drawer_close()).clicked() {
                        close = true;
                    }
                    ui.label(
                        egui::RichText::new(lang.dlman_count(history.len()))
                            .font(egui::FontId::proportional(10.5))
                            .color(TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !history.is_empty() && ui.button(lang.dlman_clear()).clicked() {
                            action = ManagerAction::Clear;
                        }
                    });
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

    #[test]
    fn test_status_text_follows_download_result() {
        let ok = DownloadRecord {
            filename: "a.jar".to_string(),
            bytes: 10,
            ok: true,
            millis: 1500,
        };
        let failed = DownloadRecord {
            ok: false,
            ..ok.clone()
        };
        assert_eq!(status_text(&ok, amc_core::Language::English), "Completed");
        assert_eq!(status_text(&failed, amc_core::Language::Russian), "Ошибка");
    }
}
