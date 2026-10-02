#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use amc_core::init_logging;
use amc_core::paths::LauncherPaths;
use amc_ui::{setup_fonts, LauncherApp};
use eframe::egui::vec2;
use eframe::NativeOptions;

fn app_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .ok()?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

/// Storage root resolution order (CONCEPT "Хранение на диске" + portable):
/// `portable.txt` next to the exe wins, then `AMC_ROOT`, then the saved
/// `root_override` from settings, then the OS default.
fn resolve_paths() -> LauncherPaths {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("portable.txt").is_file() {
                if let Ok(paths) = LauncherPaths::custom(dir.to_path_buf()) {
                    tracing::info!("Portable mode: storage at {}", dir.display());
                    return paths;
                }
            }
        }
    }
    if let Ok(root) = std::env::var("AMC_ROOT") {
        if let Ok(paths) = LauncherPaths::custom(root) {
            return paths;
        }
    }
    if let Ok(default) = LauncherPaths::default_paths() {
        if let Ok(cfg) = amc_core::LauncherConfig::load_from_path(&default.config_file()) {
            if let Some(root) = cfg.root_override {
                if let Ok(paths) = LauncherPaths::custom(root) {
                    return paths;
                }
            }
        }
        return default;
    }
    LauncherPaths::custom("./AlephLauncher").expect("unwritable fallback root")
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let paths = resolve_paths();
    let logs_dir = Some(paths.logs_dir());

    // Initialize structured logging to stdout and logs/launcher.log
    init_logging(logs_dir.as_deref());
    tracing::info!("Starting Aleph Minecraft Client Launcher (AMC Launcher)...");

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Aleph Launcher")
        .with_app_id("aleph_launcher")
        .with_inner_size(vec2(1120.0, 700.0))
        .with_min_inner_size(vec2(960.0, 600.0))
        .with_decorations(false)
        .with_transparent(false);

    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = NativeOptions {
        viewport,
        vsync: true,
        renderer: eframe::Renderer::Glow, // Fast hardware-accelerated OpenGL backend
        ..Default::default()
    };

    eframe::run_native(
        "Aleph Launcher",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            let mut app = LauncherApp::new(cc, paths);
            app.load_initial_manifest();
            // CONCEPT: launcher update check runs only at startup.
            app.check_launcher_update();
            Ok(Box::new(app))
        }),
    )
}
