# AMC Launcher — Roadmap to v1.0 (EN)

> **Living document. Product source:** [CONCEPT.md](CONCEPT.md) / [CONCEPT.ru.md](CONCEPT.ru.md).
> Russian version of this plan: [ROADMAP-1.0.ru.md](ROADMAP-1.0.ru.md).
>
> - Snapshot date: 2026-09-14. Codebase: `main` @ `1ba4cf1`
>   (`feat(ui): implement smooth reactive animations, aleph.icu aesthetic, ...`).
> - The concept **keeps evolving** — every concept change triggers a review of this plan
>   (see section 7 "Update process").
> - Current repo layout: [ARCH_SPEC.md](../ARCH_SPEC.md), `crates/` (6 crates), `src/main.rs`.

## 0. How to read this document

Item statuses:

- `✅ Done` — implemented in code at `1ba4cf1`, covered by manual/auto tests.
- `🟡 Partial` — skeleton exists but does not cover the whole concept item.
- `⬜ Todo` — concept item with no work done yet.
- `🚫 Out` — deliberately NOT in v1.0 (see section 1 "Boundaries").
- `❓ Open` — undecided, waits for a concept update.

Phase priority: `P0` — foundation (this commit), `P1–P3` — critical launch path,
`P4–P8` — core value, `P9–P13` — finishing to "1.0".

## 1. What v1.0 is

### 1.1. Goal

A fast (cold boot <300 ms, ~35 MB idle), modular (6 crates), open-source Rust launcher
(`egui`/`eframe` + `wgpu`/`glow`) that takes a newcomer from first launch to gameplay with one button,
and gives advanced users MultiMC/Prism-grade instances — on hardware down to GT710, on Windows and Linux,
with zero telemetry (network only: Mojang, Adoptium, Modrinth, CurseForge).

### 1.2. Technical frame (fixed for 1.0)

- Rust 2021 workspace: `amc-core`, `amc-auth`, `amc-downloader`, `amc-minecraft`, `amc-mods`, `amc-ui` + `src/main.rs`.
- GUI: `egui 0.29` + `eframe` (`wgpu` with `glow`/OpenGL 3.3+ fallback), 60–144 FPS, Aleph dark theme (`#080606` / `#8B1A2A`).
- Single release channel for users (debug/release are only build profiles, no stable/beta).
- Each instance is self-contained: no assets/libraries deduplication between instances.
- Feedback only via external site/GitHub, no in-app "report a bug" button.
- The launcher does not sell or offer to buy a license — out of scope.

### 1.3. Boundaries: what is NOT in v1.0 (`🚫 Out`)

- Shared server browser. Instead — favorites + auto-inject into every instance (P8).
- Cloud sync (WetID/cloud) — future versions.
- Aleph Guard / anticheat (integrity checks, scanning, cheat countermeasures) — 2.0.
- Instance export/sharing with other users — not needed in v1.0.
  (But **import** of common formats IS in 1.0 — see P4.)
- Profile import from other launchers (TLauncher, MultiMC, etc.) — not required at all.
- Built-in skin editor — not in 1.0 (pick/upload only).
- Friends chat in the launcher — in-game chat only.
- Custom themes — most likely absent (decision not final, `❓ Open`).
- macOS — not a priority (baseline support, no guarantees).
- Separate stable/beta update channel — none.

## 2. Starting point: what is already done (audit of `1ba4cf1`)

### `amc-core` — `✅ Done` (foundation)
- `types.rs`: `LoaderType` (Vanilla/Fabric/Quilt/Forge/NeoForge/OptiFine), `ReleaseType`
  (Release/Snapshot/Beta/Alpha/Old), `Instance` (create/persist `instances.json`, `clone_instance`
  copying `config/mods/resourcepacks/shaderpacks`, `ensure_directories` incl. `screenshots`,
  `get_game_dir` + `custom_dir`), `LaunchOptions` (RAM min/max, `java_path`, JVM args, resolution,
  fullscreen, `quick_play_server/port`), basic stats (`total_played_minutes`, `last_played`).
- `paths.rs`: `LauncherPaths` (`instances/versions/assets/libraries/runtimes/cache/logs`,
  `config.json/accounts.json/instances.json`), `custom()` for drive change.
