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
// Dynamic palettes
// ---------------------------------------------------------------------------
// Call sites keep using the same `theme::NAME` constants as before, but each
// constant is now a palette token: it resolves to the color of the active
// shell palette when it reaches the painter. Swapping the palette repaints
// every shared page, so a shell can reproduce its reference look (gray/green
// Modrinth, light MultiMC, dark-blue LabyMod, …) without forking pages.

/// Palette token; its public names are the `theme::` color constants below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellColor {
    Bg,
    Card,
    CardHover,
    Field,
    BgCard,
    BgActive,
    Border,
    BorderSoft,
    BorderStrong,
    BorderSubtle,
    BorderDefault,
    BorderAccent,
    Text,
    Head,
    Muted,
    MutedDim,
    TextDim,
    Accent,
    AccentLight,
    AccentDim,
    AccentSoft,
    Danger,
    Success,
    Warn,
    Info,
    Ruby,
    RubyLight,
    RubyDim,
    RubySoft,
    TextPrimary,
    TextHeading,
    TextMuted,
    BgElevated,
    BgHover,
    BgInput,
    Warning,
    BadgeSnapshot,
    BadgeBeta,
    BadgeAlpha,
    BadgeOld,
}

/// One shell style; selects both the palette tokens and the egui visuals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PaletteId {
    /// Aleph Studio (dark ruby-black, the default).
    #[default]
    Aleph,
    /// MultiMC / Prism (Qt light gray).
    Multimc,
    /// LabyMod launcher (dark navy surfaces, blue primary).
    Labymod,
    /// Millida (dark violet).
    Millida,
    /// Modrinth (gray surfaces, green accent).
    Modrinth,
}

/// Concrete colors of one palette.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// App canvas behind the central panel.
    pub bg: Color32,
    /// Cards / raised panels.
    pub card: Color32,
    /// Hover state of cards and rows.
    pub card_hover: Color32,
    /// Inset fields and inputs.
    pub field: Color32,
    /// Lower (deeper) surface used by sidebars.
    pub bg_card: Color32,
    /// Pressed/active surface.
    pub bg_active: Color32,
    /// Default hairline border.
    pub border: Color32,
    /// Softer border for inner cards.
    pub border_soft: Color32,
    /// Strong border for emphasized frames.
    pub border_strong: Color32,
    /// Accent-tinted subtle border.
    pub border_subtle: Color32,
    /// Accent-tinted default border.
    pub border_default: Color32,
    /// Accent-tinted strong border.
    pub border_accent: Color32,
    /// Primary text.
    pub text: Color32,
    /// Headings.
    pub head: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Faintest text (footer chrome).
    pub muted_dim: Color32,
    /// Dimmest text tier.
    pub text_dim: Color32,
    /// Primary accent (buttons, selection).
    pub accent: Color32,
    /// Accent hover/light.
    pub accent_light: Color32,
    /// Accent, disabled/launching.
    pub accent_dim: Color32,
    /// Accent-tinted soft surface.
    pub accent_soft: Color32,
    /// Error color.
    pub danger: Color32,
    /// Success color.
    pub success: Color32,
    /// Warning color.
    pub warn: Color32,
    /// Info color.
    pub info: Color32,
    /// Border color of hovered egui-native widgets (per-style hover accent).
    pub hover_stroke: Color32,
    /// Corner rounding used by shared widgets and pages.
    pub rounding: f32,
}

