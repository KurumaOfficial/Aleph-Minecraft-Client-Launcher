#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::vec2;
use eframe::NativeOptions;
use amc_core::init_logging;
use amc_ui::{setup_fonts, LauncherApp};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize structured logging
    init_logging();
    tracing::info!("Starting Aleph Minecraft Client Launcher (AMC Launcher)...");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Aleph Launcher")
            .with_inner_size(vec2(1080.0, 680.0))
            .with_min_inner_size(vec2(940.0, 580.0))
            .with_decorations(false) // Frameless window with custom titlebar
            .with_transparent(false),
        renderer: eframe::Renderer::Wgpu,
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
