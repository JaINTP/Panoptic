use tauri::Manager;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let log_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create log dir: {}", e))?;
    }

    let file_appender = tracing_appender::rolling::never(log_dir.clone(), "error.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    Box::leak(Box::new(guard));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let console_layer = fmt::layer().with_target(true).with_ansi(true);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Logging initialized. Logs are saved to: {:?}", log_dir);
    Ok(())
}
