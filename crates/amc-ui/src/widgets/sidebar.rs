use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui};
use crate::theme::{
    BG_ELEVATED, BORDER_DEFAULT, BORDER_SUBTLE, RUBY, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED,
    TEXT_PRIMARY,
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

        // Background and right border
        ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_DEFAULT),
        );

        ui.vertical(|ui| {
            // Logo Header
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
            let bottom_items_height = 100.0;
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
            ui.add_space(10.0);

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
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 58.0), Sense::hover());

        // Bottom separator
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_SUBTLE),
        );

        // Square ruby outline (from Aleph Studio mockup)
        let sq_size = 24.0;
        let sq_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 16.0, rect.center().y - sq_size / 2.0),
            vec2(sq_size, sq_size),
        );
        ui.painter().rect_stroke(sq_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));
        ui.painter().text(
            sq_rect.center(),
            egui::Align2::CENTER_CENTER,
            "ℵ",
            egui::FontId::proportional(15.0),
            RUBY_LIGHT,
        );

        // Title text
        ui.painter().text(
            Pos2::new(sq_rect.right() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Aleph Launcher",
            egui::FontId::proportional(14.0),
            TEXT_HEADING,
        );
    }

    fn nav_item(ui: &mut Ui, label: &str, icon: &str, is_active: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());

        let is_hovered = resp.hovered();

        // Background
        let bg_color = if is_active {
            Color32::from_rgba_premultiplied(139, 26, 42, 35)
        } else if is_hovered {
            Color32::from_rgba_premultiplied(139, 26, 42, 14)
        } else {
            Color32::TRANSPARENT
        };

        if bg_color != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, Rounding::ZERO, bg_color);
        }

        // Active left accent bar (3px ruby)
        if is_active {
            let bar_rect = Rect::from_min_max(rect.left_top(), Pos2::new(rect.left() + 3.0, rect.bottom()));
            ui.painter().rect_filled(bar_rect, Rounding::ZERO, RUBY);
        }

        // Text color
        let text_color = if is_active {
            RUBY_LIGHT
        } else if is_hovered {
            TEXT_PRIMARY
        } else {
            TEXT_MUTED
        };

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

        resp.clicked()
    }
}