impl Palette {
    const fn resolve(self, token: ShellColor) -> Color32 {
        match token {
            ShellColor::Bg => self.bg,
            ShellColor::Card => self.card,
            ShellColor::CardHover => self.card_hover,
            ShellColor::Field => self.field,
            ShellColor::BgCard => self.bg_card,
            ShellColor::BgActive => self.bg_active,
            ShellColor::Border => self.border,
            ShellColor::BorderSoft => self.border_soft,
            ShellColor::BorderStrong => self.border_strong,
            ShellColor::BorderSubtle => self.border_subtle,
            ShellColor::BorderDefault => self.border_default,
            ShellColor::BorderAccent => self.border_accent,
            ShellColor::Text => self.text,
            ShellColor::Head => self.head,
            ShellColor::Muted => self.muted,
            ShellColor::MutedDim => self.muted_dim,
            ShellColor::TextDim => self.text_dim,
            ShellColor::Accent => self.accent,
            ShellColor::AccentLight => self.accent_light,
            ShellColor::AccentDim => self.accent_dim,
            ShellColor::AccentSoft => self.accent_soft,
            ShellColor::Danger => self.danger,
            ShellColor::Success => self.success,
            ShellColor::Warn => self.warn,
            ShellColor::Info => self.info,
            // Aliases share the canonical entries so a palette only defines
            // each color once.
            ShellColor::Ruby => self.accent,
            ShellColor::RubyLight => self.accent_light,
            ShellColor::RubyDim => self.accent_dim,
            ShellColor::RubySoft => self.accent_soft,
            ShellColor::TextPrimary => self.text,
            ShellColor::TextHeading => self.head,
            ShellColor::TextMuted => self.muted,
            ShellColor::BgElevated => self.card,
            ShellColor::BgHover => self.card_hover,
            ShellColor::BgInput => self.field,
            ShellColor::Warning => self.warn,
            ShellColor::BadgeSnapshot => self.info,
            ShellColor::BadgeBeta => self.success,
            ShellColor::BadgeAlpha => self.warn,
            ShellColor::BadgeOld => self.text_dim,
        }
    }
}

/// Unmultiplied color at a custom alpha (const-friendly for palette tables;
/// stored premultiplied like `Color32` does internally).
const fn tint(color: Color32, alpha: u8) -> Color32 {
    let a = alpha as u32;
    Color32::from_rgba_premultiplied(
        (color.r() as u32 * a / 255) as u8,
        (color.g() as u32 * a / 255) as u8,
        (color.b() as u32 * a / 255) as u8,
        alpha,
    )
}

// ---------------------------------------------------------------------------
// Palettes
// ---------------------------------------------------------------------------

/// Aleph Studio palette (dark ruby-black): the original AMC look.
pub const ALEPH: Palette = Palette {
    bg: Color32::from_rgb(0x08, 0x06, 0x06),
    card: Color32::from_rgb(0x0F, 0x0B, 0x0B),
    card_hover: Color32::from_rgb(0x16, 0x0D, 0x0E),
    field: Color32::from_rgb(0x0A, 0x07, 0x08),
    bg_card: Color32::from_rgb(0x0C, 0x08, 0x09),
    bg_active: Color32::from_rgb(0x18, 0x0C, 0x0E),
    border: Color32::from_rgb(0x1C, 0x09, 0x0B),
    border_soft: Color32::from_rgb(0x15, 0x08, 0x0A),
    border_strong: Color32::from_rgb(0x29, 0x0B, 0x0F),
    border_subtle: Color32::from_rgba_premultiplied(139, 26, 42, 28),
    border_default: Color32::from_rgba_premultiplied(139, 26, 42, 45),
    border_accent: Color32::from_rgba_premultiplied(181, 34, 57, 100),
    text: Color32::from_rgb(0xE8, 0xDA, 0xDA),
    head: Color32::from_rgb(0xD4, 0xC4, 0xBB),
    muted: Color32::from_rgb(0x7C, 0x6B, 0x64),
    muted_dim: Color32::from_rgb(0x57, 0x46, 0x3F),
    text_dim: Color32::from_rgb(0x5A, 0x4A, 0x44),
    accent: Color32::from_rgb(0x8B, 0x1A, 0x2A),
    accent_light: Color32::from_rgb(0xB5, 0x22, 0x39),
    accent_dim: Color32::from_rgb(0x5C, 0x10, 0x19),
    accent_soft: Color32::from_rgb(0x16, 0x0A, 0x0C),
    danger: Color32::from_rgb(0xB5, 0x22, 0x39),
    success: Color32::from_rgb(0x50, 0xB0, 0x50),
    warn: Color32::from_rgb(0xC0, 0x7A, 0x30),
    info: Color32::from_rgb(0x70, 0x90, 0xC8),
    hover_stroke: Color32::from_rgb(0x1C, 0x09, 0x0B),
    rounding: 0.0,
};

// ---------------------------------------------------------------------------
// Modrinth-style colors (green accent on gray surfaces)
// ---------------------------------------------------------------------------
// Values follow the public dark-theme CSS variables of the Modrinth app
// (`--surface-1` canvas, `--surface-3` raised rows, muted text, green
// `--brand`); only colors and layout ideas are borrowed, no assets or code.

