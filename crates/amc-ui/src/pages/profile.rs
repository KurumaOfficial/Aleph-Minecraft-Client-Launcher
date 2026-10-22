use amc_auth::{Account, AccountType};
use amc_core::Language;
use egui::{
    pos2, vec2, Color32, FontFamily, FontId, Rect, Rounding, ScrollArea, Sense, Stroke,
    TextureOptions, Ui,
};
use image::RgbaImage;
use std::time::{Duration, Instant};

use crate::theme::{
    self, lerp_color, with_alpha, BG_ELEVATED, BORDER_DEFAULT, MODRINTH_SURFACE,
    MODRINTH_SURFACE_HIGH, MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST, MODRINTH_TEXT_SECONDARY,
    TEXT_HEADING, TEXT_MUTED,
};
use crate::widgets::icons::{self, Icon};
use crate::widgets::modrinth_ui::{self, ModrinthButtonKind};
use crate::widgets::pixel;

/// Page language for the player profile. Aleph keeps its compact original
/// composition; Modrinth follows the reference app's page-header layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProfileSkin {
    #[default]
    Amc,
    Modrinth,
    Millida,
}

/// Player profile tab (CONCEPT "Профиль игрока", P7): nickname, skin and
/// cape in one place. Editing reuses the login modal (nickname) and the
/// Skins tab, so this page stays a thin honest summary.
#[derive(Default)]
pub struct ProfilePage {
    tex: Option<egui::TextureHandle>,
    tex_key: Option<(bool, usize, usize, bool)>,
    copied_at: Option<Instant>,
}

pub enum ProfileAction {
    None,
    OpenLogin,
    OpenSkins,
    OpenAccounts,
}

impl ProfilePage {
    fn ensure_tex(
        &mut self,
        ctx: &egui::Context,
        skin: Option<&RgbaImage>,
        slim: bool,
        circular: bool,
    ) {
        let key = (
            slim,
            skin.map_or(0, |s| s.as_raw().as_ptr() as usize),
            skin.map_or(0, |s| s.as_raw().len()),
            circular,
        );
        if self.tex_key == Some(key) {
            return;
        }
        let head = crate::pages::skins::extract_head_avatar(skin, slim);
        let image = if circular { circular_head(&head) } else { head };
        self.tex = Some(ctx.load_texture("profile_avatar", image, TextureOptions::NEAREST));
        self.tex_key = Some(key);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        skin: Option<&RgbaImage>,
        slim: bool,
        lang: Language,
        page_skin: ProfileSkin,
    ) -> ProfileAction {
        if page_skin == ProfileSkin::Modrinth {
            return self.show_modrinth(ui, account, skin, slim, lang);
        }
        if page_skin == ProfileSkin::Millida {
            return self.show_millida(ui, account, skin, slim, lang);
        }

        let mut action = ProfileAction::None;
        self.ensure_tex(ui.ctx(), skin, slim, false);

        ui.add_space(20.0);
        ui.label(
            egui::RichText::new(lang.profile_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            // Avatar card.
            let (av_rect, _) = ui.allocate_exact_size(vec2(120.0, 120.0), Sense::hover());
            ui.painter()
                .rect_filled(av_rect, crate::theme::rounding(), BG_ELEVATED);
            ui.painter().rect_stroke(
                av_rect,
                crate::theme::rounding(),
                egui::Stroke::new(1.0_f32, BORDER_DEFAULT),
            );
            if let Some(tex) = &self.tex {
                ui.painter().image(
                    tex.id(),
                    av_rect.shrink(8.0),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            ui.add_space(20.0);

            ui.vertical(|ui| {
                let name = account.map(|a| a.username.as_str()).unwrap_or("—");
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(lang.profile_nickname())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(name)
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(TEXT_HEADING),
                    );
                    if let Some(acc) = account {
                        super::super::modals::identity::account_tag(ui, acc.account_type, lang);
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(lang.skins_title())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    if ui.button(lang.profile_title()).clicked() {
                        action = ProfileAction::OpenSkins;
                    }
                    if ui.button(lang.bottom_btn_login()).clicked() {
                        action = ProfileAction::OpenLogin;
                    }
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(lang.profile_cape_none())
                        .font(egui::FontId::proportional(12.0))
                        .color(TEXT_MUTED),
                );
            });
        });

        if let Some(acc) = account {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(lang.profile_uuid_label(&acc.uuid))
                    .font(egui::FontId::monospace(11.0))
                    .color(TEXT_MUTED),
            );
        }

        action
    }

