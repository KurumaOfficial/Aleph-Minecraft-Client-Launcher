//! Millida Launcher-inspired shell (layout only).
//!
//! Credits:
//! - Millida Launcher — Millida contributors, GPL-3.0 —
//!   <https://github.com/millida/launcher>
//!
//! Layout references studied for metrics only (no code or assets are reused):
//! - default layout: an edge-to-edge lobby — a pixel stage background, four
//!   196 px section tiles on the left (drawn at the reference's 0.78 zoom),
//!   the account/utility cluster in the top-right corner, a promo card on the
//!   right and the bottom dock (500 px mode card + 290 px Play column);
//! - classic layout: a 236 px navigation rail with 41 px items and the
//!   account card pinned to the bottom.
//!
//! AMC mapping: the lobby is the Home tab; its tiles and the account pill
//! open the shared pages, Play launches the selected instance, the corner
//! buttons toggle the console/download manager and open Settings. Other
//! pages keep the shared layout behind the classic rail. Pixel silhouettes
//! (5 px controls, 10 px panels) are drawn from plain rectangles; no
//! third-party artwork, fonts or code is reused.

use crate::pages::skins::Skin3d;
use crate::shells::{short_name, tab_label, ShellAction};
use crate::theme;
use crate::widgets::brand;
use crate::widgets::icons::{self, Icon};
use crate::widgets::NavTab;
use amc_core::Language;
use egui::{pos2, vec2, Align2, Color32, Rect, Response, Rounding, Sense, Stroke, Ui};
use image::RgbaImage;

/// The project page opened by the lobby promo card.
const PROJECT_URL: &str = "https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher";

/// Classic navigation rail width (reference: `.sidebar` 236 px).
pub const RAIL_WIDTH: f32 = 236.0;
const RAIL_ITEM_H: f32 = 41.0;
const RAIL_BRAND_H: f32 = 64.0;

/// Reference pixel language: controls cut one 5 px step, panels and cards 10.
const CUT_CONTROL: f32 = 5.0;
const CUT_PANEL: f32 = 10.0;

// Default lobby metrics (reference values with the 0.78 tile zoom applied).
const TILE_W: f32 = 153.0;
const TILE_H: f32 = 97.0;
const TILE_GAP: f32 = 9.4;
const DOCK_W: f32 = 812.0;
const DOCK_H: f32 = 141.0;
const DOCK_PAD: f32 = 8.0;
const MODE_W: f32 = 500.0;
const PLAY_W: f32 = 290.0;
const PLAY_H: f32 = 83.0;
const CHEST_H: f32 = 34.0;

/// Text on the mint Play button (`--m-on-accent` in the reference).
const ON_ACCENT: Color32 = Color32::from_rgb(0x06, 0x20, 0x0E);
const MODE_ART: Color32 = Color32::from_rgb(0x14, 0x35, 0x66);
const MODE_BLUE: Color32 = Color32::from_rgb(0x1F, 0x4F, 0x99);
const MODE_BLUE_HOVER: Color32 = Color32::from_rgb(0x26, 0x60, 0xB8);
const MODE_EMPTY: Color32 = Color32::from_rgb(0xB8, 0x76, 0x0D);
const MODE_EMPTY_HOVER: Color32 = Color32::from_rgb(0xCF, 0x8A, 0x15);

/// Classic rail entries. Profile is reached from the pinned account card, so
/// every shared page keeps exactly one entrance.
pub const RAIL_TABS: [NavTab; 5] = [
    NavTab::Home,
    NavTab::Modpacks,
    NavTab::Mods,
    NavTab::Skins,
    NavTab::Settings,
];

/// Section tiles of the lobby, in the reference's top-to-bottom order.
pub const TILE_TARGETS: [(NavTab, Icon, TileTone); 4] = [
    (NavTab::Modpacks, Icon::Package, TileTone::Ruby),
    (NavTab::Mods, Icon::Puzzle, TileTone::Ember),
    (NavTab::Skins, Icon::Diamond, TileTone::Sky),
    (NavTab::Settings, Icon::Gear, TileTone::Moss),
];

/// Section color of a lobby tile (solid plate, rim and bottom band).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileTone {
    Ruby,
    Ember,
    Sky,
    Moss,
}

#[derive(Debug, Clone, Copy)]
struct TileColors {
    base: Color32,
    hover: Color32,
    outline: Color32,
    band: Color32,
    ray: Color32,
}

impl TileTone {
    fn colors(self) -> TileColors {
        match self {
            Self::Ruby => TileColors {
                base: Color32::from_rgb(0xE0, 0x1F, 0x49),
                hover: Color32::from_rgb(0xE4, 0x3A, 0x5F),
                outline: Color32::from_rgb(0x44, 0x08, 0x15),
                band: Color32::from_rgb(0x77, 0x0D, 0x24),
                ray: Color32::from_rgba_unmultiplied(255, 170, 190, 72),
            },
            Self::Ember => TileColors {
                base: Color32::from_rgb(0xD2, 0x52, 0x1F),
                hover: Color32::from_rgb(0xE2, 0x65, 0x2E),
                outline: Color32::from_rgb(0x3D, 0x12, 0x05),
                band: Color32::from_rgb(0x7A, 0x2A, 0x0C),
                ray: Color32::from_rgba_unmultiplied(255, 196, 140, 72),
            },
            Self::Sky => TileColors {
                base: Color32::from_rgb(0x2F, 0x8F, 0xD6),
                hover: Color32::from_rgb(0x3D, 0xA0, 0xE8),
                outline: Color32::from_rgb(0x0A, 0x2A, 0x44),
                band: Color32::from_rgb(0x17, 0x50, 0x7D),
                ray: Color32::from_rgba_unmultiplied(170, 225, 255, 72),
            },
            Self::Moss => TileColors {
                base: Color32::from_rgb(0x3E, 0x87, 0x35),
                hover: Color32::from_rgb(0x47, 0x97, 0x3B),
                outline: Color32::from_rgb(0x11, 0x24, 0x0E),
                band: Color32::from_rgb(0x1D, 0x42, 0x1B),
                ray: Color32::from_rgba_unmultiplied(94, 198, 77, 76),
            },
        }
    }
}