- `config.rs`: `UiSettings` (`aleph-dark` theme, scale, `close_after_launch`, version filters,
  language) + autosave from settings.
- `i18n.rs`: EN/RU/UK, live switching without restart, large type-safe dictionary.

### `amc-auth` — `🟡 Partial`
- `✅` Three types (`Microsoft`/`WetID`/`Offline`), `AccountManager` (multi-account,
  active, `accounts.json` persist), MS Device Code Flow (code request + polling),
  offline nickname + Offline-UUID, WetID login/password + token verify.
- `⬜` Gaps vs concept: WetID is currently a login/password form but must go **via browser**;
  no per-launch identity picker + "remember choice"; no second-WetID ban;
  no WetID+license combo logic (see P6).

### `amc-downloader` — `🟡 Partial`
- `✅` Parallel engine (16 workers, semaphore), ×3 retries, SHA-1, skip existing files,
  byte-level progress + tracker, Adoptium OpenJDK 8/17/21 (zip/tar.gz, top-level dir strip).
- `⬜` No pause/resume, speed limit, queue UI, per-file ETA (see P3).

### `amc-minecraft` — `🟡 Partial`
- `✅` Version manifest (fetch + cache), `VersionDetails` (fetch/merge), rules/OS-arch filter,
  assets (index + objects), `FabricLoader`/`QuiltLoader` (version + profile + merge),
  `ForgeLoader`/`NeoForgeLoader`/`OptiFineLoader` (**version lookup only**),
  `ArgumentBuilder` (JVM + game args), `MinecraftLauncher` (spawn, stdout/stderr capture,
  `GameEvent`), `ServerPinger` (SLP), crash diagnostics (OOM / Java / Fabric deps / Mixin).
- `⬜` The launch pipeline in `amc-ui/src/app.rs` currently completes only for
  **Fabric/Quilt/Vanilla**; the rest falls back to vanilla — full Forge/NeoForge/OptiFine
  pipeline still needed (P2/P4).

### `amc-mods` — `🟡 Partial`
- `✅` Modrinth API v2 (search + file), CurseForge (search + file), `LocalModManager`
  (`mods/` scan, enable/disable via `.disabled`, `fabric.mod.json`/`mods.toml`/`mcmod.info` parsing),
  Mod/ResourcePack/Shader categories routed to `mods/resourcepacks/shaderpacks`.
- `⬜` No pre-install compatibility check, no filters (category/popularity/MC version),
  no one-button updates/notifications/rollback/pin, no `.mrpack`/Curse `.zip` import, no Discover (P4).

### `amc-ui` — `🟡 Partial`
- `✅` Sidebar/Home/Instances/Mods/Skins/Settings, TitleBar, BottomBar with skin avatar,
  LoginModal (3 tabs), ConsoleModal (5000-line buffer, search, autoscroll, copy, `.log` export, highlighting),
  DownloadOverlay, Home (versions + search + filters + direct connect), Instances
  (create/edit/delete/clone/select/launch, RAM/loader/version), Mods (installed/search/install),
  Skins (2D Classic/Slim preview, PNG load via `rfd`, Steve/Alex reset, PNG export),
  Settings (General/Java&Memory/Accounts: language, behavior, resolution, folders, RAM sliders,
  Java picker + reset, JVM args + G1GC/ZGC presets, account list).
- `⬜` No first-run wizard, no Simple/Pro modes as separate screens, no instance templates/icons/folder-tags,
  no disk usage, no download manager, no screenshot gallery, no world backups,
  no server favorites, no profile tab (nickname/skin/cape), no 3D skin preview (P1–P10).

## 3. Matrix "concept → code"

