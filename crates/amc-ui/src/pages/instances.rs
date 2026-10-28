use crate::pages::instance_manage::InstanceManage;
use crate::theme::{
    lerp_color, palette_id, with_alpha, PaletteId, BG_CARD, BG_HOVER, BORDER_DEFAULT,
    BORDER_STRONG, MODRINTH_BG, MODRINTH_FIELD, MODRINTH_GREEN, MODRINTH_MUTED, MODRINTH_SURFACE,
    MODRINTH_SURFACE_HIGH, MODRINTH_SURFACE_HOVER, MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST,
    MODRINTH_TEXT_SECONDARY, RUBY, RUBY_DIM, RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY,
    WARNING,
};
use crate::widgets::choice_chip;
use crate::widgets::full::tag;
use crate::widgets::icons::{self, icon_button, Icon};
use amc_auth::Account;
use amc_core::types::{Instance, InstanceIcon, InstanceTemplate, LoaderType};
use amc_core::Language;
use amc_mods::ModSearchResult;
use egui::{pos2, vec2, Align2, Color32, Rect, Rounding, ScrollArea, Sense, Stroke, TextEdit, Ui};
use std::collections::{BTreeMap, HashMap, HashSet};
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
    /// Modrinth library: a new empty custom group was created.
    GroupCreated(String),
}

/// Visual skin of the instance list. `Amc` is the launcher's own card grid;
/// `Modrinth` replicates the library page of the Modrinth App (big cards
/// with a hover Play button under the search toolbar); `Multimc` is the
/// icon-grid instance view of MultiMC (grouped 100px cells).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibrarySkin {
    #[default]
    Amc,
    Modrinth,
    Multimc,
    /// LabyMod: dark hero header above the shared instance cards.
    Labymod,
}

/// Loader accent shared by the card strip, the MultiMC icon tiles and the
/// app's instance toolbar.
pub fn loader_color(loader: LoaderType) -> Color32 {
    match loader {
        LoaderType::Vanilla => Color32::from_rgb(0x38, 0x8E, 0x3C),
        LoaderType::Fabric => RUBY_LIGHT.into(),
        LoaderType::Quilt => Color32::from_rgb(0x9C, 0x27, 0xB0),
        LoaderType::Forge => Color32::from_rgb(0xE6, 0x51, 0x00),
        LoaderType::NeoForge => Color32::from_rgb(0xFF, 0x6D, 0x00),
        LoaderType::OptiFine => Color32::from_rgb(0x00, 0x83, 0x8F),
    }
}

/// LabyMod Instances header: dark rounded banner with a blue glow and the
/// instance count chip, matching the launcher's hero surfaces.
fn labymod_instances_header(ui: &mut Ui, lang: Language) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 106.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(16.0), BG_CARD);
    painter.rect_stroke(
        rect,
        Rounding::same(16.0),
        Stroke::new(1.0_f32, BORDER_DEFAULT),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 23.0),
        Align2::LEFT_CENTER,
        lang.nav_instances().to_uppercase(),
        egui::FontId::proportional(10.5),
        crate::theme::ACCENT_LIGHT.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 49.0),
        Align2::LEFT_CENTER,
        lang.labymod_instances_title(),
        egui::FontId::new(27.0, egui::FontFamily::Name("bold".into())),
        TEXT_HEADING.into(),
    );
    painter.text(
        pos2(rect.left() + 20.0, rect.top() + 79.0),
        Align2::LEFT_CENTER,
        lang.labymod_instances_subtitle(),
        egui::FontId::proportional(11.5),
        TEXT_MUTED.into(),
    );
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
    /// Card-grid style of the list section (AMC cards vs Modrinth library).
    pub skin: LibrarySkin,
}

pub struct InstancesPage {
    pub search_query: String,
    pub show_create_modal: bool,
    pub new_instance_name: String,
    pub new_instance_version: String,
    pub new_instance_loader: LoaderType,
    pub new_instance_ram: u32,
    /// Modrinth create dialog: first-frame focus for the name field.
    create_modal_seen: bool,

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

    // Background disk-usage scan (CONCEPT disk space).
    disk_sizes: HashMap<Uuid, u64>,
    disk_scanned_for: Vec<Uuid>,
    disk_rx: Option<std::sync::mpsc::Receiver<Vec<(Uuid, u64, bool)>>>,
    local_badges: HashMap<Uuid, bool>,

    // Organization (CONCEPT, P2): tag filter, sort, icon texture cache.
    pub active_tag: Option<String>,
    pub sort_mode: InstanceSort,
    /// Modrinth library: card grouping mode.
    pub group_by: LibraryGroupBy,
    /// Modrinth library: loader/version checkboxes.
    modrinth_filters: ModrinthFilters,
    /// Modrinth library: collapsed section keys (`"Loader:Fabric"`).
    modrinth_collapsed: HashSet<String>,
    icon_textures: HashMap<Uuid, egui::TextureHandle>,
    icon_failed: std::collections::HashSet<Uuid>,
    /// Memoized filter+sort (per-frame `to_lowercase`/`sort` avoided):
    /// recomputed only when query/tag/sort/filters or instance revision changes.
    filter_cache: Vec<usize>,
    filter_key: Option<InstanceFilterKey>,
    tags_cache: Vec<String>,
    tags_rev: u64,
    trash_cache: Vec<amc_core::TrashedInstance>,
    trash_at: Option<std::time::Instant>,

    // Modrinth library: New group modal.
    show_new_group_modal: bool,
    new_group_name: String,
    new_group_search: String,
    new_group_selected: HashSet<Uuid>,
    // Modrinth library: right-click card menu (instance, pointer position).
    modrinth_menu: Option<(Uuid, egui::Pos2)>,
    modrinth_menu_fresh: bool,

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
    /// Collapsed MultiMC group headers (reference `VisualGroup` collapse).
    multimc_collapsed: std::collections::HashSet<String>,
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
            create_modal_seen: false,

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
            group_by: LibraryGroupBy::default(),
            modrinth_filters: ModrinthFilters::default(),
            modrinth_collapsed: HashSet::new(),
            icon_textures: HashMap::new(),
            icon_failed: std::collections::HashSet::new(),
            filter_cache: Vec::new(),
            filter_key: None,
            tags_cache: Vec::new(),
            tags_rev: 0,
            trash_cache: Vec::new(),
            trash_at: None,
            show_new_group_modal: false,
            new_group_name: String::new(),
            new_group_search: String::new(),
            new_group_selected: HashSet::new(),
            modrinth_menu: None,
            modrinth_menu_fresh: false,
            manage: InstanceManage::default(),
            discover_open: false,
            discover: Vec::new(),
            discover_loading: false,
            discover_requested: false,
            discover_install: None,
            port_notice: None,
            show_clone_modal: false,
            clone_instance_id: None,
            clone_with_saves: true,
            shortcut_request: None,
            multimc_collapsed: std::collections::HashSet::new(),
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

/// MultiMC instance-view label: at most two wrapped lines with an ellipsis.
/// Keeps one very long instance name from stretching its whole grid row.
fn multimc_name_galley(
    painter: &egui::Painter,
    name: &str,
    font: egui::FontId,
    max_w: f32,
) -> std::sync::Arc<egui::Galley> {
    let full = painter.layout(name.to_owned(), font.clone(), Color32::PLACEHOLDER, max_w);
    let line_h = painter
        .layout_no_wrap("Mg".to_owned(), font.clone(), Color32::PLACEHOLDER)
        .size()
        .y;
    let max_h = line_h * 2.0 + 1.0;
    if full.size().y <= max_h || name.is_empty() {
        return full;
    }
    let chars: Vec<char> = name.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    let mut best = "…".to_string();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let candidate = format!("{}…", chars[..mid].iter().collect::<String>().trim_end());
        let galley = painter.layout(candidate.clone(), font.clone(), Color32::PLACEHOLDER, max_w);
        if galley.size().y <= max_h {
            best = candidate;
            lo = mid;
        } else {
            hi = mid.saturating_sub(1);
        }
    }
    painter.layout(best, font, Color32::PLACEHOLDER, max_w)
}

/// Foreground for native dialog action buttons: light MultiMC needs dark
/// labels, the dark shells keep white.
fn dialog_button_fg(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::WHITE
    } else {
        TEXT_HEADING.into()
    }
}

/// Accent primary action for the LabyMod and Millida dialogs. It mirrors the
/// shell's toolbar create button; other shells keep the native egui button so
/// the original Simple/Professional visuals stay untouched.
fn shell_primary_button(ui: &mut Ui, label: &str, enabled: bool) -> bool {
    let button = egui::Button::new(egui::RichText::new(label).strong().color(Color32::WHITE))
        .fill(RUBY)
        .stroke(Stroke::new(1.0_f32, RUBY_LIGHT))
        .rounding(crate::theme::rounding());
    ui.add_enabled(enabled, button).clicked()
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
    /// Modrinth library: loader label, ascending.
    Loader,
    /// Modrinth library: numeric-aware game version, ascending.
    GameVersion,
}

impl InstanceSort {
    /// Aleph (AMC) sort choices. Kept small so the shared page is unchanged.
    pub const ALL: [Self; 4] = [Self::Recent, Self::Name, Self::Created, Self::Playtime];

    /// Modrinth library ordering, matching the reference menu order.
    pub const MODRINTH_ALL: [Self; 6] = [
        Self::Name,
        Self::Recent,
        Self::Playtime,
        Self::Created,
        Self::Loader,
        Self::GameVersion,
    ];

    pub fn label(&self, lang: Language) -> &'static str {
        match self {
            Self::Recent => lang.inst_sort_recent(),
            Self::Name => lang.inst_sort_name(),
            Self::Created => lang.inst_sort_created(),
            Self::Playtime => lang.inst_sort_played(),
            Self::Loader => lang.inst_word_loader(),
            Self::GameVersion => lang.inst_word_game_version(),
        }
    }

    /// Reference wording for the Modrinth library menu.
    pub fn modrinth_label(&self, lang: Language) -> &'static str {
        match self {
            Self::Recent => lang.inst_sort_last_played(),
            Self::Name => lang.inst_sort_name(),
            Self::Created => lang.inst_sort_date_created(),
            Self::Playtime => lang.inst_sort_hours_played(),
            Self::Loader => lang.inst_word_loader(),
            Self::GameVersion => lang.inst_word_game_version(),
        }
    }
}

/// Modrinth library card grouping (reference `sort-menu.vue`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryGroupBy {
    #[default]
    Group,
    Loader,
    GameVersion,
    None,
}

impl LibraryGroupBy {
    pub const MODRINTH_ALL: [Self; 4] = [Self::Group, Self::Loader, Self::GameVersion, Self::None];

    pub fn label(&self, lang: Language) -> &'static str {
        match self {
            Self::Group => lang.inst_group_custom(),
            Self::Loader => lang.inst_word_loader(),
            Self::GameVersion => lang.inst_word_game_version(),
            Self::None => lang.inst_group_none(),
        }
    }
}

/// Modrinth library filter selections (reference `filters` in `use-library.ts`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ModrinthFilters {
    loaders: Vec<LoaderType>,
    versions: Vec<String>,
}

impl ModrinthFilters {
    fn is_empty(&self) -> bool {
        self.loaders.is_empty() && self.versions.is_empty()
    }

    fn toggle_loader(&mut self, loader: LoaderType) {
        if let Some(pos) = self.loaders.iter().position(|l| *l == loader) {
            self.loaders.remove(pos);
        } else {
            self.loaders.push(loader);
        }
    }

    fn toggle_version(&mut self, version: &str) {
        if let Some(pos) = self.versions.iter().position(|v| v == version) {
            self.versions.remove(pos);
        } else {
            self.versions.push(version.to_string());
        }
    }
}

/// Memoization key for the Modrinth/Aleph filtered instance indices.
#[derive(Debug, PartialEq, Eq)]
struct InstanceFilterKey {
    query: String,
    tag: Option<String>,
    sort: InstanceSort,
    loaders: Vec<LoaderType>,
    versions: Vec<String>,
    rev: u64,
}

/// Search + tag + loader/version predicate for the instance list (pure).
fn matches_filter(
    inst: &Instance,
    query: &str,
    active_tag: Option<&str>,
    filters: &ModrinthFilters,
) -> bool {
    if let Some(tag) = active_tag {
        if !inst.tags.iter().any(|t| t == tag) {
            return false;
        }
    }
    if !filters.loaders.is_empty() && !filters.loaders.contains(&inst.loader) {
        return false;
    }
    if !filters.versions.is_empty() && !filters.versions.iter().any(|v| v == &inst.game_version) {
        return false;
    }
    if query.is_empty() {
        return true;
    }
    inst.name.to_lowercase().contains(query)
        || inst.game_version.to_lowercase().contains(query)
        || inst.loader.as_str().to_lowercase().contains(query)
}

/// Numeric-aware Minecraft version comparison (`1.9` < `1.10`).
fn natural_version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut a_parts = a.split(['.', '-', '+', '_']);
    let mut b_parts = b.split(['.', '-', '+', '_']);
    loop {
        match (a_parts.next(), b_parts.next()) {
            (Some(x), Some(y)) => {
                let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(xn), Ok(yn)) => xn.cmp(&yn),
                    _ => x.to_ascii_lowercase().cmp(&y.to_ascii_lowercase()),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
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
        InstanceSort::Loader => a
            .loader
            .as_str()
            .to_ascii_lowercase()
            .cmp(&b.loader.as_str().to_ascii_lowercase())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        InstanceSort::GameVersion => natural_version_cmp(&a.game_version, &b.game_version)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
    }
}

/// Cheap revision of the instance list for cache invalidation (no alloc).
/// Any rename/version/loader/pin/play/tag/group change flips the revision.
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
        inst.created_at.timestamp_millis().hash(&mut hasher);
        inst.last_played
            .map(|dt| dt.timestamp_millis())
            .hash(&mut hasher);
        inst.group.hash(&mut hasher);
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

/// Tailwind `brightness-110`-style per-channel multiply, clamped to 255.
fn m_brightness(color: Color32, factor: f32) -> Color32 {
    let scale = |v: u8| ((v as f32 * factor).min(255.0)) as u8;
    Color32::from_rgb(scale(color.r()), scale(color.g()), scale(color.b()))
}

/// Width a Modrinth-style toolbar button occupies for `label` (+ icon);
/// used to reserve room for the search field next to it.
fn m_button_width(ui: &Ui, icon: Option<Icon>, label: &str, height: f32) -> f32 {
    let text = if height > 34.0 { 15.0 } else { 13.5 };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(text),
        Color32::PLACEHOLDER,
    );
    20.0 + galley.size().x + if icon.is_some() { 24.0 } else { 0.0 }
}

