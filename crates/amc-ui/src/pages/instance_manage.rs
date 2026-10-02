use crate::theme::{TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use amc_core::types::Instance;
use amc_core::{HealthReport, Language};
use egui::{vec2, Color32, ScrollArea, TextEdit};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Per-instance management modal (CONCEPT P2 + P9): world backups, config
/// files and integrity live behind the 🛠 card button so the grid stays
/// glanceable. All operations are synchronous filesystem work; nothing
/// here touches `instances.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ManageTab {
    #[default]
    Worlds,
    Configs,
    Health,
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
        }

        let game_dir = inst.get_game_dir(instances_dir);
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
                }
                if let Some(msg) = self.status.clone() {
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
        let saves = amc_core::list_saves(game_dir);
        if saves.is_empty() {
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
                                    }
                                    Err(e) => {
                                        self.status = Some(format!("Backup failed: {e}"));
                                    }
                                }
                            }
                        });
                    });

                    let backups = amc_core::list_backups(backups_root, &inst.name, world);
                    if backups.is_empty() {
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
                            }
                            Err(e) => self.status = Some(format!("Delete failed: {e}")),
                        }
                    }
                    if let Some(path) = restore {
                        match amc_core::restore_world(&path, &saves_dir, world) {
                            Ok(()) => {
                                self.status = Some(format!("{}: {world}", lang.editor_saved()));
                            }
                            Err(e) => self.status = Some(format!("Restore failed: {e}")),
                        }
                    }
                    ui.add_space(4.0);
                }
            });
    }

    fn show_configs(&mut self, ui: &mut egui::Ui, game_dir: &Path, lang: Language) {
        let files = Self::list_configs(game_dir);
        if files.is_empty() {
            ui.label(
                egui::RichText::new(lang.editor_empty())
                    .font(egui::FontId::proportional(13.0))
                    .color(TEXT_MUTED),
            );
            return;
        }
        ui.horizontal(|ui| {
            // File list.
            ScrollArea::vertical()
                .id_salt("cfg_list")
                .auto_shrink([false, false])
                .max_height(320.0)
                .show(ui, |ui| {
                    ui.set_min_width(180.0);
                    for rel in &files {
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
                            }
                            Err(e) => {
                                self.status = Some(format!("Save failed: {e}"));
                            }
                        }
                    }
                }
            });
        });
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
                    let dot = if check.ok { "✓" } else { "✗" };
                    let color = if check.ok {
                        TEXT_MUTED
                    } else {
                        Color32::from_rgb(0xE8, 0x55, 0x70)
                    };
                    let mut text = format!("{dot} {}", check.name);
                    if !check.detail.is_empty() {
                        text.push_str(&format!(" — {}", check.detail));
                    }
                    ui.label(
                        egui::RichText::new(text)
                            .font(egui::FontId::monospace(11.0))
                            .color(color),
                    );
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceManage;
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
}
