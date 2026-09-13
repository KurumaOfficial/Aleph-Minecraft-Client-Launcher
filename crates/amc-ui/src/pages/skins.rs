use egui::{vec2, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui};
use amc_auth::Account;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};

pub struct SkinsPage {
    pub is_slim_model: bool,
    pub active_skin_name: String,
}

impl Default for SkinsPage {
    fn default() -> Self {
        Self {
            is_slim_model: false,
            active_skin_name: "Steve (По умолчанию)".to_string(),
        }
    }
}

impl SkinsPage {
    pub fn show(&mut self, ui: &mut Ui, account: Option<&Account>) {
        ui.add_space(20.0);

        ui.label(
            egui::RichText::new("Управление скинами")
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Skin preview box (from HTML mockup)
        let preview_width = 460.0;
        let preview_height = 360.0;
        let (rect, _) = ui.allocate_exact_size(vec2(preview_width, preview_height), Sense::hover());

        ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, BORDER_DEFAULT));

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        child.add_space(30.0);

        // Model preview placeholder
        let model_rect = Rect::from_center_size(
            Pos2::new(rect.center().x, rect.top() + 140.0),
            vec2(140.0, 200.0),
        );
        child.painter().rect_filled(model_rect, Rounding::ZERO, BG_HOVER);
        child.painter().rect_stroke(model_rect, Rounding::ZERO, Stroke::new(1.0, BORDER_STRONG));

        child.painter().text(
            model_rect.center(),
            egui::Align2::CENTER_CENTER,
            "👤\n3D МОДЕЛЬ",
            egui::FontId::proportional(16.0),
            RUBY_LIGHT,
        );

        child.add_space(180.0);

        child.label(
            egui::RichText::new(&self.active_skin_name)
                .font(egui::FontId::proportional(14.0))
                .strong()
                .color(TEXT_HEADING),
        );

        child.add_space(10.0);

        // Action buttons
        child.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

            let btn_upload = egui::Button::new(
                egui::RichText::new("ВЫБРАТЬ ФАЙЛ (.PNG)")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(RUBY)
            .stroke(Stroke::new(1.0, RUBY_LIGHT))
            .min_size(vec2(160.0, 34.0));

            if ui.add(btn_upload).clicked() {
                // Future file dialog integration
            }

            let btn_reset = egui::Button::new(
                egui::RichText::new("СБРОСИТЬ СКИН")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(TEXT_PRIMARY),
            )
            .fill(BG_HOVER)
            .stroke(Stroke::new(1.0, BORDER_DEFAULT))
            .min_size(vec2(130.0, 34.0));

            if ui.add(btn_reset).clicked() {
                self.active_skin_name = "Steve (По умолчанию)".to_string();
            }
        });

        ui.add_space(20.0);

        // Account status info
        if let Some(acc) = account {
            ui.label(
                egui::RichText::new(format!("Аккаунт: {} ({})", acc.username, acc.account_type.as_str()))
                    .font(egui::FontId::proportional(13.0))
                    .color(TEXT_MUTED),
            );
        }
    }
}
