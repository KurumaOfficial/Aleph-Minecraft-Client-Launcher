use egui::{vec2, Color32, Rounding, Ui};
use amc_core::types::ReleaseType;
use crate::theme::{BADGE_ALPHA, BADGE_BETA, BADGE_OLD, BADGE_SNAPSHOT, RUBY};

pub fn draw_custom_badge(ui: &mut Ui, text: &str, bg_color: Color32) {
    let font = egui::FontId::proportional(10.0);
    let padding = vec2(8.0, 3.0);
    let text_size = ui.painter().layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE).size();
    let badge_size = text_size + padding * 2.0;

    let (rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::ZERO, bg_color);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        font,
        Color32::WHITE,
    );
}

pub fn draw_badge(ui: &mut Ui, release_type: ReleaseType) {
    let (text, bg_color) = match release_type {
        ReleaseType::Release => ("РЕЛИЗ", RUBY),
        ReleaseType::Snapshot => ("СНАПШОТ", BADGE_SNAPSHOT),
        ReleaseType::Beta => ("БЕТА", BADGE_BETA),
        ReleaseType::Alpha => ("АЛЬФА", BADGE_ALPHA),
        ReleaseType::Old => ("СТАРАЯ", BADGE_OLD),
    };

    draw_custom_badge(ui, text, bg_color);
}