/// Selected build shown by the lobby dock.
pub struct LobbyInstance<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub loader: &'a str,
    pub minutes: u64,
}

/// Everything the lobby needs from the app, copied out before the call so
/// the shell never touches launcher state directly.
pub struct LobbyCtx<'a> {
    pub instance: Option<LobbyInstance<'a>>,
    pub account: Option<&'a str>,
    pub launching: bool,
    /// Current skin image/slim flag; the lobby renders it on the pedestal.
    pub skin: Option<&'a RgbaImage>,
    pub slim: bool,
}

// ---------------------------------------------------------------------------
// Pixel silhouettes
// ---------------------------------------------------------------------------

/// Fill a rectangle whose four corners have a `cut`-sized square step carved
/// out (the reference `--px-5` / `--px-10` clip path). Three non-overlapping
/// strips keep translucent fills flat.
fn stepped_rect(painter: &egui::Painter, rect: Rect, cut: f32, fill: Color32) {
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

/// Layered frame from the reference: a dark silhouette of the same cut with
/// the plate inset by the outline width (the inner cut stays equal, so the
/// stepped line keeps its thickness on the corners).
fn stepped_plate(
    painter: &egui::Painter,
    rect: Rect,
    cut: f32,
    fill: Color32,
    outline: Color32,
    outline_w: f32,
) {
    if outline_w > 0.0 {
        stepped_rect(painter, rect, cut, outline);
        stepped_rect(painter, rect.shrink(outline_w), cut, fill);
    } else {
        stepped_rect(painter, rect, cut, fill);
    }
}

/// Top light and bottom shadow facets inside a plate (never crossing the cut
/// steps, so the silhouette stays clean).
fn stepped_facets(
    painter: &egui::Painter,
    rect: Rect,
    cut: f32,
    inset: f32,
    top: Color32,
    bottom: Color32,
) {
    let inner = rect.shrink(inset);
    if inner.width() <= cut * 2.0 + 2.0 || inner.height() < 8.0 {
        return;
    }
    if top.a() > 0 {
        painter.rect_filled(
            Rect::from_min_max(
                pos2(inner.left() + cut, inner.top()),
                pos2(inner.right() - cut, inner.top() + 2.0),
            ),
            Rounding::ZERO,
            top,
        );
    }
    if bottom.a() > 0 {
        painter.rect_filled(
            Rect::from_min_max(
                pos2(inner.left() + cut, inner.bottom() - 4.0),
                pos2(inner.right() - cut, inner.bottom()),
            ),
            Rounding::ZERO,
            bottom,
        );
    }
}

/// The lobby dock only cuts its top corners (its bottom sits on the window
/// edge), so it needs its own two-strip silhouette.
fn top_cut_plate(
    painter: &egui::Painter,
    rect: Rect,
    cut: f32,
    fill: Color32,
    outline: Color32,
    outline_w: f32,
) {
    let paint = |r: Rect, color: Color32| {
        let c = cut.min(r.width() * 0.5).min(r.height() * 0.5);
        painter.rect_filled(
            Rect::from_min_max(
                pos2(r.left() + c, r.top()),
                pos2(r.right() - c, r.top() + c),
            ),
            Rounding::ZERO,
            color,
        );
        painter.rect_filled(
            Rect::from_min_max(pos2(r.left(), r.top() + c), r.right_bottom()),
            Rounding::ZERO,
            color,
        );
    };
    paint(rect, outline);
    paint(rect.shrink(outline_w), fill);
}

/// Soft radial halo built from a triangle fan (vertex colours fade to zero
/// alpha at the rim), so atmospheric light reads as a gradient instead of
/// nested opaque circles.
fn radial_glow(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: Color32) {
    if radius <= 0.0 || color.a() == 0 {
        return;
    }
    let segments = 48;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, color);
    for index in 0..segments {
        let angle = index as f32 / segments as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(
            center + vec2(angle.cos(), angle.sin()) * radius,
            Color32::TRANSPARENT,
        );
    }
    for index in 1..=segments {
        let next = if index == segments { 1 } else { index + 1 };
        mesh.add_triangle(0, index as u32, next as u32);
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Pointer target with a pointing-hand cursor.
fn hit(ui: &mut Ui, rect: Rect, key: impl std::hash::Hash) -> Response {
    let resp = ui.interact(rect, ui.id().with(key), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

// ---------------------------------------------------------------------------
// Classic navigation rail
// ---------------------------------------------------------------------------

fn rail_icon(tab: NavTab) -> Icon {
    match tab {
        NavTab::Home => Icon::Play,
        NavTab::Modpacks => Icon::Package,
        NavTab::Mods => Icon::Puzzle,
        NavTab::Skins => Icon::Diamond,
        NavTab::Profile => Icon::Person,
        NavTab::Settings => Icon::Gear,
    }
}

fn rail_label(tab: NavTab, lang: Language) -> &'static str {
    match tab {
        NavTab::Home => lang.shell_rail_play(),
        NavTab::Modpacks => lang.shell_rail_builds(),
        NavTab::Mods => lang.shell_rail_catalog(),
        other => tab_label(other, lang),
    }
}

/// Small rail shortcut button (gallery, console, downloads).
fn rail_utility(
    ui: &mut Ui,
    painter: &egui::Painter,
    rect: Rect,
    key: &'static str,
    icon: Icon,
    tip: &str,
) -> Response {
    let resp = hit(ui, rect, key).on_hover_text(tip);
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    stepped_plate(
        painter,
        rect,
        CUT_CONTROL,
        theme::lerp_color(theme::CARD, theme::CARD_HOVER, hover),
        Color32::from_black_alpha(70),
        1.0,
    );
    icons::draw(
        painter,
        icon,
        Rect::from_center_size(rect.center(), vec2(15.0, 15.0)),
        theme::lerp_color(theme::MUTED, theme::HEAD, hover),
        1.4,
    );
    resp
}

/// Vintage rail used by every page except the lobby.
pub fn show_rail(
    ui: &mut Ui,
    lang: Language,
    current_tab: NavTab,
    account_name: Option<&str>,
) -> ShellAction {
    let mut action = ShellAction::default();
    let bar = ui.max_rect();
    let painter = ui.painter().clone();
    painter.rect_filled(bar, Rounding::ZERO, theme::CARD);
    painter.line_segment(
        [
            pos2(bar.right() - 1.0, bar.top()),
            pos2(bar.right() - 1.0, bar.bottom()),
        ],
        Stroke::new(1.0_f32, theme::with_alpha(theme::BORDER_STRONG, 150)),
    );

    // Brand row: pixel mark plus wordmark; the utility shortcuts sit at the
    // row's right edge (the lobby keeps them in its corner cluster instead).
    let mark = Rect::from_min_size(pos2(bar.left() + 16.0, bar.top() + 18.0), vec2(26.0, 26.0));
    brand::hollow_square(&painter, mark, brand::BRAND_RED);
    painter.text(
        pos2(mark.right() + 10.0, mark.center().y - 6.0),
        Align2::LEFT_CENTER,
        "AMC",
        theme::display(15.0),
        theme::HEAD.into(),
    );
    painter.text(
        pos2(mark.right() + 10.0, mark.center().y + 8.0),
        Align2::LEFT_CENTER,
        "LAUNCHER",
        theme::mono(8.0),
        theme::with_alpha(theme::HEAD, 170),
    );

    let util_y = bar.top() + 17.0;
    let downloads = Rect::from_min_size(pos2(bar.right() - 16.0 - 28.0, util_y), vec2(28.0, 28.0));
    let console = Rect::from_min_size(
        pos2(downloads.left() - 6.0 - 28.0, util_y),
        vec2(28.0, 28.0),
    );
    let gallery = Rect::from_min_size(pos2(console.left() - 6.0 - 28.0, util_y), vec2(28.0, 28.0));
    if rail_utility(
        ui,
        &painter,
        gallery,
        "rail_gallery",
        Icon::Gallery,
        lang.screens_title(),
    )
    .clicked()
    {
        action.gallery = true;
    }
    if rail_utility(
        ui,
        &painter,
        console,
        "rail_console",
        Icon::Console,
        lang.titlebar_console_tooltip(),
    )
    .clicked()
    {
        action.console = true;
    }
    if rail_utility(
        ui,
        &painter,
        downloads,
        "rail_downloads",
        Icon::Download,
        lang.dlman_title(),
    )
    .clicked()
    {
        action.downloads = true;
    }

    // Section items: 41 px rows, 17 px icon, 14 px label; the active row gets
    // the reference's accent-soft fill, accent text and 3 px edge bar.
    let mut y = bar.top() + RAIL_BRAND_H;
    for (index, tab) in RAIL_TABS.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(bar.left() + 8.0, y),
            vec2(bar.width() - 16.0, RAIL_ITEM_H),
        );
        let resp = hit(ui, rect, ("rail", index));
        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active = *tab == current_tab;
        if active {
            stepped_rect(
                &painter,
                rect,
                CUT_CONTROL,
                theme::with_alpha(theme::ACCENT, 33),
            );
            painter.rect_filled(
                Rect::from_min_max(
                    pos2(bar.left(), rect.top() + 6.0),
                    pos2(bar.left() + 3.0, rect.bottom() - 6.0),
                ),
                Rounding::ZERO,
                theme::ACCENT,
            );
        } else if hover > 0.02 {
            stepped_rect(
                &painter,
                rect,
                CUT_CONTROL,
                theme::lerp_color(Color32::TRANSPARENT, theme::CARD_HOVER, hover),
            );
        }
        let color: Color32 = if active {
            theme::ACCENT.into()
        } else {
            theme::lerp_color(theme::MUTED, theme::HEAD, hover)
        };
        icons::draw(
            &painter,
            rail_icon(*tab),
            Rect::from_center_size(pos2(rect.left() + 20.5, rect.center().y), vec2(17.0, 17.0)),
            color,
            1.45,
        );
        painter.text(
            pos2(rect.left() + 38.0, rect.center().y),
            Align2::LEFT_CENTER,
            short_name(rail_label(*tab, lang), 20),
            egui::FontId::proportional(14.0),
            color,
        );
        if resp.clicked() {
            action.tab = Some(*tab);
        }
        y += RAIL_ITEM_H + 2.0;
    }

    // Pinned account card (also the Profile page entrance).
    let card = Rect::from_min_size(
        pos2(bar.left() + 12.0, bar.bottom() - 12.0 - 56.0),
        vec2(bar.width() - 24.0, 56.0),
    );
    let resp = hit(ui, card, "rail_account");
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let profile_active = current_tab == NavTab::Profile;
    stepped_plate(
        &painter,
        card,
        CUT_CONTROL,
        theme::lerp_color(theme::FIELD, theme::CARD_HOVER, hover),
        Color32::from_black_alpha(102),
        2.0,
    );
    stepped_facets(
        &painter,
        card,
        CUT_CONTROL,
        2.0,
        Color32::from_white_alpha(76),
        Color32::from_black_alpha(82),
    );
    let avatar = Rect::from_min_size(
        pos2(card.left() + 10.0, card.center().y - 18.0),
        vec2(36.0, 36.0),
    );
    stepped_plate(
        &painter,
        avatar,
        CUT_CONTROL,
        theme::with_alpha(theme::ACCENT_DIM, 235),
        Color32::from_black_alpha(80),
        2.0,
    );
    match account_name.and_then(|name| name.chars().next()) {
        Some(letter) => {
            painter.text(
                avatar.center(),
                Align2::CENTER_CENTER,
                letter.to_uppercase(),
                theme::display(17.0),
                Color32::WHITE,
            );
        }
        None => {
            icons::draw(
                &painter,
                Icon::Person,
                Rect::from_center_size(avatar.center(), vec2(18.0, 18.0)),
                Color32::WHITE,
                1.4,
            );
        }
    }
    let name_x = avatar.right() + 10.0;
    painter.text(
        pos2(name_x, card.center().y - 8.0),
        Align2::LEFT_CENTER,
        short_name(account_name.unwrap_or_else(|| lang.shell_nickname()), 13),
        egui::FontId::proportional(13.0),
        if profile_active {
            theme::ACCENT.into()
        } else {
            theme::HEAD.into()
        },
    );
    painter.text(
        pos2(name_x, card.center().y + 10.0),
        Align2::LEFT_CENTER,
        if account_name.is_some() {
            lang.nav_profile()
        } else {
            lang.bottom_btn_login()
        },
        egui::FontId::proportional(11.0),
        theme::MUTED.into(),
    );
    icons::draw(
        &painter,
        Icon::CaretRight,
        Rect::from_center_size(pos2(card.right() - 14.0, card.center().y), vec2(14.0, 14.0)),
        theme::MUTED,
        1.4,
    );
    if let Some(name) = account_name {
        if name.chars().count() > 13 {
            resp.clone().on_hover_text(name);
        }
    }
    if resp.clicked() {
        if account_name.is_some() {
            action.tab = Some(NavTab::Profile);
        } else {
            action.login = true;
        }
    }

    action
}

// ---------------------------------------------------------------------------
// Default lobby
// ---------------------------------------------------------------------------

fn tile_label(index: usize, lang: Language) -> &'static str {
    match index {
        0 => lang.shell_tile_builds(),
        1 => lang.shell_tile_mods(),
        2 => lang.shell_tile_skins(),
        _ => lang.shell_tile_settings(),
    }
}

