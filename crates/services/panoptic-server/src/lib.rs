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
) {
    let state = AppState {
        auth_tx,
        state_rx,
        css_version_rx,
        settings_path,
    };
    let app = AppRouter::build(state, plugins);
    let addr = "127.0.0.1:3000";
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind Axum server to {}: {}", addr, e);
            return;
        }
    };
    info!("Axum server listening on http://{}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        error!("Axum server error: {}", e);
    }
}
