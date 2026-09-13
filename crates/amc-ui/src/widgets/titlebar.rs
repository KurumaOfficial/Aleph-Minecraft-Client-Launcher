use egui::{vec2, Color32, Response, Sense, Stroke, Ui, UiBuilder, ViewportCommand};
use crate::theme::{BORDER_SUBTLE, RUBY, TEXT_MUTED, TEXT_PRIMARY};

pub struct TitleBar;

impl TitleBar {
    pub fn show(ui: &mut Ui, _title: &str) {
        let title_bar_height = 36.0;
        let (rect, response) = ui.allocate_exact_size(
            vec2(ui.available_width(), title_bar_height),
            Sense::click_and_drag(),
        );

        if response.dragged() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Draw bottom border
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_SUBTLE),
        );

        // Render controls on the right
        let mut controls_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );

        controls_ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

        // Close button
        if Self::window_button(&mut controls_ui, "✕", true).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        // Maximize button
        if Self::window_button(&mut controls_ui, "□", false).clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // Minimize button
        if Self::window_button(&mut controls_ui, "—", false).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
    }

    fn window_button(ui: &mut Ui, symbol: &str, is_close: bool) -> Response {
        let (rect, resp) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());

        let bg = if resp.is_pointer_button_down_on() {
            if is_close {
                RUBY
            } else {
                Color32::from_rgb(0x1a, 0x14, 0x14)
            }
        } else if resp.hovered() {
            if is_close {
                RUBY
            } else {
                Color32::from_rgb(0x14, 0x0e, 0x0e)
            }
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 0.0, bg);
        }

        let text_color = if resp.hovered() && is_close {
            Color32::WHITE
        } else if resp.hovered() {
            TEXT_PRIMARY
        } else {
            TEXT_MUTED
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            symbol,
            egui::FontId::proportional(12.0),
            text_color,
        );

        resp
    }
}
