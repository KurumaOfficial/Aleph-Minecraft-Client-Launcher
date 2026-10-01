//! Widgets ported 1:1 from the Full reference prototype (look and metrics).
//! Only cache-free widgets live here; SVG/PNG-backed ones stay out.

use crate::theme;
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Response, Rounding, Sense, Stroke, Ui};

pub fn text_width(ui: &Ui, text: &str, font: &FontId) -> f32 {
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    })
}

pub fn fit_text(ui: &Ui, text: &str, font: &FontId, max_w: f32) -> String {
    if text_width(ui, text, font) <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        out.push(ch);
        if text_width(ui, &format!("{out}…"), font) > max_w {
            out.pop();
            break;
        }
    }
    format!("{out}…")
}

pub fn kicker(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.label(
            egui::RichText::new("■")
                .font(theme::mono(10.0))
                .color(theme::ACCENT),
        );
        ui.label(
            egui::RichText::new(text.to_uppercase())
                .font(theme::mono(11.0))
                .color(theme::ACCENT_LIGHT),
        );
    });
}

pub fn section_head(ui: &mut Ui, label: &str, title: &str) {
    kicker(ui, label);
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(title)
            .font(theme::display_reg(34.0))
            .color(theme::HEAD),
    );
    ui.add_space(6.0);
}

pub fn nav_link(ui: &mut Ui, label: &str, active: bool) -> Response {
    let font = theme::mono(11.0);
    let w = text_width(ui, &label.to_uppercase(), &font) + 4.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 40.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("nav", label)), response.hovered(), 0.15);
    let painter = ui.painter().clone();
    let color = if active {
        theme::HEAD
    } else {
        theme::lerp_color(theme::MUTED, theme::HEAD, hover)
    };
    painter.text(
        rect.center_top() + vec2(0.0, 12.0),
        Align2::CENTER_TOP,
        label.to_uppercase(),
        font,
        color,
    );
    if active {
        painter.rect_filled(
            Rect::from_min_max(
                pos2(rect.center().x - w * 0.3, rect.bottom() - 6.0),
                pos2(rect.center().x + w * 0.3, rect.bottom() - 4.0),
            ),
            Rounding::ZERO,
            theme::ACCENT,
        );
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn red_button(ui: &mut Ui, label: &str, width: f32, height: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("red", label)), response.hovered(), 0.15);
    let pressed = response.is_pointer_button_down_on();
    let painter = ui.painter().clone();
    let fill = if pressed {
        theme::ACCENT_DIM
    } else {
        theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover)
    };
    painter.rect_filled(rect, Rounding::ZERO, fill);
    if hover > 0.02 && !pressed {
        painter.rect_stroke(
            Rect::from_min_max(rect.min - vec2(3.0, 3.0), rect.max + vec2(3.0, 3.0)),
            Rounding::ZERO,
            Stroke::new(
                1.0_f32,
                theme::with_alpha(theme::ACCENT_LIGHT, (40.0 * hover) as u8),
            ),
        );
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        fit_text(
            ui,
            &label.to_uppercase(),
            &theme::display(11.0),
            width - 28.0,
        ),
        theme::display(11.0),
        Color32::WHITE,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn outline_button(ui: &mut Ui, label: &str, width: f32, height: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("out", label)), response.hovered(), 0.15);
    let painter = ui.painter().clone();
    if hover > 0.01 {
        painter.rect_filled(
            rect,
            Rounding::ZERO,
            theme::with_alpha(theme::HEAD, (14.0 * hover) as u8),
        );
    }
    painter.rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(
            1.0_f32,
            theme::lerp_color(theme::BORDER, theme::HEAD, hover * 0.5),
        ),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        theme::display(11.0),
        theme::lerp_color(theme::MUTED, theme::HEAD, hover),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn ghost_link(ui: &mut Ui, label: &str) -> Response {
    let font = theme::mono(11.0);
    let w = text_width(ui, &label.to_uppercase(), &font) + 2.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("ghost", label)), response.hovered(), 0.15);
    let painter = ui.painter().clone();
    painter.text(
        rect.left_top(),
        Align2::LEFT_TOP,
        label.to_uppercase(),
        font,
        theme::lerp_color(theme::MUTED, theme::HEAD, hover),
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(rect.left(), rect.bottom() - 5.0),
            pos2(
                rect.left() + w * (0.5 + 0.5 * hover.max(0.15)),
                rect.bottom() - 4.0,
            ),
        ),
        Rounding::ZERO,
        theme::lerp_color(theme::ACCENT_DIM, theme::ACCENT, hover),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn tag_width(ui: &mut Ui, label: &str, active: bool, width: f32) -> Response {
    let font = theme::mono(10.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 30.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("tagw", label)), response.hovered(), 0.12);
    let painter = ui.painter();
    let color = if active {
        theme::ACCENT_LIGHT
    } else {
        theme::lerp_color(theme::MUTED, theme::HEAD, hover)
    };
    if active {
        painter.rect_filled(rect, Rounding::ZERO, theme::ACCENT_SOFT);
    }
    painter.rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, if active { theme::ACCENT } else { theme::BORDER }),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        font,
        color,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn tag(ui: &mut Ui, label: &str, active: bool) -> Response {
    let font = theme::mono(10.0);
    let w = text_width(ui, &label.to_uppercase(), &font) + 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(w, 30.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("tag", label)), response.hovered(), 0.12);
    let painter = ui.painter();
    let color = if active {
        theme::ACCENT_LIGHT
    } else {
        theme::lerp_color(theme::MUTED, theme::HEAD, hover)
    };
    if active {
        painter.rect_filled(rect, Rounding::ZERO, theme::ACCENT_SOFT);
    }
    painter.rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, if active { theme::ACCENT } else { theme::BORDER }),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        font,
        color,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Cover-fit image paint (aspect-preserving crop).
pub fn image_cover(
    painter: &egui::Painter,
    rect: Rect,
    texture: &egui::TextureHandle,
    tint: Color32,
) {
    let source = texture.size_vec2();
    if source.x <= 0.0 || source.y <= 0.0 {
        return;
    }
    let dest_aspect = rect.width() / rect.height().max(1.0);
    let source_aspect = source.x / source.y;
    let uv = if source_aspect > dest_aspect {
        let span = dest_aspect / source_aspect;
        let pad = (1.0 - span) * 0.5;
        Rect::from_min_max(pos2(pad, 0.0), pos2(1.0 - pad, 1.0))
    } else {
        let span = source_aspect / dest_aspect;
        let pad = (1.0 - span) * 0.5;
        Rect::from_min_max(pos2(0.0, pad), pos2(1.0, 1.0 - pad))
    };
    painter.image(texture.id(), rect, uv, tint);
}

/// Thin double ruby square decoration for hero art.
pub fn red_square_deco(painter: &egui::Painter, rect: Rect) {
    painter.rect_stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0_f32, theme::with_alpha(theme::ACCENT, 56)),
    );
    let inner = rect.shrink(40.0);
    if inner.width() > 20.0 && inner.height() > 20.0 {
        painter.rect_stroke(
            inner,
            Rounding::ZERO,
            Stroke::new(1.0_f32, theme::with_alpha(theme::ACCENT, 20)),
        );
    }
}

pub fn card<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let width = ui.available_width();
    egui::Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1.0_f32, theme::BORDER_SOFT))
        .rounding(Rounding::ZERO)
        .inner_margin(egui::Margin::symmetric(20.0, 18.0))
        .show(ui, |ui| {
            ui.set_width((width - 40.0).max(80.0));
            add_contents(ui)
        })
        .inner
}
