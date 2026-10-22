use crate::theme::{
    lerp_color, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_LIGHT, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::pixel;
use egui::{vec2, Color32, Sense, Stroke, Ui};

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
    let millida = crate::theme::palette_id() == crate::theme::PaletteId::Millida;
    let active_text: Color32 = if millida {
        crate::theme::BG.into()
    } else {
        Color32::WHITE
    };
    let text_col = lerp_color(
        lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
        active_text,
        act_t,
    );

    if millida {
        pixel::plate(ui.painter(), rect, 5.0, bg, stroke_col);
    } else {
        ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
        ui.painter().rect_stroke(
            rect,
            crate::theme::rounding(),
            Stroke::new(1.0_f32, stroke_col),
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        text_col,
    );

    resp.clicked()
}
