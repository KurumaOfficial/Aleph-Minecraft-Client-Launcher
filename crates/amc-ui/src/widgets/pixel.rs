//! Pixel/cut-corner primitives shared by the Millida-inspired surfaces.

use egui::{pos2, Color32, Painter, Rect, Rounding};

/// Fill a rectangle with square corners cut by `cut` pixels.
pub fn rect(painter: &Painter, rect: Rect, cut: f32, fill: Color32) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let c = cut
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5)
        .max(0.0);
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + c, rect.top()),
            pos2(rect.right() - c, rect.top() + c),
        ),
        Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left(), rect.top() + c),
            pos2(rect.right(), rect.bottom() - c),
        ),
        Rounding::ZERO,
        fill,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + c, rect.bottom() - c),
            pos2(rect.right() - c, rect.bottom()),
        ),
        Rounding::ZERO,
        fill,
    );
}

/// Cut-corner plate with a one-pixel outline.
pub fn plate(painter: &Painter, area: Rect, cut: f32, fill: Color32, outline: Color32) {
    rect(painter, area, cut, outline);
    rect(painter, area.shrink(1.0), cut, fill);
}
