use crate::theme::{RUBY_LIGHT, TEXT_HEADING, TEXT_MUTED, TEXT_PRIMARY};
use crate::widgets::choice_chip;
use amc_auth::{login_offline, Account};
use amc_core::{AppMode, HardwareReport, Language};
use egui::{vec2, Ui};

/// First-run wizard steps (CONCEPT "Первый запуск", ROADMAP P1).
/// The wizard is mandatory but never blocking: every step has a preselected
/// default, so a clean profile completes it in three clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardStep {
    #[default]
    Language,
    Mode,
    Account,
}

/// Outcome of one wizard frame. The app (`LauncherApp`) owns persistence
/// (config save) and side effects (account storage, login modal).
/// Deliberately not `PartialEq`: it can carry an `Account`, and comparing
/// stored secrets is a bad idea — tests use `matches!` instead.
#[derive(Debug, Clone)]
pub enum WizardAction {
    None,
    CreateOffline(Account),
    OpenMicrosoftLogin,
    Finished,
}

pub struct WizardFlow {
    pub step: WizardStep,
    pub language: Language,
    pub mode: AppMode,
    pub nickname: String,
    pub nickname_invalid: bool,
}

impl WizardFlow {
    pub fn new(language: Language, mode: AppMode) -> Self {
        Self {
            step: WizardStep::Language,
            language,
            mode,
            nickname: String::new(),
            nickname_invalid: false,
        }
    }

    pub fn next(&mut self) {
        self.step = match self.step {
            WizardStep::Language => WizardStep::Mode,
            WizardStep::Mode | WizardStep::Account => WizardStep::Account,
        };
    }

    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Language | WizardStep::Mode => WizardStep::Language,
            WizardStep::Account => WizardStep::Mode,
        };
    }

    /// Offline-row "Continue": empty nickname means "decide later" and simply
    /// finishes the wizard; a valid nickname builds the account (the app
    /// persists it); an invalid one flags the inline error.
    pub fn submit_account(&mut self) -> WizardAction {
        let name = self.nickname.trim().to_string();
        if name.is_empty() {
            return WizardAction::Finished;
        }
        match login_offline(&name) {
            Ok(account) => WizardAction::CreateOffline(account),
            Err(_) => {
                self.nickname_invalid = true;
                WizardAction::None
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> WizardAction {
        let mut action = WizardAction::None;
        let lang = self.language;
        egui::Window::new(lang.wizard_title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .min_width(480.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                self.show_steps(ui, lang);
                ui.add_space(12.0);
                match self.step {
                    WizardStep::Language => self.show_language(ui),
                    WizardStep::Mode => self.show_mode(ui, lang),
                    WizardStep::Account => {
                        if let Some(a) = self.show_account(ui, lang) {
                            action = a;
                        }
                    }
                }
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if self.step != WizardStep::Language && ui.button(lang.wizard_back()).clicked()
                    {
                        self.back();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.step != WizardStep::Account
                            && ui.button(lang.wizard_next()).clicked()
                        {
                            self.next();
                        }
                    });
                });
            });
        action
    }

    fn show_steps(&self, ui: &mut Ui, lang: Language) {
        let steps = [
            (WizardStep::Language, lang.wizard_step_language()),
            (WizardStep::Mode, lang.wizard_step_mode()),
            (WizardStep::Account, lang.wizard_step_account()),
        ];
        ui.horizontal(|ui| {
            for (i, (step, label)) in steps.iter().enumerate() {
                if i > 0 {
                    ui.label(egui::RichText::new("›").color(TEXT_MUTED));
                }
                let active = *step == self.step;
                ui.label(egui::RichText::new(*label).color(if active {
                    RUBY_LIGHT
                } else {
                    TEXT_MUTED
                }));
            }
        });
    }

    fn show_language(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for l in Language::ALL {
                if choice_chip(
                    ui,
                    l.display_with_flag(),
                    130.0,
                    34.0,
                    12.0,
                    self.language == l,
                ) {
                    self.language = l;
                }
            }
        });
    }

    fn show_mode(&mut self, ui: &mut Ui, lang: Language) {
        for mode in AppMode::ALL {
            let (name, desc) = match mode {
                AppMode::Simple => (
                    lang.settings_mode_simple(),
                    lang.settings_mode_simple_desc(),
                ),
                AppMode::Professional => (lang.settings_mode_pro(), lang.settings_mode_pro_desc()),
            };
            ui.horizontal(|ui| {
                if choice_chip(ui, name, 170.0, 36.0, 12.5, self.mode == mode) {
                    self.mode = mode;
                }
                ui.label(egui::RichText::new(desc).color(TEXT_MUTED));
            });
            ui.add_space(6.0);
        }
    }

    fn show_account(&mut self, ui: &mut Ui, lang: Language) -> Option<WizardAction> {
        let mut action = None;

        // Microsoft: licensed servers, handled by the existing login modal.
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui.button(lang.wizard_ms_title()).clicked() {
                    action = Some(WizardAction::OpenMicrosoftLogin);
                }
                ui.label(egui::RichText::new(lang.wizard_ms_desc()).color(TEXT_MUTED));
            });
        });
        ui.add_space(8.0);

        // Offline nickname: validated locally, no network involved.
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(lang.wizard_offline_title())
                    .strong()
                    .color(TEXT_HEADING),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.nickname)
                        .hint_text(lang.wizard_offline_hint())
                        .desired_width(220.0),
                );
                if ui.button(lang.wizard_next()).clicked() {
                    action = Some(self.submit_account());
                }
            });
            if self.nickname_invalid {
                ui.label(egui::RichText::new(lang.wizard_offline_invalid()).color(RUBY_LIGHT));
            } else {
                ui.label(egui::RichText::new(lang.wizard_offline_hint()).color(TEXT_MUTED));
            }
        });
        ui.add_space(8.0);

        // WetID via external browser page arrives in P6 — shown, not offered.
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.wizard_wetid_title())
                        .strong()
                        .color(TEXT_MUTED),
                );
                ui.label(egui::RichText::new(lang.wizard_wetid_soon()).color(TEXT_MUTED));
            });
        });

        action
    }
}

