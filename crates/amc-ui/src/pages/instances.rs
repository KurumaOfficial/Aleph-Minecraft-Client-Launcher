use crate::pages::instance_manage::InstanceManage;
use crate::theme::{
    lerp_color, BG_CARD, BG_HOVER, BORDER_DEFAULT, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING,
    TEXT_MUTED, TEXT_PRIMARY, WARNING,
};
use crate::widgets::choice_chip;
use crate::widgets::full::tag;
use crate::widgets::icons::{self, icon_button, Icon};
use amc_auth::Account;
use amc_core::types::{Instance, InstanceIcon, InstanceTemplate, LoaderType};
use amc_core::Language;
use amc_mods::ModSearchResult;
use egui::{vec2, Color32, ScrollArea, Sense, Stroke, TextEdit, Ui};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum InstanceAction {
    None,
    Created(Instance),
    Updated(Instance),
    /// Soft-delete: the app moves the game dir to trash.
    Trashed(Uuid),
    Restored(Uuid),
    TrashPurged(Uuid),
    TrashEmptied,
    Selected(Uuid),
    /// Duplicate with an optional worlds copy (CONCEPT P2).
    Cloned(Uuid, bool),
    Launch(Uuid),
}

/// Ambient data for the instances page, bundled so `show` stays under
/// the argument-count lint.
pub struct InstancesContext<'a> {
    pub instances_dir: &'a Path,
    pub trash_dir: &'a Path,
    pub backups_root: &'a Path,
    pub accounts: &'a [Account],
    /// Defined + used group names for the edit-modal hint (CONCEPT P2).
    pub group_names: Vec<String>,
    /// Hardware-picked default RAM for the create dialog (CONCEPT P2).
    pub suggested_ram_mb: u32,
}

pub struct InstancesPage {
    pub search_query: String,
    pub show_create_modal: bool,
    pub new_instance_name: String,
    pub new_instance_version: String,
    pub new_instance_loader: LoaderType,
    pub new_instance_ram: u32,

    // Edit modal
    pub show_edit_modal: bool,
    pub edit_instance_id: Option<Uuid>,
    pub edit_instance_name: String,
    pub edit_instance_version: String,
    pub edit_instance_loader: LoaderType,
    pub edit_loader_pin: Option<String>,
    pub loader_pin_reset: Option<Uuid>,
    pub edit_instance_ram: u32,
    pub edit_instance_icon: Option<String>,
    pub edit_instance_tags: String,
    pub edit_instance_pinned: bool,
    pub edit_instance_account: Option<Uuid>,
    pub edit_instance_group: String,
    pub edit_use_custom_size: bool,
    pub edit_window_w: u32,
    pub edit_window_h: u32,
    /// None = launcher default, Some(false) = windowed, Some(true) = fullscreen.
    pub edit_fullscreen: Option<bool>,
    pub edit_instance_notes: String,
    pub edit_instance_pre: String,
    pub edit_instance_post: String,

    // Background disk-usage scan (CONCEPT "Место на диске").
    disk_sizes: HashMap<Uuid, u64>,
    disk_scanned_for: Vec<Uuid>,
    disk_rx: Option<std::sync::mpsc::Receiver<Vec<(Uuid, u64, bool)>>>,
    local_badges: HashMap<Uuid, bool>,

    // Organization (CONCEPT, P2): tag filter, sort, icon texture cache.
    pub active_tag: Option<String>,
    pub sort_mode: InstanceSort,
    icon_textures: HashMap<Uuid, egui::TextureHandle>,
    icon_failed: std::collections::HashSet<Uuid>,
    /// Memoized filter+sort (per-frame `to_lowercase`/`sort` avoided):
    /// recomputed only when query/tag/sort or instance revision changes.
    filter_cache: Vec<usize>,
    filter_key: Option<(String, Option<String>, InstanceSort, u64)>,
    tags_cache: Vec<String>,
    tags_rev: u64,
    trash_cache: Vec<amc_core::TrashedInstance>,
    trash_at: Option<std::time::Instant>,

    // Per-instance management (worlds / configs / health modal).
    manage: InstanceManage,

    // Discover trending modpacks (CONCEPT P4): list + one-click .mrpack
    // import into the selected instance. Fetched once per session.
    pub discover_open: bool,
    pub discover: Vec<ModSearchResult>,
    pub discover_loading: bool,
    pub discover_requested: bool,
    pub discover_install: Option<String>,
    /// Version port notice for the app shell: (instance id, old, new).
    /// The app toasts it and re-runs the mod update check (CONCEPT P4).
    pub port_notice: Option<(Uuid, String, String)>,
    // Clone options modal (CONCEPT P2 worlds choice).
    pub show_clone_modal: bool,
    pub clone_instance_id: Option<Uuid>,
    pub clone_with_saves: bool,
    /// Desktop shortcut request for the app shell (CONCEPT P11/P14).
    pub shortcut_request: Option<Uuid>,
}

impl Default for InstancesPage {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            show_create_modal: false,
            new_instance_name: "My Instance".to_string(),
            new_instance_version: "1.20.1".to_string(),
            new_instance_loader: LoaderType::Fabric,
            new_instance_ram: 4096,

            show_edit_modal: false,
            edit_instance_id: None,
            edit_instance_name: String::new(),
            edit_instance_version: String::new(),
            edit_instance_loader: LoaderType::Vanilla,
            edit_loader_pin: None,
            loader_pin_reset: None,
            edit_instance_ram: 4096,
            edit_instance_icon: None,
            edit_instance_tags: String::new(),
            edit_instance_pinned: false,
            edit_instance_account: None,
            edit_instance_group: String::new(),
            edit_use_custom_size: false,
            edit_window_w: 1280,
            edit_window_h: 720,
            edit_fullscreen: None,
            edit_instance_notes: String::new(),
            edit_instance_pre: String::new(),
            edit_instance_post: String::new(),

