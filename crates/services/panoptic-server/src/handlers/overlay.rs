use crate::AppState;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    css: Option<String>,
}

pub async fn get_overlay() -> impl IntoResponse {
    let html = include_str!("../overlay.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

pub async fn get_overlay_css(State(state): State<AppState>) -> impl IntoResponse {
    let default_css = include_str!("../../../../../examples/now-playing/now-playing-default.css");

    let css = if let Some(ref path) = state.settings_path {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                    settings
                        .css
                        .filter(|c| !c.is_empty())
                        .unwrap_or_else(|| default_css.to_string())
                } else {
                    default_css.to_string()
                }
            } else {
                default_css.to_string()
            }
        } else {
            default_css.to_string()
        }
    } else {
        default_css.to_string()
    };

    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], css)
}

#[cfg(test)]
mod tests {
    use super::*;
    use panoptic_core::{AuthState, PlaybackState};
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_get_overlay() {
        let response = get_overlay().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("<!DOCTYPE html>"));
        assert!(body_str.contains("panoptic-overlay-wrapper"));
    }

    #[tokio::test]
    async fn test_get_overlay_css_default() {
        let (auth_tx, _auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let state = AppState {
            auth_tx,
            state_rx,
            settings_path: None,
        };

        let response = get_overlay_css(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "text/css; charset=utf-8"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("--panoptic-overlay-card-background"));
    }

    #[tokio::test]
    async fn test_get_overlay_css_custom() {
        let (auth_tx, _auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let temp_dir = std::path::PathBuf::from(manifest_dir)
            .join("target")
            .join("tmp_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let settings_file = temp_dir.join("settings.json");

        let settings_content = r#"{"css": ".my-custom-class { color: green; }"}"#;
        std::fs::write(&settings_file, settings_content).unwrap();

        let state = AppState {
            auth_tx,
            state_rx,
            settings_path: Some(settings_file.clone()),
        };

        let response = get_overlay_css(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "text/css; charset=utf-8"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains(".my-custom-class { color: green; }"));

        // Clean up the temporary file
        let _ = std::fs::remove_file(settings_file);
    }
}
