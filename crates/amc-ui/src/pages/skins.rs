use crate::theme;
use crate::theme::{
    lerp_color, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_LIGHT,
    TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::icons::{self, Icon};
use amc_auth::Account;
use amc_core::skins::SkinLibrary;
use egui::{pos2, vec2, Color32, Rect, ScrollArea, Sense, Stroke, TextureOptions, Ui};
use image::RgbaImage;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const STEVE_SKIN: &[u8] = include_bytes!("../../../../assets/modrinth/skins/steve.png");

/// Visual skin for the shared skin page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinsSkin {
    #[default]
    Amc,
    Modrinth,
    Millida,
    Labymod,
}

pub struct SkinsPage {
    pub is_slim_model: bool,
    pub active_skin_name: String,
    pub custom_skin_path: Option<PathBuf>,
    pub skin_image: Option<RgbaImage>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub dirty: bool,
    pub avatar_dirty: bool,
    skin3d: Skin3d,
    saved_skins: SkinLibrary,
    saved_root: Option<PathBuf>,
    saved_heads: HashMap<String, egui::TextureHandle>,
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
            skin3d: Skin3d::default(),
            saved_skins: SkinLibrary::default(),
            saved_root: None,
            saved_heads: HashMap::new(),
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
        root_dir: &Path,
    ) {
        if skin == SkinsSkin::Modrinth {
            self.ensure_saved_library(root_dir);
            self.show_modrinth(ui, account, lang);
            return;
        }
        if skin == SkinsSkin::Millida {
            self.ensure_saved_library(root_dir);
            self.show_millida(ui, account, lang);
            return;
        }
        if skin == SkinsSkin::Labymod {
            self.ensure_saved_library(root_dir);
            self.show_labymod(ui, account, lang);
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

    /// Millida wardrobe: native 3D preview on a cut-corner stage with the
    /// skin library in a pixel plate rail.
    fn show_millida(&mut self, ui: &mut Ui, account: Option<&Account>, lang: amc_core::Language) {
        self.ensure_preview_texture(ui.ctx());

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            let (mark, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            millida_pixel_plate(
                ui.painter(),
                mark,
                3.0,
                theme::ACCENT.into(),
                theme::ACCENT_LIGHT.into(),
            );
            ui.label(
                egui::RichText::new(lang.skins_selector_title())
                    .font(theme::display(26.0))
                    .color(theme::HEAD),
            );
        });
        ui.add_space(12.0);

        let available = ui.available_width();
        let gap = 16.0;
        let left_w = (available * 0.46).clamp(280.0, 350.0);
        let right_w = (available - left_w - gap).max(240.0);
        let panel_h = (ui.available_height() - 4.0).max(420.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(left_w, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_millida_preview(ui, account, lang, left_w, panel_h),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(right_w, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.show_millida_skin_list(ui, lang, right_w));
                },
            );
        });
    }

    fn show_millida_preview(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        lang: amc_core::Language,
        width: f32,
        panel_h: f32,
    ) {
        let preview_h = (panel_h - 154.0).max(250.0);
        let (card, _) = ui.allocate_exact_size(vec2(width, preview_h), Sense::hover());
        millida_pixel_plate(
            ui.painter(),
            card,
            12.0,
            theme::BG_CARD.into(),
            theme::BORDER_STRONG.into(),
        );
        ui.painter().circle_filled(
            pos2(card.center().x, card.center().y + 8.0),
            preview_h * 0.33,
            theme::with_alpha(theme::ACCENT, 20),
        );

        let viewer = card.shrink2(vec2(12.0, 26.0));
        self.skin3d
            .show(ui, viewer, self.skin_image.as_ref(), self.is_slim_model);
        ui.painter().text(
            pos2(card.center().x, card.bottom() - 15.0),
            egui::Align2::CENTER_CENTER,
            lang.skins_drag_rotate(),
            egui::FontId::proportional(10.5),
            theme::MUTED.into(),
        );

        let chip = egui::Rect::from_min_size(
            pos2(card.left() + 14.0, card.top() + 14.0),
            vec2(98.0, 24.0),
        );
        millida_pixel_plate(
            ui.painter(),
            chip,
            4.0,
            theme::with_alpha(theme::BG, 220),
            theme::ACCENT.into(),
        );
        ui.painter().text(
            chip.center(),
            egui::Align2::CENTER_CENTER,
            if self.is_slim_model {
                lang.skins_model_slim()
            } else {
                lang.skins_model_classic()
            },
            egui::FontId::proportional(10.5),
            theme::TEXT.into(),
        );

        ui.add_space(10.0);
        let (tray, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
        millida_pixel_plate(
            ui.painter(),
            tray,
            5.0,
            theme::BG.into(),
            theme::BORDER_STRONG.into(),
        );
        let segment_w = (tray.width() - 8.0) / 2.0;
        for (idx, (label, active, slim)) in [
            (lang.skins_model_classic(), !self.is_slim_model, false),
            (lang.skins_model_slim(), self.is_slim_model, true),
        ]
        .into_iter()
        .enumerate()
        {
            let seg = egui::Rect::from_min_size(
                pos2(tray.left() + 4.0 + idx as f32 * segment_w, tray.top() + 4.0),
                vec2(segment_w, 28.0),
            );
            let resp = ui.interact(seg, ui.id().with(("millida_model", idx)), Sense::click());
            if active {
                millida_pixel_rect(ui.painter(), seg, 4.0, theme::ACCENT.into());
            }
            ui.painter().text(
                seg.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::new(11.5, egui::FontFamily::Name("bold".into())),
                if active {
                    theme::BG.into()
                } else {
                    theme::TEXT.into()
                },
            );
            if resp.clicked() && self.is_slim_model != slim {
                self.is_slim_model = slim;
                self.dirty = true;
                self.avatar_dirty = true;
            }
        }

        ui.add_space(8.0);
        let button_w = ((width - 8.0) / 2.0).max(110.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            if millida_skin_button(ui, Icon::Folder, lang.skins_load_skin(), true, button_w) {
                self.load_skin_dialog(lang);
            }
            let reset = if self.is_slim_model {
                lang.skins_btn_reset_alex()
            } else {
                lang.skins_btn_reset_steve()
            };
            if millida_skin_button(ui, Icon::Refresh, reset, false, button_w) {
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
                .font(egui::FontId::proportional(11.0))
                .color(theme::MUTED),
        );
    }

    fn show_millida_skin_list(&mut self, ui: &mut Ui, lang: amc_core::Language, _width: f32) {
        let width = ui.available_width();
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.nav_skins())
                    .font(theme::display(16.0))
                    .color(theme::HEAD),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if millida_skin_button(
                    ui,
                    Icon::Plus,
                    lang.skins_add(),
                    true,
                    (width * 0.43).clamp(108.0, 154.0),
                ) {
                    self.load_skin_dialog(lang);
                }
            });
        });
        ui.add_space(10.0);

        let (card, _) = ui.allocate_exact_size(vec2(width, 74.0), Sense::hover());
        millida_pixel_plate(
            ui.painter(),
            card,
            8.0,
            theme::BG_CARD.into(),
            theme::BORDER_STRONG.into(),
        );
        let head = egui::Rect::from_min_size(
            pos2(card.left() + 12.0, card.top() + 12.0),
            vec2(50.0, 50.0),
        );
        millida_pixel_plate(
            ui.painter(),
            head,
            6.0,
            theme::BG.into(),
            theme::BORDER_DEFAULT.into(),
        );
        if let Some(tex) = &self.texture_handle {
            ui.painter().image(
                tex.id(),
                head.shrink(5.0),
                egui::Rect::from_min_max(pos2(0.25, 0.0), pos2(0.75, 0.25)),
                Color32::WHITE,
            );
        }
        ui.painter().text(
            pos2(card.left() + 74.0, card.center().y - 9.0),
            egui::Align2::LEFT_CENTER,
            truncate_skin_name(&self.active_skin_name, 24),
            egui::FontId::new(13.0, egui::FontFamily::Name("bold".into())),
            theme::HEAD.into(),
        );
        ui.painter().text(
            pos2(card.left() + 74.0, card.center().y + 11.0),
            egui::Align2::LEFT_CENTER,
            if self.is_slim_model {
                lang.skins_model_slim()
            } else {
                lang.skins_model_classic()
            },
            egui::FontId::proportional(11.0),
            theme::MUTED.into(),
        );
        let badge = egui::Rect::from_min_size(
            pos2(card.right() - 78.0, card.center().y - 11.0),
            vec2(66.0, 22.0),
        );
        millida_pixel_plate(
            ui.painter(),
            badge,
            4.0,
            theme::BG.into(),
            theme::ACCENT.into(),
        );
        ui.painter().text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            lang.skins_active_badge(),
            egui::FontId::proportional(10.0),
            theme::ACCENT_LIGHT.into(),
        );

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(lang.skins_defaults())
                .font(egui::FontId::proportional(12.0))
                .strong()
                .color(theme::MUTED),
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
                let (tile, resp) = ui.allocate_exact_size(vec2(tile_w, 96.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                let active = self.skin_image.is_none() && self.is_slim_model == slim;
                millida_pixel_plate(
                    ui.painter(),
                    tile,
                    8.0,
                    theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover),
                    if active {
                        theme::ACCENT.into()
                    } else {
                        theme::BORDER_STRONG.into()
                    },
                );
                let head = egui::Rect::from_center_size(
                    pos2(tile.center().x, tile.top() + 36.0),
                    vec2(46.0, 46.0),
                );
                millida_pixel_plate(
                    ui.painter(),
                    head,
                    5.0,
                    theme::BG.into(),
                    theme::BORDER_DEFAULT.into(),
                );
                ui.painter().image(
                    tex.id(),
                    head.shrink(5.0),
                    egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().text(
                    pos2(tile.center().x, tile.bottom() - 16.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.5),
                    theme::TEXT.into(),
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
                .font(egui::FontId::proportional(12.0))
                .strong()
                .color(theme::MUTED),
        );
        ui.add_space(8.0);
        self.show_millida_saved(ui, lang, width);
    }

    fn show_millida_saved(&mut self, ui: &mut Ui, lang: amc_core::Language, width: f32) {
        if self.saved_skins.is_empty() {
            let (empty, resp) = ui.allocate_exact_size(vec2(width, 58.0), Sense::click());
            millida_pixel_plate(
                ui.painter(),
                empty,
                6.0,
                theme::with_alpha(theme::BG_CARD, 150),
                theme::BORDER_DEFAULT.into(),
            );
            icons::draw_centered(
                ui.painter(),
                Icon::Plus,
                pos2(empty.left() + 24.0, empty.center().y),
                14.0,
                theme::MUTED,
            );
            ui.painter().text(
                pos2(empty.left() + 42.0, empty.center().y),
                egui::Align2::LEFT_CENTER,
                lang.skins_no_saved(),
                egui::FontId::proportional(12.0),
                theme::MUTED.into(),
            );
            if resp.clicked() {
                self.load_skin_dialog(lang);
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        } else {
            ui.label(
                egui::RichText::new(lang.skins_saved_hint())
                    .font(egui::FontId::proportional(10.5))
                    .color(theme::MUTED),
            );
            ui.add_space(6.0);
            let skins = self.saved_skins.skins().to_vec();
            for skin in skins {
                let path = self.saved_skins.path_of(&skin);
                let current = self.custom_skin_path.as_deref() == Some(path.as_path());
                let (row, resp) = ui.allocate_exact_size(vec2(width, 56.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                millida_pixel_plate(
                    ui.painter(),
                    row,
                    6.0,
                    theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover),
                    if current {
                        theme::ACCENT.into()
                    } else {
                        theme::BORDER_STRONG.into()
                    },
                );
                let key = skin.id.to_string();
                if !self.saved_heads.contains_key(&key) {
                    if let Ok(image) = image::open(&path) {
                        let rgba = image.to_rgba8();
                        let head = extract_head_avatar(Some(&rgba), skin.slim);
                        let texture = ui.ctx().load_texture(
                            format!("saved_skin_{key}"),
                            head,
                            TextureOptions::NEAREST,
                        );
                        self.saved_heads.insert(key.clone(), texture);
                    }
                }
                let head =
                    Rect::from_min_size(pos2(row.left() + 10.0, row.top() + 8.0), vec2(40.0, 40.0));
                millida_pixel_plate(
                    ui.painter(),
                    head,
                    5.0,
                    theme::BG.into(),
                    theme::BORDER_DEFAULT.into(),
                );
                if let Some(texture) = self.saved_heads.get(&key) {
                    ui.painter().image(
                        texture.id(),
                        head.shrink(4.0),
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                ui.painter().text(
                    pos2(row.left() + 60.0, row.center().y - 8.0),
                    egui::Align2::LEFT_CENTER,
                    truncate_skin_name(&skin.name, 24),
                    egui::FontId::new(12.5, egui::FontFamily::Name("bold".into())),
                    theme::HEAD.into(),
                );
                ui.painter().text(
                    pos2(row.left() + 60.0, row.center().y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    if skin.slim {
                        lang.skins_model_slim()
                    } else {
                        lang.skins_model_classic()
                    },
                    egui::FontId::proportional(10.5),
                    theme::MUTED.into(),
                );
                let remove = Rect::from_min_size(
                    pos2(row.right() - 74.0, row.center().y - 11.0),
                    vec2(64.0, 22.0),
                );
                let remove_resp = ui.interact(
                    remove,
                    ui.id().with(("saved_skin_remove", skin.id)),
                    Sense::click(),
                );
                ui.painter().text(
                    remove.center(),
                    egui::Align2::CENTER_CENTER,
                    lang.skins_remove_saved(),
                    egui::FontId::proportional(10.5),
                    if remove_resp.hovered() {
                        theme::DANGER.into()
                    } else {
                        theme::MUTED.into()
                    },
                );
                if remove_resp.clicked() {
                    if let Err(err) = self.saved_skins.remove(skin.id) {
                        tracing::warn!("saved skin remove failed: {err}");
                    }
                    self.saved_heads.remove(&key);
                    if current {
                        self.custom_skin_path = None;
                    }
                } else if resp.clicked() {
                    if let Ok(image) = image::open(&path) {
                        self.skin_image = Some(image.to_rgba8());
                        self.custom_skin_path = Some(path);
                        self.active_skin_name = skin.name.clone();
                        self.is_slim_model = skin.slim;
                        self.dirty = true;
                        self.avatar_dirty = true;
                    }
                }
                if resp.hovered() && !remove_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                ui.add_space(6.0);
            }
        }

        if self.skin_image.is_some() {
            ui.add_space(4.0);
            if millida_skin_button(
                ui,
                Icon::Check,
                lang.skins_save_current(),
                false,
                width.max(120.0),
            ) {
                self.save_current_skin();
            }
        }
    }

    /// Modrinth reference composition: a sticky preview column and a skin
    /// library rail with the active skin, default heads and an add tile.
    /// LabyMod skin selector: a large 3D preview on the left and the active
    /// skin / defaults / saved library on the right.
    fn show_labymod(&mut self, ui: &mut Ui, account: Option<&Account>, lang: amc_core::Language) {
        self.ensure_preview_texture(ui.ctx());
        ui.add_space(12.0);
        labymod_skins_header(ui, lang);
        ui.add_space(12.0);

        let available = ui.available_width();
        let gap = 14.0;
        let left = (available * 0.44).clamp(300.0, 420.0);
        let right = (available - left - gap).max(240.0);
        let panel_h = (ui.available_height() - 4.0).max(420.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(left, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.show_labymod_preview(ui, account, lang, left, panel_h),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(right, panel_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.show_labymod_list(ui, lang, right));
                },
            );
        });
    }

    fn show_labymod_preview(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        lang: amc_core::Language,
        width: f32,
        panel_h: f32,
    ) {
        // Reserve the full action stack below the preview. At 960x600 the
        // previous 140 px reserve pushed the account footer under the window.
        let preview_h = (panel_h - 170.0).max(240.0);
        let (rect, _) = ui.allocate_exact_size(vec2(width, preview_h), Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(18.0), BG_CARD);
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(18.0),
            Stroke::new(1.0_f32, BORDER_STRONG),
        );
        let glow = ui.painter().with_clip_rect(rect);
        glow.circle_filled(
            egui::pos2(rect.center().x, rect.center().y + 8.0),
            preview_h * 0.32,
            theme::with_alpha(theme::ACCENT, 18),
        );

        let viewer = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 12.0, rect.top() + 28.0),
            egui::pos2(rect.right() - 12.0, rect.bottom() - 46.0),
        );
        self.skin3d
            .show(ui, viewer, self.skin_image.as_ref(), self.is_slim_model);

        let model = if self.is_slim_model {
            lang.skins_model_slim()
        } else {
            lang.skins_model_classic()
        };
        let chip = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 14.0, rect.top() + 14.0),
            vec2(102.0, 24.0),
        );
        ui.painter().rect_filled(
            chip,
            egui::Rounding::same(12.0),
            theme::with_alpha(theme::BG, 190),
        );
        ui.painter().text(
            chip.center(),
            egui::Align2::CENTER_CENTER,
            model,
            egui::FontId::proportional(10.5),
            theme::TEXT_PRIMARY.into(),
        );
        ui.painter().text(
            egui::pos2(rect.center().x, rect.bottom() - 16.0),
            egui::Align2::CENTER_CENTER,
            truncate_skin_name(&self.active_skin_name, 28),
            egui::FontId::proportional(11.5),
            theme::TEXT_MUTED.into(),
        );

        ui.add_space(8.0);
        if let Some(new_slim) = labymod_model_switch(ui, self.is_slim_model, lang, width) {
            self.is_slim_model = new_slim;
            self.dirty = true;
            self.avatar_dirty = true;
        }

        ui.add_space(8.0);
        let button_w = ((width - 8.0) / 2.0).max(110.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            if labymod_skin_button(ui, Icon::Folder, lang.skins_load_skin(), true, button_w) {
                self.load_skin_dialog(lang);
            }
            let reset = if self.is_slim_model {
                lang.skins_btn_reset_alex()
            } else {
                lang.skins_btn_reset_steve()
            };
            if labymod_skin_button(ui, Icon::Refresh, reset, false, button_w) {
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
                .font(egui::FontId::proportional(11.0))
                .color(theme::TEXT_MUTED),
        );
    }

    fn show_labymod_list(&mut self, ui: &mut Ui, lang: amc_core::Language, width: f32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.nav_skins())
                    .font(egui::FontId::new(
                        17.0,
                        egui::FontFamily::Name("bold".into()),
                    ))
                    .color(TEXT_HEADING),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if labymod_skin_button(
                    ui,
                    Icon::Plus,
                    lang.skins_add(),
                    true,
                    (width * 0.42).clamp(108.0, 154.0),
                ) {
                    self.load_skin_dialog(lang);
                }
            });
        });
        ui.add_space(10.0);

        let (card, _) = ui.allocate_exact_size(vec2(width, 72.0), Sense::hover());
        ui.painter()
            .rect_filled(card, egui::Rounding::same(14.0), BG_CARD);
        ui.painter().rect_stroke(
            card,
            egui::Rounding::same(14.0),
            Stroke::new(1.0_f32, theme::with_alpha(theme::ACCENT, 110)),
        );
        let head = egui::Rect::from_min_size(
            egui::pos2(card.left() + 12.0, card.top() + 12.0),
            vec2(48.0, 48.0),
        );
        ui.painter().rect_filled(
            head,
            egui::Rounding::same(10.0),
            theme::with_alpha(theme::ACCENT, 40),
        );
        if let Some(tex) = &self.texture_handle {
            ui.painter().image(
                tex.id(),
                head.shrink(4.0),
                egui::Rect::from_min_max(pos2(0.25, 0.0), pos2(0.75, 0.25)),
                Color32::WHITE,
            );
        }
        let name_chars = if width < 300.0 { 16 } else { 28 };
        ui.painter().text(
            egui::pos2(card.left() + 72.0, card.center().y - 9.0),
            egui::Align2::LEFT_CENTER,
            truncate_skin_name(&self.active_skin_name, name_chars),
            egui::FontId::proportional(13.5),
            TEXT_HEADING.into(),
        );
        ui.painter().text(
            egui::pos2(card.left() + 72.0, card.center().y + 11.0),
            egui::Align2::LEFT_CENTER,
            if self.is_slim_model {
                lang.skins_model_slim()
            } else {
                lang.skins_model_classic()
            },
            egui::FontId::proportional(11.0),
            TEXT_MUTED.into(),
        );
        if width >= 300.0 {
            let badge = egui::Rect::from_min_size(
                egui::pos2(card.right() - 78.0, card.center().y - 12.0),
                vec2(66.0, 24.0),
            );
            ui.painter().rect_filled(
                badge,
                egui::Rounding::same(12.0),
                theme::with_alpha(theme::ACCENT, 40),
            );
            ui.painter().text(
                badge.center(),
                egui::Align2::CENTER_CENTER,
                lang.skins_active_badge(),
                egui::FontId::proportional(10.5),
                theme::ACCENT_LIGHT.into(),
            );
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(lang.skins_defaults())
                .font(egui::FontId::proportional(12.5))
                .strong()
                .color(TEXT_MUTED),
        );
        ui.add_space(8.0);
        let gap = 8.0;
        let tile_w = ((width - gap) / 2.0).max(90.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);
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
                    theme::lerp_color(BG_CARD, theme::CARD_HOVER, hover),
                );
                ui.painter().rect_stroke(
                    tile,
                    egui::Rounding::same(12.0),
                    Stroke::new(
                        1.0_f32,
                        if active {
                            theme::ACCENT
                        } else {
                            theme::BORDER_STRONG
                        },
                    ),
                );
                let head = egui::Rect::from_center_size(
                    egui::pos2(tile.center().x, tile.top() + 34.0),
                    vec2(44.0, 44.0),
                );
                ui.painter().image(
                    tex.id(),
                    head,
                    egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().text(
                    egui::pos2(tile.center().x, tile.bottom() - 17.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.5),
                    TEXT_PRIMARY.into(),
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
                .color(TEXT_MUTED),
        );
        ui.add_space(8.0);
        self.show_labymod_saved(ui, lang, width);
    }

    fn show_labymod_saved(&mut self, ui: &mut Ui, lang: amc_core::Language, width: f32) {
        if self.saved_skins.is_empty() {
            let (empty, resp) = ui.allocate_exact_size(vec2(width, 62.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            ui.painter().rect_filled(
                empty,
                egui::Rounding::same(12.0),
                theme::lerp_color(Color32::TRANSPARENT, BG_CARD, hover),
            );
            ui.painter().rect_stroke(
                empty,
                egui::Rounding::same(12.0),
                Stroke::new(1.0_f32, BORDER_STRONG),
            );
            icons::draw(
                ui.painter(),
                Icon::Plus,
                egui::Rect::from_center_size(
                    egui::pos2(empty.left() + 24.0, empty.center().y),
                    vec2(15.0, 15.0),
                ),
                theme::ACCENT_LIGHT,
                1.5,
            );
            ui.painter().text(
                egui::pos2(empty.left() + 42.0, empty.center().y),
                egui::Align2::LEFT_CENTER,
                lang.skins_no_saved(),
                egui::FontId::proportional(12.0),
                TEXT_MUTED.into(),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                self.load_skin_dialog(lang);
            }
        } else {
            ui.label(
                egui::RichText::new(lang.skins_saved_hint())
                    .font(egui::FontId::proportional(11.0))
                    .color(TEXT_MUTED),
            );
            ui.add_space(6.0);
            let skins = self.saved_skins.skins().to_vec();
            for skin in skins {
                let path = self.saved_skins.path_of(&skin);
                let current = self.custom_skin_path.as_deref() == Some(path.as_path());
                let (row, resp) = ui.allocate_exact_size(vec2(width, 58.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                ui.painter().rect_filled(
                    row,
                    egui::Rounding::same(12.0),
                    theme::lerp_color(BG_CARD, theme::CARD_HOVER, hover),
                );
                ui.painter().rect_stroke(
                    row,
                    egui::Rounding::same(12.0),
                    Stroke::new(
                        1.0_f32,
                        if current {
                            theme::ACCENT
                        } else {
                            theme::BORDER_STRONG
                        },
                    ),
                );
                let key = skin.id.to_string();
                if !self.saved_heads.contains_key(&key) {
                    if let Ok(image) = image::open(&path) {
                        let rgba = image.to_rgba8();
                        let head = extract_head_avatar(Some(&rgba), skin.slim);
                        let texture = ui.ctx().load_texture(
                            format!("saved_skin_{key}"),
                            head,
                            TextureOptions::NEAREST,
                        );
                        self.saved_heads.insert(key.clone(), texture);
                    }
                }
                let head = egui::Rect::from_min_size(
                    egui::pos2(row.left() + 10.0, row.top() + 9.0),
                    vec2(40.0, 40.0),
                );
                ui.painter().rect_filled(
                    head,
                    egui::Rounding::same(8.0),
                    theme::with_alpha(theme::ACCENT, 32),
                );
                if let Some(texture) = self.saved_heads.get(&key) {
                    ui.painter().image(
                        texture.id(),
                        head.shrink(3.0),
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                ui.painter().text(
                    egui::pos2(row.left() + 60.0, row.center().y - 8.0),
                    egui::Align2::LEFT_CENTER,
                    truncate_skin_name(&skin.name, 24),
                    egui::FontId::proportional(12.5),
                    TEXT_HEADING.into(),
                );
                ui.painter().text(
                    egui::pos2(row.left() + 60.0, row.center().y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    if skin.slim {
                        lang.skins_model_slim()
                    } else {
                        lang.skins_model_classic()
                    },
                    egui::FontId::proportional(10.5),
                    TEXT_MUTED.into(),
                );
                let remove = egui::Rect::from_min_size(
                    egui::pos2(row.right() - 74.0, row.center().y - 11.0),
                    vec2(64.0, 22.0),
                );
                let remove_resp = ui.interact(
                    remove,
                    ui.id().with(("saved_skin_remove", skin.id)),
                    Sense::click(),
                );
                ui.painter().rect_filled(
                    remove,
                    egui::Rounding::same(8.0),
                    if remove_resp.hovered() {
                        theme::with_alpha(theme::DANGER, 24)
                    } else {
                        Color32::TRANSPARENT
                    },
                );
                ui.painter().text(
                    remove.center(),
                    egui::Align2::CENTER_CENTER,
                    lang.skins_remove_saved(),
                    egui::FontId::proportional(10.5),
                    theme::DANGER.into(),
                );
                if remove_resp.clicked() {
                    if let Err(err) = self.saved_skins.remove(skin.id) {
                        tracing::warn!("saved skin remove failed: {err}");
                    }
                    self.saved_heads.remove(&key);
                    if current {
                        self.custom_skin_path = None;
                    }
                } else if resp.clicked() {
                    if let Ok(image) = image::open(&path) {
                        self.skin_image = Some(image.to_rgba8());
                        self.custom_skin_path = Some(path);
                        self.active_skin_name = skin.name.clone();
                        self.is_slim_model = skin.slim;
                        self.dirty = true;
                        self.avatar_dirty = true;
                    }
                }
                if resp.hovered() && !remove_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                ui.add_space(6.0);
            }
        }

        if self.skin_image.is_some() {
            ui.add_space(4.0);
            if labymod_skin_button(
                ui,
                Icon::Check,
                lang.skins_save_current(),
                false,
                width.max(120.0),
            ) {
                self.save_current_skin();
            }
        }
    }

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
        let left_w = (available * 0.46).clamp(210.0, 360.0);
        let right_w = (available - left_w - gap).max(190.0);
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
        // Leave room for the model switch, both skin actions and the account
        // footer; at the 960x600 minimum the old fixed reserve clipped them.
        let preview_h = (panel_h - 170.0).max(250.0);
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

        let viewer_rect = Rect::from_min_max(
            pos2(rect.left() + 12.0, rect.top() + 34.0),
            pos2(rect.right() - 12.0, rect.bottom() - 34.0),
        );
        self.skin3d.show(
            ui,
            viewer_rect,
            self.skin_image.as_ref(),
            self.is_slim_model,
        );
        ui.painter().text(
            pos2(rect.center().x, rect.bottom() - 17.0),
            egui::Align2::CENTER_CENTER,
            lang.skins_drag_rotate(),
            egui::FontId::proportional(10.5),
            crate::theme::MODRINTH_TEXT_SECONDARY,
        );

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

    fn ensure_saved_library(&mut self, root_dir: &Path) {
        if self.saved_root.as_deref() == Some(root_dir) {
            return;
        }
        self.saved_skins = SkinLibrary::load(root_dir);
        self.saved_root = Some(root_dir.to_path_buf());
        self.saved_heads.clear();
    }

    /// Modrinth saved-skin library: rows with a real head preview, apply on
    /// click and a separate remove action. The empty state doubles as the
    /// "load a skin" affordance from the reference.
    fn show_modrinth_saved(&mut self, ui: &mut Ui, lang: amc_core::Language, width: f32) {
        if self.saved_skins.is_empty() {
            let (empty, resp) = ui.allocate_exact_size(vec2(width, 62.0), Sense::click());
            let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            ui.painter().rect_filled(
                empty,
                egui::Rounding::same(12.0),
                crate::theme::lerp_color(
                    Color32::TRANSPARENT,
                    crate::theme::MODRINTH_SURFACE,
                    hover,
                ),
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
        } else {
            ui.label(
                egui::RichText::new(lang.skins_saved_hint())
                    .font(egui::FontId::proportional(11.0))
                    .color(crate::theme::MODRINTH_TEXT_SECONDARY),
            );
            ui.add_space(6.0);
            let skins = self.saved_skins.skins().to_vec();
            for skin in skins {
                let path = self.saved_skins.path_of(&skin);
                let current = self.custom_skin_path.as_deref() == Some(path.as_path());
                let (row, resp) = ui.allocate_exact_size(vec2(width, 58.0), Sense::click());
                let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                ui.painter().rect_filled(
                    row,
                    egui::Rounding::same(12.0),
                    crate::theme::lerp_color(
                        crate::theme::MODRINTH_SURFACE,
                        crate::theme::MODRINTH_SURFACE_HOVER,
                        hover,
                    ),
                );
                ui.painter().rect_stroke(
                    row,
                    egui::Rounding::same(12.0),
                    Stroke::new(
                        1.0_f32,
                        if current {
                            crate::theme::MODRINTH_GREEN
                        } else {
                            crate::theme::MODRINTH_SURFACE_HIGH
                        },
                    ),
                );

                let key = skin.id.to_string();
                if !self.saved_heads.contains_key(&key) {
                    if let Ok(image) = image::open(&path) {
                        let rgba = image.to_rgba8();
                        let head = extract_head_avatar(Some(&rgba), skin.slim);
                        let texture = ui.ctx().load_texture(
                            format!("saved_skin_{key}"),
                            head,
                            TextureOptions::NEAREST,
                        );
                        self.saved_heads.insert(key.clone(), texture);
                    }
                }
                let head =
                    Rect::from_min_size(pos2(row.left() + 10.0, row.top() + 9.0), vec2(40.0, 40.0));
                ui.painter().rect_filled(
                    head,
                    egui::Rounding::same(8.0),
                    crate::theme::MODRINTH_SURFACE_HOVER,
                );
                if let Some(texture) = self.saved_heads.get(&key) {
                    ui.painter().image(
                        texture.id(),
                        head.shrink(3.0),
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                ui.painter().text(
                    pos2(row.left() + 60.0, row.center().y - 8.0),
                    egui::Align2::LEFT_CENTER,
                    truncate_skin_name(&skin.name, 24),
                    egui::FontId::proportional(12.5),
                    crate::theme::MODRINTH_TEXT_CONTRAST,
                );
                ui.painter().text(
                    pos2(row.left() + 60.0, row.center().y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    if skin.slim {
                        lang.skins_model_slim()
                    } else {
                        lang.skins_model_classic()
                    },
                    egui::FontId::proportional(10.5),
                    crate::theme::MODRINTH_TEXT_SECONDARY,
                );
                let remove = Rect::from_min_size(
                    pos2(row.right() - 74.0, row.center().y - 11.0),
                    vec2(64.0, 22.0),
                );
                let remove_resp = ui.interact(
                    remove,
                    ui.id().with(("saved_skin_remove", skin.id)),
                    Sense::click(),
                );
                ui.painter().rect_filled(
                    remove,
                    egui::Rounding::same(8.0),
                    if remove_resp.hovered() {
                        crate::theme::with_alpha(crate::theme::DANGER, 26)
                    } else {
                        Color32::TRANSPARENT
                    },
                );
                ui.painter().text(
                    remove.center(),
                    egui::Align2::CENTER_CENTER,
                    lang.skins_remove_saved(),
                    egui::FontId::proportional(10.5),
                    crate::theme::DANGER.into(),
                );
                if remove_resp.clicked() {
                    if let Err(err) = self.saved_skins.remove(skin.id) {
                        tracing::warn!("saved skin remove failed: {err}");
                    }
                    self.saved_heads.remove(&key);
                    if current {
                        self.custom_skin_path = None;
                    }
                } else if resp.clicked() {
                    if let Ok(image) = image::open(&path) {
                        self.skin_image = Some(image.to_rgba8());
                        self.custom_skin_path = Some(path);
                        self.active_skin_name = skin.name.clone();
                        self.is_slim_model = skin.slim;
                        self.dirty = true;
                        self.avatar_dirty = true;
                    }
                }
                if resp.hovered() && !remove_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                ui.add_space(6.0);
            }
        }

        if self.skin_image.is_some() {
            ui.add_space(4.0);
            if modrinth_skin_button(
                ui,
                Icon::Check,
                lang.skins_save_current(),
                false,
                width.max(120.0),
            ) {
                self.save_current_skin();
            }
        }
    }

    fn save_current_skin(&mut self) {
        let Some(image) = self.skin_image.as_ref() else {
            return;
        };
        let mut buf = Cursor::new(Vec::new());
        let encoded = image::DynamicImage::ImageRgba8(image.clone())
            .write_to(&mut buf, image::ImageFormat::Png)
            .is_ok();
        if !encoded {
            tracing::warn!("saved skin encode failed");
            return;
        }
        let name = self.active_skin_name.clone();
        match self
            .saved_skins
            .add(&name, self.is_slim_model, buf.get_ref())
        {
            Ok(saved) => {
                self.saved_heads.remove(&saved.id.to_string());
            }
            Err(err) => tracing::warn!("saved skin add failed: {err}"),
        }
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
        self.show_modrinth_saved(ui, lang, width);
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

fn labymod_skins_header(ui: &mut Ui, lang: amc_core::Language) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::Rounding::same(16.0), BG_CARD);
    painter.rect_stroke(
        rect,
        egui::Rounding::same(16.0),
        Stroke::new(1.0_f32, BORDER_STRONG),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 22.0),
        egui::Align2::LEFT_CENTER,
        lang.nav_skins().to_uppercase(),
        egui::FontId::proportional(10.5),
        theme::ACCENT_LIGHT.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 47.0),
        egui::Align2::LEFT_CENTER,
        lang.skins_title(),
        egui::FontId::new(27.0, egui::FontFamily::Name("bold".into())),
        TEXT_HEADING.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 74.0),
        egui::Align2::LEFT_CENTER,
        lang.labymod_skins_subtitle(),
        egui::FontId::proportional(11.5),
        TEXT_MUTED.into(),
    );
}

fn labymod_model_switch(
    ui: &mut Ui,
    slim: bool,
    lang: amc_core::Language,
    width: f32,
) -> Option<bool> {
    let mut result = None;
    let (tray, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    ui.painter()
        .rect_filled(tray, egui::Rounding::same(18.0), theme::BG);
    ui.painter().rect_stroke(
        tray,
        egui::Rounding::same(18.0),
        Stroke::new(1.0_f32, BORDER_STRONG),
    );
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
        let resp = ui.interact(rect, ui.id().with(("laby_skin_model", idx)), Sense::click());
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        if active {
            ui.painter()
                .rect_filled(rect, egui::Rounding::same(14.0), theme::ACCENT);
        } else if hover > 0.02 {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(14.0),
                theme::with_alpha(theme::CARD_HOVER, (hover * 210.0) as u8),
            );
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            if active {
                Color32::WHITE
            } else {
                TEXT_MUTED.into()
            },
        );
        if resp.clicked() {
            result = Some(value);
        }
    }
    result
}

fn labymod_skin_button(ui: &mut Ui, icon: Icon, label: &str, primary: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let fill = if primary {
        theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover * 0.55)
    } else {
        theme::lerp_color(Color32::TRANSPARENT, theme::CARD_HOVER, hover)
    };
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(10.0), fill);
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(10.0),
        Stroke::new(
            1.0_f32,
            if primary {
                theme::ACCENT_LIGHT
            } else {
                BORDER_STRONG
            },
        ),
    );
    let text_color = if primary {
        Color32::WHITE
    } else {
        TEXT_PRIMARY.into()
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.0),
        text_color,
    );
    let content_w = 15.0 + 7.0 + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(left + 7.5, rect.center().y), vec2(15.0, 15.0)),
        text_color,
        1.5,
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

#[derive(Clone, Copy)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

impl V3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn rotate_y(self, cos: f32, sin: f32) -> Self {
        Self {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

#[derive(Clone, Copy)]
struct UvRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl UvRect {
    const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Normalised corners with a sub-pixel inset so nearest-neighbour
    /// sampling never bleeds the adjacent skin region into a face.
    fn corners(self) -> [(f32, f32); 4] {
        const SCALE: f32 = 1.0 / 64.0;
        const INSET: f32 = 0.03;
        let u0 = (self.x + INSET) * SCALE;
        let v0 = (self.y + INSET) * SCALE;
        let u1 = (self.x + self.w - INSET) * SCALE;
        let v1 = (self.y + self.h - INSET) * SCALE;
        [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
    }
}

/// Six skin faces in front/back/right/left/top/bottom order.
#[derive(Clone, Copy)]
struct BoxUv {
    faces: [UvRect; 6],
}

/// One textured, culled model face before projection.
struct SkinFace {
    verts: [V3; 4],
    uv: [(f32, f32); 4],
    normal: V3,
}

/// Native software-projected 3D player preview. The mesh is drawn by egui's
/// painter (no WebView/WebGL), sampled straight from the skin PNG.
pub struct Skin3d {
    texture: Option<egui::TextureHandle>,
    key: Option<(bool, usize, usize)>,
    yaw: f32,
}

impl Default for Skin3d {
    fn default() -> Self {
        Self {
            texture: None,
            key: None,
            yaw: std::f32::consts::FRAC_PI_8,
        }
    }
}

impl Skin3d {
    pub(crate) fn show(&mut self, ui: &mut Ui, rect: Rect, skin: Option<&RgbaImage>, slim: bool) {
        if rect.width() < 20.0 || rect.height() < 40.0 {
            return;
        }
        self.ensure_texture(ui.ctx(), skin, slim);
        let resp = ui.interact(
            rect,
            ui.id().with("modrinth_skin3d"),
            Sense::click_and_drag(),
        );
        if resp.dragged() {
            self.yaw = (self.yaw - resp.drag_delta().x * 0.011).rem_euclid(std::f32::consts::TAU);
        }
        if resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        if let Some(texture) = &self.texture {
            let shadow = pos2(rect.center().x, rect.center().y + rect.height() * 0.415);
            draw_soft_shadow(
                ui.painter(),
                shadow,
                rect.width() * 0.24,
                rect.height() * 0.038,
            );
            paint_player(ui.painter(), rect, texture.id(), slim, self.yaw);
        }
    }

    fn ensure_texture(&mut self, ctx: &egui::Context, skin: Option<&RgbaImage>, slim: bool) {
        let key = (
            slim,
            skin.map_or(0, |image| image.as_raw().as_ptr() as usize),
            skin.map_or(0, |image| image.as_raw().len()),
        );
        if self.key == Some(key) {
            return;
        }
        let image = match skin {
            Some(image) => image.clone(),
            None if slim => generate_default_skin(true),
            None => embedded_steve().unwrap_or_else(|| generate_default_skin(false)),
        };
        let size = [image.width() as usize, image.height() as usize];
        if size[0] == 0 || size[1] == 0 {
            return;
        }
        let pixels = image
            .pixels()
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        self.texture = Some(ctx.load_texture(
            "modrinth_skin3d",
            egui::ColorImage { size, pixels },
            TextureOptions::NEAREST,
        ));
        self.key = Some(key);
    }
}

/// One face after perspective projection, ready for depth sorting.
struct ProjectedFace {
    depth: f32,
    points: [egui::Pos2; 4],
    uvs: [(f32, f32); 4],
    lum: u8,
}

fn paint_player(
    painter: &egui::Painter,
    rect: Rect,
    texture: egui::TextureId,
    slim: bool,
    yaw: f32,
) {
    let mut faces = Vec::with_capacity(72);
    build_player(&mut faces, slim);

    let (sin, cos) = yaw.sin_cos();
    let camera = V3::new(0.0, 16.0, -62.0);
    let focal = rect.height() * 1.8;
    let light = V3::new(-0.42, 0.78, -0.46);
    let mut visible: Vec<ProjectedFace> = Vec::with_capacity(faces.len());

    for face in faces {
        let normal = face.normal.rotate_y(cos, sin);
        let centroid = face.verts.iter().fold(V3::new(0.0, 0.0, 0.0), |sum, v| {
            V3::new(sum.x + v.x, sum.y + v.y, sum.z + v.z)
        });
        let centroid =
            V3::new(centroid.x / 4.0, centroid.y / 4.0, centroid.z / 4.0).rotate_y(cos, sin);
        let to_camera = V3::new(
            camera.x - centroid.x,
            camera.y - centroid.y,
            camera.z - centroid.z,
        );
        if normal.dot(to_camera) <= 0.001 {
            continue;
        }
        let depth = centroid.z - camera.z;
        if depth <= 1.0 {
            continue;
        }

        let mut points = [egui::Pos2::ZERO; 4];
        let mut valid = true;
        for (slot, vertex) in points.iter_mut().zip(face.verts.iter()) {
            let p = vertex.rotate_y(cos, sin);
            let d = p.z - camera.z;
            if d <= 1.0 {
                valid = false;
                break;
            }
            let scale = focal / d;
            *slot = pos2(
                rect.center().x + p.x * scale,
                rect.center().y - (p.y - 16.0) * scale,
            );
        }
        if !valid {
            continue;
        }
        let lum = 0.62 + 0.38 * normal.dot(light).max(0.0);
        visible.push(ProjectedFace {
            depth,
            points,
            uvs: face.uv,
            lum: (lum.clamp(0.0, 1.0) * 255.0) as u8,
        });
    }

    visible.sort_by(|a, b| b.depth.total_cmp(&a.depth));

    let mut mesh = egui::Mesh {
        texture_id: texture,
        ..Default::default()
    };
    for face in visible {
        let base = mesh.vertices.len() as u32;
        let color = Color32::from_gray(face.lum);
        for (point, uv) in face.points.iter().zip(face.uvs.iter()) {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *point,
                uv: pos2(uv.0, uv.1),
                color,
            });
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_soft_shadow(painter: &egui::Painter, center: egui::Pos2, rx: f32, ry: f32) {
    if rx <= 0.0 || ry <= 0.0 {
        return;
    }
    let segments = 32u32;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, Color32::from_black_alpha(64));
    for index in 0..segments {
        let angle = index as f32 / segments as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(
            pos2(center.x + angle.cos() * rx, center.y + angle.sin() * ry),
            Color32::TRANSPARENT,
        );
    }
    for index in 1..=segments {
        let next = if index == segments { 1 } else { index + 1 };
        mesh.add_triangle(0, index, next);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn build_player(faces: &mut Vec<SkinFace>, slim: bool) {
    let head = BoxUv {
        faces: [
            UvRect::new(8.0, 8.0, 8.0, 8.0),
            UvRect::new(24.0, 8.0, 8.0, 8.0),
            UvRect::new(0.0, 8.0, 8.0, 8.0),
            UvRect::new(16.0, 8.0, 8.0, 8.0),
            UvRect::new(8.0, 0.0, 8.0, 8.0),
            UvRect::new(16.0, 0.0, 8.0, 8.0),
        ],
    };
    let hat = BoxUv {
        faces: [
            UvRect::new(40.0, 8.0, 8.0, 8.0),
            UvRect::new(56.0, 8.0, 8.0, 8.0),
            UvRect::new(32.0, 8.0, 8.0, 8.0),
            UvRect::new(48.0, 8.0, 8.0, 8.0),
            UvRect::new(40.0, 0.0, 8.0, 8.0),
            UvRect::new(48.0, 0.0, 8.0, 8.0),
        ],
    };
    let body = BoxUv {
        faces: [
            UvRect::new(20.0, 20.0, 8.0, 12.0),
            UvRect::new(32.0, 20.0, 8.0, 12.0),
            UvRect::new(16.0, 20.0, 4.0, 12.0),
            UvRect::new(28.0, 20.0, 4.0, 12.0),
            UvRect::new(20.0, 16.0, 8.0, 4.0),
            UvRect::new(28.0, 16.0, 8.0, 4.0),
        ],
    };
    let jacket = BoxUv {
        faces: [
            UvRect::new(20.0, 36.0, 8.0, 12.0),
            UvRect::new(32.0, 36.0, 8.0, 12.0),
            UvRect::new(16.0, 36.0, 4.0, 12.0),
            UvRect::new(28.0, 36.0, 4.0, 12.0),
            UvRect::new(20.0, 32.0, 8.0, 4.0),
            UvRect::new(28.0, 32.0, 8.0, 4.0),
        ],
    };
    let left_leg = BoxUv {
        faces: [
            UvRect::new(20.0, 52.0, 4.0, 12.0),
            UvRect::new(28.0, 52.0, 4.0, 12.0),
            UvRect::new(16.0, 52.0, 4.0, 12.0),
            UvRect::new(24.0, 52.0, 4.0, 12.0),
            UvRect::new(20.0, 48.0, 4.0, 4.0),
            UvRect::new(24.0, 48.0, 4.0, 4.0),
        ],
    };
    let left_leg_overlay = BoxUv {
        faces: [
            UvRect::new(4.0, 52.0, 4.0, 12.0),
            UvRect::new(12.0, 52.0, 4.0, 12.0),
            UvRect::new(0.0, 52.0, 4.0, 12.0),
            UvRect::new(8.0, 52.0, 4.0, 12.0),
            UvRect::new(4.0, 48.0, 4.0, 4.0),
            UvRect::new(8.0, 48.0, 4.0, 4.0),
        ],
    };
    let right_leg = BoxUv {
        faces: [
            UvRect::new(4.0, 20.0, 4.0, 12.0),
            UvRect::new(12.0, 20.0, 4.0, 12.0),
            UvRect::new(0.0, 20.0, 4.0, 12.0),
            UvRect::new(8.0, 20.0, 4.0, 12.0),
            UvRect::new(4.0, 16.0, 4.0, 4.0),
            UvRect::new(8.0, 16.0, 4.0, 4.0),
        ],
    };
    let right_leg_overlay = BoxUv {
        faces: [
            UvRect::new(4.0, 36.0, 4.0, 12.0),
            UvRect::new(12.0, 36.0, 4.0, 12.0),
            UvRect::new(0.0, 36.0, 4.0, 12.0),
            UvRect::new(8.0, 36.0, 4.0, 12.0),
            UvRect::new(4.0, 32.0, 4.0, 4.0),
            UvRect::new(8.0, 32.0, 4.0, 4.0),
        ],
    };
    let arm_w = if slim { 3.0 } else { 4.0 };
    let right_arm = BoxUv {
        faces: [
            UvRect::new(44.0, 20.0, arm_w, 12.0),
            UvRect::new(44.0 + arm_w + 4.0, 20.0, arm_w, 12.0),
            UvRect::new(40.0, 20.0, 4.0, 12.0),
            UvRect::new(44.0 + arm_w, 20.0, 4.0, 12.0),
            UvRect::new(44.0, 16.0, arm_w, 4.0),
            UvRect::new(44.0 + arm_w, 16.0, arm_w, 4.0),
        ],
    };
    let right_arm_overlay = BoxUv {
        faces: [
            UvRect::new(44.0, 36.0, arm_w, 12.0),
            UvRect::new(44.0 + arm_w + 4.0, 36.0, arm_w, 12.0),
            UvRect::new(40.0, 36.0, 4.0, 12.0),
            UvRect::new(44.0 + arm_w, 36.0, 4.0, 12.0),
            UvRect::new(44.0, 32.0, arm_w, 4.0),
            UvRect::new(44.0 + arm_w, 32.0, arm_w, 4.0),
        ],
    };
    let left_arm = BoxUv {
        faces: [
            UvRect::new(36.0, 52.0, arm_w, 12.0),
            UvRect::new(36.0 + arm_w + 4.0, 52.0, arm_w, 12.0),
            UvRect::new(32.0, 52.0, 4.0, 12.0),
            UvRect::new(36.0 + arm_w, 52.0, 4.0, 12.0),
            UvRect::new(36.0, 48.0, arm_w, 4.0),
            UvRect::new(36.0 + arm_w, 48.0, arm_w, 4.0),
        ],
    };
    let left_arm_overlay = BoxUv {
        faces: [
            UvRect::new(52.0, 52.0, arm_w, 12.0),
            UvRect::new(52.0 + arm_w + 4.0, 52.0, arm_w, 12.0),
            UvRect::new(48.0, 52.0, 4.0, 12.0),
            UvRect::new(52.0 + arm_w, 52.0, 4.0, 12.0),
            UvRect::new(52.0, 48.0, arm_w, 4.0),
            UvRect::new(52.0 + arm_w, 48.0, arm_w, 4.0),
        ],
    };

    push_box(
        faces,
        V3::new(0.0, 28.0, 0.0),
        V3::new(8.0, 8.0, 8.0),
        &head,
        0.0,
    );
    push_box(
        faces,
        V3::new(0.0, 28.0, 0.0),
        V3::new(8.0, 8.0, 8.0),
        &hat,
        0.5,
    );
    push_box(
        faces,
        V3::new(0.0, 18.0, 0.0),
        V3::new(8.0, 12.0, 4.0),
        &body,
        0.0,
    );
    push_box(
        faces,
        V3::new(0.0, 18.0, 0.0),
        V3::new(8.0, 12.0, 4.0),
        &jacket,
        0.25,
    );
    let arm_x = 4.0 + arm_w / 2.0;
    push_box(
        faces,
        V3::new(-arm_x, 18.0, 0.0),
        V3::new(arm_w, 12.0, 4.0),
        &right_arm,
        0.0,
    );
    push_box(
        faces,
        V3::new(-arm_x, 18.0, 0.0),
        V3::new(arm_w, 12.0, 4.0),
        &right_arm_overlay,
        0.25,
    );
    push_box(
        faces,
        V3::new(arm_x, 18.0, 0.0),
        V3::new(arm_w, 12.0, 4.0),
        &left_arm,
        0.0,
    );
    push_box(
        faces,
        V3::new(arm_x, 18.0, 0.0),
        V3::new(arm_w, 12.0, 4.0),
        &left_arm_overlay,
        0.25,
    );
    push_box(
        faces,
        V3::new(-2.0, 6.0, 0.0),
        V3::new(4.0, 12.0, 4.0),
        &right_leg,
        0.0,
    );
    push_box(
        faces,
        V3::new(-2.0, 6.0, 0.0),
        V3::new(4.0, 12.0, 4.0),
        &right_leg_overlay,
        0.25,
    );
    push_box(
        faces,
        V3::new(2.0, 6.0, 0.0),
        V3::new(4.0, 12.0, 4.0),
        &left_leg,
        0.0,
    );
    push_box(
        faces,
        V3::new(2.0, 6.0, 0.0),
        V3::new(4.0, 12.0, 4.0),
        &left_leg_overlay,
        0.25,
    );
}

fn push_box(faces: &mut Vec<SkinFace>, center: V3, size: V3, uv: &BoxUv, inflate: f32) {
    let hx = size.x / 2.0 + inflate;
    let hy = size.y / 2.0 + inflate;
    let hz = size.z / 2.0 + inflate;
    let (x0, x1) = (center.x - hx, center.x + hx);
    let (y0, y1) = (center.y - hy, center.y + hy);
    let (z0, z1) = (center.z - hz, center.z + hz);

    push_face(
        faces,
        [
            V3::new(x0, y1, z0),
            V3::new(x1, y1, z0),
            V3::new(x1, y0, z0),
            V3::new(x0, y0, z0),
        ],
        V3::new(0.0, 0.0, -1.0),
        uv.faces[0],
    );
    push_face(
        faces,
        [
            V3::new(x1, y1, z1),
            V3::new(x0, y1, z1),
            V3::new(x0, y0, z1),
            V3::new(x1, y0, z1),
        ],
        V3::new(0.0, 0.0, 1.0),
        uv.faces[1],
    );
    push_face(
        faces,
        [
            V3::new(x1, y1, z1),
            V3::new(x1, y1, z0),
            V3::new(x1, y0, z0),
            V3::new(x1, y0, z1),
        ],
        V3::new(1.0, 0.0, 0.0),
        uv.faces[2],
    );
    push_face(
        faces,
        [
            V3::new(x0, y1, z0),
            V3::new(x0, y1, z1),
            V3::new(x0, y0, z1),
            V3::new(x0, y0, z0),
        ],
        V3::new(-1.0, 0.0, 0.0),
        uv.faces[3],
    );
    push_face(
        faces,
        [
            V3::new(x0, y1, z0),
            V3::new(x1, y1, z0),
            V3::new(x1, y1, z1),
            V3::new(x0, y1, z1),
        ],
        V3::new(0.0, 1.0, 0.0),
        uv.faces[4],
    );
    push_face(
        faces,
        [
            V3::new(x0, y0, z1),
            V3::new(x1, y0, z1),
            V3::new(x1, y0, z0),
            V3::new(x0, y0, z0),
        ],
        V3::new(0.0, -1.0, 0.0),
        uv.faces[5],
    );
}

fn push_face(faces: &mut Vec<SkinFace>, verts: [V3; 4], normal: V3, uv: UvRect) {
    faces.push(SkinFace {
        verts,
        uv: uv.corners(),
        normal,
    });
}

fn millida_pixel_rect(painter: &egui::Painter, rect: Rect, cut: f32, fill: Color32) {
    let c = cut
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5)
        .max(0.0);
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + c, rect.top()),
            pos2(rect.right() - c, rect.top() + c),
        ),
        egui::Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left(), rect.top() + c),
            pos2(rect.right(), rect.bottom() - c),
        ),
        egui::Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + c, rect.bottom() - c),
            pos2(rect.right() - c, rect.bottom()),
        ),
        egui::Rounding::ZERO,
        fill,
    );
}