    fn show_millida(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        skin: Option<&RgbaImage>,
        slim: bool,
        lang: Language,
    ) -> ProfileAction {
        self.ensure_tex(ui.ctx(), skin, slim, false);
        let mut action = ProfileAction::None;
        let now = Instant::now();
        let copied = self
            .copied_at
            .is_some_and(|at| now.duration_since(at) < Duration::from_millis(1500));
        if self.copied_at.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    let (mark, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                    pixel::plate(
                        ui.painter(),
                        mark,
                        3.0,
                        theme::ACCENT.into(),
                        theme::ACCENT_LIGHT.into(),
                    );
                    ui.label(
                        egui::RichText::new(lang.profile_title())
                            .font(theme::display(26.0))
                            .color(theme::HEAD),
                    );
                });
                ui.add_space(12.0);

                let width = ui.available_width();
                let (card, _) = ui.allocate_exact_size(vec2(width, 176.0), Sense::hover());
                pixel::plate(
                    ui.painter(),
                    card,
                    10.0,
                    theme::BG_CARD.into(),
                    theme::BORDER_STRONG.into(),
                );
                let avatar = Rect::from_min_size(
                    pos2(card.left() + 20.0, card.top() + 24.0),
                    vec2(96.0, 96.0),
                );
                pixel::plate(
                    ui.painter(),
                    avatar,
                    8.0,
                    theme::BG.into(),
                    theme::ACCENT.into(),
                );
                if let Some(tex) = &self.tex {
                    ui.painter().image(
                        tex.id(),
                        avatar.shrink(4.0),
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                let text_x = avatar.right() + 20.0;
                let name = account
                    .map(|a| a.username.as_str())
                    .unwrap_or_else(|| lang.modrinth_no_account());
                ui.painter().text(
                    pos2(text_x, card.top() + 38.0),
                    egui::Align2::LEFT_CENTER,
                    name,
                    theme::display(24.0),
                    theme::HEAD.into(),
                );
                if let Some(acc) = account {
                    let label = account_type_label(acc, lang);
                    let badge_w = 72.0;
                    let badge = Rect::from_min_size(
                        pos2(
                            text_x
                                + ui.painter()
                                    .layout_no_wrap(
                                        name.to_owned(),
                                        theme::display(24.0),
                                        Color32::PLACEHOLDER,
                                    )
                                    .size()
                                    .x
                                + 10.0,
                            card.top() + 27.0,
                        ),
                        vec2(badge_w, 22.0),
                    );
                    pixel::plate(
                        ui.painter(),
                        badge,
                        4.0,
                        theme::BG.into(),
                        theme::ACCENT.into(),
                    );
                    ui.painter().text(
                        badge.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0),
                        theme::ACCENT_LIGHT.into(),
                    );
                }

                let (status, dot): (&str, Color32) = match account {
                    Some(acc) if acc.account_type == AccountType::Offline => {
                        (lang.modrinth_profile_status_offline(), theme::MUTED.into())
                    }
                    Some(_) => (
                        lang.modrinth_profile_status_signed_in(),
                        theme::ACCENT.into(),
                    ),
                    None => (
                        lang.modrinth_profile_status_signed_out(),
                        theme::MUTED.into(),
                    ),
                };
                ui.painter()
                    .circle_filled(pos2(text_x + 4.0, card.top() + 69.0), 3.5, dot);
                ui.painter().text(
                    pos2(text_x + 13.0, card.top() + 69.0),
                    egui::Align2::LEFT_CENTER,
                    status,
                    egui::FontId::proportional(12.0),
                    theme::MUTED.into(),
                );

                let primary_label = if account.is_some() {
                    lang.modrinth_profile_change_skin()
                } else {
                    lang.bottom_btn_login()
                };
                let secondary_label = if account.is_some() {
                    lang.modrinth_profile_manage_account()
                } else {
                    lang.modrinth_profile_change_skin()
                };
                let primary =
                    Rect::from_min_size(pos2(text_x, card.top() + 104.0), vec2(156.0, 36.0));
                let secondary = Rect::from_min_size(
                    pos2(primary.right() + 8.0, card.top() + 104.0),
                    vec2(196.0, 36.0),
                );
                if millida_profile_button(
                    ui,
                    primary,
                    primary_label,
                    if account.is_some() {
                        Some(Icon::Palette)
                    } else {
                        Some(Icon::Person)
                    },
                    true,
                ) {
                    action = if account.is_some() {
                        ProfileAction::OpenSkins
                    } else {
                        ProfileAction::OpenLogin
                    };
                }
                if millida_profile_button(
                    ui,
                    secondary,
                    secondary_label,
                    if account.is_some() {
                        Some(Icon::Gear)
                    } else {
                        Some(Icon::Palette)
                    },
                    false,
                ) {
                    action = if account.is_some() {
                        ProfileAction::OpenAccounts
                    } else {
                        ProfileAction::OpenSkins
                    };
                }

                ui.add_space(18.0);
                ui.label(
                    egui::RichText::new(lang.modrinth_profile_details())
                        .font(theme::display(18.0))
                        .color(theme::HEAD),
                );
                ui.add_space(10.0);

                let row_h = 58.0;
                let (details, _) = ui.allocate_exact_size(vec2(width, row_h * 3.0), Sense::hover());
                pixel::plate(
                    ui.painter(),
                    details,
                    10.0,
                    theme::BG_CARD.into(),
                    theme::BORDER_STRONG.into(),
                );
                let username = account.map(|a| a.username.as_str()).unwrap_or("—");
                let username_row = Rect::from_min_size(details.min, vec2(width, row_h));
                paint_millida_detail_row(
                    ui,
                    username_row,
                    Icon::Person,
                    lang.profile_nickname(),
                    username,
                    false,
                );
                if let Some(acc) = account {
                    let label = account_type_label(acc, lang);
                    let badge = Rect::from_min_size(
                        pos2(username_row.right() - 88.0, username_row.center().y - 11.0),
                        vec2(74.0, 22.0),
                    );
                    pixel::plate(
                        ui.painter(),
                        badge,
                        4.0,
                        theme::BG.into(),
                        theme::ACCENT.into(),
                    );
                    ui.painter().text(
                        badge.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0),
                        theme::ACCENT_LIGHT.into(),
                    );
                }

