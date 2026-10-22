//! Small controls shared by the Modrinth-inspired pages.
//!
//! The reference app uses the same rounded, high-contrast button family on
//! every screen. Keeping the drawing here means Profile, Settings and later
//! pages share one metric set instead of drifting apart.

use crate::theme::{
    self, lerp_color, with_alpha, MODRINTH_GREEN, MODRINTH_SURFACE_HIGH, MODRINTH_SURFACE_HOVER,
    MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST, MODRINTH_TEXT_SECONDARY,
};
use crate::widgets::icons::{self, Icon};
use egui::{pos2, vec2, Color32, FontFamily, FontId, Rect, Response, Rounding, Sense, Stroke, Ui};

/// Visual weight of a Modrinth button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModrinthButtonKind {
    /// Brand-green filled action (reference `Button type="colored"`).
    Primary,
    /// Raised gray button (reference `Button type="base"`).
    Secondary,
    /// Transparent quiet action used in rows and menus.
    Quiet,
}

impl ModrinthButtonKind {
    fn text_color(self, hover: f32) -> Color32 {
        match self {
            Self::Primary => theme::MODRINTH_BG,
            Self::Secondary => lerp_color(MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST, hover),
            Self::Quiet => lerp_color(MODRINTH_TEXT_SECONDARY, MODRINTH_TEXT_CONTRAST, hover),
        }
    }

    fn fill(self, hover: f32) -> Color32 {
        match self {
            Self::Primary => lerp_color(MODRINTH_GREEN, theme::MODRINTH_GREEN_LIGHT, hover),
            Self::Secondary => lerp_color(MODRINTH_SURFACE_HOVER, MODRINTH_SURFACE_HIGH, hover),
            Self::Quiet => with_alpha(MODRINTH_SURFACE_HOVER, (hover * 210.0) as u8),
        }
    }
}

/// Draw a Modrinth button with an optional leading icon. Width is measured
/// from the label so translated strings never clip.
pub fn button(
    ui: &mut Ui,
    label: &str,
    icon: Option<Icon>,
    kind: ModrinthButtonKind,
    min_width: f32,
) -> Response {
    let height = 48.0;
    let font = FontId::new(15.0, FontFamily::Name("bold".into()));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let icon_space = if icon.is_some() { 26.0 } else { 0.0 };
    let width = (galley.size().x + icon_space + 32.0).max(min_width);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    paint_button(ui, rect, &galley, icon, kind, hover, resp.enabled());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Compact icon-only action (copy, overflow, row trailing controls).
pub fn icon_button(ui: &mut Ui, icon: Icon, size: f32, tooltip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    if hover > 0.01 {
        ui.painter().rect_filled(
            rect,
            Rounding::same(8.0),
            with_alpha(MODRINTH_SURFACE_HOVER, (hover * 230.0) as u8),
        );
    }
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
        lerp_color(MODRINTH_TEXT_SECONDARY, MODRINTH_TEXT_CONTRAST, hover),
        1.4,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(tooltip)
}

/// Full-row Modrinth switch: label on the left, 40×22 track on the right.
pub fn toggle(ui: &mut Ui, value: &mut bool, label: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let t = ui
        .ctx()
        .animate_bool_responsive(resp.id.with("knob"), *value);
    let text = lerp_color(MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST, hover);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(13.0),
        Color32::PLACEHOLDER,
    );
    ui.painter().galley(
        pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
        galley,
        text,
    );

    let track =
        Rect::from_center_size(pos2(rect.right() - 21.0, rect.center().y), vec2(40.0, 22.0));
    let fill = if *value {
        lerp_color(MODRINTH_GREEN, theme::MODRINTH_GREEN_LIGHT, hover)
    } else {
        lerp_color(MODRINTH_SURFACE_HOVER, MODRINTH_SURFACE_HIGH, hover)
    };
    ui.painter().rect_filled(track, Rounding::same(11.0), fill);
    if !*value {
        ui.painter().rect_stroke(
            track,
            Rounding::same(11.0),
            Stroke::new(
                1.0_f32,
                with_alpha(MODRINTH_TEXT_CONTRAST, hover.mul_add(28.0, 30.0) as u8),
            ),
        );
    }
    let knob_x = (track.left() + 11.0) + (track.width() - 22.0) * t;
    let knob = pos2(knob_x, track.center().y);
    ui.painter().circle_filled(
        knob,
        if resp.hovered() { 9.0 } else { 8.0 },
        if *value {
            theme::MODRINTH_BG
        } else {
            MODRINTH_TEXT
        },
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        *value = !*value;
    }
    resp
}

/// Small rounded account-type/status pill.
pub fn pill(ui: &mut Ui, label: &str, accent: bool) -> Response {
    let font = FontId::new(11.0, FontFamily::Name("bold".into()));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let width = (galley.size().x + 22.0).max(54.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
    let (fill, stroke, text) = if accent {
        (
            with_alpha(MODRINTH_GREEN, 24),
            with_alpha(MODRINTH_GREEN, 90),
            MODRINTH_GREEN,
        )
    } else {
        (
            with_alpha(MODRINTH_SURFACE_HOVER, 190),
            MODRINTH_SURFACE_HIGH,
            MODRINTH_TEXT_SECONDARY,
        )
    };
    ui.painter().rect_filled(rect, Rounding::same(12.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(12.0), Stroke::new(1.0_f32, stroke));
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text,
    );
    resp
}

/// Horizontal hairline used between page sections.
pub fn divider(ui: &mut Ui) {
    let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(line, Rounding::ZERO, MODRINTH_SURFACE_HIGH);
}

fn paint_button(
    ui: &Ui,
    rect: Rect,
    galley: &std::sync::Arc<egui::Galley>,
    icon: Option<Icon>,
    kind: ModrinthButtonKind,
    hover: f32,
    enabled: bool,
) {
    let fill = if enabled {
        kind.fill(hover)
    } else {
        with_alpha(MODRINTH_SURFACE_HOVER, 120)
    };
    let text = if enabled {
        kind.text_color(hover)
    } else {
        with_alpha(MODRINTH_TEXT_SECONDARY, 110)
    };
    ui.painter().rect_filled(rect, Rounding::same(16.0), fill);
    if kind != ModrinthButtonKind::Primary {
        let stroke = if hover > 0.4 {
            with_alpha(MODRINTH_TEXT_CONTRAST, 55)
        } else {
            MODRINTH_SURFACE_HIGH
        };
        ui.painter()
            .rect_stroke(rect, Rounding::same(16.0), Stroke::new(1.0_f32, stroke));
    }

    let content_w = galley.size().x + if icon.is_some() { 26.0 } else { 0.0 };
    let left = rect.center().x - content_w / 2.0;
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(left + 10.0, rect.center().y), vec2(20.0, 20.0)),
            text,
            1.5,
        );
    }
    let text_x = left + if icon.is_some() { 26.0 } else { 0.0 };
    ui.painter().galley(
        pos2(text_x, rect.center().y - galley.size().y / 2.0),
        galley.clone(),
        text,
    );
}
