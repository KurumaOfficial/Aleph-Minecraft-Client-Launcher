use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, Style,
    TextStyle, Visuals,
};

pub const BG: Color32 = Color32::from_rgb(0x08, 0x06, 0x06);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x0F, 0x0B, 0x0B);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x14, 0x0D, 0x0E);
pub const BG_CARD: Color32 = Color32::from_rgb(0x0C, 0x08, 0x09);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x18, 0x0C, 0x0E);
pub const BG_INPUT: Color32 = Color32::from_rgb(0x0A, 0x07, 0x08);

pub const RUBY: Color32 = Color32::from_rgb(0x8B, 0x1A, 0x2A);
pub const RUBY_LIGHT: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
pub const RUBY_DIM: Color32 = Color32::from_rgb(0x5C, 0x10, 0x19);
pub const RUBY_SOFT: Color32 = Color32::from_rgb(0x16, 0x0A, 0x0C);
pub const RUBY_GLOW: Color32 = Color32::from_rgba_premultiplied(181, 34, 57, 30);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE8, 0xDA, 0xDA);
pub const TEXT_HEADING: Color32 = Color32::from_rgb(0xD4, 0xC4, 0xBB);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x7C, 0x6B, 0x64);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 28);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 45);
pub const BORDER_STRONG: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 75);
pub const BORDER_ACCENT: Color32 = Color32::from_rgba_premultiplied(181, 34, 57, 100);

pub const SUCCESS: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const WARNING: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);
pub const INFO: Color32 = Color32::from_rgb(0x70, 0x90, 0xC8);

pub const BADGE_SNAPSHOT: Color32 = Color32::from_rgb(0x70, 0x90, 0xC8);
pub const BADGE_BETA: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const BADGE_ALPHA: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);
pub const BADGE_OLD: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

/// Linear interpolation between two colors, zero allocations, optimized for real-time UI transitions.
#[inline]
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t).round() as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t).round() as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t).round() as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t).round() as u8,
    )
}

pub fn setup_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "segoe".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/SegoeUI.ttf")),
    );
    fonts.font_data.insert(
        "segoe_bold".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/SegoeUI-Bold.ttf")),
    );
    fonts.font_data.insert(
        "unbounded_bold".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/Unbounded-Bold.ttf")),
    );
    fonts.font_data.insert(
        "jetbrains".to_owned(),
        FontData::from_static(include_bytes!("../../../assets/fonts/JetBrainsMono-Medium.ttf")),
    );

    // Primary proportional font
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "segoe".to_owned());

    // Monospace font
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "jetbrains".to_owned());

    ctx.set_fonts(fonts);
}

pub fn apply_aleph_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG;
    visuals.window_fill = BG_ELEVATED;
    visuals.extreme_bg_color = BG_INPUT;
    visuals.faint_bg_color = RUBY_SOFT;
    visuals.window_stroke = Stroke::new(1.0, BORDER_DEFAULT);
    visuals.window_rounding = Rounding::ZERO;
    visuals.menu_rounding = Rounding::ZERO;

    visuals.widgets.noninteractive.bg_fill = BG_ELEVATED;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.noninteractive.rounding = Rounding::ZERO;

    visuals.widgets.inactive.bg_fill = BG_ELEVATED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::ZERO;

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, RUBY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.rounding = Rounding::ZERO;

    visuals.widgets.active.bg_fill = RUBY_DIM;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, RUBY_LIGHT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::ZERO;

    visuals.widgets.open.bg_fill = BG_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, RUBY);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, TEXT_HEADING);
    visuals.widgets.open.rounding = Rounding::ZERO;

    visuals.selection.bg_fill = RUBY_DIM;
    visuals.selection.stroke = Stroke::new(1.0, RUBY);
    visuals.hyperlink_color = RUBY_LIGHT;
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.clip_rect_margin = 0.0;

    ctx.set_visuals(visuals);

    let mut style: Style = (*ctx.style()).clone();
    style.animation_time = 0.15; // 150ms smooth transition matching aleph.icu --duration-fast
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 2.0;

    style.text_styles = [
        (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_style(style);
}
