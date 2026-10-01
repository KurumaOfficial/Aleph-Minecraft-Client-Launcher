//! Aleph Studio visual system, 1:1 with the Full reference prototype.
//!
//! Canonical Full tokens (`BG`, `CARD`, `ACCENT`, …) are the source of truth.
//! Legacy `BG_*` / `RUBY_*` / `TEXT_*` / `BORDER_*` names are kept as exact
//! aliases so existing widgets keep compiling untouched.

use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle,
    Visuals,
};

// ---------------------------------------------------------------------------
// Full canonical palette
// ---------------------------------------------------------------------------

pub const BG: Color32 = Color32::from_rgb(0x08, 0x06, 0x06);
pub const CARD: Color32 = Color32::from_rgb(0x0F, 0x0B, 0x0B);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x16, 0x0D, 0x0E);
pub const FIELD: Color32 = Color32::from_rgb(0x0A, 0x07, 0x08);

pub const BORDER: Color32 = Color32::from_rgb(0x1C, 0x09, 0x0B);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x15, 0x08, 0x0A);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x29, 0x0B, 0x0F);

pub const TEXT: Color32 = Color32::from_rgb(0xE8, 0xDA, 0xDA);
pub const HEAD: Color32 = Color32::from_rgb(0xD4, 0xC4, 0xBB);
pub const MUTED: Color32 = Color32::from_rgb(0x7C, 0x6B, 0x64);
pub const MUTED_DIM: Color32 = Color32::from_rgb(0x57, 0x46, 0x3F);

pub const ACCENT: Color32 = Color32::from_rgb(0x8B, 0x1A, 0x2A);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x5C, 0x10, 0x19);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x16, 0x0A, 0x0C);

pub const DANGER: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
pub const SUCCESS: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const WARN: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);
pub const INFO: Color32 = Color32::from_rgb(0x70, 0x90, 0xC8);

// ---------------------------------------------------------------------------
// Legacy aliases (same values, kept for existing widgets)
// ---------------------------------------------------------------------------

pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x0F, 0x0B, 0x0B);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x16, 0x0D, 0x0E);
pub const BG_CARD: Color32 = Color32::from_rgb(0x0C, 0x08, 0x09);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x18, 0x0C, 0x0E);
pub const BG_INPUT: Color32 = Color32::from_rgb(0x0A, 0x07, 0x08);

pub const RUBY: Color32 = Color32::from_rgb(0x8B, 0x1A, 0x2A);
pub const RUBY_LIGHT: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
pub const RUBY_DIM: Color32 = Color32::from_rgb(0x5C, 0x10, 0x19);
pub const RUBY_SOFT: Color32 = Color32::from_rgb(0x16, 0x0A, 0x0C);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE8, 0xDA, 0xDA);
pub const TEXT_HEADING: Color32 = Color32::from_rgb(0xD4, 0xC4, 0xBB);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x7C, 0x6B, 0x64);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 28);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 45);
pub const BORDER_ACCENT: Color32 = Color32::from_rgba_premultiplied(181, 34, 57, 100);

pub const WARNING: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);

pub const BADGE_SNAPSHOT: Color32 = Color32::from_rgb(0x70, 0x90, 0xC8);
pub const BADGE_BETA: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const BADGE_ALPHA: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);
pub const BADGE_OLD: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linear interpolation between two colors (straight alpha, Full 1:1).
#[inline]
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Unbounded Bold display face (Full `display`).
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("display".into()))
}

/// Unbounded Regular display face (Full `displayreg`).
pub fn display_reg(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("displayreg".into()))
}

/// JetBrains Mono face (Full `mono`, Segoe fallback keeps Cyrillic readable).
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("mono".into()))
}

/// Cormorant Garamond face (Full `serif`).
pub fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("serif".into()))
}

/// Cormorant Garamond Italic face (Full `serifit`).
pub fn serif_it(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("serifit".into()))
}

pub fn setup_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "unbounded-bold".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/Unbounded-Bold.ttf")),
    );
    fonts.font_data.insert(
        "unbounded".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/Unbounded-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "jb-mono".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Medium.ttf"
        )),
    );
    fonts.font_data.insert(
        "jb-mono-reg".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "cormorant".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/CormorantGaramond-Medium.ttf"
        )),
    );
    fonts.font_data.insert(
        "cormorant-sb".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/CormorantGaramond-SemiBold.ttf"
        )),
    );
    fonts.font_data.insert(
        "cormorant-it".to_owned(),
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/CormorantGaramond-Italic.ttf"
        )),
    );
    fonts.font_data.insert(
        "ui".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/SegoeUI.ttf")),
    );
    fonts.font_data.insert(
        "ui-bold".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/SegoeUI-Bold.ttf")),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "ui".to_owned());
    fonts.families.insert(
        FontFamily::Name("bold".into()),
        vec!["ui-bold".to_owned(), "ui".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("display".into()),
        vec![
            "unbounded-bold".to_owned(),
            "ui-bold".to_owned(),
            "ui".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Name("displayreg".into()),
        vec!["unbounded".to_owned(), "ui".to_owned()],
    );
    // NOTE: Full ships mono without a UI fallback; Segoe is appended so
    // Cyrillic labels never render as tofu boxes.
    fonts.families.insert(
        FontFamily::Name("mono".into()),
        vec![
            "jb-mono".to_owned(),
            "jb-mono-reg".to_owned(),
            "ui".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Name("serif".into()),
        vec![
            "cormorant".to_owned(),
            "cormorant-sb".to_owned(),
            "ui".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Name("serifit".into()),
        vec![
            "cormorant-it".to_owned(),
            "cormorant".to_owned(),
            "ui".to_owned(),
        ],
    );

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "jb-mono".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(1, "jb-mono-reg".to_owned());
    ctx.set_fonts(fonts);
}

pub fn apply_aleph_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.animation_time = 0.15;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.interact_size = egui::vec2(28.0, 26.0);
    style.spacing.scroll.bar_width = 6.0;

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.extreme_bg_color = FIELD;
    visuals.faint_bg_color = ACCENT_SOFT;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.window_rounding = Rounding::ZERO;
    visuals.menu_rounding = Rounding::ZERO;
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 12.0),
        blur: 32.0,
        spread: 0.0,
        color: Color32::from_black_alpha(160),
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.selection.bg_fill = with_alpha(ACCENT, 80);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT_LIGHT);
    visuals.hyperlink_color = ACCENT_LIGHT;
    visuals.override_text_color = Some(TEXT);
    visuals.striped = false;

    let card_stroke = Stroke::new(1.0_f32, BORDER_SOFT);
    let accent_stroke = Stroke::new(1.0_f32, BORDER);
    for widgets in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.open,
    ] {
        widgets.bg_fill = CARD;
        widgets.weak_bg_fill = FIELD;
        widgets.bg_stroke = card_stroke;
        widgets.fg_stroke = Stroke::new(1.0_f32, TEXT);
        widgets.rounding = Rounding::ZERO;
    }
    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = accent_stroke;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, HEAD);
    visuals.widgets.hovered.rounding = Rounding::ZERO;

    visuals.widgets.active.bg_fill = ACCENT_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, ACCENT_LIGHT);
    visuals.widgets.active.rounding = Rounding::ZERO;

    style.visuals = visuals;

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Name("display".into())),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(16.0, FontFamily::Name("serif".into())),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(12.0, FontFamily::Name("mono".into())),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Name("mono".into())),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );

    ctx.set_style(style);
}
