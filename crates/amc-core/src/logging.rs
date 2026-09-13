use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(log_dir: Option<&Path>) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,amc_launcher=debug,amc_core=debug,amc_auth=debug,amc_downloader=debug,amc_minecraft=debug,amc_mods=debug,amc_ui=debug"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_names(true);

    if let Some(dir) = log_dir {
        let _ = std::fs::create_dir_all(dir);
        let log_file_path = dir.join("launcher.log");
        if let Ok(file) = OpenOptions::new().create(true).write(true).append(true).open(&log_file_path) {
            let file_writer = Mutex::new(file);
            let file_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_thread_names(true)
                .with_writer(file_writer);

            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .try_init();
            return;
        }
    }

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .try_init();
}
