# AMC Launcher — Concept (EN)

> **Living document, product source of truth.**
> Russian version: [CONCEPT.ru.md](CONCEPT.ru.md).
> Implementation plan: [ROADMAP-1.0.md](ROADMAP-1.0.md) / [ROADMAP-1.0.ru.md](ROADMAP-1.0.ru.md).
>
> - Synced with the working copy `AMC_Launcher_Concept.md` on 2026-09-14.
> - 2026-09-30: decisions from the working copy that were lost in the first
>   sync are folded back in (wizard, modes, CLI, shortcuts, integrity, etc.).
>   No new decisions were made — completeness only.
> - The concept **keeps evolving**: currently unclear points will be clarified in future iterations.
> - Update rules: edit this file (EN) mirrored with [CONCEPT.ru.md](CONCEPT.ru.md) (RU),
>   then update `ROADMAP-1.0*.md` if needed. Open questions live at the bottom.
>   Change history is tracked via git log.

---

*Part of the Aleph Trust ecosystem. Technologies: Rust + Egui + wgpu. Target hardware — down to GT710, Windows 7 or Linux.*

## Licensing
- Distribution model (open-source / closed) is not finalized yet.

## Monetization
- No ads anywhere in the launcher (unlike TLauncher).
- Donations / developer support as the monetization path, but every launcher feature stays free.

## First launch: setup wizard
First-launch step order:

1. **UI language selection** (auto-detected from the system, changeable immediately).
2. **Familiar launcher choice** — which existing launcher feels closest to the user:
   - **Simple** (Legacy, TLauncher and similar) — minimal feature set.
   - **Advanced** (MultiMC, Modrinth App) — extended functionality.

   This choice affects the interface and partially the available functionality. It can be changed later at any time.
3. **Account sign-in offer** (see "Accounts"). WetID login/registration opens via browser (external page), not an embedded form.

The wizard is mandatory for everyone — skipping it straight into professional mode with defaults is not allowed. First launch also includes a tutorial tour / hints for newcomers.

First launch also runs a built-in hardware speed/compatibility test (keeping the GT710-level minimum in mind). The test can be re-run manually at any time later, not only at first setup. If the hardware is weaker — the launcher suggests lightweight settings/mode (but does not block). The hardware test does not affect the simple/professional mode choice from step 2 — only the user picks the mode; these are independent things.

If the PC already has a .minecraft folder from another launcher, AMC offers to copy the already downloaded files instead of re-downloading (saves traffic and time).

Simple and professional modes can be freely switched at any time from settings.

## Operation modes

### Simple mode
- Only one active setup at a time (no parallel ones — that is what professional mode with multiple instances is for).
- Minecraft version selection, including snapshot/pre-release as well as old alpha/beta/classic versions (matters for the Legacy style).
- "Quick start" button — instant vanilla launch for newcomers; the version is the last one the user played (not always the newest release).
- Downloading mods right from the launcher via Modrinth or CurseForge.
- Search and install individual mods, not only full packs.
- Auto-suggestion of the current mod loader version (Forge/Fabric/etc.) when required.
- One-button update of the whole modpack.
- Porting a pack to another Minecraft version.

### Professional mode
- Multiple independent instances, like MultiMC / Prism Launcher.
- Quick-start templates (vanilla+, optimized, etc.) besides an empty instance — AMC-team only, no user/community templates in v1.0.
- Instance customization: own icon (picture upload or a ready-made set) and name per instance.
- Cloning (duplicating) an existing instance — when cloning you can choose whether to copy worlds/saves or only mods/settings.
- Organizing many instances via folders/tags, sorting (by date/name/playtime), pinning favorites on top and quick search by name.
- Personal notes per instance (seed, who we play with, etc.).
- Pre-launch/post-exit scripts — custom commands before/after an instance launch, for advanced users.
- Several instances can be linked into a group with shared Java/RAM settings — a change applies to the whole group at once.

## Instance card
- Visible without clicking: icon, name, Minecraft version, loader, playtime, last launch date.

## Mod/pack search: ratings
- Search results show ratings/reviews pulled from Modrinth/CurseForge.

## Home screen
- Different screens for simple and professional modes (not one universal interface).

## Drag-and-drop install
- A mod or a whole pack can be installed by dragging a file into the launcher window.
- A ready world (save) can be imported into an instance by dragging its folder.

## Offline mode
- After the first launch, offline play is possible (no internet).

