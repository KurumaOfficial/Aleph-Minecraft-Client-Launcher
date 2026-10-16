use crate::theme::{TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use crate::widgets::icons::{self, Icon};
use amc_core::types::Instance;
use amc_core::{HealthReport, Language};
use egui::{vec2, Color32, ScrollArea, TextEdit};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Per-instance management modal (CONCEPT P2 + P9): world backups, config
/// files and integrity live behind the wrench card button so the grid stays
/// glanceable. All operations are synchronous filesystem work; nothing
/// here touches `instances.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ManageTab {
    #[default]
    Worlds,
    Configs,
    Health,
    Archive,
}

#[derive(Default)]
pub struct InstanceManage {
    pub open: bool,
    pub instance_id: Option<Uuid>,
    tab: ManageTab,
    status: Option<String>,
    shown_for: Option<Uuid>,
    // Config editor state.
    cfg_selected: Option<PathBuf>,
    cfg_text: String,
    cfg_filter: String,
    cfg_find: String,
    cfg_visible: Vec<PathBuf>,
    cfg_visible_key: Option<(String, usize)>,
    cfg_hits: usize,
    cfg_hits_key: Option<(String, usize)>,
    // Cached directory listings (CONCEPT P10 60fps budget): directory
    // walks never run per frame, at most every few seconds or after a
    // mutation performed here.
    cfg_files: Vec<PathBuf>,
    cfg_lower: Vec<String>,
    saves_cache: Vec<String>,
    backups_cache: HashMap<String, Vec<PathBuf>>,
    archives_cache: Vec<PathBuf>,
    archives_sizes: HashMap<PathBuf, u64>,
    lists_at: Option<std::time::Instant>,
    lists_dirty: bool,
    // Health state.
    health: Option<HealthReport>,
}

impl InstanceManage {
    pub fn open_for(&mut self, id: Uuid) {
        self.open = true;
        self.instance_id = Some(id);
        self.status = None;
    }

    /// Editable config candidates: small text files under `config/` plus
    /// the root `options.txt`. Pure over (relative-path, size) for tests.
    fn is_editable(rel: &Path, size: u64) -> bool {
        if size > 256 * 1024 {
            return false;
        }
        if rel == Path::new("options.txt") {
            return true;
        }
        if !rel.starts_with("config") {
            return false;
        }
        matches!(
            rel.extension().and_then(|e| e.to_str()),
            Some("txt" | "cfg" | "json" | "toml" | "properties" | "yml" | "yaml" | "ini")
        )
    }

