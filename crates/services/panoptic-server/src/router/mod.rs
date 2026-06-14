use crate::handlers::{
    auth::spotify_callback,
    health::HealthHandler,
    overlay::{get_overlay, get_overlay_css},
    track::{get_current_track, get_playback},
};
use crate::AppState;
use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

pub struct AppRouter;

impl AppRouter {
    pub fn build(state: AppState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/health", get(HealthHandler::check))
            .route("/callback", get(spotify_callback))
            .route("/current-track", get(get_current_track))
            .route("/playback", get(get_playback))
            .route("/overlay/now-playing", get(get_overlay))
            .route("/overlay/now-playing/style.css", get(get_overlay_css))
            .layer(cors)
            .with_state(state)
    }
}