/// Modrinth-style toolbar button (reference `ButtonFrame`: 36px tall,
/// rounded-xl, secondary gray or brand green with black content).
fn m_button(ui: &mut Ui, icon: Option<Icon>, label: &str, brand: bool, height: f32) -> bool {
    m_button_enabled(ui, icon, label, brand, height, true)
}

/// `m_button` with the reference's disabled state (50% content).
fn m_button_enabled(
    ui: &mut Ui,
    icon: Option<Icon>,
    label: &str,
    brand: bool,
    height: f32,
    enabled: bool,
) -> bool {
    let text_size = if height > 34.0 { 15.0 } else { 13.5 };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(text_size),
        Color32::PLACEHOLDER,
    );
    let width = 20.0 + galley.size().x + if icon.is_some() { 24.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hover = if enabled {
        ui.ctx().animate_bool_responsive(resp.id, resp.hovered())
    } else {
        0.0
    };
    let (base, hot) = if brand {
        (MODRINTH_GREEN, m_brightness(MODRINTH_GREEN, 1.1))
    } else {
        (
            MODRINTH_SURFACE_HOVER,
            m_brightness(MODRINTH_SURFACE_HOVER, 1.1),
        )
    };
    let radius = Rounding::same(12.0);
    let fill = if enabled {
        lerp_color(base, hot, hover)
    } else {
        MODRINTH_SURFACE_HOVER
    };
    ui.painter().rect_filled(rect, radius, fill);
    let content = if brand {
        if enabled {
            Color32::BLACK
        } else {
            MODRINTH_MUTED
        }
    } else if enabled {
        MODRINTH_TEXT_CONTRAST
    } else {
        MODRINTH_MUTED
    };
    if brand && enabled {
        // Reference colored buttons carry a soft top highlight.
        ui.painter().line_segment(
            [
                pos2(rect.left() + 10.0, rect.top() + 1.0),
                pos2(rect.right() - 10.0, rect.top() + 1.0),
            ],
            Stroke::new(1.0_f32, with_alpha(Color32::WHITE, 70)),
        );
    } else if !brand {
        let stroke = if enabled {
            MODRINTH_SURFACE_HIGH
        } else {
            with_alpha(MODRINTH_SURFACE_HIGH, 150)
        };
        ui.painter()
            .rect_stroke(rect, radius, Stroke::new(1.0_f32, stroke));
    }
    let icon_w = if icon.is_some() { 18.0 } else { 0.0 };
    let icon_gap = if icon.is_some() { 6.0 } else { 0.0 };
    let content_w = icon_w + icon_gap + galley.size().x;
    let left = rect.center().x - content_w / 2.0;
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(
                pos2(left + icon_w / 2.0, rect.center().y),
                vec2(icon_w, icon_w),
            ),
            content,
            1.6,
        );
    }
    ui.painter().galley(
        pos2(
            left + icon_w + icon_gap,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        content,
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    enabled && resp.clicked()
}

/// Modrinth-style chip used by the creation dialog (loader/template picks).
/// Active chips use the brand green with dark content, like the reference's
/// selected `Chips` item; inactive chips sit on surface-4.
fn modrinth_pick_chip(ui: &mut Ui, label: &str, active: bool) -> bool {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(13.5),
        Color32::PLACEHOLDER,
    );
    let width = (galley.size().x + 22.0).max(58.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let radius = Rounding::same(10.0);
    let fill = if active {
        lerp_color(MODRINTH_GREEN, crate::theme::MODRINTH_GREEN_LIGHT, hover)
    } else {
        lerp_color(
            MODRINTH_SURFACE_HOVER,
            m_brightness(MODRINTH_SURFACE_HOVER, 1.1),
            hover,
        )
    };
    ui.painter().rect_filled(rect, radius, fill);
    if !active {
        ui.painter()
            .rect_stroke(rect, radius, Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH));
    }
    let text = if active {
        MODRINTH_BG
    } else {
        MODRINTH_TEXT_CONTRAST
    };
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Modrinth-style RAM slider: green track, white thumb and a compact value
/// plate on the right. Keeps the dialog visually consistent instead of
/// falling back to egui's default slider chrome.
fn modrinth_ram_slider(ui: &mut Ui, value: &mut u32, width: f32) -> egui::Response {
    const MIN_MB: u32 = 1024;
    const MAX_MB: u32 = 16384;
    const STEP_MB: u32 = 512;

    let (rect, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    let plate_w = 76.0;
    let track_w = (width - plate_w - 16.0).max(80.0);
    let track = Rect::from_center_size(
        pos2(rect.left() + track_w / 2.0, rect.center().y),
        vec2(track_w, 6.0),
    );
    let resp = ui.interact(
        Rect::from_min_max(
            pos2(rect.left(), rect.top()),
            pos2(track.right() + 10.0, rect.bottom()),
        ),
        ui.id().with("modrinth_ram_slider"),
        Sense::click_and_drag(),
    );
    if resp.dragged() || resp.clicked() {
        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
            let t = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
            let raw = MIN_MB as f32 + t * (MAX_MB - MIN_MB) as f32;
            let stepped = ((raw / STEP_MB as f32).round() as u32) * STEP_MB;
            *value = stepped.clamp(MIN_MB, MAX_MB);
        }
    }
    let value = (*value).clamp(MIN_MB, MAX_MB);
    let t = (value - MIN_MB) as f32 / (MAX_MB - MIN_MB) as f32;
    ui.painter()
        .rect_filled(track, Rounding::same(3.0), MODRINTH_SURFACE_HIGH);
    ui.painter().rect_filled(
        Rect::from_min_size(track.min, vec2(track.width() * t, track.height())),
        Rounding::same(3.0),
        MODRINTH_GREEN,
    );
    let knob = pos2(track.left() + track.width() * t, track.center().y);
    ui.painter().circle_filled(
        knob,
        if resp.hovered() || resp.dragged() {
            8.0
        } else {
            7.0
        },
        Color32::WHITE,
    );
    ui.painter().circle_stroke(
        knob,
        if resp.hovered() || resp.dragged() {
            8.0
        } else {
            7.0
        },
        Stroke::new(1.0_f32, with_alpha(MODRINTH_BG, 90)),
    );

    let plate = Rect::from_min_size(
        pos2(rect.right() - plate_w, rect.center().y - 14.0),
        vec2(plate_w, 28.0),
    );
    ui.painter()
        .rect_filled(plate, Rounding::same(8.0), MODRINTH_FIELD);
    ui.painter().rect_stroke(
        plate,
        Rounding::same(8.0),
        Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
    );
    ui.painter().text(
        plate.center(),
        Align2::CENTER_CENTER,
        format!("{value} MB"),
        egui::FontId::proportional(12.0),
        MODRINTH_TEXT_CONTRAST,
    );
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    resp
}

/// Modrinth-style combobox button: leading icon, current value, caret.
fn m_combo(ui: &mut Ui, icon: Icon, label: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(15.0),
        Color32::PLACEHOLDER,
    );
    let width = 20.0 + 18.0 + 6.0 + galley.size().x + 4.0 + 14.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    let radius = Rounding::same(12.0);
    ui.painter().rect_filled(
        rect,
        radius,
        lerp_color(
            MODRINTH_SURFACE_HOVER,
            m_brightness(MODRINTH_SURFACE_HOVER, 1.1),
            hover,
        ),
    );
    ui.painter()
        .rect_stroke(rect, radius, Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH));
    let mut x = rect.left() + 10.0;
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(x + 9.0, rect.center().y), vec2(18.0, 18.0)),
        MODRINTH_TEXT,
        1.5,
    );
    x += 18.0 + 6.0;
    let text_x = x;
    x += galley.size().x + 4.0;
    icons::draw(
        ui.painter(),
        Icon::CaretDown,
        Rect::from_center_size(pos2(x + 7.0, rect.center().y), vec2(13.0, 13.0)),
        MODRINTH_TEXT,
        1.5,
    );
    ui.painter().galley(
        pos2(text_x, rect.center().y - galley.size().y / 2.0),
        galley,
        MODRINTH_TEXT_CONTRAST,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Toggle one library combobox popup and close the other ones.
fn toggle_library_popup(ui: &Ui, id: egui::Id) {
    let open = ui.memory(|mem| mem.is_popup_open(id));
    ui.memory_mut(|mem| mem.close_popup());
    if !open {
        ui.memory_mut(|mem| mem.open_popup(id));
    }
}

/// Show a Modrinth-styled popup under a toolbar widget (reference surface-4
/// menu: 14px radius, surface-5 border, soft drop shadow).
fn modrinth_popup<R>(
    ui: &mut Ui,
    id: egui::Id,
    response: &egui::Response,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let old_fill = ui.style().visuals.window_fill;
    let old_stroke = ui.style().visuals.window_stroke;
    let old_rounding = ui.style().visuals.window_rounding;
    let old_shadow = ui.style().visuals.window_shadow;
    {
        let visuals = &mut ui.style_mut().visuals;
        visuals.window_fill = MODRINTH_SURFACE_HOVER;
        visuals.window_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
        visuals.window_rounding = Rounding::same(14.0);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(110),
        };
    }
    let result = egui::popup::popup_below_widget(
        ui,
        id,
        response,
        egui::popup::PopupCloseBehavior::CloseOnClickOutside,
        add_contents,
    );
    {
        let visuals = &mut ui.style_mut().visuals;
        visuals.window_fill = old_fill;
        visuals.window_stroke = old_stroke;
        visuals.window_rounding = old_rounding;
        visuals.window_shadow = old_shadow;
    }
    result
}

/// One row in a Modrinth combobox popup (reference option: semibold 15px,
/// selected row gets the green text and a faint green wash).
fn modrinth_menu_option(ui: &mut Ui, label: &str, selected: bool) -> bool {
    modrinth_menu_option_enabled(ui, label, selected, true)
}

/// `modrinth_menu_option` with a disabled state (used by "Clear filters").
fn modrinth_menu_option_enabled(ui: &mut Ui, label: &str, selected: bool, enabled: bool) -> bool {
    let width = ui.available_width().max(150.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 40.0), Sense::hover());
    let resp = ui.interact(rect, ui.id().with(label), Sense::click());
    let hover = enabled && resp.hovered();
    if selected {
        ui.painter().rect_filled(
            rect.shrink2(vec2(2.0, 1.0)),
            Rounding::same(10.0),
            with_alpha(MODRINTH_GREEN, 28),
        );
    } else if hover {
        ui.painter().rect_filled(
            rect.shrink2(vec2(2.0, 1.0)),
            Rounding::same(10.0),
            MODRINTH_SURFACE_HIGH,
        );
    }
    let color = if !enabled {
        with_alpha(MODRINTH_TEXT, 120)
    } else if selected {
        MODRINTH_GREEN
    } else {
        MODRINTH_TEXT_CONTRAST
    };
    ui.painter().text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(15.0),
        color,
    );
    if enabled && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    enabled && resp.clicked()
}

