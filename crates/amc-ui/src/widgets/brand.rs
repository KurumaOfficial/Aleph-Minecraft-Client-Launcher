//! AMC Launcher identity marks.
//!
//! The product name and the hollow square are intentionally shared by every
//! shell. Layouts may follow their references, but the launcher is always
//! presented as AMC Launcher rather than carrying another launcher's brand.

use egui::{Color32, Painter, Rect, Rounding};

/// The fixed brand red of the AMC square (sampled from `assets/icon.png`).
pub const BRAND_RED: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
/// User-facing product name.
pub const APP_NAME: &str = "AMC Launcher";
/// Uppercase lockup text used by compact shell chrome.
pub const APP_NAME_UPPER: &str = "AMC LAUNCHER";

/// Draw the hollow square mark with a transparent centre. Four strips avoid
/// painting an opaque background into the hole, so the mark works on light,
/// dark and gradient surfaces alike.
pub fn hollow_square(painter: &Painter, rect: Rect, color: Color32) {
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }
    let thickness = (rect.width() * 0.17)
        .clamp(1.5, 7.0)
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0);
    let inner_h = (rect.height() - thickness * 2.0).max(0.0);
    painter.rect_filled(
        Rect::from_min_size(rect.min, egui::vec2(rect.width(), thickness)),
        Rounding::ZERO,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(
            egui::pos2(rect.left(), rect.bottom() - thickness),
            egui::vec2(rect.width(), thickness),
        ),
        Rounding::ZERO,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + thickness),
            egui::vec2(thickness, inner_h),
        ),
        Rounding::ZERO,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(
            egui::pos2(rect.right() - thickness, rect.top() + thickness),
            egui::vec2(thickness, inner_h),
        ),
        Rounding::ZERO,
        color,
    );
}
