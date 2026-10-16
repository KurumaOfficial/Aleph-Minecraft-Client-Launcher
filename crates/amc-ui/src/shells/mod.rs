//! Alternative launcher shells (CONCEPT "Режимы работы"): visual styles that
//! change how AMC's own buttons and panels are arranged around the shared
//! pages. Only layout ideas are borrowed from other launchers — no
//! third-party code, artwork, fonts or other assets are copied.
//!
//! Per-style credits (also repeated in each submodule):
//! - MultiMC Launcher — MultiMC Contributors, Apache-2.0 —
//!   <https://github.com/MultiMC/Launcher>
//! - Prism Launcher — PrismLauncher Contributors, GPL-3.0 —
//!   <https://github.com/PrismLauncher/PrismLauncher>
//! - Modrinth App (Theseus) — Modrinth / Theseus contributors, GPL-3.0 —
//!   <https://github.com/modrinth/code> (`apps/app-frontend`)
//! - Millida Launcher — Millida contributors, GPL-3.0 —
//!   <https://github.com/millida/launcher>
//! - Legacy Minecraft Launcher (Mojang Studios, proprietary): the layout is
//!   reconstructed from public screenshots only; no assets are reused and no
//!   source code is involved.
//!
//! Layout references studied (layout/hierarchy only):
//! - MultiMC/Prism: main toolbar on top (add instance, folders, settings,
//!   accounts), central instance list, status bar at the bottom.
//! - Modrinth App: left navigation sidebar, gray surfaces with a green
//!   accent, custom title bar.
//! - Millida: hero banner plus a lobby grid of tiles, content opens from a
//!   tile.
//! - Legacy: a row of plain text tabs on top and a Play panel at the bottom
//!   with the version selector.

pub mod legacy;
pub mod millida;
pub mod modrinth;
pub mod multimc;
pub mod ultra;

use crate::widgets::NavTab;
use amc_core::Language;

/// Localized page name for a shared `NavTab` (used by shell toolbars and
/// docks as button labels).
pub fn tab_label(tab: NavTab, lang: Language) -> &'static str {
    match tab {
        NavTab::Home => lang.nav_home(),
        NavTab::Modpacks => lang.nav_instances(),
        NavTab::Mods => lang.nav_mods(),
        NavTab::Skins => lang.nav_skins(),
        NavTab::Profile => lang.nav_profile(),
        NavTab::Settings => lang.nav_settings(),
    }
}

/// Actions a shell asks the app to perform. The app maps them onto its
/// existing handlers (tab switch, launch, login modal, exit), so shells
/// never touch launch or navigation state directly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShellAction {
    /// Switch the active page.
    pub tab: Option<NavTab>,
    /// Start the shared launch flow.
    pub launch: bool,
    /// Open the login/account modal.
    pub login: bool,
    /// Close the launcher window.
    pub exit: bool,
}

impl ShellAction {
    /// Merge `other` into `self`; earlier non-`None`/`false` values win.
    pub fn merge(&mut self, other: ShellAction) {
        if self.tab.is_none() {
            self.tab = other.tab;
        }
        self.launch |= other.launch;
        self.login |= other.login;
        self.exit |= other.exit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_action_merge_keeps_first_tab() {
        let mut base = ShellAction {
            tab: Some(NavTab::Home),
            ..ShellAction::default()
        };
        base.merge(ShellAction {
            tab: Some(NavTab::Settings),
            launch: true,
            ..ShellAction::default()
        });
        assert_eq!(base.tab, Some(NavTab::Home));
        assert!(base.launch);
    }

    #[test]
    fn test_shell_action_merge_fills_empty_fields() {
        let mut base = ShellAction::default();
        base.merge(ShellAction {
            tab: Some(NavTab::Modpacks),
            login: true,
            exit: true,
            ..ShellAction::default()
        });
        assert_eq!(base.tab, Some(NavTab::Modpacks));
        assert!(base.login);
        assert!(base.exit);
        assert!(!base.launch);
    }
}
