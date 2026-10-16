//! Painter-drawn vector icons for chrome and list widgets.
//!
//! The bundled UI fonts have no emoji coverage, so glyph "icons" such as
//! `👤` render as tofu boxes. Everything here is stroked/filled with
//! `egui::Painter` primitives instead and scales with the target rect.

use crate::theme::{lerp_color, with_alpha};
use amc_core::types::LoaderType;
use egui::{
    pos2, vec2, Color32, Painter, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui, Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    // Navigation chrome
    Home,
    Grid,
    Diamond,
    Person,
    Card,
    Gear,
    Power,
    // Title bar
    Gallery,
    Console,
    Download,
    Minimize,
    Maximize,
    Restore,
    Close,
    CaretDown,
    CaretUp,
    CaretRight,
    ChevronRight,
    // Actions
    Search,
    Trash,
    Folder,
    Refresh,
    Play,
    Clipboard,
    Link,
    Wrench,
    Check,
    Warning,
    Bulb,
    // Inline list badges
    Star,
    Heart,
    HeartOutline,
    Bolt,
    Flame,
    Globe,
    Puzzle,
    Palette,
    Sun,
    Sparkle,
    Clock,
    Disk,
    Hourglass,
    Tag,
    Pin,
    PinOutline,
    Dot,
    List,
    // Loader badges (used when an instance has no custom icon; the Forge
    // badge reuses `Hammer` from the list below)
    Cube,
    Spool,
    Needle,
    // Instance icon presets (keys of `InstanceIcon::ALL` in core)
    Pickaxe,
    Sword,
    Shield,
    Bow,
    Potion,
    Dragon,
    Castle,
    Wheat,
    Alien,
    // Version list badges (the Forge loader badge reuses `Hammer`)
    Package,
    Flask,
    Hammer,
    Doc,
}

impl Icon {
    /// Every icon; used by the render test and by future icon pickers.
    pub const ALL: &'static [Icon] = &[
        Icon::Home,
        Icon::Grid,
        Icon::Diamond,
        Icon::Person,
        Icon::Card,
        Icon::Gear,
        Icon::Power,
        Icon::Gallery,
        Icon::Console,
        Icon::Download,
        Icon::Minimize,
        Icon::Maximize,
        Icon::Restore,
        Icon::Close,
        Icon::CaretDown,
        Icon::CaretUp,
        Icon::CaretRight,
        Icon::ChevronRight,
        Icon::Search,
        Icon::Trash,
        Icon::Folder,
        Icon::Refresh,
        Icon::Play,
        Icon::Clipboard,
        Icon::Link,
        Icon::Wrench,
        Icon::Check,
        Icon::Warning,
        Icon::Bulb,
        Icon::Star,
        Icon::Heart,
        Icon::HeartOutline,
        Icon::Bolt,
        Icon::Flame,
        Icon::Globe,
        Icon::Puzzle,
        Icon::Palette,
        Icon::Sun,
        Icon::Sparkle,
        Icon::Clock,
        Icon::Disk,
        Icon::Hourglass,
        Icon::Tag,
        Icon::Pin,
        Icon::PinOutline,
        Icon::Dot,
        Icon::List,
        Icon::Cube,
        Icon::Spool,
        Icon::Needle,
        Icon::Hammer,
        Icon::Pickaxe,
        Icon::Sword,
        Icon::Shield,
        Icon::Bow,
        Icon::Potion,
        Icon::Dragon,
        Icon::Castle,
        Icon::Wheat,
        Icon::Alien,
        Icon::Package,
        Icon::Flask,
        Icon::Doc,
    ];
}