            disk_sizes: HashMap::new(),
            disk_scanned_for: Vec::new(),
            disk_rx: None,
            local_badges: HashMap::new(),
            active_tag: None,
            sort_mode: InstanceSort::default(),
            icon_textures: HashMap::new(),
            icon_failed: std::collections::HashSet::new(),
            filter_cache: Vec::new(),
            filter_key: None,
            tags_cache: Vec::new(),
            tags_rev: 0,
            trash_cache: Vec::new(),
            trash_at: None,
            manage: InstanceManage::default(),
            discover_open: true,
            discover: Vec::new(),
            discover_loading: false,
            discover_requested: false,
            discover_install: None,
            port_notice: None,
            show_clone_modal: false,
            clone_instance_id: None,
            clone_with_saves: true,
            shortcut_request: None,
        }
    }
}

/// True when `mods/` holds jars the launcher did not install itself
/// (CONCEPT local-mod badge, P2). `.disabled` files count as mods.
/// Pure over a filename list for tests; the IO wrapper is below.
fn has_local_mods_in(list: &[String], installed: &[String]) -> bool {
    list.iter().any(|name| {
        let base = name.strip_suffix(".disabled").unwrap_or(name);
        base.ends_with(".jar") && !installed.iter().any(|n| n == name || n == base)
    })
}

fn has_local_mods(game_dir: &Path, installed: &[String]) -> bool {
    let mods = game_dir.join("mods");
    let entries = match std::fs::read_dir(&mods) {
        Ok(entries) => entries,
        Err(_) => return false,
    };
    let names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    has_local_mods_in(&names, installed)
}

/// Shorten a card title to one line (pure, unit-tested).
fn truncate_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        name.to_string()
    } else {
        format!("{}…", name.chars().take(max_chars).collect::<String>())
    }
}

/// Instance list ordering (CONCEPT: folders/tags/sort, P2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstanceSort {
    /// Recently played first, never-played last.
    #[default]
    Recent,
    Name,
    Created,
    Playtime,
}

impl InstanceSort {
    pub const ALL: [Self; 4] = [Self::Recent, Self::Name, Self::Created, Self::Playtime];

    pub fn label(&self, lang: Language) -> &'static str {
        match self {
            Self::Recent => lang.inst_sort_recent(),
            Self::Name => lang.inst_sort_name(),
            Self::Created => lang.inst_sort_created(),
            Self::Playtime => lang.inst_sort_played(),
        }
    }
}

/// Search + tag predicate for the instance list (pure, unit-tested).
fn matches_filter(inst: &Instance, query: &str, active_tag: Option<&str>) -> bool {
    if let Some(tag) = active_tag {
        if !inst.tags.iter().any(|t| t == tag) {
            return false;
        }
    }
    if query.is_empty() {
        return true;
    }
    inst.name.to_lowercase().contains(query)
        || inst.game_version.to_lowercase().contains(query)
        || inst.loader.as_str().to_lowercase().contains(query)
}

/// Pinned instances float above the rest, then the sort mode applies.
/// `None` last-played counts as ancient. Pure, unit-tested.
fn compare_instances(a: &Instance, b: &Instance, mode: InstanceSort) -> std::cmp::Ordering {
    match (a.pinned, b.pinned) {
        (true, false) => return std::cmp::Ordering::Less,
        (false, true) => return std::cmp::Ordering::Greater,
        _ => {}
    }
    match mode {
        InstanceSort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        InstanceSort::Recent => b.last_played.cmp(&a.last_played),
        InstanceSort::Created => b.created_at.cmp(&a.created_at),
        InstanceSort::Playtime => b.total_played_minutes.cmp(&a.total_played_minutes),
    }
}