## Mod loaders
- Support for all existing loaders (Forge, Fabric, Quilt, NeoForge, etc.) with auto-detection when installing mods/packs.
- OptiFine is handled as a regular mod, not a separate loader. Code status: in the current development OptiFine is temporarily available as a loader choice (fast P2 path); in P4 it moves to mod-style installation onto a compatible instance.
- Before installing a mod, the launcher checks and shows compatibility with the current Minecraft version/loader.
- When creating an instance with an old/unsafe loader (e.g. an old Forge with known vulnerabilities), the launcher warns about it.
- Mod/pack search with filters: category, popularity, Minecraft version.
- Mods/resource packs can be added to a personal wishlist/favorites (shared across instances) without installing immediately.
- Several mods can be selected at once and updated/deleted in one batch action.
- Downloaded files are integrity-checked via hash sums.
- If a mod requires another mod as a dependency, the launcher installs the dependency automatically.
- If two installed mods are known to be incompatible, the launcher warns about it before launch.
- When installing a mod, similar/compatible mods are suggested ("frequently installed together").

## CLI mode
- A no-GUI mode (CLI) is needed for automation and server use.

## Launcher builds
- Debug and release builds in code (single release channel for users, no separate stable/beta).

## Mod sources
- Built-in sources: only Modrinth and CurseForge. An external/local mod file can also be loaded manually. No custom repositories/sites support.
- No security (antivirus) scanning of manually uploaded local mods — the user is responsible.
- Instances with local (non-marketplace) mods get a visual badge in professional mode — already in v1.0, preparing for Aleph Trust server access restrictions in 2.0.

## Disk storage
- Configurable storage location for instances/mods (e.g. another drive).

## Instance storage
- No deduplication of shared files (assets/libraries) between instances — each instance is fully separate and self-contained.

## Logs and console
- Built-in game log/console viewer right in the launcher.
- Copy/upload log button — for quickly sending it to support or friends.

## Shortcuts and launch
- Desktop / Start-menu shortcuts can be created for quick-launching a specific instance without opening the launcher.
- On laptops with two GPUs (integrated + discrete), the GPU used to run an instance can be chosen.
- Resolution and windowed/fullscreen mode are configured from the launcher separately per instance.
- While the game is running, the taskbar/tray shows the instance's own icon, not the stock Minecraft icon.

## Config editor
- Professional mode includes a built-in editor for mod config files.

## Disk usage
- Per-instance/folder disk usage display.
- Before downloading a large pack, the launcher checks free space and warns if it is short.

## Java management
- Hybrid model: the launcher downloads and switches the required Java version automatically, but the user keeps a manual configuration option.
- A dedicated Java manager screen listing all installed/downloaded versions (add/remove).
- If Java is already installed on the system — the launcher finds it (including known locations of other launchers, without fully importing their profiles) and offers to use it instead of downloading its own.

## Server-required mods
- If a server requires specific mods, joining shows a window with the needed mod list and an auto-install button — the launcher installs them itself from trusted sources.

## Server list (server browser)
- Not implemented in v1.0. It is doubtful whether it is needed at all — it feels more like an advertising tool than a useful feature (players usually already know where they want to play).
- Without a shared browser, the user can still keep their server in favorites/quick access.
- A server added to launcher favorites automatically appears in the server list of every instance — no need to add it manually in-game. There is no separate "play" button for a favorite server in the launcher itself — the server is simply pre-registered in the in-game multiplayer menu.
- Online status and ping of a favorite server are visible right in the launcher, without joining the game, including the online player list.

## Autostart
- The launcher can be configured to autostart with the OS.

## Supported OS
- Windows and Linux (macOS is not a priority for v1.0).
- The Windows installer checks for required system components (.NET/VC++ etc.) and installs the missing ones.

## Resource packs and shaders
- Built-in search and management from the launcher (like mods).
- The resource pack manager shows texture image previews.
- Search filter by texture resolution (16x/32x/64x etc.).
- Priority between several resource packs is not configured by dragging in the launcher — only the standard/manual way in files.
- Shader installation does not check OptiFine/Iris compatibility — it installs with no conflict warning.

## Launcher auto-update
- The launcher updates itself automatically.
- Update checks happen only at launcher startup, never periodically while running.
- After an update, a popup shows the changelog ("what's new").
- Before installing an update, its digital signature is verified (tamper protection).
- No hints for antivirus false positives on launcher/mod files — out of the launcher's responsibility.
- Rolling back to a previous launcher version is possible (the launcher is open-source), but the old version will not support "trusted launch mode". That mode is only needed to join Aleph Trust servers — on regular servers and offline the old version works fine. Updating is mandatory only when joining Aleph Trust servers specifically.
- "Trusted mode" is available only to official signed AMC builds — including the portable version and Linux builds, not just the Windows installer. A self-compiled build from sources works as a regular launcher for everything except Aleph Trust servers.
- Single release channel for everyone — no opt-in early access to launcher features.

## Low-end hardware optimization
- For GT710-level and weaker hardware, the launcher automatically suggests installing optimization mods (like Sodium/Lithium).
- Automatically raises the game process priority for better performance.
- Built-in launcher overlay with FPS and system load on top of the game.

