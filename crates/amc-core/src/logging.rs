use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,amc_launcher=debug,amc_core=debug,amc_auth=debug,amc_downloader=debug,amc_minecraft=debug,amc_mods=debug,amc_ui=debug"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_names(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .try_init();
}