/// Cheap revision of the instance list for cache invalidation (no alloc).
/// Any rename/version/loader/pin/play/tag change flips the revision.
fn instances_rev(instances: &[Instance]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    instances.len().hash(&mut hasher);
    for inst in instances {
        inst.id.hash(&mut hasher);
        inst.pinned.hash(&mut hasher);
        inst.name.hash(&mut hasher);
        inst.game_version.hash(&mut hasher);
        inst.loader.as_str().hash(&mut hasher);
        inst.total_played_minutes.hash(&mut hasher);
        inst.tags.len().hash(&mut hasher);
        for tag in &inst.tags {
            tag.hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Split a tags field (`"a, b,,c"`) into clean values, order kept.
fn parse_tags(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let tag = part.trim().to_string();
        if !tag.is_empty() && !out.contains(&tag) {
            out.push(tag);
        }
    }
    out
}

impl InstancesPage {
    /// Cached custom `icon.png` texture for a card. Missing or broken files
    /// are remembered as failed so later frames don't re-hit the disk; the
    /// negative cache is cleared whenever the edit modal saves.
    fn instance_icon_texture(
        &mut self,
        ctx: &egui::Context,
        inst_id: Uuid,
        game_dir: &Path,
    ) -> Option<egui::TextureHandle> {
        if let Some(tex) = self.icon_textures.get(&inst_id) {
            return Some(tex.clone());
        }
        if self.icon_failed.contains(&inst_id) {
            return None;
        }
        let bytes = std::fs::read(game_dir.join(InstanceIcon::CUSTOM_ICON_FILE)).ok()?;
        if bytes.is_empty() {
            self.icon_failed.insert(inst_id);
            return None;
        }
        let img = image::load_from_memory(&bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        if w == 0 || h == 0 || w > 512 || h > 512 {
            self.icon_failed.insert(inst_id);
            return None;
        }
        let pixels: Vec<Color32> = rgba
            .pixels()
            .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let tex = ctx.load_texture(
            format!("inst-icon-{inst_id}"),
            egui::ColorImage {
                size: [w, h],
                pixels,
            },
            egui::TextureOptions::LINEAR,
        );
        self.icon_textures.insert(inst_id, tex.clone());
        Some(tex)
    }

    /// Poll a finished background scan, or (re)start one when the instance
    /// set changed. Measuring runs on a worker thread — the UI never blocks.
    /// Besides sizes it flags instances holding local (non-launcher) mods.
    fn poll_disk_sizes(&mut self, instances: &[Instance], instances_dir: &Path) {
        if let Some(rx) = &self.disk_rx {
            match rx.try_recv() {
                Ok(entries) => {
                    let mut sizes = HashMap::new();
                    let mut badges = HashMap::new();
                    for (id, bytes, has_local) in entries {
                        sizes.insert(id, bytes);
                        badges.insert(id, has_local);
                    }
                    self.disk_sizes = sizes;
                    self.local_badges = badges;
                    self.disk_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.disk_rx = None;
                }
            }
            return;
        }
        let ids: Vec<Uuid> = instances.iter().map(|i| i.id).collect();
        if ids == self.disk_scanned_for {
            return;
        }
        self.disk_scanned_for = ids;
        let jobs: Vec<(Uuid, PathBuf, Vec<String>)> = instances
            .iter()
            .map(|i| {
                (
                    i.id,
                    i.get_game_dir(instances_dir),
                    i.installed_by_launcher.clone(),
                )
            })
            .collect();
        let (tx, rx) = std::sync::mpsc::channel();
        self.disk_rx = Some(rx);
        std::thread::spawn(move || {
            let out: Vec<(Uuid, u64, bool)> = jobs
                .into_iter()
                .map(|(id, dir, installed)| {
                    let bytes = Instance::disk_usage_dir(&dir);
                    let local = has_local_mods(&dir, &installed);
                    (id, bytes, local)
                })
                .collect();
            let _ = tx.send(out);
        });
    }

    /// Force disk + badge rescan (call after mods change on disk).
    pub fn refresh_scans(&mut self) {
        self.disk_scanned_for.clear();
        self.filter_key = None;
        self.tags_rev = 0;
        self.trash_at = None;
    }

    /// Fill the create dialog from a quick-start template (ROADMAP P2).
    /// Loader and RAM always apply; the name applies only while untouched.
    fn apply_template(&mut self, template: InstanceTemplate, lang: Language) {
        self.new_instance_loader = template.loader;
        self.new_instance_ram = template.ram_mb;
        if self.new_instance_name.trim().is_empty() || self.new_instance_name == "My Instance" {
            self.new_instance_name = template.default_name(lang);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        instances: &mut [Instance],
        selected_instance: &mut Option<Uuid>,
        ctx: &InstancesContext,
        lang: Language,
    ) -> InstanceAction {
        let mut action = InstanceAction::None;
        let instances_dir = ctx.instances_dir;
        let accounts = ctx.accounts;
        self.poll_disk_sizes(instances, instances_dir);
        ui.add_space(20.0);

        // Header
        ui.label(
            egui::RichText::new(lang.inst_title())
                .font(egui::FontId::proportional(26.0))
                .strong()
                .color(TEXT_HEADING),
        );

        ui.add_space(14.0);

        // Control Row
        ui.horizontal(|ui| {
            let search_width = (ui.available_width() - 180.0).max(200.0);
            ui.add(
                TextEdit::singleline(&mut self.search_query)
                    .hint_text(egui::RichText::new(lang.inst_search_hint()).color(TEXT_MUTED))
                    .desired_width(search_width)
                    .font(egui::FontId::proportional(14.0)),
            );

            ui.add_space(10.0);

            let (cr_rect, cr_resp) = ui.allocate_exact_size(vec2(160.0, 36.0), Sense::click());
            let cr_hover = ui
                .ctx()
                .animate_bool_responsive(cr_resp.id, cr_resp.hovered());
            let cr_bg = lerp_color(RUBY, RUBY_LIGHT, cr_hover);
            let cr_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, cr_hover);

            ui.painter()
                .rect_filled(cr_rect, crate::theme::rounding(), cr_bg);
            ui.painter().rect_stroke(
                cr_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, cr_stroke),
            );
            ui.painter().text(
                cr_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.inst_btn_create(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            if cr_resp.clicked() {
                self.new_instance_ram = ctx.suggested_ram_mb;
                self.show_create_modal = true;
            }
        });

        ui.add_space(14.0);

        // Tag filter chips + sort selector (memoized: O(T) hash check,
        // rebuild only when the instance set changes).
        let rev = instances_rev(instances);
        if rev != self.tags_rev {
            let mut seen = std::collections::HashSet::new();
            let mut tags: Vec<String> = Vec::new();
            for inst in instances.iter() {
                for t in &inst.tags {
                    if seen.insert(t.clone()) {
                        tags.push(t.clone());
                    }
                }
            }
            tags.sort();
            self.tags_cache = tags;
            self.tags_rev = rev;
        }
        if let Some(active) = self.active_tag.clone() {
            if !self.tags_cache.contains(&active) {
                self.active_tag = None;
            }
        }
        let mut tag_pick: Option<Option<String>> = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            if tag(ui, lang.inst_tag_all(), self.active_tag.is_none()).clicked() {
                tag_pick = Some(None);
            }
            for t in &self.tags_cache {
                let is_active = self.active_tag.as_deref() == Some(t.as_str());
                if tag(ui, t, is_active).clicked() {
                    tag_pick = Some(if is_active { None } else { Some(t.clone()) });
                }
            }
        });
        if let Some(pick) = tag_pick {
            self.active_tag = pick;
        }

        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            ui.label(
                egui::RichText::new(lang.inst_sort_title())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
            for mode in InstanceSort::ALL {
                if tag(ui, mode.label(lang), self.sort_mode == mode).clicked() {
                    self.sort_mode = mode;
                }
            }
        });

        ui.add_space(8.0);

        // Discover trending modpacks (CONCEPT P4): inside the instances
        // tab, never on the home screen.
        ui.horizontal(|ui| {
            let caret = if self.discover_open {
                Icon::CaretDown
            } else {
                Icon::CaretRight
            };
            let fill = ui.visuals().widgets.inactive.weak_bg_fill;
            if icons::icon_text_button(
                ui,
                Icon::Flame,
                Some(caret),
                lang.discover_title(),
                TEXT_HEADING,
                egui::FontId::proportional(14.0),
                fill,
            )
            .clicked()
            {
                self.discover_open = !self.discover_open;
            }
            if self.discover_open
                && !self.discover_loading
                && ui.button(lang.discover_refresh()).clicked()
            {
                self.discover_requested = true;
                self.discover_loading = true;
            }
        });
        if self.discover_open {
            if self.discover.is_empty() && !self.discover_loading {
                // First paint fetches once; refresh reuses the button above.
                self.discover_requested = true;
                self.discover_loading = true;
            }
            if self.discover_loading && self.discover.is_empty() {
                ui.horizontal(|ui| {
                    ui.spinner();
                });
            }
            let mut install: Option<String> = None;
            for pack in &self.discover {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(truncate_name(&pack.title, 34))
                            .font(egui::FontId::proportional(12.0))
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    icons::icon_label(
                        ui,
                        Icon::Download,
                        &pack.downloads.to_string(),
                        TEXT_MUTED,
                        egui::FontId::proportional(11.0),
                    );
                    if pack.rating > 0 {
                        icons::icon_label(
                            ui,
                            Icon::Star,
                            &pack.rating.to_string(),
                            TEXT_MUTED,
                            egui::FontId::proportional(11.0),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(lang.discover_install()).clicked() {
                            install = Some(pack.id.clone());
                        }
                    });
                });
            }
            if let Some(slug) = install {
                self.discover_install = Some(slug);
            }
            ui.add_space(8.0);
        }

        // Filter + sort instances (memoized: per-frame `to_lowercase`/`sort`
        // avoided, recompute only when query/tag/sort/revision changes).
        let cache_valid = match &self.filter_key {
            Some((q, t, s, r)) => {
                *q == self.search_query
                    && *t == self.active_tag
                    && *s == self.sort_mode
                    && *r == rev
            }
            None => false,
        };
        if !cache_valid {
            let query = self.search_query.trim().to_lowercase();
            let mut indices: Vec<usize> = instances
                .iter()
                .enumerate()
                .filter(|(_, inst)| matches_filter(inst, &query, self.active_tag.as_deref()))
                .map(|(idx, _)| idx)
                .collect();
            let sort_mode = self.sort_mode;
            indices.sort_by(|&a, &b| compare_instances(&instances[a], &instances[b], sort_mode));
            self.filter_cache = indices;
            self.filter_key = Some((
                self.search_query.clone(),
                self.active_tag.clone(),
                self.sort_mode,
                rev,
            ));
        }
        // Small index vec clone per frame (tens of ids max); the expensive
        // filter+sort+lowercase above runs only on miss.
        let filtered_indices = self.filter_cache.clone();

        // Instances list
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

                if instances.is_empty() {
                    ui.add_space(50.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(lang.inst_empty())
                                .font(egui::FontId::proportional(16.0))
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(10.0);
                        if ui
                            .button(egui::RichText::new(lang.inst_btn_create()).color(RUBY_LIGHT))
                            .clicked()
                        {
                            self.new_instance_ram = ctx.suggested_ram_mb;
                            self.show_create_modal = true;
                        }
                    });
                    return;
                }

                if filtered_indices.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(lang.home_empty())
                                .font(egui::FontId::proportional(14.0))
                                .color(TEXT_MUTED),
                        );
                    });
                    return;
                }

                // Full-style card grid: fixed 270px cards, auto-fill rows.
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                    for &idx in &filtered_indices {
                        let inst = &instances[idx];
                        instance_card(
                            self,
                            ui,
                            inst,
                            selected_instance,
                            instances_dir,
                            lang,
                            &mut action,
                        );
                    }
                });
            });

        /// Full-style instance card: loader accent strip, icon, name, meta,
        /// quick actions and a Play button. Click selects the instance.
        fn instance_card(
            page: &mut InstancesPage,
            ui: &mut Ui,
            inst: &Instance,
            selected_instance: &mut Option<Uuid>,
            instances_dir: &Path,
            lang: Language,
            action: &mut InstanceAction,
        ) {
            const CARD_W: f32 = 270.0;
            const CARD_H: f32 = 216.0;
            let inst_id = inst.id;
            let is_selected = selected_instance.as_ref() == Some(&inst_id);

            let (rect, resp) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());

            let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
            let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

            let base_bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
            let bg = lerp_color(base_bg, RUBY_DIM, sel_t);

            let stroke_col =
                lerp_color(lerp_color(BORDER_DEFAULT, RUBY, hover_t), RUBY_LIGHT, sel_t);
            let border_width = if is_selected { 1.5_f32 } else { 1.0_f32 };

            ui.painter().rect_filled(rect, crate::theme::rounding(), bg);

            // Loader accent strip on top (Square motif from aleph.icu)
            let loader_color = match inst.loader {
                LoaderType::Vanilla => Color32::from_rgb(0x38, 0x8E, 0x3C),
                LoaderType::Fabric => RUBY_LIGHT.into(),
                LoaderType::Quilt => Color32::from_rgb(0x9C, 0x27, 0xB0),
                LoaderType::Forge => Color32::from_rgb(0xE6, 0x51, 0x00),
                LoaderType::NeoForge => Color32::from_rgb(0xFF, 0x6D, 0x00),
                LoaderType::OptiFine => Color32::from_rgb(0x00, 0x83, 0x8F),
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.left_top(),
                    egui::pos2(rect.right(), rect.top() + 3.0),
                ),
                crate::theme::rounding(),
                loader_color,
            );
            ui.painter().rect_stroke(
                rect,
                crate::theme::rounding(),
                Stroke::new(border_width, stroke_col),
            );

            let mut card = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect.shrink(12.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            // Header: loader icon + name + version.
            card.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
                match page.instance_icon_texture(
                    ui.ctx(),
                    inst_id,
                    &inst.get_game_dir(instances_dir),
                ) {
                    Some(tex) => {
                        ui.painter().image(
                            tex.id(),
                            icon_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        ui.painter().rect_stroke(
                            icon_rect,
                            crate::theme::rounding(),
                            Stroke::new(1.0_f32, BORDER_DEFAULT),
                        );
                    }
                    None => {
                        let badge = match inst.icon.as_deref().and_then(icons::instance_preset) {
                            Some(preset) => preset,
                            None => icons::loader_badge(inst.loader),
                        };
                        ui.painter()
                            .rect_filled(icon_rect, crate::theme::rounding(), RUBY_DIM);
                        icons::draw_centered(
                            ui.painter(),
                            badge,
                            icon_rect.center(),
                            20.0,
                            Color32::WHITE,
                        );
                    }
                }
                ui.add_space(8.0);
                ui.vertical(|inner| {
                    inner.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(truncate_name(&inst.name, 22))
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(TEXT_HEADING),
                        );
                        if inst.pinned {
                            let (pin_rect, _) =
                                ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());
                            icons::draw_centered(
                                ui.painter(),
                                Icon::Pin,
                                pin_rect.center(),
                                12.0,
                                RUBY_LIGHT,
                            );
                        }
                    });
                    inner.label(
                        egui::RichText::new(format!("MC {}", inst.game_version))
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                    inner.label(
                        egui::RichText::new(inst.loader.as_str())
                            .font(egui::FontId::proportional(12.0))
                            .color(RUBY_LIGHT),
                    );
                });
            });

            card.add_space(6.0);

            // Stats line: playtime, disk, RAM, sessions.
            let play_time_str = if inst.total_played_minutes == 0 {
                lang.inst_never_played().to_string()
            } else if inst.total_played_minutes < 60 {
                format!("{} {}", inst.total_played_minutes, lang.inst_mins_suffix())
            } else {
                format!(
                    "{} {} {} {}",
                    inst.total_played_minutes / 60,
                    lang.inst_hours_suffix(),
                    inst.total_played_minutes % 60,
                    lang.inst_mins_suffix()
                )
            };
            let size_text = match page.disk_sizes.get(&inst_id) {
                Some(bytes) => lang.disk_size(*bytes),
                None => "…".to_string(),
            };
            card.horizontal(|ui| {
                icons::icon_label(
                    ui,
                    Icon::Clock,
                    &lang.inst_playtime_label(&play_time_str),
                    TEXT_MUTED,
                    egui::FontId::proportional(11.0),
                );
                icons::icon_label(
                    ui,
                    Icon::Disk,
                    &size_text,
                    TEXT_MUTED,
                    egui::FontId::proportional(11.0),
                );
                if let Some(ram) = inst.ram_mb {
                    ui.label(
                        egui::RichText::new(format!("{ram} MB"))
                            .font(egui::FontId::proportional(11.0))
                            .color(TEXT_MUTED),
                    );
                }
                if inst.sessions > 0 {
                    icons::icon_label(
                        ui,
                        Icon::Play,
                        &lang.inst_sessions_label(inst.sessions),
                        TEXT_MUTED,
                        egui::FontId::proportional(11.0),
                    );
                }
            });
            if page.local_badges.get(&inst_id).copied().unwrap_or(false) {
                card.label(
                    egui::RichText::new(lang.badge_local_mods())
                        .font(egui::FontId::proportional(10.0))
                        .strong()
                        .color(WARNING),
                );
            }
            if !inst.tags.is_empty() {
                icons::icon_label(
                    &mut card,
                    Icon::Tag,
                    &truncate_name(&inst.tags.join(", "), 40),
                    TEXT_MUTED,
                    egui::FontId::proportional(11.0),
                );
            }

            card.add_space(6.0);

            // Quick actions row.
            let inst_dir = inst.get_game_dir(instances_dir);
            card.horizontal(|ui| {
                if icon_button(ui, Icon::Folder, TEXT_PRIMARY)
                    .on_hover_text(lang.inst_btn_folder())
                    .clicked()
                {
                    let _ = std::fs::create_dir_all(&inst_dir);
                    let _ = open::that(&inst_dir);
                }
                if icon_button(ui, Icon::Gear, TEXT_PRIMARY)
                    .on_hover_text(lang.inst_btn_edit())
                    .clicked()
                {
                    page.edit_instance_id = Some(inst_id);
                    page.edit_instance_name = inst.name.clone();
                    page.edit_instance_version = inst.game_version.clone();
                    page.edit_instance_loader = inst.loader;
                    page.edit_loader_pin = inst.loader_version.clone();
                    page.edit_instance_ram = inst.ram_mb.unwrap_or(4096);
                    page.edit_instance_icon = inst.icon.clone();
                    page.edit_instance_tags = inst.tags.join(", ");
                    page.edit_instance_pinned = inst.pinned;
                    page.edit_instance_account = inst.default_account;
                    page.edit_instance_group = inst.group.clone().unwrap_or_default();
                    page.edit_use_custom_size = inst.window_size.is_some();
                    page.edit_window_w = inst.window_size.map(|(w, _)| w).unwrap_or(1280);
                    page.edit_window_h = inst.window_size.map(|(_, h)| h).unwrap_or(720);
                    page.edit_fullscreen = inst.fullscreen;
                    page.edit_instance_notes = inst.notes.clone();
                    page.edit_instance_pre = inst.pre_launch_cmd.clone();
                    page.edit_instance_post = inst.post_exit_cmd.clone();
                    page.show_edit_modal = true;
                }
                if icon_button(ui, Icon::Clipboard, TEXT_PRIMARY)
                    .on_hover_text(lang.inst_btn_clone())
                    .clicked()
                {
                    page.clone_instance_id = Some(inst_id);
                    page.clone_with_saves = true;
                    page.show_clone_modal = true;
                }
                if icon_button(ui, Icon::Link, TEXT_PRIMARY)
                    .on_hover_text(lang.inst_btn_shortcut())
                    .clicked()
                {
                    page.shortcut_request = Some(inst_id);
                }
                if icon_button(ui, Icon::Wrench, TEXT_PRIMARY)
                    .on_hover_text(lang.editor_title())
                    .clicked()
                {
                    page.manage.open_for(inst_id);
                }
                if icon_button(ui, Icon::Trash, TEXT_MUTED)
                    .on_hover_text(lang.inst_btn_delete())
                    .clicked()
                {
                    *action = InstanceAction::Trashed(inst_id);
                }
            });

            card.add_space(6.0);

            // Play button fills the remaining card height (at least 34px).
            let play_h = (rect.bottom() - 12.0 - card.min_rect().bottom()).max(34.0);
            let play_w = card.available_width();
            let (pb_rect, pb_resp) = card.allocate_exact_size(vec2(play_w, play_h), Sense::click());
            let pb_hover = card
                .ctx()
                .animate_bool_responsive(pb_resp.id, pb_resp.hovered());
            let pb_bg = lerp_color(RUBY, RUBY_LIGHT, pb_hover);
            let pb_stroke = lerp_color(RUBY_LIGHT, Color32::WHITE, pb_hover);

            card.painter()
                .rect_filled(pb_rect, crate::theme::rounding(), pb_bg);
            card.painter().rect_stroke(
                pb_rect,
                crate::theme::rounding(),
                Stroke::new(1.0_f32, pb_stroke),
            );
            card.painter().text(
                pb_rect.center(),
                egui::Align2::CENTER_CENTER,
                lang.inst_btn_play(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            if pb_resp.clicked() {
                *action = InstanceAction::Launch(inst_id);
            }

            if resp.clicked() {
                *selected_instance = Some(inst_id);
                *action = InstanceAction::Selected(inst_id);
            }
        }

        // Trash (CONCEPT "Корзина", P2): soft-deleted instances wait here
        // for restore or permanent deletion. FS scan throttled to 2s —
        // never per frame.
        let trash_stale = match self.trash_at {
            Some(at) => at.elapsed() > std::time::Duration::from_secs(2),
            None => true,
        };
        if trash_stale {
            self.trash_cache = amc_core::list_trash(ctx.trash_dir);
            self.trash_at = Some(std::time::Instant::now());
        }
        let trashed = self.trash_cache.clone();
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(lang.trash_title())
                    .font(egui::FontId::proportional(18.0))
                    .strong()
                    .color(TEXT_HEADING),
            );
            if !trashed.is_empty() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(lang.trash_empty_all()).clicked() {
                        action = InstanceAction::TrashEmptied;
                    }
                });
            }
        });
        ui.add_space(6.0);
        if trashed.is_empty() {
            ui.label(
                egui::RichText::new(lang.trash_empty())
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_MUTED),
            );
        }
        for entry in &trashed {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} • {} {}",
                        truncate_name(&entry.instance.name, 28),
                        entry.instance.game_version,
                        entry.instance.loader.as_str(),
                    ))
                    .font(egui::FontId::proportional(12.0))
                    .color(TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(lang.trash_delete_forever()).clicked() {
                        action = InstanceAction::TrashPurged(entry.instance.id);
                    }
                    if ui.button(lang.trash_restore()).clicked() {
                        action = InstanceAction::Restored(entry.instance.id);
                    }
                });
            });
        }

        // Create Instance Modal Dialog
        if self.show_create_modal {
            egui::Window::new(lang.inst_modal_create_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(lang.tmpl_title())
                            .strong()
                            .color(TEXT_HEADING),
                    );
                    ui.horizontal(|ui| {
                        for tmpl in InstanceTemplate::ALL {
                            let active = InstanceTemplate::matching(
                                self.new_instance_loader,
                                self.new_instance_ram,
                            )
                            .map(|t| t.id)
                                == Some(tmpl.id);
                            if choice_chip(ui, lang.tmpl_name(tmpl.id), 130.0, 32.0, 12.0, active) {
                                self.apply_template(tmpl, lang);
                            }
                        }
                    });
                    let tmpl_desc =
                        InstanceTemplate::matching(self.new_instance_loader, self.new_instance_ram)
                            .map(|t| lang.tmpl_desc(t.id))
                            .unwrap_or_else(|| lang.tmpl_custom_note());
                    ui.label(egui::RichText::new(tmpl_desc).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.new_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.new_instance_version).desired_width(400.0),
                    );

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("loader_select")
                        .selected_text(self.new_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Vanilla,
                                "Vanilla",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Fabric,
                                "Fabric",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Quilt,
                                "Quilt",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::Forge,
                                "Forge",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::NeoForge,
                                "NeoForge",
                            );
                            ui.selectable_value(
                                &mut self.new_instance_loader,
                                LoaderType::OptiFine,
                                "OptiFine",
                            );
                        });
                    if self.new_instance_loader == LoaderType::Forge
                        && amc_minecraft::loaders::forge::is_legacy_unsafe_line(
                            &self.new_instance_version,
                        )
                    {
                        ui.label(
                            egui::RichText::new(lang.inst_warn_legacy_forge())
                                .font(egui::FontId::proportional(12.0))
                                .color(WARNING),
                        );
                    }

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {} MB",
                            lang.inst_modal_ram(),
                            self.new_instance_ram
                        ))
                        .color(TEXT_PRIMARY),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.new_instance_ram, 1024..=16384).step_by(512.0),
                    );

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.inst_modal_btn_create())
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .clicked()
                        {
                            let mut inst = Instance::new(
                                &self.new_instance_name,
                                &self.new_instance_version,
                                self.new_instance_loader,
                            );
                            inst.ram_mb = Some(self.new_instance_ram);
                            *selected_instance = Some(inst.id);
                            action = InstanceAction::Created(inst);
                            self.show_create_modal = false;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            self.show_create_modal = false;
                        }
                    });
                });
        }

        // Clone Instance Modal Dialog: duplicate with an optional
        // worlds/saves copy (CONCEPT P2).
        if self.show_clone_modal {
            egui::Window::new(lang.clone_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(360.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.clone_with_saves, lang.clone_saves());
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.inst_btn_clone())
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .clicked()
                        {
                            if let Some(id) = self.clone_instance_id {
                                action = InstanceAction::Cloned(id, self.clone_with_saves);
                            }
                            self.show_clone_modal = false;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            self.show_clone_modal = false;
                        }
                    });
                });
        }

        // Edit Instance Modal Dialog
        if self.show_edit_modal {
            let mut close_edit = false;
            let mut save_edit = false;

            egui::Window::new(lang.inst_modal_edit_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY));
                    ui.add(TextEdit::singleline(&mut self.edit_instance_name).desired_width(400.0));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_version).desired_width(400.0),
                    );

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("edit_loader_select")
                        .selected_text(self.edit_instance_loader.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Vanilla,
                                "Vanilla",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Fabric,
                                "Fabric",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Quilt,
                                "Quilt",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::Forge,
                                "Forge",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::NeoForge,
                                "NeoForge",
                            );
                            ui.selectable_value(
                                &mut self.edit_instance_loader,
                                LoaderType::OptiFine,
                                "OptiFine",
                            );
                        });
                    if self.edit_instance_loader == LoaderType::Forge
                        && amc_minecraft::loaders::forge::is_legacy_unsafe_line(
                            &self.edit_instance_version,
                        )
                    {
                        ui.label(
                            egui::RichText::new(lang.inst_warn_legacy_forge())
                                .font(egui::FontId::proportional(12.0))
                                .color(WARNING),
                        );
                    }
                    // Pinned loader version (reproducible launches): reset
                    // returns to latest on the next launch.
                    if let Some(pin) = self.edit_loader_pin.clone() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.loader_pinned(&pin))
                                    .font(egui::FontId::proportional(12.0))
                                    .color(TEXT_MUTED),
                            );
                            if ui.button(lang.loader_pin_reset()).clicked() {
                                if let Some(id) = self.edit_instance_id {
                                    self.loader_pin_reset = Some(id);
                                }
                                self.edit_loader_pin = None;
                            }
                        });
                    }

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {} MB",
                            lang.inst_modal_ram(),
                            self.edit_instance_ram
                        ))
                        .color(TEXT_PRIMARY),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.edit_instance_ram, 1024..=32768).step_by(512.0),
                    );

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_icon_title()).color(TEXT_PRIMARY));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                        for (key, _) in InstanceIcon::ALL {
                            let active = self.edit_instance_icon.as_deref() == Some(key);
                            let (rect, resp) =
                                ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
                            let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                            let bg = lerp_color(BG_CARD, BG_HOVER, hover_t);
                            let stroke_col = if active {
                                RUBY_LIGHT.into()
                            } else {
                                lerp_color(BORDER_DEFAULT, RUBY, hover_t)
                            };
                            ui.painter().rect_filled(rect, crate::theme::rounding(), bg);
                            ui.painter().rect_stroke(
                                rect,
                                crate::theme::rounding(),
                                Stroke::new(1.0_f32, stroke_col),
                            );
                            icons::draw_centered(
                                ui.painter(),
                                icons::instance_preset(key).unwrap_or(Icon::Grid),
                                rect.center(),
                                20.0,
                                TEXT_PRIMARY,
                            );
                            if resp.clicked() {
                                self.edit_instance_icon = Some(key.to_string());
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(lang.inst_icon_custom()).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PNG image", &["png"])
                                .pick_file()
                            {
                                if let Some(target_id) = self.edit_instance_id {
                                    if let Some(inst) = instances.iter().find(|i| i.id == target_id)
                                    {
                                        let dest = inst
                                            .get_game_dir(instances_dir)
                                            .join(InstanceIcon::CUSTOM_ICON_FILE);
                                        if let Some(parent) = dest.parent() {
                                            let _ = std::fs::create_dir_all(parent);
                                        }
                                        let _ = std::fs::copy(&path, &dest);
                                        self.icon_failed.remove(&target_id);
                                        self.icon_textures.remove(&target_id);
                                    }
                                }
                            }
                        }
                        if ui.button(lang.inst_icon_none()).clicked() {
                            self.edit_instance_icon = None;
                        }
                        if ui.button(lang.inst_icon_remove()).clicked() {
                            if let Some(target_id) = self.edit_instance_id {
                                if let Some(inst) = instances.iter().find(|i| i.id == target_id) {
                                    let _ = std::fs::remove_file(
                                        inst.get_game_dir(instances_dir)
                                            .join(InstanceIcon::CUSTOM_ICON_FILE),
                                    );
                                    self.icon_failed.remove(&target_id);
                                    self.icon_textures.remove(&target_id);
                                }
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_tags_label()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_tags)
                            .hint_text(lang.inst_tags_hint())
                            .desired_width(400.0),
                    );
                    ui.add_space(6.0);
                    ui.checkbox(&mut self.edit_instance_pinned, lang.inst_pin_label());

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_account_label()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("edit_account_select")
                        .selected_text(
                            self.edit_instance_account
                                .and_then(|id| accounts.iter().find(|a| a.id == id))
                                .map(|a| a.username.clone())
                                .unwrap_or_else(|| lang.inst_account_default().to_string()),
                        )
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    self.edit_instance_account.is_none(),
                                    lang.inst_account_default(),
                                )
                                .clicked()
                            {
                                self.edit_instance_account = None;
                            }
                            for acc in accounts {
                                if ui
                                    .selectable_label(
                                        self.edit_instance_account == Some(acc.id),
                                        &acc.username,
                                    )
                                    .clicked()
                                {
                                    self.edit_instance_account = Some(acc.id);
                                }
                            }
                        });

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_group_label()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_group)
                            .hint_text(lang.inst_group_hint())
                            .desired_width(400.0),
                    );
                    if !ctx.group_names.is_empty() {
                        ui.label(
                            egui::RichText::new(ctx.group_names.join(" • "))
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );
                    }

                    ui.add_space(10.0);
                    ui.checkbox(&mut self.edit_use_custom_size, lang.inst_res_custom());
                    if self.edit_use_custom_size {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("W: {}", self.edit_window_w))
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.edit_window_w, 640..=3840)
                                    .step_by(10.0),
                            );
                            ui.label(
                                egui::RichText::new(format!("H: {}", self.edit_window_h))
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.edit_window_h, 480..=2160)
                                    .step_by(10.0),
                            );
                        });
                    }
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(lang.inst_res_mode()).color(TEXT_PRIMARY));
                    egui::ComboBox::from_id_salt("edit_display_mode_select")
                        .selected_text(match self.edit_fullscreen {
                            None => lang.inst_res_default(),
                            Some(false) => lang.inst_res_windowed(),
                            Some(true) => lang.inst_res_fullscreen(),
                        })
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    self.edit_fullscreen.is_none(),
                                    lang.inst_res_default(),
                                )
                                .clicked()
                            {
                                self.edit_fullscreen = None;
                            }
                            if ui
                                .selectable_label(
                                    self.edit_fullscreen == Some(false),
                                    lang.inst_res_windowed(),
                                )
                                .clicked()
                            {
                                self.edit_fullscreen = Some(false);
                            }
                            if ui
                                .selectable_label(
                                    self.edit_fullscreen == Some(true),
                                    lang.inst_res_fullscreen(),
                                )
                                .clicked()
                            {
                                self.edit_fullscreen = Some(true);
                            }
                        });

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_notes_label()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_notes)
                            .hint_text(lang.inst_notes_hint())
                            .desired_width(400.0),
                    );
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(lang.inst_pre_label()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_pre)
                            .hint_text(lang.inst_cmd_hint())
                            .desired_width(400.0)
                            .font(egui::FontId::monospace(12.0)),
                    );
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(lang.inst_post_label()).color(TEXT_PRIMARY));
                    ui.add(
                        TextEdit::singleline(&mut self.edit_instance_post)
                            .hint_text(lang.inst_cmd_hint())
                            .desired_width(400.0)
                            .font(egui::FontId::monospace(12.0)),
                    );

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new(lang.inst_modal_btn_save())
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .clicked()
                        {
                            save_edit = true;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            close_edit = true;
                        }
                    });
                });

            if save_edit {
                if let Some(target_id) = self.edit_instance_id {
                    if let Some(inst) = instances.iter_mut().find(|i| i.id == target_id) {
                        inst.name = self.edit_instance_name.clone();
                        // Porting (CONCEPT P4): a version change moves the
                        // pin and raises a notice; the app re-checks mods.
                        let old_version = inst.game_version.clone();
                        let new_version = self.edit_instance_version.clone();
                        if inst.port_to_version(&new_version) {
                            self.port_notice =
                                Some((target_id, old_version, inst.game_version.clone()));
                        }
                        inst.loader = self.edit_instance_loader;
                        inst.ram_mb = Some(self.edit_instance_ram);
                        inst.icon = self.edit_instance_icon.clone();
                        inst.tags = parse_tags(&self.edit_instance_tags);
                        inst.pinned = self.edit_instance_pinned;
                        inst.default_account = self.edit_instance_account;
                        let group_name = self.edit_instance_group.trim().to_string();
                        inst.group = if group_name.is_empty() {
                            None
                        } else {
                            Some(group_name)
                        };
                        inst.window_size = if self.edit_use_custom_size {
                            Some((self.edit_window_w, self.edit_window_h))
                        } else {
                            None
                        };
                        inst.fullscreen = self.edit_fullscreen;
                        inst.notes = self.edit_instance_notes.clone();
                        inst.pre_launch_cmd = self.edit_instance_pre.trim().to_string();
                        inst.post_exit_cmd = self.edit_instance_post.trim().to_string();
                        action = InstanceAction::Updated(inst.clone());
                    }
                    self.icon_textures.remove(&target_id);
                }
                self.icon_failed.clear();
                self.show_edit_modal = false;
            }

            if close_edit {
                self.show_edit_modal = false;
            }
        }

        // Per-instance management modal (worlds / configs / health).
        let egui_ctx = ui.ctx().clone();
        self.manage.show(
            &egui_ctx,
            instances,
            ctx.instances_dir,
            ctx.backups_root,
            lang,
        );

        // Trash mutations need an immediate rescan, not the 2s throttle.
        if matches!(
            action,
            InstanceAction::Trashed(_)
                | InstanceAction::Restored(_)
                | InstanceAction::TrashPurged(_)
                | InstanceAction::TrashEmptied
        ) {
            self.trash_at = None;
        }

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_local_mods_in() {
        let installed = vec!["sodium.jar".to_string()];
        assert!(!has_local_mods_in(&[], &installed));
        assert!(!has_local_mods_in(&["sodium.jar".to_string()], &installed));
        assert!(has_local_mods_in(
            &["sodium.jar".to_string(), "mystery.jar".to_string()],
            &installed
        ));
        // Disabled files count as mods too.
        assert!(has_local_mods_in(&["old.jar.disabled".to_string()], &[]));
        assert!(!has_local_mods_in(&["config.txt".to_string()], &[]));
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("Short", 22), "Short");
        assert_eq!(
            truncate_name("Exactly twenty-two chars!!", 22),
            "Exactly twenty-two cha…"
        );
        assert_eq!(truncate_name("", 22), "");
    }

    #[test]
    fn test_apply_template_fills_form() {
        let lang = Language::English;
        let mut page = InstancesPage {
            new_instance_name: String::new(),
            ..InstancesPage::default()
        };
        page.apply_template(InstanceTemplate::OPTIMIZED, lang);
        assert_eq!(page.new_instance_loader, LoaderType::Fabric);
        assert_eq!(page.new_instance_ram, 6144);
        assert_eq!(page.new_instance_name, "Optimized");
    }

    #[test]
    fn test_apply_template_keeps_custom_name() {
        let lang = Language::English;
        let mut page = InstancesPage {
            new_instance_name: "My Pack".to_string(),
            ..InstancesPage::default()
        };
        page.apply_template(InstanceTemplate::EMPTY, lang);
        assert_eq!(page.new_instance_loader, LoaderType::Vanilla);
        assert_eq!(page.new_instance_ram, 2048);
        assert_eq!(page.new_instance_name, "My Pack");
    }

    fn tagged(name: &str, tags: &[&str], pinned: bool, mins: u64) -> Instance {
        let mut inst = Instance::new(name, "1.20.1", LoaderType::Vanilla);
        inst.tags = tags.iter().map(|t| t.to_string()).collect();
        inst.pinned = pinned;
        inst.total_played_minutes = mins;
        inst
    }

    #[test]
    fn test_matches_filter_query_and_tag() {
        let inst = tagged("Survival Friends", &["smp", "vanilla"], false, 0);
        assert!(matches_filter(&inst, "", None));
        assert!(matches_filter(&inst, "surv", None));
        assert!(!matches_filter(&inst, "creative", None));
        assert!(matches_filter(&inst, "", Some("smp")));
        assert!(!matches_filter(&inst, "", Some("modded")));
        assert!(!matches_filter(&inst, "surv", Some("modded")));
    }

    #[test]
    fn test_compare_instances_pinned_first_then_mode() {
        let pinned = tagged("Pinned", &[], true, 0);
        let played = tagged("Played", &[], false, 120);
        let fresh = tagged("Fresh", &[], false, 0);
        // Pinned floats above everything in every mode.
        assert_eq!(
            compare_instances(&pinned, &played, InstanceSort::Playtime),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_instances(&played, &pinned, InstanceSort::Name),
            std::cmp::Ordering::Greater
        );
        // Playtime mode: most played first.
        assert_eq!(
            compare_instances(&played, &fresh, InstanceSort::Playtime),
            std::cmp::Ordering::Less
        );
        // Name mode: alphabetical, case-insensitive.
        assert_eq!(
            compare_instances(&fresh, &played, InstanceSort::Name),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_parse_tags() {
        assert_eq!(
            parse_tags("smp, vanilla ,, smp,"),
            vec!["smp".to_string(), "vanilla".to_string()]
        );
        assert!(parse_tags("  , ").is_empty());
    }
}