/// Vector stand-in for the persisted instance-icon preset keys
/// (`InstanceIcon::ALL` in core). Presentation-only: the stored keys keep
/// their format, so existing `instances.json` files stay valid.
pub fn instance_preset(key: &str) -> Option<Icon> {
    Some(match key {
        "pickaxe" => Icon::Pickaxe,
        "sword" => Icon::Sword,
        "shield" => Icon::Shield,
        "bow" => Icon::Bow,
        "potion" => Icon::Potion,
        "dragon" => Icon::Dragon,
        "castle" => Icon::Castle,
        "wheat" => Icon::Wheat,
        "gear" => Icon::Gear,
        "alien" => Icon::Alien,
        "art" => Icon::Palette,
        "fire" => Icon::Flame,
        _ => return None,
    })
}

/// Badge icon for a loader when an instance has no custom icon.
pub fn loader_badge(loader: LoaderType) -> Icon {
    match loader {
        LoaderType::Vanilla => Icon::Cube,
        LoaderType::Fabric => Icon::Spool,
        LoaderType::Quilt => Icon::Needle,
        LoaderType::Forge => Icon::Hammer,
        LoaderType::NeoForge => Icon::Bolt,
        LoaderType::OptiFine => Icon::Sparkle,
    }
}

/// Draw `icon` centered in `rect` with a stroke of `width` device pixels.
pub fn draw(painter: &Painter, icon: Icon, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    let center = rect.center();
    let scale = rect.width().min(rect.height()) / 2.0;
    let p = |x: f32, y: f32| pos2(center.x + x * scale, center.y + y * scale);
    let r = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(p(x0, y0), p(x1, y1));
    let line = |points: &[Vec2]| {
        painter.add(Shape::line(
            points.iter().map(|v| p(v.x, v.y)).collect(),
            stroke,
        ));
    };
    let seg = |a: Vec2, b: Vec2| {
        painter.line_segment([p(a.x, a.y), p(b.x, b.y)], stroke);
    };
    let circle = |cx: f32, cy: f32, radius: f32| {
        painter.circle_stroke(p(cx, cy), radius * scale, stroke);
    };
    let arc = |cx: f32, cy: f32, radius: f32, from: f32, to: f32| {
        let steps = 16;
        let mut points = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = from + (to - from) * (i as f32 / steps as f32);
            points.push(p(cx + radius * t.cos(), cy + radius * t.sin()));
        }
        painter.add(Shape::line(points, stroke));
    };
    let disc = |cx: f32, cy: f32, radius: f32| {
        painter.circle_filled(p(cx, cy), radius * scale, color);
    };
    let fill = |points: &[Vec2]| {
        painter.add(Shape::convex_polygon(
            points.iter().map(|v| p(v.x, v.y)).collect(),
            color,
            Stroke::NONE,
        ));
    };

    match icon {
        Icon::Home => {
            line(&[vec2(-0.9, -0.1), vec2(0.0, -0.95), vec2(0.9, -0.1)]);
            line(&[vec2(-0.68, -0.32), vec2(-0.68, 0.85)]);
            line(&[vec2(0.68, -0.32), vec2(0.68, 0.85)]);
            seg(vec2(-0.68, 0.85), vec2(0.68, 0.85));
        }
        Icon::Grid => {
            painter.rect_stroke(r(-0.85, -0.85, -0.18, -0.18), Rounding::ZERO, stroke);
            painter.rect_stroke(r(0.18, -0.85, 0.85, -0.18), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.85, 0.18, -0.18, 0.85), Rounding::ZERO, stroke);
            painter.rect_stroke(r(0.18, 0.18, 0.85, 0.85), Rounding::ZERO, stroke);
        }
        Icon::Diamond => {
            line(&[
                vec2(0.0, -0.9),
                vec2(0.9, 0.0),
                vec2(0.0, 0.9),
                vec2(-0.9, 0.0),
                vec2(0.0, -0.9),
            ]);
        }
        Icon::Person => {
            circle(0.0, -0.38, 0.3);
            arc(0.0, 0.85, 0.72, std::f32::consts::PI, std::f32::consts::TAU);
        }
        Icon::Card => {
            painter.rect_stroke(r(-0.85, -0.62, 0.85, 0.62), Rounding::ZERO, stroke);
            seg(vec2(-0.55, 0.18), vec2(0.55, 0.18));
            seg(vec2(-0.55, 0.42), vec2(0.05, 0.42));
            circle(-0.5, -0.25, 0.14);
        }
        Icon::Gear => {
            circle(0.0, 0.0, 0.3);
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                seg(
                    vec2(a.cos() * 0.52, a.sin() * 0.52),
                    vec2(a.cos() * 0.92, a.sin() * 0.92),
                );
            }
        }
        Icon::Power => {
            arc(
                0.0,
                0.0,
                0.82,
                -std::f32::consts::FRAC_PI_2 + 0.55,
                std::f32::consts::PI * 1.5 - 0.55,
            );
            seg(vec2(0.0, -0.95), vec2(0.0, 0.05));
        }
        Icon::Gallery => {
            painter.rect_stroke(r(-0.9, -0.72, 0.9, 0.72), Rounding::ZERO, stroke);
            circle(-0.38, -0.28, 0.14);
            line(&[
                vec2(-0.72, 0.5),
                vec2(-0.25, -0.02),
                vec2(0.05, 0.28),
                vec2(0.35, -0.12),
                vec2(0.75, 0.5),
            ]);
        }
        Icon::Console => {
            painter.rect_stroke(r(-0.9, -0.7, 0.9, 0.7), Rounding::ZERO, stroke);
            line(&[vec2(-0.5, -0.32), vec2(-0.12, 0.0), vec2(-0.5, 0.32)]);
            seg(vec2(0.0, 0.3), vec2(0.5, 0.3));
        }
        Icon::Download => {
            seg(vec2(0.0, -0.9), vec2(0.0, 0.35));
            seg(vec2(0.0, 0.42), vec2(-0.42, -0.02));
            seg(vec2(0.0, 0.42), vec2(0.42, -0.02));
            seg(vec2(-0.72, 0.85), vec2(0.72, 0.85));
        }
        Icon::Minimize => seg(vec2(-0.7, 0.4), vec2(0.7, 0.4)),
        Icon::Maximize => {
            painter.rect_stroke(r(-0.8, -0.8, 0.8, 0.8), Rounding::ZERO, stroke);
        }
        Icon::Restore => {
            painter.rect_stroke(r(-0.82, -0.82, 0.38, 0.38), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.38, -0.38, 0.82, 0.82), Rounding::ZERO, stroke);
        }
        Icon::Close => {
            seg(vec2(-0.62, -0.62), vec2(0.62, 0.62));
            seg(vec2(0.62, -0.62), vec2(-0.62, 0.62));
        }
        Icon::CaretDown => {
            line(&[vec2(-0.62, -0.28), vec2(0.0, 0.34), vec2(0.62, -0.28)]);
        }
        Icon::ChevronRight => {
            line(&[vec2(-0.32, -0.62), vec2(0.32, 0.0), vec2(-0.32, 0.62)]);
        }
        Icon::Search => {
            circle(-0.18, -0.18, 0.58);
            seg(vec2(0.24, 0.24), vec2(0.85, 0.85));
        }
        Icon::Trash => {
            seg(vec2(-0.78, -0.5), vec2(0.78, -0.5));
            line(&[
                vec2(-0.32, -0.5),
                vec2(-0.32, -0.78),
                vec2(0.32, -0.78),
                vec2(0.32, -0.5),
            ]);
            line(&[
                vec2(-0.58, -0.5),
                vec2(-0.45, 0.82),
                vec2(0.45, 0.82),
                vec2(0.58, -0.5),
            ]);
            seg(vec2(-0.18, -0.18), vec2(-0.18, 0.48));
            seg(vec2(0.18, -0.18), vec2(0.18, 0.48));
        }
        Icon::Folder => {
            line(&[
                vec2(-0.9, -0.62),
                vec2(-0.25, -0.62),
                vec2(0.0, -0.36),
                vec2(0.9, -0.36),
                vec2(0.9, 0.75),
                vec2(-0.9, 0.75),
                vec2(-0.9, -0.62),
            ]);
        }
        Icon::Refresh => {
            arc(0.0, 0.0, 0.72, 0.7, 5.4);
            let end = 0.7_f32;
            let tip = vec2(0.72 * end.cos(), 0.72 * end.sin());
            seg(tip, vec2(tip.x - 0.42, tip.y - 0.05));
            seg(tip, vec2(tip.x + 0.1, tip.y + 0.5));
        }
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![p(-0.55, -0.85), p(0.8, 0.0), p(-0.55, 0.85)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Clipboard => {
            painter.rect_stroke(r(-0.65, -0.72, 0.65, 0.85), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.32, -0.95, 0.32, -0.5), Rounding::ZERO, stroke);
            seg(vec2(-0.32, 0.0), vec2(0.32, 0.0));
            seg(vec2(-0.32, 0.4), vec2(0.32, 0.4));
        }
        Icon::Link => {
            circle(-0.32, -0.32, 0.44);
            circle(0.32, 0.32, 0.44);
            seg(vec2(-0.05, -0.05), vec2(0.05, 0.05));
        }
        Icon::Wrench => {
            arc(0.32, -0.32, 0.52, 2.4, 5.6);
            seg(vec2(-0.75, 0.75), vec2(0.05, -0.05));
        }
        Icon::Package => {
            painter.rect_stroke(r(-0.8, -0.28, 0.8, 0.85), Rounding::ZERO, stroke);
            line(&[
                vec2(-0.8, -0.28),
                vec2(-0.52, -0.78),
                vec2(0.52, -0.78),
                vec2(0.8, -0.28),
            ]);
            seg(vec2(-0.8, -0.28), vec2(0.8, -0.28));
            seg(vec2(0.0, -0.78), vec2(0.0, -0.28));
        }
        Icon::Flask => {
            line(&[
                vec2(-0.26, -0.85),
                vec2(-0.26, -0.3),
                vec2(-0.72, 0.62),
                vec2(0.72, 0.62),
                vec2(0.26, -0.3),
                vec2(0.26, -0.85),
            ]);
            seg(vec2(-0.42, -0.85), vec2(0.42, -0.85));
            seg(vec2(-0.5, 0.2), vec2(0.5, 0.2));
        }
        Icon::Hammer => {
            line(&[
                vec2(0.12, -0.9),
                vec2(0.88, -0.14),
                vec2(0.52, 0.22),
                vec2(-0.24, -0.54),
                vec2(0.12, -0.9),
            ]);
            seg(vec2(-0.82, 0.82), vec2(0.2, -0.2));
        }
        Icon::Doc => {
            painter.rect_stroke(r(-0.6, -0.85, 0.6, 0.85), Rounding::ZERO, stroke);
            line(&[
                vec2(0.2, -0.85),
                vec2(0.6, -0.45),
                vec2(0.2, -0.45),
                vec2(0.2, -0.85),
            ]);
            seg(vec2(-0.32, 0.05), vec2(0.32, 0.05));
            seg(vec2(-0.32, 0.45), vec2(0.32, 0.45));
        }
        Icon::CaretUp => line(&[vec2(-0.62, 0.28), vec2(0.0, -0.34), vec2(0.62, 0.28)]),
        Icon::CaretRight => line(&[vec2(-0.28, -0.62), vec2(0.34, 0.0), vec2(-0.28, 0.62)]),
        Icon::Check => line(&[vec2(-0.7, 0.05), vec2(-0.22, 0.55), vec2(0.72, -0.5)]),
        Icon::Warning => {
            line(&[
                vec2(0.0, -0.85),
                vec2(0.92, 0.72),
                vec2(-0.92, 0.72),
                vec2(0.0, -0.85),
            ]);
            seg(vec2(0.0, -0.3), vec2(0.0, 0.22));
            disc(0.0, 0.48, 0.1);
        }
        Icon::Bulb => {
            circle(0.0, -0.22, 0.55);
            seg(vec2(-0.3, 0.44), vec2(0.3, 0.44));
            seg(vec2(-0.22, 0.66), vec2(0.22, 0.66));
            seg(vec2(-0.12, 0.88), vec2(0.12, 0.88));
        }
        Icon::Star => {
            let mut points = Vec::with_capacity(11);
            for i in 0..=10 {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                let radius = if i % 2 == 0 { 0.95 } else { 0.42 };
                points.push(vec2(a.cos() * radius, a.sin() * radius));
            }
            line(&points);
        }
        Icon::Heart => {
            disc(-0.33, -0.25, 0.36);
            disc(0.33, -0.25, 0.36);
            fill(&[vec2(-0.66, -0.1), vec2(0.66, -0.1), vec2(0.0, 0.85)]);
        }
        Icon::HeartOutline => {
            arc(-0.33, -0.28, 0.34, 0.5, std::f32::consts::PI);
            arc(0.33, -0.28, 0.34, 0.0, std::f32::consts::PI - 0.5);
            line(&[vec2(-0.67, -0.28), vec2(0.0, 0.85), vec2(0.67, -0.28)]);
        }
        Icon::Bolt => {
            line(&[
                vec2(0.18, -0.95),
                vec2(-0.55, 0.08),
                vec2(-0.05, 0.08),
                vec2(-0.18, 0.95),
                vec2(0.55, -0.08),
                vec2(0.05, -0.08),
                vec2(0.18, -0.95),
            ]);
        }
        Icon::Flame => {
            line(&[
                vec2(0.0, 0.92),
                vec2(-0.48, 0.35),
                vec2(-0.4, -0.12),
                vec2(-0.05, -0.5),
                vec2(0.1, -0.9),
                vec2(0.4, -0.3),
                vec2(0.52, 0.12),
                vec2(0.0, 0.92),
            ]);
            line(&[vec2(0.0, 0.55), vec2(-0.16, 0.15), vec2(0.04, -0.12)]);
        }
        Icon::Globe => {
            circle(0.0, 0.0, 0.85);
            seg(vec2(-0.85, 0.0), vec2(0.85, 0.0));
            let mut meridian = Vec::with_capacity(17);
            for i in 0..=16 {
                let t = i as f32 / 16.0 * std::f32::consts::TAU;
                meridian.push(vec2(0.38 * t.sin(), 0.85 * t.cos()));
            }
            line(&meridian);
        }
        Icon::Puzzle => {
            painter.rect_stroke(r(-0.75, -0.35, 0.75, 0.8), Rounding::ZERO, stroke);
            arc(
                0.0,
                -0.35,
                0.32,
                std::f32::consts::PI,
                std::f32::consts::TAU,
            );
        }
        Icon::Palette => {
            circle(0.0, 0.0, 0.82);
            seg(vec2(0.5, 0.62), vec2(0.95, 0.95));
            disc(-0.38, -0.28, 0.14);
            disc(0.02, -0.45, 0.14);
            disc(0.38, -0.15, 0.14);
            disc(-0.1, 0.38, 0.14);
        }
        Icon::Sun => {
            circle(0.0, 0.0, 0.42);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                seg(
                    vec2(a.cos() * 0.62, a.sin() * 0.62),
                    vec2(a.cos() * 0.92, a.sin() * 0.92),
                );
            }
        }
        Icon::Sparkle => {
            line(&[
                vec2(0.0, -0.95),
                vec2(0.18, -0.18),
                vec2(0.95, 0.0),
                vec2(0.18, 0.18),
                vec2(0.0, 0.95),
                vec2(-0.18, 0.18),
                vec2(-0.95, 0.0),
                vec2(-0.18, -0.18),
                vec2(0.0, -0.95),
            ]);
        }
        Icon::Clock => {
            circle(0.0, 0.0, 0.82);
            seg(vec2(0.0, 0.0), vec2(0.0, -0.5));
            seg(vec2(0.0, 0.0), vec2(0.42, 0.12));
        }
        Icon::Disk => {
            painter.rect_stroke(r(-0.8, -0.8, 0.8, 0.8), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.35, -0.8, 0.35, -0.3), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.55, 0.02, 0.55, 0.8), Rounding::ZERO, stroke);
            seg(vec2(-0.2, 0.3), vec2(0.2, 0.3));
        }
        Icon::Hourglass => {
            seg(vec2(-0.6, -0.82), vec2(0.6, -0.82));
            seg(vec2(-0.6, 0.82), vec2(0.6, 0.82));
            line(&[vec2(-0.52, -0.82), vec2(0.0, 0.0), vec2(-0.52, 0.82)]);
            line(&[vec2(0.52, -0.82), vec2(0.0, 0.0), vec2(0.52, 0.82)]);
        }
        Icon::Tag => {
            line(&[
                vec2(-0.72, -0.62),
                vec2(0.3, -0.62),
                vec2(0.85, 0.0),
                vec2(0.3, 0.62),
                vec2(-0.72, 0.62),
                vec2(-0.72, -0.62),
            ]);
            circle(-0.28, 0.0, 0.15);
        }
        Icon::Pin => {
            disc(0.0, -0.32, 0.42);
            fill(&[vec2(-0.34, -0.02), vec2(0.34, -0.02), vec2(0.0, 0.88)]);
        }
        Icon::PinOutline => {
            circle(0.0, -0.32, 0.42);
            line(&[vec2(-0.36, 0.0), vec2(0.0, 0.85), vec2(0.36, 0.0)]);
            circle(0.0, -0.32, 0.16);
        }
        Icon::Dot => disc(0.0, 0.0, 0.72),
        Icon::List => {
            seg(vec2(-0.7, -0.5), vec2(0.7, -0.5));
            seg(vec2(-0.7, 0.0), vec2(0.7, 0.0));
            seg(vec2(-0.7, 0.5), vec2(0.7, 0.5));
        }
        Icon::Cube => {
            line(&[
                vec2(-0.8, 0.72),
                vec2(0.1, 0.72),
                vec2(0.1, -0.2),
                vec2(-0.8, -0.2),
                vec2(-0.8, 0.72),
            ]);
            line(&[
                vec2(-0.8, -0.2),
                vec2(-0.38, -0.62),
                vec2(0.52, -0.62),
                vec2(0.1, -0.2),
            ]);
            line(&[vec2(0.52, -0.62), vec2(0.52, 0.3), vec2(0.1, 0.72)]);
        }
        Icon::Spool => {
            circle(0.0, 0.0, 0.72);
            circle(0.0, 0.0, 0.42);
            seg(vec2(0.3, 0.3), vec2(0.88, 0.88));
        }
        Icon::Needle => {
            seg(vec2(-0.8, 0.8), vec2(0.42, -0.42));
            circle(0.52, -0.52, 0.2);
            seg(vec2(0.62, -0.42), vec2(0.9, 0.15));
        }
        Icon::Pickaxe => {
            arc(0.0, 0.0, 0.85, 3.35, 6.0);
            seg(vec2(0.15, -0.35), vec2(0.6, 0.85));
        }
        Icon::Sword => {
            line(&[
                vec2(0.0, -0.95),
                vec2(0.16, -0.5),
                vec2(0.16, 0.22),
                vec2(-0.16, 0.22),
                vec2(-0.16, -0.5),
                vec2(0.0, -0.95),
            ]);
            seg(vec2(-0.5, 0.22), vec2(0.5, 0.22));
            seg(vec2(0.0, 0.22), vec2(0.0, 0.78));
            disc(0.0, 0.86, 0.12);
        }
        Icon::Shield => {
            line(&[
                vec2(0.0, -0.9),
                vec2(0.68, -0.62),
                vec2(0.68, 0.05),
                vec2(0.0, 0.9),
                vec2(-0.68, 0.05),
                vec2(-0.68, -0.62),
                vec2(0.0, -0.9),
            ]);
        }
        Icon::Bow => {
            arc(0.15, 0.0, 0.85, -1.35, 1.35);
            seg(vec2(0.33, -0.83), vec2(0.33, 0.83));
        }
        Icon::Potion => {
            painter.rect_stroke(r(-0.24, -0.85, 0.24, -0.42), Rounding::ZERO, stroke);
            circle(0.0, 0.22, 0.6);
            seg(vec2(-0.42, 0.32), vec2(0.42, 0.32));
            seg(vec2(-0.34, -0.85), vec2(0.34, -0.85));
        }
        Icon::Dragon => {
            line(&[
                vec2(-0.72, 0.1),
                vec2(-0.66, -0.32),
                vec2(-0.3, -0.5),
                vec2(-0.12, -0.9),
                vec2(0.18, -0.5),
                vec2(0.62, -0.28),
                vec2(0.85, 0.05),
                vec2(0.4, 0.1),
                vec2(0.6, 0.5),
                vec2(0.12, 0.32),
                vec2(-0.35, 0.6),
                vec2(-0.7, 0.42),
                vec2(-0.72, 0.1),
            ]);
            disc(-0.38, -0.18, 0.09);
        }
        Icon::Castle => {
            painter.rect_stroke(r(-0.8, -0.05, 0.8, 0.85), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.8, -0.5, -0.45, -0.05), Rounding::ZERO, stroke);
            painter.rect_stroke(r(-0.17, -0.5, 0.17, -0.05), Rounding::ZERO, stroke);
            painter.rect_stroke(r(0.45, -0.5, 0.8, -0.05), Rounding::ZERO, stroke);
            arc(0.0, 0.85, 0.28, std::f32::consts::PI, std::f32::consts::TAU);
        }
        Icon::Wheat => {
            seg(vec2(0.0, 0.9), vec2(0.0, -0.35));
            for y in [0.42, 0.08, -0.26] {
                seg(vec2(0.0, y), vec2(0.32, y - 0.3));
                seg(vec2(0.0, y), vec2(-0.32, y - 0.3));
            }
            line(&[
                vec2(0.0, -0.9),
                vec2(0.16, -0.5),
                vec2(0.0, -0.3),
                vec2(-0.16, -0.5),
                vec2(0.0, -0.9),
            ]);
        }
        Icon::Alien => {
            line(&[
                vec2(-0.62, -0.05),
                vec2(-0.36, -0.62),
                vec2(0.36, -0.62),
                vec2(0.62, -0.05),
            ]);
            painter.rect_stroke(r(-0.62, -0.05, 0.62, 0.35), Rounding::ZERO, stroke);
            line(&[vec2(-0.4, -0.62), vec2(-0.55, -0.9)]);
            line(&[vec2(0.4, -0.62), vec2(0.55, -0.9)]);
            disc(-0.28, 0.12, 0.1);
            disc(0.28, 0.12, 0.1);
            seg(vec2(-0.42, 0.35), vec2(-0.55, 0.72));
            seg(vec2(0.42, 0.35), vec2(0.55, 0.72));
            seg(vec2(0.0, 0.35), vec2(0.0, 0.72));
        }
    }
}