/// Canvas (`--surface-1` #16181c).
pub const MODRINTH_BG: Color32 = Color32::from_rgb(0x16, 0x18, 0x1C);
/// Raised rows / cards (`--surface-3` #27292e).
pub const MODRINTH_SURFACE: Color32 = Color32::from_rgb(0x27, 0x29, 0x2E);
/// Buttons, inputs and hover rows (`--surface-4` #34363c).
pub const MODRINTH_SURFACE_HOVER: Color32 = Color32::from_rgb(0x34, 0x36, 0x3C);
/// Dividers and stronger outlines (`--surface-5` #42444a).
pub const MODRINTH_SURFACE_HIGH: Color32 = Color32::from_rgb(0x42, 0x44, 0x4A);
/// Inset fields / inputs (the reference uses the raised gray here too).
pub const MODRINTH_FIELD: Color32 = Color32::from_rgb(0x34, 0x36, 0x3C);
/// Primary text (`--color-base` #b0bac5).
pub const MODRINTH_TEXT: Color32 = Color32::from_rgb(0xB0, 0xBA, 0xC5);
/// Heading / contrast text (`--color-contrast` #ffffff).
pub const MODRINTH_TEXT_CONTRAST: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
/// Secondary text (`--color-secondary` #96a2b0).
pub const MODRINTH_TEXT_SECONDARY: Color32 = Color32::from_rgb(0x96, 0xA2, 0xB0);
/// Faint secondary text.
pub const MODRINTH_MUTED: Color32 = Color32::from_rgb(0x72, 0x7C, 0x88);
/// Brand green (`--brand`).
pub const MODRINTH_GREEN: Color32 = Color32::from_rgb(0x1B, 0xD9, 0x6A);
/// Brand green, hover/light.
pub const MODRINTH_GREEN_LIGHT: Color32 = Color32::from_rgb(0x43, 0xE4, 0x88);
/// Brand green, disabled/launching.
pub const MODRINTH_GREEN_DIM: Color32 = Color32::from_rgb(0x12, 0x8A, 0x45);

