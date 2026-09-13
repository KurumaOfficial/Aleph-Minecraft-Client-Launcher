use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui};
use crate::theme::{
    lerp_color, BG_ELEVATED, BORDER_DEFAULT, BORDER_SUBTLE, RUBY, RUBY_DIM, RUBY_LIGHT,
    TEXT_DIM, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavTab {
    #[default]
    Home,
    Modpacks,
    Mods,
    Skins,
    Settings,
}

pub struct Sidebar;

impl Sidebar {
    pub fn show(ui: &mut Ui, current_tab: &mut NavTab, lang: amc_core::Language) -> bool {
        let mut exit_clicked = false;
        let rect = ui.available_rect_before_wrap();

        // Background and right 1px border
        ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_DEFAULT),
        );

        ui.vertical(|ui| {
            // Logo Header (Aleph Studio Square Motif)
            Self::draw_logo(ui);

            // Navigation Items
            ui.add_space(8.0);
            if Self::nav_item(ui, lang.nav_home(), "⌂", *current_tab == NavTab::Home) {
                *current_tab = NavTab::Home;
            }
            if Self::nav_item(ui, lang.nav_instances(), "⊞", *current_tab == NavTab::Modpacks) {
                *current_tab = NavTab::Modpacks;
            }
            if Self::nav_item(ui, lang.nav_mods(), "◈", *current_tab == NavTab::Mods) {
                *current_tab = NavTab::Mods;
            }
            if Self::nav_item(ui, lang.nav_skins(), "👤", *current_tab == NavTab::Skins) {
                *current_tab = NavTab::Skins;
            }

            // Push bottom items down
            let bottom_items_height = 96.0;
            let space_left = ui.available_height() - bottom_items_height;
            if space_left > 0.0 {
                ui.add_space(space_left);
            }

            // Bottom border separator
            let sep_y = ui.cursor().top();
            ui.painter().line_segment(
                [Pos2::new(rect.left(), sep_y), Pos2::new(rect.right(), sep_y)],
                Stroke::new(1.0, BORDER_SUBTLE),
            );
            ui.add_space(8.0);

            if Self::nav_item(ui, lang.nav_settings(), "⚙", *current_tab == NavTab::Settings) {
                *current_tab = NavTab::Settings;
            }

            if Self::nav_item(ui, lang.nav_exit(), "⏻", false) {
                exit_clicked = true;
            }
        });

        exit_clicked
    }

    fn draw_logo(ui: &mut Ui) {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::hover());

        // Subtle bottom border
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_SUBTLE),
        );

        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());

        // Aleph Studio Square Motif icon (26x26)
        let sq_size = 26.0;
        let sq_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 16.0, rect.center().y - sq_size / 2.0),
            vec2(sq_size, sq_size),
        );

        let sq_bg = lerp_color(
            Color32::from_rgba_premultiplied(139, 26, 42, 16),
            Color32::from_rgba_premultiplied(181, 34, 57, 45),
            hover_t,
        );
        let sq_stroke = lerp_color(RUBY, RUBY_LIGHT, hover_t);

        ui.painter().rect_filled(sq_rect, Rounding::ZERO, sq_bg);
        ui.painter().rect_stroke(sq_rect, Rounding::ZERO, Stroke::new(1.5, sq_stroke));

        // Central 'ℵ' glyph
        ui.painter().text(
            sq_rect.center(),
            egui::Align2::CENTER_CENTER,
            "ℵ",
            egui::FontId::proportional(15.0),
            lerp_color(RUBY_LIGHT, Color32::WHITE, hover_t),
        );

        // Title and Studio subtitle
        let title_col = lerp_color(TEXT_HEADING, Color32::WHITE, hover_t);
        ui.painter().text(
            Pos2::new(sq_rect.right() + 10.0, rect.center().y - 7.0),
            egui::Align2::LEFT_CENTER,
            "Aleph Launcher",
            egui::FontId::proportional(13.5),
            title_col,
        );

        ui.painter().text(
            Pos2::new(sq_rect.right() + 10.0, rect.center().y + 8.0),
            egui::Align2::LEFT_CENTER,
            "ALEPH.ICU STUDIO",
            egui::FontId::monospace(9.0),
            TEXT_DIM,
        );
    }

    fn nav_item(ui: &mut Ui, label: &str, icon: &str, is_active: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());

        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active_t = ui.ctx().animate_bool(resp.id.with("active"), is_active);

        // Smooth background interpolation
        let bg_alpha = (active_t * 42.0 + hover_t * 22.0 * (1.0 - active_t)).round() as u8;
        if bg_alpha > 0 {
            let bg_color = Color32::from_rgba_premultiplied(139, 26, 42, bg_alpha);
            ui.painter().rect_filled(rect, Rounding::ZERO, bg_color);
        }

        // Animated Left Accent Indicator Bar (Square motif)
        let bar_w = 3.0 * active_t + 1.5 * hover_t * (1.0 - active_t);
        if bar_w > 0.1 {
            let bar_color = lerp_color(RUBY_DIM, RUBY_LIGHT, active_t.max(hover_t));
            let bar_rect = Rect::from_min_max(rect.left_top(), Pos2::new(rect.left() + bar_w, rect.bottom()));
            ui.painter().rect_filled(bar_rect, Rounding::ZERO, bar_color);
        }

        // Smooth color transition for text & icon
        let normal_text = lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t);
        let text_color = lerp_color(normal_text, RUBY_LIGHT, active_t);

        // Icon
        ui.painter().text(
            Pos2::new(rect.left() + 16.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(14.0),
            text_color,
        );

        // Label
        ui.painter().text(
            Pos2::new(rect.left() + 40.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );

        // Active indicator square on the right (square motif from aleph.icu)
        if active_t > 0.05 {
            let sq_dot_size = 4.0;
            let sq_dot_rect = Rect::from_min_size(
                Pos2::new(rect.right() - 14.0, rect.center().y - sq_dot_size / 2.0),
                vec2(sq_dot_size, sq_dot_size),
            );
            let dot_col = Color32::from_rgba_premultiplied(
                RUBY_LIGHT.r(),
                RUBY_LIGHT.g(),
                RUBY_LIGHT.b(),
                (active_t * 220.0) as u8,
            );
            ui.painter().rect_filled(sq_dot_rect, Rounding::ZERO, dot_col);
        }

        resp.clicked()
    }
}
