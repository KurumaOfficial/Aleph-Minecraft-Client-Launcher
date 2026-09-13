use egui::{vec2, Color32, Rounding, Sense, Stroke, TextureOptions, Ui};
use amc_auth::Account;
use image::RgbaImage;
use std::path::PathBuf;
use crate::theme::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY,
};

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
            active_skin_name: "Steve (По умолчанию)".to_string(),
            custom_skin_path: None,
            skin_image: None,
            texture_handle: None,
            dirty: true,
            avatar_dirty: true,
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

        // Update preview texture if dirty
        if self.dirty || self.texture_handle.is_none() {
            let img = match &self.skin_image {
                Some(img) => composite_front_skin(img, self.is_slim_model),
                None => {
                    let default_skin = generate_default_skin(self.is_slim_model);
                    composite_front_skin(&default_skin, self.is_slim_model)
                }
            };
            self.texture_handle = Some(ui.ctx().load_texture("skin_preview", img, TextureOptions::NEAREST));
            self.dirty = false;
        }

        // Preview box
        let preview_width = 460.0;
        let preview_height = 420.0;
        let (rect, _) = ui.allocate_exact_size(vec2(preview_width, preview_height), Sense::hover());

        ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, BORDER_DEFAULT));

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

            let btn_classic = egui::Button::new(
                egui::RichText::new("Classic (4px)")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if classic_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if classic_active { RUBY } else { BG_HOVER })
            .stroke(Stroke::new(1.0, if classic_active { RUBY_LIGHT } else { BORDER_DEFAULT }))
            .min_size(vec2(110.0, 28.0));

            if ui.add(btn_classic).clicked() && self.is_slim_model {
                self.is_slim_model = false;
                self.dirty = true;
                self.avatar_dirty = true;
            }

            let btn_slim = egui::Button::new(
                egui::RichText::new("Slim (Alex 3px)")
                    .font(egui::FontId::proportional(11.0))
                    .strong()
                    .color(if slim_active { Color32::WHITE } else { TEXT_MUTED }),
            )
            .fill(if slim_active { RUBY } else { BG_HOVER })
            .stroke(Stroke::new(1.0, if slim_active { RUBY_LIGHT } else { BORDER_DEFAULT }))
            .min_size(vec2(110.0, 28.0));

            if ui.add(btn_slim).clicked() && !self.is_slim_model {
                self.is_slim_model = true;
                self.dirty = true;
                self.avatar_dirty = true;
            }
        });

        child.add_space(16.0);

        // 2D Character Display
        if let Some(tex) = &self.texture_handle {
            let (img_rect, _) = child.allocate_exact_size(vec2(130.0, 260.0), Sense::hover());
            child.painter().rect_filled(img_rect, Rounding::ZERO, BG_HOVER);
            child.painter().rect_stroke(img_rect, Rounding::ZERO, Stroke::new(1.0, BORDER_STRONG));
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
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Minecraft Skins (*.png)", &["png"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        self.skin_image = Some(img.to_rgba8());
                        self.active_skin_name = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Кастомный скин".to_string());
                        self.custom_skin_path = Some(path);
                        self.dirty = true;
                        self.avatar_dirty = true;
                    }
                }
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
                self.skin_image = None;
                self.custom_skin_path = None;
                self.active_skin_name = if self.is_slim_model {
                    "Alex (По умолчанию)".to_string()
                } else {
                    "Steve (По умолчанию)".to_string()
                };
                self.dirty = true;
                self.avatar_dirty = true;
            }
        });

        ui.add_space(18.0);

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

pub fn extract_head_avatar(skin: Option<&RgbaImage>, is_slim: bool) -> egui::ColorImage {
    let default_img;
    let img = match skin {
        Some(s) => s,
        None => {
            default_img = generate_default_skin(is_slim);
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

    let blit = |dest: &mut RgbaImage, src: &RgbaImage, sx: u32, sy: u32, sw: u32, sh: u32, dx: u32, dy: u32| {
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
            let col = if y < 30 { pants_color } else { [60, 60, 60, 255] };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }
    for y in 52..64 {
        for x in 20..24 {
            let col = if y < 62 { pants_color } else { [60, 60, 60, 255] };
            skin.put_pixel(x, y, image::Rgba(col));
        }
    }

    skin
}