fn playtime_text(minutes: u64, lang: Language) -> String {
    if minutes < 60 {
        format!("{} {}", minutes, lang.inst_mins_suffix())
    } else {
        format!("{} {}", minutes / 60, lang.inst_hours_suffix())
    }
}

fn loader_icon(loader: Option<&str>) -> Icon {
    match loader.map(|name| name.to_ascii_lowercase()) {
        Some(name) if name.contains("fabric") => Icon::Spool,
        Some(name) if name.contains("quilt") => Icon::Needle,
        Some(name) if name.contains("forge") => Icon::Hammer,
        Some(name) if name.contains("optifine") => Icon::Sun,
        Some(_) => Icon::Cube,
        None => Icon::Package,
    }
}

fn draw_scene(painter: &egui::Painter, stage: Rect) {
    let sky_top = Color32::from_rgb(0x0B, 0x18, 0x10);
    let sky_low = Color32::from_rgb(0x25, 0x51, 0x31);
    let ground_top = Color32::from_rgb(0x1B, 0x3F, 0x25);
    let ground_low = Color32::from_rgb(0x0A, 0x1A, 0x10);
    let horizon = stage.top() + stage.height() * 0.62;

    // Sky and ground in chunky pixel bands (the reference scene is pixel art,
    // so banding is part of the language rather than a gradient artifact).
    let bands = 32;
    let band_h = stage.height() / bands as f32;
    for i in 0..bands {
        let top = stage.top() + band_h * i as f32;
        let t = i as f32 / (bands - 1) as f32;
        let color = if top + band_h * 0.5 <= horizon {
            theme::lerp_color(sky_top, sky_low, (t / 0.62).clamp(0.0, 1.0))
        } else {
            let ground_t = ((top - horizon).max(0.0) / (stage.height() * 0.38)).clamp(0.0, 1.0);
            theme::lerp_color(ground_top, ground_low, ground_t)
        };
        painter.rect_filled(
            Rect::from_min_size(pos2(stage.left(), top), vec2(stage.width(), band_h + 1.0)),
            Rounding::ZERO,
            color,
        );
    }

    // Horizon line and a soft sun glow behind the centrepiece.
    painter.rect_filled(
        Rect::from_min_max(
            pos2(stage.left(), horizon),
            pos2(stage.right(), horizon + 2.0),
        ),
        Rounding::ZERO,
        theme::with_alpha(theme::ACCENT, 28),
    );
    let sun = pos2(stage.center().x + stage.width() * 0.12, horizon - 36.0);
    radial_glow(painter, sun, 260.0, theme::with_alpha(theme::ACCENT, 22));

    // Ground checker (very faint; reads as pixel grass up close).
    let cell = 34.0;
    let mut y = horizon;
    let mut row = 0;
    while y < stage.bottom() {
        let mut x = stage.left();
        let mut col = 0;
        while x < stage.right() {
            if (row + col) % 2 == 0 {
                painter.rect_filled(
                    Rect::from_min_size(pos2(x, y), vec2(cell, cell)),
                    Rounding::ZERO,
                    Color32::from_white_alpha(3),
                );
            }
            x += cell;
            col += 1;
        }
        y += cell;
        row += 1;
    }

    // Stepped scrim towards the dock so the bottom UI always reads.
    let scrim_top = stage.top() + stage.height() * 0.55;
    let steps = 6;
    let step_h = (stage.bottom() - scrim_top) / steps as f32;
    for i in 0..steps {
        let top = scrim_top + step_h * i as f32;
        painter.rect_filled(
            Rect::from_min_size(pos2(stage.left(), top), vec2(stage.width(), step_h + 1.0)),
            Rounding::ZERO,
            Color32::from_black_alpha(18 + i as u8 * 12),
        );
    }

    for (fx, fy, size) in [
        (0.18, 0.30, 3.0),
        (0.32, 0.18, 2.0),
        (0.72, 0.24, 2.0),
        (0.84, 0.42, 3.0),
        (0.12, 0.52, 2.0),
        (0.64, 0.50, 2.0),
    ] {
        painter.rect_filled(
            Rect::from_center_size(
                pos2(
                    stage.left() + stage.width() * fx,
                    stage.top() + stage.height() * fy,
                ),
                vec2(size, size),
            ),
            Rounding::ZERO,
            Color32::from_white_alpha(64),
        );
    }
}