/// MultiMC / Prism palette (Qt Fusion "Bright" theme the launcher ships
/// with: window `#eff0f1`, base `#fcfcfc`, text `#31363b`, highlight
/// `#3daee9`, link `#2980b9`). Rounding is zero — Fusion draws sharp
/// rectangles.
const MULTIMC: Palette = Palette {
    bg: Color32::from_rgb(0xEF, 0xF0, 0xF1),
    card: Color32::from_rgb(0xEF, 0xF0, 0xF1),
    card_hover: Color32::from_rgb(0xDA, 0xEC, 0xF9),
    field: Color32::from_rgb(0xFC, 0xFC, 0xFC),
    bg_card: Color32::from_rgb(0xE6, 0xE6, 0xE6),
    bg_active: Color32::from_rgb(0xCF, 0xE8, 0xFA),
    border: Color32::from_rgb(0xB8, 0xB8, 0xB8),
    border_soft: Color32::from_rgb(0xDC, 0xDC, 0xDC),
    border_strong: Color32::from_rgb(0x9A, 0x9A, 0x9A),
    border_subtle: Color32::from_rgb(0xDC, 0xDC, 0xDC),
    border_default: Color32::from_rgb(0xAD, 0xAD, 0xAD),
    border_accent: Color32::from_rgb(0x3D, 0xAE, 0xE9),
    text: Color32::from_rgb(0x31, 0x36, 0x3B),
    head: Color32::from_rgb(0x31, 0x36, 0x3B),
    muted: Color32::from_rgb(0x5E, 0x5E, 0x5E),
    muted_dim: Color32::from_rgb(0x80, 0x80, 0x80),
    text_dim: Color32::from_rgb(0x6E, 0x6E, 0x6E),
    accent: Color32::from_rgb(0x3D, 0xAE, 0xE9),
    accent_light: Color32::from_rgb(0x66, 0xC1, 0xEF),
    accent_dim: Color32::from_rgb(0x29, 0x80, 0xB9),
    accent_soft: Color32::from_rgb(0xD9, 0xEE, 0xFB),
    danger: Color32::from_rgb(0xC0, 0x39, 0x2B),
    success: Color32::from_rgb(0x3C, 0x8E, 0x3C),
    warn: Color32::from_rgb(0xC0, 0x7A, 0x30),
    info: Color32::from_rgb(0x29, 0x80, 0xB9),
    hover_stroke: Color32::from_rgb(0x9A, 0x9A, 0x9A),
    rounding: 0.0,
};
/// LabyMod launcher palette (dark navy surfaces, blue primary button);
/// values follow the launcher's public Flathub screenshots.
const LABYMOD: Palette = Palette {
    bg: Color32::from_rgb(0x0B, 0x0F, 0x17),
    card: Color32::from_rgb(0x16, 0x1B, 0x25),
    card_hover: Color32::from_rgb(0x1D, 0x24, 0x31),
    field: Color32::from_rgb(0x10, 0x14, 0x1D),
    bg_card: Color32::from_rgb(0x12, 0x17, 0x21),
    bg_active: Color32::from_rgb(0x1A, 0x22, 0x33),
    border: Color32::from_rgb(0x2A, 0x31, 0x42),
    border_soft: Color32::from_rgb(0x20, 0x26, 0x33),
    border_strong: Color32::from_rgb(0x3A, 0x44, 0x5C),
    border_subtle: tint(Color32::from_rgb(0x3B, 0x82, 0xF6), 45),
    border_default: tint(Color32::from_rgb(0x3B, 0x82, 0xF6), 80),
    border_accent: tint(Color32::from_rgb(0x3B, 0x82, 0xF6), 150),
    text: Color32::from_rgb(0xE3, 0xE8, 0xF2),
    head: Color32::from_rgb(0xF5, 0xF7, 0xFB),
    muted: Color32::from_rgb(0x98, 0xA2, 0xB8),
    muted_dim: Color32::from_rgb(0x6E, 0x79, 0x90),
    text_dim: Color32::from_rgb(0x82, 0x8C, 0xA2),
    accent: Color32::from_rgb(0x3B, 0x82, 0xF6),
    accent_light: Color32::from_rgb(0x60, 0x9E, 0xFF),
    accent_dim: Color32::from_rgb(0x24, 0x58, 0xB8),
    accent_soft: Color32::from_rgb(0x16, 0x25, 0x42),
    danger: Color32::from_rgb(0xE0, 0x5A, 0x64),
    success: Color32::from_rgb(0x4C, 0xC3, 0x8A),
    warn: Color32::from_rgb(0xE0, 0xA3, 0x40),
    info: Color32::from_rgb(0x4F, 0x9C, 0xF9),
    hover_stroke: Color32::from_rgb(0x4B, 0x57, 0x70),
    rounding: 12.0,
};
/// Millida palette (dark charcoal with the mint-green accent). Values follow
/// the design tokens published in the launcher's public GPL source tree
/// (`--m-bg`, `--m-surface`, `--m-accent`, `--m-danger`, …).
const MILLIDA: Palette = Palette {
    bg: Color32::from_rgb(0x16, 0x17, 0x18),
    card: Color32::from_rgb(0x1E, 0x1F, 0x20),
    card_hover: Color32::from_rgb(0x26, 0x27, 0x2A),
    field: Color32::from_rgb(0x13, 0x13, 0x14),
    bg_card: Color32::from_rgb(0x13, 0x13, 0x14),
    bg_active: Color32::from_rgb(0x2A, 0x2B, 0x2E),
    border: tint(Color32::from_rgb(0xFF, 0xFF, 0xFF), 18),
    border_soft: tint(Color32::from_rgb(0xFF, 0xFF, 0xFF), 12),
    border_strong: tint(Color32::from_rgb(0xFF, 0xFF, 0xFF), 36),
    border_subtle: tint(Color32::from_rgb(0x5E, 0xC6, 0x4D), 33),
    border_default: tint(Color32::from_rgb(0x5E, 0xC6, 0x4D), 45),
    border_accent: tint(Color32::from_rgb(0x5E, 0xC6, 0x4D), 100),
    text: Color32::from_rgb(0xED, 0xEF, 0xEE),
    head: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    muted: tint(Color32::from_rgb(0xED, 0xEF, 0xEE), 204),
    muted_dim: tint(Color32::from_rgb(0xED, 0xEF, 0xEE), 168),
    text_dim: tint(Color32::from_rgb(0xED, 0xEF, 0xEE), 133),
    accent: Color32::from_rgb(0x5E, 0xC6, 0x4D),
    accent_light: Color32::from_rgb(0x70, 0xD5, 0x5F),
    accent_dim: Color32::from_rgb(0x40, 0x87, 0x2F),
    accent_soft: tint(Color32::from_rgb(0x5E, 0xC6, 0x4D), 33),
    danger: Color32::from_rgb(0xFF, 0x6B, 0x5E),
    success: Color32::from_rgb(0x5E, 0xC6, 0x4D),
    warn: Color32::from_rgb(0xF5, 0xAA, 0x38),
    info: Color32::from_rgb(0x6A, 0xA9, 0xE9),
    hover_stroke: tint(Color32::from_rgb(0x5E, 0xC6, 0x4D), 150),
    rounding: 8.0,
};