| Concept item | Status | Location / note |
|---|---|---|
| Wizard 1: language choice, OS auto-detect | ✅ Done | Wizard + `detect_system()` ✅ |
| Wizard 2: Simple / Pro choice | ✅ Done | Wizard + free switching from settings ✅ |
| Wizard 3: sign-in (MS/WetID/no account), WetID via browser | 🟡 Partial | `login.rs`, `microsoft.rs`, `wetid.rs`; browser WetID — P6 |
| Hardware test + lightweight mode (non-blocking, independent of Simple/Pro) | ⬜ Todo | P1 |
| Simple: versions incl. snapshot/pre, quick start | 🟡 Partial | Home + filters exist; "quick start" as one button — P2 |
| Simple: per-mod download/search, loader hint | 🟡 Partial | Search exists; loader hint + compat — P4 |
| Simple: one-button pack update, version porting | ⬜ Todo | P4 |
| Pro: instances, templates, icon+name, clone, folders/tags | 🟡 Partial | Clone ✅ (`clone_instance`); templates/icons/folder-tags — P2 |
| Different home screens for Simple/Pro | ⬜ Todo | P1 |
| Drag-and-drop mod/pack into window | ⬜ Todo | P4 |
| Offline play after first launch | 🟡 Partial | Needs cache audit (manifest/assets/libs); P11 |
| All loaders + auto-detect + compat + filters | 🟡 Partial | Lookup exists; full install/launch + UI — P2/P4 |
| Single release channel | ✅ Done | Only `dev`/`release` profiles |
| Sources: only Modrinth/CurseForge + manual file | ✅ Done | Policy holds; manual file = `mods/` folder + dialog |
| Storage location choice (another drive) | 🟡 Partial | `LauncherPaths::custom` + `custom_game_dir` exist; selection UI — P2 |
| Self-contained instances, no dedup | ✅ Done | Confirmed by architecture |
| In-launcher console/logs | ✅ Done | `console.rs` + `launch.rs` |
| Disk usage per instance/folder | ✅ Done | Card + pre-download free-space check ✅ |
| Hybrid Java: auto + manual + system/other-launcher discovery | 🟡 Partial | Auto+manual ✅; system discovery — P5 |
| Server browser | 🚫 Out | Not built; favorites + inject — P8 (`servers.dat` inject at launch ✅, favorites UI — no) |
| Windows + Linux | 🟡 Partial | Primary Win, Linux X11/Wayland via `eframe`; Win7/GT710 — verify in P11 |
| Resource packs/shaders from launcher | 🟡 Partial | Folder routing ✅; search UI — P10 |
| Launcher self-update | ⬜ Todo | P11 |
| Sodium/Lithium suggestion on weak hardware | ⬜ Todo | P1 (with hardware test) + P4 |
| One-button mod/pack updates + rollback | ⬜ Todo | P4 |
| Discover inside instances tab | ⬜ Todo | P4 |
| Download manager (pause/limit/queue/retry/progress/ETA/parallel) | 🟡 Partial | Parallel+retry ✅; rest — P3 |
| Installer + portable | ⬜ Todo | P11 |
| Unlicensed nicknames resolved by server | ✅ Done | Policy fixed |
| Import `.mrpack`/Curse `.zip`, version pinning | ⬜ Todo | P4 (import in 1.0; outward sharing — `🚫 Out`) |
| Import from other launchers | 🚫 Out | Not built |
| RU/EN/UK | ✅ Done | `i18n.rs` |
| Dark theme only; custom — `❓ Open` | ✅/❓ | Dark ✅; custom awaits concept decision |
| Play statistics (hours, per instance) | 🟡 Partial | Minutes + last played ✅; extend — P10 |
| RAM/JVM in both modes, auto-RAM, game args | 🟡 Partial | Manual ✅ + presets; hardware-based auto-RAM — P1/P2 |
| Steam co-op for all account types, no chat | ⬜ Todo | P13 |
| Cloud | 🚫 Out | — |
| News feed | ❓ Open | Undecided; plan default: not in 1.0 without a "yes" |
| Screenshot gallery | 🟡 Partial | `ScreensModal` gallery + 🖼 titlebar button ✅; delete/share — no (P10) |
| Multi-MS ✅, single WetID | 🟡 Partial | Second-WetID ban — P6 |
| Anonymous auto crash reports + conflicts only in logs | ⬜/✅ | Non-interference ✅; auto reports — P12 (opt-out) |
| World backup (manual + scheduled) | ⬜ Todo | P9 |
| Guard/anticheat | 🚫 Out | 2.0 |
| Skins: free selection/upload, NameMC validation + auto command, launcher fallback | 🟡 Partial | Upload+2D ✅; rest — P7 |
| Account combo logic + per-launch picker + "remember" | ⬜ Todo | P6 |
| Profile (nickname/skin/cape) | 🟡 Partial | Summary tab + navigation (`NavTab::Profile`) ✅; NameMC auto-command/cape — no (P7) |
| After launch: minimize/keep; parallel launches incl. same instance ×N | 🟡 Partial | `close_after_launch` bool ✅; 3 states + parallel — P2 |

