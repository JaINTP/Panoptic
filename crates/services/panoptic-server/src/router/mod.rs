use crate::handlers::{
    art::get_art,
    auth::auth_callback,
    health::HealthHandler,
    overlay::{get_active_effects, get_overlay, get_overlay_css, get_overlay_version},
    track::{get_current_track, get_playback},
};
use axum::{routing::get, Router};
use panoptic_core::AppState;
use tower_http::cors::{Any, CorsLayer};

pub struct AppRouter;

impl AppRouter {
    pub fn build(
        state: AppState,
        plugins: std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>,
    ) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let mut router = Router::new()
            .route("/health", get(HealthHandler::check))
            .route("/art", get(get_art))
            .route("/callback/:provider", get(auth_callback))
            .route("/current-track", get(get_current_track))
            .route("/playback", get(get_playback))
            .route("/overlay/now-playing", get(get_overlay))
            .route("/overlay/version", get(get_overlay_version))
            .route("/overlay/effects", get(get_active_effects))
            .route("/overlay/:id/style.css", get(get_overlay_css));

        // Let each plugin register its own routes
        for plugin in plugins.iter() {
            router = plugin.register_routes(router);
        }

        router.layer(cors).with_state(state)
    }
}