    fn list_configs(game_dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![game_dir.join("config")];
        if game_dir.join("options.txt").is_file() {
            out.push(PathBuf::from("options.txt"));
        }
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_symlink() {
                    continue;
                }
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let Ok(rel) = path.strip_prefix(game_dir).map(Path::to_path_buf) else {
                    continue;
                };
                let size = entry.metadata().map(|m| m.len()).unwrap_or(u64::MAX);
                if Self::is_editable(&rel, size) {
                    out.push(rel);
                }
            }
        }
        out.sort();
        out
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        instances: &[Instance],
        instances_dir: &Path,
        backups_root: &Path,
        lang: Language,
    ) {
        let Some(id) = self.instance_id else {
            self.open = false;
            return;
        };
        if !self.open {
            return;
        }
        let Some(inst) = instances.iter().find(|i| i.id == id).cloned() else {
            self.open = false;
            self.instance_id = None;
            return;
        };
        if self.shown_for != Some(id) {
            self.shown_for = Some(id);
            self.cfg_selected = None;
            self.cfg_text.clear();
            self.status = None;
            self.health = None;
            self.lists_dirty = true;
        }

        let game_dir = inst.get_game_dir(instances_dir);
        // Throttled listing refresh: directory walks never run per frame.
        let lists_stale = self
            .lists_at
            .is_none_or(|at| at.elapsed() >= std::time::Duration::from_secs(3));
        if self.lists_dirty || lists_stale {
            self.refresh_lists(&game_dir, backups_root, &inst.name);
        }
        let mut close = false;
        egui::Window::new(inst.name.clone())
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(640.0)
            .default_size(vec2(700.0, 520.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (ManageTab::Worlds, lang.worlds_title()),
                        (ManageTab::Configs, lang.editor_title()),
                        (ManageTab::Health, lang.health_title()),
                        (ManageTab::Archive, lang.archive_title()),
                    ] {
                        let active = self.tab == tab;
                        let text = if active {
                            egui::RichText::new(label).strong().color(TEXT_HEADING)
                        } else {
                            egui::RichText::new(label).color(TEXT_MUTED)
                        };
                        if ui.button(text).clicked() {
                            self.tab = tab;
                            self.status = None;
                        }
                    }
                });
                ui.add_space(8.0);
                match self.tab {
                    ManageTab::Worlds => self.show_worlds(ui, &inst, &game_dir, backups_root, lang),
                    ManageTab::Configs => self.show_configs(ui, &game_dir, lang),
                    ManageTab::Health => self.show_health(ui, &inst, instances_dir, lang),
                    ManageTab::Archive => {
                        self.show_archive(ui, &inst, &game_dir, backups_root, lang)
                    }
                }
                if let Some(msg) = self.status.as_ref() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(msg)
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(lang.home_drawer_close()).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            self.open = false;
        }
    }

    fn show_worlds(
        &mut self,
        ui: &mut egui::Ui,
        inst: &Instance,
        game_dir: &Path,
        backups_root: &Path,
        lang: Language,
    ) {
        if self.saves_cache.is_empty() {
            ui.label(
                egui::RichText::new(lang.world_no_saves())
                    .font(egui::FontId::proportional(13.0))
                    .color(TEXT_MUTED),
            );
            return;
        }
        ui.label(
            egui::RichText::new(lang.world_auto_label())
                .font(egui::FontId::proportional(11.0))
                .color(TEXT_MUTED),
        );
        ui.add_space(6.0);
        let saves_dir = game_dir.join("saves");
        let saves = self.saves_cache.clone();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for world in &saves {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(world)
                                .font(egui::FontId::proportional(13.0))
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(lang.world_backup_now()).clicked() {
                                match amc_core::backup_world(
                                    backups_root,
                                    &inst.name,
                                    world,
                                    &saves_dir,
                                ) {
                                    Ok(path) => {
                                        self.status = Some(path.to_string_lossy().to_string());
                                        self.lists_dirty = true;
                                    }
                                    Err(e) => {
                                        self.status = Some(lang.err_backup_failed(&e.to_string()));
                                    }
                                }
                            }
                        });
                    });

                    let no_backups;
                    let backups: Vec<PathBuf> = match self.backups_cache.get(world) {
                        Some(cached) => {
                            no_backups = cached.is_empty();
                            cached.clone()
                        }
                        None => {
                            no_backups = true;
                            Vec::new()
                        }
                    };
                    if no_backups {
                        ui.label(
                            egui::RichText::new(format!("  {}", lang.world_no_backups()))
                                .font(egui::FontId::proportional(11.0))
                                .color(TEXT_MUTED),
                        );
                    }
                    let mut restore: Option<PathBuf> = None;
                    let mut delete: Option<PathBuf> = None;
                    for backup in &backups {
                        let label = backup.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("  {label}"))
                                    .font(egui::FontId::monospace(11.0))
                                    .color(TEXT_MUTED),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button(lang.world_delete()).clicked() {
                                        delete = Some(backup.clone());
                                    }
                                    if ui.small_button(lang.world_restore()).clicked() {
                                        restore = Some(backup.clone());
                                    }
                                },
                            );
                        });
                    }
                    if let Some(path) = delete {
                        match amc_core::delete_backup(&path) {
                            Ok(()) => {
                                self.status = Some(path.to_string_lossy().to_string());
                                self.lists_dirty = true;
                            }
                            Err(e) => self.status = Some(lang.err_delete_failed(&e.to_string())),
                        }
                    }
                    if let Some(path) = restore {
                        match amc_core::restore_world(&path, &saves_dir, world) {
                            Ok(()) => {
                                self.status = Some(format!("{}: {world}", lang.editor_saved()));
                                self.lists_dirty = true;
                            }
                            Err(e) => self.status = Some(lang.err_restore_failed(&e.to_string())),
                        }
                    }
                    ui.add_space(4.0);
                }
            });
    }

    /// Re-scan configs, saves, backups and archives in one pass.
    fn refresh_lists(&mut self, game_dir: &Path, backups_root: &Path, inst_name: &str) {
        self.cfg_files = Self::list_configs(game_dir);
        self.cfg_lower = self
            .cfg_files
            .iter()
            .map(|p| p.to_string_lossy().to_lowercase())
            .collect();
        self.cfg_visible_key = None;
        self.cfg_hits_key = None;
        // Keep the selected file only if it still exists.
        if let Some(selected) = self.cfg_selected.clone() {
            if !self.cfg_files.contains(&selected) {
                self.cfg_selected = None;
                self.cfg_text.clear();
            }
        }
        self.saves_cache = amc_core::list_saves(game_dir);
        self.backups_cache = self
            .saves_cache
            .iter()
            .map(|world| {
                (
                    world.clone(),
                    amc_core::list_backups(backups_root, inst_name, world),
                )
            })
            .collect();
        self.archives_cache = list_archives(backups_root, inst_name);
        self.archives_sizes = self
            .archives_cache
            .iter()
            .map(|p| {
                (
                    p.clone(),
                    std::fs::metadata(p).map(|m| m.len()).unwrap_or(0),
                )
            })
            .collect();
        self.lists_at = Some(std::time::Instant::now());
        self.lists_dirty = false;
    }

    fn show_configs(&mut self, ui: &mut egui::Ui, game_dir: &Path, lang: Language) {
        if self.cfg_files.is_empty() {
            ui.label(
                egui::RichText::new(lang.editor_empty())
                    .font(egui::FontId::proportional(13.0))
                    .color(TEXT_MUTED),
            );
            return;
        }
        // File filter (CONCEPT P2 config editor search).
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(14.0), egui::Sense::hover());
            icons::draw(ui.painter(), Icon::Search, rect, TEXT_MUTED, 1.3);
            ui.add(
                TextEdit::singleline(&mut self.cfg_filter)
                    .hint_text(lang.editor_filter_hint())
                    .desired_width(200.0),
            );
        });
        ui.add_space(4.0);
        // Memoized file filter over pre-lowercased names (no per-file
        // `to_string_lossy().to_lowercase()` per frame).
        let needle = self.cfg_filter.trim().to_lowercase();
        let vis_key = (needle.clone(), self.cfg_files.len());
        if self.cfg_visible_key.as_ref() != Some(&vis_key) {
            if needle.is_empty() {
                self.cfg_visible = self.cfg_files.clone();
            } else {
                self.cfg_visible = self
                    .cfg_files
                    .iter()
                    .zip(self.cfg_lower.iter())
                    .filter(|(_, lower)| lower.contains(&needle))
                    .map(|(rel, _)| rel.clone())
                    .collect();
            }
            self.cfg_visible_key = Some(vis_key);
        }
        let visible = self.cfg_visible.clone();
        ui.horizontal(|ui| {
            // File list.
            ScrollArea::vertical()
                .id_salt("cfg_list")
                .auto_shrink([false, false])
                .max_height(320.0)
                .show(ui, |ui| {
                    ui.set_min_width(180.0);
                    for rel in &visible {
                        let selected = self.cfg_selected.as_ref() == Some(rel);
                        let label = rel.to_string_lossy().to_string();
                        let text = if selected {
                            egui::RichText::new(label).strong().color(TEXT_HEADING)
                        } else {
                            egui::RichText::new(label).color(TEXT_MUTED)
                        };
                        if ui.button(text).clicked() {
                            self.cfg_selected = Some(rel.clone());
                            self.cfg_text =
                                std::fs::read_to_string(game_dir.join(rel)).unwrap_or_default();
                            self.status = None;
                        }
                    }
                });
            // Editor.
            ui.vertical(|ui| {
                if let Some(rel) = self.cfg_selected.clone() {
                    ui.label(
                        egui::RichText::new(rel.to_string_lossy())
                            .font(egui::FontId::monospace(11.0))
                            .color(TEXT_MUTED),
                    );
                    // In-content search: match counter, no cursor jumps —
                    // egui multiline has no search API in v1.0 scope.
                    // Hits memoized by (find, text_len); the full
                    // `to_lowercase` over up to 256KB runs only on change.
                    ui.horizontal(|ui| {
                        let (find_rect, _) =
                            ui.allocate_exact_size(vec2(12.0, 12.0), egui::Sense::hover());
                        icons::draw(ui.painter(), Icon::Search, find_rect, TEXT_MUTED, 1.3);
                        ui.add(
                            TextEdit::singleline(&mut self.cfg_find)
                                .hint_text(lang.editor_find_hint())
                                .desired_width(160.0),
                        );
                        let find = self.cfg_find.trim().to_lowercase();
                        if !find.is_empty() {
                            let hits_key = (find.clone(), self.cfg_text.len());
                            if self.cfg_hits_key.as_ref() != Some(&hits_key) {
                                self.cfg_hits = self.cfg_text.to_lowercase().matches(&find).count();
                                self.cfg_hits_key = Some(hits_key);
                            }
                            let hits = self.cfg_hits;
                            ui.label(
                                egui::RichText::new(lang.editor_matches(hits))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(TEXT_MUTED),
                            );
                        }
                    });
                    ScrollArea::vertical()
                        .id_salt("cfg_edit")
                        .auto_shrink([false, false])
                        .max_height(280.0)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.cfg_text)
                                    .font(egui::FontId::monospace(12.0))
                                    .desired_width(f32::INFINITY)
                                    .code_editor(),
                            );
                        });
                    if ui.button(lang.inst_modal_btn_save()).clicked() {
                        match std::fs::write(game_dir.join(&rel), &self.cfg_text) {
                            Ok(()) => {
                                self.status = Some(lang.editor_saved().to_string());
                                self.lists_dirty = true;
                            }
                            Err(e) => {
                                self.status = Some(lang.err_save_failed(&e.to_string()));
                            }
                        }
                    }
                }
            });
        });
    }

    fn show_archive(
        &mut self,
        ui: &mut egui::Ui,
        inst: &Instance,
        game_dir: &Path,
        backups_root: &Path,
        lang: Language,
    ) {
        ui.label(
            egui::RichText::new(lang.archive_desc())
                .font(egui::FontId::proportional(12.0))
                .color(TEXT_MUTED),
        );
        ui.add_space(8.0);
        if ui.button(lang.archive_create()).clicked() {
            match archive_instance(game_dir, backups_root, &inst.name) {
                Ok((path, skipped)) => {
                    let mut msg = path.to_string_lossy().to_string();
                    if skipped > 0 {
                        msg.push_str(&format!(" ({skipped} skipped)"));
                    }
                    self.status = Some(msg);
                    self.lists_dirty = true;
                }
                Err(e) => self.status = Some(lang.err_archive_failed(&e.to_string())),
            }
        }
        ui.add_space(8.0);
        let mut delete: Option<PathBuf> = None;
        for archive in &self.archives_cache.clone() {
            let label = archive.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let size = self.archives_sizes.get(archive).copied().unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{label} ({})", lang.disk_size(size)))
                        .font(egui::FontId::monospace(11.0))
                        .color(TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(lang.world_delete()).clicked() {
                        delete = Some(archive.clone());
                    }
                });
            });
        }
        if let Some(path) = delete {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    self.status = Some(path.to_string_lossy().to_string());
                    self.lists_dirty = true;
                }
                Err(e) => self.status = Some(lang.err_delete_failed(&e.to_string())),
            }
        }
    }

    fn show_health(
        &mut self,
        ui: &mut egui::Ui,
        inst: &Instance,
        instances_dir: &Path,
        lang: Language,
    ) {
        if self.health.is_none() {
            self.health = Some(HealthReport::verify(instances_dir, inst));
        }
        // Re-borrow after the lazy verify above.
        let (score, all_ok, checks) = match &self.health {
            Some(report) => (report.score(), report.all_ok(), report.checks.clone()),
            None => (0, false, Vec::new()),
        };
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{score}/100"))
                    .font(egui::FontId::proportional(26.0))
                    .strong()
                    .color(if all_ok {
                        Color32::from_rgb(0x50, 0xB0, 0x50)
                    } else {
                        Color32::from_rgb(0xE8, 0x55, 0x70)
                    }),
            );
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(lang.health_title())
                        .font(egui::FontId::proportional(14.0))
                        .strong()
                        .color(TEXT_HEADING),
                );
                if all_ok {
                    ui.label(
                        egui::RichText::new(lang.health_ok())
                            .font(egui::FontId::proportional(12.0))
                            .color(TEXT_MUTED),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(lang.health_repair()).clicked() {
                    let done = HealthReport::repair(instances_dir, inst);
                    self.status = if done.is_empty() {
                        Some(lang.health_ok().to_string())
                    } else {
                        Some(done.join("\n"))
                    };
                    self.health = Some(HealthReport::verify(instances_dir, inst));
                }
                if ui.button(lang.health_verify()).clicked() {
                    self.health = Some(HealthReport::verify(instances_dir, inst));
                    self.status = None;
                }
            });
        });
        ui.add_space(8.0);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for check in &checks {
                    let color = if check.ok {
                        TEXT_MUTED.into()
                    } else {
                        Color32::from_rgb(0xE8, 0x55, 0x70)
                    };
                    let mut text = check.name.clone();
                    if !check.detail.is_empty() {
                        text.push_str(&format!(" — {}", check.detail));
                    }
                    ui.horizontal(|ui| {
                        let (state_rect, _) =
                            ui.allocate_exact_size(vec2(12.0, 12.0), egui::Sense::hover());
                        icons::draw(
                            ui.painter(),
                            if check.ok { Icon::Check } else { Icon::Close },
                            state_rect,
                            color,
                            1.3,
                        );
                        ui.label(
                            egui::RichText::new(&text)
                                .font(egui::FontId::monospace(11.0))
                                .color(color),
                        );
                    });
                }
            });
    }
}