fn draw_centerpiece(ui: &mut Ui, stage: Rect, ctx: &LobbyCtx<'_>, skin3d: &mut Skin3d) {
    let painter = ui.painter().clone();
    let base_y = stage.bottom() - DOCK_H - 16.0;
    let cx = stage.center().x;
    let glow = pos2(cx, base_y - 118.0);
    radial_glow(&painter, glow, 210.0, theme::with_alpha(theme::ACCENT, 34));

    painter.rect_filled(
        Rect::from_center_size(pos2(cx, base_y), vec2(210.0, 10.0)),
        Rounding::ZERO,
        Color32::from_black_alpha(40),
    );
    let steps = [
        (230.0_f32, Color32::from_rgb(0x1B, 0x2A, 0x20)),
        (180.0, Color32::from_rgb(0x22, 0x3A, 0x28)),
        (130.0, Color32::from_rgb(0x2A, 0x47, 0x32)),
    ];
    for (index, (width, fill)) in steps.iter().enumerate() {
        let step = Rect::from_center_size(
            pos2(cx, base_y - 12.0 - index as f32 * 12.0),
            vec2(*width, 12.0),
        );
        painter.rect_filled(step, Rounding::ZERO, *fill);
        painter.rect_filled(
            Rect::from_min_max(step.left_top(), pos2(step.right(), step.top() + 2.0)),
            Rounding::ZERO,
            Color32::from_white_alpha(24),
        );
    }

    // The reference lobby puts a full 3D player on the plinth. Draw the
    // current skin through the same native egui mesh used by the skins page.
    let viewer = Rect::from_center_size(pos2(cx, base_y - 180.0), vec2(250.0, 330.0));
    skin3d.show(ui, viewer, ctx.skin, ctx.slim);

    for (dx, dy, size, alpha) in [
        (-112.0, -102.0, 4.0, 120u8),
        (110.0, -72.0, 3.0, 90),
        (-124.0, -26.0, 3.0, 80),
        (104.0, -18.0, 4.0, 110),
        (-64.0, -146.0, 3.0, 90),
        (66.0, -154.0, 2.0, 70),
    ] {
        painter.rect_filled(
            Rect::from_center_size(pos2(cx + dx, viewer.center().y + dy), vec2(size, size)),
            Rounding::ZERO,
            Color32::from_white_alpha(alpha),
        );
    }
}

