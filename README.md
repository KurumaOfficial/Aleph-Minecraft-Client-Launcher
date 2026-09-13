<p align="center">
  <img src="assets/social_preview.png" alt="Aleph Minecraft Client Launcher" width="100%">
</p>

# ℵ Aleph Minecraft Client Launcher (AMC Launcher)

<div align="center">

**High-performance, modular, open-source Minecraft launcher built from scratch in Rust.**

[![Rust 2021](https://img.shields.io/badge/Rust-2021_Edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPL_v3-red.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-blue.svg?style=flat-square)](https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher)

[English](README.md) • [Русский](README.ru.md)

[About](#-about) • [Features](#-features) • [Workspace Architecture](#-workspace-architecture) • [Building from Source](#-building-from-source) • [Roadmap](#-roadmap)

---

</div>

## 📌 About

Most modern Minecraft launchers suffer from two extremes: they are either sluggish Electron apps sitting idle at 800+ MB of RAM, or bloated with shady adware, tracking, and closed-source telemetry.

**AMC Launcher** is built from the ground up as an uncompromising alternative:
- **Instant startup:** Interface launches in ~300 ms with an ultra-light footprint (~30 MB idle RAM).
- **Pure Rust:** Architected as a modular 6-crate Cargo Workspace with strict separation of concerns.
- **Zero telemetry:** Direct network communication only with official APIs: Mojang, Adoptium, Modrinth, and CurseForge. No tracking, no analytics, no third-party telemetry.
- **Aleph Studio aesthetic:** Custom dark ruby design (`#080606`, `#8B1A2A`), smooth framerate (60–144+ FPS via hardware OpenGL/wgpu), and native frameless window controls.

---

## ⚡ Features

### 🎮 Launch & Multi-Loader Support
- **Full version coverage:** From the latest Minecraft releases (1.21+) and snapshots down to classic Alpha and Beta.
- **First-class mod loaders:** Fabric, Quilt, Minecraft Forge, NeoForge, OptiFine, and clean Vanilla.
- **Automated Java management:** The launcher detects the required OpenJDK version (8, 17, or 21) for the selected game version, downloads it directly from the official Adoptium API, and unpacks it into an isolated `runtimes/` directory. No manual `JAVA_HOME` configuration needed.
- **High-throughput downloader (16 workers):** Parallel async downloads for client JARs, libraries, and assets with SHA-1 hash integrity validation. Relaunching an already downloaded version is near-instant (~100 ms).

### 🧩 Instance Isolation
- **Sandboxed profiles:** Each instance has its own isolated directory for mods, configs, resource packs, and save files.
- **In-place editing:** Adjust game version, loader, profile name, and RAM allocation directly on the instance card.
- **Playtime tracking:** Accurately tracks accumulated in-game time and timestamp of your last session.
- **One-click access:** Open any instance folder directly in your system file manager.

### 🌐 Mod Manager (Modrinth & CurseForge)
- **Dual-provider search:** Unified mod catalog querying both **Modrinth API v2** and **CurseForge (CurseTools API)**.
- **1-Click installation:** Direct `.jar` download directly into the active instance's `mods/` directory.
- **Local mod management:** View installed mods with native manifest parsing (`fabric.mod.json`, `mods.toml`). Toggle mods on or off without deleting them (via `.disabled` extension).

### 👤 Skins & Personalization
- **2D Skin Viewer:** Real-time composite preview rendering all outer and inner clothing layers (head, helmet, body, jacket, arms, sleeves, legs, pants).
- **Model switching:** Seamlessly toggle between Classic (4px) and Slim / Alex (3px) geometry.
- **Native file dialog:** Pick any local PNG skin using the OS native dialog (`rfd`).
- **Dynamic avatar:** Automatically crops the player's face from the active skin and renders it in the bottom status bar.

### 📟 Game Console & Diagnostics
- **Live stream capture:** Intercepts `stdout` and `stderr` streams from the running Minecraft process with a 5,000-line buffer.
- **Syntax color-coding:** Fatal errors and exceptions highlighted in ruby red, warnings in amber, debug logs in muted gray.
- **Built-in tools:** Real-time log search, autoscroll lock, clipboard copying, and single-click export to a `.log` file.

### 🌐 Multi-Language Localization
- **Instant runtime switching:** Seamless live toggling between **English**, **Русский** (Russian), and **Українська** (Ukrainian) directly in settings with zero launcher restart required.
- **Type-safe architecture:** Zero-cost compiled dictionary system in pure Rust ensuring complete coverage and consistent typography across all views, controls, and dialogs.

### 🚀 JVM GC Tuning Presets
- **Aikar's G1GC Preset:** Battle-tested Java Virtual Machine flags configured to eliminate chunk-loading micro-stutters and frame drops.
- **Generational ZGC Preset:** Ultra-low pause garbage collector with sub-millisecond pauses for modern Java 17+.

---

## 🏗 Workspace Architecture

The project is structured as a modular Cargo Workspace:

```
AMC_Launcher/
├── crates/
│   ├── amc-core/          # Instance models, version definitions, LauncherPaths, config store
│   ├── amc-auth/          # Auth schemes (Offline, Microsoft Device Code Flow, WetID)
│   ├── amc-downloader/    # Async download engine, SHA-1 verification, Adoptium OpenJDK
│   ├── amc-minecraft/     # Mojang API, JVM argument builder, process runner, SLP server ping
│   ├── amc-mods/          # Modrinth API v2, CurseForge API, local JAR manifest parser
│   └── amc-ui/            # GUI via eframe/egui: windows, custom widgets, Aleph theme
├── assets/                # Typography (Segoe UI, Unbounded, JetBrains Mono) and icons
└── src/
    └── main.rs            # Application entry point, viewport setup, OpenGL configuration
```

---

## 🛠 Building from Source

### Prerequisites
- **Rust Toolchain:** Version `1.80` or newer (`stable` recommended).
- **Operating System:** Windows 10/11 x64 (primary target), Linux (X11 / Wayland).

### Instructions

1. Clone the repository:
```bash
git clone https://github.com/KurumaOfficial/Aleph-Minecraft-Client-Launcher.git
cd Aleph-Minecraft-Client-Launcher
```

2. Run test suites across all crates:
```bash
cargo test --workspace
```

3. Run in development mode:
```bash
cargo run
```

4. Build an optimized release binary:
```bash
cargo build --release
```
The compiled binary will be located at: `target/release/amc-launcher.exe`.

---

## 🗺 Roadmap

- [x] Complete architectural rewrite as a modular Cargo Workspace (6 crates).
- [x] Automated OpenJDK 8, 17, 21 provisioning via Adoptium API.
- [x] Multi-loader support (Fabric, Quilt, Forge, NeoForge, OptiFine, Vanilla).
- [x] Dual-provider mod browser (Modrinth v2 + CurseForge).
- [x] 2D composite skin preview with dynamic status-bar avatar slicing.
- [x] Interactive game console with log export and live filtering.
- [x] Curated JVM garbage collection presets (Aikar's G1GC & Generational ZGC).
- [x] Full tri-lingual localization (English, Русский, Українська).
- [x] TCP Server List Ping (SLP) status and latency monitor.
- [ ] 3D skin model viewport.
- [ ] Modpack export/import support (`.mrpack` and CurseForge `.zip`).
- [ ] One-click update checks for installed mods.

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0-or-later)**. See the [LICENSE](LICENSE) file for details.
