use crate::shells::short_name;
use crate::theme::{
    lerp_color, ACCENT_DIM, BG_CARD, BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, BORDER_STRONG, RUBY,
    RUBY_DIM, RUBY_LIGHT, SUCCESS, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::widgets::brand;
use crate::widgets::draw_custom_badge;
use crate::widgets::full::{ghost_link, image_cover, kicker, red_button, red_square_deco, tag};
use crate::widgets::icons::{self, Icon};
use amc_core::types::{GameVersion, ReleaseType};
use amc_minecraft::{FavoriteServerEntry, ServerStatus};
use egui::{vec2, Color32, ScrollArea, Sense, Stroke, TextEdit, Ui};
use image::RgbaImage;
use std::collections::{HashMap, HashSet};

/// Project page used by the LabyMod news card.
const PROJECT_URL: &str = "https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher";
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterTab {
    All,
    #[default]
    Releases,
    Snapshots,
    Betas,
    Alphas,
    Old,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Newest,
    Oldest,
}

pub struct HomePage {
    pub search_query: String,
    pub active_filter: FilterTab,
    pub sort_order: SortOrder,
    pub featured_server: Option<ServerStatus>,
    pub is_pinging: bool,
    pub direct_connect_request: Option<(String, u16)>,
    pub hero_play_request: bool,
    pub drawer_play_request: Option<String>,
    /// LabyMod Home: the launch button's secondary segment opens Instances.
    pub labymod_instances_request: bool,
    /// LabyMod Home: Friends card opens Profile (signed in) or Login.
    pub labymod_friends_request: bool,
    /// LabyMod Home: Quick Play server manager modal.
    pub labymod_servers_open: bool,
    drawer_open: bool,
    drawer_version: Option<String>,
    hero_texture: Option<egui::TextureHandle>,
    hero_skin3d: super::skins::Skin3d,
    hero_failed: bool,
    installed_cache: HashSet<String>,
    installed_for: usize,
    /// Memoized version filter (the full Mojang list is ~1000 entries):
    /// recomputed only when the query, tab, order or list changes —
    /// never per frame. First/last ids guard against same-length
    /// manifest swaps that a bare `len` key would miss.
    filter_cache: Vec<GameVersion>,
    filter_key: Option<(String, FilterTab, SortOrder, usize, String, String)>,
    // Server favorites (CONCEPT, P8): no shared browser, but the player's
    // own servers persist here and are injected into every instance.
    // Ping results are transient (keyed by address); the list itself is
    // owned by the app and persisted to favorites.json.
    fav_name_input: String,
    fav_addr_input: String,
    fav_error: Option<String>,
    pub fav_ping_request: bool,
    pub fav_pinging: bool,
    fav_pinged_for: Vec<String>,
    pub fav_status: HashMap<String, ServerStatus>,
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_filter: FilterTab::Releases,
            sort_order: SortOrder::Newest,
            featured_server: None,
            is_pinging: false,
            direct_connect_request: None,
            hero_play_request: false,
            drawer_play_request: None,
            labymod_instances_request: false,
            labymod_friends_request: false,
            labymod_servers_open: false,
            drawer_open: false,
            drawer_version: None,
            hero_texture: None,
            hero_skin3d: super::skins::Skin3d::default(),
            hero_failed: false,
            installed_cache: HashSet::new(),
            installed_for: usize::MAX,
            filter_cache: Vec::new(),
            filter_key: None,
            fav_name_input: String::new(),
            fav_addr_input: String::new(),
            fav_error: None,
            fav_ping_request: false,
            fav_pinging: false,
            fav_pinged_for: Vec::new(),
            fav_status: HashMap::new(),
        }
    }
}

/// Ambient data the Home page needs beyond the version list itself.
/// Bundled so `show` stays under the argument-count lint.
pub struct HomeContext<'a> {
    pub versions_dir: &'a Path,
    pub instances_count: usize,
    pub memory_mb: u32,
    /// LabyMod shells render the dark hero/friends/news layout.
    pub labymod: bool,
    /// Selected instance name, used as the hero profile name.
    pub profile_name: Option<&'a str>,
    pub account_name: Option<&'a str>,
    pub launch_version: Option<&'a str>,
    pub skin: Option<&'a RgbaImage>,
    pub slim: bool,
    pub launching: bool,
}