/// Modrinth palette: gray surfaces with the brand green as the accent.
const MODRINTH: Palette = Palette {
    bg: MODRINTH_BG,
    card: MODRINTH_SURFACE,
    card_hover: MODRINTH_SURFACE_HOVER,
    field: MODRINTH_FIELD,
    bg_card: Color32::from_rgb(0x12, 0x14, 0x17),
    bg_active: MODRINTH_SURFACE_HOVER,
    border: Color32::from_rgb(0x1E, 0x20, 0x25),
    border_soft: Color32::from_rgb(0x1A, 0x1C, 0x20),
    border_strong: tint(Color32::WHITE, 90),
    border_subtle: MODRINTH_SURFACE_HOVER,
    border_default: MODRINTH_SURFACE_HIGH,
    border_accent: tint(MODRINTH_GREEN, 100),
    text: MODRINTH_TEXT,
    head: MODRINTH_TEXT_CONTRAST,
    muted: MODRINTH_MUTED,
    muted_dim: Color32::from_rgb(0x51, 0x59, 0x63),
    text_dim: Color32::from_rgb(0x5A, 0x63, 0x6E),
    accent: MODRINTH_GREEN,
    accent_light: MODRINTH_GREEN_LIGHT,
    accent_dim: MODRINTH_GREEN_DIM,
    accent_soft: tint(MODRINTH_GREEN, 26),
    danger: Color32::from_rgb(0xFF, 0x49, 0x6B),
    success: MODRINTH_GREEN,
    warn: Color32::from_rgb(0xF5, 0xA5, 0x24),
    info: Color32::from_rgb(0x7D, 0xB5, 0xFF),
    hover_stroke: tint(Color32::WHITE, 70),
    rounding: 6.0,
};

const PALETTES: [Palette; 5] = [ALEPH, MULTIMC, LABYMOD, MILLIDA, MODRINTH];

use std::sync::atomic::{AtomicUsize, Ordering};

static ACTIVE_PALETTE: AtomicUsize = AtomicUsize::new(0);

/// Switch the palette every `theme::` color token resolves to; called
/// together with the matching `apply_*_theme` visuals when the shell changes.
pub fn set_palette(id: PaletteId) {
    ACTIVE_PALETTE.store(id as usize, Ordering::Relaxed);
}

/// Palette currently active.
pub fn palette_id() -> PaletteId {
    match ACTIVE_PALETTE.load(Ordering::Relaxed) {
        1 => PaletteId::Multimc,
        2 => PaletteId::Labymod,
        3 => PaletteId::Millida,
        4 => PaletteId::Modrinth,
        _ => PaletteId::Aleph,
    }
}

/// Concrete palette of `id`, independent of what is currently active (used
/// by the `apply_*_theme` visuals functions).
pub const fn palette_of(id: PaletteId) -> Palette {
    PALETTES[id as usize]
}

/// Corner rounding of the active palette; shared widgets and pages call this
/// instead of hardcoding `Rounding::ZERO` so softer styles round the whole
/// app while Aleph stays sharp.
pub fn rounding() -> Rounding {
    Rounding::same(active_palette().rounding)
}

fn active_palette() -> Palette {
    PALETTES[ACTIVE_PALETTE
        .load(Ordering::Relaxed)
        .min(PALETTES.len() - 1)]
}