fn draw_tile(ui: &mut Ui, rect: Rect, index: usize, lang: Language) -> Response {
    let resp = hit(ui, rect, ("tile", index));
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let painter = ui.painter().clone();
    let (_, icon, tone) = TILE_TARGETS[index];
    let colors = tone.colors();

    stepped_plate(
        &painter,
        rect,
        8.0,
        theme::lerp_color(colors.base, colors.hover, hover),
        colors.outline,
        3.0,
    );
    stepped_facets(
        &painter,
        rect,
        8.0,
        3.0,
        Color32::from_white_alpha(64),
        Color32::from_black_alpha(72),
    );

    let inner = rect.shrink(3.0);
    let band_h = 34.0;
    let band_top = inner.bottom() - band_h;
    let art_center = pos2(rect.center().x, (band_top - 29.0).max(rect.top() + 24.0));
    for (dx, dy) in [(-64.0, 24.0), (-44.0, 50.0), (64.0, 24.0), (44.0, 50.0)] {
        painter.line_segment(
            [art_center, pos2(art_center.x + dx, art_center.y + dy)],
            Stroke::new(6.0_f32, colors.ray),
        );
    }
    icons::draw(
        &painter,
        icon,
        Rect::from_center_size(
            pos2(art_center.x + 2.0, art_center.y + 3.0),
            vec2(36.0, 36.0),
        ),
        Color32::from_black_alpha(64),
        1.7,
    );
    icons::draw(
        &painter,
        icon,
        Rect::from_center_size(art_center, vec2(36.0, 36.0)),
        Color32::WHITE,
        1.7,
    );

    // Bottom band: a seam, the section plate and the carved bottom corners.
    painter.rect_filled(
        Rect::from_min_max(
            pos2(inner.left(), band_top - 2.0),
            pos2(inner.right(), band_top),
        ),
        Rounding::ZERO,
        colors.outline,
    );
    let split = inner.bottom() - 8.0;
    painter.rect_filled(
        Rect::from_min_max(
            inner.left_top() + vec2(0.0, band_top - inner.top()),
            pos2(inner.right(), split),
        ),
        Rounding::ZERO,
        colors.band,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(inner.left() + 8.0, split),
            pos2(inner.right() - 8.0, inner.bottom()),
        ),
        Rounding::ZERO,
        colors.band,
    );
    painter.rect_filled(
        Rect::from_min_max(
            pos2(inner.left() + 8.0, inner.bottom() - 4.0),
            pos2(inner.right() - 8.0, inner.bottom()),
        ),
        Rounding::ZERO,
        Color32::from_black_alpha(90),
    );

    let label = short_name(tile_label(index, lang), 16);
    let label_y = band_top + band_h / 2.0 - 1.0;
    painter.text(
        pos2(inner.left() + 9.0, label_y + 1.0),
        Align2::LEFT_CENTER,
        &label,
        theme::display(16.5),
        Color32::from_black_alpha(90),
    );
    painter.text(
        pos2(inner.left() + 8.0, label_y),
        Align2::LEFT_CENTER,
        label,
        theme::display(16.5),
        Color32::WHITE,
    );

    resp
}

fn corner_button(ui: &mut Ui, rect: Rect, key: &'static str, icon: Icon, tip: &str) -> Response {
    let resp = hit(ui, rect, key).on_hover_text(tip);
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let painter = ui.painter().clone();
    let drawn = rect.translate(vec2(0.0, -2.0 * hover));
    stepped_plate(
        &painter,
        drawn,
        CUT_CONTROL,
        theme::lerp_color(theme::CARD, theme::CARD_HOVER, hover),
        Color32::from_black_alpha(102),
        2.0,
    );
    stepped_facets(
        &painter,
        drawn,
        CUT_CONTROL,
        2.0,
        Color32::from_white_alpha(64),
        Color32::from_black_alpha(76),
    );
    icons::draw(
        &painter,
        icon,
        Rect::from_center_size(drawn.center(), vec2(24.0, 24.0)),
        theme::HEAD,
        1.6,
    );
    resp
}