impl HomePage {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        ctx_data: &HomeContext,
        favorites: &mut Vec<FavoriteServerEntry>,
        lang: amc_core::Language,
    ) {
        if ctx_data.labymod {
            self.ensure_hero_texture(ui.ctx());
            self.show_labymod(ui, ctx_data, favorites, lang);
            self.show_labymod_servers_modal(ui.ctx(), favorites, lang);
            return;
        }
        self.refresh_installed(versions, ctx_data.versions_dir);
        self.ensure_hero_texture(ui.ctx());
        self.show_hero(
            ui,
            versions,
            selected_version,
            ctx_data.instances_count,
            ctx_data.memory_mb,
            lang,
        );

        ui.add_space(20.0);

        self.show_favorites(ui, favorites, lang);

        ui.add_space(20.0);

        // Section Header: Versions + Selected Badge
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.home_title())
                    .font(egui::FontId::proportional(26.0))
                    .strong()
                    .color(TEXT_HEADING),
            );

            if let Some(sel) = selected_version.as_deref() {
                ui.add_space(12.0);
                draw_custom_badge(ui, &lang.home_selected_badge(sel), RUBY);
            }
        });

        ui.add_space(14.0);

        // Control Row (Search + Sort)
        ui.horizontal(|ui| {
            // Search Input
            let search_width = (ui.available_width() - 170.0).max(180.0);
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text(egui::RichText::new(lang.home_search_hint()).color(TEXT_MUTED))
                .desired_width(search_width)
                .font(egui::FontId::proportional(13.0));

            ui.add(search_edit);

            ui.add_space(8.0);

            // Sort Toggle Button (label plus a painter-drawn caret: the
            // bundled fonts carry no â–¾/â–´ coverage in the UI face).
            let sort_label = match self.sort_order {
                SortOrder::Newest => lang.home_sort_newest(),
                SortOrder::Oldest => lang.home_sort_oldest(),
            };

            let (sort_rect, sort_resp) = ui.allocate_exact_size(vec2(140.0, 32.0), Sense::click());
            let sort_hover = ui
                .ctx()
                .animate_bool_responsive(sort_resp.id, sort_resp.hovered());
            let sort_bg = lerp_color(BG_ELEVATED, BG_HOVER, sort_hover);
            let sort_stroke = lerp_color(BORDER_DEFAULT, RUBY, sort_hover);
            let sort_text = lerp_color(TEXT_PRIMARY, Color32::WHITE, sort_hover);

            ui.painter()
                .rect_filled(sort_rect, crate::theme::rounding(), sort_bg);
            ui.painter().rect_stroke(
                sort_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, sort_stroke),
            );
            let sort_galley = ui.painter().layout_no_wrap(
                sort_label.to_owned(),
                egui::FontId::proportional(12.0),
                sort_text,
            );
            let caret = match self.sort_order {
                SortOrder::Newest => Icon::CaretDown,
                SortOrder::Oldest => Icon::CaretUp,
            };
            let content_w = sort_galley.size().x + 16.0;
            ui.painter().galley(
                egui::pos2(
                    sort_rect.center().x - content_w / 2.0,
                    sort_rect.center().y - sort_galley.size().y / 2.0,
                ),
                sort_galley,
                sort_text,
            );
            icons::draw(
                ui.painter(),
                caret,
                egui::Rect::from_center_size(
                    egui::pos2(sort_rect.right() - 14.0, sort_rect.center().y),
                    vec2(10.0, 10.0),
                ),
                sort_text,
                1.3,
            );

            if sort_resp.clicked() {
                self.sort_order = match self.sort_order {
                    SortOrder::Newest => SortOrder::Oldest,
                    SortOrder::Oldest => SortOrder::Newest,
                };
            }
        });

        ui.add_space(10.0);

        // Filter Tabs
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
            self.filter_tab_btn(ui, lang.home_tab_all(), FilterTab::All);
            self.filter_tab_btn(ui, lang.home_tab_releases(), FilterTab::Releases);
            self.filter_tab_btn(ui, lang.home_tab_snapshots(), FilterTab::Snapshots);
            self.filter_tab_btn(ui, lang.home_tab_betas(), FilterTab::Betas);
            self.filter_tab_btn(ui, lang.home_tab_alphas(), FilterTab::Alphas);
            self.filter_tab_btn(ui, lang.home_tab_old(), FilterTab::Old);
        });

        ui.add_space(10.0);

        // Filter and sort version list (memoized: the full list is big,
        // recompute only when inputs change; cache hit is zero-alloc).
        let first_id = versions.first().map(|v| v.id.as_str()).unwrap_or("");
        let last_id = versions.last().map(|v| v.id.as_str()).unwrap_or("");
        let cache_valid = match &self.filter_key {
            Some((q, f, s, len, first, last)) => {
                *q == self.search_query
                    && *f == self.active_filter
                    && *s == self.sort_order
                    && *len == versions.len()
                    && first.as_str() == first_id
                    && last.as_str() == last_id
            }
            None => false,
        };
        if !cache_valid {
            let query = self.search_query.trim().to_lowercase();
            let mut filtered: Vec<GameVersion> = versions
                .iter()
                .filter(|v| {
                    let matches_filter = match self.active_filter {
                        FilterTab::All => true,
                        FilterTab::Releases => v.release_type == ReleaseType::Release,
                        FilterTab::Snapshots => v.release_type == ReleaseType::Snapshot,
                        FilterTab::Betas => v.release_type == ReleaseType::Beta,
                        FilterTab::Alphas => v.release_type == ReleaseType::Alpha,
                        FilterTab::Old => v.release_type == ReleaseType::Old,
                    };

                    let matches_search = if query.is_empty() {
                        true
                    } else {
                        v.id.to_lowercase().contains(&query)
                    };

                    matches_filter && matches_search
                })
                .cloned()
                .collect();

            if self.sort_order == SortOrder::Oldest {
                filtered.sort_by_key(|v| v.release_time);
            } else {
                filtered.sort_by_key(|v| std::cmp::Reverse(v.release_time));
            }
            self.filter_cache = filtered;
            self.filter_key = Some((
                self.search_query.clone(),
                self.active_filter,
                self.sort_order,
                versions.len(),
                first_id.to_string(),
                last_id.to_string(),
            ));
        }
        let filtered = &self.filter_cache;

        if filtered.is_empty() {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                if versions.is_empty() {
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(lang.home_loading_versions())
                            .font(egui::FontId::proportional(15.0))
                            .color(TEXT_MUTED),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(lang.home_empty())
                            .font(egui::FontId::proportional(15.0))
                            .color(TEXT_MUTED),
                    );
                }
            });
            return;
        }

        // Virtualized ScrollArea
        let row_height = 56.0;
        let num_rows = filtered.len();

        let mut drawer_id: Option<String> = None;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                ui.spacing_mut().item_spacing = vec2(0.0, 6.0);

                for idx in row_range {
                    let v = &filtered[idx];
                    let is_selected = selected_version.as_deref() == Some(&v.id);

                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::click());

                    let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                    let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

                    let base_bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                    let bg = lerp_color(base_bg, RUBY_DIM, sel_t);

                    let stroke_col =
                        lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, sel_t);
                    let border_width = if is_selected { 1.5_f32 } else { 1.0_f32 };

                    ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
                    ui.painter().rect_stroke(
                        rect,
                        crate::theme::rounding(),
                        Stroke::new(border_width, stroke_col),
                    );

                    // Active left indicator bar (Square motif)
                    if sel_t > 0.01 {
                        let bar_rect = egui::Rect::from_min_max(
                            rect.left_top(),
                            egui::pos2(rect.left() + 3.0 * sel_t, rect.bottom()),
                        );
                        ui.painter()
                            .rect_filled(bar_rect, crate::theme::rounding(), RUBY_LIGHT);
                    }

                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );

                    child.add_space(14.0);

                    // Version type icon
                    let icon = match v.release_type {
                        ReleaseType::Release => Icon::Package,
                        ReleaseType::Snapshot => Icon::Flask,
                        ReleaseType::Beta => Icon::Gear,
                        ReleaseType::Alpha => Icon::Hammer,
                        ReleaseType::Old => Icon::Doc,
                    };
                    let (icon_rect, _) =
                        child.allocate_exact_size(egui::Vec2::splat(18.0), Sense::hover());
                    icons::draw(
                        child.painter(),
                        icon,
                        icon_rect.shrink(1.0),
                        TEXT_MUTED,
                        1.4,
                    );
                    child.add_space(10.0);

                    // Version ID
                    let ver_color = lerp_color(
                        lerp_color(TEXT_HEADING, Color32::WHITE, hover_t),
                        Color32::WHITE,
                        sel_t,
                    );
                    child.label(
                        egui::RichText::new(&v.id)
                            .font(egui::FontId::proportional(15.0))
                            .strong()
                            .color(ver_color),
                    );

                    child.add_space(12.0);

                    // Release type badge
                    let badge_color = match v.release_type {
                        ReleaseType::Release => RUBY.into(),
                        ReleaseType::Snapshot => Color32::from_rgb(0xB8, 0x86, 0x0B),
                        _ => Color32::from_rgb(0x4A, 0x55, 0x68),
                    };
                    draw_custom_badge(&mut child, v.release_type.as_str(), badge_color);

                    // Release date on right
                    let date_str = v.release_time.format("%d.%m.%Y").to_string();
                    let date_width = 80.0;
                    let space = child.available_width() - date_width - 16.0;
                    if space > 0.0 {
                        child.add_space(space);
                    }

                    child.label(
                        egui::RichText::new(date_str)
                            .font(egui::FontId::proportional(12.0))
                            .color(lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t)),
                    );

                    if resp.clicked() {
                        *selected_version = Some(v.id.clone());
                        drawer_id = Some(v.id.clone());
                    }
                }
            });

        if let Some(id) = drawer_id {
            self.open_drawer(Some(id));
        }

        self.show_drawer(ui, versions, selected_version, lang);
    }

    pub fn open_drawer(&mut self, id: Option<String>) {
        self.drawer_version = id.clone();
        self.drawer_open = id.is_some();
    }

    pub fn close_drawer(&mut self) {
        self.drawer_open = false;
        self.drawer_version = None;
    }

    /// Rescan downloaded versions when the manifest list changes.
    fn refresh_installed(&mut self, versions: &[GameVersion], versions_dir: &Path) {
        if versions.len() == self.installed_for {
            return;
        }
        self.installed_for = versions.len();
        self.installed_cache.clear();
        let entries = match std::fs::read_dir(versions_dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let has_json = std::fs::read_dir(&dir).map(|inner| {
                inner.flatten().any(|f| {
                    f.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
            });
            if has_json.unwrap_or(false) {
                self.installed_cache
                    .insert(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    fn ensure_hero_texture(&mut self, ctx: &egui::Context) {
        if self.hero_texture.is_some() || self.hero_failed {
            return;
        }
        match image::load_from_memory(include_bytes!("../../../../assets/png/back_versions.png")) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                let pixels: Vec<Color32> = rgba
                    .pixels()
                    .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
                    .collect();
                let tex = ctx.load_texture(
                    "home_hero",
                    egui::ColorImage {
                        size: [w, h],
                        pixels,
                    },
                    egui::TextureOptions::LINEAR,
                );
                self.hero_texture = Some(tex);
            }
            Err(_) => self.hero_failed = true,
        }
    }
}

/// Default favorite name: explicit name wins, otherwise the host part.
/// Pure for tests.
fn fav_display_name(name: &str, host: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        host.to_string()
    } else {
        trimmed.to_string()
    }
}

impl HomePage {
    /// Server favorites section (CONCEPT, P8). Deliberately no "play"
    /// button: favorite servers are pre-registered in every instance's
    /// multiplayer menu instead of launching from here.
    fn show_favorites(
        &mut self,
        ui: &mut Ui,
        favorites: &mut Vec<FavoriteServerEntry>,
        lang: amc_core::Language,
    ) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.fav_title())
                    .font(egui::FontId::proportional(20.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Keep the refresh action off the panel edge at 960 px.
                ui.add_space(4.0);
                if ui.button(lang.fav_refresh()).clicked() {
                    self.request_fav_ping(favorites);
                }
            });
        });

        ui.add_space(8.0);

        // Add row.
        ui.horizontal(|ui| {
            let name_w = 150.0;
            let addr_w = (ui.available_width() - name_w - 110.0).max(140.0);
            ui.add(
                TextEdit::singleline(&mut self.fav_name_input)
                    .hint_text(egui::RichText::new(lang.fav_name_hint()).color(TEXT_MUTED))
                    .desired_width(name_w)
                    .font(egui::FontId::proportional(12.0)),
            );
            ui.add(
                TextEdit::singleline(&mut self.fav_addr_input)
                    .hint_text(egui::RichText::new(lang.fav_address_hint()).color(TEXT_MUTED))
                    .desired_width(addr_w)
                    .font(egui::FontId::proportional(12.0)),
            );
            if ui.button(lang.fav_add()).clicked() {
                self.fav_error = None;
                match amc_minecraft::parse_server_address(&self.fav_addr_input) {
                    Some((host, port)) => {
                        let address = if self.fav_addr_input.trim().contains(':') {
                            self.fav_addr_input.trim().to_string()
                        } else {
                            format!("{host}:{port}")
                        };
                        let name = fav_display_name(&self.fav_name_input, &host);
                        if let Some(existing) = favorites.iter_mut().find(|f| f.address == address)
                        {
                            existing.name = name;
                        } else {
                            favorites.push(FavoriteServerEntry { name, address });
                        }
                        self.fav_name_input.clear();
                        self.fav_addr_input.clear();
                        self.request_fav_ping(favorites);
                    }
                    None => {
                        self.fav_error = Some(lang.fav_invalid().to_string());
                    }
                }
            }
        });

        if let Some(err) = self.fav_error.as_ref() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(err).color(Color32::from_rgb(0xE8, 0x55, 0x70)));
        }

        ui.add_space(8.0);

        if favorites.is_empty() {
            ui.label(
                egui::RichText::new(lang.fav_empty())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
        }

        let mut remove_idx: Option<usize> = None;
        if !favorites.is_empty() {
            egui::ScrollArea::vertical()
                .id_salt("labymod_fav_list")
                .max_height(240.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (idx, fav) in favorites.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Fixed columns keep a long server name or address
                            // from running under the Remove button at 960 px.
                            let full = ui.available_width();
                            let remove_w = 72.0;
                            let name_w = (full * 0.32).clamp(96.0, 150.0);
                            let has_status = self.fav_status.contains_key(&fav.address);
                            let status_w = if has_status || self.fav_pinging {
                                18.0
                            } else {
                                0.0
                            };
                            let addr_w = (full - name_w - remove_w - status_w - 24.0).max(80.0);

                            ui.allocate_ui_with_layout(
                                vec2(name_w, 22.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let label = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&fav.name)
                                                .font(egui::FontId::proportional(13.0))
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        )
                                        .truncate(),
                                    );
                                    if fav.name.chars().count() > 20 {
                                        label.on_hover_text(&fav.name);
                                    }
                                },
                            );
                            ui.add_space(8.0);
                            ui.allocate_ui_with_layout(
                                vec2(addr_w, 22.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let label = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&fav.address)
                                                .font(egui::FontId::proportional(12.0))
                                                .color(TEXT_MUTED),
                                        )
                                        .truncate(),
                                    );
                                    if fav.address.chars().count() > 24 {
                                        label.on_hover_text(&fav.address);
                                    }
                                },
                            );
                            if let Some(status) = self.fav_status.get(&fav.address) {
                                let (dot_color, text) = if status.is_online {
                                    (
                                        SUCCESS,
                                        format!(
                                            "{} ms • {}/{} • {}",
                                            status.ping_ms,
                                            status.online_players,
                                            status.max_players,
                                            status.motd
                                        ),
                                    )
                                } else {
                                    (RUBY_LIGHT, status.motd.clone())
                                };
                                let (dot_rect, dot_resp) =
                                    ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                                icons::draw_centered(
                                    ui.painter(),
                                    Icon::Dot,
                                    dot_rect.center(),
                                    10.0,
                                    dot_color,
                                );
                                let _ = dot_resp.on_hover_text(text);
                            } else if self.fav_pinging {
                                ui.add(egui::Spinner::new().size(12.0));
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(lang.fav_remove()).clicked() {
                                        remove_idx = Some(idx);
                                    }
                                },
                            );
                        });
                    }
                });
        }
        if let Some(idx) = remove_idx {
            favorites.remove(idx);
            self.request_fav_ping(favorites);
        }

        // First paint (or list change) triggers one ping pass.
        // Allocation-free change check: length + ordered addresses.
        let fav_changed = favorites.len() != self.fav_pinged_for.len()
            || favorites
                .iter()
                .zip(self.fav_pinged_for.iter())
                .any(|(fav, pinged)| &fav.address != pinged);
        if !self.fav_pinging && fav_changed && !favorites.is_empty() {
            self.request_fav_ping(favorites);
        }
    }

    /// Arm a favorites ping pass; the app task fills `fav_status`.
    fn request_fav_ping(&mut self, favorites: &[FavoriteServerEntry]) {
        self.fav_pinged_for = favorites.iter().map(|f| f.address.clone()).collect();
        self.fav_pinging = true;
        self.fav_ping_request = true;
    }

    /// Full-style hero: cover art, red-square deco, display titles,
    /// big Play + ghost pick link, stats row.
    /// LabyMod Home: dark hero canvas with the selected skin, a large blue
    /// launch button and Quick Play, plus the account/friends/news column.
    fn show_labymod(
        &mut self,
        ui: &mut Ui,
        ctx: &HomeContext,
        favorites: &[FavoriteServerEntry],
        lang: amc_core::Language,
    ) {
        let area = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(area, egui::Rounding::ZERO, crate::theme::BG);
        let inner = area.shrink2(vec2(14.0, 14.0));
        ui.allocate_new_ui(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                let width = ui.available_width();
                let height = ui.available_height();
                let right = (width * 0.235).clamp(240.0, 300.0);
                let gap = 14.0;
                let main = (width - right - gap).max(360.0);
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(main, height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| self.show_labymod_hero(ui, main, height, ctx, favorites, lang),
                    );
                    ui.add_space(gap);
                    ui.allocate_ui_with_layout(
                        vec2(right, height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.show_labymod_side(
                                ui,
                                right,
                                height,
                                lang,
                                ctx.account_name.is_some(),
                            )
                        },
                    );
                });
            },
        );
    }

    fn show_labymod_hero(
        &mut self,
        ui: &mut Ui,
        width: f32,
        height: f32,
        ctx: &HomeContext,
        favorites: &[FavoriteServerEntry],
        lang: amc_core::Language,
    ) {
        let texture = self.hero_texture.clone();
        let (card, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(card, egui::Rounding::same(18.0), BG_CARD);
        if let Some(tex) = texture.as_ref() {
            rounded_image(painter, card, 18.0, tex, Color32::from_rgb(120, 124, 136));
        }
        painter.rect_filled(
            card,
            egui::Rounding::same(18.0),
            Color32::from_rgba_unmultiplied(7, 9, 14, 196),
        );
        painter.rect_stroke(
            card,
            egui::Rounding::same(18.0),
            Stroke::new(1.0_f32, BORDER_STRONG),
        );

        let cx = card.center().x;
        let viewer = egui::Rect::from_min_max(
            egui::pos2(cx - width * 0.24, card.top() + 58.0),
            egui::pos2(cx + width * 0.24, card.bottom() - 155.0),
        );
        painter.circle_filled(
            egui::pos2(cx, viewer.center().y + 24.0),
            (viewer.height() * 0.32).min(width * 0.20),
            crate::theme::with_alpha(crate::theme::ACCENT, 20),
        );
        self.hero_skin3d.show(ui, viewer, ctx.skin, ctx.slim);

        let name = ctx
            .account_name
            .or(ctx.profile_name)
            .unwrap_or_else(|| lang.legacy_guest());
        ui.painter().text(
            egui::pos2(cx, card.top() + 34.0),
            egui::Align2::CENTER_CENTER,
            short_name(name, 26),
            egui::FontId::proportional(29.0),
            Color32::WHITE,
        );

        let default_chip = egui::Rect::from_center_size(
            egui::pos2(cx + width * 0.19, card.bottom() - 134.0),
            vec2(100.0, 24.0),
        );
        ui.painter().rect_filled(
            default_chip,
            egui::Rounding::same(12.0),
            crate::theme::with_alpha(crate::theme::BG, 170),
        );
        icons::draw(
            ui.painter(),
            Icon::Dot,
            egui::Rect::from_center_size(
                egui::pos2(default_chip.left() + 16.0, default_chip.center().y),
                vec2(9.0, 9.0),
            ),
            Color32::from_rgb(0xE0, 0xC0, 0x60),
            1.5,
        );
        ui.painter().text(
            egui::pos2(default_chip.left() + 28.0, default_chip.center().y),
            egui::Align2::LEFT_CENTER,
            lang.labymod_default(),
            egui::FontId::proportional(10.5),
            TEXT_PRIMARY.into(),
        );

        let button_w = (width * 0.56).clamp(360.0, 560.0);
        let button = egui::Rect::from_center_size(
            egui::pos2(cx, card.bottom() - 91.0),
            vec2(button_w, 60.0),
        );
        let btn_resp = ui.interact(button, ui.id().with("labymod_launch"), Sense::click());
        let hover = ui
            .ctx()
            .animate_bool_responsive(btn_resp.id, btn_resp.hovered());
        let fill = crate::theme::lerp_color(
            crate::theme::ACCENT,
            crate::theme::ACCENT_LIGHT,
            hover * 0.5,
        );
        ui.painter()
            .rect_filled(button, egui::Rounding::same(14.0), fill);
        ui.painter().rect_stroke(
            button,
            egui::Rounding::same(14.0),
            Stroke::new(1.0_f32, crate::theme::ACCENT_LIGHT),
        );
        let split = button.right() - 54.0;
        ui.painter().line_segment(
            [
                egui::pos2(split, button.top() + 8.0),
                egui::pos2(split, button.bottom() - 8.0),
            ],
            Stroke::new(1.0_f32, crate::theme::with_alpha(Color32::WHITE, 55)),
        );
        icons::draw(
            ui.painter(),
            Icon::Play,
            egui::Rect::from_center_size(
                egui::pos2(button.left() + 28.0, button.center().y),
                vec2(20.0, 20.0),
            ),
            Color32::WHITE,
            1.8,
        );
        let version = ctx.launch_version.unwrap_or("1.20.1");
        let launch = if ctx.launching {
            lang.labymod_launching().to_string()
        } else {
            format!("{} {version}", lang.labymod_launch())
        };
        ui.painter().text(
            egui::pos2(button.center().x - 8.0, button.center().y - 9.0),
            egui::Align2::CENTER_CENTER,
            launch,
            egui::FontId::proportional(17.0),
            Color32::WHITE,
        );
        ui.painter().text(
            egui::pos2(button.center().x - 8.0, button.center().y + 12.0),
            egui::Align2::CENTER_CENTER,
            brand::APP_NAME,
            egui::FontId::proportional(10.0),
            crate::theme::with_alpha(Color32::WHITE, 175),
        );
        let swap = egui::Rect::from_min_max(
            egui::pos2(split + 1.0, button.top() + 1.0),
            egui::pos2(button.right() - 1.0, button.bottom() - 1.0),
        );
        let swap_resp = ui.interact(swap, ui.id().with("labymod_swap"), Sense::click());
        icons::draw(
            ui.painter(),
            Icon::Refresh,
            egui::Rect::from_center_size(swap.center(), vec2(20.0, 20.0)),
            if swap_resp.hovered() {
                Color32::WHITE
            } else {
                crate::theme::with_alpha(Color32::WHITE, 200)
            },
            1.7,
        );
        if swap_resp.clicked() {
            self.labymod_instances_request = true;
        } else if btn_resp.clicked() && !ctx.launching {
            self.hero_play_request = true;
        }
        if (btn_resp.hovered() || swap_resp.hovered()) && !ctx.launching {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let quick = egui::Rect::from_min_max(
            egui::pos2(card.left() + 14.0, card.bottom() - 50.0),
            egui::pos2(card.right() - 14.0, card.bottom() - 10.0),
        );
        ui.painter().rect_filled(
            quick,
            egui::Rounding::same(12.0),
            crate::theme::with_alpha(crate::theme::BG, 205),
        );
        ui.painter().rect_stroke(
            quick,
            egui::Rounding::same(12.0),
            Stroke::new(1.0_f32, crate::theme::with_alpha(BORDER_STRONG, 150)),
        );
        ui.painter().text(
            egui::pos2(quick.left() + 14.0, quick.center().y),
            egui::Align2::LEFT_CENTER,
            lang.labymod_quick_play(),
            egui::FontId::proportional(10.5),
            TEXT_MUTED.into(),
        );
        let mut x = quick.left() + 112.0;
        if favorites.is_empty() {
            ui.painter().text(
                egui::pos2(x, quick.center().y),
                egui::Align2::LEFT_CENTER,
                lang.labymod_no_servers(),
                egui::FontId::proportional(10.0),
                TEXT_MUTED.into(),
            );
        }
        for (idx, fav) in favorites.iter().take(5).enumerate() {
            let label = short_name(&fav.name, 18);
            let text_w = ui
                .painter()
                .layout_no_wrap(
                    label.clone(),
                    egui::FontId::proportional(10.0),
                    Color32::PLACEHOLDER,
                )
                .size()
                .x;
            let chip_w = (text_w + 34.0).clamp(76.0, 158.0);
            if x + chip_w > quick.right() - 50.0 {
                break;
            }
            let chip = egui::Rect::from_min_size(
                egui::pos2(x, quick.top() + 5.0),
                vec2(chip_w, quick.height() - 10.0),
            );
            let resp = ui.interact(chip, ui.id().with(("labymod_quick", idx)), Sense::click());
            ui.painter().rect_filled(
                chip,
                egui::Rounding::same(9.0),
                if resp.hovered() {
                    crate::theme::with_alpha(crate::theme::CARD_HOVER, 220)
                } else {
                    crate::theme::with_alpha(crate::theme::CARD, 190)
                },
            );
            icons::draw(
                ui.painter(),
                Icon::Globe,
                egui::Rect::from_center_size(
                    egui::pos2(chip.left() + 12.0, chip.center().y),
                    vec2(12.0, 12.0),
                ),
                crate::theme::ACCENT_LIGHT,
                1.4,
            );
            ui.painter().text(
                egui::pos2(chip.left() + 22.0, chip.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(10.0),
                TEXT_PRIMARY.into(),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            resp.clone()
                .on_hover_text(format!("{}\n{}", fav.name, fav.address));
            if resp.clicked() {
                if let Some((host, port)) = amc_minecraft::parse_server_address(&fav.address) {
                    self.direct_connect_request = Some((host, port));
                }
            }
            x += chip_w + 7.0;
        }

        // Add/manage saved servers. The manager reuses the shared favorites
        // form, so LabyMod users are not locked out of Quick Play editing.
        let add = egui::Rect::from_min_size(
            egui::pos2(quick.right() - 42.0, quick.top() + 5.0),
            vec2(30.0, quick.height() - 10.0),
        );
        let add_resp = ui.interact(add, ui.id().with("labymod_add_server"), Sense::click());
        ui.painter().rect_filled(
            add,
            egui::Rounding::same(9.0),
            if add_resp.hovered() {
                crate::theme::with_alpha(crate::theme::ACCENT, 45)
            } else {
                crate::theme::with_alpha(crate::theme::CARD, 190)
            },
        );
        icons::draw(
            ui.painter(),
            Icon::Plus,
            egui::Rect::from_center_size(add.center(), vec2(13.0, 13.0)),
            crate::theme::ACCENT_LIGHT,
            1.6,
        );
        if add_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if add_resp.clicked() {
            self.labymod_servers_open = true;
        }
    }

    fn show_labymod_servers_modal(
        &mut self,
        ctx: &egui::Context,
        favorites: &mut Vec<FavoriteServerEntry>,
        lang: amc_core::Language,
    ) {
        if !self.labymod_servers_open {
            return;
        }
        let mut open = self.labymod_servers_open;
        egui::Window::new(lang.labymod_quick_play())
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .frame(
                egui::Frame::none()
                    .fill(BG_CARD.into())
                    .stroke(Stroke::new(1.0_f32, BORDER_STRONG))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(18.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(460.0);
                self.show_favorites(ui, favorites, lang);
            });
        self.labymod_servers_open = open;
    }

    fn show_labymod_side(
        &mut self,
        ui: &mut Ui,
        width: f32,
        height: f32,
        lang: amc_core::Language,
        signed_in: bool,
    ) {
        // The account chip lives in the titlebar now (as in the reference);
        // the right column starts directly with Friends.
        let remaining = height.max(260.0);
        let friends_h = (remaining * 0.52).max(150.0);
        let news_h = (remaining - friends_h - 12.0).max(140.0);

        let (friends, friends_resp) =
            ui.allocate_exact_size(vec2(width, friends_h), Sense::click());
        let friends_hover = ui
            .ctx()
            .animate_bool_responsive(friends_resp.id, friends_resp.hovered());
        ui.painter().rect_filled(
            friends,
            egui::Rounding::same(16.0),
            lerp_color(BG_CARD, BG_HOVER, friends_hover),
        );
        ui.painter().rect_stroke(
            friends,
            egui::Rounding::same(16.0),
            Stroke::new(1.0_f32, BORDER_STRONG),
        );
        ui.painter().text(
            egui::pos2(friends.left() + 16.0, friends.top() + 18.0),
            egui::Align2::LEFT_CENTER,
            lang.modrinth_friends().to_uppercase(),
            egui::FontId::proportional(11.5),
            TEXT_HEADING.into(),
        );
        ui.painter().text(
            egui::pos2(friends.left() + 16.0, friends.top() + 48.0),
            egui::Align2::LEFT_CENTER,
            "OFFLINE",
            egui::FontId::proportional(9.0),
            TEXT_MUTED.into(),
        );
        let hint = if signed_in {
            lang.modrinth_no_friends_yet()
        } else {
            lang.modrinth_no_friends()
        };
        // Paint the hint at a fixed card position instead of nesting another
        // allocating UI: a child allocation rewound the parent cursor and let
        // the News card overlap the lower half of this card.
        let hint_galley = ui.painter().layout(
            hint.to_owned(),
            egui::FontId::proportional(11.5),
            TEXT_MUTED.into(),
            (friends.width() - 32.0).max(80.0),
        );
        ui.painter().galley(
            egui::pos2(friends.left() + 16.0, friends.top() + 66.0),
            hint_galley,
            TEXT_MUTED.into(),
        );
        ui.painter().text(
            egui::pos2(friends.left() + 16.0, friends.bottom() - 15.0),
            egui::Align2::LEFT_CENTER,
            if signed_in {
                lang.nav_profile()
            } else {
                lang.modrinth_sign_in()
            },
            egui::FontId::proportional(11.0),
            if signed_in {
                TEXT_MUTED.into()
            } else {
                crate::theme::ACCENT_LIGHT.into()
            },
        );
        icons::draw(
            ui.painter(),
            Icon::ChevronRight,
            egui::Rect::from_center_size(
                egui::pos2(friends.right() - 18.0, friends.bottom() - 15.0),
                vec2(13.0, 13.0),
            ),
            TEXT_MUTED,
            1.3,
        );
        if friends_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if friends_resp.clicked() {
            self.labymod_friends_request = true;
        }

        ui.add_space(12.0);
        let (news, _) = ui.allocate_exact_size(vec2(width, news_h), Sense::hover());
        ui.painter()
            .rect_filled(news, egui::Rounding::same(16.0), BG_CARD);
        ui.painter().rect_stroke(
            news,
            egui::Rounding::same(16.0),
            Stroke::new(1.0_f32, BORDER_STRONG),
        );
        ui.painter().text(
            egui::pos2(news.left() + 16.0, news.top() + 18.0),
            egui::Align2::LEFT_CENTER,
            lang.modrinth_news().to_uppercase(),
            egui::FontId::proportional(11.5),
            TEXT_HEADING.into(),
        );
        let show_all = ui.painter().layout_no_wrap(
            lang.shell_more_news().to_owned(),
            egui::FontId::proportional(9.5),
            Color32::PLACEHOLDER,
        );
        let show_all_rect = egui::Rect::from_center_size(
            egui::pos2(
                news.right() - 16.0 - show_all.size().x / 2.0,
                news.top() + 18.0,
            ),
            show_all.size() + vec2(10.0, 12.0),
        );
        let show_all_resp = ui.interact(
            show_all_rect,
            ui.id().with("labymod_news_all"),
            Sense::click(),
        );
        let show_all_hover = ui
            .ctx()
            .animate_bool_responsive(show_all_resp.id, show_all_resp.hovered());
        ui.painter().galley(
            egui::pos2(
                news.right() - 16.0 - show_all.size().x,
                news.top() + 18.0 - show_all.size().y / 2.0,
            ),
            show_all,
            crate::theme::lerp_color(crate::theme::ACCENT_LIGHT, Color32::WHITE, show_all_hover),
        );
        if show_all_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if show_all_resp.clicked() {
            let _ = open::that(PROJECT_URL);
        }

        let news_click = egui::Rect::from_min_max(
            egui::pos2(news.left() + 10.0, news.top() + 34.0),
            egui::pos2(news.right() - 10.0, news.bottom() - 6.0),
        );
        let news_resp = ui.interact(
            news_click,
            ui.id().with("labymod_news_open"),
            Sense::click(),
        );
        if news_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if news_resp.clicked() {
            let _ = open::that(PROJECT_URL);
        }
        let thumb = egui::Rect::from_min_max(
            egui::pos2(news.left() + 14.0, news.top() + 38.0),
            egui::pos2(news.right() - 14.0, news.bottom() - 52.0),
        );
        ui.painter().rect_filled(
            thumb,
            egui::Rounding::same(10.0),
            crate::theme::with_alpha(crate::theme::BG, 180),
        );
        if let Some(tex) = self.hero_texture.as_ref() {
            rounded_image(
                ui.painter(),
                thumb,
                10.0,
                tex,
                Color32::from_rgb(105, 110, 122),
            );
        }
        ui.painter().rect_filled(
            thumb,
            egui::Rounding::same(10.0),
            Color32::from_rgba_unmultiplied(7, 9, 14, 120),
        );
        ui.painter().text(
            egui::pos2(news.left() + 16.0, news.bottom() - 34.0),
            egui::Align2::LEFT_CENTER,
            lang.modrinth_news_updates(),
            egui::FontId::proportional(11.5),
            TEXT_PRIMARY.into(),
        );
        ui.painter().text(
            egui::pos2(news.left() + 16.0, news.bottom() - 19.0),
            egui::Align2::LEFT_CENTER,
            lang.modrinth_news_today(),
            egui::FontId::proportional(9.5),
            TEXT_MUTED.into(),
        );
    }

    fn show_hero(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        instances_count: usize,
        memory_mb: u32,
        lang: amc_core::Language,
    ) {
        let width = ui.available_width();
        let hero_h = 300.0;
        let (hero, _) = ui.allocate_exact_size(vec2(width, hero_h), Sense::hover());
        let painter = ui.painter().clone();
        match &self.hero_texture {
            Some(tex) => image_cover(&painter, hero, tex, Color32::WHITE),
            None => {
                painter.rect_filled(hero, crate::theme::rounding(), BG_CARD);
            }
        }
        painter.rect_filled(
            hero,
            crate::theme::rounding(),
            Color32::from_rgba_unmultiplied(8, 6, 6, 208),
        );

        let deco_side = 360.0_f32.min(hero_h - 60.0);
        let deco = egui::Rect::from_center_size(
            egui::pos2(hero.right() - 90.0, hero.center().y),
            vec2(deco_side, deco_side),
        );
        red_square_deco(&painter, deco);

        let text_rect = egui::Rect::from_min_max(
            egui::pos2(hero.left() + 40.0, hero.top() + 32.0),
            egui::pos2(hero.right() - 40.0, hero.bottom() - 24.0),
        );
        let mut play_clicked = false;
        let mut pick_clicked = false;
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if red_button(ui, lang.bottom_btn_play(), 300.0, 50.0).clicked() {
                        play_clicked = true;
                    }
                    ui.add_space(24.0);
                    if ghost_link(ui, lang.home_hero_pick()).clicked() {
                        pick_clicked = true;
                    }
                });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(lang.home_hero_hint())
                        .font(crate::theme::serif_it(17.0))
                        .color(TEXT_MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("LAUNCHER")
                        .font(crate::theme::display(54.0))
                        .color(RUBY_LIGHT),
                );
                ui.label(
                    egui::RichText::new("AMC")
                        .font(crate::theme::display(54.0))
                        .color(Color32::from_rgb(0xEA, 0xE2, 0xDC)),
                );
                ui.add_space(10.0);
                kicker(ui, lang.home_hero_kicker());
            });
        });
        if play_clicked {
            self.hero_play_request = true;
        }
        if pick_clicked {
            self.open_drawer(
                selected_version
                    .clone()
                    .or_else(|| versions.first().map(|v| v.id.clone())),
            );
        }

        ui.add_space(26.0);

        let stats = [
            (
                "01",
                format!("{:02}", versions.len()),
                lang.home_stat_versions(),
            ),
            (
                "02",
                format!("{:02}", instances_count),
                lang.home_stat_instances(),
            ),
            ("03", format!("{memory_mb} MB"), lang.home_stat_ram()),
        ];
        ui.horizontal(|ui| {
            let w = ((width - 40.0) / 3.0).max(180.0);
            for (num, big, small) in stats {
                ui.vertical(|ui| {
                    ui.set_width(w);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(ui.min_rect().min, vec2(w, 1.0)),
                        crate::theme::rounding(),
                        BORDER_STRONG,
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(num)
                            .font(crate::theme::mono(11.0))
                            .color(ACCENT_DIM),
                    );
                    ui.label(
                        egui::RichText::new(big)
                            .font(crate::theme::display(22.0))
                            .color(TEXT_HEADING),
                    );
                    ui.label(
                        egui::RichText::new(small)
                            .font(crate::theme::mono(10.0))
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(20.0);
            }
        });

        ui.add_space(10.0);
    }

    /// Full-style details drawer for the selected version: dim backdrop,
    /// slide-in panel, click-outside to close, Play launches directly.
    fn show_drawer(
        &mut self,
        ui: &mut Ui,
        versions: &[GameVersion],
        selected_version: &mut Option<String>,
        lang: amc_core::Language,
    ) {
        let ctx = ui.ctx().clone();
        let anim = ctx.animate_bool_with_time(egui::Id::new("home_drawer"), self.drawer_open, 0.18);
        if !self.drawer_open && anim <= 0.001 {
            return;
        }
        let screen = ctx.screen_rect();
        let slide = (1.0 - anim) * 60.0;
        let panel = egui::Rect::from_min_max(
            egui::pos2(screen.right() - 400.0 - 14.0 + slide, screen.top() + 14.0),
            egui::pos2(screen.right() - 14.0 + slide, screen.bottom() - 14.0),
        );

        let mut close = false;
        let mut play_id: Option<String> = None;

        egui::Area::new(egui::Id::new("home_drawer_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(&ctx, |ui| {
                ui.set_min_size(screen.size());
                let painter = ui.painter().clone();
                let dim = egui::Rect::from_min_size(screen.min, screen.size());
                painter.rect_filled(
                    dim,
                    crate::theme::rounding(),
                    Color32::from_rgba_unmultiplied(0, 0, 0, (120.0 * anim) as u8),
                );
                let dim_resp = ui.interact(dim, ui.id().with("drawer-dim"), Sense::click());
                if dim_resp.clicked() {
                    if let Some(pos) = dim_resp.interact_pointer_pos() {
                        if !panel.contains(pos) {
                            close = true;
                        }
                    }
                }

                painter.rect_filled(panel, crate::theme::rounding(), BG_CARD);
                painter.rect_stroke(
                    panel,
                    crate::theme::rounding(),
                    Stroke::new(1.0_f32, BORDER_DEFAULT),
                );

                let inner = panel.shrink(18.0);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(lang.home_title())
                                .font(crate::theme::display(20.0))
                                .color(TEXT_HEADING),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if tag(ui, lang.home_drawer_close(), false).clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(12.0);

                    match versions
                        .iter()
                        .find(|v| Some(&v.id) == self.drawer_version.as_ref())
                    {
                        Some(v) => {
                            ui.label(
                                egui::RichText::new(&v.id)
                                    .font(crate::theme::display(20.0))
                                    .color(TEXT_HEADING),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} • {}",
                                    v.release_type.as_str(),
                                    v.release_time.format("%d.%m.%Y")
                                ))
                                .font(crate::theme::mono(11.0))
                                .color(TEXT_MUTED),
                            );
                            ui.add_space(10.0);
                            let installed = self.installed_cache.contains(&v.id);
                            let _ = tag(
                                ui,
                                if installed {
                                    lang.home_drawer_installed()
                                } else {
                                    lang.home_drawer_not_installed()
                                },
                                installed,
                            );
                            ui.add_space(16.0);
                            if red_button(ui, lang.inst_btn_play(), 260.0, 44.0).clicked() {
                                *selected_version = Some(v.id.clone());
                                play_id = Some(v.id.clone());
                                close = true;
                            }
                        }
                        None => {
                            ui.label(
                                egui::RichText::new(lang.home_empty())
                                    .font(crate::theme::serif_it(15.0))
                                    .color(TEXT_MUTED),
                            );
                        }
                    }
                });
            });

        if close {
            self.close_drawer();
        }
        if let Some(id) = play_id {
            self.drawer_play_request = Some(id);
        }
    }

    fn filter_tab_btn(&mut self, ui: &mut Ui, label: &str, tab: FilterTab) {
        let is_active = self.active_filter == tab;
        let (rect, resp) = ui.allocate_exact_size(vec2(78.0, 28.0), Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let active_t = ui.ctx().animate_bool(resp.id.with("act"), is_active);

        let bg = lerp_color(lerp_color(BG_ELEVATED, BG_HOVER, hover_t), RUBY, active_t);
        let stroke_col = lerp_color(
            lerp_color(BORDER_DEFAULT, RUBY, hover_t),
            RUBY_LIGHT,
            active_t,
        );
        let text_col = lerp_color(
            lerp_color(TEXT_MUTED, TEXT_PRIMARY, hover_t),
            Color32::WHITE,
            active_t,
        );

        ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
        ui.painter().rect_stroke(
            rect,
            crate::theme::rounding(),
            Stroke::new(1.0_f32, stroke_col),
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_col,
        );

        if resp.clicked() {
            self.active_filter = tab;
        }
    }
}

