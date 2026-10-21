use crate::theme::{
    lerp_color, with_alpha, ACCENT, BG, BORDER_SUBTLE, CARD_HOVER, DANGER, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, Icon};
use egui::{pos2, vec2, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, ViewportCommand};

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

    /// Millida-style frameless title bar: 44 px tall with the brand centred
    /// and only the three window buttons on the right. Utility shortcuts
    /// (console, screenshots, downloads) live in the shell's own chrome.
    pub fn show_pixel(ui: &mut Ui, title: &str) -> TitleBarResponse {
        let response = TitleBarResponse::default();
        let height = 44.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), height));

        ui.painter().rect_filled(bar_rect, egui::Rounding::ZERO, BG);
        ui.painter().line_segment(
            [bar_rect.left_bottom(), bar_rect.right_bottom()],
            Stroke::new(1.0_f32, with_alpha(Color32::BLACK, 110)),
        );

        let button_w = 40.0;
        let button_h = 32.0;
        let gap = 2.0;
        let padding_right = 8.0;
        let controls_w = button_w * 3.0 + gap * 2.0 + padding_right;
        let drag_rect = Rect::from_min_size(
            bar_rect.min,
            vec2((bar_rect.width() - controls_w).max(0.0), height),
        );
        let drag_resp = ui.allocate_rect(drag_rect, Sense::click_and_drag());
        if drag_resp.dragged() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if drag_resp.double_clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // Centred brand: a rounded mark plus the wordmark.
        let name = title.to_uppercase();
        let galley =
            ui.painter()
                .layout_no_wrap(name, crate::theme::display(12.5), TEXT_HEADING.into());
        let total = 22.0 + 9.0 + galley.size().x;
        let left = bar_rect.center().x - total / 2.0;
        let mark = Rect::from_min_size(pos2(left, bar_rect.center().y - 11.0), vec2(22.0, 22.0));
        ui.painter()
            .rect_filled(mark, egui::Rounding::same(8.0), ACCENT);
        let diamond = Rect::from_center_size(mark.center(), vec2(9.0, 9.0));
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                diamond.center_top(),
                diamond.right_center(),
                diamond.center_bottom(),
                diamond.left_center(),
            ],
            Color32::from_white_alpha(235),
            Stroke::NONE,
        ));
        ui.painter().galley(
            pos2(left + 31.0, bar_rect.center().y - galley.size().y / 2.0),
            galley,
            TEXT_HEADING.into(),
        );

        let top = bar_rect.center().y - button_h / 2.0;
        let controls_left = bar_rect.right() - padding_right - button_w * 3.0 - gap * 2.0;

        let min_rect = Rect::from_min_size(pos2(controls_left, top), vec2(button_w, button_h));
        if Self::draw_window_button(ui, min_rect, Icon::Minimize, false).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        let max_icon = if is_maximized {
            Icon::Restore
        } else {
            Icon::Maximize
        };
        let max_rect = Rect::from_min_size(
            pos2(controls_left + button_w + gap, top),
            vec2(button_w, button_h),
        );
        if Self::draw_window_button(ui, max_rect, max_icon, false).clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        let close_rect = Rect::from_min_size(
            pos2(controls_left + (button_w + gap) * 2.0, top),
            vec2(button_w, button_h),
        );
        if Self::draw_window_button(ui, close_rect, Icon::Close, true).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        ui.advance_cursor_after_rect(bar_rect);
        response
    }

    fn draw_window_button(ui: &mut Ui, rect: Rect, icon: Icon, is_close: bool) -> Response {
        let resp = ui.allocate_rect(rect, Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if hover > 0.02 {
            let fill = if is_close {
                with_alpha(DANGER, (hover * 33.0) as u8)
            } else {
                with_alpha(CARD_HOVER, (hover * 255.0) as u8)
            };
            ui.painter()
                .rect_filled(rect, egui::Rounding::same(8.0), fill);
        }
        let color: Color32 = if is_close {
            lerp_color(TEXT_MUTED, DANGER, hover)
        } else {
            lerp_color(TEXT_MUTED, TEXT_HEADING, hover)
        };
        icons::draw(
            ui.painter(),
            icon,
            rect.shrink2(vec2(13.0, 9.0)),
            color,
            1.35,
        );
        resp
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