fn draw_cluster(
    ui: &mut Ui,
    stage: Rect,
    lang: Language,
    ctx: &LobbyCtx<'_>,
    action: &mut ShellAction,
) {
    let height = 56.0;
    let gap = 10.0;
    let y = stage.top() + 16.0;
    let right = stage.right() - 16.0;

    let settings = Rect::from_min_size(pos2(right - 56.0, y), vec2(56.0, height));
    let downloads = Rect::from_min_size(pos2(settings.left() - gap - 56.0, y), vec2(56.0, height));
    let console = Rect::from_min_size(pos2(downloads.left() - gap - 56.0, y), vec2(56.0, height));

    let name = ctx
        .account
        .map(|name| short_name(name, 14))
        .unwrap_or_else(|| lang.shell_nickname().to_string());
    let name_width = ui
        .painter()
        .layout_no_wrap(
            name.clone(),
            egui::FontId::proportional(15.0),
            Color32::WHITE,
        )
        .size()
        .x
        .min(150.0);
    let pill_w = 5.0 + 46.0 + 10.0 + name_width + 8.0 + 16.0 + 12.0;
    let pill = Rect::from_min_size(pos2(console.left() - gap - pill_w, y), vec2(pill_w, height));

    let pill_resp = hit(ui, pill, "lobby_account");
    let pill_hover = ui
        .ctx()
        .animate_bool_responsive(pill_resp.id, pill_resp.hovered());
    let painter = ui.painter().clone();
    stepped_plate(
        &painter,
        pill,
        CUT_CONTROL,
        theme::lerp_color(theme::CARD, theme::CARD_HOVER, pill_hover),
        Color32::from_black_alpha(102),
        2.0,
    );
    stepped_facets(
        &painter,
        pill,
        CUT_CONTROL,
        2.0,
        Color32::from_white_alpha(76),
        Color32::from_black_alpha(82),
    );
    let avatar = Rect::from_min_size(
        pos2(pill.left() + 5.0, pill.center().y - 23.0),
        vec2(46.0, 46.0),
    );
    stepped_plate(
        &painter,
        avatar,
        CUT_CONTROL,
        theme::with_alpha(theme::ACCENT_DIM, 235),
        Color32::from_black_alpha(80),
        2.0,
    );
    match ctx.account.and_then(|name| name.chars().next()) {
        Some(letter) => {
            painter.text(
                avatar.center(),
                Align2::CENTER_CENTER,
                letter.to_uppercase(),
                theme::display(20.0),
                Color32::WHITE,
            );
        }
        None => {
            icons::draw(
                &painter,
                Icon::Person,
                Rect::from_center_size(avatar.center(), vec2(22.0, 22.0)),
                Color32::WHITE,
                1.5,
            );
        }
    }
    let text_x = avatar.right() + 10.0;
    painter.text(
        pos2(text_x, pill.center().y - 9.0),
        Align2::LEFT_CENTER,
        &name,
        egui::FontId::proportional(15.0),
        theme::HEAD.into(),
    );
    painter.text(
        pos2(text_x, pill.center().y + 10.0),
        Align2::LEFT_CENTER,
        if ctx.account.is_some() {
            tab_label(NavTab::Profile, lang)
        } else {
            lang.bottom_btn_login()
        },
        egui::FontId::proportional(12.0),
        theme::MUTED.into(),
    );
    icons::draw(
        &painter,
        Icon::CaretDown,
        Rect::from_center_size(pos2(pill.right() - 20.0, pill.center().y), vec2(16.0, 16.0)),
        theme::MUTED,
        1.4,
    );
    if let Some(full) = ctx.account {
        if full.chars().count() > 14 {
            pill_resp.clone().on_hover_text(full);
        }
    }
    if pill_resp.clicked() {
        if ctx.account.is_some() {
            action.tab = Some(NavTab::Profile);
        } else {
            action.login = true;
        }
    }

    let console_resp = corner_button(
        ui,
        console,
        "lobby_console",
        Icon::Console,
        lang.titlebar_console_tooltip(),
    );
    if console_resp.clicked() {
        action.console = true;
    }
    let downloads_resp = corner_button(
        ui,
        downloads,
        "lobby_downloads",
        Icon::Download,
        lang.dlman_title(),
    );
    if downloads_resp.clicked() {
        action.downloads = true;
    }
    let settings_resp = corner_button(
        ui,
        settings,
        "lobby_settings",
        Icon::Gear,
        tab_label(NavTab::Settings, lang),
    );
    if settings_resp.clicked() {
        action.tab = Some(NavTab::Settings);
    }
}

fn draw_promo(ui: &mut Ui, stage: Rect, lang: Language) {
    let rect = Rect::from_min_size(
        pos2(
            stage.right() - 20.0 - 300.0,
            stage.top() + stage.height() * 0.5 - 164.0 * 0.58,
        ),
        vec2(300.0, 164.0),
    );
    let resp = hit(ui, rect, "lobby_promo").on_hover_text(PROJECT_URL);
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let painter = ui.painter().clone();
    stepped_plate(
        &painter,
        rect,
        CUT_PANEL,
        theme::lerp_color(
            Color32::from_rgba_unmultiplied(10, 30, 16, 200),
            Color32::from_rgba_unmultiplied(16, 48, 26, 220),
            hover,
        ),
        Color32::from_black_alpha(128),
        3.0,
    );
    stepped_facets(
        &painter,
        rect,
        CUT_PANEL,
        3.0,
        Color32::from_white_alpha(30),
        Color32::from_black_alpha(90),
    );

    let art = pos2(rect.right() - 66.0, rect.top() + 66.0);
    radial_glow(&painter, art, 72.0, theme::with_alpha(theme::ACCENT, 42));
    painter.circle_filled(art, 34.0, theme::with_alpha(theme::ACCENT, 30));
    icons::draw(
        &painter,
        Icon::Sword,
        Rect::from_center_size(art, vec2(52.0, 52.0)),
        Color32::from_rgb(0xFF, 0xD2, 0x3F),
        2.0,
    );
    for (dx, dy, size) in [(-60.0, -46.0, 4.0), (-88.0, 12.0, 3.0), (-46.0, 48.0, 4.0)] {
        painter.rect_filled(
            Rect::from_center_size(pos2(art.x + dx, art.y + dy), vec2(size, size)),
            Rounding::ZERO,
            Color32::from_white_alpha(110),
        );
    }

    let x = rect.left() + 18.0;
    painter.text(
        pos2(x, rect.top() + 24.0),
        Align2::LEFT_CENTER,
        lang.shell_promo_label().to_uppercase(),
        theme::display(11.0),
        Color32::from_rgb(0xFF, 0xD2, 0x3F),
    );
    painter.text(
        pos2(x, rect.top() + 52.0),
        Align2::LEFT_CENTER,
        "AMC",
        theme::display(26.0),
        Color32::WHITE,
    );
    painter.text(
        pos2(x, rect.top() + 78.0),
        Align2::LEFT_CENTER,
        "Launcher",
        theme::display(16.0),
        Color32::from_white_alpha(200),
    );
    let cta = Rect::from_min_size(pos2(x, rect.top() + 108.0), vec2(96.0, 28.0));
    stepped_plate(
        &painter,
        cta,
        CUT_CONTROL,
        theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
        Color32::TRANSPARENT,
        0.0,
    );
    painter.text(
        cta.center(),
        Align2::CENTER_CENTER,
        lang.shell_promo_action(),
        egui::FontId::proportional(12.5),
        ON_ACCENT,
    );

    if resp.clicked() {
        ui.ctx().output_mut(|output| {
            output.open_url = Some(egui::output::OpenUrl {
                url: PROJECT_URL.to_owned(),
                new_tab: true,
            });
        });
    }
}

