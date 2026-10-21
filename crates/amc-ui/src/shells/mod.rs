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

/// Cut a display name to `max` characters with a trailing ellipsis (shell
/// chrome: instance names in panels keep their row height instead of
/// wrapping).
pub fn short_name(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        name.to_string()
    } else {
        format!("{}…", name.chars().take(max).collect::<String>())
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
    /// Open the create-instance dialog (MultiMC "Add Instance").
    pub new_instance: bool,
    /// Open the edit dialog of the selected instance (MultiMC instance
    /// toolbar: the icon button, the name button, Change Group, Edit
    /// Instance and Edit Notes all open it in the reference).
    pub edit_instance: bool,
    /// Open the manage view of the selected instance (worlds, screenshots).
    pub manage: bool,
    /// Move the selected instance to the trash.
    pub delete_instance: bool,
    /// Clone the selected instance together with its worlds.
    pub clone_instance: bool,
    /// Create a desktop shortcut for the selected instance.
    pub shortcut_instance: bool,
    /// Switch the active account (MultiMC accounts menu).
    pub account: Option<uuid::Uuid>,
    /// Select an instance (Legacy profile dropdown, profile editor).
    pub select_instance: Option<uuid::Uuid>,
    /// Toggle the console overlay (Millida lobby corner).
    pub console: bool,
    /// Toggle the download manager (Millida lobby corner).
    pub downloads: bool,
    /// Toggle the screenshots gallery (Millida rail shortcut).
    pub gallery: bool,
}

impl ShellAction {
    /// Merge `other` into `self`; earlier non-`None`/`false` values win.
    pub fn merge(&mut self, other: ShellAction) {
        if self.tab.is_none() {
            self.tab = other.tab;
        }
        if self.account.is_none() {
            self.account = other.account;
        }
        if self.select_instance.is_none() {
            self.select_instance = other.select_instance;
        }
        self.console |= other.console;
        self.downloads |= other.downloads;
        self.gallery |= other.gallery;
        self.launch |= other.launch;
        self.login |= other.login;
        self.exit |= other.exit;
        self.new_instance |= other.new_instance;
        self.edit_instance |= other.edit_instance;
        self.manage |= other.manage;
        self.delete_instance |= other.delete_instance;
        self.clone_instance |= other.clone_instance;
        self.shortcut_instance |= other.shortcut_instance;
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
        let account = uuid::Uuid::new_v4();
        let instance = uuid::Uuid::new_v4();
        base.merge(ShellAction {
            tab: Some(NavTab::Modpacks),
            login: true,
            exit: true,
            new_instance: true,
            account: Some(account),
            select_instance: Some(instance),
            console: true,
            gallery: true,
            ..ShellAction::default()
        });
        assert_eq!(base.tab, Some(NavTab::Modpacks));
        assert_eq!(base.account, Some(account));
        assert_eq!(base.select_instance, Some(instance));
        assert!(base.console);
        assert!(base.gallery);
        assert!(base.login);
        assert!(base.exit);
        assert!(base.new_instance);
        assert!(!base.launch);
        assert!(!base.delete_instance);
    }

    #[test]
    fn test_short_name_truncates_with_ellipsis() {
        assert_eq!(short_name("Short", 10), "Short");
        let long = "a".repeat(40);
        let cut = short_name(&long, 30);
        assert_eq!(cut.chars().count(), 31);
        assert!(cut.ends_with('…'));
        // Multi-byte characters count as characters, not bytes.
        assert_eq!(short_name("Привет", 6), "Привет");
        assert_eq!(short_name("Привет", 3), "При…");
    }
}
