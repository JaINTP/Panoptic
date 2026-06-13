pub mod handlers {
    pub mod auth;
    pub mod health;
    pub mod track;
}
pub mod router;

use crate::router::AppRouter;
use panoptic_core::{AuthState, PlaybackState};
use tokio::sync::watch;

#[derive(Clone)]
pub struct AppState {
    pub auth_tx: watch::Sender<AuthState>,
    pub state_rx: watch::Receiver<PlaybackState>,
}

pub async fn start_server(
    state_rx: watch::Receiver<PlaybackState>,
    auth_tx: watch::Sender<AuthState>,
) {
    let state = AppState { auth_tx, state_rx };
    let app = AppRouter::build(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