fn draw_dock(
    ui: &mut Ui,
    stage: Rect,
    lang: Language,
    ctx: &LobbyCtx<'_>,
    action: &mut ShellAction,
) {
    let dock = Rect::from_min_size(
        pos2(stage.center().x - DOCK_W / 2.0, stage.bottom() - DOCK_H),
        vec2(DOCK_W, DOCK_H),
    );
    let mode_rect = Rect::from_min_size(
        pos2(dock.left() + DOCK_PAD, dock.top() + DOCK_PAD),
        vec2(MODE_W, DOCK_H - DOCK_PAD - 10.0),
    );
    let mode_resp = hit(ui, mode_rect, "lobby_mode");
    let mode_hover = ui
        .ctx()
        .animate_bool_responsive(mode_resp.id, mode_resp.hovered());

    let play = Rect::from_min_size(
        pos2(mode_rect.right() + 6.0, mode_rect.top()),
        vec2(PLAY_W, PLAY_H),
    );
    let play_resp = hit(ui, play, "lobby_play");
    let play_hover = ui
        .ctx()
        .animate_bool_responsive(play_resp.id, play_resp.hovered());

    let painter = ui.painter().clone();
    top_cut_plate(
        &painter,
        dock,
        CUT_PANEL,
        Color32::from_rgba_unmultiplied(4, 20, 10, 140),
        Color32::from_black_alpha(115),
        3.0,
    );

    // Mode card: blue with the selected build, amber while nothing is picked.
    let is_empty = ctx.instance.is_none();
    let (base, hover, art_bg, art_fg) = if is_empty {
        (
            MODE_EMPTY,
            MODE_EMPTY_HOVER,
            Color32::BLACK,
            Color32::from_rgb(0xFF, 0xD3, 0x6B),
        )
    } else {
        (MODE_BLUE, MODE_BLUE_HOVER, MODE_ART, Color32::WHITE)
    };
    stepped_plate(
        &painter,
        mode_rect,
        CUT_PANEL,
        theme::lerp_color(base, hover, mode_hover),
        Color32::from_black_alpha(90),
        3.0,
    );
    stepped_facets(
        &painter,
        mode_rect,
        CUT_PANEL,
        3.0,
        Color32::from_white_alpha(56),
        Color32::from_black_alpha(76),
    );
    let art = Rect::from_min_size(
        pos2(mode_rect.left() + 8.0, mode_rect.top() + 8.0),
        vec2(138.0, 104.0),
    );
    stepped_plate(
        &painter,
        art,
        CUT_CONTROL,
        art_bg,
        Color32::from_black_alpha(70),
        2.0,
    );
    icons::draw(
        &painter,
        loader_icon(ctx.instance.as_ref().map(|instance| instance.loader)),
        Rect::from_center_size(art.center(), vec2(48.0, 48.0)),
        art_fg,
        1.6,
    );

    let text_x = art.right() + 16.0;
    painter.text(
        pos2(text_x, mode_rect.top() + 24.0),
        Align2::LEFT_CENTER,
        lang.shell_mode_today().to_uppercase(),
        theme::display(13.0),
        Color32::from_white_alpha(184),
    );
    let title = ctx
        .instance
        .as_ref()
        .map(|instance| short_name(instance.name, 11))
        .unwrap_or_else(|| lang.shell_mode_empty().to_string());
    painter.text(
        pos2(text_x + 1.0, mode_rect.top() + 58.0),
        Align2::LEFT_CENTER,
        &title,
        theme::display(26.0),
        Color32::from_black_alpha(80),
    );
    painter.text(
        pos2(text_x, mode_rect.top() + 56.0),
        Align2::LEFT_CENTER,
        &title,
        theme::display(26.0),
        Color32::WHITE,
    );
    let meta = match ctx.instance.as_ref() {
        Some(instance) => format!("{} · {}", instance.loader, instance.version),
        None => lang.shell_pick_build().to_string(),
    };
    painter.text(
        pos2(text_x, mode_rect.top() + 86.0),
        Align2::LEFT_CENTER,
        meta,
        egui::FontId::proportional(12.0),
        Color32::from_white_alpha(210),
    );
    let swap = Rect::from_center_size(
        pos2(mode_rect.right() - 26.0, mode_rect.center().y),
        vec2(36.0, 36.0),
    );
    stepped_plate(
        &painter,
        swap,
        CUT_CONTROL,
        Color32::from_black_alpha(64),
        Color32::TRANSPARENT,
        0.0,
    );
    icons::draw(
        &painter,
        Icon::Refresh,
        Rect::from_center_size(swap.center(), vec2(18.0, 18.0)),
        Color32::from_white_alpha(220),
        1.5,
    );
    if mode_resp.clicked() {
        action.tab = Some(NavTab::Modpacks);
    }

    // Play column: the 83 px primary action plus the 34 px status strip.
    let play_base: Color32 = if ctx.launching {
        theme::ACCENT_DIM.into()
    } else {
        theme::ACCENT.into()
    };
    let play_target: Color32 = if ctx.launching {
        theme::ACCENT_DIM.into()
    } else {
        theme::ACCENT_LIGHT.into()
    };
    stepped_plate(
        &painter,
        play,
        CUT_CONTROL,
        theme::lerp_color(play_base, play_target, play_hover),
        Color32::from_black_alpha(40),
        2.0,
    );
    stepped_facets(
        &painter,
        play,
        CUT_CONTROL,
        2.0,
        Color32::from_white_alpha(86),
        Color32::from_black_alpha(96),
    );
    let play_text = if ctx.launching {
        lang.bottom_btn_launching()
    } else {
        lang.shell_rail_play()
    };
    let font = if ctx.launching {
        theme::display(22.0)
    } else {
        theme::display(34.0)
    };
    let hours = ctx
        .instance
        .as_ref()
        .filter(|instance| instance.minutes > 0)
        .map(|instance| playtime_text(instance.minutes, lang));
    match hours {
        Some(hours) if !ctx.launching => {
            painter.text(
                pos2(play.center().x, play.center().y - 11.0),
                Align2::CENTER_CENTER,
                play_text,
                font,
                ON_ACCENT,
            );
            painter.text(
                pos2(play.center().x, play.center().y + 20.0),
                Align2::CENTER_CENTER,
                hours,
                theme::display(19.0),
                Color32::from_black_alpha(216),
            );
        }
        _ => {
            painter.text(
                play.center(),
                Align2::CENTER_CENTER,
                play_text,
                font,
                ON_ACCENT,
            );
        }
    }
    if play_resp.clicked() && !ctx.launching {
        if ctx.instance.is_some() {
            action.launch = true;
        } else {
            action.tab = Some(NavTab::Modpacks);
        }
    }

    let strip = Rect::from_min_size(
        pos2(play.left(), play.bottom() + 6.0),
        vec2(PLAY_W, CHEST_H),
    );
    stepped_plate(
        &painter,
        strip,
        CUT_CONTROL,
        Color32::from_black_alpha(82),
        Color32::TRANSPARENT,
        0.0,
    );
    let (strip_icon, strip_text, strip_color): (Icon, &str, Color32) = if ctx.launching {
        (
            Icon::Hourglass,
            lang.bottom_btn_launching(),
            theme::MUTED.into(),
        )
    } else if ctx.instance.is_some() {
        (
            Icon::Check,
            lang.shell_ready_to_launch(),
            theme::ACCENT.into(),
        )
    } else {
        (Icon::Package, lang.shell_pick_build(), theme::MUTED.into())
    };
    icons::draw(
        &painter,
        strip_icon,
        Rect::from_center_size(
            pos2(strip.left() + 14.0 + 8.0, strip.center().y),
            vec2(16.0, 16.0),
        ),
        strip_color,
        1.4,
    );
    painter.text(
        pos2(strip.left() + 14.0 + 16.0 + 8.0, strip.center().y),
        Align2::LEFT_CENTER,
        strip_text,
        egui::FontId::proportional(13.0),
        theme::MUTED.into(),
    );
}