/// One-time weak-hardware notice shown right after the wizard.
/// Returns true once the user dismisses it.
pub fn show_hardware_notice(ctx: &egui::Context, lang: Language, report: &HardwareReport) -> bool {
    let mut dismissed = false;
    let ram_text = report
        .total_ram_mb
        .map(|mb| format!("{} GB", mb / 1024))
        .unwrap_or_else(|| "?".to_string());
    let disk_text = report
        .free_disk_mb
        .map(|mb| {
            if mb >= 1024 {
                format!("{} GB", mb / 1024)
            } else {
                format!("{mb} MB")
            }
        })
        .unwrap_or_else(|| "?".to_string());
    egui::Window::new(lang.wizard_hw_title())
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .min_width(440.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(lang.wizard_hw_body()).color(TEXT_PRIMARY));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.wizard_hw_specs(
                    &ram_text,
                    report.cpu_threads,
                    &disk_text,
                ))
                .color(TEXT_MUTED),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(lang.wizard_hw_ok()).clicked() {
                        dismissed = true;
                    }
                });
            });
        });
    dismissed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow() -> WizardFlow {
        WizardFlow::new(Language::English, AppMode::Simple)
    }

    #[test]
    fn test_step_navigation() {
        let mut flow = flow();
        assert_eq!(flow.step, WizardStep::Language);
        flow.back();
        assert_eq!(flow.step, WizardStep::Language); // clamped at start
        flow.next();
        assert_eq!(flow.step, WizardStep::Mode);
        flow.next();
        assert_eq!(flow.step, WizardStep::Account);
        flow.next();
        assert_eq!(flow.step, WizardStep::Account); // clamped at end
        flow.back();
        assert_eq!(flow.step, WizardStep::Mode);
        flow.back();
        assert_eq!(flow.step, WizardStep::Language);
    }

    #[test]
    fn test_submit_empty_nickname_finishes() {
        let mut flow = flow();
        flow.step = WizardStep::Account;
        flow.nickname = "   ".to_string();
        assert!(matches!(flow.submit_account(), WizardAction::Finished));
        assert!(!flow.nickname_invalid);
    }

    #[test]
    fn test_submit_invalid_nickname_flags_error() {
        let mut flow = flow();
        flow.nickname = "ab".to_string();
        assert!(matches!(flow.submit_account(), WizardAction::None));
        assert!(flow.nickname_invalid);
    }

    #[test]
    fn test_submit_valid_nickname_builds_account() {
        let mut flow = flow();
        flow.nickname = "Steve_99".to_string();
        match flow.submit_account() {
            WizardAction::CreateOffline(account) => {
                assert_eq!(account.username, "Steve_99");
            }
            other => panic!("expected CreateOffline, got {other:?}"),
        }
        assert!(!flow.nickname_invalid);
    }
}
