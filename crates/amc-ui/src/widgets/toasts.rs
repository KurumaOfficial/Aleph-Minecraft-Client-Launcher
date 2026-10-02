use crate::theme::{BG_ELEVATED, BORDER_DEFAULT, RUBY_LIGHT, TEXT_PRIMARY};
use egui::{vec2, Align2, Color32, Rounding, Sense, Stroke};
use std::time::{Duration, Instant};

/// In-app toast notifications (CONCEPT "Уведомления", P10).
/// Toasts auto-expire; per-type switches live in config (`NotifSettings`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Updates,
    Downloads,
    Launcher,
}

impl ToastKind {
    fn enabled(&self, cfg: &amc_core::config::NotifSettings) -> bool {
        match self {
            Self::Updates => cfg.updates,
            Self::Downloads => cfg.downloads,
            Self::Launcher => cfg.launcher,
        }
    }

    fn accent(&self) -> Color32 {
        match self {
            Self::Updates => RUBY_LIGHT,
            Self::Downloads => Color32::from_rgb(0x50, 0xB0, 0x50),
            Self::Launcher => Color32::from_rgb(0x70, 0x90, 0xC8),
        }
    }
}

pub struct Toast {
    text: String,
    kind: ToastKind,
    until: Instant,
}

pub fn push_toast(
    toasts: &mut Vec<Toast>,
    cfg: &amc_core::config::NotifSettings,
    kind: ToastKind,
    text: String,
) {
    if !kind.enabled(cfg) {
        return;
    }
    toasts.retain(|t| t.kind != kind || t.text != text);
    toasts.push(Toast {
        text,
        kind,
        until: Instant::now() + Duration::from_secs(6),
    });
    toasts.truncate(4);
}

/// Top-right stacked toasts. Call every frame; expired ones are dropped.
pub fn show_toasts(ctx: &egui::Context, toasts: &mut Vec<Toast>) {
    toasts.retain(|t| Instant::now() < t.until);
    if toasts.is_empty() {
        return;
    }
    egui::Area::new(egui::Id::new("toasts"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            ctx.screen_rect().right() - 330.0,
            ctx.screen_rect().top() + 88.0,
        ))
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for toast in toasts.iter() {
                    let (rect, _) = ui.allocate_exact_size(vec2(300.0, 44.0), Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::ZERO, BG_ELEVATED);
                    ui.painter().rect_stroke(
                        rect,
                        Rounding::ZERO,
                        Stroke::new(1.0_f32, BORDER_DEFAULT),
                    );
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            rect.left_top(),
                            egui::pos2(rect.left() + 3.0, rect.bottom()),
                        ),
                        Rounding::ZERO,
                        toast.kind.accent(),
                    );
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        &toast.text,
                        egui::FontId::proportional(12.0),
                        TEXT_PRIMARY,
                    );
                }
            });
        });
}
