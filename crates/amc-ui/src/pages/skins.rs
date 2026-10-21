use crate::theme::{
    lerp_color, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_LIGHT,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, Icon};
use amc_auth::Account;
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, TextureOptions, Ui};
use image::RgbaImage;
use std::path::PathBuf;

const STEVE_SKIN: &[u8] = include_bytes!("../../../../assets/modrinth/skins/steve.png");

/// Visual skin for the shared skin page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinsSkin {
    #[default]
    Amc,
    Modrinth,
}

pub struct SkinsPage {
    pub is_slim_model: bool,
    pub active_skin_name: String,
    pub custom_skin_path: Option<PathBuf>,
    pub skin_image: Option<RgbaImage>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub dirty: bool,
    pub avatar_dirty: bool,
}

impl Default for SkinsPage {
    fn default() -> Self {
        Self {
            is_slim_model: false,
            active_skin_name: "Steve".to_string(),
            custom_skin_path: None,
            skin_image: None,
            texture_handle: None,
            dirty: true,
            avatar_dirty: true,
        }
    }
}

impl SkinsPage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        lang: amc_core::Language,
        skin: SkinsSkin,
    ) {
        if skin == SkinsSkin::Modrinth {
            self.show_modrinth(ui, account, lang);
            return;
        }
        ui.add_space(20.0);

        ui.label(
            egui::RichText::new(lang.skins_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Update preview texture if dirty
        self.ensure_preview_texture(ui.ctx());

        // Preview box
        let preview_width = 460.0;
        let preview_height = 420.0;
        let (rect, _) = ui.allocate_exact_size(vec2(preview_width, preview_height), Sense::hover());

        ui.painter()
            .rect_filled(rect, crate::theme::rounding(), BG_ELEVATED);
        ui.painter().rect_stroke(
            rect,
            crate::theme::rounding(),
            Stroke::new(1.0_f32, BORDER_DEFAULT),
        );

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        child.add_space(20.0);

        // Model type switch: Classic (4px) vs Slim (3px)
        child.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

            let classic_active = !self.is_slim_model;
            let slim_active = self.is_slim_model;

            let (cb_rect, cb_resp) = ui.allocate_exact_size(vec2(110.0, 28.0), Sense::click());
            let cb_hover = ui
                .ctx()
                .animate_bool_responsive(cb_resp.id, cb_resp.hovered());
            let cb_act = ui
                .ctx()
                .animate_bool(cb_resp.id.with("act"), classic_active);
            let cb_bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, cb_hover), RUBY, cb_act);
            let cb_stroke = lerp_color(
                lerp_color(BORDER_DEFAULT, RUBY, cb_hover),
                RUBY_LIGHT,
                cb_act,
            );
            let cb_text = lerp_color(
                lerp_color(TEXT_MUTED, TEXT_PRIMARY, cb_hover),
                Color32::WHITE,
                cb_act,
            );

            ui.painter()
                .rect_filled(cb_rect, crate::theme::rounding(), cb_bg);
            ui.painter().rect_stroke(
                cb_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, cb_stroke),
            );
            ui.painter().text(
                cb_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.skins_classic(),
                egui::FontId::proportional(11.0),
                cb_text,
            );

            if cb_resp.clicked() && self.is_slim_model {
                self.is_slim_model = false;
                self.dirty = true;
                self.avatar_dirty = true;
            }

            let (sb_rect, sb_resp) = ui.allocate_exact_size(vec2(110.0, 28.0), Sense::click());
            let sb_hover = ui
                .ctx()
                .animate_bool_responsive(sb_resp.id, sb_resp.hovered());
            let sb_act = ui.ctx().animate_bool(sb_resp.id.with("act"), slim_active);
            let sb_bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, sb_hover), RUBY, sb_act);
            let sb_stroke = lerp_color(
                lerp_color(BORDER_DEFAULT, RUBY, sb_hover),
                RUBY_LIGHT,
                sb_act,
            );
            let sb_text = lerp_color(
                lerp_color(TEXT_MUTED, TEXT_PRIMARY, sb_hover),
                Color32::WHITE,
                sb_act,
            );

            ui.painter()
                .rect_filled(sb_rect, crate::theme::rounding(), sb_bg);
            ui.painter().rect_stroke(
                sb_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, sb_stroke),
            );
            ui.painter().text(
                sb_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.skins_slim(),
                egui::FontId::proportional(11.0),
                sb_text,
            );

            if sb_resp.clicked() && !self.is_slim_model {
                self.is_slim_model = true;
                self.dirty = true;
                self.avatar_dirty = true;
            }
        });

        child.add_space(16.0);

        // 2D Character Display
        if let Some(tex) = &self.texture_handle {
            let (img_rect, _) = child.allocate_exact_size(vec2(130.0, 260.0), Sense::hover());
            child
                .painter()
                .rect_filled(img_rect, crate::theme::rounding(), BG_HOVER);
            child.painter().rect_stroke(
                img_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, BORDER_STRONG),
            );
            child.painter().image(
                tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        child.add_space(14.0);

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

            let (up_rect, up_resp) = ui.allocate_exact_size(vec2(160.0, 34.0), Sense::click());
            let up_hover = ui
                .ctx()
                .animate_bool_responsive(up_resp.id, up_resp.hovered());
            let up_bg = lerp_color(RUBY, RUBY_LIGHT, up_hover);
            let up_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, up_hover);

            ui.painter()
                .rect_filled(up_rect, crate::theme::rounding(), up_bg);
            ui.painter().rect_stroke(
                up_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, up_stroke),
            );
            ui.painter().text(
                up_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.skins_btn_load(),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );

            if up_resp.clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Minecraft Skins (*.png)", &["png"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        self.skin_image = Some(img.to_rgba8());
                        self.active_skin_name = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| lang.skins_custom_name().to_string());
                        self.custom_skin_path = Some(path);
                        self.dirty = true;
                        self.avatar_dirty = true;
                    }
                }
            }

            let reset_label = if self.is_slim_model {
                lang.skins_btn_reset_alex()
            } else {
                lang.skins_btn_reset_steve()
            };
            let (rs_rect, rs_resp) = ui.allocate_exact_size(vec2(130.0, 34.0), Sense::click());
            let rs_hover = ui
                .ctx()
                .animate_bool_responsive(rs_resp.id, rs_resp.hovered());
            let rs_bg = lerp_color(BG_HOVER, BG_CARD, rs_hover);
            let rs_stroke = lerp_color(BORDER_DEFAULT, RUBY, rs_hover);
            let rs_text = lerp_color(TEXT_PRIMARY, Color32::WHITE, rs_hover);

            ui.painter()
                .rect_filled(rs_rect, crate::theme::rounding(), rs_bg);
            ui.painter().rect_stroke(
                rs_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, rs_stroke),
            );
            ui.painter().text(
                rs_rect.center(),
                egui::Align2::CENTER_CENTER,
                reset_label,
                egui::FontId::proportional(11.0),
                rs_text,
            );

            if rs_resp.clicked() {
                self.skin_image = None;
                self.custom_skin_path = None;
                self.active_skin_name = if self.is_slim_model {
                    "Alex".to_string()
                } else {
                    "Steve".to_string()
                };
                self.dirty = true;
                self.avatar_dirty = true;
            }
        });

        ui.add_space(18.0);

        // Account status info
        if let Some(acc) = account {
            let acc_label = lang.skins_account_label();
            ui.label(
                egui::RichText::new(format!(
                    "{acc_label}: {} ({})",
                    acc.username,
                    acc.account_type.as_str()
                ))
                .font(egui::FontId::proportional(13.0))
                .color(TEXT_MUTED),
            );
        }
    }

    fn ensure_preview_texture(&mut self, ctx: &egui::Context) {
        if !self.dirty && self.texture_handle.is_some() {
            return;
        }
        let image = match &self.skin_image {
            Some(image) => composite_front_skin(image, self.is_slim_model),
            None => {
                let default = if self.is_slim_model {
                    generate_default_skin(true)
                } else {
                    embedded_steve().unwrap_or_else(|| generate_default_skin(false))
                };
                composite_front_skin(&default, self.is_slim_model)
            }
        };
        self.texture_handle =
            Some(ctx.load_texture("skin_preview", image, TextureOptions::NEAREST));
        self.dirty = false;
    }

    fn load_skin_dialog(&mut self, lang: amc_core::Language) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Minecraft Skins (*.png)", &["png"])
            .pick_file()
        {
            if let Ok(img) = image::open(&path) {
                self.skin_image = Some(img.to_rgba8());
                self.active_skin_name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| lang.skins_custom_name().to_string());
                self.custom_skin_path = Some(path);
                self.dirty = true;
                self.avatar_dirty = true;
            }
        }
    }

    /// Modrinth reference composition: a sticky preview column and a skin
    /// library rail with the active skin, default heads and an add tile.
    fn show_modrinth(&mut self, ui: &mut Ui, account: Option<&Account>, lang: amc_core::Language) {
        self.ensure_preview_texture(ui.ctx());

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(lang.skins_selector_title())
                .font(egui::FontId::proportional(24.0))
                .strong()
                .color(TEXT_HEADING),
        );
        ui.add_space(12.0);

        let available = ui.available_width();
        let gap = 16.0;
        let left_w = (available * 0.46).clamp(270.0, 360.0);
        let right_w = (available - left_w - gap).max(220.0);
        let panel_h = (ui.available_height() - 4.0).max(420.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(left_w, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_modrinth_preview(ui, account, lang, left_w, panel_h),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(right_w, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.show_modrinth_skin_list(ui, lang, right_w));
                },
            );
        });
    }

    fn show_modrinth_preview(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        lang: amc_core::Language,
        width: f32,
        panel_h: f32,
    ) {
        let preview_h = (panel_h - 136.0).max(250.0);
        let (rect, _) = ui.allocate_exact_size(vec2(width, preview_h), Sense::hover());
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(20.0),
            crate::theme::MODRINTH_SURFACE,
        );
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(20.0),
            Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
        );
        ui.painter().circle_filled(
            pos2(rect.center().x, rect.center().y + 10.0),
            preview_h * 0.34,
            crate::theme::with_alpha(crate::theme::MODRINTH_GREEN, 12),
        );

        if let Some(tex) = &self.texture_handle {
            let image_h = (preview_h - 54.0).min(400.0);
            let image_w = image_h / 2.0;
            let image_rect = Rect::from_center_size(
                pos2(rect.center().x, rect.center().y + 8.0),
                vec2(image_w, image_h),
            );
            ui.painter().image(
                tex.id(),
                image_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let chip = Rect::from_min_size(
            pos2(rect.left() + 14.0, rect.top() + 14.0),
            vec2(92.0, 24.0),
        );
        ui.painter().rect_filled(
            chip,
            egui::Rounding::same(12.0),
            crate::theme::with_alpha(crate::theme::MODRINTH_BG, 170),
        );
        ui.painter().text(
            chip.center(),
            egui::Align2::CENTER_CENTER,
            if self.is_slim_model {
                lang.skins_model_slim()
            } else {
                lang.skins_model_classic()
            },
            egui::FontId::proportional(11.0),
            crate::theme::MODRINTH_TEXT,
        );

        ui.add_space(10.0);
        if let Some(new_slim) = modrinth_model_switch(ui, self.is_slim_model, lang, width) {
            self.is_slim_model = new_slim;
            self.dirty = true;
            self.avatar_dirty = true;
        }

        ui.add_space(8.0);
        let button_w = ((width - 8.0) / 2.0).max(110.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            if modrinth_skin_button(ui, Icon::Folder, lang.skins_load_skin(), true, button_w) {
                self.load_skin_dialog(lang);
            }
            let reset_label = if self.is_slim_model {
                lang.skins_btn_reset_alex()
            } else {
                lang.skins_btn_reset_steve()
            };
            if modrinth_skin_button(ui, Icon::Refresh, reset_label, false, button_w) {
                self.skin_image = None;
                self.custom_skin_path = None;
                self.active_skin_name = if self.is_slim_model {
                    "Alex".to_string()
                } else {
                    "Steve".to_string()
                };
                self.dirty = true;
                self.avatar_dirty = true;
            }
        });

        ui.add_space(8.0);
        let status = match account {
            Some(acc) => format!(
                "{}: {} ({})",
                lang.skins_account_label(),
                acc.username,
                acc.account_type.as_str()
            ),
            None => lang.skins_demo_notice().to_string(),
        };
        ui.label(
            egui::RichText::new(status)
                .font(egui::FontId::proportional(11.5))
                .color(crate::theme::MODRINTH_TEXT_SECONDARY),
        );
    }

    fn show_modrinth_skin_list(&mut self, ui: &mut Ui, lang: amc_core::Language, width: f32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.nav_skins())
                    .font(egui::FontId::proportional(16.0))
                    .strong()
                    .color(crate::theme::MODRINTH_TEXT_CONTRAST),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if modrinth_skin_button(
                    ui,
                    Icon::Plus,
                    lang.skins_add(),
                    true,
                    (width * 0.42).clamp(104.0, 150.0),
                ) {
                    self.load_skin_dialog(lang);
                }
            });
        });
        ui.add_space(10.0);

        // Active skin.
        let (card, _) = ui.allocate_exact_size(vec2(width, 72.0), Sense::hover());
        ui.painter().rect_filled(
            card,
            egui::Rounding::same(12.0),
            crate::theme::MODRINTH_SURFACE,
        );
        ui.painter().rect_stroke(
            card,
            egui::Rounding::same(12.0),
            Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
        );
        let head_rect = Rect::from_min_size(
            pos2(card.left() + 12.0, card.top() + 12.0),
            vec2(48.0, 48.0),
        );
        ui.painter().rect_filled(
            head_rect,
            egui::Rounding::same(8.0),
            crate::theme::MODRINTH_SURFACE_HOVER,
        );
        if let Some(tex) = &self.texture_handle {
            ui.painter().image(
                tex.id(),
                head_rect.shrink(5.0),
                Rect::from_min_max(pos2(0.25, 0.0), pos2(0.75, 0.25)),
                Color32::WHITE,
            );
        }
        ui.painter().text(
            pos2(card.left() + 72.0, card.center().y - 9.0),
            egui::Align2::LEFT_CENTER,
            truncate_skin_name(&self.active_skin_name, 28),
            egui::FontId::proportional(13.5),
            crate::theme::MODRINTH_TEXT_CONTRAST,
        );
        ui.painter().text(
            pos2(card.left() + 72.0, card.center().y + 11.0),
            egui::Align2::LEFT_CENTER,
            if self.is_slim_model {
                lang.skins_model_slim()
            } else {
                lang.skins_model_classic()
            },
            egui::FontId::proportional(11.0),
            crate::theme::MODRINTH_TEXT_SECONDARY,
        );
        let badge = Rect::from_min_size(
            pos2(card.right() - 78.0, card.center().y - 12.0),
            vec2(66.0, 24.0),
        );
        ui.painter().rect_filled(
            badge,
            egui::Rounding::same(12.0),
            crate::theme::with_alpha(crate::theme::MODRINTH_GREEN, 24),
        );
        ui.painter().text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            lang.skins_active_badge(),
            egui::FontId::proportional(10.5),
            crate::theme::MODRINTH_GREEN,
        );

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(lang.skins_defaults())
                .font(egui::FontId::proportional(12.5))
                .strong()
                .color(crate::theme::MODRINTH_TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        let tile_gap = 8.0;
        let tile_w = ((width - tile_gap) / 2.0).max(90.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(tile_gap, 0.0);
            for (slim, label, tex) in [
                (false, "Steve", default_head_texture(ui.ctx(), false)),
                (true, "Alex", default_head_texture(ui.ctx(), true)),
            ] {
                let (tile, resp) = ui.allocate_exact_size(vec2(tile_w, 92.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                let active = self.skin_image.is_none() && self.is_slim_model == slim;
                ui.painter().rect_filled(
                    tile,
                    egui::Rounding::same(12.0),
                    crate::theme::lerp_color(
                        crate::theme::MODRINTH_SURFACE,
                        crate::theme::MODRINTH_SURFACE_HOVER,
                        hover,
                    ),
                );
                ui.painter().rect_stroke(
                    tile,
                    egui::Rounding::same(12.0),
                    Stroke::new(
                        1.0_f32,
                        if active {
                            crate::theme::MODRINTH_GREEN
                        } else {
                            crate::theme::MODRINTH_SURFACE_HIGH
                        },
                    ),
                );
                let head = Rect::from_center_size(
                    pos2(tile.center().x, tile.top() + 34.0),
                    vec2(44.0, 44.0),
                );
                ui.painter().image(
                    tex.id(),
                    head,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().text(
                    pos2(tile.center().x, tile.bottom() - 17.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    crate::theme::MODRINTH_TEXT_CONTRAST,
                );
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    self.is_slim_model = slim;
                    self.skin_image = None;
                    self.custom_skin_path = None;
                    self.active_skin_name = label.to_string();
                    self.dirty = true;
                    self.avatar_dirty = true;
                }
            }
        });

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(lang.skins_saved_title())
                .font(egui::FontId::proportional(12.5))
                .strong()
                .color(crate::theme::MODRINTH_TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        let (empty, resp) = ui.allocate_exact_size(vec2(width, 62.0), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        ui.painter().rect_filled(
            empty,
            egui::Rounding::same(12.0),
            crate::theme::lerp_color(Color32::TRANSPARENT, crate::theme::MODRINTH_SURFACE, hover),
        );
        ui.painter().rect_stroke(
            empty,
            egui::Rounding::same(12.0),
            Stroke::new(1.0_f32, crate::theme::MODRINTH_SURFACE_HIGH),
        );
        icons::draw(
            ui.painter(),
            Icon::Plus,
            Rect::from_center_size(
                pos2(empty.left() + 24.0, empty.center().y),
                vec2(15.0, 15.0),
            ),
            crate::theme::MODRINTH_GREEN,
            1.5,
        );
        ui.painter().text(
            pos2(empty.left() + 42.0, empty.center().y),
            egui::Align2::LEFT_CENTER,
            lang.skins_no_saved(),
            egui::FontId::proportional(12.0),
            crate::theme::MODRINTH_TEXT_SECONDARY,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.load_skin_dialog(lang);
        }
    }
}

fn truncate_skin_name(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        name.to_string()
    } else {
        format!("{}…", name.chars().take(max).collect::<String>())
    }
}

fn embedded_steve() -> Option<RgbaImage> {
    image::load_from_memory(STEVE_SKIN)
        .ok()
        .map(|img| img.to_rgba8())
}

fn default_head_texture(ctx: &egui::Context, slim: bool) -> egui::TextureHandle {
    let id = egui::Id::new(("modrinth_default_head", slim));
    if let Some(tex) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return tex;
    }
    let image = extract_head_avatar(None, slim);
    let texture = ctx.load_texture(
        format!("modrinth-default-head-{slim}"),
        image,
        TextureOptions::NEAREST,
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

fn modrinth_model_switch(
    ui: &mut Ui,
    slim: bool,
    lang: amc_core::Language,
    width: f32,
) -> Option<bool> {
    let mut result = None;
    let (tray, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    ui.painter()
        .rect_filled(tray, egui::Rounding::same(18.0), crate::theme::MODRINTH_BG);
    let segment_w = (tray.width() - 8.0) / 2.0;
    for (idx, (label, active, value)) in [
        (lang.skins_model_classic(), !slim, false),
        (lang.skins_model_slim(), slim, true),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::from_min_size(
            pos2(tray.left() + 4.0 + idx as f32 * segment_w, tray.top() + 4.0),
            vec2(segment_w, 28.0),
        );
        let resp = ui.interact(rect, ui.id().with(("skin_model", idx)), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(14.0),
                crate::theme::MODRINTH_GREEN,
            );
        } else if hover > 0.02 {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(14.0),
                crate::theme::with_alpha(
                    crate::theme::MODRINTH_SURFACE_HIGH,
                    (hover * 255.0) as u8,
                ),
            );
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            if active {
                crate::theme::MODRINTH_BG
            } else {
                crate::theme::MODRINTH_TEXT
            },
        );
        if resp.clicked() {
            result = Some(value);
        }
    }
    result
}

fn modrinth_skin_button(ui: &mut Ui, icon: Icon, label: &str, primary: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill = if primary {
        crate::theme::lerp_color(
            crate::theme::MODRINTH_GREEN,
            crate::theme::MODRINTH_GREEN_LIGHT,
            hover * 0.6,
        )
    } else {
        crate::theme::lerp_color(
            Color32::TRANSPARENT,
            crate::theme::MODRINTH_SURFACE_HOVER,
            hover,
        )
    };
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(10.0), fill);
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(10.0),
        Stroke::new(
            1.0_f32,
            if primary {
                crate::theme::MODRINTH_GREEN
            } else {
                crate::theme::MODRINTH_SURFACE_HIGH
            },
        ),
    );
    let text_color = if primary {
        crate::theme::MODRINTH_BG
    } else {
        crate::theme::MODRINTH_TEXT
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.5),
        text_color,
    );
    let content_w = 15.0 + 7.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(left + 7.5, rect.center().y), vec2(15.0, 15.0)),
        text_color,
        1.4,
    );
    ui.painter().galley(
        pos2(left + 22.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn extract_head_avatar(skin: Option<&RgbaImage>, is_slim: bool) -> egui::ColorImage {
    let default_img;
    let img = match skin {
        Some(s) => s,
        None => {
            default_img = if is_slim {
                generate_default_skin(true)
            } else {
                embedded_steve().unwrap_or_else(|| generate_default_skin(false))
            };
            &default_img
        }
    };

    let mut head = RgbaImage::new(8, 8);
    for y in 0..8 {
        for x in 0..8 {
            let px = 8 + x;
            let py = 8 + y;
            if px < img.width() && py < img.height() {
                head.put_pixel(x, y, *img.get_pixel(px, py));
            }
        }
    }

    for y in 0..8 {
        for x in 0..8 {
            let px = 40 + x;
            let py = 8 + y;
            if px < img.width() && py < img.height() {
                let pixel = img.get_pixel(px, py);
                if pixel[3] > 10 {
                    head.put_pixel(x, y, *pixel);
                }
            }
        }
    }

    let size = [head.width() as usize, head.height() as usize];
    let pixels: Vec<Color32> = head
        .pixels()
        .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    egui::ColorImage { size, pixels }
}

fn composite_front_skin(skin: &RgbaImage, is_slim: bool) -> egui::ColorImage {
    let arm_w = if is_slim { 3 } else { 4 };
    let mut out = RgbaImage::new(16, 32);

    let blit = |dest: &mut RgbaImage,
                src: &RgbaImage,
                sx: u32,
                sy: u32,
                sw: u32,
                sh: u32,
                dx: u32,
                dy: u32| {
        for y in 0..sh {
            for x in 0..sw {
                let px = sx + x;
                let py = sy + y;
                if px < src.width() && py < src.height() {
                    let pixel = src.get_pixel(px, py);
                    if pixel[3] > 10 {
                        dest.put_pixel(dx + x, dy + y, *pixel);
                    }
                }
            }
        }
    };

    // 1. Head (8x8) at dx = 4, dy = 0
    blit(&mut out, skin, 8, 8, 8, 8, 4, 0);
    // Head outer layer (hat) (8x8)
    blit(&mut out, skin, 40, 8, 8, 8, 4, 0);

    // 2. Torso (8x12) at dx = 4, dy = 8
    blit(&mut out, skin, 20, 20, 8, 12, 4, 8);
    // Torso outer layer (jacket)
    if skin.height() >= 64 {
        blit(&mut out, skin, 20, 36, 8, 12, 4, 8);
    }

    // 3. Right Arm (arm_w x 12) at dx = 4 - arm_w, dy = 8
    let r_arm_dx = 4 - arm_w;
    blit(&mut out, skin, 44, 20, arm_w, 12, r_arm_dx, 8);
    if skin.height() >= 64 {
        blit(&mut out, skin, 44, 36, arm_w, 12, r_arm_dx, 8);
    }

    // 4. Left Arm (arm_w x 12) at dx = 12, dy = 8
    if skin.height() >= 64 {
        blit(&mut out, skin, 36, 52, arm_w, 12, 12, 8);
        blit(&mut out, skin, 52, 52, arm_w, 12, 12, 8);
    } else {
        blit(&mut out, skin, 44, 20, arm_w, 12, 12, 8);
    }

    // 5. Right Leg (4x12) at dx = 4, dy = 20
    blit(&mut out, skin, 4, 20, 4, 12, 4, 20);
    if skin.height() >= 64 {
        blit(&mut out, skin, 4, 36, 4, 12, 4, 20);
    }

    // 6. Left Leg (4x12) at dx = 8, dy = 20
    if skin.height() >= 64 {
        blit(&mut out, skin, 20, 52, 4, 12, 8, 20);
        blit(&mut out, skin, 4, 52, 4, 12, 8, 20);
    } else {
        blit(&mut out, skin, 4, 20, 4, 12, 8, 20);
    }

    let size = [out.width() as usize, out.height() as usize];
    let pixels: Vec<Color32> = out
        .pixels()
        .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    egui::ColorImage { size, pixels }
}

fn generate_default_skin(is_slim: bool) -> RgbaImage {
    let mut skin = RgbaImage::new(64, 64);

    let (skin_color, hair_color, eye_color, shirt_color, pants_color) = if is_slim {
        // Alex palette
        (
            [235, 185, 150, 255],
            [190, 100, 40, 255],
            [40, 140, 70, 255],
            [90, 125, 80, 255],
            [95, 75, 60, 255],
        )
    } else {
        // Steve palette
        (
            [219, 172, 142, 255],
            [74, 45, 23, 255],
            [45, 45, 180, 255],
            [0, 160, 175, 255],
            [43, 59, 137, 255],
        )
    };

    // Head base (8, 8, 8, 8)
    for y in 8..16 {
        for x in 8..16 {
            skin.put_pixel(x, y, image::Rgba(skin_color));
        }
    }
    // Hair on top of face (first 2 rows)
    for y in 8..10 {
        for x in 8..16 {
            skin.put_pixel(x, y, image::Rgba(hair_color));
        }
    }
    // Eyes at y = 12
    skin.put_pixel(9, 12, image::Rgba([255, 255, 255, 255]));
    skin.put_pixel(10, 12, image::Rgba(eye_color));
    skin.put_pixel(13, 12, image::Rgba(eye_color));
    skin.put_pixel(14, 12, image::Rgba([255, 255, 255, 255]));

    // Torso (20, 20, 8, 12)
    for y in 20..32 {
        for x in 20..28 {
            skin.put_pixel(x, y, image::Rgba(shirt_color));
        }
    }

    // Arms
    let arm_w = if is_slim { 3 } else { 4 };
    for y in 20..32 {
        for x in 44..(44 + arm_w) {
            let col = if y < 24 { shirt_color } else { skin_color };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }
    for y in 52..64 {
        for x in 36..(36 + arm_w) {
            let col = if y < 56 { shirt_color } else { skin_color };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }

    // Legs
    for y in 20..32 {
        for x in 4..8 {
            let col = if y < 30 {
                pants_color
            } else {
                [60, 60, 60, 255]
            };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }
    for y in 52..64 {
        for x in 20..24 {
            let col = if y < 62 {
                pants_color
            } else {
                [60, 60, 60, 255]
            };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }

    skin
}