/// One checkbox row in the library filter popup; the box sits on the right
/// like the reference `DropdownFilterBar` submenu.
fn modrinth_check_option(ui: &mut Ui, label: &str, checked: bool, id: egui::Id) -> bool {
    let width = ui.available_width().max(160.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    let resp = ui.interact(rect, id, Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(vec2(2.0, 1.0)),
            Rounding::same(8.0),
            MODRINTH_SURFACE_HIGH,
        );
    }
    ui.painter().text(
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        MODRINTH_TEXT_CONTRAST,
    );
    let box_rect =
        Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(18.0, 18.0));
    if checked {
        ui.painter()
            .rect_filled(box_rect, Rounding::same(5.0), MODRINTH_GREEN);
        icons::draw(
            ui.painter(),
            Icon::Check,
            box_rect.shrink(4.0),
            Color32::BLACK,
            2.0,
        );
    } else {
        ui.painter().rect_stroke(
            box_rect,
            Rounding::same(5.0),
            Stroke::new(1.5_f32, MODRINTH_SURFACE_HIGH),
        );
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Reference `InstanceGroup` header: 40px row, bottom border, caret, semibold
/// name and a count chip. Returns true when clicked (collapse/expand).
fn modrinth_group_header(ui: &mut Ui, title: &str, count: usize, collapsed: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
    let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
    ui.painter().line_segment(
        [
            pos2(rect.left(), rect.bottom() - 0.5),
            pos2(rect.right(), rect.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
    );
    let caret = if collapsed {
        Icon::CaretRight
    } else {
        Icon::CaretDown
    };
    icons::draw(
        ui.painter(),
        caret,
        Rect::from_center_size(pos2(rect.left() + 10.0, rect.center().y), vec2(16.0, 16.0)),
        lerp_color(MODRINTH_TEXT_SECONDARY, MODRINTH_TEXT_CONTRAST, hover),
        1.6,
    );
    let title_pos = pos2(rect.left() + 34.0, rect.center().y);
    let title_color = lerp_color(MODRINTH_TEXT, MODRINTH_TEXT_CONTRAST, hover);
    let galley = ui.painter().layout_no_wrap(
        title.to_owned(),
        egui::FontId::proportional(16.0),
        title_color,
    );
    let text_w = galley.size().x;
    ui.painter().galley(
        pos2(title_pos.x, title_pos.y - galley.size().y / 2.0),
        galley,
        title_color,
    );
    if count > 0 {
        let count_text = count.to_string();
        let count_galley = ui.painter().layout_no_wrap(
            count_text,
            egui::FontId::proportional(12.0),
            MODRINTH_TEXT_SECONDARY,
        );
        let chip = Rect::from_min_size(
            pos2(title_pos.x + text_w + 8.0, rect.center().y - 10.0),
            vec2(count_galley.size().x + 14.0, 20.0),
        );
        ui.painter()
            .rect_filled(chip, Rounding::same(6.0), MODRINTH_SURFACE);
        ui.painter().rect_stroke(
            chip,
            Rounding::same(6.0),
            Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
        );
        ui.painter().galley(
            pos2(
                chip.center().x - count_galley.size().x / 2.0,
                chip.center().y - count_galley.size().y / 2.0,
            ),
            count_galley,
            MODRINTH_TEXT_SECONDARY,
        );
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// One rendered library section (reference `InstanceGroup`).
struct LibrarySection {
    key: String,
    title: String,
    indices: Vec<usize>,
}

/// Build the sections for the selected Modrinth grouping mode. Custom groups
/// sort alphabetically with "Ungrouped" last; loaders follow the reference
/// Fabric â†’ Forge â†’ NeoForge priority, versions are newest first.
fn modrinth_sections(
    instances: &[Instance],
    filtered: &[usize],
    group_by: LibraryGroupBy,
    known_groups: &[String],
    lang: Language,
) -> Vec<LibrarySection> {
    match group_by {
        LibraryGroupBy::None => {
            if filtered.is_empty() {
                Vec::new()
            } else {
                vec![LibrarySection {
                    key: "None".to_string(),
                    title: String::new(),
                    indices: filtered.to_vec(),
                }]
            }
        }
        LibraryGroupBy::Group => {
            let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for &idx in filtered {
                let name = instances[idx].group.clone().unwrap_or_default();
                map.entry(name).or_default().push(idx);
            }
            for name in known_groups {
                if !name.trim().is_empty() {
                    map.entry(name.clone()).or_default();
                }
            }
            map.entry(String::new()).or_default();

            let ungrouped = map.remove("").unwrap_or_default();
            let mut sections: Vec<LibrarySection> = map
                .into_iter()
                .map(|(name, indices)| LibrarySection {
                    key: format!("Group:{name}"),
                    title: name,
                    indices,
                })
                .collect();
            sections.sort_by_key(|s| s.title.to_ascii_lowercase());
            sections.push(LibrarySection {
                key: "Group:".to_string(),
                title: lang.inst_group_ungrouped().to_string(),
                indices: ungrouped,
            });
            sections
        }
        LibraryGroupBy::Loader => {
            let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for &idx in filtered {
                map.entry(instances[idx].loader.as_str().to_string())
                    .or_default()
                    .push(idx);
            }
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort_by(|a, b| {
                loader_group_priority(a)
                    .cmp(&loader_group_priority(b))
                    .then_with(|| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()))
            });
            keys.into_iter()
                .map(|name| {
                    let indices = map.remove(&name).unwrap_or_default();
                    LibrarySection {
                        key: format!("Loader:{name}"),
                        title: name,
                        indices,
                    }
                })
                .collect()
        }
        LibraryGroupBy::GameVersion => {
            let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for &idx in filtered {
                map.entry(instances[idx].game_version.clone())
                    .or_default()
                    .push(idx);
            }
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort_by(|a, b| natural_version_cmp(b, a));
            keys.into_iter()
                .map(|version| {
                    let indices = map.remove(&version).unwrap_or_default();
                    LibrarySection {
                        key: format!("Game version:{version}"),
                        title: version,
                        indices,
                    }
                })
                .collect()
        }
    }
}

/// Reference `libraryLoaderPriority`, then alphabetical for other loaders.
fn loader_group_priority(loader: &str) -> u8 {
    match loader.to_ascii_lowercase().as_str() {
        "fabric" => 0,
        "forge" => 1,
        "neoforge" => 2,
        _ => 3,
    }
}

/// Modrinth `InputFrame`: rounded-xl, surface-4 fill, surface-5 border,
/// optional leading icon. Returns the frame response.
fn modrinth_input(
    ui: &mut Ui,
    text: &mut String,
    id: egui::Id,
    width: f32,
    height: f32,
    hint: &str,
    icon: Option<Icon>,
) -> egui::Response {
    let left_pad = if icon.is_some() { 38.0 } else { 14.0 };
    let right_pad = 12.0;
    let field_left = ui.cursor().left();
    let text_w = (width - left_pad - right_pad).max(40.0);
    let v_pad = ((height - 18.0) / 2.0).max(4.0);
    let edit = TextEdit::singleline(text)
        .id(id)
        .frame(true)
        .desired_width(text_w)
        .margin(egui::Margin {
            left: left_pad,
            right: right_pad,
            top: v_pad,
            bottom: v_pad,
        })
        .hint_text(egui::RichText::new(hint).color(MODRINTH_TEXT_SECONDARY))
        .font(egui::FontId::proportional(14.0));
    let resp = ui
        .scope(|ui| {
            let visuals = &mut ui.style_mut().visuals;
            visuals.extreme_bg_color = MODRINTH_FIELD;
            visuals.widgets.inactive.bg_fill = MODRINTH_FIELD;
            visuals.widgets.inactive.weak_bg_fill = MODRINTH_FIELD;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
            visuals.widgets.inactive.rounding = Rounding::same(12.0);
            visuals.widgets.hovered.bg_fill = MODRINTH_FIELD;
            visuals.widgets.hovered.weak_bg_fill = MODRINTH_FIELD;
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
            visuals.widgets.hovered.rounding = Rounding::same(12.0);
            visuals.widgets.active.bg_fill = MODRINTH_FIELD;
            visuals.widgets.active.weak_bg_fill = MODRINTH_FIELD;
            visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, MODRINTH_GREEN);
            visuals.widgets.active.rounding = Rounding::same(12.0);
            visuals.override_text_color = Some(MODRINTH_TEXT_CONTRAST);
            visuals.selection.bg_fill = with_alpha(MODRINTH_GREEN, 90);
            ui.add(edit)
        })
        .inner;
    let rect = Rect::from_min_size(
        pos2(field_left, resp.rect.center().y - height / 2.0),
        vec2(width, height),
    );
    if let Some(icon) = icon {
        icons::draw(
            ui.painter(),
            icon,
            Rect::from_center_size(pos2(rect.left() + 22.0, rect.center().y), vec2(18.0, 18.0)),
            MODRINTH_TEXT_SECONDARY,
            1.5,
        );
    }
    if resp.has_focus() {
        ui.painter().rect_stroke(
            rect,
            Rounding::same(12.0),
            Stroke::new(1.0_f32, MODRINTH_GREEN),
        );
    }
    if resp.hovered() || resp.has_focus() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
    }
    resp
}

#[derive(Debug, Clone, Copy)]
enum ContextChoice {
    Play,
    Edit,
    Duplicate,
    Manage,
    Shortcut,
    Folder,
    Delete,
}

/// One row of the reference card context menu (36px, icon + 14px label).
fn context_menu_item(ui: &mut Ui, icon: Icon, label: &str, danger: bool) -> bool {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    let resp = ui.interact(
        rect,
        ui.id().with(("modrinth_context_item", label)),
        Sense::click(),
    );
    if resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(vec2(1.0, 1.0)),
            Rounding::same(8.0),
            MODRINTH_SURFACE_HIGH,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let color = if danger {
        Color32::from_rgb(0xE8, 0x20, 0x0D)
    } else {
        MODRINTH_TEXT_CONTRAST
    };
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(17.0, 17.0)),
        if danger { color } else { MODRINTH_TEXT },
        1.6,
    );
    ui.painter().text(
        pos2(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        color,
    );
    resp.clicked()
}

/// Reference context-menu divider (1px surface-5, 4px vertical air).
fn context_menu_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(rect.left() + 6.0, rect.center().y),
            pos2(rect.right() - 6.0, rect.center().y),
        ],
        Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
    );
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

    /// Memoized filter+sort of the instance list (per-frame `to_lowercase`
    /// and `sort` avoided: recomputed only when query/tag/sort/filters or the
    /// instance revision changes).
    fn filtered_indices(&mut self, instances: &[Instance]) -> Vec<usize> {
        let rev = instances_rev(instances);
        let cache_valid = match &self.filter_key {
            Some(key) => {
                key.query == self.search_query
                    && key.tag == self.active_tag
                    && key.sort == self.sort_mode
                    && key.loaders == self.modrinth_filters.loaders
                    && key.versions == self.modrinth_filters.versions
                    && key.rev == rev
            }
            None => false,
        };
        if !cache_valid {
            let query = self.search_query.trim().to_lowercase();
            let mut indices: Vec<usize> = instances
                .iter()
                .enumerate()
                .filter(|(_, inst)| {
                    matches_filter(
                        inst,
                        &query,
                        self.active_tag.as_deref(),
                        &self.modrinth_filters,
                    )
                })
                .map(|(idx, _)| idx)
                .collect();
            let sort_mode = self.sort_mode;
            indices.sort_by(|&a, &b| compare_instances(&instances[a], &instances[b], sort_mode));
            self.filter_cache = indices;
            self.filter_key = Some(InstanceFilterKey {
                query: self.search_query.clone(),
                tag: self.active_tag.clone(),
                sort: self.sort_mode,
                loaders: self.modrinth_filters.loaders.clone(),
                versions: self.modrinth_filters.versions.clone(),
                rev,
            });
        }
        // Small index vec clone per frame (tens of ids max); the expensive
        // filter+sort+lowercase above runs only on miss.
        self.filter_cache.clone()
    }

    /// Modrinth App shell: the library page — "Library" heading, search and
    /// New buttons, sort/group/filter bar, big cards with a hover Play
    /// button. Layout mirrors `apps/app-frontend` (`library` components);
    /// colors are the reference's dark-theme CSS tokens.
    fn show_modrinth_library(
        &mut self,
        ui: &mut Ui,
        instances: &mut [Instance],
        selected_instance: &mut Option<Uuid>,
        ctx: &InstancesContext,
        lang: Language,
        action: &mut InstanceAction,
    ) {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(lang.inst_library_title())
                .font(egui::FontId::proportional(24.0))
                .strong()
                .color(MODRINTH_TEXT_CONTRAST),
        );
        ui.add_space(12.0);

        // Toolbar row 1: search field, "New group", brand "New instance".
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            let group_w = m_button_width(ui, Some(Icon::Plus), lang.inst_btn_new_group(), 36.0);
            let new_w = m_button_width(ui, Some(Icon::Plus), lang.inst_btn_new_instance(), 36.0);
            let field_w = (ui.available_width() - group_w - new_w - 16.0).max(160.0);
            self.modrinth_search_field(ui, field_w, lang);
            if m_button(ui, Some(Icon::Plus), lang.inst_btn_new_group(), false, 36.0) {
                self.show_new_group_modal = true;
                self.new_group_name.clear();
                self.new_group_search.clear();
                self.new_group_selected.clear();
                self.modrinth_menu = None;
                ui.ctx().memory_mut(|mem| {
                    mem.request_focus(egui::Id::new("modrinth_new_group_name"));
                });
            }
            if m_button(
                ui,
                Some(Icon::Plus),
                lang.inst_btn_new_instance(),
                true,
                36.0,
            ) {
                self.new_instance_ram = ctx.suggested_ram_mb;
                self.show_create_modal = true;
            }
        });

        // Toolbar row 2: sort and group comboboxes, divider, filter.
        ui.add_space(8.0);
        let sort_id = ui.id().with("modrinth_sort_popup");
        let group_id = ui.id().with("modrinth_group_popup");
        let filter_id = ui.id().with("modrinth_filter_popup");
        let mut sort_resp = None;
        let mut group_resp = None;
        let mut filter_resp = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            let sort = m_combo(ui, Icon::Sort, self.sort_mode.modrinth_label(lang));
            if sort.clicked() {
                toggle_library_popup(ui, sort_id);
            }
            sort_resp = Some(sort);

            let group = m_combo(ui, Icon::Grid, self.group_by.label(lang));
            if group.clicked() {
                toggle_library_popup(ui, group_id);
            }
            group_resp = Some(group);

            ui.add_space(8.0);
            let (divider, _) = ui.allocate_exact_size(vec2(1.0, 24.0), Sense::hover());
            ui.painter()
                .rect_filled(divider, Rounding::ZERO, MODRINTH_SURFACE_HIGH);
            ui.add_space(8.0);
            let filter = m_combo(ui, Icon::Filter, lang.inst_filter_by());
            if filter.clicked() {
                toggle_library_popup(ui, filter_id);
            }
            filter_resp = Some(filter);
        });

        if let Some(sort) = sort_resp {
            modrinth_popup(ui, sort_id, &sort, |ui| {
                ui.set_min_width(204.0);
                for mode in InstanceSort::MODRINTH_ALL {
                    if modrinth_menu_option(ui, mode.modrinth_label(lang), self.sort_mode == mode) {
                        self.sort_mode = mode;
                        self.filter_key = None;
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        }
        if let Some(group) = group_resp {
            modrinth_popup(ui, group_id, &group, |ui| {
                ui.set_min_width(204.0);
                for mode in LibraryGroupBy::MODRINTH_ALL {
                    if modrinth_menu_option(ui, mode.label(lang), self.group_by == mode) {
                        self.group_by = mode;
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        }
        if let Some(filter) = filter_resp {
            modrinth_popup(ui, filter_id, &filter, |ui| {
                self.modrinth_filter_menu(ui, instances, lang);
            });
        }

        ui.add_space(12.0);

        let filtered = self.filtered_indices(instances);
        let sections = if instances.is_empty() {
            Vec::new()
        } else {
            modrinth_sections(instances, &filtered, self.group_by, &ctx.group_names, lang)
        };
        let show_headers = sections.len() > 1;
        let searching = !self.search_query.trim().is_empty();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if instances.is_empty() {
                    ui.add_space(28.0);
                    let card_w = ui.available_width().min(580.0);
                    ui.vertical_centered(|ui| {
                        let (card, _) = ui.allocate_exact_size(vec2(card_w, 242.0), Sense::hover());
                        ui.painter()
                            .rect_filled(card, Rounding::same(18.0), MODRINTH_SURFACE);
                        ui.painter().rect_stroke(
                            card,
                            Rounding::same(18.0),
                            Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH),
                        );
                        let icon = Rect::from_center_size(
                            pos2(card.center().x, card.top() + 56.0),
                            vec2(64.0, 64.0),
                        );
                        ui.painter().rect_filled(
                            icon,
                            Rounding::same(16.0),
                            with_alpha(MODRINTH_GREEN, 22),
                        );
                        ui.painter().rect_stroke(
                            icon,
                            Rounding::same(16.0),
                            Stroke::new(1.0_f32, with_alpha(MODRINTH_GREEN, 90)),
                        );
                        icons::draw_centered(
                            ui.painter(),
                            Icon::Cube,
                            icon.center(),
                            28.0,
                            MODRINTH_GREEN,
                        );
                        ui.painter().text(
                            pos2(card.center().x, card.top() + 116.0),
                            Align2::CENTER_CENTER,
                            lang.inst_empty_title(),
                            egui::FontId::new(18.0, egui::FontFamily::Name("bold".into())),
                            MODRINTH_TEXT_CONTRAST,
                        );
                        let hint = ui.painter().layout(
                            lang.inst_empty_hint().to_owned(),
                            egui::FontId::proportional(12.5),
                            MODRINTH_TEXT_SECONDARY,
                            (card_w - 80.0).max(160.0),
                        );
                        ui.painter().galley(
                            pos2(card.center().x - hint.size().x / 2.0, card.top() + 140.0),
                            hint,
                            MODRINTH_TEXT_SECONDARY,
                        );
                        let button = Rect::from_center_size(
                            pos2(card.center().x, card.bottom() - 42.0),
                            vec2(190.0, 38.0),
                        );
                        let resp = ui.interact(
                            button,
                            ui.id().with("modrinth_empty_create"),
                            Sense::click(),
                        );
                        let hover = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
                        ui.painter().rect_filled(
                            button,
                            Rounding::same(10.0),
                            lerp_color(
                                MODRINTH_GREEN,
                                crate::theme::MODRINTH_GREEN_LIGHT,
                                hover * 0.55,
                            ),
                        );
                        icons::draw(
                            ui.painter(),
                            Icon::Plus,
                            Rect::from_center_size(
                                pos2(button.left() + 38.0, button.center().y),
                                vec2(16.0, 16.0),
                            ),
                            MODRINTH_BG,
                            1.6,
                        );
                        ui.painter().text(
                            pos2(button.left() + 54.0, button.center().y),
                            Align2::LEFT_CENTER,
                            lang.inst_btn_new_instance(),
                            egui::FontId::proportional(12.5),
                            MODRINTH_BG,
                        );
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            self.new_instance_ram = ctx.suggested_ram_mb;
                            self.show_create_modal = true;
                        }
                    });
                    return;
                }
                if filtered.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(lang.inst_library_no_results())
                                .font(egui::FontId::proportional(14.0))
                                .color(MODRINTH_TEXT_SECONDARY),
                        );
                    });
                    return;
                }

                for (section_idx, section) in sections.iter().enumerate() {
                    if section_idx > 0 {
                        ui.add_space(12.0);
                    }
                    if show_headers {
                        let collapsed =
                            !searching && self.modrinth_collapsed.contains(&section.key);
                        if modrinth_group_header(
                            ui,
                            &section.title,
                            section.indices.len(),
                            collapsed,
                        ) {
                            if collapsed {
                                self.modrinth_collapsed.remove(&section.key);
                            } else {
                                self.modrinth_collapsed.insert(section.key.clone());
                            }
                        }
                        ui.add_space(10.0);
                    }
                    let collapsed = show_headers
                        && !searching
                        && self.modrinth_collapsed.contains(&section.key);
                    if collapsed {
                        continue;
                    }
                    self.modrinth_card_grid(
                        ui,
                        instances,
                        &section.indices,
                        selected_instance,
                        ctx.instances_dir,
                        action,
                    );
                }
            });

        // Right-click menu is an overlay so it is not clipped by the scroll area.
        self.modrinth_context_menu(ui.ctx(), instances, ctx.instances_dir, lang, action);
        // New group modal lives above the library content.
        if let Some(next) =
            self.modrinth_new_group_modal(ui.ctx(), instances, ctx.instances_dir, lang)
        {
            *action = next;
        }
    }

    /// Grid of cards for one library section (reference `InstanceGroup`).
    fn modrinth_card_grid(
        &mut self,
        ui: &mut Ui,
        instances: &[Instance],
        indices: &[usize],
        selected_instance: &mut Option<Uuid>,
        instances_dir: &Path,
        action: &mut InstanceAction,
    ) {
        if indices.is_empty() {
            return;
        }
        // Reference grid: cards of at least 160px, 12px gaps (`gap-3`);
        // card height = width + 54px (two 12px paddings, the square icon,
        // the name line and the loader line).
        let avail = ui.available_width();
        let cols = (((avail + 12.0) / 172.0).floor() as usize).max(1);
        let card_w = (avail - 12.0 * cols.saturating_sub(1) as f32) / cols as f32;
        let card_h = card_w + 54.0;
        for (row_idx, chunk) in indices.chunks(cols).enumerate() {
            if row_idx > 0 {
                ui.add_space(12.0);
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                for &idx in chunk {
                    let inst = &instances[idx];
                    self.modrinth_card(
                        ui,
                        inst,
                        vec2(card_w, card_h),
                        selected_instance,
                        instances_dir,
                        action,
                    );
                }
            });
        }
    }

    /// Filter popup contents: loader/version checkboxes and a clear action.
    fn modrinth_filter_menu(&mut self, ui: &mut Ui, instances: &[Instance], lang: Language) {
        ui.set_min_width(232.0);
        ui.set_max_width(280.0);
        let mut loaders: Vec<LoaderType> = instances.iter().map(|inst| inst.loader).collect();
        loaders.sort_by_key(|l| l.as_str().to_ascii_lowercase());
        loaders.dedup();
        let mut versions: Vec<String> = instances
            .iter()
            .map(|inst| inst.game_version.clone())
            .collect();
        versions.sort_by(|a, b| natural_version_cmp(b, a));
        versions.dedup();

        let mut changed = false;
        ScrollArea::vertical()
            .max_height(360.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                if !loaders.is_empty() {
                    ui.label(
                        egui::RichText::new(lang.inst_word_loader())
                            .font(egui::FontId::proportional(13.0))
                            .strong()
                            .color(MODRINTH_TEXT_SECONDARY),
                    );
                    ui.add_space(3.0);
                    for loader in loaders {
                        let checked = self.modrinth_filters.loaders.contains(&loader);
                        if modrinth_check_option(
                            ui,
                            loader.as_str(),
                            checked,
                            egui::Id::new(("modrinth_filter_loader", loader.as_str())),
                        ) {
                            self.modrinth_filters.toggle_loader(loader);
                            changed = true;
                        }
                    }
                }
                if !versions.is_empty() {
                    if !self.modrinth_filters.loaders.is_empty() {
                        ui.add_space(8.0);
                    }
                    ui.label(
                        egui::RichText::new(lang.inst_word_game_version())
                            .font(egui::FontId::proportional(13.0))
                            .strong()
                            .color(MODRINTH_TEXT_SECONDARY),
                    );
                    ui.add_space(3.0);
                    for version in versions {
                        let checked = self.modrinth_filters.versions.contains(&version);
                        if modrinth_check_option(
                            ui,
                            &version,
                            checked,
                            egui::Id::new(("modrinth_filter_version", version.as_str())),
                        ) {
                            self.modrinth_filters.toggle_version(&version);
                            changed = true;
                        }
                    }
                }
            });

        let active = !self.modrinth_filters.is_empty();
        ui.add_space(6.0);
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, Rounding::ZERO, MODRINTH_SURFACE_HIGH);
        ui.add_space(3.0);
        if modrinth_menu_option_enabled(ui, lang.inst_filter_clear(), false, active) && active {
            self.modrinth_filters = ModrinthFilters::default();
            changed = true;
        }
        if changed {
            self.filter_key = None;
        }
    }

    /// One Modrinth library card: square icon, name, loader + version and
    /// the hover Play button (reference `instance-card-view`).
    fn modrinth_card(
        &mut self,
        ui: &mut Ui,
        inst: &Instance,
        size: egui::Vec2,
        selected_instance: &mut Option<Uuid>,
        instances_dir: &Path,
        action: &mut InstanceAction,
    ) {
        let inst_id = inst.id;
        let is_selected = selected_instance.as_ref() == Some(&inst_id);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        let hover_t = ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
        let sel_t = ui.ctx().animate_bool(resp.id.with("sel"), is_selected);

        // `hover:brightness-110` (selected cards lighten as well).
        let lift = hover_t.max(sel_t) * 0.1;
        let radius = Rounding::same(20.0);
        ui.painter()
            .rect_filled(rect, radius, m_brightness(MODRINTH_SURFACE, 1.0 + lift));
        let border = if is_selected {
            // `color-mix(in srgb, text-primary 40%, transparent)`.
            with_alpha(MODRINTH_TEXT_CONTRAST, 102)
        } else {
            MODRINTH_SURFACE_HOVER
        };
        ui.painter()
            .rect_stroke(rect, radius, Stroke::new(1.0_f32, border));

        // Square icon (reference: `aspect-square rounded-2xl`).
        let icon_rect = Rect::from_min_size(
            pos2(rect.left() + 12.0, rect.top() + 12.0),
            vec2(size.x - 24.0, size.x - 24.0),
        );
        match self.instance_icon_texture(ui.ctx(), inst_id, &inst.get_game_dir(instances_dir)) {
            Some(tex) => {
                ui.painter().image(
                    tex.id(),
                    icon_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                if let Some(preset) = inst.icon.as_deref().and_then(icons::instance_preset) {
                    ui.painter().rect_filled(
                        icon_rect,
                        Rounding::same(16.0),
                        MODRINTH_SURFACE_HOVER,
                    );
                    icons::draw_centered(
                        ui.painter(),
                        preset,
                        icon_rect.center(),
                        icon_rect.width() * 0.34,
                        MODRINTH_TEXT,
                    );
                } else if let Some(tex) =
                    crate::widgets::modrinth_assets::loader_texture(ui.ctx(), inst.loader)
                {
                    // Reference default artwork (bundled PNG assets).
                    ui.painter().image(
                        tex.id(),
                        icon_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter().rect_filled(
                        icon_rect,
                        Rounding::same(16.0),
                        MODRINTH_SURFACE_HOVER,
                    );
                    icons::draw_centered(
                        ui.painter(),
                        icons::loader_badge(inst.loader),
                        icon_rect.center(),
                        icon_rect.width() * 0.34,
                        MODRINTH_TEXT,
                    );
                }
            }
        }

        // Name + "<Loader> <version>" lines (reference: text-base strong
        // over text-sm, loader capitalized).
        let text_x = rect.left() + 12.0;
        let max_chars = (((size.x - 40.0) / 9.0).max(6.0)) as usize;
        ui.painter().text(
            pos2(text_x, icon_rect.bottom() + 12.0),
            Align2::LEFT_TOP,
            truncate_name(&inst.name, max_chars),
            egui::FontId::proportional(16.0),
            MODRINTH_TEXT_CONTRAST,
        );
        ui.painter().text(
            pos2(text_x, icon_rect.bottom() + 36.0),
            Align2::LEFT_TOP,
            format!("{} {}", inst.loader.as_str(), inst.game_version),
            egui::FontId::proportional(14.0),
            MODRINTH_TEXT,
        );

        // Hover Play button: 48px green circle, 75% -> 100% + fade in.
        if hover_t > 0.01 {
            let growth = 18.0 + 6.0 * hover_t;
            let center = pos2(icon_rect.right() - 30.0, icon_rect.bottom() - 30.0);
            let play = ui.interact(
                Rect::from_center_size(center, vec2(48.0, 48.0)),
                resp.id.with("play"),
                Sense::click(),
            );
            let alpha = (hover_t * 255.0) as u8;
            let green = if play.hovered() {
                m_brightness(MODRINTH_GREEN, 1.1)
            } else {
                MODRINTH_GREEN
            };
            ui.painter()
                .circle_filled(center, growth, with_alpha(green, alpha));
            // The reference colors the glyph with `--color-accent-contrast`.
            icons::draw(
                ui.painter(),
                Icon::Play,
                Rect::from_center_size(
                    pos2(center.x + 1.0, center.y),
                    vec2(growth * 0.84, growth * 0.84),
                ),
                with_alpha(Color32::BLACK, alpha),
                1.6,
            );
            if play.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if play.clicked() {
                *action = InstanceAction::Launch(inst_id);
            }
        }

        // Selection disc in the top-right corner (reference overlay).
        if hover_t > 0.01 || is_selected {
            let center = pos2(rect.right() - 33.0, rect.top() + 34.0);
            let alpha = if is_selected {
                255
            } else {
                (hover_t * 255.0) as u8
            };
            let sel = ui.interact(
                Rect::from_center_size(center, vec2(44.0, 44.0)),
                resp.id.with("select"),
                Sense::click(),
            );
            if is_selected {
                ui.painter()
                    .circle_filled(center, 12.0, with_alpha(MODRINTH_TEXT_CONTRAST, alpha));
                icons::draw(
                    ui.painter(),
                    Icon::Check,
                    Rect::from_center_size(center, vec2(13.0, 13.0)),
                    with_alpha(Color32::BLACK, alpha),
                    2.2,
                );
            } else {
                ui.painter().circle_stroke(
                    center,
                    12.0,
                    Stroke::new(2.0_f32, with_alpha(MODRINTH_TEXT, alpha)),
                );
            }
            if sel.clicked() {
                *selected_instance = Some(inst_id);
                *action = InstanceAction::Selected(inst_id);
            }
        }

        if resp.clicked() {
            *selected_instance = Some(inst_id);
            *action = InstanceAction::Selected(inst_id);
        }
        if resp.secondary_clicked() {
            let pos = ui
                .ctx()
                .pointer_interact_pos()
                .unwrap_or_else(|| resp.rect.center());
            self.modrinth_menu = Some((inst_id, pos));
            self.modrinth_menu_fresh = true;
            ui.memory_mut(|mem| mem.close_popup());
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if inst.name.chars().count() > max_chars {
            // Cards are as narrow as 160 px in the reference grid; keep the
            // full instance name reachable when the painted line ellipsizes.
            resp.on_hover_text(inst.name.as_str());
        }
    }

    /// Reference `ContextMenu` for an instance card: Play, Edit, Duplicate,
    /// Manage, shortcut, folder, Delete — same order and icon placement.
    fn modrinth_context_menu(
        &mut self,
        ctx: &egui::Context,
        instances: &[Instance],
        instances_dir: &Path,
        lang: Language,
        action: &mut InstanceAction,
    ) {
        let Some((inst_id, pointer)) = self.modrinth_menu else {
            return;
        };
        let Some(inst) = instances.iter().find(|inst| inst.id == inst_id) else {
            self.modrinth_menu = None;
            return;
        };

        let screen = ctx.screen_rect();
        let width = 248.0;
        let height = 290.0;
        let pos = pos2(
            pointer
                .x
                .min(screen.right() - width - 8.0)
                .max(screen.left() + 8.0),
            pointer
                .y
                .min(screen.bottom() - height - 8.0)
                .max(screen.top() + 8.0),
        );

        let mut choice: Option<ContextChoice> = None;
        let mut close = false;
        egui::Area::new(egui::Id::new("modrinth_context_backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                let _ = rect;
                if resp.clicked()
                    || (resp.secondary_clicked() && !self.modrinth_menu_fresh)
                    || ui.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    close = true;
                }
            });

        egui::Area::new(egui::Id::new("modrinth_context_menu"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(MODRINTH_SURFACE_HOVER)
                    .stroke(Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH))
                    .rounding(Rounding::same(14.0))
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(110),
                    })
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.set_width(236.0);
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        if context_menu_item(ui, Icon::Play, lang.inst_ctx_play(), false) {
                            choice = Some(ContextChoice::Play);
                        }
                        context_menu_divider(ui);
                        if context_menu_item(ui, Icon::Wrench, lang.inst_ctx_edit(), false) {
                            choice = Some(ContextChoice::Edit);
                        }
                        if context_menu_item(ui, Icon::Clipboard, lang.inst_ctx_duplicate(), false)
                        {
                            choice = Some(ContextChoice::Duplicate);
                        }
                        if context_menu_item(ui, Icon::List, lang.inst_ctx_manage(), false) {
                            choice = Some(ContextChoice::Manage);
                        }
                        if context_menu_item(ui, Icon::Link, lang.inst_ctx_shortcut(), false) {
                            choice = Some(ContextChoice::Shortcut);
                        }
                        if context_menu_item(ui, Icon::Folder, lang.inst_ctx_folder(), false) {
                            choice = Some(ContextChoice::Folder);
                        }
                        context_menu_divider(ui);
                        if context_menu_item(ui, Icon::Trash, lang.inst_ctx_delete(), true) {
                            choice = Some(ContextChoice::Delete);
                        }
                    });
            });

        self.modrinth_menu_fresh = false;
        if close {
            self.modrinth_menu = None;
            return;
        }
        match choice {
            Some(ContextChoice::Play) => {
                *action = InstanceAction::Launch(inst_id);
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Edit) => {
                self.open_edit_modal(inst);
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Duplicate) => {
                self.clone_instance_id = Some(inst_id);
                self.clone_with_saves = true;
                self.show_clone_modal = true;
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Manage) => {
                self.open_manage(inst_id);
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Shortcut) => {
                self.shortcut_request = Some(inst_id);
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Folder) => {
                let dir = inst.get_game_dir(instances_dir);
                let _ = std::fs::create_dir_all(&dir);
                let _ = open::that(dir);
                self.modrinth_menu = None;
            }
            Some(ContextChoice::Delete) => {
                *action = InstanceAction::Trashed(inst_id);
                self.modrinth_menu = None;
            }
            None => {}
        }
    }

    /// Modrinth App "Create instance" dialog. The reference uses a 600-ish
    /// rounded modal on surface-3 with a 24px title, labeled fields, loader
    /// chips and a brand-green primary action. The AMC dialog keeps the
    /// template shortcuts and RAM control but presents them in that
    /// composition, so Modrinth no longer falls back to a generic egui window.
    fn modrinth_create_modal(
        &mut self,
        ctx: &egui::Context,
        lang: Language,
        selected_instance: &mut Option<Uuid>,
    ) -> Option<InstanceAction> {
        if !self.show_create_modal {
            self.create_modal_seen = false;
            return None;
        }

        let screen = ctx.screen_rect();
        let modal_w = 600.0_f32.min(screen.width() - 32.0);
        let content_w = modal_w - 36.0;
        // The measured body is ~450px; 540 keeps the dialog below the custom
        // titlebar and fully inside the 960x600 minimum.
        let modal_h = 540.0_f32.min(screen.height() - 56.0);
        let modal_pos = pos2(
            screen.center().x - modal_w / 2.0,
            (screen.center().y - modal_h / 2.0).max(screen.top() + 52.0),
        );

        let mut close = false;
        let mut create = false;

        egui::Area::new(egui::Id::new("modrinth_create_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(150));
                if resp.clicked() {
                    close = true;
                }
            });

        egui::Area::new(egui::Id::new("modrinth_create_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(MODRINTH_SURFACE)
                    .stroke(Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH))
                    .rounding(Rounding::same(16.0))
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 12.0),
                        blur: 32.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(150),
                    })
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.set_width(content_w);
                        ui.spacing_mut().item_spacing.y = 6.0;

                        // Header: title + quiet close button.
                        let (header, _) =
                            ui.allocate_exact_size(vec2(content_w, 30.0), Sense::hover());
                        ui.painter().text(
                            pos2(header.left(), header.center().y),
                            Align2::LEFT_CENTER,
                            lang.inst_modal_create_title(),
                            egui::FontId::new(23.0, egui::FontFamily::Name("bold".into())),
                            MODRINTH_TEXT_CONTRAST,
                        );
                        let close_rect = Rect::from_center_size(
                            pos2(header.right() - 14.0, header.center().y),
                            vec2(32.0, 32.0),
                        );
                        let close_resp = ui.interact(
                            close_rect,
                            egui::Id::new("modrinth_create_close"),
                            Sense::click(),
                        );
                        if close_resp.hovered() {
                            ui.painter().rect_filled(
                                close_rect,
                                Rounding::same(8.0),
                                with_alpha(MODRINTH_TEXT_CONTRAST, 16),
                            );
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        icons::draw(
                            ui.painter(),
                            Icon::Close,
                            close_rect.shrink(9.0),
                            MODRINTH_TEXT,
                            1.7,
                        );
                        if close_resp.clicked() {
                            close = true;
                        }

                        ui.add_space(8.0);

                        // Name.
                        ui.label(
                            egui::RichText::new(lang.inst_modal_name())
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(MODRINTH_TEXT_CONTRAST),
                        );
                        ui.add_space(4.0);
                        let name_resp = modrinth_input(
                            ui,
                            &mut self.new_instance_name,
                            egui::Id::new("modrinth_create_name"),
                            content_w,
                            38.0,
                            "My Instance",
                            None,
                        );
                        if !self.create_modal_seen {
                            ui.memory_mut(|mem| {
                                mem.request_focus(egui::Id::new("modrinth_create_name"))
                            });
                            self.create_modal_seen = true;
                        }

                        ui.add_space(8.0);

                        // Game version.
                        ui.label(
                            egui::RichText::new(lang.inst_modal_version())
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(MODRINTH_TEXT_CONTRAST),
                        );
                        ui.add_space(4.0);
                        let version_resp = modrinth_input(
                            ui,
                            &mut self.new_instance_version,
                            egui::Id::new("modrinth_create_version"),
                            content_w,
                            38.0,
                            "1.20.1",
                            None,
                        );
                        if self.new_instance_loader == LoaderType::Forge
                            && amc_minecraft::loaders::forge::is_legacy_unsafe_line(
                                &self.new_instance_version,
                            )
                        {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(lang.inst_warn_legacy_forge())
                                    .font(egui::FontId::proportional(12.0))
                                    .color(WARNING),
                            );
                        }

                        ui.add_space(8.0);

                        // Loader chips.
                        ui.label(
                            egui::RichText::new(lang.inst_modal_loader())
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(MODRINTH_TEXT_CONTRAST),
                        );
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                            for loader in [
                                LoaderType::Vanilla,
                                LoaderType::Fabric,
                                LoaderType::Quilt,
                                LoaderType::Forge,
                                LoaderType::NeoForge,
                                LoaderType::OptiFine,
                            ] {
                                if modrinth_pick_chip(
                                    ui,
                                    loader.as_str(),
                                    self.new_instance_loader == loader,
                                ) {
                                    self.new_instance_loader = loader;
                                }
                            }
                        });

                        ui.add_space(8.0);

                        // Memory.
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.inst_modal_ram())
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(MODRINTH_TEXT_CONTRAST),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} / 16384 MB",
                                            self.new_instance_ram
                                        ))
                                        .font(egui::FontId::proportional(12.0))
                                        .color(MODRINTH_TEXT_SECONDARY),
                                    );
                                },
                            );
                        });
                        ui.add_space(2.0);
                        modrinth_ram_slider(ui, &mut self.new_instance_ram, content_w);

                        ui.add_space(8.0);

                        // Template shortcuts (AMC feature, reference chip row).
                        let tmpl_desc = InstanceTemplate::matching(
                            self.new_instance_loader,
                            self.new_instance_ram,
                        )
                        .map(|t| lang.tmpl_desc(t.id))
                        .unwrap_or_else(|| lang.tmpl_custom_note());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.tmpl_title())
                                    .font(egui::FontId::proportional(14.0))
                                    .strong()
                                    .color(MODRINTH_TEXT_CONTRAST),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(truncate_name(tmpl_desc, 56))
                                            .font(egui::FontId::proportional(11.5))
                                            .color(MODRINTH_TEXT_SECONDARY),
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                            for tmpl in InstanceTemplate::ALL {
                                let active = InstanceTemplate::matching(
                                    self.new_instance_loader,
                                    self.new_instance_ram,
                                )
                                .map(|t| t.id)
                                    == Some(tmpl.id);
                                if modrinth_pick_chip(ui, lang.tmpl_name(tmpl.id), active) {
                                    self.apply_template(tmpl, lang);
                                }
                            }
                        });

                        ui.add_space(12.0);
                        let (divider, _) =
                            ui.allocate_exact_size(vec2(content_w, 1.0), Sense::hover());
                        ui.painter().rect_filled(
                            divider.expand2(vec2(20.0, 0.0)),
                            Rounding::ZERO,
                            MODRINTH_SURFACE_HIGH,
                        );
                        ui.add_space(12.0);

                        let can_create = !self.new_instance_name.trim().is_empty()
                            && !self.new_instance_version.trim().is_empty();
                        let enter = (name_resp.has_focus()
                            || version_resp.has_focus()
                            || name_resp.lost_focus()
                            || version_resp.lost_focus())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if m_button_enabled(
                                ui,
                                Some(Icon::Plus),
                                lang.inst_modal_btn_create(),
                                true,
                                38.0,
                                can_create,
                            ) || (can_create && enter)
                            {
                                create = true;
                            }
                            ui.add_space(8.0);
                            if m_button(
                                ui,
                                Some(Icon::Close),
                                lang.inst_modal_btn_cancel(),
                                false,
                                38.0,
                            ) {
                                close = true;
                            }
                        });
                    });
            });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }

        if create {
            let mut inst = Instance::new(
                &self.new_instance_name,
                &self.new_instance_version,
                self.new_instance_loader,
            );
            inst.ram_mb = Some(self.new_instance_ram);
            *selected_instance = Some(inst.id);
            self.show_create_modal = false;
            self.create_modal_seen = false;
            return Some(InstanceAction::Created(inst));
        }
        if close {
            self.show_create_modal = false;
            self.create_modal_seen = false;
        }
        None
    }

    /// Reference `new-group-modal.vue`: 560px modal, name field, 400px
    /// scrollable instance list with Add/Added, Cancel/Create group.
    fn modrinth_new_group_modal(
        &mut self,
        ctx: &egui::Context,
        instances: &mut [Instance],
        instances_dir: &Path,
        lang: Language,
    ) -> Option<InstanceAction> {
        if !self.show_new_group_modal {
            return None;
        }

        let screen = ctx.screen_rect();
        let modal_w = 560.0;
        let content_w = modal_w - 48.0;
        let body_h = (screen.height() - 300.0).clamp(240.0, 400.0);
        let modal_h = body_h + 282.0;
        let modal_pos = pos2(
            screen.center().x - modal_w / 2.0,
            (screen.center().y - modal_h / 2.0).max(screen.top() + 8.0),
        );

        let mut close = false;
        let mut create = false;
        let mut enter = false;

        egui::Area::new(egui::Id::new("modrinth_group_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(145));
            });

        egui::Area::new(egui::Id::new("modrinth_new_group_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(MODRINTH_SURFACE)
                    .stroke(Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH))
                    .rounding(Rounding::same(16.0))
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 12.0),
                        blur: 32.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(150),
                    })
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_width(content_w);

                        // Title + close.
                        let (header, _) =
                            ui.allocate_exact_size(vec2(content_w, 32.0), Sense::hover());
                        ui.painter().text(
                            pos2(header.left(), header.center().y),
                            Align2::LEFT_CENTER,
                            lang.inst_group_create(),
                            egui::FontId::proportional(24.0),
                            MODRINTH_TEXT_CONTRAST,
                        );
                        let close_rect = Rect::from_center_size(
                            pos2(header.right() - 14.0, header.center().y),
                            vec2(32.0, 32.0),
                        );
                        let close_resp = ui.interact(
                            close_rect,
                            egui::Id::new("modrinth_group_close"),
                            Sense::click(),
                        );
                        if close_resp.hovered() {
                            ui.painter().rect_filled(
                                close_rect,
                                Rounding::same(8.0),
                                with_alpha(MODRINTH_TEXT_CONTRAST, 16),
                            );
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        icons::draw(
                            ui.painter(),
                            Icon::Close,
                            close_rect.shrink(9.0),
                            MODRINTH_TEXT,
                            1.7,
                        );
                        if close_resp.clicked() {
                            close = true;
                        }

                        ui.add_space(18.0);
                        ui.label(
                            egui::RichText::new(lang.inst_group_name())
                                .font(egui::FontId::proportional(15.0))
                                .strong()
                                .color(MODRINTH_TEXT_CONTRAST),
                        );
                        ui.add_space(8.0);
                        let name_resp = modrinth_input(
                            ui,
                            &mut self.new_group_name,
                            egui::Id::new("modrinth_new_group_name"),
                            content_w,
                            40.0,
                            lang.inst_group_name_hint(),
                            None,
                        );
                        enter = (name_resp.has_focus() || name_resp.lost_focus())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        ui.add_space(20.0);
                        let (divider, _) =
                            ui.allocate_exact_size(vec2(content_w, 1.0), Sense::hover());
                        ui.painter().rect_filled(
                            divider.expand2(vec2(24.0, 0.0)),
                            Rounding::ZERO,
                            MODRINTH_SURFACE_HIGH,
                        );

                        // Reference list body: `h-[400px] bg-surface-2` with
                        // 16px vertical padding, a search field and 4px rows.
                        let (body_rect, _) =
                            ui.allocate_exact_size(vec2(content_w, body_h), Sense::hover());
                        ui.painter().rect_filled(
                            body_rect.expand2(vec2(24.0, 0.0)),
                            Rounding::ZERO,
                            MODRINTH_SURFACE,
                        );

                        let add_w = {
                            let font = egui::FontId::proportional(14.0);
                            let add = ui.painter().layout_no_wrap(
                                lang.inst_group_add().to_owned(),
                                font.clone(),
                                Color32::WHITE,
                            );
                            let added = ui.painter().layout_no_wrap(
                                lang.inst_group_added().to_owned(),
                                font,
                                Color32::WHITE,
                            );
                            (add.size().x.max(16.0 + 6.0 + added.size().x) + 24.0).max(88.0)
                        };

                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(body_rect)
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                            |ui| {
                                ui.add_space(16.0);
                                ui.horizontal(|ui| {
                                    modrinth_input(
                                        ui,
                                        &mut self.new_group_search,
                                        egui::Id::new("modrinth_new_group_search"),
                                        content_w,
                                        40.0,
                                        lang.inst_group_search(),
                                        Some(Icon::Search),
                                    );
                                });
                                ui.add_space(12.0);

                                let mut rows: Vec<usize> = instances
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, inst)| {
                                        let query = self.new_group_search.trim().to_lowercase();
                                        query.is_empty()
                                            || inst.name.to_lowercase().contains(&query)
                                    })
                                    .map(|(idx, _)| idx)
                                    .collect();
                                rows.sort_by_key(|&idx| {
                                    (
                                        instances[idx]
                                            .group
                                            .as_deref()
                                            .is_some_and(|g| !g.is_empty()),
                                        instances[idx].name.to_lowercase(),
                                    )
                                });

                                let list_h = body_h - 16.0 - 40.0 - 12.0 - 16.0;
                                ui.allocate_ui_with_layout(
                                    vec2(content_w, list_h),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ScrollArea::vertical()
                                            .max_height(list_h)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                if rows.is_empty() {
                                                    ui.add_space(48.0);
                                                    ui.vertical_centered(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(
                                                                lang.inst_group_empty(),
                                                            )
                                                            .font(egui::FontId::proportional(15.0))
                                                            .color(MODRINTH_TEXT_SECONDARY),
                                                        );
                                                    });
                                                    return;
                                                }
                                                for (row_index, idx) in rows.into_iter().enumerate()
                                                {
                                                    if row_index > 0 {
                                                        ui.add_space(4.0);
                                                    }
                                                    let inst = &instances[idx];
                                                    let inst_id = inst.id;
                                                    let selected =
                                                        self.new_group_selected.contains(&inst_id);
                                                    let (row, row_resp) = ui.allocate_exact_size(
                                                        vec2(content_w, 44.0),
                                                        Sense::hover(),
                                                    );
                                                    let btn_rect = Rect::from_min_size(
                                                        pos2(
                                                            row.right() - add_w,
                                                            row.center().y - 18.0,
                                                        ),
                                                        vec2(add_w, 36.0),
                                                    );
                                                    let btn_resp = ui.interact(
                                                        btn_rect,
                                                        egui::Id::new((
                                                            "modrinth_group_add",
                                                            inst_id,
                                                        )),
                                                        Sense::click(),
                                                    );
                                                    if row_resp.hovered() || btn_resp.hovered() {
                                                        ui.painter().rect_filled(
                                                            row.expand2(vec2(24.0, 0.0)),
                                                            Rounding::ZERO,
                                                            MODRINTH_SURFACE_HOVER,
                                                        );
                                                    }

                                                    let fade = if selected { 150 } else { 255 };
                                                    let icon_rect = Rect::from_min_size(
                                                        pos2(row.left(), row.top() + 6.0),
                                                        vec2(32.0, 32.0),
                                                    );
                                                    match self.instance_icon_texture(
                                                        ui.ctx(),
                                                        inst_id,
                                                        &inst.get_game_dir(instances_dir),
                                                    ) {
                                                        Some(tex) => {
                                                            ui.painter().image(
                                                                tex.id(),
                                                                icon_rect,
                                                                Rect::from_min_max(
                                                                    pos2(0.0, 0.0),
                                                                    pos2(1.0, 1.0),
                                                                ),
                                                                with_alpha(Color32::WHITE, fade),
                                                            );
                                                        }
                                                        None => {
                                                            if let Some(preset) = inst
                                                                .icon
                                                                .as_deref()
                                                                .and_then(icons::instance_preset)
                                                            {
                                                                ui.painter().rect_filled(
                                                                    icon_rect,
                                                                    Rounding::same(8.0),
                                                                    MODRINTH_SURFACE_HOVER,
                                                                );
                                                                icons::draw_centered(
                                                                    ui.painter(),
                                                                    preset,
                                                                    icon_rect.center(),
                                                                    14.0,
                                                                    with_alpha(MODRINTH_TEXT, fade),
                                                                );
                                                            } else if let Some(tex) =
                                                                crate::widgets::modrinth_assets::loader_texture(
                                                                    ui.ctx(),
                                                                    inst.loader,
                                                                )
                                                            {
                                                                ui.painter().image(
                                                                    tex.id(),
                                                                    icon_rect,
                                                                    Rect::from_min_max(
                                                                        pos2(0.0, 0.0),
                                                                        pos2(1.0, 1.0),
                                                                    ),
                                                                    with_alpha(Color32::WHITE, fade),
                                                                );
                                                            } else {
                                                                ui.painter().rect_filled(
                                                                    icon_rect,
                                                                    Rounding::same(8.0),
                                                                    MODRINTH_SURFACE_HOVER,
                                                                );
                                                                icons::draw_centered(
                                                                    ui.painter(),
                                                                    icons::loader_badge(inst.loader),
                                                                    icon_rect.center(),
                                                                    14.0,
                                                                    with_alpha(MODRINTH_TEXT, fade),
                                                                );
                                                            }
                                                        }
                                                    }

                                                    let group = inst
                                                        .group
                                                        .as_deref()
                                                        .filter(|g| !g.is_empty());
                                                    let name_color =
                                                        with_alpha(MODRINTH_TEXT_CONTRAST, fade);
                                                    let name_limit =
                                                        if group.is_some() { 30 } else { 42 };
                                                    let name_galley = ui.painter().layout_no_wrap(
                                                        truncate_name(&inst.name, name_limit),
                                                        egui::FontId::proportional(15.0),
                                                        name_color,
                                                    );
                                                    let name_pos = pos2(
                                                        icon_rect.right() + 10.0,
                                                        row.center().y - name_galley.size().y / 2.0,
                                                    );
                                                    let name_w = name_galley.size().x;
                                                    ui.painter().galley(
                                                        name_pos,
                                                        name_galley,
                                                        name_color,
                                                    );
                                                    if let Some(group) = group {
                                                        let tag_text = truncate_name(group, 24);
                                                        let tag_galley =
                                                            ui.painter().layout_no_wrap(
                                                                tag_text,
                                                                egui::FontId::proportional(12.0),
                                                                with_alpha(
                                                                    MODRINTH_TEXT_SECONDARY,
                                                                    fade,
                                                                ),
                                                            );
                                                        let tag = Rect::from_min_size(
                                                            pos2(
                                                                name_pos.x + name_w + 8.0,
                                                                row.center().y - 10.0,
                                                            ),
                                                            vec2(tag_galley.size().x + 14.0, 20.0),
                                                        );
                                                        ui.painter().rect_filled(
                                                            tag,
                                                            Rounding::same(4.0),
                                                            with_alpha(
                                                                MODRINTH_SURFACE_HOVER,
                                                                fade,
                                                            ),
                                                        );
                                                        ui.painter().rect_stroke(
                                                            tag,
                                                            Rounding::same(4.0),
                                                            Stroke::new(
                                                                1.0_f32,
                                                                with_alpha(
                                                                    MODRINTH_SURFACE_HIGH,
                                                                    fade,
                                                                ),
                                                            ),
                                                        );
                                                        ui.painter().galley(
                                                            pos2(
                                                                tag.center().x
                                                                    - tag_galley.size().x / 2.0,
                                                                tag.center().y
                                                                    - tag_galley.size().y / 2.0,
                                                            ),
                                                            tag_galley,
                                                            with_alpha(
                                                                MODRINTH_TEXT_SECONDARY,
                                                                fade,
                                                            ),
                                                        );
                                                    }

                                                    let btn_hover =
                                                        ui.ctx().animate_bool_responsive(
                                                            btn_resp.id,
                                                            btn_resp.hovered(),
                                                        );
                                                    let (btn_bg, btn_border, btn_text) = if selected
                                                    {
                                                        (
                                                            Color32::TRANSPARENT,
                                                            MODRINTH_SURFACE_HIGH,
                                                            MODRINTH_TEXT,
                                                        )
                                                    } else {
                                                        (
                                                            lerp_color(
                                                                MODRINTH_SURFACE_HOVER,
                                                                m_brightness(
                                                                    MODRINTH_SURFACE_HOVER,
                                                                    1.1,
                                                                ),
                                                                btn_hover,
                                                            ),
                                                            MODRINTH_SURFACE_HIGH,
                                                            MODRINTH_TEXT_CONTRAST,
                                                        )
                                                    };
                                                    ui.painter().rect_filled(
                                                        btn_rect,
                                                        Rounding::same(10.0),
                                                        btn_bg,
                                                    );
                                                    ui.painter().rect_stroke(
                                                        btn_rect,
                                                        Rounding::same(10.0),
                                                        Stroke::new(1.0_f32, btn_border),
                                                    );
                                                    let label = if selected {
                                                        lang.inst_group_added()
                                                    } else {
                                                        lang.inst_group_add()
                                                    };
                                                    let galley = ui.painter().layout_no_wrap(
                                                        label.to_owned(),
                                                        egui::FontId::proportional(14.0),
                                                        btn_text,
                                                    );
                                                    let content_w = if selected {
                                                        16.0 + 6.0 + galley.size().x
                                                    } else {
                                                        galley.size().x
                                                    };
                                                    let content_left =
                                                        btn_rect.center().x - content_w / 2.0;
                                                    if selected {
                                                        icons::draw(
                                                            ui.painter(),
                                                            Icon::Check,
                                                            Rect::from_center_size(
                                                                pos2(
                                                                    content_left + 8.0,
                                                                    btn_rect.center().y,
                                                                ),
                                                                vec2(14.0, 14.0),
                                                            ),
                                                            MODRINTH_TEXT,
                                                            1.8,
                                                        );
                                                    }
                                                    let text_left = content_left
                                                        + if selected { 22.0 } else { 0.0 };
                                                    ui.painter().galley(
                                                        pos2(
                                                            text_left,
                                                            btn_rect.center().y
                                                                - galley.size().y / 2.0,
                                                        ),
                                                        galley,
                                                        btn_text,
                                                    );
                                                    if btn_resp.hovered() {
                                                        ui.ctx().set_cursor_icon(
                                                            egui::CursorIcon::PointingHand,
                                                        );
                                                    }
                                                    if btn_resp.clicked() {
                                                        if selected {
                                                            self.new_group_selected
                                                                .remove(&inst_id);
                                                        } else {
                                                            self.new_group_selected.insert(inst_id);
                                                        }
                                                    }
                                                }
                                            });
                                    },
                                );
                            },
                        );

                        let (footer_divider, _) =
                            ui.allocate_exact_size(vec2(content_w, 1.0), Sense::hover());
                        ui.painter().rect_filled(
                            footer_divider.expand2(vec2(24.0, 0.0)),
                            Rounding::ZERO,
                            MODRINTH_SURFACE_HIGH,
                        );
                        ui.add_space(16.0);
                        let can_create = !self.new_group_name.trim().is_empty();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if m_button_enabled(
                                ui,
                                Some(Icon::Plus),
                                lang.inst_group_create(),
                                true,
                                40.0,
                                can_create,
                            ) {
                                create = true;
                            }
                            ui.add_space(8.0);
                            if m_button(
                                ui,
                                Some(Icon::Close),
                                lang.inst_group_cancel(),
                                false,
                                40.0,
                            ) {
                                close = true;
                            }
                        });
                    });
            });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }

        if create || enter {
            let name: String = self
                .new_group_name
                .trim()
                .chars()
                .take(256)
                .collect::<String>();
            if !name.is_empty() {
                let selected = self.new_group_selected.clone();
                for inst in instances.iter_mut() {
                    if selected.contains(&inst.id) {
                        inst.group = Some(name.clone());
                    }
                }
                let first = instances
                    .iter()
                    .find(|inst| selected.contains(&inst.id))
                    .cloned();
                self.show_new_group_modal = false;
                self.new_group_fresh_reset();
                return Some(match first {
                    Some(inst) => InstanceAction::Updated(inst),
                    None => InstanceAction::GroupCreated(name),
                });
            }
        }
        if close {
            self.show_new_group_modal = false;
            self.new_group_fresh_reset();
        }
        None
    }

    fn new_group_fresh_reset(&mut self) {
        self.new_group_name.clear();
        self.new_group_search.clear();
        self.new_group_selected.clear();
    }

    /// Modrinth-style search input (reference `InputFrame`: rounded-xl,
    /// surface-4 fill, surface-5 border, leading search icon). A native
    /// `TextEdit` inside a styled scope keeps click-to-caret behaviour intact;
    /// the icon and clear action are painted on top.
    fn modrinth_search_field(&mut self, ui: &mut Ui, width: f32, lang: Language) {
        let edit_id = ui.id().with("modrinth_library_search");
        let has_query = !self.search_query.is_empty();
        let right_pad = if has_query { 36.0 } else { 14.0 };
        let text_w = (width - 38.0 - right_pad).max(80.0);
        let field_left = ui.cursor().left();
        let edit = TextEdit::singleline(&mut self.search_query)
            .id(edit_id)
            .frame(true)
            .desired_width(text_w)
            .margin(egui::Margin {
                left: 38.0,
                right: right_pad,
                top: 9.0,
                bottom: 9.0,
            })
            .hint_text(
                egui::RichText::new(lang.inst_library_search()).color(MODRINTH_TEXT_SECONDARY),
            )
            .font(egui::FontId::proportional(15.0));
        let resp = ui
            .scope(|ui| {
                let visuals = &mut ui.style_mut().visuals;
                visuals.extreme_bg_color = MODRINTH_FIELD;
                visuals.widgets.inactive.bg_fill = MODRINTH_FIELD;
                visuals.widgets.inactive.weak_bg_fill = MODRINTH_FIELD;
                visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
                visuals.widgets.inactive.rounding = Rounding::same(12.0);
                visuals.widgets.hovered.bg_fill = MODRINTH_FIELD;
                visuals.widgets.hovered.weak_bg_fill = MODRINTH_FIELD;
                visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, MODRINTH_SURFACE_HIGH);
                visuals.widgets.hovered.rounding = Rounding::same(12.0);
                visuals.widgets.active.bg_fill = MODRINTH_FIELD;
                visuals.widgets.active.weak_bg_fill = MODRINTH_FIELD;
                visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, MODRINTH_GREEN);
                visuals.widgets.active.rounding = Rounding::same(12.0);
                visuals.override_text_color = Some(MODRINTH_TEXT_CONTRAST);
                visuals.selection.bg_fill = with_alpha(MODRINTH_GREEN, 90);
                ui.add(edit)
            })
            .inner;
        let rect = Rect::from_min_size(
            pos2(field_left, resp.rect.center().y - 18.0),
            vec2(width, 36.0),
        );
        icons::draw(
            ui.painter(),
            Icon::Search,
            Rect::from_center_size(pos2(rect.left() + 22.0, rect.center().y), vec2(18.0, 18.0)),
            MODRINTH_TEXT_SECONDARY,
            1.5,
        );
        if resp.has_focus() {
            ui.painter().rect_stroke(
                rect,
                Rounding::same(12.0),
                Stroke::new(1.0_f32, MODRINTH_GREEN),
            );
        }
        if has_query {
            let clear_rect = Rect::from_center_size(
                pos2(rect.right() - 19.0, rect.center().y),
                vec2(24.0, 24.0),
            );
            let clear_resp = ui.interact(
                clear_rect,
                ui.id().with("modrinth_library_search_clear"),
                Sense::click(),
            );
            if clear_resp.hovered() {
                ui.painter().rect_filled(
                    clear_rect,
                    Rounding::same(7.0),
                    with_alpha(MODRINTH_TEXT_CONTRAST, 18),
                );
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            icons::draw(
                ui.painter(),
                Icon::Close,
                clear_rect.shrink(7.0),
                if clear_resp.hovered() {
                    MODRINTH_TEXT_CONTRAST
                } else {
                    MODRINTH_TEXT_SECONDARY
                },
                1.6,
            );
            if clear_resp.on_hover_text(lang.mods_search_clear()).clicked() {
                self.search_query.clear();
                ui.memory_mut(|m| m.request_focus(edit_id));
            }
        }
        if ui.memory(|m| m.has_focus(edit_id)) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.search_query.clear();
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }
    }

    /// Fill the edit dialog from `inst` and open it (shared by the card's
    /// gear button and the shell instance toolbar).
    pub fn open_edit_modal(&mut self, inst: &Instance) {
        self.edit_instance_id = Some(inst.id);
        self.edit_instance_name = inst.name.clone();
        self.edit_instance_version = inst.game_version.clone();
        self.edit_instance_loader = inst.loader;
        self.edit_loader_pin = inst.loader_version.clone();
        self.edit_instance_ram = inst.ram_mb.unwrap_or(4096);
        self.edit_instance_icon = inst.icon.clone();
        self.edit_instance_tags = inst.tags.join(", ");
        self.edit_instance_pinned = inst.pinned;
        self.edit_instance_account = inst.default_account;
        self.edit_instance_group = inst.group.clone().unwrap_or_default();
        self.edit_use_custom_size = inst.window_size.is_some();
        self.edit_window_w = inst.window_size.map(|(w, _)| w).unwrap_or(1280);
        self.edit_window_h = inst.window_size.map(|(_, h)| h).unwrap_or(720);
        self.edit_fullscreen = inst.fullscreen;
        self.edit_instance_notes = inst.notes.clone();
        self.edit_instance_pre = inst.pre_launch_cmd.clone();
        self.edit_instance_post = inst.post_exit_cmd.clone();
        self.show_edit_modal = true;
    }

    /// Open the manage dialog (worlds / configs / health) of an instance.
    pub fn open_manage(&mut self, id: Uuid) {
        self.manage.open_for(id);
    }

    /// MultiMC shell: the reference instance view — 100px cells with a 48px
    /// icon and the name plate below, grouped under framed headers with a
    /// collapse square. Metrics follow `ui/instanceview/InstanceDelegate`
    /// and `VisualGroup`: 5px margins and spacing, 105px cell pitch.
    fn show_multimc_view(
        &mut self,
        ui: &mut Ui,
        instances: &[Instance],
        selected_instance: &mut Option<Uuid>,
        ctx: &InstancesContext,
        lang: Language,
        action: &mut InstanceAction,
    ) {
        if instances.is_empty() {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(lang.inst_empty_title())
                        .font(egui::FontId::proportional(15.0))
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(lang.inst_empty_hint())
                        .font(egui::FontId::proportional(12.0))
                        .color(TEXT_MUTED),
                );
                ui.add_space(12.0);
                let button = egui::Button::new(
                    egui::RichText::new(lang.shell_add_instance())
                        .font(egui::FontId::proportional(12.0)),
                )
                .min_size(vec2(160.0, 32.0));
                if ui.add(button).clicked() {
                    self.new_instance_ram = ctx.suggested_ram_mb;
                    self.show_create_modal = true;
                }
            });
            return;
        }

        // Proxy-model grouping of the reference: the empty group sorts
        // first, then the named groups; instances sort by name inside.
        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, inst) in instances.iter().enumerate() {
            groups
                .entry(inst.group.clone().unwrap_or_default())
                .or_default()
                .push(idx);
        }
        for indices in groups.values_mut() {
            indices.sort_by_key(|&i| instances[i].name.to_lowercase());
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
                ui.add_space(3.0);
                for (group, indices) in &groups {
                    let collapsed = self.multimc_collapsed.contains(group);
                    // MultiMC labels its default (ungrouped) bucket too.
                    let group_label = if group.is_empty() {
                        lang.nav_instances()
                    } else {
                        group.as_str()
                    };
                    if multimc_group_header(ui, group_label, collapsed) {
                        if collapsed {
                            self.multimc_collapsed.remove(group);
                        } else {
                            self.multimc_collapsed.insert(group.clone());
                        }
                    }
                    if collapsed {
                        continue;
                    }
                    let avail = ui.available_width();
                    let cols = (((avail - 10.0) / 105.0).floor() as usize).max(1);
                    for row in indices.chunks(cols) {
                        let text_h = row
                            .iter()
                            .map(|&idx| {
                                multimc_name_galley(
                                    ui.painter(),
                                    &instances[idx].name,
                                    egui::FontId::proportional(12.0),
                                    94.0,
                                )
                                .size()
                                .y
                            })
                            .fold(0.0_f32, f32::max);
                        let cell_h = 48.0 + 5.0 + text_h + 6.0;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
                            ui.add_space(5.0);
                            for &idx in row {
                                multimc_cell(
                                    self,
                                    ui,
                                    &instances[idx],
                                    cell_h,
                                    selected_instance,
                                    ctx.instances_dir,
                                    action,
                                );
                            }
                        });
                    }
                    ui.add_space(3.0);
                }
            });

        /// Framed group strip (reference: top rule with short fading side
        /// rules, a collapse square and the bold group name at 60% alpha).
        fn multimc_group_header(ui: &mut Ui, name: &str, collapsed: bool) -> bool {
            let h = 26.0;
            let (rect, resp) =
                ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
            let painter = ui.painter();
            let line = with_alpha(TEXT_PRIMARY, 90);
            let top = rect.top() + 0.5;
            painter.line_segment(
                [pos2(rect.left() + 5.0, top), pos2(rect.right() - 5.0, top)],
                Stroke::new(1.0_f32, line),
            );
            // The reference fades the side rules out over roughly one text
            // line instead of closing the group into a full box.
            for x in [rect.left() + 3.5, rect.right() - 3.5] {
                for (offset, alpha) in [(0.0, 90), (4.0, 64), (8.0, 38), (12.0, 16)] {
                    painter.line_segment(
                        [pos2(x, top + 2.0 + offset), pos2(x, top + 6.0 + offset)],
                        Stroke::new(1.0_f32, with_alpha(TEXT_PRIMARY, alpha)),
                    );
                }
            }
            let square =
                Rect::from_min_size(pos2(rect.left() + 9.0, rect.top() + 6.0), vec2(13.0, 13.0));
            painter.rect_stroke(square, Rounding::ZERO, Stroke::new(1.0_f32, line));
            let c = square.center();
            painter.line_segment(
                [pos2(c.x - 3.5, c.y), pos2(c.x + 3.5, c.y)],
                Stroke::new(1.0_f32, line),
            );
            if collapsed {
                painter.line_segment(
                    [pos2(c.x, c.y - 3.5), pos2(c.x, c.y + 3.5)],
                    Stroke::new(1.0_f32, line),
                );
            }
            if !name.is_empty() {
                // Qt uses a bold sans face here, not the display family.
                let font = egui::FontId::new(12.5, egui::FontFamily::Name("bold".into()));
                let max_w = (rect.right() - (square.right() + 7.0) - 8.0).max(24.0);
                let visible = crate::shells::short_name(name, ((max_w / 7.2) as usize).max(1));
                painter.text(
                    pos2(square.right() + 7.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &visible,
                    font,
                    with_alpha(TEXT_PRIMARY, 153),
                );
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let resp = if name.chars().count() > 36 {
                resp.on_hover_text(name)
            } else {
                resp
            };
            resp.clicked()
        }

        /// One instance cell: 48px icon (custom texture or the default
        /// tile with the loader/preset badge), name plate below (window
        /// color at 160 alpha, or the highlight when selected).
        fn multimc_cell(
            page: &mut InstancesPage,
            ui: &mut Ui,
            inst: &Instance,
            cell_h: f32,
            selected_instance: &mut Option<Uuid>,
            instances_dir: &Path,
            action: &mut InstanceAction,
        ) {
            let inst_id = inst.id;
            let is_selected = selected_instance.as_ref() == Some(&inst_id);
            let (rect, resp) = ui.allocate_exact_size(vec2(100.0, cell_h), Sense::click());
            let icon_rect =
                Rect::from_center_size(pos2(rect.center().x, rect.top() + 24.0), vec2(48.0, 48.0));
            match page.instance_icon_texture(ui.ctx(), inst_id, &inst.get_game_dir(instances_dir)) {
                Some(tex) => {
                    ui.painter().image(
                        tex.id(),
                        icon_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().rect_stroke(
                        icon_rect,
                        Rounding::ZERO,
                        Stroke::new(1.0_f32, BORDER_DEFAULT),
                    );
                }
                None => {
                    // Default icon tile: the reference icon art is a light
                    // square; the badge stands in for it.
                    ui.painter().rect_filled(
                        icon_rect,
                        Rounding::ZERO,
                        with_alpha(Color32::WHITE, 170),
                    );
                    ui.painter().rect_stroke(
                        icon_rect,
                        Rounding::ZERO,
                        Stroke::new(1.0_f32, BORDER_DEFAULT),
                    );
                    let badge = match inst.icon.as_deref().and_then(icons::instance_preset) {
                        Some(preset) => preset,
                        None => icons::loader_badge(inst.loader),
                    };
                    icons::draw_centered(
                        ui.painter(),
                        badge,
                        icon_rect.center(),
                        30.0,
                        loader_color(inst.loader),
                    );
                }
            }
            let plate = Rect::from_min_max(
                pos2(rect.left(), icon_rect.bottom() + 5.0),
                pos2(rect.right(), rect.bottom()),
            );
            if is_selected {
                ui.painter()
                    .rect_filled(plate, Rounding::ZERO, crate::theme::ACCENT);
            } else {
                ui.painter()
                    .rect_filled(plate, Rounding::ZERO, with_alpha(crate::theme::BG, 160));
            }
            let galley = multimc_name_galley(
                ui.painter(),
                &inst.name,
                egui::FontId::proportional(12.0),
                94.0,
            );
            let color = if is_selected {
                Color32::from_rgb(0xEF, 0xF0, 0xF1)
            } else {
                TEXT_PRIMARY.into()
            };
            ui.painter().galley(
                pos2(rect.center().x - galley.size().x / 2.0, plate.top() + 3.0),
                galley,
                color,
            );
            if resp.clicked() {
                *selected_instance = Some(inst_id);
                *action = InstanceAction::Selected(inst_id);
            }
            if resp.double_clicked() {
                *action = InstanceAction::Launch(inst_id);
            }
            let _ = resp.on_hover_text(inst.get_game_dir(instances_dir).display().to_string());
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

        match ctx.skin {
            LibrarySkin::Modrinth => {
                // Modrinth shell: the shared "Modpacks" page renders the
                // reference app's library (heading, toolbar, card grid).
                self.show_modrinth_library(
                    ui,
                    instances,
                    selected_instance,
                    ctx,
                    lang,
                    &mut action,
                );
            }
            LibrarySkin::Multimc => {
                // MultiMC shell: the shared page renders the reference's
                // icon-grid instance view instead of the card list.
                self.show_multimc_view(ui, instances, selected_instance, ctx, lang, &mut action);
            }
            LibrarySkin::Amc | LibrarySkin::Labymod => {
                let labymod = ctx.skin == LibrarySkin::Labymod;
                if labymod {
                    labymod_instances_header(ui, lang);
                    ui.add_space(12.0);
                } else {
                    ui.add_space(20.0);

                    // Header
                    ui.label(
                        egui::RichText::new(lang.inst_title())
                            .font(egui::FontId::proportional(26.0))
                            .strong()
                            .color(TEXT_HEADING),
                    );

                    ui.add_space(14.0);
                }

                // Control Row
                ui.horizontal(|ui| {
                    let search_width = (ui.available_width() - 180.0).max(200.0);
                    ui.add(
                        TextEdit::singleline(&mut self.search_query)
                            .hint_text(
                                egui::RichText::new(lang.inst_search_hint()).color(TEXT_MUTED),
                            )
                            .desired_width(search_width)
                            .font(egui::FontId::proportional(14.0)),
                    );

                    ui.add_space(10.0);

                    let (cr_rect, cr_resp) =
                        ui.allocate_exact_size(vec2(160.0, 36.0), Sense::click());
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
                    // The trending feed is secondary to the user's own
                    // instances. Cap it to the shorter of 220 px and a third
                    // of the remaining page height so opening it at 960x600
                    // cannot push the local grid off-screen.
                    let discover_max = (ui.available_height() * 0.28).clamp(80.0, 200.0);
                    egui::ScrollArea::vertical()
                        .id_salt("discover_pack_list")
                        .max_height(discover_max)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
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
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button(lang.discover_install()).clicked() {
                                                install = Some(pack.id.clone());
                                            }
                                        },
                                    );
                                });
                            }
                        });
                    if let Some(slug) = install {
                        self.discover_install = Some(slug);
                    }
                    ui.add_space(8.0);
                }

                // Filter + sort instances (memoized — see `filtered_indices`).
                let filtered_indices = self.filtered_indices(instances);

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
                                    .button(
                                        egui::RichText::new(lang.inst_btn_create())
                                            .color(RUBY_LIGHT),
                                    )
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
            }
        }

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
            let loader_color = loader_color(inst.loader);
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
                // Four compact stats must fit the 270px card at the minimum
                // window size; the default 8px gaps clip the session count.
                ui.spacing_mut().item_spacing.x = 4.0;
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
                    page.open_edit_modal(inst);
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
            if inst.name.chars().count() > 22 {
                resp.on_hover_text(inst.name.as_str());
            }
        }

        // Trash (CONCEPT trash, P2): soft-deleted instances wait here
        // for restore or permanent deletion. FS scan throttled to 2s —
        // never per frame. The Modrinth library skin has no trash section.
        if !matches!(ctx.skin, LibrarySkin::Modrinth | LibrarySkin::Multimc) {
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
        }

        // Create Instance Modal Dialog
        if self.show_create_modal {
            if ctx.skin == LibrarySkin::Modrinth {
                if let Some(created) = self.modrinth_create_modal(ui.ctx(), lang, selected_instance)
                {
                    action = created;
                }
            } else {
                let mut open = true;
                if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape))
                    && !ui.ctx().memory(|m| m.any_popup_open())
                {
                    open = false;
                }
                let labymod = ctx.skin == LibrarySkin::Labymod;
                let shell_accent = labymod || palette_id() == PaletteId::Millida;
                let max_h = (ui.ctx().screen_rect().height() - 120.0).max(320.0);
                let old_heading = if shell_accent {
                    let old = ui
                        .ctx()
                        .style()
                        .text_styles
                        .get(&egui::TextStyle::Heading)
                        .cloned();
                    ui.ctx().style_mut(|style| {
                        style
                            .text_styles
                            .insert(egui::TextStyle::Heading, egui::FontId::proportional(18.0));
                    });
                    old
                } else {
                    None
                };
                // The LabyMod frame adds inner padding; nudge the dialog down so
                // its top edge stays clear of the custom titlebar.
                let anchor_offset = if labymod {
                    vec2(0.0, 16.0)
                } else {
                    vec2(0.0, 0.0)
                };
                let mut window = egui::Window::new(lang.inst_modal_create_title())
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, anchor_offset)
                    .min_width(420.0)
                    .max_height(max_h);
                if labymod {
                    window = window.frame(
                        egui::Frame::none()
                            .fill(BG_CARD.into())
                            .stroke(Stroke::new(1.0_f32, BORDER_STRONG))
                            .rounding(Rounding::same(16.0))
                            .inner_margin(egui::Margin::same(18.0)),
                    );
                }
                window.show(ui.ctx(), |ui| {
                    let scroll_h = (max_h - 120.0).max(220.0);
                    ScrollArea::vertical()
                        .max_height(scroll_h)
                        .auto_shrink([false, true])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                        )
                        .show(ui, |ui| {
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
                                    if choice_chip(
                                        ui,
                                        lang.tmpl_name(tmpl.id),
                                        130.0,
                                        32.0,
                                        12.0,
                                        active,
                                    ) {
                                        self.apply_template(tmpl, lang);
                                    }
                                }
                            });
                            let tmpl_desc = InstanceTemplate::matching(
                                self.new_instance_loader,
                                self.new_instance_ram,
                            )
                            .map(|t| lang.tmpl_desc(t.id))
                            .unwrap_or_else(|| lang.tmpl_custom_note());
                            ui.label(egui::RichText::new(tmpl_desc).color(TEXT_MUTED));
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.new_instance_name)
                                    .desired_width(400.0),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.new_instance_version)
                                    .desired_width(400.0),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY),
                            );
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
                                    "{} {} MB",
                                    lang.inst_modal_ram(),
                                    self.new_instance_ram
                                ))
                                .color(TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.new_instance_ram, 1024..=16384)
                                    .step_by(512.0),
                            );
                        });
                    if shell_accent {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);
                    } else {
                        ui.add_space(18.0);
                    }
                    ui.horizontal(|ui| {
                        let can_create = !self.new_instance_name.trim().is_empty()
                            && !self.new_instance_version.trim().is_empty();
                        let clicked = if shell_accent {
                            shell_primary_button(ui, lang.inst_modal_btn_create(), can_create)
                        } else {
                            let create = egui::Button::new(
                                egui::RichText::new(lang.inst_modal_btn_create())
                                    .strong()
                                    .color(dialog_button_fg(ui)),
                            );
                            ui.add_enabled(can_create, create).clicked()
                        };
                        if clicked {
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

                if let Some(font) = old_heading {
                    ui.ctx().style_mut(|style| {
                        style.text_styles.insert(egui::TextStyle::Heading, font);
                    });
                }

                if !open {
                    self.show_create_modal = false;
                }
            }
        }

        // Clone Instance Modal Dialog: duplicate with an optional
        // worlds/saves copy (CONCEPT P2).
        if self.show_clone_modal {
            let mut open = true;
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape))
                && !ui.ctx().memory(|m| m.any_popup_open())
            {
                open = false;
            }
            let accent = ctx.skin == LibrarySkin::Labymod || palette_id() == PaletteId::Millida;
            let labymod = ctx.skin == LibrarySkin::Labymod;
            let old_heading = if accent {
                let old = ui
                    .ctx()
                    .style()
                    .text_styles
                    .get(&egui::TextStyle::Heading)
                    .cloned();
                ui.ctx().style_mut(|style| {
                    style
                        .text_styles
                        .insert(egui::TextStyle::Heading, egui::FontId::proportional(18.0));
                });
                old
            } else {
                None
            };
            let mut window = egui::Window::new(lang.clone_title())
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(360.0);
            if labymod {
                window = window.frame(
                    egui::Frame::none()
                        .fill(BG_CARD.into())
                        .stroke(Stroke::new(1.0_f32, BORDER_STRONG))
                        .rounding(Rounding::same(16.0))
                        .inner_margin(egui::Margin::same(18.0)),
                );
            }
            window.show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                ui.checkbox(&mut self.clone_with_saves, lang.clone_saves());
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    let clicked = if accent {
                        shell_primary_button(ui, lang.inst_btn_clone(), true)
                    } else {
                        ui.button(
                            egui::RichText::new(lang.inst_btn_clone())
                                .strong()
                                .color(dialog_button_fg(ui)),
                        )
                        .clicked()
                    };
                    if clicked {
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
            if let Some(font) = old_heading {
                ui.ctx().style_mut(|style| {
                    style.text_styles.insert(egui::TextStyle::Heading, font);
                });
            }
            if !open {
                self.show_clone_modal = false;
            }
        }

        // Edit Instance Modal Dialog
        if self.show_edit_modal {
            let mut close_edit = false;
            let mut save_edit = false;
            let mut open = true;
            let accent = ctx.skin == LibrarySkin::Labymod || palette_id() == PaletteId::Millida;
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape))
                && !ui.ctx().memory(|m| m.any_popup_open())
            {
                open = false;
            }

            let max_h = (ui.ctx().screen_rect().height() - 160.0).max(320.0);
            let old_heading = ui
                .ctx()
                .style()
                .text_styles
                .get(&egui::TextStyle::Heading)
                .cloned();
            ui.ctx().style_mut(|style| {
                style
                    .text_styles
                    .insert(egui::TextStyle::Heading, egui::FontId::proportional(18.0));
            });
            egui::Window::new(lang.inst_modal_edit_title())
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .min_width(440.0)
                .max_height(max_h)
                .show(ui.ctx(), |ui| {
                    let scroll_h = (max_h - 110.0).max(220.0);
                    ScrollArea::vertical()
                        .max_height(scroll_h)
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_name()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_name)
                                    .desired_width(400.0),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_version()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_version)
                                    .desired_width(400.0),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_modal_loader()).color(TEXT_PRIMARY),
                            );
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
                                    "{} {} MB",
                                    lang.inst_modal_ram(),
                                    self.edit_instance_ram
                                ))
                                .color(TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.edit_instance_ram, 1024..=32768)
                                    .step_by(512.0),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_icon_title()).color(TEXT_PRIMARY),
                            );
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                                for (key, _) in InstanceIcon::ALL {
                                    let active = self.edit_instance_icon.as_deref() == Some(key);
                                    let (rect, resp) =
                                        ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
                                    let hover_t =
                                        ui.ctx().animate_bool_responsive(resp.id, resp.hovered());
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
                                            if let Some(inst) =
                                                instances.iter().find(|i| i.id == target_id)
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
                                        if let Some(inst) =
                                            instances.iter().find(|i| i.id == target_id)
                                        {
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
                            ui.label(
                                egui::RichText::new(lang.inst_tags_label()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_tags)
                                    .hint_text(lang.inst_tags_hint())
                                    .desired_width(400.0),
                            );
                            ui.add_space(6.0);
                            ui.checkbox(&mut self.edit_instance_pinned, lang.inst_pin_label());

                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_account_label()).color(TEXT_PRIMARY),
                            );
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
                            ui.label(
                                egui::RichText::new(lang.inst_group_label()).color(TEXT_PRIMARY),
                            );
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
                            ui.label(
                                egui::RichText::new(lang.inst_notes_label()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_notes)
                                    .hint_text(lang.inst_notes_hint())
                                    .desired_width(400.0),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(lang.inst_pre_label()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_pre)
                                    .hint_text(lang.inst_cmd_hint())
                                    .desired_width(400.0)
                                    .font(egui::FontId::monospace(12.0)),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(lang.inst_post_label()).color(TEXT_PRIMARY),
                            );
                            ui.add(
                                TextEdit::singleline(&mut self.edit_instance_post)
                                    .hint_text(lang.inst_cmd_hint())
                                    .desired_width(400.0)
                                    .font(egui::FontId::monospace(12.0)),
                            );
                        });
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let can_save = !self.edit_instance_name.trim().is_empty()
                            && !self.edit_instance_version.trim().is_empty();
                        let clicked = if accent {
                            shell_primary_button(ui, lang.inst_modal_btn_save(), can_save)
                        } else {
                            let save = egui::Button::new(
                                egui::RichText::new(lang.inst_modal_btn_save())
                                    .strong()
                                    .color(dialog_button_fg(ui)),
                            );
                            ui.add_enabled(can_save, save).clicked()
                        };
                        if clicked {
                            save_edit = true;
                        }

                        if ui.button(lang.inst_modal_btn_cancel()).clicked() {
                            close_edit = true;
                        }
                    });
                });

            if let Some(font) = old_heading {
                ui.ctx().style_mut(|style| {
                    style.text_styles.insert(egui::TextStyle::Heading, font);
                });
            }

            if !open {
                close_edit = true;
            }

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
        let none = ModrinthFilters::default();
        assert!(matches_filter(&inst, "", None, &none));
        assert!(matches_filter(&inst, "surv", None, &none));
        assert!(!matches_filter(&inst, "creative", None, &none));
        assert!(matches_filter(&inst, "", Some("smp"), &none));
        assert!(!matches_filter(&inst, "", Some("modded"), &none));
        assert!(!matches_filter(&inst, "surv", Some("modded"), &none));
    }

    #[test]
    fn test_matches_filter_loader_and_version() {
        let mut inst = Instance::new("Pack", "1.20.1", LoaderType::Fabric);
        let mut filters = ModrinthFilters::default();
        filters.toggle_loader(LoaderType::Fabric);
        assert!(matches_filter(&inst, "", None, &filters));
        filters.toggle_loader(LoaderType::Forge);
        filters.toggle_loader(LoaderType::Fabric);
        assert!(!matches_filter(&inst, "", None, &filters));
        filters.toggle_loader(LoaderType::Forge);
        filters.toggle_version("1.20.1");
        assert!(matches_filter(&inst, "", None, &filters));
        inst.game_version = "1.21.1".to_string();
        assert!(!matches_filter(&inst, "", None, &filters));
    }

    #[test]
    fn test_natural_version_order() {
        assert_eq!(natural_version_cmp("1.9", "1.10"), std::cmp::Ordering::Less);
        assert_eq!(
            natural_version_cmp("1.21.1", "1.21"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            natural_version_cmp("1.20.1", "1.20.1"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_modrinth_sort_game_version_is_numeric() {
        let mut old = tagged("Old", &[], false, 0);
        old.game_version = "1.9".to_string();
        let mut new = tagged("New", &[], false, 0);
        new.game_version = "1.10".to_string();
        assert_eq!(
            compare_instances(&old, &new, InstanceSort::GameVersion),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_modrinth_custom_group_sections() {
        let mut alpha = tagged("Alpha instance", &[], false, 0);
        alpha.group = Some("Alpha".to_string());
        let mut zeta = tagged("Zeta instance", &[], false, 0);
        zeta.group = Some("Zeta".to_string());
        let ungrouped = tagged("Ungrouped instance", &[], false, 0);
        let instances = vec![alpha, zeta, ungrouped];
        let filtered: Vec<usize> = (0..instances.len()).collect();
        let sections = modrinth_sections(
            &instances,
            &filtered,
            LibraryGroupBy::Group,
            &["Empty".to_string()],
            Language::English,
        );
        let titles: Vec<&str> = sections.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, vec!["Alpha", "Empty", "Zeta", "Ungrouped"]);
        assert_eq!(sections[0].indices, vec![0]);
        assert!(sections[1].indices.is_empty());
        assert_eq!(sections[2].indices, vec![1]);
        assert_eq!(sections[3].indices, vec![2]);
    }

    #[test]
    fn test_modrinth_loader_and_version_sections() {
        let mut fabric_old = tagged("Fabric old", &[], false, 0);
        fabric_old.game_version = "1.9".to_string();
        fabric_old.loader = LoaderType::Fabric;
        let mut forge = tagged("Forge", &[], false, 0);
        forge.game_version = "1.10".to_string();
        forge.loader = LoaderType::Forge;
        let mut fabric_new = tagged("Fabric new", &[], false, 0);
        fabric_new.game_version = "1.21".to_string();
        fabric_new.loader = LoaderType::Fabric;
        let instances = vec![fabric_old, forge, fabric_new];
        let filtered: Vec<usize> = (0..instances.len()).collect();

        let loaders = modrinth_sections(
            &instances,
            &filtered,
            LibraryGroupBy::Loader,
            &[],
            Language::English,
        );
        assert_eq!(loaders[0].title, "Fabric");
        assert_eq!(loaders[0].indices, vec![0, 2]);
        assert_eq!(loaders[1].title, "Forge");

        let versions = modrinth_sections(
            &instances,
            &filtered,
            LibraryGroupBy::GameVersion,
            &[],
            Language::English,
        );
        let titles: Vec<&str> = versions.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, vec!["1.21", "1.10", "1.9"]);
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