                let uuid_row = username_row.translate(vec2(0.0, row_h));
                paint_millida_row_separator(ui, uuid_row);
                let uuid = account
                    .map(|a| a.uuid.as_str())
                    .filter(|uuid| !uuid.trim().is_empty());
                paint_millida_detail_row(
                    ui,
                    uuid_row,
                    Icon::Card,
                    lang.modrinth_profile_player_uuid(),
                    uuid.unwrap_or_else(|| lang.modrinth_profile_no_uuid()),
                    true,
                );
                if let Some(uuid) = uuid {
                    let copy_rect = Rect::from_center_size(
                        pos2(uuid_row.right() - 28.0, uuid_row.center().y),
                        vec2(32.0, 32.0),
                    );
                    let resp = ui
                        .interact(copy_rect, ui.id().with("millida_copy_uuid"), Sense::click())
                        .on_hover_text(lang.modrinth_profile_copy_uuid());
                    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    icons::draw(
                        ui.painter(),
                        if copied { Icon::Check } else { Icon::Clipboard },
                        Rect::from_center_size(copy_rect.center(), vec2(15.0, 15.0)),
                        if copied {
                            theme::ACCENT.into()
                        } else {
                            lerp_color(theme::MUTED, theme::HEAD, hover)
                        },
                        1.4,
                    );
                    if copied {
                        ui.painter().text(
                            pos2(copy_rect.left() - 8.0, copy_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            lang.modrinth_profile_copied(),
                            egui::FontId::proportional(10.5),
                            theme::ACCENT.into(),
                        );
                    }
                    if resp.clicked() {
                        ui.ctx().copy_text(uuid.to_owned());
                        self.copied_at = Some(Instant::now());
                    }
                }

                let model_row = uuid_row.translate(vec2(0.0, row_h));
                paint_millida_row_separator(ui, model_row);
                paint_millida_detail_row(
                    ui,
                    model_row,
                    Icon::Diamond,
                    lang.modrinth_profile_skin_model(),
                    if slim {
                        lang.skins_model_slim()
                    } else {
                        lang.skins_model_classic()
                    },
                    false,
                );
                icons::draw(
                    ui.painter(),
                    Icon::ChevronRight,
                    Rect::from_center_size(
                        pos2(model_row.right() - 26.0, model_row.center().y),
                        vec2(14.0, 14.0),
                    ),
                    theme::MUTED,
                    1.4,
                );
                let model_resp = ui.interact(
                    Rect::from_min_max(
                        pos2(model_row.left(), model_row.top()),
                        pos2(model_row.right() - 46.0, model_row.bottom()),
                    ),
                    ui.id().with("millida_profile_skin_row"),
                    Sense::click(),
                );
                if model_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if model_resp.clicked() {
                    action = ProfileAction::OpenSkins;
                }

                ui.add_space(18.0);
            });

        action
    }

    fn show_modrinth(
        &mut self,
        ui: &mut Ui,
        account: Option<&Account>,
        skin: Option<&RgbaImage>,
        slim: bool,
        lang: Language,
    ) -> ProfileAction {
        self.ensure_tex(ui.ctx(), skin, slim, true);
        let mut action = ProfileAction::None;
        let now = Instant::now();
        let copied = self
            .copied_at
            .is_some_and(|at| now.duration_since(at) < Duration::from_millis(1500));
        if self.copied_at.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal_top(|ui| {
                    let (avatar, avatar_resp) =
                        ui.allocate_exact_size(vec2(96.0, 96.0), Sense::click());
                    ui.painter().circle_filled(
                        avatar.center(),
                        avatar.width() / 2.0,
                        MODRINTH_SURFACE_HIGH,
                    );
                    if let Some(tex) = &self.tex {
                        ui.painter().image(
                            tex.id(),
                            avatar,
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ui.painter().circle_stroke(
                        avatar.center(),
                        avatar.width() / 2.0 - 0.5,
                        Stroke::new(1.0_f32, with_alpha(Color32::WHITE, 38)),
                    );
                    if avatar_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if avatar_resp.clicked() {
                        action = ProfileAction::OpenSkins;
                    }

                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            let name = account
                                .map(|a| a.username.as_str())
                                .unwrap_or_else(|| lang.modrinth_no_account());
                            ui.label(
                                egui::RichText::new(name)
                                    .font(egui::FontId::proportional(24.0))
                                    .strong()
                                    .color(MODRINTH_TEXT_CONTRAST),
                            );
                            if let Some(acc) = account {
                                let accented = acc.account_type != AccountType::Offline;
                                modrinth_ui::pill(ui, account_type_label(acc, lang), accented);
                            }
                        });

                        ui.add_space(5.0);
                        let (status, dot) = account_status(account, lang);
                        ui.horizontal(|ui| {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 3.5, dot);
                            ui.label(
                                egui::RichText::new(status)
                                    .font(egui::FontId::proportional(13.0))
                                    .color(MODRINTH_TEXT_SECONDARY),
                            );
                        });

                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            if account.is_some() {
                                if modrinth_ui::button(
                                    ui,
                                    lang.modrinth_profile_change_skin(),
                                    Some(Icon::Palette),
                                    ModrinthButtonKind::Primary,
                                    0.0,
                                )
                                .clicked()
                                {
                                    action = ProfileAction::OpenSkins;
                                }
                                if modrinth_ui::button(
                                    ui,
                                    lang.modrinth_profile_manage_account(),
                                    Some(Icon::Gear),
                                    ModrinthButtonKind::Secondary,
                                    0.0,
                                )
                                .clicked()
                                {
                                    action = ProfileAction::OpenAccounts;
                                }
                            } else {
                                if modrinth_ui::button(
                                    ui,
                                    lang.bottom_btn_login(),
                                    Some(Icon::Person),
                                    ModrinthButtonKind::Primary,
                                    0.0,
                                )
                                .clicked()
                                {
                                    action = ProfileAction::OpenLogin;
                                }
                                if modrinth_ui::button(
                                    ui,
                                    lang.modrinth_profile_change_skin(),
                                    Some(Icon::Palette),
                                    ModrinthButtonKind::Secondary,
                                    0.0,
                                )
                                .clicked()
                                {
                                    action = ProfileAction::OpenSkins;
                                }
                            }
                        });
                    });
                });

                ui.add_space(18.0);
                modrinth_ui::divider(ui);
                ui.add_space(18.0);

                ui.label(
                    egui::RichText::new(lang.modrinth_profile_details())
                        .font(egui::FontId::proportional(18.0))
                        .strong()
                        .color(MODRINTH_TEXT_CONTRAST),
                );
                ui.add_space(10.0);

                let width = ui.available_width();
                let row_h = 58.0;
                let (card, _) = ui.allocate_exact_size(vec2(width, row_h * 3.0), Sense::hover());
                ui.painter()
                    .rect_filled(card, Rounding::same(14.0), MODRINTH_SURFACE);
                ui.painter().rect_stroke(
                    card,
                    Rounding::same(14.0),
                    Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
                );

                let username = account.map(|a| a.username.as_str()).unwrap_or("—");
                let username_row = Rect::from_min_size(card.min, vec2(width, row_h));
                paint_detail_row(
                    ui,
                    username_row,
                    Icon::Person,
                    lang.profile_nickname(),
                    username,
                    false,
                );
                if let Some(acc) = account {
                    paint_type_badge(
                        ui,
                        username_row,
                        account_type_label(acc, lang),
                        acc.account_type != AccountType::Offline,
                    );
                }

                let uuid_row = username_row.translate(vec2(0.0, row_h));
                paint_row_separator(ui, uuid_row);
                let uuid = account
                    .map(|a| a.uuid.as_str())
                    .filter(|uuid| !uuid.trim().is_empty());
                paint_detail_row(
                    ui,
                    uuid_row,
                    Icon::Card,
                    lang.modrinth_profile_player_uuid(),
                    uuid.unwrap_or_else(|| lang.modrinth_profile_no_uuid()),
                    true,
                );
                if let Some(uuid) = uuid {
                    let copy_rect = Rect::from_center_size(
                        pos2(uuid_row.right() - 30.0, uuid_row.center().y),
                        vec2(34.0, 34.0),
                    );
                    let resp = ui
                        .interact(copy_rect, ui.id().with("profile_copy_uuid"), Sense::click())
                        .on_hover_text(lang.modrinth_profile_copy_uuid());
                    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    if hover > 0.01 {
                        ui.painter().rect_filled(
                            copy_rect,
                            Rounding::same(8.0),
                            with_alpha(MODRINTH_SURFACE_HIGH, (hover * 210.0) as u8),
                        );
                    }
                    icons::draw(
                        ui.painter(),
                        if copied { Icon::Check } else { Icon::Clipboard },
                        Rect::from_center_size(copy_rect.center(), vec2(16.0, 16.0)),
                        if copied {
                            theme::MODRINTH_GREEN
                        } else {
                            lerp_color(MODRINTH_TEXT_SECONDARY, MODRINTH_TEXT_CONTRAST, hover)
                        },
                        1.4,
                    );
                    if copied {
                        ui.painter().text(
                            pos2(copy_rect.left() - 8.0, copy_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            lang.modrinth_profile_copied(),
                            FontId::proportional(11.0),
                            theme::MODRINTH_GREEN,
                        );
                    }
                    if resp.clicked() {
                        ui.ctx().copy_text(uuid.to_owned());
                        self.copied_at = Some(Instant::now());
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }

                let model_row = uuid_row.translate(vec2(0.0, row_h));
                paint_row_separator(ui, model_row);
                paint_detail_row(
                    ui,
                    model_row,
                    Icon::Diamond,
                    lang.modrinth_profile_skin_model(),
                    if slim {
                        lang.skins_model_slim()
                    } else {
                        lang.skins_model_classic()
                    },
                    false,
                );
                icons::draw(
                    ui.painter(),
                    Icon::ChevronRight,
                    Rect::from_center_size(
                        pos2(model_row.right() - 28.0, model_row.center().y),
                        vec2(15.0, 15.0),
                    ),
                    MODRINTH_TEXT_SECONDARY,
                    1.4,
                );
                let model_resp = ui.interact(
                    Rect::from_min_max(
                        pos2(model_row.left(), model_row.top()),
                        pos2(model_row.right() - 48.0, model_row.bottom()),
                    ),
                    ui.id().with("profile_skin_row"),
                    Sense::click(),
                );
                if model_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if model_resp.clicked() {
                    action = ProfileAction::OpenSkins;
                }

                ui.add_space(16.0);
            });

        action
    }
}

