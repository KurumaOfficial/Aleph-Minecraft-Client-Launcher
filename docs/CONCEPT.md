# AMC Launcher — Concept (EN)

> **Living document, product source of truth.**
> Russian version: [CONCEPT.ru.md](CONCEPT.ru.md).
> Implementation plan: [ROADMAP-1.0.md](ROADMAP-1.0.md) / [ROADMAP-1.0.ru.md](ROADMAP-1.0.ru.md).
>
> - Synced with the working copy `AMC_Launcher_Concept.md` on 2026-09-14.
> - The concept **keeps evolving**: currently unclear points will be clarified in future iterations.
> - Update rules: edit this file (EN) mirrored with [CONCEPT.ru.md](CONCEPT.ru.md) (RU),
>   then update `ROADMAP-1.0*.md` if needed. Open questions live at the bottom.
>   Change history is tracked via git log.

---

*Part of the Aleph Trust ecosystem. Technologies: Rust + Egui + wgpu. Target hardware — down to GT710, Windows 7 or Linux.*

## Licensing
- Distribution model (open-source / closed) is not finalized yet.

## First launch: setup wizard
First-launch step order:

1. **UI language selection** (auto-detected from the system, changeable immediately).
2. **Familiar launcher choice** — which existing launcher feels closest to the user:
   - **Simple** (Legacy, TLauncher and similar) — minimal feature set.
   - **Advanced** (MultiMC, Modrinth App) — extended functionality.

   This choice affects the interface and partially the available functionality. It can be changed later at any time.
3. **Account sign-in offer** (see "Accounts"). WetID login/registration opens via browser (external page), not an embedded form.

First launch also runs a built-in hardware speed/compatibility test (keeping the GT710-level minimum in mind). If the hardware is weaker — the launcher suggests lightweight settings/mode (but does not block). The hardware test does not affect the simple/professional mode choice from step 2 — only the user picks the mode; these are independent things.

Simple and professional modes can be freely switched at any time from settings.

## Operation modes

### Simple mode
- Minecraft version selection, including snapshot/pre-release.
- "Quick start" button — instant vanilla launch for newcomers.
- Downloading mods right from the launcher via Modrinth or CurseForge.
- Search and install individual mods, not only full packs.
- Auto-suggestion of the current mod loader version (Forge/Fabric/etc.) when required.
- One-button update of the whole modpack.
- Porting a pack to another Minecraft version.

### Professional mode
- Multiple independent instances, like MultiMC / Prism Launcher.
- Quick-start templates (vanilla+, optimized, etc.) besides an empty instance.
- Instance customization: own icon and name per instance.
- Cloning (duplicating) an existing instance.
- Organizing many instances via folders/tags.

## Home screen
- Different screens for simple and professional modes (not one universal interface).

## Drag-and-drop install
- A mod or a whole pack can be installed by dragging a file into the launcher window.

## Offline mode
- After the first launch, offline play is possible (no internet).

## Mod loaders
- Support for all existing loaders (Forge, Fabric, Quilt, NeoForge, etc.) with auto-detection when installing mods/packs.
- Before installing a mod, the launcher checks and shows compatibility with the current Minecraft version/loader.
- Mod/pack search with filters: category, popularity, Minecraft version.

## Launcher builds
- Debug and release builds in code (single release channel for users, no separate stable/beta).

## Mod sources
- Built-in sources: only Modrinth and CurseForge. An external/local mod file can also be loaded manually. No custom repositories/sites support.

## Disk storage
- Configurable storage location for instances/mods (e.g. another drive).

## Instance storage
- No deduplication of shared files (assets/libraries) between instances — each instance is fully separate and self-contained.

## Logs and console
- Built-in game log/console viewer right in the launcher.

## Disk usage
- Per-instance/folder disk usage display.

## Java management
- Hybrid model: the launcher downloads and switches the required Java version automatically, but the user keeps a manual configuration option.
- If Java is already installed on the system — the launcher finds it (including known locations of other launchers, without fully importing their profiles) and offers to use it instead of downloading its own.

## Server list (server browser)
- Not implemented in v1.0. It is doubtful whether it is needed at all — it feels more like an advertising tool than a useful feature (players usually already know where they want to play).
- Without a shared browser, the user can still keep their server in favorites/quick access.
- A server added to launcher favorites automatically appears in the server list of every instance — no need to add it manually in-game.

## Supported OS
- Windows and Linux (macOS is not a priority for v1.0).

## Resource packs and shaders
- Built-in search and management from the launcher (like mods).

## Launcher auto-update
- The launcher updates itself automatically.

