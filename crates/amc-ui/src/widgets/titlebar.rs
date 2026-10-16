use crate::theme::{
    lerp_color, BORDER_SUBTLE, CARD_HOVER, DANGER, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, Icon};
use egui::{vec2, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, ViewportCommand};

#[derive(Debug, Clone, Copy, Default)]
pub struct TitleBarResponse {
    pub console_clicked: bool,
    pub gallery_clicked: bool,
    pub downloads_clicked: bool,
}

pub struct TitleBar;

impl TitleBar {
    pub fn show(ui: &mut Ui, title: &str, lang: amc_core::Language) -> TitleBarResponse {
        let mut response = TitleBarResponse::default();
        let height = 36.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), height));

        // Draw bottom separator
        ui.painter().line_segment(
            [bar_rect.left_bottom(), bar_rect.right_bottom()],
            Stroke::new(1.0_f32, BORDER_SUBTLE),
        );

        // 6 buttons on right: Gallery, Downloads, Console, Minimize, Maximize, Close
        let button_w = 36.0;
        let controls_width = button_w * 6.0;
        let drag_rect = Rect::from_min_size(
            bar_rect.min,
            vec2((bar_rect.width() - controls_width).max(0.0), height),
        );

        // Drag area for the window
        let drag_resp = ui.allocate_rect(drag_rect, Sense::click_and_drag());
        if drag_resp.dragged() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if drag_resp.double_clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // Draw title inside drag area
        ui.painter().text(
            Pos2::new(drag_rect.left() + 16.0, drag_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.0),
            TEXT_HEADING.into(),
        );

        // Window controls on the right
        let controls_start_x = bar_rect.right() - controls_width;

        // Gallery Button (screenshots from every instance)
        let gal_rect = Rect::from_min_size(
            Pos2::new(controls_start_x, bar_rect.top()),
            vec2(button_w, height),
        );
        let gal_resp = Self::draw_control_button(ui, gal_rect, Icon::Gallery, false);
        if gal_resp.clicked() {
            response.gallery_clicked = true;
        }
        gal_resp.on_hover_text(lang.screens_title());

        // Console Button
        let con_rect = Rect::from_min_size(
            Pos2::new(controls_start_x + button_w, bar_rect.top()),
            vec2(button_w, height),
        );
        let con_resp = Self::draw_control_button(ui, con_rect, Icon::Console, false);
        if con_resp.clicked() {
            response.console_clicked = true;
        }
        con_resp.on_hover_text(lang.titlebar_console_tooltip());

        // Downloads Button (manager: active overlay plus history)
        let dl_rect = Rect::from_min_size(
            Pos2::new(controls_start_x + button_w * 2.0, bar_rect.top()),
            vec2(button_w, height),
        );
        let dl_resp = Self::draw_control_button(ui, dl_rect, Icon::Download, false);
        if dl_resp.clicked() {
            response.downloads_clicked = true;
        }
        dl_resp.on_hover_text(lang.dlman_title());

        // Minimize Button
        let min_rect = Rect::from_min_size(
            Pos2::new(controls_start_x + button_w * 3.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, min_rect, Icon::Minimize, false).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        // Maximize Button
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        let max_icon = if is_maximized {
            Icon::Restore
        } else {
            Icon::Maximize
        };
        let max_rect = Rect::from_min_size(
            Pos2::new(controls_start_x + button_w * 4.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, max_rect, max_icon, false).clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // Close Button
        let close_rect = Rect::from_min_size(
            Pos2::new(controls_start_x + button_w * 5.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, close_rect, Icon::Close, true).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        // Advance cursor
        ui.advance_cursor_after_rect(bar_rect);
        response
    }

    fn draw_control_button(ui: &mut Ui, rect: Rect, icon: Icon, is_close: bool) -> Response {
        let resp = ui.allocate_rect(rect, Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let press_t = ui
            .ctx()
            .animate_bool_responsive(resp.id.with("pr"), resp.is_pointer_button_down_on());

        let target_bg: Color32 = if is_close {
            DANGER.into()
        } else {
            CARD_HOVER.into()
        };

        let mut bg = lerp_color(Color32::TRANSPARENT, target_bg, hover_t);
        if is_close && press_t > 0.0 {
            bg = lerp_color(bg, Color32::BLACK, press_t * 0.25);
        }

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
        }

        let target_line = if is_close {
            Color32::WHITE
        } else {
            TEXT_PRIMARY.into()
        };
        let line_color = lerp_color(TEXT_MUTED, target_line, hover_t);

        icons::draw(ui.painter(), icon, rect.shrink(11.5), line_color, 1.4);

        resp
    }
}