fn paint_millida_detail_row(
    ui: &Ui,
    rect: Rect,
    icon: Icon,
    label: &str,
    value: &str,
    monospace: bool,
) {
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(rect.left() + 26.0, rect.center().y), vec2(16.0, 16.0)),
        theme::ACCENT,
        1.35,
    );
    ui.painter().text(
        pos2(rect.left() + 48.0, rect.center().y - 10.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(10.5),
        theme::MUTED.into(),
    );
    let font = if monospace {
        FontId::new(12.0, FontFamily::Monospace)
    } else {
        FontId::proportional(13.0)
    };
    ui.painter().text(
        pos2(rect.left() + 48.0, rect.center().y + 10.0),
        egui::Align2::LEFT_CENTER,
        value,
        font,
        theme::HEAD.into(),
    );
}

fn paint_millida_row_separator(ui: &Ui, row: Rect) {
    ui.painter().line_segment(
        [
            pos2(row.left() + 14.0, row.top()),
            pos2(row.right() - 14.0, row.top()),
        ],
        Stroke::new(1.0_f32, theme::BORDER_STRONG),
    );
}

fn millida_profile_button(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    icon: Option<Icon>,
    primary: bool,
) -> bool {
    let resp = ui.interact(
        rect,
        ui.id().with(("millida_profile_btn", label)),
        Sense::click(),
    );
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let (fill, outline, text) = if primary {
        (
            theme::lerp_color(theme::ACCENT, theme::ACCENT_LIGHT, hover),
            theme::ACCENT_LIGHT.into(),
            theme::BG.into(),
        )
    } else {
        (
            theme::lerp_color(theme::BG, theme::BG_HOVER, hover),
            theme::BORDER_STRONG.into(),
            theme::TEXT.into(),
        )
    };
    pixel::plate(ui.painter(), rect, 5.0, fill, outline);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::new(11.5, FontFamily::Name("bold".into())),
        Color32::PLACEHOLDER,
    );
    let icon_space = if icon.is_some() { 22.0 } else { 0.0 };
    let total = icon_space + galley.size().x;
    let left = rect.center().x - total / 2.0;
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(left + 8.0, rect.center().y), vec2(15.0, 15.0)),
            text,
            1.4,
        );
    }
    ui.painter().galley(
        pos2(left + icon_space, rect.center().y - galley.size().y / 2.0),
        galley,
        text,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn circular_head(head: &egui::ColorImage) -> egui::ColorImage {
    const OUT: usize = 96;
    let src_w = head.size[0].max(1);
    let src_h = head.size[1].max(1);
    let mut pixels = Vec::with_capacity(OUT * OUT);
    let center = OUT as f32 / 2.0;
    let radius = center - 1.0;
    for y in 0..OUT {
        for x in 0..OUT {
            let sx = x * src_w / OUT;
            let sy = y * src_h / OUT;
            let src = head.pixels[sy.min(src_h - 1) * src_w + sx.min(src_w - 1)];
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let edge = (radius + 1.0 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
            let alpha = (src.a() as f32 * edge) as u8;
            pixels.push(Color32::from_rgba_unmultiplied(
                src.r(),
                src.g(),
                src.b(),
                alpha,
            ));
        }
    }
    egui::ColorImage {
        size: [OUT, OUT],
        pixels,
    }
}

fn account_type_label(account: &Account, lang: Language) -> &'static str {
    match account.account_type {
        AccountType::Microsoft => lang.account_tag_ms(),
        AccountType::WetId => lang.account_tag_wetid(),
        AccountType::Offline => lang.account_tag_offline(),
    }
}

fn account_status(account: Option<&Account>, lang: Language) -> (&'static str, Color32) {
    match account {
        Some(acc) if acc.account_type == AccountType::Offline => (
            lang.modrinth_profile_status_offline(),
            MODRINTH_TEXT_SECONDARY,
        ),
        Some(_) => (
            lang.modrinth_profile_status_signed_in(),
            theme::MODRINTH_GREEN,
        ),
        None => (
            lang.modrinth_profile_status_signed_out(),
            theme::MODRINTH_MUTED,
        ),
    }
}

fn paint_detail_row(ui: &Ui, rect: Rect, icon: Icon, label: &str, value: &str, monospace: bool) {
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(rect.left() + 28.0, rect.center().y), vec2(17.0, 17.0)),
        MODRINTH_TEXT_SECONDARY,
        1.35,
    );
    ui.painter().text(
        pos2(rect.left() + 52.0, rect.center().y - 10.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        MODRINTH_TEXT_SECONDARY,
    );
    let font = if monospace {
        FontId::new(12.5, FontFamily::Monospace)
    } else {
        FontId::proportional(13.5)
    };
    ui.painter().text(
        pos2(rect.left() + 52.0, rect.center().y + 10.0),
        egui::Align2::LEFT_CENTER,
        value,
        font,
        MODRINTH_TEXT,
    );
}

