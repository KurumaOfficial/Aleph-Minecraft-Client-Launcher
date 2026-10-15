use egui::{vec2, Color32, Rounding, Sense, Stroke};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::theme::{BG_CARD, BORDER_DEFAULT, TEXT_MUTED};

/// Screenshot gallery modal (CONCEPT "Скриншоты", P10): thumbnails from
/// every instance's `screenshots/` folder, click opens externally,
/// trash button deletes the file. Textures are cached per path and
/// capped so huge galleries stay light. The file list itself refreshes
/// on open, on delete and at most every few seconds — never per frame.
#[derive(Default)]
pub struct ScreensModal {
    pub open: bool,
    thumbs: HashMap<PathBuf, egui::TextureHandle>,
    failed: std::collections::HashSet<PathBuf>,
    shots: Vec<PathBuf>,
    shots_at: Option<std::time::Instant>,
}

/// Minimum interval between gallery rescans while open.
const SHOTS_REFRESH: std::time::Duration = std::time::Duration::from_secs(3);
/// Gallery thumbnail bounds and decode cap (avoids OOM on huge PNGs).
const SHOT_THUMB_W: u32 = 320;
const SHOT_THUMB_H: u32 = 200;
const MAX_SHOT_BYTES: usize = 8_388_608;

/// Gallery outcome for the app shell (toast on delete).
#[derive(Debug, Default, PartialEq, Eq)]
pub enum ScreensAction {
    #[default]
    None,
    Deleted(PathBuf),
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
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        instances_dir: &Path,
        lang: amc_core::Language,
    ) -> ScreensAction {
        if !self.open {
            // Forget the list while closed so the next open rescans.
            self.shots_at = None;
            return ScreensAction::None;
        }
        // Throttled rescan: directory walks never run per frame.
        let stale = self.shots_at.is_none_or(|at| at.elapsed() >= SHOTS_REFRESH);
        if stale {
            self.shots = list_shots(instances_dir);
            self.shots_at = Some(std::time::Instant::now());
        }
        let mut close = false;
        let mut action = ScreensAction::None;
        let mut deleted: Vec<PathBuf> = Vec::new();
        egui::Window::new(lang.screens_title())
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(640.0)
            .default_size(vec2(680.0, 480.0))
            .show(ctx, |ui| {
                if self.shots.is_empty() {
                    ui.label(egui::RichText::new(lang.screens_empty()).color(TEXT_MUTED));
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                            for i in 0..self.shots.len() {
                                let path = self.shots[i].clone();
                                ui.vertical(|ui| {
                                    ui.set_max_width(200.0);
                                    let tex = self.thumb(ui.ctx(), &path);
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
                                        let _ = open::that(&path);
                                    }
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    ui.horizontal(|ui| {
                                        let mut name = path
                                            .file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_default();
                                        if name.chars().count() > 22 {
                                            name = format!(
                                                "{}…",
                                                name.chars().take(22).collect::<String>()
                                            );
                                        }
                                        ui.label(
                                            egui::RichText::new(name)
                                                .font(egui::FontId::proportional(10.0))
                                                .color(TEXT_MUTED),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .button(
                                                        egui::RichText::new("🗑").color(TEXT_MUTED),
                                                    )
                                                    .clicked()
                                                    && std::fs::remove_file(&path).is_ok()
                                                {
                                                    self.thumbs.remove(&path);
                                                    self.failed.remove(&path);
                                                    deleted.push(path);
                                                }
                                            },
                                        );
                                    });
                                });
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
        // Evict deleted shots from the cached list immediately.
        for gone in deleted {
            self.shots.retain(|p| p != &gone);
            if action == ScreensAction::None {
                action = ScreensAction::Deleted(gone);
            }
        }
        action
    }

    fn thumb(&mut self, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
        if let Some(tex) = self.thumbs.get(path) {
            return Some(tex.clone());
        }
        if self.failed.contains(path) {
            return None;
        }
        let bytes = std::fs::read(path).ok()?;
        if bytes.is_empty() || bytes.len() > MAX_SHOT_BYTES {
            self.failed.insert(path.to_path_buf());
            return None;
        }
        let img = image::load_from_memory(&bytes).ok()?;
        // Downscale for a light gallery; aspect preserved.
        let small = img.thumbnail(SHOT_THUMB_W, SHOT_THUMB_H);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_instances(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "amc_shots_test_{tag}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("inst").join("screenshots")).unwrap();
        dir
    }

    #[test]
    fn test_list_shots_filters_extensions() {
        let root = temp_instances("ext");
        let shots = root.join("inst").join("screenshots");
        std::fs::write(shots.join("a.png"), b"fake").unwrap();
        std::fs::write(shots.join("b.jpg"), b"fake").unwrap();
        std::fs::write(shots.join("c.txt"), b"fake").unwrap();
        std::fs::write(shots.join("noext"), b"fake").unwrap();
        let listed = list_shots(&root);
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|p| {
            p.extension()
                .map(|e| e == "png" || e == "jpg")
                .unwrap_or(false)
        }));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_list_shots_missing_dir_is_empty() {
        let missing = std::env::temp_dir().join("amc_shots_test_missing_dir_xyz");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(list_shots(&missing).is_empty());
    }
}