// ---------------------------------------------------------------------------
// Public color tokens (resolve against the active palette)
// ---------------------------------------------------------------------------

pub const BG: ShellColor = ShellColor::Bg;
pub const CARD: ShellColor = ShellColor::Card;
pub const CARD_HOVER: ShellColor = ShellColor::CardHover;
pub const FIELD: ShellColor = ShellColor::Field;

pub const BORDER: ShellColor = ShellColor::Border;
pub const BORDER_SOFT: ShellColor = ShellColor::BorderSoft;
pub const BORDER_STRONG: ShellColor = ShellColor::BorderStrong;

pub const TEXT: ShellColor = ShellColor::Text;
pub const HEAD: ShellColor = ShellColor::Head;
pub const MUTED: ShellColor = ShellColor::Muted;
pub const MUTED_DIM: ShellColor = ShellColor::MutedDim;

pub const ACCENT: ShellColor = ShellColor::Accent;
pub const ACCENT_LIGHT: ShellColor = ShellColor::AccentLight;
pub const ACCENT_DIM: ShellColor = ShellColor::AccentDim;
pub const ACCENT_SOFT: ShellColor = ShellColor::AccentSoft;

pub const DANGER: ShellColor = ShellColor::Danger;
pub const SUCCESS: ShellColor = ShellColor::Success;
pub const WARN: ShellColor = ShellColor::Warn;
pub const INFO: ShellColor = ShellColor::Info;

// Legacy aliases (same tokens, kept for existing widgets).

pub const BG_ELEVATED: ShellColor = ShellColor::BgElevated;
pub const BG_HOVER: ShellColor = ShellColor::BgHover;
pub const BG_CARD: ShellColor = ShellColor::BgCard;
pub const BG_ACTIVE: ShellColor = ShellColor::BgActive;
pub const BG_INPUT: ShellColor = ShellColor::BgInput;

pub const RUBY: ShellColor = ShellColor::Ruby;
pub const RUBY_LIGHT: ShellColor = ShellColor::RubyLight;
pub const RUBY_DIM: ShellColor = ShellColor::RubyDim;
pub const RUBY_SOFT: ShellColor = ShellColor::RubySoft;

pub const TEXT_PRIMARY: ShellColor = ShellColor::TextPrimary;
pub const TEXT_HEADING: ShellColor = ShellColor::TextHeading;
pub const TEXT_MUTED: ShellColor = ShellColor::TextMuted;
pub const TEXT_DIM: ShellColor = ShellColor::TextDim;

pub const BORDER_SUBTLE: ShellColor = ShellColor::BorderSubtle;
pub const BORDER_DEFAULT: ShellColor = ShellColor::BorderDefault;
pub const BORDER_ACCENT: ShellColor = ShellColor::BorderAccent;

pub const WARNING: ShellColor = ShellColor::Warning;

pub const BADGE_SNAPSHOT: ShellColor = ShellColor::BadgeSnapshot;
pub const BADGE_BETA: ShellColor = ShellColor::BadgeBeta;
pub const BADGE_ALPHA: ShellColor = ShellColor::BadgeAlpha;
pub const BADGE_OLD: ShellColor = ShellColor::BadgeOld;