## Low-end hardware optimization
- For GT710-level and weaker hardware, the launcher automatically suggests installing optimization mods (like Sodium/Lithium).

## Mod/pack updates
- If an installed mod or pack has an update — the launcher notifies and offers a one-button update.
- If a pack update broke the game — one-button rollback to the previous state is available.
- The launcher does not sell/offer to buy an official licensed account — that is out of scope.

## Discover (trending packs)
- Section with popular/trending packs — inside the instances tab, not on the home screen (to avoid clutter).

## Download manager
- A full manager is needed: pause/resume, speed limit, queue, auto-retry on failure, progress/ETA per file.
- Parallel multi-file downloads for speed.

## Distribution format
- Both an installer and a portable version (no install).

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

## Co-op / Steam
- Instead of self-hosting/Open to LAN — Steam integration similar to a recently released mod: friend invites and co-op play via Steam.
- Implemented as a built-in launcher feature if possible, not a separate mod.
- Available to all players regardless of account type (license, WetID, custom nickname).
- No friends chat in the launcher itself — only in-game chat.

## Cloud sync
- Not implemented in v1.0. Planned for future versions (via WetID/cloud).

## News feed
- Undecided whether a news/articles feed is needed on the home screen.

## Screenshots
- Built-in in-game screenshot gallery.

## Multi-account on device
- Multiple licensed (Microsoft) accounts can be added on one device.
- WetID — only one account per person.

## Diagnostics and data
- Anonymous crash reports are sent automatically for problem diagnostics.
- On mod/dependency conflicts the launcher does not try to resolve the conflict automatically — the error is simply shown in logs after a crash.

## World (save) management
- Backup and restore worlds right from the launcher.
- Backups available both on schedule (automatically) and manually by button — user's choice.

## v1.0 boundaries (moved to 2.0)
- Aleph Guard integration / anticheat mode — not in v1.0. 2.0 will bring integrity checks, full device scanning and cheat countermeasures.
- Instance export/sharing with other users — not needed in v1.0.
- Idea for 2.0: if an instance contains a local (not from Modrinth/CurseForge) mod — entry to Aleph Trust servers is blocked by default, unless the server explicitly allowed that exact mod.

## Skin system
- Separate tab: pick a ready skin from a selection or upload your own file — all free, no built-in editor in v1.0.
- Goal — cover the maximum share of users on the maximum number of servers, so skins are visible to each other even without a license. Implementation will be "hacky" in places, but better than nothing.
- The share that cannot be covered globally will be covered by the launcher's own skin system — visible only to AMC Launcher users. This is not an artificial limitation, but a consequence of no other technical way.
- **No-license mechanics:** user picks a skin in the launcher → when joining a server with a skin plugin (e.g. via a command like `/skins <name>`) the launcher validates the skin via NameMC and sends the needed command from the client itself so the server applies the skin. For servers with other skin plugins/systems, separate workarounds will be invented — a set of hacks for maximum coverage.

## Accounts
The third wizard step offers three options:

1. **Official licensed account (Microsoft)** — allows joining licensed servers.
2. **WetID** — optional Aleph Trust ecosystem account. Without WetID sign-in, account features and Aleph Trust servers are unavailable.
3. **Continue without an account** — the user types a nickname to play under, changeable at any moment.

Combination logic:
- WetID-only sign-in → account functionality opens up (Aleph Trust servers, etc.).
- WetID + licensed account sign-in → on game launch the user can choose: play under a self-typed nickname (unlicensed) or under the licensed account (licensed servers access).
- The launcher asks for the choice (license / WetID / custom nickname) on every launch, but with a "remember choice" checkbox.
- Strictly one WetID account per person — binding several WetIDs to one device/launcher profile (e.g. for family members) is not allowed.

## Player profile
- Separate Minecraft profile settings tab: nickname, skin, cape.

---

## After game launch
- Launcher behavior is configurable: on weak PCs it can collapse into a minimal state with minimal system load (only needed background actions), or stay fully open — user's choice.
- Multiple instances can be launched simultaneously, including the same instance several times in parallel.

## Feedback
- No in-launcher "report a bug" button — feedback only via external site/GitHub.

## Open questions (for next iterations)
- [ ] Whether WetID account status (friends, trust, etc.) is shown in the launcher — undecided.
- [ ] Whether a news/articles feed is needed on the home screen — undecided.
- [ ] Custom UI themes — most likely not in v1.0, decision not final.
- [ ] Launcher licensing/distribution model — not finalized.
- [ ] Instance import/export format — not chosen (goal: .mrpack, CurseForge .zip, etc.).
- [ ] Whether a shared server browser is needed at all — doubtful (definitely not in v1.0).