## Mod/pack updates
- If an installed mod or pack has an update — the launcher notifies and offers a one-button update.
- If a pack update broke the game — one-button rollback to the previous state is available. An autobackup is made before every pack update automatically.
- An update shows the changelog: what was added/removed/updated.
- The launcher does not sell/offer to buy an official licensed account — that is out of scope.

## Discover (trending packs)
- Section with popular/trending packs — inside the instances tab, not on the home screen (to avoid clutter).

## Download manager
- A full manager is needed: pause/resume, speed limit, queue, auto-retry on failure, progress/ETA per file.
- Parallel multi-file downloads for speed.
- Contextual progress display: detailed per-file progress on the downloads page; when navigating to other launcher sections — a single shared progress bar at the bottom (Steam-style).

## Distribution format
- Both an installer and a portable version (no install).
- For Linux — several packaging formats (AppImage, deb, etc.), not one universal.
- The portable version keeps settings in its own program folder — can be moved to another PC (e.g. from a flash drive) with all settings.

## Unlicensed nicknames
- Conflicts of identical nicknames between unlicensed players on one server are resolved by the server itself, not the launcher.

## Instance import/export
- Format not chosen yet, but ideally — support for all common formats (Modrinth .mrpack, CurseForge, etc.).
- Mod versions in a pack can be pinned so they are not auto-updated.

## Import from other launchers
- Not required (TLauncher, MultiMC, etc. — no migration).

## Localization
- Russian, English, Ukrainian at launch.

## Interface theme
- Dark theme only.
- Custom themes are considered optional, but undecided — most likely there will be none. Reason: at the current stage the Aleph Studio-style UI visually "blends together", but there is a prototype of the same design where this is not a problem — treated as a temporary current-development issue, not a future decision.

## Play statistics
- Built-in statistics: hours played, etc., including per-instance.

