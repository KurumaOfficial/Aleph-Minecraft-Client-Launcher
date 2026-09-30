use crate::theme::{
    lerp_color, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY,
};
use egui::{vec2, Color32, Rounding, Sense, Stroke, Ui};

/// Selectable option chip shared by the settings groups and the first-run
/// wizard. Returns true when clicked.
pub fn choice_chip(
    ui: &mut Ui,
    label: &str,
    w: f32,
    h: f32,
    font_size: f32,
    is_active: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let act_t = ui.ctx().animate_bool(resp.id.with("act"), is_active);

    let bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, hover_t), RUBY, act_t);
    let stroke_col = lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, act_t);
    let text_col = lerp_color(
        lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
        Color32::WHITE,
        act_t,
    );

    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
    ui.painter()
        .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0_f32, stroke_col));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        text_col,
    );

    resp.clicked()
}