impl From<ShellColor> for Color32 {
    fn from(token: ShellColor) -> Self {
        active_palette().resolve(token)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linear interpolation between two colors (straight alpha, Full 1:1).
/// Accepts raw `Color32`s and palette tokens alike.
#[inline]
pub fn lerp_color(a: impl Into<Color32>, b: impl Into<Color32>, t: f32) -> Color32 {
    let (a, b) = (a.into(), b.into());
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// The same color at a different alpha; accepts tokens and raw colors.
pub fn with_alpha(color: impl Into<Color32>, alpha: u8) -> Color32 {
    let color = color.into();
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

    // Belt and braces: the bundled faces have no emoji coverage, so every
    // family ends with egui's emoji fonts. Chrome icons are painter-drawn
    // vectors; this only keeps stray glyphs in user text off tofu boxes
    // (e.g. a nickname pasted from a chat).
    for family in [
        FontFamily::Proportional,
        FontFamily::Monospace,
        FontFamily::Name("bold".into()),
        FontFamily::Name("display".into()),
        FontFamily::Name("displayreg".into()),
        FontFamily::Name("mono".into()),
        FontFamily::Name("serif".into()),
        FontFamily::Name("serifit".into()),
    ] {
        let chain = fonts.families.entry(family).or_default();
        for fallback in ["NotoEmoji-Regular", "emoji-icon-font"] {
            if !chain.iter().any(|name| name == fallback) {
                chain.push(fallback.to_owned());
            }
        }
    }

    ctx.set_fonts(fonts);
}

/// Spacing and typography shared by every palette; palette functions only
/// swap `style.visuals` after calling this.
fn apply_common_style(style: &mut egui::Style) {
    style.animation_time = 0.15;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.interact_size = egui::vec2(28.0, 26.0);
    style.spacing.scroll.bar_width = 6.0;

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
}

/// Apply the palette of the active shell. Call [`set_palette`] first, then
/// this, and every egui-native widget (scroll bars, text edits, tooltips,
/// windows) matches the tokens the shared pages draw with.
pub fn apply_active_theme(ctx: &Context) {
    let id = palette_id();
    let light = matches!(id, PaletteId::Multimc);
    let mut style = (*ctx.style()).clone();
    apply_common_style(&mut style);
    // Aleph's editorial body face is a serif; the four launcher replicas use
    // plain sans-serif chrome like their references. Without this, native
    // labels (empty states, hints, settings descriptions) silently inherit
    // Cormorant and look like a stray editorial font in Qt/Modrinth UI.
    if !matches!(id, PaletteId::Aleph) {
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    }
    style.visuals = visuals_from(&palette_of(id), light);
    ctx.set_style(style);
}

/// Aleph Studio theme (the studio default, dark ruby-black).
pub fn apply_aleph_theme(ctx: &Context) {
    set_palette(PaletteId::Aleph);
    apply_active_theme(ctx);
}

/// Modrinth-style theme entry point (gray surfaces, green accent).
pub fn apply_modrinth_theme(ctx: &Context) {
    set_palette(PaletteId::Modrinth);
    apply_active_theme(ctx);
}

/// Egui visuals derived from one concrete palette; `light` selects the light
/// baseline (MultiMC) instead of the dark one.
fn visuals_from(palette: &Palette, light: bool) -> Visuals {
    let mut visuals = if light {
        Visuals::light()
    } else {
        Visuals::dark()
    };
    let rounding = Rounding::same(palette.rounding);
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.card;
    visuals.extreme_bg_color = palette.field;
    visuals.faint_bg_color = palette.accent_soft;
    visuals.window_stroke = Stroke::new(1.0_f32, palette.border);
    visuals.window_rounding = rounding;
    visuals.menu_rounding = rounding;
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 12.0),
        blur: 32.0,
        spread: 0.0,
        color: Color32::from_black_alpha(if light { 90 } else { 160 }),
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.selection.bg_fill = with_alpha(palette.accent, 80);
    visuals.selection.stroke = Stroke::new(1.0_f32, palette.accent_light);
    visuals.hyperlink_color = if light {
        palette.accent_dim
    } else {
        palette.accent_light
    };
    visuals.override_text_color = Some(palette.text);
    visuals.striped = false;

    let card_stroke = Stroke::new(1.0_f32, palette.border_soft);
    let hover_stroke = Stroke::new(1.0_f32, palette.hover_stroke);
    for widgets in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.open,
    ] {
        widgets.bg_fill = palette.card;
        widgets.weak_bg_fill = palette.field;
        widgets.bg_stroke = card_stroke;
        widgets.fg_stroke = Stroke::new(1.0_f32, palette.text);
        widgets.rounding = rounding;
    }
    visuals.widgets.hovered.bg_fill = palette.card_hover;
    visuals.widgets.hovered.weak_bg_fill = palette.card_hover;
    visuals.widgets.hovered.bg_stroke = hover_stroke;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, palette.head);
    visuals.widgets.hovered.rounding = rounding;

    visuals.widgets.active.bg_fill = palette.accent_soft;
    visuals.widgets.active.weak_bg_fill = palette.accent_soft;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, palette.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, palette.accent_light);
    visuals.widgets.active.rounding = rounding;

    visuals
}
