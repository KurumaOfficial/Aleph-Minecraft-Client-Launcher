//! Post-exit session summary.
//!
//! The game itself is already closed when this window appears, so the
//! summary is built from the just-finished session plus the aggregate
//! counters that live in `instances.json`. Nothing is written here: the
//! app updates playtime first and hands over a snapshot.

use crate::theme::{
    with_alpha, BG_CARD, BORDER_DEFAULT, BORDER_SUBTLE, CARD, SUCCESS, TEXT_HEADING, TEXT_MUTED,
    TEXT_PRIMARY,
};
use crate::widgets::icons::{self, icon_button, Icon};
use amc_core::Language;
use chrono::{DateTime, Local, Utc};
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, Sense, Stroke, Ui};

/// Snapshot shown after a clean game exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummaryData {
    pub instance_name: String,
    /// Seconds of the session that just ended (never zero in real use).
    pub session_seconds: u64,
    /// All-time playtime after adding the session that just ended.
    pub total_minutes: u64,
    /// Number of sessions including the one that just ended.
    pub sessions: u64,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SessionSummaryAction {
    #[default]
    None,
    Close,
}

/// Compact duration for the summary: minutes, hours/minutes or days.
pub fn format_playtime(minutes: u64, lang: Language) -> String {
    if minutes < 60 {
        return format!("{minutes} {}", lang.inst_mins_suffix());
    }
    let days = minutes / (24 * 60);
    let hours = (minutes / 60) % 24;
    let mins = minutes % 60;
    if days > 0 {
        let mut parts = vec![format!("{days} {}", lang.sess_days_suffix())];
        if hours > 0 {
            parts.push(format!("{hours} {}", lang.inst_hours_suffix()));
        }
        if mins > 0 {
            parts.push(format!("{mins} {}", lang.inst_mins_suffix()));
        }
        parts.join(" ")
    } else if mins == 0 {
        format!("{hours} {}", lang.inst_hours_suffix())
    } else {
        format!(
            "{hours} {} {mins} {}",
            lang.inst_hours_suffix(),
            lang.inst_mins_suffix()
        )
    }
}

/// App-session duration from seconds (`<1 min` for very short runs).
pub fn format_session(seconds: u64, lang: Language) -> String {
    if seconds < 60 {
        lang.sess_less_than_minute().to_string()
    } else {
        format_playtime(seconds / 60, lang)
    }
}

/// Local timestamp with a compact, locale-independent numeric layout.
pub fn format_local_time(at: DateTime<Utc>) -> String {
    at.with_timezone(&Local)
        .format("%d.%m.%Y %H:%M")
        .to_string()
}

fn stat_tile(ui: &mut Ui, width: f32, label: &str, value: &str, icon: Icon) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 64.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(8.0), BG_CARD);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(8.0),
        Stroke::new(1.0_f32, BORDER_SUBTLE),
    );
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(
            pos2(rect.left() + 20.0, rect.top() + 20.0),
            vec2(13.0, 13.0),
        ),
        TEXT_MUTED,
        1.3,
    );
    ui.painter().text(
        pos2(rect.left() + 31.0, rect.top() + 20.0),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(10.5),
        TEXT_MUTED.into(),
    );
    let value_rect = Rect::from_min_max(
        pos2(rect.left() + 12.0, rect.top() + 34.0),
        pos2(rect.right() - 8.0, rect.bottom() - 6.0),
    );
    let galley = ui.painter().layout(
        value.to_owned(),
        egui::FontId::proportional(13.5),
        TEXT_HEADING.into(),
        value_rect.width(),
    );
    ui.painter().galley(
        pos2(value_rect.left(), value_rect.top()),
        galley,
        TEXT_HEADING.into(),
    );
}

