use egui::{vec2, Color32, Rounding, Sense, Stroke};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::theme::{BG_CARD, BORDER_DEFAULT, TEXT_MUTED};

/// Screenshot gallery modal (CONCEPT "Скриншоты", P10): thumbnails from
/// every instance's `screenshots/` folder, click opens externally.
/// Textures are cached per path and capped so huge galleries stay light.
pub struct ScreensModal {
    pub open: bool,
    thumbs: HashMap<PathBuf, egui::TextureHandle>,
    failed: std::collections::HashSet<PathBuf>,
}

impl Default for ScreensModal {
    fn default() -> Self {
        Self {
            open: false,
            thumbs: HashMap::new(),
            failed: std::collections::HashSet::new(),
        }
    }
}

fn list_shots(instances_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(instances_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let shots = entry.path().join("screenshots");
        let Ok(files) = std::fs::read_dir(&shots) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|e| e == "png" || e == "jpg")
                    .unwrap_or(false)
            {
                out.push(path);
            }
            if out.len() >= 60 {
                return out;
            }
        }
    }
    out.sort();
    out.reverse();
    out
}

impl ScreensModal {
    pub fn show(&mut self, ctx: &egui::Context, instances_dir: &Path, lang: amc_core::Language) {
        if !self.open {
            return;
        }
        let mut close = false;
        egui::Window::new(lang.screens_title())
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(640.0)
            .default_size(vec2(680.0, 480.0))
            .show(ctx, |ui| {
                let shots = list_shots(instances_dir);
                if shots.is_empty() {
                    ui.label(egui::RichText::new(lang.screens_empty()).color(TEXT_MUTED));
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                            for path in &shots {
                                let tex = self.thumb(ui.ctx(), path);
                                let (rect, resp) =
                                    ui.allocate_exact_size(vec2(200.0, 120.0), Sense::click());
                                ui.painter().rect_filled(rect, Rounding::ZERO, BG_CARD);
                                ui.painter().rect_stroke(
                                    rect,
                                    Rounding::ZERO,
                                    Stroke::new(1.0_f32, BORDER_DEFAULT),
                                );
                                if let Some(t) = tex {
                                    let inner = rect.shrink(4.0);
                                    ui.painter().image(
                                        t.id(),
                                        inner,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        Color32::WHITE,
                                    );
                                }
                                if resp.clicked() {
                                    let _ = open::that(path);
                                }
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                            }
                        });
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(lang.home_drawer_close()).clicked() {
                        close = true;
                    }
                    ui.label(egui::RichText::new(lang.screens_open()).color(TEXT_MUTED));
                });
            });
        if close {
            self.open = false;
        }
    }

    fn thumb(&mut self, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
        if let Some(tex) = self.thumbs.get(path) {
            return Some(tex.clone());
        }
        if self.failed.contains(path) {
            return None;
        }
        let bytes = std::fs::read(path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        // Downscale for a light gallery; aspect preserved.
        let small = img.thumbnail(320, 200);
        let rgba = small.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        if w == 0 || h == 0 {
            self.failed.insert(path.to_path_buf());
            return None;
        }
        let pixels: Vec<Color32> = rgba
            .pixels()
            .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
            .collect();
        let tex = ctx.load_texture(
            format!("shot-{}", path.display()),
            egui::ColorImage {
                size: [w, h],
                pixels,
            },
            egui::TextureOptions::LINEAR,
        );
        self.thumbs.insert(path.to_path_buf(), tex.clone());
        Some(tex)
    }
}
