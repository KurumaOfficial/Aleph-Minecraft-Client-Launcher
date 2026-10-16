use egui::{vec2, Color32, Rounding, Ui};

pub fn draw_custom_badge(ui: &mut Ui, text: &str, bg_color: impl Into<Color32>) {
    let bg_color = bg_color.into();
    let font = egui::FontId::proportional(10.0);
    let padding = vec2(8.0, 3.0);
    let text_size = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE)
        .size();
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