/// The default lobby (Home tab): stage, section tiles, corner cluster, promo
/// and the bottom dock.
pub fn show_lobby(
    ui: &mut Ui,
    lang: Language,
    ctx: &LobbyCtx<'_>,
    skin3d: &mut Skin3d,
) -> ShellAction {
    let mut action = ShellAction::default();
    let stage = ui.max_rect();

    draw_scene(ui.painter(), stage);
    draw_centerpiece(ui, stage, ctx, skin3d);

    let mut y = stage.top() + 20.0;
    // The four tiles share the space above the dock. At 960x600 the fixed
    // 97 px height ran 27 px into the dock, hiding the Settings tile; scale
    // the tiles down instead of letting them collide with the dock.
    let dock_top = stage.bottom() - DOCK_H;
    let tile_h = ((dock_top - 10.0 - y - 3.0 * TILE_GAP) / 4.0).clamp(84.0, TILE_H);
    for (index, (target, _, _)) in TILE_TARGETS.iter().enumerate() {
        let rect = Rect::from_min_size(pos2(stage.left() + 20.0, y), vec2(TILE_W, tile_h));
        if draw_tile(ui, rect, index, lang).clicked() {
            action.tab = Some(*target);
        }
        y += tile_h + TILE_GAP;
    }

    draw_cluster(ui, stage, lang, ctx, &mut action);
    draw_promo(ui, stage, lang);
    draw_dock(ui, stage, lang, ctx, &mut action);
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rail_covers_every_page_once() {
        for tab in [
            NavTab::Home,
            NavTab::Modpacks,
            NavTab::Mods,
            NavTab::Skins,
            NavTab::Settings,
        ] {
            assert_eq!(
                RAIL_TABS
                    .iter()
                    .filter(|candidate| **candidate == tab)
                    .count(),
                1,
                "{tab:?} must appear exactly once in the rail"
            );
        }
        // Profile has its own pinned entrance (the account card).
        assert!(!RAIL_TABS.contains(&NavTab::Profile));
    }

    #[test]
    fn test_tiles_open_every_lobby_section() {
        let targets: Vec<NavTab> = TILE_TARGETS.iter().map(|(tab, _, _)| *tab).collect();
        assert_eq!(
            targets,
            vec![
                NavTab::Modpacks,
                NavTab::Mods,
                NavTab::Skins,
                NavTab::Settings
            ]
        );
    }

    #[test]
    fn test_playtime_switches_from_minutes_to_hours() {
        assert_eq!(playtime_text(30, Language::English), "30 min");
        assert_eq!(playtime_text(59, Language::Russian), "59 мин");
        assert_eq!(playtime_text(60, Language::English), "1 h");
        assert_eq!(playtime_text(185, Language::Ukrainian), "3 год");
    }
}
