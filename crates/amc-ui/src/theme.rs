use egui::{Color32, Context, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x08, 0x06, 0x06);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x0F, 0x0B, 0x0B);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x11, 0x0A, 0x0A);

pub const RUBY: Color32 = Color32::from_rgb(0x8B, 0x1A, 0x2A);
pub const RUBY_LIGHT: Color32 = Color32::from_rgb(0xB5, 0x22, 0x39);
pub const RUBY_DIM: Color32 = Color32::from_rgb(0x5C, 0x10, 0x19);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE8, 0xDA, 0xDA);
pub const TEXT_HEADING: Color32 = Color32::from_rgb(0xD4, 0xC4, 0xBB);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 25);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 40);
pub const BORDER_STRONG: Color32 = Color32::from_rgba_premultiplied(139, 26, 42, 65);

pub const SUCCESS: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const BADGE_SNAPSHOT: Color32 = Color32::from_rgb(0x70, 0x90, 0xC8);
pub const BADGE_BETA: Color32 = Color32::from_rgb(0x50, 0xB0, 0x50);
pub const BADGE_ALPHA: Color32 = Color32::from_rgb(0xC0, 0x7A, 0x30);
pub const BADGE_OLD: Color32 = Color32::from_rgb(0x5A, 0x4A, 0x44);

pub fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Check if custom fonts exist in assets/fonts/
    let fonts_dir = std::path::Path::new("assets/fonts");
    let segoe_path = fonts_dir.join("SegoeUI.ttf");
    let segoe_bold_path = fonts_dir.join("SegoeUI-Bold.ttf");
    let jetbrains_path = fonts_dir.join("JetBrainsMono-Regular.ttf");

    if segoe_path.is_file() {
        if let Ok(data) = std::fs::read(&segoe_path) {
            fonts.font_data.insert("segoe".to_owned(), egui::FontData::from_owned(data));
            fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "segoe".to_owned());
        }
    }

    if segoe_bold_path.is_file() {
        if let Ok(data) = std::fs::read(&segoe_bold_path) {
            fonts.font_data.insert("segoe_bold".to_owned(), egui::FontData::from_owned(data));
        }
    }

    if jetbrains_path.is_file() {
        if let Ok(data) = std::fs::read(&jetbrains_path) {
            fonts.font_data.insert("jetbrains".to_owned(), egui::FontData::from_owned(data));
            fonts.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "jetbrains".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_aleph_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG;
    visuals.window_fill = BG_ELEVATED;
    visuals.window_stroke = Stroke::new(1.0, BORDER_DEFAULT);
    visuals.window_rounding = Rounding::ZERO;

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

    visuals.widgets.active.bg_fill = RUBY;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, RUBY_LIGHT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::ZERO;

    visuals.selection.bg_fill = RUBY_DIM;
    visuals.selection.stroke = Stroke::new(1.0, RUBY);

    ctx.set_visuals(visuals);

    let mut style: Style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);

    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_style(style);
}
