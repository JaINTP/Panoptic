pub mod handlers;
pub mod router;

use crate::router::AppRouter;
use panoptic_core::{AppState, AuthState, PlaybackState};
use tokio::sync::watch;
use tracing::{error, info};

pub async fn start_server(
    state_rx: watch::Receiver<PlaybackState>,
    auth_tx: watch::Sender<AuthState>,
    css_version_rx: watch::Receiver<u32>,
    settings_path: Option<std::path::PathBuf>,
    plugins: std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>,
    app_handle: Option<tauri::AppHandle>,
) {
    let state = AppState {
        auth_tx,
        state_rx,
        css_version_rx,
        settings_path,
        app_handle,
    };
    let app = AppRouter::build(state, plugins);

    // Bind on both IPv4 and IPv6 loopback so that OAuth callbacks reach the
    // server regardless of how the OS resolves "localhost".  On Windows 10+
    // browsers typically resolve localhost to ::1 (IPv6) while a plain
    // 127.0.0.1 bind would silently refuse those connections, breaking the
    // Twitch auth redirect flow.
    let ipv4 = tokio::net::TcpListener::bind("127.0.0.1:3000").await;
    let ipv6 = tokio::net::TcpListener::bind("[::1]:3000").await;

    match (ipv4, ipv6) {
        (Ok(v4), Ok(v6)) => {
            info!("Axum server listening on http://127.0.0.1:3000 and http://[::1]:3000");
            let app_v6 = app.clone();
            tokio::spawn(async move {
                if let Err(e) = axum::serve(v6, app_v6).await {
                    error!("Axum IPv6 server error: {}", e);
                }
            });
            if let Err(e) = axum::serve(v4, app).await {
                error!("Axum IPv4 server error: {}", e);
            }
        }
        (Ok(v4), Err(_)) => {
            info!("Axum server listening on http://127.0.0.1:3000 (IPv6 unavailable)");
            if let Err(e) = axum::serve(v4, app).await {
                error!("Axum server error: {}", e);
            }
        }
        (Err(_), Ok(v6)) => {
            info!("Axum server listening on http://[::1]:3000 (IPv4 unavailable)");
            if let Err(e) = axum::serve(v6, app).await {
                error!("Axum server error: {}", e);
            }
        }
        (Err(e4), Err(e6)) => {
            error!(
                "Failed to bind Axum server on any address: IPv4={} IPv6={}",
                e4, e6
            );
        }
    }
}
