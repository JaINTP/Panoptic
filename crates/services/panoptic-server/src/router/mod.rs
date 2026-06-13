use crate::handlers::{auth::spotify_callback, health::HealthHandler, track::get_current_track};
use crate::AppState;
use axum::{routing::get, Router};

pub struct AppRouter;

impl AppRouter {
    pub fn build(state: AppState) -> Router {
        Router::new()
            .route("/health", get(HealthHandler::check))
            .route("/callback", get(spotify_callback))
            .route("/current-track", get(get_current_track))
            .with_state(state)
    }
}