/// Draw the summary window. The caller keeps the data around until the
/// returned action is `Close` (Escape, the close icon or the footer button).
pub fn show(
    ctx: &egui::Context,
    lang: Language,
    data: &SessionSummaryData,
) -> SessionSummaryAction {
    let mut action = SessionSummaryAction::None;
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return SessionSummaryAction::Close;
    }

    let screen = ctx.screen_rect();
    let width = (screen.width() - 80.0).clamp(360.0, 520.0);
    let content_w = width - 36.0;

    egui::Window::new(lang.sess_title())
        .id(egui::Id::new("session_summary_modal"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, -12.0))
        .frame(
            egui::Frame::none()
                .fill(CARD.into())
                .stroke(Stroke::new(1.0_f32, BORDER_DEFAULT))
                .rounding(Rounding::same(14.0))
                .inner_margin(egui::Margin::ZERO)
                .shadow(egui::epaint::Shadow {
                    offset: vec2(0.0, 8.0),
                    blur: 26.0,
                    spread: 0.0,
                    color: Color32::from_black_alpha(120),
                }),
        )
        .show(ctx, |ui| {
            ui.set_width(content_w);
            ui.set_max_width(content_w);

            // Header: success mark, title and a conventional close cross.
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                let (mark, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
                ui.painter()
                    .circle_filled(mark.center(), 14.0, with_alpha(SUCCESS, 30));
                icons::draw_centered(ui.painter(), Icon::Check, mark.center(), 15.0, SUCCESS);
                ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(lang.sess_title())
                            .font(egui::FontId::new(
                                16.0,
                                egui::FontFamily::Name("bold".into()),
                            ))
                            .color(TEXT_HEADING),
                    );
                    ui.label(
                        egui::RichText::new(lang.sess_complete())
                            .font(egui::FontId::proportional(11.5))
                            .color(TEXT_MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if icon_button(ui, Icon::Close, TEXT_MUTED)
                        .on_hover_text(lang.sess_close())
                        .clicked()
                    {
                        action = SessionSummaryAction::Close;
                    }
                    ui.add_space(2.0);
                });
            });
            ui.add_space(10.0);
            ui.separator();

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.set_max_width(content_w - 36.0);
                    let name_visible = crate::shells::short_name(&data.instance_name, 64);
                    let name = ui.add(
                        egui::Label::new(
                            egui::RichText::new(&name_visible)
                                .font(egui::FontId::proportional(17.0))
                                .strong()
                                .color(TEXT_HEADING),
                        )
                        .wrap(),
                    );
                    if data.instance_name.chars().count() > 64 {
                        name.on_hover_text(&data.instance_name);
                    }
                    ui.add_space(2.0);
                    let session_label = if data.sessions == 0 {
                        lang.sess_first_session().to_string()
                    } else {
                        lang.sess_session_number(data.sessions)
                    };
                    ui.label(
                        egui::RichText::new(session_label)
                            .font(egui::FontId::proportional(11.5))
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(18.0);
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(lang.sess_current_time())
                            .font(egui::FontId::proportional(11.5))
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(1.0);
                    ui.label(
                        egui::RichText::new(format_session(data.session_seconds, lang))
                            .font(egui::FontId::new(
                                29.0,
                                egui::FontFamily::Name("bold".into()),
                            ))
                            .color(TEXT_PRIMARY),
                    );
                });
                ui.add_space(18.0);
            });

            ui.add_space(14.0);
            let gap = 8.0;
            let tile_w = ((content_w - 36.0 - gap * 2.0) / 3.0).max(92.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                stat_tile(
                    ui,
                    tile_w,
                    lang.sess_total_time(),
                    &format_playtime(data.total_minutes, lang),
                    Icon::Clock,
                );
                ui.add_space(gap);
                stat_tile(
                    ui,
                    tile_w,
                    lang.sess_sessions(),
                    &data.sessions.to_string(),
                    Icon::Play,
                );
                ui.add_space(gap);
                let last = data
                    .last_played
                    .map(format_local_time)
                    .unwrap_or_else(|| lang.inst_never_played().to_string());
                stat_tile(ui, tile_w, lang.sess_last_session(), &last, Icon::Hourglass);
                ui.add_space(18.0);
            });

            if data.sessions == 0 {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.label(
                        egui::RichText::new(lang.sess_empty_history())
                            .font(egui::FontId::proportional(11.5))
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(18.0);
                });
            }
            ui.add_space(14.0);

            ui.separator();
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(18.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(lang.sess_close())
                                    .font(egui::FontId::proportional(12.5)),
                            )
                            .min_size(vec2(92.0, 30.0)),
                        )
                        .clicked()
                    {
                        action = SessionSummaryAction::Close;
                    }
                });
            });
            ui.add_space(8.0);
        });

    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_session_short_runs() {
        assert_eq!(format_session(0, Language::English), "<1 min");
        assert_eq!(format_session(59, Language::Russian), "<1 мин");
        assert_eq!(format_session(60, Language::English), "1 min");
        assert_eq!(format_session(3_599, Language::English), "59 min");
        assert_eq!(format_session(3_600, Language::Russian), "1 ч");
        assert_eq!(format_session(5_400, Language::Ukrainian), "1 год 30 хв");
    }

    #[test]
    fn test_format_playtime_days_and_spacing() {
        assert_eq!(format_playtime(0, Language::English), "0 min");
        assert_eq!(format_playtime(59, Language::Russian), "59 мин");
        assert_eq!(format_playtime(60, Language::Russian), "1 ч");
        assert_eq!(format_playtime(185, Language::Ukrainian), "3 год 5 хв");
        assert_eq!(format_playtime(1_440, Language::English), "1 d");
        assert_eq!(format_playtime(1_500, Language::English), "1 d 1 h");
        assert_eq!(format_playtime(1_545, Language::Russian), "1 д 1 ч 45 мин");
    }

    #[test]
    fn test_local_time_format_is_stable() {
        let at = Utc.with_ymd_and_hms(2026, 10, 29, 8, 15, 0).unwrap();
        let text = format_local_time(at);
        assert_eq!(text.len(), 16, "{text}");
        assert_eq!(&text[2..3], ".");
        assert_eq!(&text[5..6], ".");
        assert_eq!(&text[10..11], " ");
        assert_eq!(&text[13..14], ":");
    }
}