## RAM/JVM settings
- Manual memory and JVM argument tuning is available in both modes (simple and professional).
- By default, instance RAM is auto-selected based on hardware.
- Besides JVM arguments, custom game launch arguments can be configured.
- Recommended JVM flags (like Aikar's flags) are not applied automatically — available manually on demand only.

## Co-op / Steam
- Instead of self-hosting/Open to LAN — Steam integration similar to a recently released mod: friend invites and co-op play via Steam.
- Implemented as a built-in launcher feature if possible, not a separate mod.
- Available to all players regardless of account type (license, WetID, custom nickname).
- No friends chat in the launcher itself — only in-game chat.
- Separately, a "create local server" button — launching your own dedicated server for a pack, not only via Steam invites. The server stops together with closing the game/launcher (does not run independently). Helping friends reach it from outside the LAN (tunneling/port forwarding) — undecided for now.

## Cloud sync
- Not implemented in v1.0. Planned for future versions (via WetID/cloud).
- In 2.0 everything syncs, including packs and mods, not just settings/skins/profile.
- In v1.0 there is a manual path: export/import of all launcher settings via file for moving to another PC (no cloud).

## News feed
- Undecided whether a news/articles feed is needed on the home screen.

## Screenshots
- Built-in in-game screenshot gallery.

## Multi-account on device
- Multiple licensed (Microsoft) accounts can be added on one device.
- WetID — only one account per person.

## Diagnostics and data
- Anonymous crash reports are sent automatically for problem diagnostics, but can be disabled in settings.
- On mod/dependency conflicts the launcher does not try to resolve the conflict automatically — the error is simply shown in logs after a crash.
- No automatic "safe mode" on repeated crashes — the user disables mods manually.
- Accessibility (font scaling, color-blind mode, etc.) is not in v1.0.

## Integrity and deletion
- A "verify/repair files" button per instance for when something is damaged or missing.
- An overall instance health score combining all checks (integrity, compatibility, conflicts, etc.) into one rating.
- Deleting an instance does not erase it immediately — it goes to trash with undo.
- When uninstalling the launcher itself — it asks whether to delete or keep the data (packs, worlds, settings).
- A full backup of an entire instance (mods+settings+worlds) can be made to an external drive for personal archive — separate from 2.0 pack sharing.

## World (save) management
- Backup and restore worlds right from the launcher.
- Backups available both on schedule (automatically) and manually by button — user's choice.
- The world backup folder can be chosen freely (e.g. a Dropbox/Google Drive synced folder).

## v1.0 boundaries (moved to 2.0)
- Aleph Guard integration / anticheat mode — not in v1.0. 2.0 will bring integrity checks, full device scanning and cheat countermeasures.
  - 2.0 device scanning works at kernel level, not just process level.
  - Cheat detection — both by known cheat-client/inject signatures and behavioral analysis.
  - Ban/punishment decisions on detected cheats are made by the server itself, not globally by Aleph Trust/the launcher.
  - Aleph Guard can be disabled when playing on regular/non-Aleph servers.
  - Server owners enable Aleph Guard themselves via plugin.
  - Aleph Guard/Aleph Trust are not free for server owners — subscription model (month/year). Whether pricing depends on server size (players/slots) and whether a free tier for small/non-commercial servers exists — undecided.
  - Server owners get an Aleph Trust web panel with analytics, bans and subscription management.
  - Ban appeals for Aleph Guard and a trial period before subscription — undecided.
  - A shared WetID ban base across network servers — undecided.
- Instance export/sharing with other users — not needed in v1.0. In 2.0 it is file-based (export/import) — manual file handoff via messenger/drive, no direct in-launcher sending to a friend.
- Cloud sync — not in v1.0. In 2.0 everything syncs, including packs and mods, not just settings/skins/profile.
- WetID friends system in 2.0: see which friends are online and on which server. Online/play status is always visible to friends, cannot be hidden. How friends are added (by nickname/ID or friend code) — undecided. Shared cross-server WetID reputation/trust level — undecided.
- Idea for 2.0: if an instance contains a local (not from Modrinth/CurseForge) mod — entry to Aleph Trust servers is blocked by default, unless the server explicitly allowed that exact mod.

## Skin system
- Separate tab: pick a ready skin from a selection or upload your own file — all free, no built-in editor in v1.0.
- No moderation/filtering of uploaded skins — anything can be uploaded.
- Goal — cover the maximum share of users on the maximum number of servers, so skins are visible to each other even without a license. Implementation will be "hacky" in places, but better than nothing.
- The share that cannot be covered globally will be covered by the launcher's own skin system — visible only to AMC Launcher users. This is not an artificial limitation, but a consequence of no other technical way.
- **No-license mechanics:** user picks a skin in the launcher → when joining a server with a skin plugin (e.g. via a command like `/skins <name>`) the launcher validates the skin via NameMC and sends the needed command from the client itself so the server applies the skin. For servers with other skin plugins/systems, separate workarounds will be invented — a set of hacks for maximum coverage.

## Accounts
The third wizard step offers three options:

1. **Official licensed account (Microsoft)** — allows joining licensed servers.
2. **WetID** — optional Aleph Trust ecosystem account. Without WetID sign-in, account features and Aleph Trust servers are unavailable. Supports two-factor authentication (2FA).
3. **Continue without an account** — the user types a nickname to play under, changeable at any moment.

Combination logic:
- WetID-only sign-in → account functionality opens up (Aleph Trust servers, etc.).
- WetID + licensed account sign-in → on game launch the user can choose: play under a self-typed nickname (unlicensed) or under the licensed account (licensed servers access).
- The launcher asks for the choice (license / WetID / custom nickname) on every launch, but with a "remember choice" checkbox.
- Strictly one WetID account per person — binding several WetIDs to one device/launcher profile (e.g. for family members) is not allowed.
- Each instance can be assigned a default account so it is not picked on every launch.

## Player profile
- Separate Minecraft profile settings tab: nickname, skin, cape.

---

## After game launch
- Launcher behavior is configurable: on weak PCs it can collapse into a minimal state with minimal system load (only needed background actions), or stay fully open — user's choice.
- Multiple instances can be launched simultaneously, including the same instance several times in parallel.
- RAM distribution across simultaneously launched instances is configurable — flexible (shared limit) or independent (each instance its own) consumption can be chosen.
- After exiting the game (not a crash), a session summary is shown (playtime, etc.).

## Hotkeys
- Custom hotkeys can be assigned in the launcher.
- Launcher UI navigation supports gamepad/controller, not just mouse and keyboard.

## Notifications
- Individual notification types (updates, etc.) can be enabled/disabled separately.

## Feedback
- No in-launcher "report a bug" button — feedback only via external site/GitHub.
- No built-in help/FAQ section either — external site only.
- No contextual hints/tooltips in simple mode — the interface is simple as is (except the first-launch tutorial tour).

## Open questions (for next iterations)
- [ ] Whether WetID account status (friends, trust, etc.) is shown in the launcher — undecided.
- [ ] Whether a news/articles feed is needed on the home screen — undecided.
- [ ] Custom UI themes — most likely not in v1.0, decision not final.
- [ ] Launcher licensing/distribution model — not finalized.
- [ ] Instance import/export format — not chosen (goal: .mrpack, CurseForge .zip, etc.).
- [ ] Whether a shared server browser is needed at all — doubtful (definitely not in v1.0).
- [ ] Whether a custom URL protocol (amc://) is needed for installing straight from Modrinth/CurseForge sites — undecided.
- [ ] Donation platform(s) (Boosty/Patreon-like, crypto, etc.) — undecided.
- [ ] Whether donors get a cosmetic profile badge — undecided.
- [ ] What WetID registration requires (email, nickname, password, etc.) — undecided.
