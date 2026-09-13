#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::vec2;
use eframe::NativeOptions;
use amc_core::paths::LauncherPaths;
use amc_core::init_logging;
use amc_ui::{setup_fonts, LauncherApp};

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

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let paths = LauncherPaths::default_paths().ok();
    let logs_dir = paths.as_ref().map(|p| p.logs_dir());

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
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            let mut app = LauncherApp::new(cc);
            app.load_initial_manifest();
            Ok(Box::new(app))
        }),
    )
}