fn paint_row_separator(ui: &Ui, row: Rect) {
    ui.painter().line_segment(
        [
            pos2(row.left() + 16.0, row.top()),
            pos2(row.right() - 16.0, row.top()),
        ],
        Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
    );
}

fn paint_type_badge(ui: &Ui, row: Rect, label: &str, accent: bool) {
    let font = FontId::new(10.5, FontFamily::Name("bold".into()));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let width = (galley.size().x + 20.0).max(58.0);
    let rect = Rect::from_center_size(
        pos2(row.right() - 18.0 - width / 2.0, row.center().y),
        vec2(width, 22.0),
    );
    let (fill, stroke, text) = if accent {
        (
            with_alpha(theme::MODRINTH_GREEN, 22),
            with_alpha(theme::MODRINTH_GREEN, 80),
            theme::MODRINTH_GREEN,
        )
    } else {
        (
            with_alpha(MODRINTH_SURFACE_HIGH, 150),
            MODRINTH_SURFACE_HIGH,
            MODRINTH_TEXT_SECONDARY,
        )
    };
    ui.painter().rect_filled(rect, Rounding::same(11.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(11.0), Stroke::new(1.0_f32, stroke));
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_head_keeps_center_and_clears_corners() {
        let head = crate::pages::skins::extract_head_avatar(None, false);
        let avatar = circular_head(&head);
        assert_eq!(avatar.size, [96, 96]);
        assert_eq!(avatar.pixels.len(), 96 * 96);
        assert_eq!(avatar.pixels[0].a(), 0);
        assert_eq!(avatar.pixels[95].a(), 0);
        assert!(avatar.pixels[48 * 96 + 48].a() > 0);
    }
}