/// Zip a whole instance game dir for personal archiving (CONCEPT
/// "бэкап сборки на диск", P9): `backups_root/instance-archives/`.
/// Best-effort per file (a running game may lock files on Windows):
/// unreadable files are counted as skipped, never fatal.
/// Returns the zip path plus the skipped count.
pub fn archive_instance(
    game_dir: &Path,
    backups_root: &Path,
    name: &str,
) -> Result<(PathBuf, usize), String> {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim();
    let stem = if stem.is_empty() { "instance" } else { stem };
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let out_dir = backups_root.join("instance-archives");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("Backup dir failed: {e}"))?;
    let dest = out_dir.join(format!("{stem}-{stamp}.zip"));

    let file = std::fs::File::create(&dest).map_err(|e| format!("Zip create failed: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut skipped = 0usize;
    let mut stack = vec![game_dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_symlink() {
                continue;
            }
            let rel = match path.strip_prefix(game_dir) {
                Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            if path.is_dir() {
                if zip.add_directory(rel, opts).is_err() {
                    skipped += 1;
                } else {
                    stack.push(path);
                }
            } else if path.is_file() {
                match std::fs::File::open(&path) {
                    Ok(mut src) => {
                        if zip.start_file(rel, opts).is_ok() {
                            if std::io::copy(&mut src, &mut zip).is_err() {
                                skipped += 1;
                            }
                        } else {
                            skipped += 1;
                        }
                    }
                    Err(_) => skipped += 1,
                }
            }
        }
    }
    zip.finish()
        .map_err(|e| format!("Zip finish failed: {e}"))?;
    Ok((dest, skipped))
}

/// Archives of one instance (name-prefixed zips), newest first.
pub fn list_archives(backups_root: &Path, name: &str) -> Vec<PathBuf> {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let prefix = format!("{}-", stem.trim());
    let dir = backups_root.join("instance-archives");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|e| e == "zip").unwrap_or(false)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&prefix))
                    .unwrap_or(false)
        })
        .collect();
    out.sort();
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::{archive_instance, list_archives, InstanceManage};
    use std::path::Path;

    #[test]
    fn test_is_editable() {
        assert!(InstanceManage::is_editable(Path::new("options.txt"), 100));
        assert!(InstanceManage::is_editable(
            Path::new("config/mod.toml"),
            100
        ));
        assert!(!InstanceManage::is_editable(
            Path::new("config/big.json"),
            300 * 1024
        ));
        assert!(!InstanceManage::is_editable(Path::new("mods/a.jar"), 100));
        assert!(!InstanceManage::is_editable(Path::new("servers.dat"), 100));
    }

    #[test]
    fn test_archive_roundtrip() {
        let root = std::env::temp_dir().join(format!(
            "amc_archive_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&root);
        let game = root.join("game");
        std::fs::create_dir_all(game.join("saves/world")).unwrap();
        std::fs::create_dir_all(game.join("mods")).unwrap();
        std::fs::write(game.join("saves/world/level.dat"), b"fake").unwrap();
        std::fs::write(game.join("mods/m.jar"), b"fake").unwrap();
        let backups = root.join("backups");

        let (zip_path, skipped) = archive_instance(&game, &backups, "My Pack/1").unwrap();
        assert_eq!(skipped, 0);
        assert!(zip_path.is_file());
        // Only this instance's archives list.
        std::fs::write(backups.join("instance-archives/other-0.zip"), b"x").unwrap();
        let listed = list_archives(&backups, "My Pack/1");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0], zip_path);
        // Zip contents survive.
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
            .collect();
        names.sort();
        assert!(names.contains(&"mods/m.jar".to_string()));
        assert!(names.contains(&"saves/world/level.dat".to_string()));
        let _ = std::fs::remove_dir_all(&root);
    }
}
