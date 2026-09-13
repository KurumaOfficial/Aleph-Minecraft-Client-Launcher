use egui::{vec2, Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, ViewportCommand};
use crate::theme::{BORDER_SUBTLE, RUBY, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};

pub struct TitleBar;

impl TitleBar {
    pub fn show(ui: &mut Ui, title: &str) {
        let height = 36.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), height));

        // Draw bottom separator
        ui.painter().line_segment(
            [bar_rect.left_bottom(), bar_rect.right_bottom()],
            Stroke::new(1.0, BORDER_SUBTLE),
        );

        // Calculate layout
        let controls_width = 36.0 * 3.0;
        let drag_rect = Rect::from_min_size(
            bar_rect.min,
            vec2((bar_rect.width() - controls_width).max(0.0), height),
        );

        // Drag area for the window
        let drag_resp = ui.allocate_rect(drag_rect, Sense::click_and_drag());
        if drag_resp.dragged() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Draw title inside drag area
        ui.painter().text(
            Pos2::new(drag_rect.left() + 16.0, drag_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.0),
            TEXT_HEADING,
        );

        // Window controls on the right (strictly click sense, no drag)
        let controls_start_x = bar_rect.right() - controls_width;

        // Minimize Button
        let min_rect = Rect::from_min_size(Pos2::new(controls_start_x, bar_rect.top()), vec2(36.0, height));
        if Self::draw_control_button(ui, min_rect, "—", false).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        // Maximize Button
        let max_rect = Rect::from_min_size(Pos2::new(controls_start_x + 36.0, bar_rect.top()), vec2(36.0, height));
        if Self::draw_control_button(ui, max_rect, "□", false).clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // Close Button
        let close_rect = Rect::from_min_size(Pos2::new(controls_start_x + 72.0, bar_rect.top()), vec2(36.0, height));
        if Self::draw_control_button(ui, close_rect, "✕", true).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        // Advance cursor
        ui.advance_cursor_after_rect(bar_rect);
    }

    fn draw_control_button(ui: &mut Ui, rect: Rect, symbol: &str, is_close: bool) -> Response {
        let resp = ui.allocate_rect(rect, Sense::click());

        let bg = if resp.is_pointer_button_down_on() {
            if is_close {
                RUBY
            } else {
                Color32::from_rgb(0x1e, 0x14, 0x15)
            }
        } else if resp.hovered() {
            if is_close {
                RUBY
            } else {
                Color32::from_rgb(0x16, 0x0D, 0x0E)
            }
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, Rounding::ZERO, bg);
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