### Folded back from the working copy (returned to the concept on 2026-09-30)

| Concept item | Status | Location / note |
|---|---|---|
| Newcomer tutorial tour; .minecraft reuse instead of re-download | ⬜ Todo | P1 |
| Quick start = last played version; single setup in Simple; classic versions | ⬜ Todo | P1/P2 |
| AMC-only templates; clone with worlds/mods choice; folders/tags/sort/pin/search; notes; pre/post scripts; Java/RAM groups | ⬜ Todo | P2 |
| Instance card (glanceable content); disk usage | ✅ Done | Card grid + usage ✅ |
| Ratings in mod search | ⬜ Todo | P4 (fields already in the APIs) |
| World import by dragging its folder | ⬜ Todo | P4 |
| OptiFine as a regular mod (temporarily a loader in code, P2) | 🟡 Partial | Move to mod-style install — P4 |
| Unsafe-loader warning | ⬜ Todo | P2/P4 |
| Wishlist; batch actions; auto-dependencies; incompatibility warning; "often installed together" | ⬜ Todo | P4 |
| CLI mode (automation, servers) | ⬜ Todo | New phase P14 |
| Local-mod instance badge; "no antivirus scan" policy | ⬜/✅ | Badge — P2; policy fixed |
| Instance shortcuts with direct launch; GPU choice; per-instance resolution; instance icon in taskbar | ⬜ Todo | P11/P14 (shortcuts need CLI) |
| Config editor (Pro) | ⬜ Todo | P2 |
| Free-space check before downloading | ✅ Done | Warning with numbers ✅ |
| Java manager screen (add/remove) | ✅ Done | System discovery + runtime list ✅ |
| Server-required mods (window + auto-install) | ⬜ Todo | P8 |
| OS autostart | ⬜ Todo | P11 |
| .NET/VC++ check in the installer | ⬜ Todo | P11 |
| Resource pack previews + resolution filter | ⬜ Todo | P10 |
| Launcher updates: at startup only, changelog, signature, portable settings, Linux formats, trusted mode | 🟡 Partial | Startup check + button + toast/changelog ✅; signature/rollback/trusted mode — no (P11) |
| Process priority + FPS overlay | ⬜ Todo | P2/P11 |
| Pack rollback + autobackup + changelog; version pinning | ⬜ Todo | P4 |
| Settings export/import via file | ⬜ Todo | P10 |
| Crash reports opt-out (concept requires opt-out, not opt-in) | 🟡 Partial | Local archive + 1-click diag-`.zip` export ✅; auto-send — no, needs an endpoint (P12) |
| Verify/repair files + health score | ⬜ Todo | P2 |
| Deletion trash; uninstall data question; instance backup to disk | ⬜ Todo | P2/P9/P11 |
| Scheduled world backups + folder choice | ⬜ Todo | P9 |
| WetID 2FA; per-launch identity picker + "remember"; default account per instance | ⬜ Todo | P6 |
| Post-exit session summary | ✅ Done | Modal on clean exit ✅ |
| Hotkeys + gamepad; per-type notifications | ⬜ Todo | P10 |
| Local server button (tunneling — ❓ open) | 🟡 Partial | Start/stop + EULA checkbox + server.jar/Java resolving ✅; tunneling — ❓ open (P13) |

## 4. Work phases

### P0 — Foundation: docs, board, CI (this commit)
- [x] `docs/CONCEPT.ru.md` + `docs/CONCEPT.md` (mirrored living document).
- [x] `docs/ROADMAP-1.0.ru.md` + `docs/ROADMAP-1.0.md` (this file + mirror).
- [ ] Remove root `AMC_Launcher_Concept.md` (moved to `docs/`), leave a redirect note in `README*`.
- [ ] Link `README.md` / `README.ru.md` to `docs/`; short "Concept & Roadmap" section.
- [ ] File GitHub Issues for phases P1–P13 with labels `1.0`, `concept`, `open-question`.
- [ ] CI already exists (`.github/workflows`) — add `cargo fmt --check`, `cargo clippy`, `cargo test --workspace`.
- Acceptance: `docs/` on `main`, README links, issues filed, green CI.

