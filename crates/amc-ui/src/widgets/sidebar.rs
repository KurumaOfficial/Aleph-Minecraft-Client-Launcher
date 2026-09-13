use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, UiBuilder};
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
    pub fn show(ui: &mut Ui, current_tab: &mut NavTab) -> bool {
        let mut exit_clicked = false;
        let width = 175.0;
        let height = ui.available_height();

        let (panel_rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        // Background and right border
        ui.painter().rect_filled(panel_rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().line_segment(
            [panel_rect.right_top(), panel_rect.right_bottom()],
            Stroke::new(1.0, BORDER_DEFAULT),
        );

        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(panel_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );

        // Logo Header
        Self::draw_logo(&mut child_ui);

        // Navigation Items
        child_ui.add_space(10.0);
        if Self::nav_item(&mut child_ui, "Главная", "⌂", *current_tab == NavTab::Home) {
            *current_tab = NavTab::Home;
        }
        if Self::nav_item(&mut child_ui, "Сборки", "⊞", *current_tab == NavTab::Modpacks) {
            *current_tab = NavTab::Modpacks;
        }
        if Self::nav_item(&mut child_ui, "Моды", "◈", *current_tab == NavTab::Mods) {
            *current_tab = NavTab::Mods;
        }
        if Self::nav_item(&mut child_ui, "Скины", "👤", *current_tab == NavTab::Skins) {
            *current_tab = NavTab::Skins;
        }

        // Spacer to push settings & exit to bottom
        let bottom_space = 100.0;
        let available = child_ui.available_height() - bottom_space;
        if available > 0.0 {
            child_ui.add_space(available);
        }

        // Bottom border separator
        let sep_y = child_ui.cursor().top();
        child_ui.painter().line_segment(
            [Pos2::new(panel_rect.left(), sep_y), Pos2::new(panel_rect.right(), sep_y)],
            Stroke::new(1.0, BORDER_SUBTLE),
        );
        child_ui.add_space(12.0);

        if Self::nav_item(&mut child_ui, "Настройки", "⚙", *current_tab == NavTab::Settings) {
            *current_tab = NavTab::Settings;
        }

        if Self::nav_item(&mut child_ui, "Выход", "⏻", false) {
            exit_clicked = true;
        }

        exit_clicked
    }

    fn draw_logo(ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());

        // Bottom separator
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, BORDER_SUBTLE),
        );

        // Square ruby outline (from Aleph Studio mockup)
        let sq_size = 26.0;
        let sq_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 16.0, rect.center().y - sq_size / 2.0),
            vec2(sq_size, sq_size),
        );
        ui.painter().rect_stroke(sq_rect, Rounding::ZERO, Stroke::new(2.0, RUBY));

        // Title text
        ui.painter().text(
            Pos2::new(sq_rect.right() + 12.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Aleph Launcher",
            egui::FontId::proportional(14.0),
            TEXT_HEADING,
        );
    }

    fn nav_item(ui: &mut Ui, label: &str, icon: &str, is_active: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());

        let is_hovered = resp.hovered();

        // Background
        let bg_color = if is_active {
            Color32::from_rgba_premultiplied(139, 26, 42, 32)
        } else if is_hovered {
            Color32::from_rgba_premultiplied(139, 26, 42, 12)
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
            Pos2::new(rect.left() + 42.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );

        resp.clicked()
    }
}