/// Compact clickable icon button with a subtle hover plate. Returns the raw
/// response so call sites can attach tooltips.
pub fn icon_button(ui: &mut Ui, icon: Icon, color: Color32) -> Response {
    let side = 22.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_responsive(response.id, response.hovered());
    if hover > 0.02 {
        ui.painter().rect_filled(
            rect,
            Rounding::ZERO,
            with_alpha(Color32::WHITE, (12.0 * hover) as u8),
        );
    }
    draw(
        ui.painter(),
        icon,
        rect.shrink(4.5),
        lerp_color(color, Color32::WHITE, hover * 0.55),
        1.4,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Draw an icon at an exact center inside an already-allocated layout rect
/// (used by widgets that paint outside the normal `Ui` flow).
pub fn draw_centered(painter: &Painter, icon: Icon, center: Pos2, size: f32, color: Color32) {
    draw(
        painter,
        icon,
        Rect::from_center_size(center, Vec2::splat(size)),
        color,
        1.4,
    );
}

/// Inline icon followed by a text label ("clock 3 h") in a centered row.
/// `font` sizes both the icon and the text so badges stay aligned.
pub fn icon_label(ui: &mut Ui, icon: Icon, text: &str, color: Color32, font: egui::FontId) {
    ui.horizontal(|ui| {
        let size = font.size + 1.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        draw(ui.painter(), icon, rect.shrink(1.0), color, 1.3);
        ui.label(egui::RichText::new(text).font(font).color(color));
    });
}

/// Flat button with a leading icon and an optional trailing icon (e.g. an
/// expand/collapse caret). `egui::Button` cannot paint vector icons, so the
/// plate, stroke and hover feedback are drawn here; `fill` and `text_color`
/// come from the call site to match the surrounding widget palette.
#[allow(clippy::too_many_arguments)]
pub fn icon_text_button(
    ui: &mut Ui,
    leading: Icon,
    trailing: Option<Icon>,
    text: &str,
    text_color: Color32,
    font: egui::FontId,
    fill: Color32,
) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), text_color);
    let icon = (font.size + 2.0).max(12.0);
    let gap = 6.0;
    let pad = vec2(10.0, 5.0);
    let extra = if trailing.is_some() { icon + gap } else { 0.0 };
    let size = vec2(
        galley.size().x + icon + gap + extra,
        galley.size().y.max(icon),
    ) + pad * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_responsive(response.id, response.hovered());
    let bg = lerp_color(fill, Color32::WHITE, hover * 0.07);
    let stroke = if response.hovered() {
        ui.visuals().widgets.hovered.bg_stroke
    } else {
        ui.visuals().widgets.inactive.bg_stroke
    };
    ui.painter().rect_filled(rect, Rounding::ZERO, bg);
    ui.painter().rect_stroke(rect, Rounding::ZERO, stroke);
    let cy = rect.center().y;
    let lead_center = pos2(rect.left() + pad.x + icon / 2.0, cy);
    draw(
        ui.painter(),
        leading,
        Rect::from_center_size(lead_center, Vec2::splat(icon)),
        text_color,
        1.35,
    );
    if let Some(trailing_icon) = trailing {
        let trail_center = pos2(rect.right() - pad.x - icon / 2.0, cy);
        draw(
            ui.painter(),
            trailing_icon,
            Rect::from_center_size(trail_center, Vec2::splat(icon * 0.85)),
            text_color,
            1.35,
        );
    }
    let text_left = rect.left() + pad.x + icon + gap;
    ui.painter().galley(
        pos2(text_left, cy - galley.size().y / 2.0),
        galley,
        text_color,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_rects_stay_inside_bounds() {
        // Every icon must fit inside its target rect: drawing with a tiny
        // rect and a normal painter context must not panic.
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for icon in Icon::ALL {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    draw(ui.painter(), *icon, rect, Color32::WHITE, 1.4);
                }
            });
        });
    }

    #[test]
    fn test_instance_presets_cover_core_keys() {
        for (key, _) in amc_core::types::InstanceIcon::ALL {
            assert!(
                instance_preset(key).is_some(),
                "missing vector icon for preset key `{key}`"
            );
        }
        assert_eq!(instance_preset("nope"), None);
    }

    #[test]
    fn test_loader_badges_are_stable() {
        for loader in [
            LoaderType::Vanilla,
            LoaderType::Fabric,
            LoaderType::Quilt,
            LoaderType::Forge,
            LoaderType::NeoForge,
            LoaderType::OptiFine,
        ] {
            // The badge must be a defined icon for every loader.
            assert!(Icon::ALL.contains(&loader_badge(loader)));
        }
    }
}