### P1 — First launch: wizard, modes, hardware
- [ ] Wizard: language (OS auto) → Simple/Pro → account (browser WetID — joint with P6).
- [ ] Two different home screens (Simple: versions + quick start; Pro: instances).
- [ ] Free Simple/Pro switching from settings (+ persist in `config.json`).
- [ ] Hardware test (CPU/GPU/RAM/disk, GT710 baseline): non-blocking, suggests a lightweight preset;
  does not affect mode choice. Links: default auto-RAM + Sodium/Lithium suggestion.
- [ ] System-language auto-detect on first start (current default is `ru`).
- Acceptance: a clean profile completes the wizard in <1 min; mode switches without losing instances;
  weak hardware gets a preset but is never blocked.

### P2 — Pro instances + launch behavior
- [ ] Templates (vanilla+, optimized, empty), icon + name, folders/tags, instance search (search exists — extend).
- [ ] Disk usage per instance/folder; root selection UI (`custom_game_dir` already in config — finish the UI).
- [ ] Vanilla "quick start"; snapshots/pre-releases in Simple (flags already exist).
- [ ] Full Forge/NeoForge/OptiFine install/launch pipeline (currently vanilla fallback).
- [ ] After-launch: 3 states (close / minimize to lite / keep) instead of bool.
- [ ] Parallel launches, including the same instance ×N (isolation via `--workDir`/lock files, per-process stats).
- [ ] Custom game args + JVM args in both modes (JVM done; game args — add).
- Acceptance: 3+ instances in parallel; same instance ×2 without save corruption; disk usage visible; root movable.

### P3 — Download manager
- [ ] Pause/resume (Range), speed limit, queue, auto-retry (partly exists), per-file + total progress/ETA.
- [ ] Dedicated manager UI (currently only `DownloadOverlay`); keep CPU/disk budget with parallelism.
- Acceptance: dropped connection resumes via auto-retry; pause/limit work; ETA is sane.

### P4 — Mods and packs (core value)
- [ ] Pre-install compatibility (MC version + loader) + filters (category/popularity/version).
- [ ] One-button update (mod and whole pack) + notifications; pack rollback; version pinning.
- [ ] Pack porting to another MC version.
- [ ] Drag-and-drop mod/pack file into the window (`egui` file-drop).
- [ ] `.mrpack` + Curse `.zip` import (outward export — `🚫 Out` in 1.0).
- [ ] Trending Discover inside the instances tab (not the home screen).
- Acceptance: installing an incompatible mod warns; update/rollback in 1 click; `.mrpack` imports.

### P5 — Hybrid Java
- [ ] System Java discovery + known other-launcher locations (no profile import) + "use it" offer.
- [ ] Auto version for the game (8/17/21 done) + manual override (done) — keep.
- Acceptance: with a system Java present, the launcher offers it instead of silently downloading.

### P6 — Concept-correct accounts
- [ ] WetID via external browser page (currently login/password form — replace), sign-in/registration.
- [ ] Per-launch identity picker (license/WetID/nickname) + "remember choice".
- [ ] Strictly one WetID per profile (second blocked with a clear error; family exception — `❓ Open`).
- [ ] WetID+license combo; multi-MS already — keep.
- Acceptance: the "Accounts" concept scenarios pass manually.

### P7 — Skins and profile
- [ ] Free skin selection + own upload, no editor.
- [ ] NameMC validation + auto-command (`/skins <name>` and analogues) sent from the client; hack pack for other plugins.
- [ ] Launcher fallback (visible only to AMC users) — document limits honestly.
- [ ] Profile tab: nickname + skin + cape. 3D preview — optional in 1.0 (already a README todo; decide: 2D suffices for 1.0, 3D if cheap).
- Acceptance: a selection skin appears on a skin-plugin server with no manual commands.

### P8 — Servers without a browser
- [ ] Favorites/quick access (direct connect exists — persist it).
- [ ] Auto-inject favorites into every instance's `servers.dat`.
- Acceptance: add a server once — it exists in all instances.

### P9 — Worlds (saves)
- [ ] Backup/restore from the launcher, manual + scheduled.
- Acceptance: button backup and scheduled backup; 1-click restore.