fn millida_pixel_plate(
    painter: &egui::Painter,
    rect: Rect,
    cut: f32,
    fill: Color32,
    outline: Color32,
) {
    millida_pixel_rect(painter, rect, cut, outline);
    millida_pixel_rect(painter, rect.shrink(1.0), cut, fill);
}

fn millida_skin_button(ui: &mut Ui, icon: Icon, label: &str, primary: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let (fill, outline, text) = if primary {
        (
            theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
            theme::ACCENT_LIGHT.into(),
            theme::BG.into(),
        )
    } else {
        (
            theme::lerp_color(theme::BG_CARD, theme::BG_HOVER, hover),
            theme::BORDER_STRONG.into(),
            theme::TEXT.into(),
        )
    };
    millida_pixel_plate(ui.painter(), rect, 5.0, fill, outline);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::new(11.5, egui::FontFamily::Name("bold".into())),
        Color32::PLACEHOLDER,
    );
    let total = 16.0 + 8.0 + galley.size().x;
    let left = rect.center().x - total / 2.0;
    icons::draw_centered(
        ui.painter(),
        icon,
        pos2(left + 8.0, rect.center().y),
        15.0,
        text,
    );
    ui.painter().galley(
        pos2(left + 24.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text,
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

fn fill_rect(skin: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for py in y..y + h {
        for px in x..x + w {
            if px < skin.width() && py < skin.height() {
                skin.put_pixel(px, py, image::Rgba(color));
            }
        }
    }
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
    let shoes = [60, 60, 60, 255];

    // Head: every face is filled so the native 3D viewer never samples an
    // empty side/back region on the generated fallback.
    fill_rect(&mut skin, 8, 8, 8, 8, skin_color);
    fill_rect(&mut skin, 8, 8, 8, 2, hair_color);
    fill_rect(&mut skin, 0, 8, 8, 8, skin_color);
    fill_rect(&mut skin, 0, 8, 8, 3, hair_color);
    fill_rect(&mut skin, 16, 8, 8, 8, skin_color);
    fill_rect(&mut skin, 16, 8, 8, 3, hair_color);
    fill_rect(&mut skin, 24, 8, 8, 8, hair_color);
    fill_rect(&mut skin, 8, 0, 8, 8, hair_color);
    fill_rect(&mut skin, 16, 0, 8, 8, skin_color);
    // Face details on the front region.
    skin.put_pixel(9, 12, image::Rgba([255, 255, 255, 255]));
    skin.put_pixel(10, 12, image::Rgba(eye_color));
    skin.put_pixel(13, 12, image::Rgba(eye_color));
    skin.put_pixel(14, 12, image::Rgba([255, 255, 255, 255]));
    fill_rect(&mut skin, 11, 14, 2, 1, [150, 90, 70, 255]);
    if !is_slim {
        fill_rect(&mut skin, 8, 14, 8, 2, [120, 75, 45, 255]);
    }

    // Torso.
    fill_rect(&mut skin, 20, 20, 8, 12, shirt_color);
    fill_rect(&mut skin, 32, 20, 8, 12, shirt_color);
    fill_rect(&mut skin, 16, 20, 4, 12, shirt_color);
    fill_rect(&mut skin, 28, 20, 4, 12, shirt_color);
    fill_rect(&mut skin, 20, 16, 8, 4, shirt_color);
    fill_rect(&mut skin, 28, 16, 8, 4, shirt_color);

    let arm_w = if is_slim { 3 } else { 4 };
    // Right arm.
    fill_rect(&mut skin, 44, 20, arm_w, 12, skin_color);
    fill_rect(&mut skin, 44, 20, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 44 + arm_w + 4, 20, arm_w, 12, skin_color);
    fill_rect(&mut skin, 44 + arm_w + 4, 20, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 40, 20, 4, 12, skin_color);
    fill_rect(&mut skin, 40, 20, 4, 4, shirt_color);
    fill_rect(&mut skin, 44 + arm_w, 20, 4, 12, skin_color);
    fill_rect(&mut skin, 44 + arm_w, 20, 4, 4, shirt_color);
    fill_rect(&mut skin, 44, 16, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 44 + arm_w, 16, arm_w, 4, skin_color);
    // Left arm.
    fill_rect(&mut skin, 36, 52, arm_w, 12, skin_color);
    fill_rect(&mut skin, 36, 52, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 36 + arm_w + 4, 52, arm_w, 12, skin_color);
    fill_rect(&mut skin, 36 + arm_w + 4, 52, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 32, 52, 4, 12, skin_color);
    fill_rect(&mut skin, 32, 52, 4, 4, shirt_color);
    fill_rect(&mut skin, 36 + arm_w, 52, 4, 12, skin_color);
    fill_rect(&mut skin, 36 + arm_w, 52, 4, 4, shirt_color);
    fill_rect(&mut skin, 36, 48, arm_w, 4, shirt_color);
    fill_rect(&mut skin, 36 + arm_w, 48, arm_w, 4, skin_color);

    // Right leg.
    fill_rect(&mut skin, 4, 20, 4, 12, pants_color);
    fill_rect(&mut skin, 12, 20, 4, 12, pants_color);
    fill_rect(&mut skin, 0, 20, 4, 12, pants_color);
    fill_rect(&mut skin, 8, 20, 4, 12, pants_color);
    fill_rect(&mut skin, 4, 16, 4, 4, pants_color);
    fill_rect(&mut skin, 8, 16, 4, 4, shoes);
    fill_rect(&mut skin, 4, 30, 4, 2, shoes);
    fill_rect(&mut skin, 12, 30, 4, 2, shoes);
    fill_rect(&mut skin, 0, 30, 4, 2, shoes);
    fill_rect(&mut skin, 8, 30, 4, 2, shoes);
    // Left leg.
    fill_rect(&mut skin, 20, 52, 4, 12, pants_color);
    fill_rect(&mut skin, 28, 52, 4, 12, pants_color);
    fill_rect(&mut skin, 16, 52, 4, 12, pants_color);
    fill_rect(&mut skin, 24, 52, 4, 12, pants_color);
    fill_rect(&mut skin, 20, 48, 4, 4, pants_color);
    fill_rect(&mut skin, 24, 48, 4, 4, shoes);
    fill_rect(&mut skin, 20, 62, 4, 2, shoes);
    fill_rect(&mut skin, 28, 62, 4, 2, shoes);
    fill_rect(&mut skin, 16, 62, 4, 2, shoes);
    fill_rect(&mut skin, 24, 62, 4, 2, shoes);

    skin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_model_builds_all_cube_faces() {
        for slim in [false, true] {
            let mut faces = Vec::new();
            build_player(&mut faces, slim);
            assert_eq!(faces.len(), 12 * 6);
            for face in faces {
                assert!(face
                    .verts
                    .iter()
                    .all(|v| { v.x.is_finite() && v.y.is_finite() && v.z.is_finite() }));
                assert!(face
                    .uv
                    .iter()
                    .all(|(u, v)| { (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v) }));
            }
        }
    }

    #[test]
    fn generated_default_skin_covers_every_model_face() {
        let skin = generate_default_skin(true);
        assert_eq!(skin.dimensions(), (64, 64));
        // Front/back/right/left/top head regions; the 3D viewer samples all
        // of them, unlike the old front-only fallback.
        for (x, y) in [(8, 8), (24, 8), (0, 8), (16, 8), (8, 0), (16, 0)] {
            assert!(skin.get_pixel(x, y)[3] > 0, "face {x},{y} is empty");
        }
    }
}
