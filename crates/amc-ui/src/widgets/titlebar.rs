use crate::theme::{
    lerp_color, with_alpha, BG, BORDER_SUBTLE, CARD_HOVER, DANGER, TEXT_HEADING, TEXT_MUTED,
    TEXT_PRIMARY,
};
use crate::widgets::brand;
use crate::widgets::icons::{self, Icon};
use egui::{
    pos2, vec2, Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, ViewportCommand,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct TitleBarResponse {
    pub console_clicked: bool,
    pub gallery_clicked: bool,
    pub downloads_clicked: bool,
    pub sidebar_toggle: bool,
    pub account_clicked: bool,
}

pub struct TitleBar;

impl TitleBar {
    pub fn show(ui: &mut Ui, title: &str, lang: amc_core::Language) -> TitleBarResponse {
        let mut response = TitleBarResponse::default();
        let height = 36.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), height));

        // Light shells (MultiMC/Legacy) need their chrome color here too;
        // otherwise the frameless top strip falls back to the black window
        // background and dark heading text becomes unreadable.
        ui.painter().rect_filled(bar_rect, egui::Rounding::ZERO, BG);

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

        // Draw the AMC hollow-square mark and title inside the drag area.
        let mark = Rect::from_center_size(
            Pos2::new(drag_rect.left() + 23.0, drag_rect.center().y),
            vec2(14.0, 14.0),
        );
        brand::hollow_square(ui.painter(), mark, brand::BRAND_RED);
        ui.painter().text(
            Pos2::new(mark.right() + 8.0, drag_rect.center().y),
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

    /// LabyMod-style title bar: a taller dark strip with the AMC identity on
    /// the left, a rounded account chip before the utility/window controls.
    pub fn show_labymod(
        ui: &mut Ui,
        title: &str,
        lang: amc_core::Language,
        account: Option<&str>,
        avatar: Option<&egui::TextureHandle>,
    ) -> TitleBarResponse {
        let mut response = TitleBarResponse::default();
        let height = 52.0;
        let rect = ui.available_rect_before_wrap();
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), height));

        ui.painter().rect_filled(bar_rect, egui::Rounding::ZERO, BG);
        ui.painter().line_segment(
            [bar_rect.left_bottom(), bar_rect.right_bottom()],
            Stroke::new(1.0_f32, BORDER_SUBTLE),
        );

        let button_w = 36.0;
        let controls_width = button_w * 6.0;
        let account_w = 210.0;
        let controls_left = bar_rect.right() - controls_width;
        let account_rect = Rect::from_min_size(
            Pos2::new(controls_left - account_w - 8.0, bar_rect.top() + 8.0),
            vec2(account_w, height - 16.0),
        );
        let drag_rect = Rect::from_min_size(
            bar_rect.min,
            vec2((account_rect.left() - bar_rect.left()).max(120.0), height),
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

        let mark = Rect::from_center_size(
            Pos2::new(bar_rect.left() + 24.0, bar_rect.center().y),
            vec2(18.0, 18.0),
        );
        brand::hollow_square(ui.painter(), mark, brand::BRAND_RED);
        ui.painter().text(
            Pos2::new(mark.right() + 10.0, bar_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::new(17.0, egui::FontFamily::Name("bold".into())),
            TEXT_HEADING.into(),
        );

        let account_resp = ui.allocate_rect(account_rect, Sense::click());
        let account_hover = ui
            .ctx()
            .animate_bool_responsive(account_resp.id, account_resp.hovered());
        ui.painter().rect_filled(
            account_rect,
            egui::Rounding::same(18.0),
            lerp_color(crate::theme::CARD, crate::theme::CARD_HOVER, account_hover),
        );
        ui.painter().rect_stroke(
            account_rect,
            egui::Rounding::same(18.0),
            Stroke::new(1.0_f32, crate::theme::BORDER),
        );
        let head = Rect::from_center_size(
            Pos2::new(account_rect.left() + 22.0, account_rect.center().y),
            vec2(28.0, 28.0),
        );
        ui.painter()
            .rect_filled(head, egui::Rounding::same(8.0), BG);
        if account.is_some() {
            if let Some(avatar) = avatar {
                ui.painter().image(
                    avatar.id(),
                    head.shrink(2.0),
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        } else {
            icons::draw(
                ui.painter(),
                Icon::Person,
                head.shrink(6.0),
                crate::theme::ACCENT_LIGHT,
                1.5,
            );
        }
        ui.painter().text(
            Pos2::new(account_rect.left() + 42.0, account_rect.center().y - 7.0),
            egui::Align2::LEFT_CENTER,
            account.unwrap_or_else(|| lang.modrinth_sign_in()),
            egui::FontId::proportional(12.5),
            TEXT_PRIMARY.into(),
        );
        ui.painter().text(
            Pos2::new(account_rect.left() + 42.0, account_rect.center().y + 8.0),
            egui::Align2::LEFT_CENTER,
            brand::APP_NAME,
            egui::FontId::proportional(9.0),
            TEXT_MUTED.into(),
        );
        icons::draw(
            ui.painter(),
            Icon::ChevronRight,
            Rect::from_center_size(
                Pos2::new(account_rect.right() - 16.0, account_rect.center().y),
                vec2(13.0, 13.0),
            ),
            TEXT_MUTED,
            1.4,
        );
        if account_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if account_resp.clicked() {
            response.account_clicked = true;
        }

        let gal_rect = Rect::from_min_size(
            Pos2::new(controls_left, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, gal_rect, Icon::Gallery, false).clicked() {
            response.gallery_clicked = true;
        }
        let con_rect = Rect::from_min_size(
            Pos2::new(controls_left + button_w, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, con_rect, Icon::Console, false).clicked() {
            response.console_clicked = true;
        }
        let dl_rect = Rect::from_min_size(
            Pos2::new(controls_left + button_w * 2.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, dl_rect, Icon::Download, false).clicked() {
            response.downloads_clicked = true;
        }
        let min_rect = Rect::from_min_size(
            Pos2::new(controls_left + button_w * 3.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, min_rect, Icon::Minimize, false).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        let max_icon = if is_maximized {
            Icon::Restore
        } else {
            Icon::Maximize
        };
        let max_rect = Rect::from_min_size(
            Pos2::new(controls_left + button_w * 4.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, max_rect, max_icon, false).clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }
        let close_rect = Rect::from_min_size(
            Pos2::new(controls_left + button_w * 5.0, bar_rect.top()),
            vec2(button_w, height),
        );
        if Self::draw_control_button(ui, close_rect, Icon::Close, true).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        ui.advance_cursor_after_rect(bar_rect);
        response
    }

    /// Modrinth App-style frameless title bar (reference: 48 px raised
    /// strip with a wordmark, history arrows, breadcrumb, running pill,
    /// quiet action icons, sidebar toggle and window controls).
    pub fn show_modrinth(
        ui: &mut Ui,
        lang: amc_core::Language,
        breadcrumb: &str,
        sidebar_open: bool,
        launching: bool,
    ) -> TitleBarResponse {
        let mut response = TitleBarResponse::default();
        let height = 48.0;
        let rect = ui.available_rect_before_wrap();
        let bar = Rect::from_min_size(rect.min, vec2(rect.width(), height));
        let painter = ui.painter().clone();

        painter.rect_filled(bar, Rounding::ZERO, crate::theme::MODRINTH_SURFACE);
        painter.line_segment(
            [bar.left_bottom(), bar.right_bottom()],
            Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
        );

        // AMC identity: hollow square + product name. Shell layout mirrors
        // the reference, but the launcher never wears another product's logo.
        let mark = Rect::from_min_size(
            pos2(bar.left() + 14.0, bar.center().y - 12.0),
            vec2(24.0, 24.0),
        );
        brand::hollow_square(&painter, mark, brand::BRAND_RED);
        let brand = painter.layout_no_wrap(
            brand::APP_NAME.to_owned(),
            egui::FontId::new(14.0, egui::FontFamily::Name("bold".into())),
            crate::theme::MODRINTH_TEXT_CONTRAST,
        );
        let brand_x = mark.right() + 9.0;
        painter.galley(
            pos2(brand_x, bar.center().y - brand.size().y / 2.0),
            brand.clone(),
            crate::theme::MODRINTH_TEXT_CONTRAST,
        );
        let brand_right = brand_x + brand.size().x;

        // Disabled browser-style history arrows (the app has no route stack).
        let mut x = brand_right + 14.0;
        for (idx, icon) in [Icon::ChevronLeft, Icon::ChevronRight]
            .into_iter()
            .enumerate()
        {
            let arrow = Rect::from_min_size(pos2(x, bar.center().y - 14.0), vec2(28.0, 28.0));
            let resp = ui.interact(
                arrow,
                ui.id().with(("modrinth_title_history", idx)),
                Sense::hover(),
            );
            painter.rect_stroke(
                arrow,
                Rounding::same(8.0),
                Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
            );
            icons::draw(
                &painter,
                icon,
                arrow.shrink(7.0),
                with_alpha(crate::theme::MODRINTH_MUTED, 90),
                1.35,
            );
            if idx == 0 {
                resp.on_hover_text(lang.modrinth_title_back());
            } else {
                resp.on_hover_text(lang.modrinth_title_forward());
            }
            x += 32.0;
        }

        let crumb = painter.layout_no_wrap(
            breadcrumb.to_owned(),
            egui::FontId::proportional(13.0),
            crate::theme::MODRINTH_TEXT,
        );
        let crumb_x = x + 8.0;
        painter.galley(
            pos2(crumb_x, bar.center().y - crumb.size().y / 2.0),
            crumb.clone(),
            crate::theme::MODRINTH_TEXT,
        );

        // Window controls, pinned to the right edge.
        let win_w = 36.0;
        let win_h = 32.0;
        let win_gap = 2.0;
        let edge = 6.0;
        let close_rect = Rect::from_min_size(
            pos2(bar.right() - edge - win_w, bar.center().y - win_h / 2.0),
            vec2(win_w, win_h),
        );
        let max_rect = close_rect.translate(vec2(-(win_w + win_gap), 0.0));
        let min_rect = max_rect.translate(vec2(-(win_w + win_gap), 0.0));

        // Right-sidebar toggle.
        let toggle = Rect::from_min_size(
            pos2(min_rect.left() - 10.0 - 32.0, bar.center().y - 16.0),
            vec2(32.0, 32.0),
        );

        // Quiet actions: download, console, gallery (matching the reference's
        // top-right icon cluster).
        let util_w = 32.0;
        let util_gap = 4.0;
        let util_left = toggle.left() - 10.0 - (util_w * 3.0 + util_gap * 2.0);
        let gallery =
            Rect::from_min_size(pos2(util_left, bar.center().y - 16.0), vec2(util_w, util_w));
        let console = gallery.translate(vec2(util_w + util_gap, 0.0));
        let downloads = console.translate(vec2(util_w + util_gap, 0.0));

        // Running-instance status pill.
        let status = if launching {
            lang.bottom_btn_launching()
        } else {
            lang.modrinth_no_running()
        };
        let status_galley = painter.layout_no_wrap(
            status.to_owned(),
            egui::FontId::proportional(11.5),
            crate::theme::MODRINTH_TEXT_SECONDARY,
        );
        let pill_w = (status_galley.size().x + 30.0).min(190.0);
        let pill = Rect::from_min_size(
            pos2(util_left - 10.0 - pill_w, bar.center().y - 15.0),
            vec2(pill_w, 30.0),
        );
        if pill.left() > crumb_x + crumb.size().x + 16.0 {
            painter.rect_filled(
                pill,
                Rounding::same(10.0),
                crate::theme::MODRINTH_SURFACE_HOVER,
            );
            painter.rect_stroke(
                pill,
                Rounding::same(10.0),
                Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
            );
            let dot = if launching {
                crate::theme::MODRINTH_GREEN
            } else {
                crate::theme::MODRINTH_MUTED
            };
            painter.circle_filled(pos2(pill.left() + 12.0, pill.center().y), 3.0, dot);
            painter.galley(
                pos2(
                    pill.left() + 22.0,
                    pill.center().y - status_galley.size().y / 2.0,
                ),
                status_galley.clone(),
                crate::theme::MODRINTH_TEXT_SECONDARY,
            );
        }

        // Empty stretch acts as the drag region (buttons stay clickable).
        let drag_left = crumb_x + crumb.size().x + 12.0;
        if drag_left < pill.left() - 12.0 {
            let drag_rect = Rect::from_min_max(
                pos2(drag_left, bar.top()),
                pos2(pill.left() - 12.0, bar.bottom()),
            );
            let drag = ui.interact(
                drag_rect,
                ui.id().with("modrinth_titlebar_drag"),
                Sense::click_and_drag(),
            );
            if drag.dragged() {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
            }
        }

        let toggle_resp = Self::draw_modrinth_quiet_button(
            ui,
            toggle,
            if sidebar_open {
                Icon::ChevronRight
            } else {
                Icon::ChevronLeft
            },
            "modrinth_sidebar_toggle",
            sidebar_open,
        );
        if toggle_resp
            .on_hover_text(lang.modrinth_toggle_sidebar())
            .clicked()
        {
            response.sidebar_toggle = true;
        }

        let gallery_resp = Self::draw_modrinth_quiet_button(
            ui,
            gallery,
            Icon::Gallery,
            "modrinth_title_gallery",
            false,
        );
        if gallery_resp.on_hover_text(lang.screens_title()).clicked() {
            response.gallery_clicked = true;
        }
        let console_resp = Self::draw_modrinth_quiet_button(
            ui,
            console,
            Icon::Console,
            "modrinth_title_console",
            false,
        );
        if console_resp
            .on_hover_text(lang.titlebar_console_tooltip())
            .clicked()
        {
            response.console_clicked = true;
        }
        let downloads_resp = Self::draw_modrinth_quiet_button(
            ui,
            downloads,
            Icon::Download,
            "modrinth_title_downloads",
            false,
        );
        if downloads_resp.on_hover_text(lang.dlman_title()).clicked() {
            response.downloads_clicked = true;
        }

        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if Self::draw_modrinth_window_button(ui, min_rect, Icon::Minimize, false, "min").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if Self::draw_modrinth_window_button(
            ui,
            max_rect,
            if is_maximized {
                Icon::Restore
            } else {
                Icon::Maximize
            },
            false,
            "max",
        )
        .clicked()
        {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }
        if Self::draw_modrinth_window_button(ui, close_rect, Icon::Close, true, "close").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }

        ui.advance_cursor_after_rect(bar);
        response
    }

    fn draw_modrinth_quiet_button(
        ui: &mut Ui,
        rect: Rect,
        icon: Icon,
        id: &str,
        active: bool,
    ) -> Response {
        let resp = ui.interact(rect, ui.id().with(id), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active {
            ui.painter().rect_filled(
                rect,
                Rounding::same(8.0),
                crate::theme::MODRINTH_SURFACE_HIGH,
            );
        } else if hover > 0.02 {
            ui.painter().rect_filled(
                rect,
                Rounding::same(8.0),
                crate::theme::lerp_color(
                    Color32::TRANSPARENT,
                    crate::theme::MODRINTH_SURFACE_HIGH,
                    hover,
                ),
            );
        }
        let color = if active {
            crate::theme::MODRINTH_TEXT_CONTRAST
        } else {
            crate::theme::lerp_color(
                crate::theme::MODRINTH_TEXT,
                crate::theme::MODRINTH_TEXT_CONTRAST,
                hover,
            )
        };
        icons::draw(ui.painter(), icon, rect.shrink(9.0), color, 1.35);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        resp
    }

    fn draw_modrinth_window_button(
        ui: &mut Ui,
        rect: Rect,
        icon: Icon,
        is_close: bool,
        id: &str,
    ) -> Response {
        let resp = ui.interact(rect, ui.id().with(("modrinth_window", id)), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if hover > 0.02 {
            let fill = if is_close {
                with_alpha(DANGER, (hover * 210.0) as u8)
            } else {
                crate::theme::lerp_color(
                    Color32::TRANSPARENT,
                    crate::theme::MODRINTH_SURFACE_HIGH,
                    hover,
                )
            };
            ui.painter().rect_filled(rect, Rounding::same(8.0), fill);
        }
        let color = if is_close {
            crate::theme::lerp_color(crate::theme::MODRINTH_TEXT, Color32::WHITE, hover)
        } else {
            crate::theme::lerp_color(
                crate::theme::MODRINTH_TEXT,
                crate::theme::MODRINTH_TEXT_CONTRAST,
                hover,
            )
        };
        icons::draw(
            ui.painter(),
            icon,
            rect.shrink2(vec2(11.0, 9.0)),
            color,
            1.35,
        );
        resp
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
        brand::hollow_square(ui.painter(), mark, brand::BRAND_RED);
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