### P10 — Stats, screenshots, resource packs
- [ ] Hours/sessions per instance (base exists) + summary; `screenshots/` gallery; resource/shader packs as full search (not just install).
- Acceptance: playtime visible per instance; screenshots open from the launcher.

### P11 — Distribution and platform
- [ ] Installer (Win) + portable; launcher self-update.
- [ ] Win10/11 + Linux (X11/Wayland); Win7 — verify and state honestly (concept requires it, `eframe/wgpu` may object — either support or amend the concept).
- [ ] GT710 perf budget: startup/RAM/FPS measurement, regression check.
- [ ] Offline audit: after first launch the game starts with no network (manifest/asset/lib/Java caches).
- [ ] Cold boot <300 ms, idle ~35 MB — confirm by measurement.
- Acceptance: both formats install; self-update works; offline launch verified.

### P12 — Diagnostics and data
- [ ] Anonymous crash reports: automatic per the concept, but with explicit opt-out in wizard/settings (zero telemetry holds: network only under consent, see 1.2).
- [ ] Conflicts stay log-only, no auto-fix (already so — do not break).
- Acceptance: reports go out only with consent; crash diagnostics cover OOM/Java/deps.

### P13 — Steam co-op
- [ ] Steam invites + co-op as a built-in feature (not a separate mod), all account types, no launcher chat.
- Acceptance: two players with different account types play via invite with no manual port forwarding.

### P14 — CLI and direct launches (from the working copy)
- [ ] Headless mode (`--launch <instance>`, `--help`, exit codes) on the same pipeline as the GUI.
- [ ] Instance shortcuts (Desktop/Start) via CLI — launch with no launcher window.
- Acceptance: a shortcut launches an instance with no GUI; `cli --help` runs in CI.

## 5. Open questions (`❓ Open` — await concept updates)

1. WetID status in launcher (friends, trust, etc.) — show or not?
2. Home news feed — needed or not? (plan default: no, until the concept says yes).
3. Custom themes — default: not in 1.0.
4. Licensing/distribution model (concept undecided; code is currently GPL-3.0 — record the decision gap).
5. Import/export format (goal: `.mrpack`, Curse `.zip`, etc. — pin the choice).
6. Shared server browser — default: not needed at all.
7. 3D skin preview in 1.0 — required or is 2D enough?
8. Win7 — support literally or amend the concept to Win10+?
9. Family WetIDs on one device — really no exceptions?
10. Custom URL protocol (amc://) for installing from sites — needed or not?
11. Donation platform(s) — not chosen.
12. Donor profile badge — yes or no?
13. What WetID registration requires — undefined.

## 6. Definition of Done for v1.0 (release checklist)

- [ ] Wizard + Simple/Pro + hardware test work from scratch (P1).
- [ ] Instances: templates, icons, folders/tags, disk usage, ×N parallel (P2).
- [ ] Download manager: pause/limit/queue/ETA (P3).
- [ ] Mods: compat, filters, update+rollback+pin, DnD, `.mrpack`/Curse import, Discover (P4).
- [ ] Hybrid Java incl. system (P5).
- [ ] Strictly concept-correct accounts incl. browser WetID (P6).
- [ ] Skins + profile (P7), server favorites with inject (P8), world backups (P9).
- [ ] Stats, screenshots, resource/shader packs (P10).
- [ ] Installer + portable + self-update + offline launch + perf budget (P11).
- [ ] Opt-out crash reports (P12), Steam co-op (P13), CLI + shortcuts (P14).
- [ ] `cargo test --workspace` + `clippy` + `fmt` green; manual checklist over section 3 matrix.
- [ ] `docs/` updated; open questions either resolved or explicitly moved to 2.0.

## 7. Update process (the concept keeps changing)

1. Product changes start with `docs/CONCEPT.ru.md` (+ mirror `docs/CONCEPT.md`).
2. Then this plan is updated (`ROADMAP-1.0.ru.md` + `ROADMAP-1.0.md`): statuses, phases, open questions.
3. Each phase gets GitHub Issue(s) with the same `P<n>` number; closing an issue = phase acceptance.
4. Unclear points are never silently invented: they are marked `❓ Open` here and in the concept until decided.
5. Behavior-changing commits: `feat(...)`/`fix(...)`; docs: `docs(...)`; mix code and docs in one commit only if the change is atomic (like this one).