/// Draws a texture clipped to a rounded rectangle using a triangle fan. The
/// regular `image_cover` helper paints a square quad, which leaves visible
/// corners on LabyMod's large rounded hero card.
fn rounded_image(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: f32,
    texture: &egui::TextureHandle,
    tint: Color32,
) {
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    if radius <= 0.5 {
        image_cover(painter, rect, texture, tint);
        return;
    }
    let steps = 6;
    let corners = [
        (
            egui::pos2(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
            std::f32::consts::PI * 1.5,
        ),
        (
            egui::pos2(rect.right() - radius, rect.top() + radius),
            -std::f32::consts::FRAC_PI_2,
            0.0,
        ),
        (
            egui::pos2(rect.right() - radius, rect.bottom() - radius),
            0.0,
            std::f32::consts::FRAC_PI_2,
        ),
        (
            egui::pos2(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ),
    ];
    let mut points = Vec::with_capacity((steps + 1) * corners.len());
    for (center, start, end) in corners {
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let angle = start + (end - start) * t;
            points.push(egui::pos2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
    }

    let uv = |p: egui::Pos2| {
        egui::pos2(
            (p.x - rect.left()) / rect.width(),
            (p.y - rect.top()) / rect.height(),
        )
    };
    let mut mesh = egui::Mesh {
        texture_id: texture.id(),
        ..Default::default()
    };
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.center(),
        uv: egui::pos2(0.5, 0.5),
        color: tint,
    });
    for p in &points {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *p,
            uv: uv(*p),
            color: tint,
        });
    }
    let n = points.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + ((i + 1) % n));
    }
    painter.add(egui::Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_open_close() {
        let mut page = HomePage::default();
        assert!(!page.drawer_open);
        page.open_drawer(Some("1.20.1".to_string()));
        assert!(page.drawer_open);
        assert_eq!(page.drawer_version.as_deref(), Some("1.20.1"));
        page.close_drawer();
        assert!(!page.drawer_open);
        assert_eq!(page.drawer_version, None);
    }

    #[test]
    fn test_drawer_none_stays_closed() {
        let mut page = HomePage::default();
        page.open_drawer(None);
        assert!(!page.drawer_open);
    }

    #[test]
    fn test_fav_display_name() {
        assert_eq!(fav_display_name("Survival", "play.ex.com"), "Survival");
        assert_eq!(fav_display_name("  ", "play.ex.com"), "play.ex.com");
        assert_eq!(fav_display_name("", "play.ex.com"), "play.ex.com");
    }
}
